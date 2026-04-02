/**
 * Synward Panel - Main UI Panel
 */

import * as vscode from 'vscode';
import { SynwardClient, Violation } from './synwardClient';

export class SynwardPanel {
    public static currentPanel: SynwardPanel | undefined;
    public static readonly viewType = 'synwardPanel';
    
    private readonly _panel: vscode.WebviewPanel;
    private readonly _extensionUri: vscode.Uri;
    private _disposables: vscode.Disposable[] = [];
    private _client: SynwardClient;
    private _currentViolations: Violation[] = [];
    private _currentFile: string = '';
    
    public static createOrShow(extensionUri: vscode.Uri, client: SynwardClient) {
        const column = vscode.window.activeTextEditor
            ? vscode.window.activeTextEditor.viewColumn
            : undefined;
        
        if (SynwardPanel.currentPanel) {
            SynwardPanel.currentPanel._panel.reveal(column);
            return;
        }
        
        const panel = vscode.window.createWebviewPanel(
            SynwardPanel.viewType,
            'Synward Validator',
            column || vscode.ViewColumn.One,
            {
                enableScripts: true,
                retainContextWhenHidden: true,
            }
        );
        
        SynwardPanel.currentPanel = new SynwardPanel(panel, extensionUri, client);
    }
    
    private constructor(
        panel: vscode.WebviewPanel,
        extensionUri: vscode.Uri,
        client: SynwardClient
    ) {
        this._panel = panel;
        this._extensionUri = extensionUri;
        this._client = client;
        
        this._update();
        
        this._panel.onDidDispose(() => this.dispose(), null, this._disposables);
        
        this._panel.webview.onDidReceiveMessage(
            async (message) => {
                switch (message.command) {
                    case 'validate':
                        await this._handleValidate(message.filePath);
                        break;
                    case 'acceptViolation':
                        await this._handleAcceptViolation(message.violationId, message.reason);
                        break;
                    case 'filterViolations':
                        this._handleFilter(message.filter);
                        break;
                    case 'showCompliance':
                        await this._handleShowCompliance();
                        break;
                    case 'showDrift':
                        await this._handleShowDrift(message.filePath);
                        break;
                    case 'openFile':
                        this._handleOpenFile(message.filePath, message.line);
                        break;
                    case 'refresh':
                        await this._handleValidate(this._currentFile);
                        break;
                }
            },
            null,
            this._disposables
        );
    }
    
    public updateViolations(file: string, violations: Violation[], qualityScore: number) {
        this._currentFile = file;
        this._currentViolations = violations;
        this._update({ violations, qualityScore, file });
    }
    
    private async _handleValidate(filePath: string) {
        const editor = vscode.window.activeTextEditor;
        if (editor && editor.document.uri.fsPath === filePath) {
            const result = await this._client.validate(
                filePath,
                editor.document.languageId,
                editor.document.getText()
            );
            this.updateViolations(filePath, result.violations, result.qualityScore);
        }
    }
    
    private async _handleAcceptViolation(violationId: string, reason: string) {
        try {
            await this._client.acceptViolation(violationId, this._currentFile, reason);
            await this._handleValidate(this._currentFile);
        } catch (error) {
            vscode.window.showErrorMessage(`Failed to accept violation: ${error}`);
        }
    }
    
    private _handleFilter(filter: string) {
        const filtered = this._currentViolations.filter(v => {
            if (filter === 'all') return true;
            if (filter === 'errors') return v.severity === 'error';
            if (filter === 'warnings') return v.severity === 'warning';
            if (filter === 'inviolable') return v.tier === 'inviolable';
            if (filter === 'strict') return v.tier === 'strict';
            if (filter === 'flexible') return v.tier === 'flexible';
            return true;
        });
        
        this._panel.webview.postMessage({
            command: 'updateViolations',
            violations: filtered,
        });
    }
    
    private async _handleShowCompliance() {
        try {
            const status = await this._client.getComplianceStatus();
            this._panel.webview.postMessage({
                command: 'showCompliance',
                status,
            });
        } catch (error) {
            vscode.window.showErrorMessage(`Failed to get compliance status: ${error}`);
        }
    }
    
    private async _handleShowDrift(filePath: string) {
        try {
            const drift = await this._client.analyzeDrift(filePath);
            this._panel.webview.postMessage({
                command: 'showDrift',
                drift,
            });
        } catch (error) {
            vscode.window.showErrorMessage(`Failed to analyze drift: ${error}`);
        }
    }
    
    private _handleOpenFile(filePath: string, line: number) {
        vscode.workspace.openTextDocument(filePath).then(doc => {
            vscode.window.showTextDocument(doc).then(editor => {
                const position = new vscode.Position(Math.max(0, line - 1), 0);
                editor.selection = new vscode.Selection(position, position);
                editor.revealRange(
                    new vscode.Range(position, position),
                    vscode.TextEditorRevealType.InCenter
                );
            });
        });
    }
    
    private _update(data?: any) {
        this._panel.webview.html = this._getHtmlForWebview(data);
    }
    
    private _getHtmlForWebview(data: any): string {
        const violations = data?.violations || [];
        const qualityScore = data?.qualityScore || 100;
        const file = data?.file || '';
        
        const grouped = this._groupViolations(violations);
        
        return `<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Synward Validator</title>
    <style>
        * { box-sizing: border-box; margin: 0; padding: 0; }
        
        body {
            font-family: var(--vscode-font-family);
            background: var(--vscode-editor-background);
            color: var(--vscode-editor-foreground);
            padding: 0;
        }
        
        /* Header */
        .header {
            background: var(--vscode-editor-inactiveSelectionBackground);
            padding: 16px 20px;
            border-bottom: 1px solid var(--vscode-panel-border);
            display: flex;
            justify-content: space-between;
            align-items: center;
        }
        
        .header-left { display: flex; align-items: center; gap: 16px; }
        .header h1 { font-size: 1.2em; font-weight: 600; }
        
        .quality-score {
            display: flex;
            align-items: center;
            gap: 8px;
            padding: 8px 16px;
            border-radius: 20px;
            font-weight: bold;
        }
        
        .quality-score.excellent { background: #22c55e20; color: #22c55e; }
        .quality-score.good { background: #84cc1620; color: #84cc16; }
        .quality-score.warning { background: #eab30820; color: #eab308; }
        .quality-score.poor { background: #ef444420; color: #ef4444; }
        
        .header-actions { display: flex; gap: 8px; }
        
        button {
            background: var(--vscode-button-background);
            color: var(--vscode-button-foreground);
            border: none;
            padding: 8px 16px;
            border-radius: 4px;
            cursor: pointer;
            font-size: 0.9em;
        }
        
        button:hover { background: var(--vscode-button-hoverBackground); }
        button.secondary { background: var(--vscode-button-secondaryBackground); }
        
        /* Tabs */
        .tabs {
            display: flex;
            border-bottom: 1px solid var(--vscode-panel-border);
            background: var(--vscode-editor-inactiveSelectionBackground);
        }
        
        .tab {
            padding: 12px 20px;
            cursor: pointer;
            border-bottom: 2px solid transparent;
            color: var(--vscode-descriptionForeground);
        }
        
        .tab:hover { color: var(--vscode-editor-foreground); }
        .tab.active { 
            border-bottom-color: var(--vscode-textLink-foreground);
            color: var(--vscode-editor-foreground);
        }
        
        /* Content */
        .content {
            padding: 16px;
            overflow-y: auto;
            height: calc(100vh - 120px);
        }
        
        /* File info */
        .file-info {
            background: var(--vscode-editor-inactiveSelectionBackground);
            padding: 12px 16px;
            border-radius: 8px;
            margin-bottom: 16px;
            display: flex;
            justify-content: space-between;
            align-items: center;
        }
        
        .file-path { font-family: monospace; font-size: 0.9em; }
        
        /* Filters */
        .filters {
            display: flex;
            gap: 8px;
            margin-bottom: 16px;
            flex-wrap: wrap;
        }
        
        .filter-btn {
            padding: 6px 12px;
            border-radius: 16px;
            background: var(--vscode-editor-inactiveSelectionBackground);
            border: 1px solid var(--vscode-panel-border);
            cursor: pointer;
            font-size: 0.85em;
        }
        
        .filter-btn.active {
            background: var(--vscode-textLink-foreground);
            color: var(--vscode-editor-background);
        }
        
        /* Violation groups */
        .tier-group {
            margin-bottom: 20px;
        }
        
        .tier-header {
            display: flex;
            align-items: center;
            gap: 8px;
            padding: 8px 12px;
            border-radius: 8px;
            margin-bottom: 8px;
            font-weight: 600;
        }
        
        .tier-header.inviolable { background: #ef444420; border-left: 4px solid #ef4444; }
        .tier-header.strict { background: #f9731620; border-left: 4px solid #f97316; }
        .tier-header.flexible { background: #22c55e20; border-left: 4px solid #22c55e; }
        
        .tier-count {
            background: var(--vscode-badge-background);
            color: var(--vscode-badge-foreground);
            padding: 2px 8px;
            border-radius: 10px;
            font-size: 0.8em;
        }
        
        /* Violation card */
        .violation-card {
            background: var(--vscode-editor-background);
            border: 1px solid var(--vscode-panel-border);
            border-radius: 8px;
            padding: 12px;
            margin-bottom: 8px;
            cursor: pointer;
            transition: all 0.2s;
        }
        
        .violation-card:hover {
            border-color: var(--vscode-textLink-foreground);
            background: var(--vscode-editor-inactiveSelectionBackground);
        }
        
        .violation-header {
            display: flex;
            justify-content: space-between;
            align-items: flex-start;
        }
        
        .violation-id {
            font-family: monospace;
            font-weight: 600;
            color: var(--vscode-textLink-foreground);
        }
        
        .violation-severity {
            padding: 2px 8px;
            border-radius: 4px;
            font-size: 0.75em;
            text-transform: uppercase;
        }
        
        .violation-severity.error { background: #ef444420; color: #ef4444; }
        .violation-severity.warning { background: #eab30820; color: #eab308; }
        .violation-severity.info { background: #3b82f620; color: #3b82f6; }
        
        .violation-message {
            margin: 8px 0;
            color: var(--vscode-editor-foreground);
        }
        
        .violation-location {
            font-family: monospace;
            font-size: 0.85em;
            color: var(--vscode-descriptionForeground);
        }
        
        .violation-actions {
            display: flex;
            gap: 8px;
            margin-top: 12px;
        }
        
        .violation-actions button {
            padding: 6px 12px;
            font-size: 0.8em;
        }
        
        /* Compliance Section */
        .compliance-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 16px;
            margin-bottom: 20px;
        }
        
        .stat-card {
            background: var(--vscode-editor-inactiveSelectionBackground);
            border-radius: 12px;
            padding: 20px;
            text-align: center;
        }
        
        .stat-value {
            font-size: 2.5em;
            font-weight: bold;
            color: var(--vscode-textLink-foreground);
        }
        
        .stat-label {
            color: var(--vscode-descriptionForeground);
            margin-top: 4px;
        }
        
        /* Drift Section */
        .drift-metrics {
            display: grid;
            grid-template-columns: repeat(4, 1fr);
            gap: 12px;
            margin: 20px 0;
        }
        
        .metric-card {
            background: var(--vscode-editor-inactiveSelectionBackground);
            border-radius: 8px;
            padding: 16px;
            text-align: center;
        }
        
        .metric-value {
            font-size: 1.5em;
            font-weight: bold;
        }
        
        .metric-bar {
            height: 4px;
            background: var(--vscode-progressBar-background);
            border-radius: 2px;
            margin-top: 8px;
        }
        
        .metric-bar-fill {
            height: 100%;
            border-radius: 2px;
            transition: width 0.3s;
        }
        
        .metric-bar-fill.high { background: #22c55e; }
        .metric-bar-fill.medium { background: #eab308; }
        .metric-bar-fill.low { background: #ef4444; }
        
        /* Empty state */
        .empty-state {
            text-align: center;
            padding: 60px 20px;
            color: var(--vscode-descriptionForeground);
        }
        
        .empty-state-icon {
            font-size: 4em;
            margin-bottom: 16px;
        }
        
        /* Hidden */
        .hidden { display: none !important; }
    </style>
</head>
<body>
    <div class="header">
        <div class="header-left">
            <h1>⚡ Synward</h1>
            <div class="quality-score ${this._getQualityClass(qualityScore)}">
                <span>📊</span>
                <span>${qualityScore.toFixed(0)}%</span>
            </div>
        </div>
        <div class="header-actions">
            <button onclick="refresh()">🔄 Refresh</button>
            <button onclick="showCompliance()">📋 Compliance</button>
            <button onclick="showDrift()">📈 Drift</button>
        </div>
    </div>
    
    <div class="tabs">
        <div class="tab active" data-tab="violations" onclick="switchTab('violations')">
            Violations (${violations.length})
        </div>
        <div class="tab" data-tab="compliance" onclick="switchTab('compliance')">
            Compliance
        </div>
        <div class="tab" data-tab="drift" onclick="switchTab('drift')">
            Drift Analysis
        </div>
    </div>
    
    <div class="content">
        <!-- Violations Tab -->
        <div id="violations-tab" class="tab-content">
            ${file ? `
                <div class="file-info">
                    <span class="file-path">📄 ${file}</span>
                    <span>${violations.length} violations</span>
                </div>
            ` : ''}
            
            <div class="filters">
                <button class="filter-btn active" onclick="filter('all')">All</button>
                <button class="filter-btn" onclick="filter('errors')">Errors</button>
                <button class="filter-btn" onclick="filter('warnings')">Warnings</button>
                <button class="filter-btn" onclick="filter('inviolable')">🚫 Inviolable</button>
                <button class="filter-btn" onclick="filter('strict')">⚠️ Strict</button>
                <button class="filter-btn" onclick="filter('flexible')">✅ Flexible</button>
            </div>
            
            ${this._renderViolations(grouped)}
        </div>
        
        <!-- Compliance Tab -->
        <div id="compliance-tab" class="tab-content hidden">
            <div class="compliance-grid">
                <div class="stat-card">
                    <div class="stat-value" id="total-exemptions">-</div>
                    <div class="stat-label">Total Exemptions</div>
                </div>
                <div class="stat-card">
                    <div class="stat-value" id="learned-patterns">-</div>
                    <div class="stat-label">Learned Patterns</div>
                </div>
                <div class="stat-card">
                    <div class="stat-value" id="user-created">-</div>
                    <div class="stat-label">User Created</div>
                </div>
                <div class="stat-card">
                    <div class="stat-value" id="occurrence-tracking">-</div>
                    <div class="stat-label">Occurrence Tracking</div>
                </div>
            </div>
            
            <h2>Contract Tiers</h2>
            
            <div class="tier-group">
                <div class="tier-header inviolable">
                    🚫 INVIOLABLE
                    <span class="tier-count">Always Blocked</span>
                </div>
                <p style="padding: 8px 12px; color: var(--vscode-descriptionForeground);">
                    Security violations, memory safety issues, undefined behavior.
                    These can never be bypassed or accepted.
                </p>
            </div>
            
            <div class="tier-group">
                <div class="tier-header strict">
                    ⚠️ STRICT
                    <span class="tier-count">Requires Reason</span>
                </div>
                <p style="padding: 8px 12px; color: var(--vscode-descriptionForeground);">
                    Logic errors, resource management, concurrency issues.
                    Must provide documented reason to accept.
                </p>
            </div>
            
            <div class="tier-group">
                <div class="tier-header flexible">
                    ✅ FLEXIBLE
                    <span class="tier-count">Auto-Learn</span>
                </div>
                <p style="padding: 8px 12px; color: var(--vscode-descriptionForeground);">
                    Style, naming, formatting conventions.
                    Automatically learned after 3 occurrences.
                </p>
            </div>
        </div>
        
        <!-- Drift Tab -->
        <div id="drift-tab" class="tab-content hidden">
            <div class="file-info">
                <span>📈 Drift Analysis</span>
                <span id="drift-trend">-</span>
            </div>
            
            <div class="drift-metrics">
                <div class="metric-card">
                    <div class="metric-value" id="metric-type">-</div>
                    <div class="metric-label">Type Strictness</div>
                    <div class="metric-bar"><div class="metric-bar-fill high" id="bar-type"></div></div>
                </div>
                <div class="metric-card">
                    <div class="metric-value" id="metric-naming">-</div>
                    <div class="metric-label">Naming Consistency</div>
                    <div class="metric-bar"><div class="metric-bar-fill high" id="bar-naming"></div></div>
                </div>
                <div class="metric-card">
                    <div class="metric-value" id="metric-errors">-</div>
                    <div class="metric-label">Error Handling</div>
                    <div class="metric-bar"><div class="metric-bar-fill medium" id="bar-errors"></div></div>
                </div>
                <div class="metric-card">
                    <div class="metric-value" id="metric-complexity">-</div>
                    <div class="metric-label">Complexity</div>
                    <div class="metric-bar"><div class="metric-bar-fill low" id="bar-complexity"></div></div>
                </div>
            </div>
            
            <div id="drift-alerts"></div>
            <div id="drift-recommendation" class="stat-card" style="text-align: left; margin-top: 20px;"></div>
        </div>
    </div>
    
    <script>
        const vscode = acquireVsCodeApi();
        
        function refresh() {
            vscode.postMessage({ command: 'refresh' });
        }
        
        function showCompliance() {
            vscode.postMessage({ command: 'showCompliance' });
        }
        
        function showDrift() {
            vscode.postMessage({ command: 'showDrift', filePath: '${file}' });
        }
        
        function filter(type) {
            document.querySelectorAll('.filter-btn').forEach(btn => btn.classList.remove('active'));
            event.target.classList.add('active');
            vscode.postMessage({ command: 'filterViolations', filter: type });
        }
        
        function switchTab(tabName) {
            document.querySelectorAll('.tab').forEach(t => t.classList.remove('active'));
            document.querySelectorAll('.tab-content').forEach(t => t.classList.add('hidden'));
            document.querySelector('[data-tab=' + tabName + ']').classList.add('active');
            document.getElementById(tabName + '-tab').classList.remove('hidden');
        }
        
        function openFile(path, line) {
            vscode.postMessage({ command: 'openFile', filePath: path, line: line });
        }
        
        function acceptViolation(id) {
            const reason = prompt('Enter reason for accepting this violation:');
            if (reason) {
                vscode.postMessage({ command: 'acceptViolation', violationId: id, reason: reason });
            }
        }
        
        // Handle messages from extension
        window.addEventListener('message', event => {
            const message = event.data;
            
            switch (message.command) {
                case 'updateViolations':
                    // Update UI with filtered violations
                    break;
                    
                case 'showCompliance':
                    document.getElementById('total-exemptions').textContent = message.status.total_exemptions || 0;
                    document.getElementById('learned-patterns').textContent = message.status.learned_patterns || 0;
                    document.getElementById('user-created').textContent = message.status.user_created || 0;
                    document.getElementById('occurrence-tracking').textContent = message.status.occurrence_tracking || 0;
                    switchTab('compliance');
                    break;
                    
                case 'showDrift':
                    const drift = message.drift;
                    document.getElementById('drift-trend').textContent = drift.trend || 'Unknown';
                    
                    const typeVal = Math.round((drift.metrics?.type_strictness || 0) * 100);
                    document.getElementById('metric-type').textContent = typeVal + '%';
                    document.getElementById('bar-type').style.width = typeVal + '%';
                    
                    const namingVal = Math.round((drift.metrics?.naming_consistency || 0) * 100);
                    document.getElementById('metric-naming').textContent = namingVal + '%';
                    document.getElementById('bar-naming').style.width = namingVal + '%';
                    
                    const errorVal = Math.round((drift.metrics?.error_handling_quality || 0) * 100);
                    document.getElementById('metric-errors').textContent = errorVal + '%';
                    document.getElementById('bar-errors').style.width = errorVal + '%';
                    
                    const cplxVal = Math.round((1 - (drift.metrics?.complexity_avg || 0)) * 100);
                    document.getElementById('metric-complexity').textContent = cplxVal + '%';
                    document.getElementById('bar-complexity').style.width = cplxVal + '%';
                    
                    if (drift.alerts?.length) {
                        document.getElementById('drift-alerts').innerHTML = drift.alerts.map(a => 
                            '<div class="violation-card"><strong>' + a.alert_type + '</strong><br>' + a.message + '</div>'
                        ).join('');
                    }
                    
                    document.getElementById('drift-recommendation').innerHTML = 
                        '<strong>Recommendation:</strong><br>' + (drift.recommendation || 'No specific recommendations.');
                    
                    switchTab('drift');
                    break;
            }
        });
    </script>
</body>
</html>`;
    }
    
    private _getQualityClass(score: number): string {
        if (score >= 90) return 'excellent';
        if (score >= 70) return 'good';
        if (score >= 50) return 'warning';
        return 'poor';
    }
    
    private _groupViolations(violations: Violation[]): Map<string, Violation[]> {
        const grouped = new Map<string, Violation[]>();
        
        for (const v of violations) {
            const tier = v.tier || 'flexible';
            if (!grouped.has(tier)) {
                grouped.set(tier, []);
            }
            grouped.get(tier)!.push(v);
        }
        
        // Order: inviolable, strict, flexible
        const ordered = new Map();
        if (grouped.has('inviolable')) ordered.set('inviolable', grouped.get('inviolable'));
        if (grouped.has('strict')) ordered.set('strict', grouped.get('strict'));
        if (grouped.has('flexible')) ordered.set('flexible', grouped.get('flexible'));
        
        return ordered;
    }
    
    private _renderViolations(grouped: Map<string, Violation[]>): string {
        if (grouped.size === 0) {
            return `
                <div class="empty-state">
                    <div class="empty-state-icon">✅</div>
                    <h2>No violations found!</h2>
                    <p>Your code passes all validation checks.</p>
                </div>
            `;
        }
        
        let html = '';
        
        for (const [tier, violations] of grouped) {
            const icons: Record<string, string> = {
                inviolable: '🚫',
                strict: '⚠️',
                flexible: '✅',
            };
            
            html += `
                <div class="tier-group">
                    <div class="tier-header ${tier}">
                        ${icons[tier] || '📋'} ${tier.toUpperCase()}
                        <span class="tier-count">${violations.length}</span>
                    </div>
            `;
            
            for (const v of violations) {
                html += `
                    <div class="violation-card" onclick="openFile('${this._currentFile}', ${v.line})">
                        <div class="violation-header">
                            <span class="violation-id">${v.id}</span>
                            <span class="violation-severity ${v.severity}">${v.severity}</span>
                        </div>
                        <div class="violation-message">${v.message}</div>
                        <div class="violation-location">Line ${v.line}${v.confidence ? ` • Confidence: ${(v.confidence * 100).toFixed(0)}%` : ''}</div>
                        ${tier !== 'inviolable' ? `
                            <div class="violation-actions">
                                <button onclick="event.stopPropagation(); acceptViolation('${v.id}')">Accept</button>
                            </div>
                        ` : ''}
                    </div>
                `;
            }
            
            html += '</div>';
        }
        
        return html;
    }
    
    public dispose() {
        SynwardPanel.currentPanel = undefined;
        
        this._panel.dispose();
        
        while (this._disposables.length) {
            const x = this._disposables.pop();
            if (x) x.dispose();
        }
    }
}
