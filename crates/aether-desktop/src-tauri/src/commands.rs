//! Tauri commands for frontend-backend communication

use serde::{Deserialize, Serialize};
use tauri::State;
use std::fs;
use std::path::PathBuf;

use crate::state::AppState;

/// Validation result for frontend
#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationResult {
    pub passed: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationError>,
    pub code_blocks: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationError {
    pub id: String,
    pub severity: String,
    pub message: String,
    pub line: Option<usize>,
    pub suggestion: Option<String>,
}

/// Configuration for the app
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AppConfiguration {
    pub languages: Vec<String>,
    pub severity: SeverityLevel,
    pub auto_fix: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum SeverityLevel {
    Basic,
    Standard,
    Strict,
}

impl Default for AppConfiguration {
    fn default() -> Self {
        Self {
            languages: vec!["rust".to_string(), "python".to_string()],
            severity: SeverityLevel::Standard,
            auto_fix: false,
        }
    }
}

// ============================================================================
// WATCHER COMMANDS - Hybrid Validation System
// ============================================================================
#[tauri::command]
pub async fn validate_code(
    code: String,
    language: String,
    _state: State<'_, AppState>,
) -> Result<ValidationResult, String> {
    use aether_validation::{ValidationPipeline, ValidationContext, layers::{SyntaxLayer, ASTLayer, LogicLayer, SecurityLayer}};

    let pipeline = ValidationPipeline::new()
        .add_layer(SyntaxLayer::new())
        .add_layer(ASTLayer::new())
        .add_layer(LogicLayer::new())
        .add_layer(SecurityLayer::new());

    let ctx = ValidationContext::for_file("snippet.ai", code.clone(), language);
    let result = pipeline.execute(&ctx).await;

    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    for (_, layer_result) in &result.results {
        for v in &layer_result.violations {
            let err = ValidationError {
                id: v.id.clone(),
                severity: format!("{:?}", v.severity).to_lowercase(),
                message: v.message.clone(),
                line: v.span.map(|s| s.line),
                suggestion: v.suggestion.clone(),
            };

            match v.severity {
                aether_validation::Severity::Error => errors.push(err),
                _ => warnings.push(err),
            }
        }
    }

    Ok(ValidationResult {
        passed: result.all_passed(),
        errors,
        warnings,
        code_blocks: 1,
    })
}

/// Get current configuration
#[tauri::command]
pub async fn get_config(state: State<'_, AppState>) -> Result<AppConfiguration, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    Ok(config.clone())
}

/// Save configuration
#[tauri::command]
pub async fn save_config(
    config: AppConfiguration,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // Save to memory
    {
        let mut current = state.config.lock().map_err(|e| e.to_string())?;
        *current = config.clone();
    }

    // Save to disk
    crate::config::save_config(&config).map_err(|e| e.to_string())?;

    Ok(())
}

/// Get system status
#[tauri::command]
pub async fn get_status(
    state: State<'_, AppState>,
) -> Result<SystemStatus, String> {
    use std::sync::atomic::Ordering;
    let watcher_running = state.watcher_running.load(Ordering::SeqCst);
    
    Ok(SystemStatus {
        version: env!("CARGO_PKG_VERSION").to_string(),
        watcher_running,
    })
}

#[derive(Debug, Serialize)]
pub struct SystemStatus {
    pub version: String,
    pub watcher_running: bool,
}

// ============================================================================
// WATCHER COMMANDS - Hybrid Validation System
// ============================================================================

use tokio::sync::mpsc;
use tauri::Emitter;
use crate::watcher::{FileWatcher, WatchConfig, WatchEvent};
use crate::validation_service::{ValidationService, ClassifiedError};
use crate::auto_fix::AutoFixService;

/// Start file watcher for hybrid validation
#[tauri::command]
pub async fn start_watcher(
    languages: Option<Vec<String>>,
    workspace: Option<String>,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<WatcherStatus, String> {
    use std::sync::atomic::Ordering;

    // Check if already running
    if state.watcher_running.load(Ordering::SeqCst) {
        return Err("Watcher already running".to_string());
    }

    // Use defaults from config if not provided
    let languages = match languages {
        Some(l) if !l.is_empty() => l,
        _ => {
            let config = state.config.lock().map_err(|e| e.to_string())?;
            if config.languages.is_empty() {
                vec!["rust".to_string()]
            } else {
                config.languages.clone()
            }
        }
    };

    let workspace_path = match workspace {
        Some(p) => PathBuf::from(p),
        None => std::env::current_dir().map_err(|e| e.to_string())?,
    };

    // Clone values for the return BEFORE any spawn
    let return_languages = languages.clone();
    let return_workspace = workspace_path.display().to_string();

    // Clone watcher_running flag for use in spawned tasks
    let watcher_running = state.watcher_running.clone();

    let config = WatchConfig {
        workspace: workspace_path,
        languages,
    };

    // Create channel for watch events
    let (tx, mut rx) = mpsc::channel(100);

    // Mark as running
    watcher_running.store(true, Ordering::SeqCst);

    // Clone for use inside spawn
    let running_flag = watcher_running.clone();
    let config_clone = config.clone();

    // Spawn watcher in background
    tokio::spawn(async move {
        let mut watcher = FileWatcher::new(&config_clone);

        if let Err(e) = watcher.start(tx).await {
            tracing::error!(error = ?e, "Failed to start watcher");
            running_flag.store(false, Ordering::SeqCst);
            return;
        }

        // Keep watcher alive - but the watcher is now blocking
        // We need to monitor the running flag in a separate way
        // For simplicity, just keep the watcher running until stop is called
        loop {
            if !running_flag.load(Ordering::SeqCst) {
                watcher.stop();
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    });

    // Clone for validation loop
    let validation_running = watcher_running.clone();
    let app_handle = app.clone();

    // Process validation events
    tokio::spawn(async move {
        let validation_service = ValidationService::default_service();

        while validation_running.load(Ordering::SeqCst) {
            tokio::select! {
                Some(event) = rx.recv() => {
                    match event {
                        WatchEvent::FileModified { path } | WatchEvent::FileCreated { path } => {
                            // Validate file
                            match validation_service.validate_file(&path).await {
                                Ok(result) => {
                                    // Emit errors to frontend
                                    for error in &result.errors {
                                        if let Err(e) = app_handle.emit("validation:error", error) {
                                            tracing::warn!(error = ?e, "Failed to emit error event");
                                        }
                                    }
                                }
                                Err(e) => {
                                    tracing::warn!(error = ?e, ?path, "Validation failed");
                                }
                            }
                        }
                        WatchEvent::Error { message } => {
                            tracing::error!(message, "Watcher error");
                        }
                        _ => {}
                    }
                }
                _ = tokio::time::sleep(std::time::Duration::from_secs(1)) => {
                    // Periodic check
                }
            }
        }
    });

    Ok(WatcherStatus {
        running: true,
        languages: return_languages,
        workspace: return_workspace,
    })
}

/// Stop file watcher
#[tauri::command]
pub async fn stop_watcher(
    state: State<'_, AppState>,
) -> Result<(), String> {
    use std::sync::atomic::Ordering;
    state.watcher_running.store(false, Ordering::SeqCst);
    Ok(())
}

/// Get current errors from watcher
#[tauri::command]
pub async fn get_errors(
    state: State<'_, AppState>,
) -> Result<Vec<ClassifiedError>, String> {
    let errors = state.watcher_errors.lock().map_err(|e| e.to_string())?;
    Ok(errors.clone())
}

/// Clear all errors
#[tauri::command]
pub async fn clear_errors(
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut errors = state.watcher_errors.lock().map_err(|e| e.to_string())?;
    errors.clear();
    Ok(())
}

/// Request auto-fix for an error
#[tauri::command]
pub async fn request_fix(
    error_id: String,
    file: String,
    line: usize,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let auto_fix = AutoFixService::new();
    let path = PathBuf::from(&file);
    
    // Detect language from extension
    let language = path.extension()
        .and_then(|e| e.to_str())
        .map(|e| match e {
            "rs" => "rust",
            "cpp" | "hpp" | "cc" => "cpp",
            "c" | "h" => "c",
            "py" => "python",
            "js" | "jsx" => "javascript",
            "ts" | "tsx" => "typescript",
            "go" => "go",
            _ => "unknown",
        })
        .unwrap_or("unknown");

    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    if let Some(proposal) = auto_fix.propose_fix(&error_id, language, line, &content, &file) {
        // Store pending fix
        {
            let mut pending = state.pending_fixes.lock().map_err(|e| e.to_string())?;
            pending.push(proposal.clone());
        }

        // Emit to frontend
        app.emit("validation:fix-proposed", &proposal)
            .map_err(|e: tauri::Error| e.to_string())?;
    }

    Ok(())
}

/// Apply a pending fix
#[tauri::command]
pub async fn apply_fix(
    error_id: String,
    file: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let auto_fix = AutoFixService::new();
    let path = PathBuf::from(&file);

    // Find pending fix
    let fix = {
        let pending = state.pending_fixes.lock().map_err(|e| e.to_string())?;
        pending.iter()
            .find(|f| f.error_id == error_id && f.file == file)
            .cloned()
    };

    if let Some(fix) = fix {
        auto_fix.apply_fix(&path, &fix)
            .map_err(|e| e.to_string())?;

        // Remove from pending
        {
            let mut pending = state.pending_fixes.lock().map_err(|e| e.to_string())?;
            pending.retain(|f| !(f.error_id == error_id && f.file == file));
        }

        Ok(())
    } else {
        Err("No pending fix found".to_string())
    }
}

/// Get watcher status
#[tauri::command]
pub async fn get_watcher_status(
    state: State<'_, AppState>,
) -> Result<WatcherStatus, String> {
    use std::sync::atomic::Ordering;
    
    let running = state.watcher_running.load(Ordering::SeqCst);
    
    let (languages, workspace) = {
        let config = state.config.lock().map_err(|e| e.to_string())?;
        (config.languages.clone(), std::env::current_dir().unwrap_or_default().display().to_string())
    };

    Ok(WatcherStatus {
        running,
        languages,
        workspace,
    })
}

/// Watcher status response
#[derive(Debug, Serialize)]
pub struct WatcherStatus {
    pub running: bool,
    pub languages: Vec<String>,
    pub workspace: String,
}

// ============================================================================
// FILESYSTEM COMMANDS - File tree and file operations
// ============================================================================

/// Directory entry for file tree
#[derive(Debug, Serialize)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub extension: Option<String>,
    pub has_errors: bool,
    pub error_count: usize,
}

/// List directory contents
#[tauri::command]
pub async fn list_directory(
    path: Option<String>,
) -> Result<Vec<FileEntry>, String> {
    let dir_path = match path {
        Some(p) => PathBuf::from(p),
        None => std::env::current_dir().map_err(|e| e.to_string())?,
    };

    let entries = fs::read_dir(&dir_path)
        .map_err(|e| format!("Failed to read directory: {}", e))?;

    let mut result = Vec::new();
    
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("?")
            .to_string();

        // Skip hidden files and common ignore patterns
        if name.starts_with('.') || name == "node_modules" || name == "target" {
            continue;
        }

        let is_dir = path.is_dir();
        let extension = path.extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_string());

        result.push(FileEntry {
            name,
            path: path.display().to_string(),
            is_dir,
            extension,
            has_errors: false, // Will be updated by frontend
            error_count: 0,
        });
    }

    // Sort: directories first, then by name
    result.sort_by(|a, b| {
        match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        }
    });

    Ok(result)
}

/// Read file contents
#[tauri::command]
pub async fn read_file(path: String) -> Result<FileContent, String> {
    let file_path = PathBuf::from(&path);
    
    let content = fs::read_to_string(&file_path)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    let language = file_path.extension()
        .and_then(|e| e.to_str())
        .map(|e| match e.to_lowercase().as_str() {
            "rs" => "rust",
            "cpp" | "cc" | "cxx" => "cpp",
            "hpp" | "hxx" => "cpp",
            "c" => "c",
            "h" => "c",
            "py" => "python",
            "js" | "jsx" => "javascript",
            "ts" | "tsx" => "typescript",
            "go" => "go",
            "java" => "java",
            "lua" => "lua",
            "glsl" | "frag" | "vert" => "glsl",
            "lex" => "lex",
            _ => "text",
        })
        .unwrap_or("text");

    Ok(FileContent {
        path: path.clone(),
        content,
        language: language.to_string(),
    })
}

#[derive(Debug, Serialize)]
pub struct FileContent {
    pub path: String,
    pub content: String,
    pub language: String,
}

/// Get workspace root directory
#[tauri::command]
pub async fn get_workspace_root() -> Result<String, String> {
    std::env::current_dir()
        .map(|p| p.display().to_string())
        .map_err(|e| e.to_string())
}

/// Save file contents
#[tauri::command]
pub async fn save_file(path: String, content: String) -> Result<(), String> {
    fs::write(&path, content)
        .map_err(|e| format!("Failed to save file: {}", e))
}
