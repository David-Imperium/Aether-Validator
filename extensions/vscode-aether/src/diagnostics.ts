/**
 * Aether Diagnostics - VS Code diagnostic collection
 */

import * as vscode from 'vscode';
import { Violation } from './aetherClient';

export class AetherDiagnostics {
    private collection: vscode.DiagnosticCollection;
    
    constructor() {
        this.collection = vscode.languages.createDiagnosticCollection('aether');
    }
    
    updateDiagnostics(uri: vscode.Uri, violations: Violation[]): void {
        const diagnostics: vscode.Diagnostic[] = violations.map(v => {
            const range = new vscode.Range(
                new vscode.Position(Math.max(0, v.line - 1), v.column),
                new vscode.Position(Math.max(0, v.line - 1), v.column + 100)
            );
            
            const severity = this.mapSeverity(v.severity);
            
            const diagnostic = new vscode.Diagnostic(
                range,
                `[${v.id}] ${v.message}`,
                severity
            );
            
            diagnostic.source = 'Aether';
            diagnostic.code = v.id;
            
            // Add related information
            if (v.tier) {
                diagnostic.message += ` [${v.tier.toUpperCase()}]`;
            }
            
            if (v.confidence !== undefined) {
                diagnostic.message += ` (confidence: ${(v.confidence * 100).toFixed(0)}%)`;
            }
            
            // Add tags based on violation type
            const tags: vscode.DiagnosticTag[] = [];
            if (v.message.toLowerCase().includes('unused')) {
                tags.push(vscode.DiagnosticTag.Unnecessary);
            }
            if (v.message.toLowerCase().includes('deprecated')) {
                tags.push(vscode.DiagnosticTag.Deprecated);
            }
            if (tags.length > 0) {
                diagnostic.tags = tags;
            }
            
            return diagnostic;
        });
        
        this.collection.set(uri, diagnostics);
    }
    
    clearDiagnostics(uri: vscode.Uri): void {
        this.collection.set(uri, []);
    }
    
    clearAll(): void {
        this.collection.clear();
    }
    
    private mapSeverity(sev: string): vscode.DiagnosticSeverity {
        switch (sev) {
            case 'error':
                return vscode.DiagnosticSeverity.Error;
            case 'warning':
                return vscode.DiagnosticSeverity.Warning;
            default:
                return vscode.DiagnosticSeverity.Information;
        }
    }
    
    dispose(): void {
        this.collection.dispose();
    }
}
