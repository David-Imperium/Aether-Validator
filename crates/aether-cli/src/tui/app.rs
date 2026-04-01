//! TUI Application State

use crate::tui::config_screen::ConfigScreen;
use crate::tui::memory_screen::MemoryScreen;
use crate::tui::mcp_screen::McpScreen;

pub struct App {
    pub mode: AppMode,
    pub config_screen: ConfigScreen,
    pub memory_screen: MemoryScreen,
    pub mcp_screen: McpScreen,
    pub should_quit: bool,
    pub status: Option<String>,
    pub menu_selected: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppMode {
    Menu,
    Dashboard,
    Validate,
    Config,
    Memory,
    McpSetup,
    Help,
}

impl std::fmt::Display for AppMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppMode::Menu => write!(f, "Menu"),
            AppMode::Dashboard => write!(f, "Dashboard"),
            AppMode::Validate => write!(f, "Validate"),
            AppMode::Config => write!(f, "Config"),
            AppMode::Memory => write!(f, "Memory"),
            AppMode::McpSetup => write!(f, "MCP Setup"),
            AppMode::Help => write!(f, "Help"),
        }
    }
}

pub const MENU_ITEMS: &[(&str, &str)] = &[
    ("Dashboard", "Project overview and status"),
    ("Validate", "Run validation on files"),
    ("Configure", "Edit .aether.toml settings"),
    ("Memory", "Browse learned patterns and decisions"),
    ("MCP Setup", "Configure MCP for your platform"),
    ("Help", "Shortcuts and documentation"),
];

impl App {
    pub fn new(project_root: std::path::PathBuf) -> Self {
        Self {
            mode: AppMode::Menu,
            config_screen: ConfigScreen::new(project_root.clone()),
            memory_screen: MemoryScreen::new(project_root.clone()),
            mcp_screen: McpScreen::new(project_root),
            should_quit: false,
            status: None,
            menu_selected: 0,
        }
    }

    pub fn next_mode(&mut self) {
        self.mode = match self.mode {
            AppMode::Menu => AppMode::Dashboard,
            AppMode::Dashboard => AppMode::Validate,
            AppMode::Validate => AppMode::Config,
            AppMode::Config => AppMode::Memory,
            AppMode::Memory => AppMode::McpSetup,
            AppMode::McpSetup => AppMode::Help,
            AppMode::Help => AppMode::Menu,
        };
    }

    pub fn prev_mode(&mut self) {
        self.mode = match self.mode {
            AppMode::Menu => AppMode::Help,
            AppMode::Dashboard => AppMode::Menu,
            AppMode::Validate => AppMode::Dashboard,
            AppMode::Config => AppMode::Validate,
            AppMode::Memory => AppMode::Config,
            AppMode::McpSetup => AppMode::Memory,
            AppMode::Help => AppMode::McpSetup,
        };
    }
}
