/**
 * Status Bar Manager - Quality score display
 */

import * as vscode from 'vscode';

export class StatusBarManager {
    private statusBarItem: vscode.StatusBarItem;
    
    constructor() {
        this.statusBarItem = vscode.window.createStatusBarItem(
            vscode.StatusBarAlignment.Right,
            100
        );
        
        this.statusBarItem.command = 'aether.showCompliance';
        this.statusBarItem.text = '$(check) Aether';
        this.statusBarItem.tooltip = 'Aether: No violations';
        this.statusBarItem.show();
    }
    
    updateQualityScore(score: number, violationCount: number): void {
        const config = vscode.workspace.getConfiguration('aether');
        
        if (!config.get<boolean>('showQualityScore')) {
            this.statusBarItem.hide();
            return;
        }
        
        this.statusBarItem.show();
        
        if (violationCount === 0) {
            this.statusBarItem.text = '$(check) Aether: 100%';
            this.statusBarItem.backgroundColor = undefined;
            this.statusBarItem.tooltip = 'Aether: No violations found';
        } else if (score >= 80) {
            this.statusBarItem.text = `$(warning) Aether: ${score.toFixed(0)}%`;
            this.statusBarItem.backgroundColor = undefined;
            this.statusBarItem.tooltip = `Aether: ${violationCount} warnings`;
        } else if (score >= 50) {
            this.statusBarItem.text = `$(alert) Aether: ${score.toFixed(0)}%`;
            this.statusBarItem.backgroundColor = new vscode.ThemeColor(
                'statusBarItem.warningBackground'
            );
            this.statusBarItem.tooltip = `Aether: ${violationCount} issues found`;
        } else {
            this.statusBarItem.text = `$(error) Aether: ${score.toFixed(0)}%`;
            this.statusBarItem.backgroundColor = new vscode.ThemeColor(
                'statusBarItem.errorBackground'
            );
            this.statusBarItem.tooltip = `Aether: ${violationCount} critical issues!`;
        }
    }
    
    showError(): void {
        this.statusBarItem.text = '$(error) Aether: Error';
        this.statusBarItem.backgroundColor = new vscode.ThemeColor(
            'statusBarItem.errorBackground'
        );
        this.statusBarItem.tooltip = 'Aether: Validation error occurred';
    }
    
    showLoading(): void {
        this.statusBarItem.text = '$(sync~spin) Aether...';
        this.statusBarItem.backgroundColor = undefined;
        this.statusBarItem.tooltip = 'Aether: Validating...';
    }
    
    dispose(): void {
        this.statusBarItem.dispose();
    }
}
