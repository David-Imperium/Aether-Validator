//! Memory Screen - Browse Aether Memory

use std::path::PathBuf;

pub struct MemoryScreen {
    /// Project root
    pub project_root: PathBuf,
    /// Selected entry
    pub selected: usize,
    /// Scroll offset
    pub scroll: usize,
    /// Memory entries (cached)
    pub entries: Vec<MemoryEntry>,
    /// Search query
    pub search_query: String,
    /// Is searching
    pub searching: bool,
}

/// A memory entry for display
#[derive(Debug, Clone)]
pub struct MemoryEntry {
    pub id: String,
    pub kind: MemoryKind,
    pub title: String,
    pub tags: Vec<String>,
    pub created: String,
}

impl MemoryEntry {
    pub fn kind_str(&self) -> &'static str {
        self.kind.as_str()
    }
}

#[derive(Debug, Clone)]
pub enum MemoryKind {
    /// Code graph entry
    CodeGraph,
    /// Decision log entry
    Decision,
    /// Validation state entry
    Validation,
    /// Drift snapshot
    Drift,
    /// Learned config
    LearnedConfig,
}

impl MemoryKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            MemoryKind::CodeGraph => "graph",
            MemoryKind::Decision => "decision",
            MemoryKind::Validation => "validation",
            MemoryKind::Drift => "drift",
            MemoryKind::LearnedConfig => "config",
        }
    }
}

impl MemoryScreen {
    pub fn new(project_root: PathBuf) -> Self {
        Self {
            project_root,
            selected: 0,
            scroll: 0,
            entries: Vec::new(),
            search_query: String::new(),
            searching: false,
        }
    }

    /// Load memory entries (placeholder - would connect to actual memory store)
    pub fn load_entries(&mut self) -> anyhow::Result<()> {
        // Placeholder entries
        self.entries = vec![
            MemoryEntry {
                id: "1".to_string(),
                kind: MemoryKind::Decision,
                title: "Accepted UNWRAP001 in main.rs:42".to_string(),
                tags: vec!["acceptance".to_string()],
                created: "2025-03-15".to_string(),
            },
            MemoryEntry {
                id: "2".to_string(),
                kind: MemoryKind::LearnedConfig,
                title: "Threshold adjusted: complexity.max_cyclomatic = 15".to_string(),
                tags: vec!["threshold".to_string(), "learned".to_string()],
                created: "2025-03-14".to_string(),
            },
        ];
        Ok(())
    }
}
