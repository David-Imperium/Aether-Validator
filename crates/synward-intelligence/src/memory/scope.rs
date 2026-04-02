//! Memory Scope - Global vs Project memory separation
//!
//! Synward uses a hybrid memory architecture:
//! - Global memory: ~/.synward/global/ (universal patterns, user preferences)
//! - Project memory: <project>/.synward/ (project-specific decisions, git-trackable)

use std::path::{Path, PathBuf};
use crate::error::{Error, Result};

/// Scope of memory storage
#[derive(Debug, Clone, PartialEq, Eq)]
#[derive(Default)]
pub enum MemoryScope {
    /// Global scope: ~/.synward/global/
    /// - Universal patterns (language best practices)
    /// - User preferences
    /// - Shared across all projects
    /// - Accessible by both MCP and CLI
    #[default]
    Global,
    
    /// Project scope: <project>/.synward/
    /// - Project-specific decisions
    /// - CodeGraph, DriftSnapshots
    /// - Versionable in git
    /// - CLI only
    Project {
        /// Root directory of the project
        root: PathBuf,
        /// Whether to use git integration
        use_git: bool,
    },
}

impl MemoryScope {
    /// Create a global scope
    pub fn global() -> Self {
        Self::Global
    }
    
    /// Create a project scope
    pub fn project(root: impl Into<PathBuf>, use_git: bool) -> Self {
        Self::Project {
            root: root.into(),
            use_git,
        }
    }
    
    /// Check if this is global scope
    pub fn is_global(&self) -> bool {
        matches!(self, Self::Global)
    }
    
    /// Check if this is project scope
    pub fn is_project(&self) -> bool {
        matches!(self, Self::Project { .. })
    }
    
    /// Check if git integration is enabled
    pub fn has_git(&self) -> bool {
        match self {
            Self::Global => false,
            Self::Project { use_git, .. } => *use_git,
        }
    }
}


/// Path resolver for memory storage
pub struct MemoryPath {
    scope: MemoryScope,
}

impl MemoryPath {
    /// Create a new MemoryPath resolver
    pub fn new(scope: MemoryScope) -> Self {
        Self { scope }
    }
    
    /// Create for global scope
    pub fn global() -> Self {
        Self::new(MemoryScope::global())
    }
    
    /// Create for project scope
    pub fn project(root: impl Into<PathBuf>, use_git: bool) -> Self {
        Self::new(MemoryScope::project(root, use_git))
    }
    
    /// Get the base path for this scope
    pub fn base(&self) -> PathBuf {
        match &self.scope {
            MemoryScope::Global => Self::global_base(),
            MemoryScope::Project { root, .. } => Self::project_base(root),
        }
    }
    
    /// Global base path: ~/.synward/
    ///
    /// Uses home directory for user-accessible storage:
    /// - Windows: C:\Users\<user>\.synward\
    /// - Linux/macOS: ~/.synward/
    pub fn global_base() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".synward")
    }

    /// Global memory subdirectory: ~/.synward/global/
    pub fn global_memory() -> PathBuf {
        Self::global_base().join("global")
    }
    
    /// Project base path: <root>/.synward/
    pub fn project_base(project_root: &Path) -> PathBuf {
        project_root.join(".synward")
    }
    
    /// Get path to memory.toml
    pub fn memory_file(&self) -> PathBuf {
        self.base().join("memory.toml")
    }
    
    /// Get path to config.toml (project only)
    pub fn config_file(&self) -> Option<PathBuf> {
        match &self.scope {
            MemoryScope::Global => None,
            MemoryScope::Project { .. } => Some(self.base().join("config.toml")),
        }
    }
    
    /// Get path to graph.json (project only)
    pub fn graph_file(&self) -> Option<PathBuf> {
        match &self.scope {
            MemoryScope::Global => None,
            MemoryScope::Project { .. } => Some(self.base().join("graph.json")),
        }
    }
    
    /// Get path to drift directory (project only)
    pub fn drift_dir(&self) -> Option<PathBuf> {
        match &self.scope {
            MemoryScope::Global => None,
            MemoryScope::Project { .. } => Some(self.base().join("drift")),
        }
    }
    
    /// Get path to decisions directory (project only)
    pub fn decisions_dir(&self) -> Option<PathBuf> {
        match &self.scope {
            MemoryScope::Global => None,
            MemoryScope::Project { .. } => Some(self.base().join("decisions")),
        }
    }
    
    /// Get path to presets directory (global only)
    pub fn presets_dir(&self) -> Option<PathBuf> {
        match &self.scope {
            MemoryScope::Global => Some(self.base().join("presets")),
            MemoryScope::Project { .. } => None,
        }
    }
    
    /// Get path to cache directory
    pub fn cache_dir(&self) -> PathBuf {
        self.base().join("cache")
    }
    
    /// Check if project has git repository
    pub fn has_git_repo(project_root: &Path) -> bool {
        project_root.join(".git").exists()
    }
    
    /// Ensure directory structure exists
    pub fn ensure_dirs(&self) -> Result<()> {
        let base = self.base();
        std::fs::create_dir_all(&base).map_err(Error::Io)?;
        
        // Create cache dir
        std::fs::create_dir_all(self.cache_dir()).map_err(Error::Io)?;
        
        // Create scope-specific dirs
        if let Some(dir) = self.presets_dir() {
            std::fs::create_dir_all(dir).map_err(Error::Io)?;
        }
        if let Some(dir) = self.drift_dir() {
            std::fs::create_dir_all(dir).map_err(Error::Io)?;
        }
        if let Some(dir) = self.decisions_dir() {
            std::fs::create_dir_all(dir).map_err(Error::Io)?;
        }
        
        Ok(())
    }
    
    /// Generate .gitignore for project .synward/
    pub fn ensure_gitignore(&self) -> Result<()> {
        match &self.scope {
            MemoryScope::Global => Ok(()),
            MemoryScope::Project { .. } => {
                let gitignore = self.base().join(".gitignore");
                let content = r#"# Synward Memory - Git Ignore
# Temporary files
*.tmp
*.lock

# Cache directory
cache/

# Keep everything else
!*.toml
!*.json
!*.md
"#;
                std::fs::write(&gitignore, content).map_err(Error::Io)?;
                Ok(())
            }
        }
    }
    
    /// Copy README template to project .synward/
    pub fn ensure_readme(&self) -> Result<()> {
        match &self.scope {
            MemoryScope::Global => Ok(()),
            MemoryScope::Project { .. } => {
                let readme = self.base().join("README.md");
                if !readme.exists() {
                    let content = include_str!("templates/synward-readme.md");
                    std::fs::write(&readme, content).map_err(Error::Io)?;
                }
                Ok(())
            }
        }
    }
    
    /// Get the scope
    pub fn scope(&self) -> &MemoryScope {
        &self.scope
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_global_scope() {
        let scope = MemoryScope::global();
        assert!(scope.is_global());
        assert!(!scope.is_project());
        assert!(!scope.has_git());
    }
    
    #[test]
    fn test_project_scope() {
        let scope = MemoryScope::project("/tmp/project", true);
        assert!(!scope.is_global());
        assert!(scope.is_project());
        assert!(scope.has_git());
    }
    
    #[test]
    fn test_project_scope_no_git() {
        let scope = MemoryScope::project("/tmp/project", false);
        assert!(scope.is_project());
        assert!(!scope.has_git());
    }
    
    #[test]
    fn test_global_paths() {
        let path = MemoryPath::global();
        // base() returns ~/.synward/ for Global scope (NOT ~/.synward/global/)
        // Use Path::new for cross-platform compatibility
        assert!(path.base().ends_with(std::path::Path::new(".synward")));
        assert!(path.memory_file().ends_with("memory.toml"));
        assert!(path.config_file().is_none());
        assert!(path.graph_file().is_none());
        assert!(path.presets_dir().is_some());
    }
    
    #[test]
    fn test_project_paths() {
        let path = MemoryPath::project("/tmp/project", false);
        // Use Path::new for cross-platform compatibility
        assert!(path.base().ends_with(std::path::Path::new(".synward")));
        assert!(path.memory_file().ends_with("memory.toml"));
        assert!(path.config_file().is_some());
        assert!(path.graph_file().is_some());
        assert!(path.presets_dir().is_none());
    }
    
    #[test]
    fn test_has_git_repo() {
        let dir = tempdir().unwrap();
        
        // No git
        assert!(!MemoryPath::has_git_repo(dir.path()));
        
        // With git
        std::fs::create_dir(dir.path().join(".git")).unwrap();
        assert!(MemoryPath::has_git_repo(dir.path()));
    }
    
    #[test]
    fn test_ensure_dirs() {
        let dir = tempdir().unwrap();
        let path = MemoryPath::project(dir.path(), false);
        
        path.ensure_dirs().unwrap();
        
        assert!(path.base().exists());
        assert!(path.cache_dir().exists());
    }
}
