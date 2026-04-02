/**
 * Synward Code Actions - Quick fixes and refactorings
 */

import * as vscode from 'vscode';
import { SynwardClient, Violation } from './synwardClient';
import { SynwardDiagnostics } from './diagnostics';

export class SynwardCodeActions implements vscode.CodeActionProvider {
    public static readonly providedKinds = [
        vscode.CodeActionKind.QuickFix,
        vscode.CodeActionKind.Refactor,
    ];
    
    private client: SynwardClient;
    private diagnostics: SynwardDiagnostics;
    
    constructor(client: SynwardClient, diagnostics: SynwardDiagnostics) {
        this.client = client;
        this.diagnostics = diagnostics;
    }
    
    async provideCodeActions(
        document: vscode.TextDocument,
        range: vscode.Range,
        context: vscode.CodeActionContext,
        _token: vscode.CancellationToken
    ): Promise<vscode.CodeAction[] | undefined> {
        const actions: vscode.CodeAction[] = [];
        
        // Get diagnostics in range
        const synwardDiagnostics = context.diagnostics.filter(
            d => d.source === 'Synward'
        );
        
        for (const diagnostic of synwardDiagnostics) {
            const violationId = typeof diagnostic.code === 'string' 
                ? diagnostic.code 
                : String(diagnostic.code);
            
            // Quick Fix: Accept violation
            const acceptAction = new vscode.CodeAction(
                `Accept ${violationId} (with reason)`,
                vscode.CodeActionKind.QuickFix
            );
            acceptAction.command = {
                command: 'synward.acceptViolation',
                title: 'Accept Violation',
                arguments: [violationId],
            };
            acceptAction.diagnostics = [diagnostic];
            actions.push(acceptAction);
            
            // Quick Fix: Get AI suggestion
            const suggestAction = new vscode.CodeAction(
                `Get AI fix suggestion for ${violationId}`,
                vscode.CodeActionKind.QuickFix
            );
            suggestAction.command = {
                command: 'synward.suggestFix',
                title: 'Suggest Fix',
                arguments: [violationId, document.getText()],
            };
            suggestAction.diagnostics = [diagnostic];
            actions.push(suggestAction);
            
            // Quick Fix: Suppress for this file
            const suppressFileAction = new vscode.CodeAction(
                `Suppress ${violationId} for this file`,
                vscode.CodeActionKind.QuickFix
            );
            suppressFileAction.edit = this.createSuppressEdit(
                document,
                violationId,
                'file'
            );
            suppressFileAction.diagnostics = [diagnostic];
            actions.push(suppressFileAction);
            
            // Quick Fix: Suppress for this line
            const suppressLineAction = new vscode.CodeAction(
                `Suppress ${violationId} for this line`,
                vscode.CodeActionKind.QuickFix
            );
            suppressLineAction.edit = this.createSuppressEdit(
                document,
                violationId,
                'line',
                diagnostic.range.start.line
            );
            suppressLineAction.diagnostics = [diagnostic];
            actions.push(suppressLineAction);
        }
        
        return actions;
    }
    
    private createSuppressEdit(
        document: vscode.TextDocument,
        violationId: string,
        scope: 'file' | 'line',
        line?: number
    ): vscode.WorkspaceEdit {
        const edit = new vscode.WorkspaceEdit();
        
        if (scope === 'file') {
            // Add file-level suppression comment at top
            const suppression = `// synward:suppress ${violationId}\n`;
            edit.insert(
                document.uri,
                new vscode.Position(0, 0),
                suppression
            );
        } else if (scope === 'line' && line !== undefined) {
            // Add line-level suppression comment
            const lineText = document.lineAt(line).text;
            const indentation = lineText.match(/^\s*/)?.[0] || '';
            const suppression = `${indentation}// synward:ignore ${violationId}\n`;
            edit.insert(
                document.uri,
                new vscode.Position(line, 0),
                suppression
            );
        }
        
        return edit;
    }
}
