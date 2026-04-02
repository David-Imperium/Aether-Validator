//! Application state management

use std::sync::Mutex;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use crate::commands::AppConfiguration;
use crate::validation_service::{ClassifiedError, ValidationResult};
use crate::auto_fix::FixProposal;

/// Global application state (MCP-driven mode)
pub struct AppState {
    pub config: Mutex<AppConfiguration>,
    
    // Watcher state
    pub watcher_running: Arc<AtomicBool>,
    pub watcher_errors: Mutex<Vec<ClassifiedError>>,
    pub validation_results: Mutex<Vec<ValidationResult>>,
    pub pending_fixes: Mutex<Vec<FixProposal>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            config: Mutex::new(crate::config::load_config()),
            watcher_running: Arc::new(AtomicBool::new(false)),
            watcher_errors: Mutex::new(Vec::new()),
            validation_results: Mutex::new(Vec::new()),
            pending_fixes: Mutex::new(Vec::new()),
        }
    }
}
