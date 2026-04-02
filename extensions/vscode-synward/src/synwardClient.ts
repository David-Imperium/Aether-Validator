/**
 * Synward Client - Communication with Synward CLI/MCP
 */

import * as vscode from 'vscode';
import { exec, spawn, ChildProcess } from 'child_process';
import * as path from 'path';

export interface ValidationResult {
    passed: boolean;
    violations: Violation[];
    qualityScore: number;
    language: string;
}

export interface Violation {
    id: string;
    rule: string;
    message: string;
    line: number;
    column: number;
    severity: 'error' | 'warning' | 'info';
    tier?: 'inviolable' | 'strict' | 'flexible';
    confidence?: number;
    complianceAction?: string;
}

export interface ComplianceStatus {
    total_exemptions: number;
    learned_patterns: number;
    user_created: number;
    config: {
        auto_accept_threshold: number;
        ask_threshold: number;
        learn_after_occurrences: number;
    };
}

export interface DriftResult {
    path: string;
    drift_score: number;
    trend: string;
    metrics: {
        type_strictness: number;
        naming_consistency: number;
        error_handling_quality: number;
        complexity_avg: number;
    };
    alerts: Array<{
        alert_type: string;
        severity: string;
        message: string;
    }>;
    recommendation: string;
}

export class SynwardClient {
    private synwardPath: string;
    private config: vscode.WorkspaceConfiguration;
    
    constructor() {
        this.config = vscode.workspace.getConfiguration('synward');
        this.synwardPath = this.config.get<string>('synwardPath') || 'synward';
    }
    
    updateConfig(): void {
        this.config = vscode.workspace.getConfiguration('synward');
        this.synwardPath = this.config.get<string>('synwardPath') || 'synward';
    }
    
    async validate(
        filePath: string,
        language: string,
        _code: string
    ): Promise<ValidationResult> {
        const mode = this.config.get<string>('validationMode') || 'balanced';
        const dubbioso = this.config.get<boolean>('dubbiosoMode') ?? true;
        
        return new Promise((resolve, reject) => {
            const args = [
                'validate',
                filePath,
                '--lang', language,
                '--format', 'json'
            ];
            
            if (dubbioso) {
                args.push('--dubbioso');
            }
            
            if (mode === 'strict') {
                args.push('--severity', 'error');
            } else if (mode === 'lenient') {
                args.push('--severity', 'warning');
            }
            
            const process = exec(
                `"${this.synwardPath}" ${args.join(' ')}`,
                { maxBuffer: 10 * 1024 * 1024 },
                (error, stdout, stderr) => {
                    if (error && !stdout) {
                        // Only reject if we have no output (might be validation failures)
                        if (stderr.includes('error:')) {
                            reject(new Error(stderr));
                            return;
                        }
                    }
                    
                    try {
                        const json = JSON.parse(stdout);
                        resolve(this.parseValidationResult(json));
                    } catch (e) {
                        // If JSON parsing fails, try to extract from text output
                        resolve(this.parseTextOutput(stdout));
                    }
                }
            );
        });
    }
    
    private parseValidationResult(json: any): ValidationResult {
        const violations: Violation[] = (json.violations || []).map((v: any) => ({
            id: v.id,
            rule: v.rule,
            message: v.message,
            line: v.line || 0,
            column: v.column || 0,
            severity: this.mapSeverity(v.severity),
            tier: v.compliance?.tier,
            confidence: v.compliance?.confidence,
            complianceAction: v.compliance?.action,
        }));
        
        return {
            passed: json.passed ?? true,
            violations,
            qualityScore: json.quality_score ?? 100,
            language: json.language ?? 'unknown',
        };
    }
    
    private parseTextOutput(text: string): ValidationResult {
        // Fallback parser for non-JSON output
        const violations: Violation[] = [];
        const lines = text.split('\n');
        
        for (const line of lines) {
            // Match patterns like: ⚠ RULE001 - Message (line 42)
            const match = line.match(/[⚠✗] ([A-Z]+-\d+|\w+) - (.+?) \(line (\d+)\)/);
            if (match) {
                violations.push({
                    id: match[1],
                    rule: match[1],
                    message: match[2],
                    line: parseInt(match[3]),
                    column: 0,
                    severity: line.includes('✗') ? 'error' : 'warning',
                });
            }
        }
        
        return {
            passed: violations.filter(v => v.severity === 'error').length === 0,
            violations,
            qualityScore: Math.max(0, 100 - violations.length * 5),
            language: 'unknown',
        };
    }
    
    private mapSeverity(sev: string): 'error' | 'warning' | 'info' {
        switch (sev?.toLowerCase()) {
            case 'critical':
            case 'error':
                return 'error';
            case 'warning':
            case 'warn':
                return 'warning';
            default:
                return 'info';
        }
    }
    
    async getComplianceStatus(): Promise<ComplianceStatus> {
        return new Promise((resolve, reject) => {
            exec(
                `"${this.synwardPath}" compliance status --json`,
                { maxBuffer: 1024 * 1024 },
                (error, stdout, _stderr) => {
                    if (error) {
                        // Return default if command fails
                        resolve({
                            total_exemptions: 0,
                            learned_patterns: 0,
                            user_created: 0,
                            config: {
                                auto_accept_threshold: 0.90,
                                ask_threshold: 0.60,
                                learn_after_occurrences: 3,
                            }
                        });
                        return;
                    }
                    
                    try {
                        resolve(JSON.parse(stdout));
                    } catch {
                        resolve({
                            total_exemptions: 0,
                            learned_patterns: 0,
                            user_created: 0,
                            config: {
                                auto_accept_threshold: 0.90,
                                ask_threshold: 0.60,
                                learn_after_occurrences: 3,
                            }
                        });
                    }
                }
            );
        });
    }
    
    async acceptViolation(
        ruleId: string,
        filePath: string,
        reason: string
    ): Promise<void> {
        return new Promise((resolve, reject) => {
            exec(
                `"${this.synwardPath}" compliance accept "${ruleId}" "${filePath}" --reason "${reason}"`,
                (error, _stdout, stderr) => {
                    if (error) {
                        reject(new Error(stderr || error.message));
                    } else {
                        resolve();
                    }
                }
            );
        });
    }
    
    async analyzeDrift(filePath: string, days: number = 30): Promise<DriftResult> {
        return new Promise((resolve, reject) => {
            exec(
                `"${this.synwardPath}" drift analyze "${filePath}" --days ${days} --json`,
                { maxBuffer: 1024 * 1024 },
                (error, stdout, _stderr) => {
                    if (error) {
                        // Return default if command fails
                        resolve({
                            path: filePath,
                            drift_score: 0,
                            trend: 'unknown',
                            metrics: {
                                type_strictness: 1,
                                naming_consistency: 1,
                                error_handling_quality: 1,
                                complexity_avg: 0.5,
                            },
                            alerts: [],
                            recommendation: 'Unable to analyze drift.',
                        });
                        return;
                    }
                    
                    try {
                        resolve(JSON.parse(stdout));
                    } catch {
                        resolve({
                            path: filePath,
                            drift_score: 0,
                            trend: 'unknown',
                            metrics: {
                                type_strictness: 1,
                                naming_consistency: 1,
                                error_handling_quality: 1,
                                complexity_avg: 0.5,
                            },
                            alerts: [],
                            recommendation: 'Unable to parse drift analysis.',
                        });
                    }
                }
            );
        });
    }
    
    async suggestFix(
        violationId: string,
        code: string,
        language: string
    ): Promise<string | null> {
        return new Promise((resolve, reject) => {
            const process = spawn(this.synwardPath, [
                'suggest',
                '--violation', violationId,
                '--lang', language,
                '--format', 'json'
            ]);
            
            let stdout = '';
            let stderr = '';
            
            process.stdin?.write(code);
            process.stdin?.end();
            
            process.stdout?.on('data', (data) => { stdout += data; });
            process.stderr?.on('data', (data) => { stderr += data; });
            
            process.on('close', (code) => {
                if (code === 0 && stdout) {
                    try {
                        const json = JSON.parse(stdout);
                        resolve(json.suggestion || json.fix || null);
                    } catch {
                        resolve(stdout || null);
                    }
                } else {
                    resolve(null);
                }
            });
            
            process.on('error', () => resolve(null));
        });
    }
    
    dispose(): void {
        // Cleanup if needed
    }
}
