// Aether Desktop - Frontend Logic

const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

// State
const state = {
    config: { languages: ['rust', 'python'], severity: 'standard', autoFix: false },
    watcherErrors: { new: [], prex: [] },
    activeFile: null,
    activeFileContent: '',
    watcherRunning: false,
    workspaceRoot: '',
    expandedDirs: new Set(),
    activeView: 'explorer',
    activePanel: 'problems'
};

// Initialize
async function init() {
    setupActivityBar();
    setupPanelTabs();
    setupProblemCategories();
    setupStatusBar();
    setupManualScan();
    setupSettingsNav();
    
    await loadConfig();
    await loadWorkspaceRoot();
    await updateStatus();
    setupTauriListeners();
    
    // Load file tree
    await loadFileTree();
    
    // Init settings
    renderSettingsSection('languages');
    
    logOutput('Aether Desktop ready', 'success');
}

// -----------------------------------------------------------------------------
// ACTIVITY BAR
// -----------------------------------------------------------------------------

function setupActivityBar() {
    document.querySelectorAll('.action-btn').forEach(btn => {
        btn.addEventListener('click', () => {
            const view = btn.dataset.view;
            
            // Problems button opens panel
            if (view === 'problems') {
                openProblemsPanel();
                document.querySelector('[data-panel="problems"]').click();
                return;
            }
            
            // Switch sidebar view
            state.activeView = view;
            document.querySelectorAll('.action-btn').forEach(b => b.classList.remove('active'));
            btn.classList.add('active');
            
            document.querySelectorAll('.sidebar-view').forEach(v => v.style.display = 'none');
            const targetView = document.getElementById(`view-${view}`);
            if (targetView) targetView.style.display = 'flex';
        });
    });
}

// -----------------------------------------------------------------------------
// PANEL TABS
// -----------------------------------------------------------------------------

function setupPanelTabs() {
    document.querySelectorAll('.panel-tab').forEach(tab => {
        tab.addEventListener('click', () => {
            state.activePanel = tab.dataset.panel;
            
            document.querySelectorAll('.panel-tab').forEach(t => t.classList.remove('active'));
            tab.classList.add('active');
            
            document.querySelectorAll('.panel-content').forEach(c => c.style.display = 'none');
            document.getElementById(`panel-${tab.dataset.panel}`).style.display = 'block';
            
            openProblemsPanel();
        });
    });
    
    // Toggle panel
    const toggleBtn = document.querySelector('.toggle-panel-btn');
    toggleBtn.addEventListener('click', () => {
        const panel = document.getElementById('problems-panel');
        const isCollapsed = panel.classList.contains('collapsed');
        
        if (isCollapsed) {
            openProblemsPanel();
        } else {
            closeProblemsPanel();
        }
    });
    
    // Clear problems
    document.getElementById('btn-clear-problems').addEventListener('click', () => {
        state.watcherErrors = { new: [], prex: [] };
        renderProblems();
        updateStatusBar();
        logOutput('Problems cleared', 'muted');
    });
    
    // Filter
    document.getElementById('filter-severity').addEventListener('change', (e) => {
        renderProblems(e.target.value);
    });
}

function openProblemsPanel() {
    const panel = document.getElementById('problems-panel');
    panel.classList.remove('collapsed');
    panel.style.height = '180px';
    
    const btn = document.querySelector('.toggle-panel-btn svg');
    btn.innerHTML = '<polyline points="18 15 12 9 6 15"/>';
}

function closeProblemsPanel() {
    const panel = document.getElementById('problems-panel');
    panel.classList.add('collapsed');
    panel.style.height = '32px';
    
    const btn = document.querySelector('.toggle-panel-btn svg');
    btn.innerHTML = '<polyline points="6 9 12 15 18 9"/>';
}

// -----------------------------------------------------------------------------
// PROBLEM CATEGORIES
// -----------------------------------------------------------------------------

function setupProblemCategories() {
    document.querySelectorAll('.category-header').forEach(header => {
        header.addEventListener('click', () => {
            const category = header.parentElement;
            const items = category.querySelector('.category-items');
            const chevron = header.querySelector('.chevron');
            
            if (category.classList.contains('expanded')) {
                category.classList.remove('expanded');
                items.style.display = 'none';
                chevron.textContent = '▸';
            } else {
                category.classList.add('expanded');
                items.style.display = 'block';
                chevron.textContent = '▾';
            }
        });
    });
}

// -----------------------------------------------------------------------------
// STATUS BAR
// -----------------------------------------------------------------------------

function setupStatusBar() {
    // Watcher toggle
    document.getElementById('status-watcher').addEventListener('click', async () => {
        try {
            if (state.watcherRunning) {
                await invoke('stop_watcher');
                state.watcherRunning = false;
                logOutput('Watcher stopped', 'muted');
            } else {
                await invoke('start_watcher');
                state.watcherRunning = true;
                logOutput('Watcher started', 'success');
            }
            updateStatusBar();
        } catch (e) {
            logOutput('Failed: ' + e, 'error');
        }
    });
    
    // Fix all
    document.getElementById('btn-fix-all').addEventListener('click', async () => {
        const fixable = [...state.watcherErrors.new, ...state.watcherErrors.prex]
            .filter(e => e.suggestion);
        
        if (fixable.length === 0) {
            logOutput('No auto-fixable issues', 'muted');
            return;
        }
        
        logOutput(`Applying ${fixable.length} fixes...`, 'muted');
        
        for (const err of fixable) {
            try {
                await invoke('apply_fix', { errorId: err.id, file: err.file });
            } catch (e) {
                logOutput(`Failed to fix ${err.id}: ${e}`, 'error');
            }
        }
        
        logOutput('Fixes applied', 'success');
    });
}

function updateStatusBar() {
    const watcher = document.getElementById('status-watcher');
    const dot = watcher.querySelector('.status-dot');
    const text = watcher.querySelector('.status-text');
    
    if (state.watcherRunning) {
        dot.classList.add('active');
        text.textContent = 'watcher attivo';
    } else {
        dot.classList.remove('active');
        text.textContent = 'watcher fermo';
    }
    
    // Error/warning counts
    const errors = state.watcherErrors.new.filter(e => e.severity === 'error').length +
                   state.watcherErrors.prex.filter(e => e.severity === 'error').length;
    const warnings = state.watcherErrors.new.filter(e => e.severity === 'warning').length +
                     state.watcherErrors.prex.filter(e => e.severity === 'warning').length;
    
    document.querySelector('#status-errors span').textContent = errors;
    document.querySelector('#status-warnings span').textContent = warnings;
    
    // Badge
    const badge = document.getElementById('activity-problem-badge');
    const total = state.watcherErrors.new.length;
    if (total > 0) {
        badge.textContent = total;
        badge.style.display = 'block';
    } else {
        badge.style.display = 'none';
    }
}

// -----------------------------------------------------------------------------
// FILE TREE
// -----------------------------------------------------------------------------

async function loadWorkspaceRoot() {
    try {
        state.workspaceRoot = await invoke('get_workspace_root');
    } catch (e) {
        console.error('Failed to get workspace root:', e);
        state.workspaceRoot = '.';
    }
}

async function loadFileTree(path = null) {
    const targetPath = path || state.workspaceRoot;
    
    try {
        const entries = await invoke('list_directory', { path: targetPath });
        renderFileTree(entries, targetPath);
    } catch (e) {
        console.error('Failed to load file tree:', e);
        document.getElementById('file-tree').innerHTML = '<div class="empty-state">Failed to load directory</div>';
    }
}

function renderFileTree(entries, basePath) {
    const container = document.getElementById('file-tree');
    const dirName = basePath.split(/[\\/]/).pop() || basePath;
    const isExpanded = state.expandedDirs.has(basePath);
    
    let html = `
        <div class="tree-item folder ${isExpanded ? 'open' : ''}" data-path="${basePath}">
            <span class="chevron">${isExpanded ? '▾' : '▸'}</span>
            <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
            </svg>
            <span class="name">${dirName}</span>
        </div>
    `;
    
    if (isExpanded) {
        for (const entry of entries) {
            if (entry.is_dir) {
                const hasSubItems = state.expandedDirs.has(entry.path);
                html += `
                    <div class="tree-item folder ${hasSubItems ? 'open' : ''}" data-path="${entry.path}" style="padding-left: 20px;">
                        <span class="chevron">${hasSubItems ? '▾' : '▸'}</span>
                        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                            <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
                        </svg>
                        <span class="name">${entry.name}</span>
                    </div>
                `;
            } else {
                const errorCount = getFileErrorCount(entry.path);
                const icon = getFileIcon(entry.extension);
                html += `
                    <div class="tree-item file ${state.activeFile === entry.path ? 'active' : ''}" 
                         data-path="${entry.path}" style="padding-left: 36px;">
                        <span class="status-dot ${errorCount > 0 ? 'red' : 'green'}"></span>
                        ${icon}
                        <span class="name">${entry.name}</span>
                        ${errorCount > 0 ? `<span class="inline-badge">${errorCount}</span>` : ''}
                    </div>
                `;
            }
        }
    }
    
    container.innerHTML = html;
    setupFileTreeEvents();
}

function getFileIcon(ext) {
    const icons = {
        rs: '<svg width="16" height="16" viewBox="0 0 24 24" fill="#e05c5c"><path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5"/></svg>',
        py: '<svg width="16" height="16" viewBox="0 0 24 24" fill="#4db8e8"><path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm0 18c-4.41 0-8-3.59-8-8s3.59-8 8-8 8 3.59 8 8-3.59 8-8 8z"/></svg>',
        js: '<svg width="16" height="16" viewBox="0 0 24 24" fill="#f5c842"><rect x="3" y="3" width="18" height="18" rx="2"/></svg>',
        ts: '<svg width="16" height="16" viewBox="0 0 24 24" fill="#3178c6"><rect x="3" y="3" width="18" height="18" rx="2"/></svg>',
        default: '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/></svg>'
    };
    return icons[ext] || icons.default;
}

function setupFileTreeEvents() {
    document.querySelectorAll('.tree-item').forEach(item => {
        item.addEventListener('click', async (e) => {
            e.stopPropagation();
            const path = item.dataset.path;
            
            if (item.classList.contains('folder')) {
                if (state.expandedDirs.has(path)) {
                    state.expandedDirs.delete(path);
                } else {
                    state.expandedDirs.add(path);
                }
                await loadFileTree(path);
            } else {
                await openFile(path);
            }
        });
    });
}

function getFileErrorCount(path) {
    return [...state.watcherErrors.new, ...state.watcherErrors.prex]
        .filter(e => e.file === path).length;
}

async function openFile(path) {
    try {
        const result = await invoke('read_file', { path });
        state.activeFile = path;
        state.activeFileContent = result.content;
        
        // Show code view
        document.getElementById('code-view').style.display = 'flex';
        document.getElementById('manual-scan-view').style.display = 'none';
        
        renderCodeContent(result.content, result.language);
        addTab(path);
        
        // Update status
        const fileName = path.split(/[\\/]/).pop();
        document.getElementById('status-file').textContent = fileName;
        document.getElementById('status-lang').textContent = result.language;
        
        logOutput(`Opened: ${fileName}`, 'muted');
    } catch (e) {
        logOutput('Failed to open file: ' + e, 'error');
    }
}

function renderCodeContent(content, language) {
    const lines = content.split('\n');
    
    // Line numbers
    document.getElementById('line-numbers').innerHTML = 
        lines.map((_, i) => `<div>${i + 1}</div>`).join('');
    
    // Code with error highlights
    const codeLines = lines.map((line, i) => {
        const lineNum = i + 1;
        const error = [...state.watcherErrors.new, ...state.watcherErrors.prex]
            .find(e => e.file === state.activeFile && e.line === lineNum);
        
        let classes = ['code-line'];
        if (error) {
            classes.push(error.severity === 'error' ? 'error-line' : 'warning-line');
        }
        
        return `<div class="${classes.join(' ')}" data-line="${lineNum}">${escapeHtml(line)}</div>`;
    }).join('');
    
    document.getElementById('code-content').innerHTML = codeLines;
    
    // Click on line to show tooltip
    document.querySelectorAll('.code-line').forEach(line => {
        line.addEventListener('click', () => showLineTooltip(line));
    });
}

function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

function showLineTooltip(lineEl) {
    const lineNum = parseInt(lineEl.dataset.line);
    const error = [...state.watcherErrors.new, ...state.watcherErrors.prex]
        .find(e => e.file === state.activeFile && e.line === lineNum);
    
    // Remove existing tooltip
    document.querySelectorAll('.inline-tooltip').forEach(t => t.remove());
    
    if (error) {
        const tooltip = document.createElement('div');
        tooltip.className = 'inline-tooltip';
        tooltip.innerHTML = `
            <div class="tooltip-header">
                <span class="tooltip-title ${error.severity}">${error.id}</span>
                <span class="tooltip-badge ${error.classification || 'new'}">${error.classification || 'new'}</span>
            </div>
            <p>${error.message}</p>
            ${error.suggestion ? `<div class="tooltip-fix"><code>${error.suggestion}</code></div>` : ''}
            <div class="tooltip-actions">
                ${error.suggestion ? '<button class="btn btn-apply" data-action="fix">Apply Fix</button>' : ''}
                <button class="btn btn-ignore" data-action="ignore">Ignore</button>
            </div>
        `;
        
        lineEl.appendChild(tooltip);
        
        tooltip.querySelector('[data-action="fix"]')?.addEventListener('click', async () => {
            await invoke('apply_fix', { errorId: error.id, file: error.file });
            tooltip.remove();
            if (state.activeFile) await openFile(state.activeFile);
        });
        
        tooltip.querySelector('[data-action="ignore"]').addEventListener('click', () => {
            tooltip.remove();
        });
    }
}

// -----------------------------------------------------------------------------
// TABS
// -----------------------------------------------------------------------------

function addTab(path) {
    const tabsBar = document.getElementById('editor-tabs');
    const fileName = path.split(/[\\/]/).pop();
    const errorCount = getFileErrorCount(path);
    
    // Remove existing tabs for this file
    tabsBar.querySelectorAll(`.tab[data-path="${path}"]`).forEach(t => t.remove());
    
    // Remove active from other tabs
    tabsBar.querySelectorAll('.tab').forEach(t => t.classList.remove('active'));
    
    const tab = document.createElement('div');
    tab.className = 'tab active';
    tab.dataset.path = path;
    tab.innerHTML = `
        <span class="status-dot ${errorCount > 0 ? 'red' : 'green'}"></span>
        <span class="tab-title">${fileName}</span>
        <button class="tab-close">×</button>
    `;
    
    tabsBar.insertBefore(tab, document.getElementById('btn-manual-scan'));
    
    tab.addEventListener('click', (e) => {
        if (!e.target.classList.contains('tab-close')) {
            openFile(path);
        }
    });
    
    tab.querySelector('.tab-close').addEventListener('click', (e) => {
        e.stopPropagation();
        closeTab(tab);
    });
}

function closeTab(tab) {
    const path = tab.dataset.path;
    tab.remove();
    
    if (state.activeFile === path) {
        state.activeFile = null;
        state.activeFileContent = '';
        document.getElementById('code-view').style.display = 'none';
        document.getElementById('status-file').textContent = '-';
        document.getElementById('status-lang').textContent = '-';
    }
}

// -----------------------------------------------------------------------------
// MANUAL SCAN
// -----------------------------------------------------------------------------

function setupManualScan() {
    document.getElementById('btn-manual-scan').addEventListener('click', () => {
        document.getElementById('code-view').style.display = 'none';
        document.getElementById('manual-scan-view').style.display = 'flex';
        
        document.querySelectorAll('.tab').forEach(t => t.classList.remove('active'));
    });
    
    document.getElementById('validate-btn').addEventListener('click', validateManualCode);
    
    document.getElementById('clear-btn').addEventListener('click', () => {
        document.getElementById('code-input').value = '';
    });
}

async function validateManualCode() {
    const code = document.getElementById('code-input').value;
    const language = document.getElementById('language-select').value;
    
    if (!code.trim()) {
        logOutput('No code to validate', 'warning');
        return;
    }
    
    const btn = document.getElementById('validate-btn');
    btn.disabled = true;
    btn.innerHTML = '<svg class="spin" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/></svg> Validating...';
    
    try {
        const result = await invoke('validate_code', { code, language });
        
        logOutput(`Scan complete: ${result.errors.length} errors, ${result.warnings.length} warnings`, 
            result.passed ? 'success' : 'error');
        
        // Add to problems
        result.errors.forEach(err => {
            err.severity = 'error';
            err.classification = 'new';
            state.watcherErrors.new.push(err);
        });
        
        result.warnings.forEach(warn => {
            warn.severity = 'warning';
            warn.classification = 'new';
            state.watcherErrors.new.push(warn);
        });
        
        renderProblems();
        updateStatusBar();
        
        // Switch to problems
        document.querySelector('[data-panel="problems"]').click();
        openProblemsPanel();
        
    } catch (e) {
        logOutput('Validation failed: ' + e, 'error');
    } finally {
        btn.disabled = false;
        btn.innerHTML = '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/></svg> Validate Code';
    }
}

// -----------------------------------------------------------------------------
// SETTINGS
// -----------------------------------------------------------------------------

function setupSettingsNav() {
    document.querySelectorAll('.setting-nav-btn').forEach(btn => {
        btn.addEventListener('click', () => {
            document.querySelectorAll('.setting-nav-btn').forEach(b => b.classList.remove('active'));
            btn.classList.add('active');
            renderSettingsSection(btn.dataset.section);
        });
    });
}

function renderSettingsSection(section) {
    const content = document.getElementById('settings-content');
    
    switch (section) {
        case 'languages':
            renderLanguagesSettings(content);
            break;
        case 'watcher':
            renderWatcherSettings(content);
            break;
        case 'mode':
            renderModeSettings(content);
            break;
    }
}

function renderLanguagesSettings(content) {
    const languages = [
        'rust', 'python', 'javascript', 'typescript',
        'cpp', 'c', 'go', 'java', 'lua',
        'glsl', 'css', 'html', 'json', 'yaml', 'lex'
    ];

    content.innerHTML = `
        <h3>LANGUAGES</h3>
        <p class="hint">Select languages to validate</p>
        <div class="checkbox-grid">
            ${languages.map(lang => `
                <label class="checkbox-item">
                    <input type="checkbox" ${state.config.languages.includes(lang) ? 'checked' : ''} data-lang="${lang}">
                    <span>${lang}</span>
                </label>
            `).join('')}
        </div>
        <button class="btn btn-primary" id="save-languages">Save</button>
    `;
    
    document.getElementById('save-languages').addEventListener('click', async () => {
        const checked = document.querySelectorAll('#settings-content input:checked');
        state.config.languages = Array.from(checked).map(el => el.dataset.lang);
        await invoke('save_config', { config: state.config });
        logOutput('Languages saved', 'success');
    });
}

function renderWatcherSettings(content) {
    content.innerHTML = `
        <h3>WATCHER</h3>
        <div class="setting-row">
            <label>Status</label>
            <div class="watcher-status ${state.watcherRunning ? 'running' : ''}">
                <span class="status-dot ${state.watcherRunning ? 'active' : ''}"></span>
                <span>${state.watcherRunning ? 'Running' : 'Stopped'}</span>
            </div>
        </div>
        <button class="btn ${state.watcherRunning ? 'btn-danger' : 'btn-primary'}" id="toggle-watcher">
            ${state.watcherRunning ? 'Stop Watcher' : 'Start Watcher'}
        </button>
    `;
    
    document.getElementById('toggle-watcher').addEventListener('click', async () => {
        try {
            if (state.watcherRunning) {
                await invoke('stop_watcher');
                state.watcherRunning = false;
                logOutput('Watcher stopped', 'muted');
            } else {
                await invoke('start_watcher');
                state.watcherRunning = true;
                logOutput('Watcher started', 'success');
            }
            updateStatusBar();
            renderWatcherSettings(content);
        } catch (e) {
            logOutput('Failed: ' + e, 'error');
        }
    });
}

function renderModeSettings(content) {
    content.innerHTML = `
        <h3>VALIDATION MODE</h3>
        <div class="mode-cards">
            <label class="mode-card ${state.config.severity === 'basic' ? 'active' : ''}">
                <input type="radio" name="severity" value="basic" ${state.config.severity === 'basic' ? 'checked' : ''}>
                <div class="mode-icon">⚡</div>
                <div class="mode-title">Basic</div>
                <div class="mode-desc">Critical errors only</div>
            </label>
            <label class="mode-card ${state.config.severity === 'standard' ? 'active' : ''}">
                <input type="radio" name="severity" value="standard" ${state.config.severity === 'standard' ? 'checked' : ''}>
                <div class="mode-icon">✓</div>
                <div class="mode-title">Standard</div>
                <div class="mode-desc">Errors + Warnings</div>
            </label>
            <label class="mode-card ${state.config.severity === 'strict' ? 'active' : ''}">
                <input type="radio" name="severity" value="strict" ${state.config.severity === 'strict' ? 'checked' : ''}>
                <div class="mode-icon">🔒</div>
                <div class="mode-title">Strict</div>
                <div class="mode-desc">All issues</div>
            </label>
        </div>
        <div class="setting-row">
            <label>
                <input type="checkbox" id="auto-fix" ${state.config.autoFix ? 'checked' : ''}>
                Auto-fix when possible
            </label>
        </div>
        <button class="btn btn-primary" id="save-mode">Save</button>
    `;
    
    document.getElementById('save-mode').addEventListener('click', async () => {
        const severity = document.querySelector('input[name="severity"]:checked').value;
        const autoFix = document.getElementById('auto-fix').checked;
        state.config.severity = severity;
        state.config.autoFix = autoFix;
        await invoke('save_config', { config: state.config });
        logOutput('Mode saved', 'success');
    });
}

// -----------------------------------------------------------------------------
// TAURI EVENTS
// -----------------------------------------------------------------------------

function setupTauriListeners() {
    listen('validation:error', (event) => {
        const error = event.payload;
        const isNew = error.classification === 'new' || error.classification === undefined;
        
        if (!error.severity) error.severity = 'error';
        
        if (isNew) {
            state.watcherErrors.new.push(error);
        } else {
            state.watcherErrors.prex.push(error);
        }
        
        logOutput(`${error.file}: ${error.message}`, 'error');
        renderProblems();
        updateStatusBar();
    });
    
    listen('validation:fix-proposed', (event) => {
        showFixProposal(event.payload);
    });
}

function showFixProposal(fix) {
    const panel = document.createElement('div');
    panel.className = 'fix-proposal-panel';
    panel.innerHTML = `
        <div class="fix-header">
            <span class="fix-icon">🔧</span>
            <span class="fix-title">${fix.description}</span>
            <span class="fix-confidence">${Math.round(fix.confidence * 100)}%</span>
        </div>
        <div class="fix-diff">
            <div class="diff-line old">- ${escapeHtml(fix.original)}</div>
            <div class="diff-line new">+ ${escapeHtml(fix.replacement)}</div>
        </div>
        <div class="fix-actions">
            <button class="btn btn-apply">Apply</button>
            <button class="btn btn-secondary">Dismiss</button>
        </div>
    `;
    
    document.querySelector('.editor-area').appendChild(panel);
    
    panel.querySelector('.btn-apply').addEventListener('click', async () => {
        await invoke('apply_fix', { errorId: fix.error_id, file: fix.file });
        panel.remove();
        if (state.activeFile === fix.file) await openFile(state.activeFile);
        logOutput('Fix applied', 'success');
    });
    
    panel.querySelector('.btn-secondary').addEventListener('click', () => panel.remove());
}

// -----------------------------------------------------------------------------
// PROBLEMS RENDERING
// -----------------------------------------------------------------------------

function renderProblems(filter = 'all') {
    const newErrors = state.watcherErrors.new;
    const prexErrors = state.watcherErrors.prex;
    
    document.getElementById('new-errors-count').textContent = newErrors.length;
    document.getElementById('prex-errors-count').textContent = prexErrors.length;
    
    // Filter
    const filterFn = filter === 'all' ? () => true : e => e.severity === filter;
    
    // New errors
    const newList = document.getElementById('new-errors-list');
    const filteredNew = newErrors.filter(filterFn);
    newList.innerHTML = filteredNew.length ? 
        filteredNew.map(e => createProblemItem(e)).join('') :
        '<div class="empty-category">No new issues</div>';
    
    // Pre-existing
    const prexList = document.getElementById('prex-errors-list');
    const filteredPrex = prexErrors.filter(filterFn);
    prexList.innerHTML = filteredPrex.length ?
        filteredPrex.map(e => createProblemItem(e)).join('') :
        '<div class="empty-category">No pre-existing issues</div>';
    
    setupProblemItemEvents();
    
    // Update code view
    if (state.activeFile && state.activeFileContent) {
        renderCodeContent(state.activeFileContent, document.getElementById('status-lang').textContent);
    }
}

function createProblemItem(error) {
    const isError = error.severity === 'error';
    return `
        <div class="problem-item ${error.severity}" data-id="${error.id}" data-file="${error.file || ''}" data-line="${error.line || 0}">
            <span class="icon ${error.severity}">${isError ? '●' : '○'}</span>
            <span class="message">${error.message}</span>
            <span class="location">${error.file?.split(/[\\/]/).pop() || ''}:${error.line || '?'}</span>
            ${error.suggestion ? '<button class="fix-btn">Fix</button>' : ''}
        </div>
    `;
}

function setupProblemItemEvents() {
    document.querySelectorAll('.problem-item').forEach(item => {
        item.addEventListener('click', () => {
            const file = item.dataset.file;
            const line = parseInt(item.dataset.line);
            
            if (file) {
                openFile(file).then(() => {
                    const codeLine = document.querySelector(`.code-line[data-line="${line}"]`);
                    codeLine?.scrollIntoView({ behavior: 'smooth', block: 'center' });
                });
            }
        });
        
        item.querySelector('.fix-btn')?.addEventListener('click', async (e) => {
            e.stopPropagation();
            await invoke('apply_fix', { errorId: item.dataset.id, file: item.dataset.file });
            if (state.activeFile) await openFile(state.activeFile);
        });
    });
}

// -----------------------------------------------------------------------------
// UTILITIES
// -----------------------------------------------------------------------------

async function loadConfig() {
    try {
        const config = await invoke('get_config');
        state.config = { ...state.config, ...config };
    } catch (e) {
        console.error('Failed to load config:', e);
    }
}

async function updateStatus() {
    try {
        const status = await invoke('get_status');
        state.watcherRunning = status.watcher_running;
    } catch (e) {
        console.error('Failed to get status:', e);
    }
}

function logOutput(msg, type = 'muted') {
    const log = document.getElementById('output-log');
    const time = new Date().toLocaleTimeString('en-US', { hour12: false });
    
    const div = document.createElement('div');
    div.className = `log-line ${type}`;
    div.textContent = `[${time}] ${msg}`;
    
    log.appendChild(div);
    log.scrollTop = log.scrollHeight;
}

// Boot
window.addEventListener('DOMContentLoaded', init);
