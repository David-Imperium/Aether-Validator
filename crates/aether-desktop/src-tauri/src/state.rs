//! Application state management

use std::sync::Mutex;

use crate::commands::AppConfiguration;

/// Global application state (MCP-driven mode)
pub struct AppState {
    pub config: Mutex<AppConfiguration>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            config: Mutex::new(crate::config::load_config()),
        }
    }
}
