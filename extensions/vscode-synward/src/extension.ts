/**
 * Synward VS Code Extension
 * 
 * Intelligent code validation with AI-powered contract enforcement.
 */

import * as vscode from 'vscode';
import { SynwardClient } from './synwardClient';
import { SynwardDiagnostics } from './diagnostics';
import { SynwardCodeActions } from './codeActions';
import { StatusBarManager } from './statusBar';
import { SynwardPanel } from './panel';

let client: SynwardClient;
let diagnostics: SynwardDiagnostics;
let statusBar: StatusBarManager;

export async function activate(context: vscode.ExtensionContext) {
    console.log('Synward extension is activating...');

    // Initialize client
    client = new SynwardClient();
    
    // Initialize diagnostics
    diagnostics = new SynwardDiagnostics();
    
    // Initialize status bar
    statusBar = new StatusBarManager();
    
    // Register code actions provider
    const codeActions = new SynwardCodeActions(client, diagnostics);
    
    // Register commands
    registerCommands(context, client, diagnostics, statusBar);
    
    // Register code actions for supported languages
    const supportedLanguages = [
        'rust', 'python', 'javascript', 'typescript', 'go', 'c', 'cpp',
        'java', 'ruby', 'php', 'swift', 'kotlin'
    ];
    
    for (const lang of supportedLanguages) {
        context.subscriptions.push(
            vscode.languages.registerCodeActionsProvider(
                { language: lang, scheme: 'file' },
                codeActions,
                { providedCodeActionKinds: SynwardCodeActions.providedKinds }
            )
        );
    }
    
    // Watch for configuration changes
    context.subscriptions.push(
        vscode.workspace.onDidChangeConfiguration(e => {
            if (e.affectsConfiguration('synward')) {
                client.updateConfig();
            }
        })
    );
    
    // Auto-validate on save if enabled
    const config = vscode.workspace.getConfiguration('synward');
    if (config.get<boolean>('enableValidation')) {
        context.subscriptions.push(
            vscode.workspace.onDidSaveTextDocument(doc => {
                if (supportedLanguages.includes(doc.languageId)) {
                    validateDocument(doc, client, diagnostics, statusBar, context);
                }
            })
        );
    }
    
    // Validate active editor on activation
    if (vscode.window.activeTextEditor) {
        validateDocument(
            vscode.window.activeTextEditor.document,
            client,
            diagnostics,
            statusBar,
            context
        );
    }
    
    // Watch for editor changes
    context.subscriptions.push(
        vscode.window.onDidChangeActiveTextEditor(editor => {
            if (editor && supportedLanguages.includes(editor.document.languageId)) {
                validateDocument(editor.document, client, diagnostics, statusBar, context);
            }
        })
    );
    
    console.log('Synward extension activated successfully');
}

function registerCommands(
    context: vscode.ExtensionContext,
    client: SynwardClient,
    diagnostics: SynwardDiagnostics,
    statusBar: StatusBarManager
) {
    // Open Synward Panel (main UI)
    context.subscriptions.push(
        vscode.commands.registerCommand('synward.openPanel', () => {
            SynwardPanel.createOrShow(context.extensionUri, client);
        })
    );
    
    // Validate current file
    context.subscriptions.push(
        vscode.commands.registerCommand('synward.validate', async () => {
            const editor = vscode.window.activeTextEditor;
            if (editor) {
                await validateDocument(editor.document, client, diagnostics, statusBar, context);
                vscode.window.showInformationMessage('Synward: Validation complete');
            }
        })
    );
    
    // Validate project
    context.subscriptions.push(
        vscode.commands.registerCommand('synward.validateProject', async () => {
            const workspaceFolders = vscode.workspace.workspaceFolders;
            if (workspaceFolders) {
                vscode.window.withProgress({
                    location: vscode.ProgressLocation.Notification,
                    title: "Synward: Validating project...",
                    cancellable: false
                }, async (progress) => {
                    for (const folder of workspaceFolders) {
                        await validateProject(folder.uri.fsPath, client, diagnostics, statusBar, progress);
                    }
                    return;
                });
            }
        })
    );
    
    // Accept violation
    context.subscriptions.push(
        vscode.commands.registerCommand('synward.acceptViolation', async (violationId: string) => {
            const reason = await vscode.window.showInputBox({
                prompt: 'Enter reason for accepting this violation',
                placeHolder: 'e.g., False positive - this is a test file'
            });
            
            if (reason) {
                const editor = vscode.window.activeTextEditor;
                if (editor) {
                    await client.acceptViolation(
                        violationId,
                        editor.document.uri.fsPath,
                        reason
                    );
                    vscode.window.showInformationMessage(`Synward: Violation ${violationId} accepted`);
                    // Re-validate
                    await validateDocument(editor.document, client, diagnostics, statusBar, context);
                }
            }
        })
    );
    
    // Show compliance status
    context.subscriptions.push(
        vscode.commands.registerCommand('synward.showCompliance', async () => {
            SynwardPanel.createOrShow(context.extensionUri, client);
            // Trigger compliance view
            setTimeout(() => {
                const panel = SynwardPanel.currentPanel;
                if (panel) {
                    client.getComplianceStatus().then(status => {
                        panel['updateCompliance'](status);
                    });
                }
            }, 100);
        })
    );
    
    // Analyze drift
    context.subscriptions.push(
        vscode.commands.registerCommand('synward.analyzeDrift', async () => {
            const editor = vscode.window.activeTextEditor;
            if (editor) {
                SynwardPanel.createOrShow(context.extensionUri, client);
                const drift = await client.analyzeDrift(editor.document.uri.fsPath, 30);
                // Panel will handle drift display
            }
        })
    );
}

async function validateDocument(
    document: vscode.TextDocument,
    client: SynwardClient,
    diagnostics: SynwardDiagnostics,
    statusBar: StatusBarManager,
    context?: vscode.ExtensionContext
): Promise<void> {
    try {
        const result = await client.validate(
            document.uri.fsPath,
            document.languageId,
            document.getText()
        );
        
        diagnostics.updateDiagnostics(document.uri, result.violations);
        statusBar.updateQualityScore(result.qualityScore, result.violations.length);
        
        // Update panel if open
        if (context && SynwardPanel.currentPanel) {
            SynwardPanel.currentPanel.updateViolations(
                document.uri.fsPath,
                result.violations,
                result.qualityScore
            );
        }
        
    } catch (error) {
        console.error('Synward validation error:', error);
        diagnostics.clearDiagnostics(document.uri);
        statusBar.showError();
    }
}

async function validateProject(
    projectPath: string,
    client: SynwardClient,
    diagnostics: SynwardDiagnostics,
    statusBar: StatusBarManager,
    progress: vscode.Progress<{ message?: string }>
): Promise<void> {
    // Find all supported files in project
    const files = await vscode.workspace.findFiles(
        '**/*.{rs,py,js,ts,go,c,cpp,java}',
        '**/node_modules/**'
    );
    
    progress.report({ message: `Found ${files.length} files` });
    
    let totalViolations = 0;
    let totalQuality = 0;
    
    for (let i = 0; i < files.length; i++) {
        const file = files[i];
        progress.report({ 
            message: `Validating ${file.fsPath.split('/').pop()} (${i + 1}/${files.length})` 
        });
        
        try {
            const document = await vscode.workspace.openTextDocument(file);
            const result = await client.validate(
                file.fsPath,
                document.languageId,
                document.getText()
            );
            
            diagnostics.updateDiagnostics(file, result.violations);
            totalViolations += result.violations.length;
            totalQuality += result.qualityScore;
            
        } catch (error) {
            console.error(`Error validating ${file.fsPath}:`, error);
        }
    }
    
    const avgQuality = files.length > 0 ? totalQuality / files.length : 0;
    statusBar.updateQualityScore(avgQuality, totalViolations);
    
    vscode.window.showInformationMessage(
        `Synward: Project validated. ${totalViolations} violations found. Quality: ${avgQuality.toFixed(0)}%`
    );
}

function generateComplianceHtml(status: any): string {
    return `<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Synward Compliance Status</title>
    <style>
        body { font-family: var(--vscode-font-family); padding: 20px; }
        h1 { color: var(--vscode-editor-foreground); }
        .stat { margin: 10px 0; }
        .stat-value { font-size: 2em; font-weight: bold; color: var(--vscode-textLink-foreground); }
        .tier { padding: 10px; margin: 10px 0; border-radius: 4px; }
        .inviolable { background: #ff444420; border-left: 4px solid #ff4444; }
        .strict { background: #ffaa0020; border-left: 4px solid #ffaa00; }
        .flexible { background: #44ff4420; border-left: 4px solid #44ff44; }
    </style>
</head>
<body>
    <h1>Compliance Engine Status</h1>
    
    <div class="stat">
        <div class="stat-value">${status.total_exemptions || 0}</div>
        <div>Total Exemptions</div>
    </div>
    
    <div class="stat">
        <div class="stat-value">${status.learned_patterns || 0}</div>
        <div>Learned Patterns</div>
    </div>
    
    <div class="stat">
        <div class="stat-value">${status.user_created || 0}</div>
        <div>User-Created Exemptions</div>
    </div>
    
    <h2>Contract Tiers</h2>
    
    <div class="tier inviolable">
        <strong>INVIOLABLE</strong><br>
        Security, memory safety, undefined behavior<br>
        <em>Always blocked, never bypassed</em>
    </div>
    
    <div class="tier strict">
        <strong>STRICT</strong><br>
        Logic errors, resource management<br>
        <em>Requires explicit acceptance with reason</em>
    </div>
    
    <div class="tier flexible">
        <strong>FLEXIBLE</strong><br>
        Style, naming, formatting<br>
        <em>Auto-learned after 3 occurrences</em>
    </div>
    
    <h2>Configuration</h2>
    <pre>${JSON.stringify(status.config || {}, null, 2)}</pre>
</body>
</html>`;
}

function generateDriftHtml(drift: any): string {
    return `<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Synward Drift Analysis</title>
    <style>
        body { font-family: var(--vscode-font-family); padding: 20px; }
        h1 { color: var(--vscode-editor-foreground); }
        .metric { display: inline-block; width: 150px; margin: 10px; padding: 15px; 
                  background: var(--vscode-editor-inactiveSelectionBackground); border-radius: 4px; }
        .metric-value { font-size: 1.5em; font-weight: bold; }
        .alert { padding: 10px; margin: 10px 0; background: #ff444420; border-left: 4px solid #ff4444; }
        .recommendation { padding: 15px; background: var(--vscode-textBlockQuote-background); 
                         border-left: 4px solid var(--vscode-textLink-foreground); margin-top: 20px; }
    </style>
</head>
<body>
    <h1>Drift Analysis</h1>
    <p>Path: ${drift.path || 'N/A'}</p>
    <p>Drift Score: <strong>${drift.drift_score?.toFixed(2) || 'N/A'}</strong></p>
    <p>Trend: <strong>${drift.trend || 'N/A'}</strong></p>
    
    <h2>Metrics</h2>
    <div class="metric">
        <div class="metric-value">${((drift.metrics?.type_strictness || 0) * 100).toFixed(0)}%</div>
        <div>Type Strictness</div>
    </div>
    <div class="metric">
        <div class="metric-value">${((drift.metrics?.naming_consistency || 0) * 100).toFixed(0)}%</div>
        <div>Naming Consistency</div>
    </div>
    <div class="metric">
        <div class="metric-value">${((drift.metrics?.error_handling_quality || 0) * 100).toFixed(0)}%</div>
        <div>Error Handling</div>
    </div>
    <div class="metric">
        <div class="metric-value">${((drift.metrics?.complexity_avg || 0) * 100).toFixed(0)}%</div>
        <div>Complexity</div>
    </div>
    
    ${drift.alerts?.length ? `
        <h2>Alerts</h2>
        ${drift.alerts.map((a: any) => `
            <div class="alert">
                <strong>${a.alert_type}</strong> (${a.severity})<br>
                ${a.message}
            </div>
        `).join('')}
    ` : ''}
    
    <div class="recommendation">
        <strong>Recommendation:</strong><br>
        ${drift.recommendation || 'No specific recommendations at this time.'}
    </div>
</body>
</html>`;
}

export function deactivate() {
    console.log('Synward extension deactivated');
    if (client) {
        client.dispose();
    }
}
