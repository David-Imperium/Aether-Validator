// Aether Desktop - Frontend Logic

const { invoke } = window.__TAURI__.core;

// State
let config = {
    tools: [],
    languages: ['rust', 'python'],
    severity: 'standard',
    proxyPort: 8080,
    watcherPaths: ['src'],
    autoFix: false,
    mode: 'both' // derived from tools
};

// Tool to method mapping
const toolMethods = {
    'droid': 'watcher',
    'claude-code': 'watcher',
    'gemini-cli': 'watcher',
    'cursor': 'watcher',
    'copilot': 'watcher',
    'aider': 'watcher',
    'windsurf': 'watcher',
    'cline': 'watcher',
    'codex': 'watcher',
    'ollama': 'watcher',
    'lm-studio': 'watcher',
    'openai-api': 'proxy',
    'anthropic-api': 'proxy',
    'other': 'both'
};

// DOM Elements
const elements = {
    // Navigation
    navTabs: document.querySelectorAll('.nav-tab'),
    tabContents: document.querySelectorAll('.tab-content'),

    // Wizard
    toolCheckboxes: document.querySelectorAll('input[name="tool"]'),
    langRust: document.getElementById('lang-rust'),
    langPython: document.getElementById('lang-python'),
    langJs: document.getElementById('lang-js'),
    langTs: document.getElementById('lang-ts'),
    langCpp: document.getElementById('lang-cpp'),
    langGo: document.getElementById('lang-go'),
    severityRadios: document.querySelectorAll('input[name="severity"]'),
    autoFix: document.getElementById('auto-fix'),
    applyConfig: document.getElementById('apply-config'),

    // Validate
    languageSelect: document.getElementById('language-select'),
    codeInput: document.getElementById('code-input'),
    validateBtn: document.getElementById('validate-btn'),
    validationResults: document.getElementById('validation-results'),
    resultStatus: document.getElementById('result-status'),
    resultCount: document.getElementById('result-count'),
    errorList: document.getElementById('error-list'),

    // Settings
    proxyPort: document.getElementById('proxy-port'),
    toggleProxy: document.getElementById('toggle-proxy'),
    proxyStatus: document.getElementById('proxy-status'),
    watcherPaths: document.getElementById('watcher-paths'),
    toggleWatcher: document.getElementById('toggle-watcher'),
    watcherStatus: document.getElementById('watcher-status'),

    // Status
    proxyStatusCard: document.getElementById('proxy-status-card'),
    watcherStatusCard: document.getElementById('watcher-status-card'),
    versionDisplay: document.getElementById('version-display'),
    activityLog: document.getElementById('activity-log')
};

// Initialize
async function init() {
    setupNavigation();
    setupWizard();
    setupValidation();
    setupSettings();
    await loadConfig();
    await updateStatus();
}

// Navigation
function setupNavigation() {
    elements.navTabs.forEach(tab => {
        tab.addEventListener('click', () => {
            const targetTab = tab.dataset.tab;

            // Update active tab
            elements.navTabs.forEach(t => t.classList.remove('active'));
            tab.classList.add('active');

            // Show target content
            elements.tabContents.forEach(content => {
                content.classList.remove('active');
                if (content.id === `${targetTab}-tab`) {
                    content.classList.add('active');
                }
            });
        });
    });
}

// Wizard
function setupWizard() {
    elements.applyConfig.addEventListener('click', saveConfig);

    // Tool selection
    elements.toolCheckboxes.forEach(cb => {
        cb.addEventListener('change', updateTools);
    });

    // Languages
    elements.langRust.addEventListener('change', updateLanguages);
    elements.langPython.addEventListener('change', updateLanguages);
    elements.langJs.addEventListener('change', updateLanguages);
    elements.langTs.addEventListener('change', updateLanguages);
    if (elements.langCpp) elements.langCpp.addEventListener('change', updateLanguages);
    if (elements.langGo) elements.langGo.addEventListener('change', updateLanguages);

    // Severity
    elements.severityRadios.forEach(radio => {
        radio.addEventListener('change', () => {
            config.severity = radio.value;
        });
    });

    // Auto-fix
    elements.autoFix.addEventListener('change', () => {
        config.autoFix = elements.autoFix.checked;
    });
}

function updateTools() {
    config.tools = [];
    let needsProxy = false;
    let needsWatcher = false;

    elements.toolCheckboxes.forEach(cb => {
        if (cb.checked) {
            config.tools.push(cb.value);
            const method = toolMethods[cb.value] || 'watcher';
            if (method === 'proxy' || method === 'both') needsProxy = true;
            if (method === 'watcher' || method === 'both') needsWatcher = true;
        }
    });

    // Derive mode from tools
    if (needsProxy && needsWatcher) {
        config.mode = 'both';
    } else if (needsProxy) {
        config.mode = 'proxy';
    } else if (needsWatcher) {
        config.mode = 'watcher';
    } else {
        config.mode = 'none';
    }
}

function updateLanguages() {
    config.languages = [];
    if (elements.langRust.checked) config.languages.push('rust');
    if (elements.langPython.checked) config.languages.push('python');
    if (elements.langJs.checked) config.languages.push('javascript');
    if (elements.langTs.checked) config.languages.push('typescript');
    if (elements.langCpp && elements.langCpp.checked) config.languages.push('cpp');
    if (elements.langGo && elements.langGo.checked) config.languages.push('go');
}

async function loadConfig() {
    try {
        const loaded = await invoke('get_config');
        config = { ...config, ...loaded };
        applyConfigToUI();
        addLog('Configuration loaded');
    } catch (e) {
        console.error('Failed to load config:', e);
    }
}

function applyConfigToUI() {
    // Tools - check boxes based on config.tools
    elements.toolCheckboxes.forEach(cb => {
        cb.checked = config.tools && config.tools.includes(cb.value);
    });

    // Languages
    elements.langRust.checked = config.languages && config.languages.includes('rust');
    elements.langPython.checked = config.languages && config.languages.includes('python');
    elements.langJs.checked = config.languages && config.languages.includes('javascript');
    elements.langTs.checked = config.languages && config.languages.includes('typescript');
    if (elements.langCpp) elements.langCpp.checked = config.languages && config.languages.includes('cpp');
    if (elements.langGo) elements.langGo.checked = config.languages && config.languages.includes('go');

    // Severity
    elements.severityRadios.forEach(radio => {
        radio.checked = radio.value === config.severity;
    });

    // Auto-fix
    elements.autoFix.checked = config.autoFix;

    // Settings
    if (elements.proxyPort) elements.proxyPort.value = config.proxyPort || 8080;
    if (elements.watcherPaths) elements.watcherPaths.value = (config.watcherPaths || ['src']).join(',');
}

async function saveConfig() {
    updateTools();
    updateLanguages();

    config.proxyPort = parseInt(elements.proxyPort?.value || '8080');
    config.watcherPaths = elements.watcherPaths?.value?.split(',').map(s => s.trim()) || ['src'];

    try {
        await invoke('save_config', { config });
        addLog('Configuration saved');

        // Start proxy/watcher based on mode
        if (config.mode === 'proxy' || config.mode === 'both') {
            try {
                await invoke('start_proxy', { port: config.proxyPort });
                proxyRunning = true;
                addLog(`Proxy started on port ${config.proxyPort}`);
            } catch (e) {
                addLog('Failed to start proxy: ' + e);
            }
        }

        if (config.mode === 'watcher' || config.mode === 'both') {
            try {
                await invoke('start_watcher');
                watcherRunning = true;
                addLog('Watcher started');
            } catch (e) {
                addLog('Failed to start watcher: ' + e);
            }
        }

        // Show success feedback and switch to Status tab
        elements.applyConfig.textContent = 'Started!';
        setTimeout(() => {
            elements.applyConfig.textContent = 'Start Validation';
            // Switch to Status tab
            document.querySelector('[data-tab="status"]')?.click();
        }, 1000);
    } catch (e) {
        console.error('Failed to save config:', e);
        addLog('Failed to save configuration');
    }
}

// Validation
function setupValidation() {
    elements.validateBtn.addEventListener('click', validateCode);

    // Also validate on Ctrl+Enter
    elements.codeInput.addEventListener('keydown', (e) => {
        if (e.ctrlKey && e.key === 'Enter') {
            validateCode();
        }
    });
}

async function validateCode() {
    const code = elements.codeInput.value;
    const language = elements.languageSelect.value;

    if (!code.trim()) {
        showResults({ passed: true, errors: [], warnings: [], code_blocks: 0 });
        return;
    }

    elements.validateBtn.disabled = true;
    elements.validateBtn.textContent = 'Validating...';

    try {
        const result = await invoke('validate_code', { code, language });
        showResults(result);
        addLog(`Validated ${result.code_blocks} code block(s) - ${result.passed ? 'passed' : 'failed'}`);
    } catch (e) {
        console.error('Validation failed:', e);
        addLog('Validation error: ' + e);
    } finally {
        elements.validateBtn.disabled = false;
        elements.validateBtn.textContent = 'Validate';
    }
}

function showResults(result) {
    elements.validationResults.classList.remove('hidden');

    const hasErrors = result.errors.length > 0;
    const hasWarnings = result.warnings.length > 0;

    elements.resultStatus.textContent = result.passed ? 'PASSED' : 'FAILED';
    elements.resultStatus.className = result.passed ? 'passed' : 'failed';

    const totalIssues = result.errors.length + result.warnings.length;
    elements.resultCount.textContent = totalIssues > 0
        ? `${totalIssues} issue(s) found`
        : 'No issues';

    // Build error list
    elements.errorList.innerHTML = '';

    result.errors.forEach(err => {
        elements.errorList.appendChild(createErrorItem(err, 'error'));
    });

    result.warnings.forEach(err => {
        elements.errorList.appendChild(createErrorItem(err, 'warning'));
    });

    if (totalIssues === 0) {
        elements.errorList.innerHTML = '<div class="error-item" style="border-left-color: var(--success)">Code looks good!</div>';
    }
}

function createErrorItem(err, type) {
    const div = document.createElement('div');
    div.className = `error-item ${type}`;

    div.innerHTML = `
        <span class="error-id">${err.id}</span>
        ${err.line ? `<span class="error-line">at line ${err.line}</span>` : ''}
        <div class="error-message">${err.message}</div>
        ${err.suggestion ? `<div class="error-suggestion">💡 ${err.suggestion}</div>` : ''}
    `;

    return div;
}

// Settings
function setupSettings() {
    if (elements.toggleProxy) {
        elements.toggleProxy.addEventListener('click', toggleProxy);
    }
    if (elements.toggleWatcher) {
        elements.toggleWatcher.addEventListener('click', toggleWatcher);
    }
}

let proxyRunning = false;
let watcherRunning = false;

async function toggleProxy() {
    try {
        if (proxyRunning) {
            await invoke('stop_proxy');
            proxyRunning = false;
            elements.toggleProxy.textContent = 'Start Proxy';
            elements.proxyStatus.textContent = 'Stopped';
            elements.proxyStatus.className = 'status-badge offline';
            addLog('Proxy stopped');
        } else {
            const port = parseInt(elements.proxyPort?.value || '8080');
            await invoke('start_proxy', { port });
            proxyRunning = true;
            elements.toggleProxy.textContent = 'Stop Proxy';
            elements.proxyStatus.textContent = 'Running';
            elements.proxyStatus.className = 'status-badge online';
            addLog(`Proxy started on port ${port}`);
        }
    } catch (e) {
        console.error('Proxy toggle failed:', e);
        addLog('Failed to toggle proxy: ' + e);
    }
}

async function toggleWatcher() {
    try {
        if (watcherRunning) {
            await invoke('stop_watcher');
            watcherRunning = false;
            elements.toggleWatcher.textContent = 'Start Watcher';
            elements.watcherStatus.textContent = 'Stopped';
            elements.watcherStatus.className = 'status-badge offline';
            addLog('Watcher stopped');
        } else {
            await invoke('start_watcher');
            watcherRunning = true;
            elements.toggleWatcher.textContent = 'Stop Watcher';
            elements.watcherStatus.textContent = 'Running';
            elements.watcherStatus.className = 'status-badge online';
            addLog('Watcher started');
        }
    } catch (e) {
        console.error('Watcher toggle failed:', e);
        addLog('Failed to toggle watcher');
    }
}

// Status
async function updateStatus() {
    try {
        const status = await invoke('get_status');

        if (elements.versionDisplay) {
            elements.versionDisplay.textContent = status.version;
        }

        proxyRunning = status.proxy_running;
        watcherRunning = status.watcher_running;

        if (elements.proxyStatusCard) {
            elements.proxyStatusCard.textContent = proxyRunning ? 'Running' : 'Stopped';
        }
        if (elements.watcherStatusCard) {
            elements.watcherStatusCard.textContent = watcherRunning ? 'Running' : 'Stopped';
        }
    } catch (e) {
        console.error('Failed to get status:', e);
    }
}

// Activity Log
function addLog(message) {
    const entry = document.createElement('div');
    entry.className = 'log-entry';

    const time = new Date().toLocaleTimeString();
    entry.textContent = `[${time}] ${message}`;

    if (elements.activityLog) {
        elements.activityLog.insertBefore(entry, elements.activityLog.firstChild);

        // Keep only last 50 entries
        while (elements.activityLog.children.length > 50) {
            elements.activityLog.removeChild(elements.activityLog.lastChild);
        }
    }
}

// Start
document.addEventListener('DOMContentLoaded', () => {
    init();
    // Poll status every 2 seconds
    setInterval(updateStatus, 2000);
});
