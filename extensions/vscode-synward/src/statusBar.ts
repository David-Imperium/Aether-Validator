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
        
        this.statusBarItem.command = 'synward.showCompliance';
        this.statusBarItem.text = '$(check) Synward';
        this.statusBarItem.tooltip = 'Synward: No violations';
        this.statusBarItem.show();
    }
    
    updateQualityScore(score: number, violationCount: number): void {
        const config = vscode.workspace.getConfiguration('synward');
        
        if (!config.get<boolean>('showQualityScore')) {
            this.statusBarItem.hide();
            return;
        }
        
        this.statusBarItem.show();
        
        if (violationCount === 0) {
            this.statusBarItem.text = '$(check) Synward: 100%';
            this.statusBarItem.backgroundColor = undefined;
            this.statusBarItem.tooltip = 'Synward: No violations found';
        } else if (score >= 80) {
            this.statusBarItem.text = `$(warning) Synward: ${score.toFixed(0)}%`;
            this.statusBarItem.backgroundColor = undefined;
            this.statusBarItem.tooltip = `Synward: ${violationCount} warnings`;
        } else if (score >= 50) {
            this.statusBarItem.text = `$(alert) Synward: ${score.toFixed(0)}%`;
            this.statusBarItem.backgroundColor = new vscode.ThemeColor(
                'statusBarItem.warningBackground'
            );
            this.statusBarItem.tooltip = `Synward: ${violationCount} issues found`;
        } else {
            this.statusBarItem.text = `$(error) Synward: ${score.toFixed(0)}%`;
            this.statusBarItem.backgroundColor = new vscode.ThemeColor(
                'statusBarItem.errorBackground'
            );
            this.statusBarItem.tooltip = `Synward: ${violationCount} critical issues!`;
        }
    }
    
    showError(): void {
        this.statusBarItem.text = '$(error) Synward: Error';
        this.statusBarItem.backgroundColor = new vscode.ThemeColor(
            'statusBarItem.errorBackground'
        );
        this.statusBarItem.tooltip = 'Synward: Validation error occurred';
    }
    
    showLoading(): void {
        this.statusBarItem.text = '$(sync~spin) Synward...';
        this.statusBarItem.backgroundColor = undefined;
        this.statusBarItem.tooltip = 'Synward: Validating...';
    }
    
    dispose(): void {
        this.statusBarItem.dispose();
    }
}
