//! Git Memory Store - Wrapper for versioning memory with git
//!
//! GitMemoryStore wraps a MemoryStore and automatically versions
//! changes to the .synward/ directory in git.

use std::path::{Path, PathBuf};
use std::process::Command;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use super::store::{MemoryEntry, MemoryStore};
use super::MemoryId;
use super::learned_config::LearnedConfig;
use super::scope::MemoryPath;

/// Git commit hash
pub type CommitHash = String;

/// Snapshot info from git log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotInfo {
    /// Commit hash
    pub hash: CommitHash,
    /// Commit message
    pub message: String,
    /// Author
    pub author: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Whitelisted git commands for security
const ALLOWED_GIT_COMMANDS: &[&str] = &[
    "add",
    "commit",
    "rev-parse",
    "diff",
    "checkout",
    "log",
];

/// Wrapper that integrates git into memory storage
pub struct GitMemoryStore {
    /// Inner memory store
    store: MemoryStore,
    
    /// Project root directory
    project_root: PathBuf,
    
    /// Auto-commit on every save
    auto_commit: bool,
    
    /// Path to .synward/ directory
    synward_path: PathBuf,
}

impl GitMemoryStore {
    /// Create a new GitMemoryStore
    pub fn new(store: MemoryStore, project_root: PathBuf, auto_commit: bool) -> Self {
        let synward_path = MemoryPath::project_base(&project_root);
        Self {
            store,
            project_root,
            auto_commit,
            synward_path,
        }
    }
    
    /// Check if git is available
    pub fn is_git_available(&self) -> bool {
        self.project_root.join(".git").exists()
    }
    
    /// Run a git command
    /// Security: Only whitelisted git commands are allowed to prevent command injection
    fn git(&self, args: &[&str]) -> Result<String> {
        // Security: Validate first argument (git subcommand) against whitelist
        if let Some(subcommand) = args.first() {
            if !ALLOWED_GIT_COMMANDS.contains(subcommand) {
                return Err(Error::Other(format!(
                    "Security: Git command '{}' not allowed. Allowed: {:?}",
                    subcommand, ALLOWED_GIT_COMMANDS
                )));
            }
        } else {
            return Err(Error::Other("Security: Empty git command not allowed".into()));
        }
        
        let output = Command::new("git")
            .args(args)
            .current_dir(&self.project_root)
            .output()
            .map_err(Error::Io)?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::Other(format!("Git error: {}", stderr)));
        }
        
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
    
    /// Add .synward/ to git staging
    fn add_synward(&self) -> Result<()> {
        self.git(&["add", ".synward/"])?;
        Ok(())
    }
    
    /// Create a commit with the given message
    fn commit(&self, message: &str) -> Result<CommitHash> {
        self.git(&["commit", "-m", message])?;
        self.get_head_hash()
    }
    
    /// Get current HEAD commit hash
    fn get_head_hash(&self) -> Result<CommitHash> {
        self.git(&["rev-parse", "HEAD"])
    }
    
    /// Check if there are staged changes
    fn has_staged_changes(&self) -> bool {
        self.git(&["diff", "--cached", "--quiet"])
            .is_err() // Exit code 1 = changes exist
    }
    
    /// Check if there are unstaged changes in .synward/
    fn has_unstaged_changes(&self) -> bool {
        self.git(&["diff", "--quiet", ".synward/"])
            .is_err()
    }
    
    /// Create a snapshot (manual commit)
    pub fn commit_snapshot(&mut self, message: &str) -> Result<CommitHash> {
        if !self.is_git_available() {
            return Err(Error::Other("Git not available in this project".into()));
        }
        
        self.add_synward()?;
        
        if self.has_staged_changes() {
            let full_message = format!("synward: {}", message);
            self.commit(&full_message)
        } else {
            // No changes to commit
            self.get_head_hash()
        }
    }
    
    /// Restore memory from a specific commit
    pub fn restore_snapshot(&mut self, commit: &CommitHash) -> Result<()> {
        if !self.is_git_available() {
            return Err(Error::Other("Git not available in this project".into()));
        }
        
        // Checkout .synward/ from specific commit
        self.git(&["checkout", commit, "--", ".synward/"])?;
        
        // Reload store
        self.store = MemoryStore::new(Some(self.synward_path.join("memory.toml")))?;
        
        Ok(())
    }
    
    /// List all snapshots (commits touching .synward/)
    pub fn list_snapshots(&self) -> Result<Vec<SnapshotInfo>> {
        if !self.is_git_available() {
            return Ok(Vec::new());
        }
        
        let log = self.git(&[
            "log",
            "--oneline",
            "--date=iso",
            "--format=%H|%s|%an|%aI",
            "--",
            ".synward/"
        ])?;
        
        let mut snapshots = Vec::new();
        for line in log.lines() {
            let parts: Vec<&str> = line.splitn(4, '|').collect();
            if parts.len() == 4 {
                if let Ok(timestamp) = DateTime::parse_from_rfc3339(parts[3]) {
                    snapshots.push(SnapshotInfo {
                        hash: parts[0].to_string(),
                        message: parts[1].to_string(),
                        author: parts[2].to_string(),
                        timestamp: timestamp.with_timezone(&Utc),
                    });
                }
            }
        }
        
        Ok(snapshots)
    }
    
    /// Get current snapshot info
    pub fn current_snapshot(&self) -> Result<Option<SnapshotInfo>> {
        if !self.is_git_available() {
            return Ok(None);
        }
        
        let _hash = self.get_head_hash()?;
        let log = self.git(&[
            "log",
            "-1",
            "--format=%H|%s|%an|%aI",
        ])?;
        
        let parts: Vec<&str> = log.splitn(4, '|').collect();
        if parts.len() == 4 {
            if let Ok(timestamp) = DateTime::parse_from_rfc3339(parts[3]) {
                return Ok(Some(SnapshotInfo {
                    hash: parts[0].to_string(),
                    message: parts[1].to_string(),
                    author: parts[2].to_string(),
                    timestamp: timestamp.with_timezone(&Utc),
                }));
            }
        }
        
        Ok(None)
    }
    
    /// Auto-commit if enabled and changes exist
    fn maybe_auto_commit(&self) -> Result<()> {
        if self.auto_commit && self.is_git_available() && self.has_unstaged_changes() {
            self.add_synward()?;
            if self.has_staged_changes() {
                self.commit("synward: auto-save")?;
            }
        }
        Ok(())
    }
    
    // Delegate to inner store
    
    /// Save entry (with optional auto-commit)
    pub fn save(&mut self, entry: MemoryEntry) -> Result<()> {
        self.store.save(entry)?;
        self.maybe_auto_commit()
    }
    
    /// Recall entries similar to code
    pub fn recall(&self, code: &str, limit: usize) -> Result<Vec<MemoryEntry>> {
        self.store.recall(code, limit)
    }
    
    /// Get entry by ID
    pub fn get(&self, id: &MemoryId) -> Option<MemoryEntry> {
        self.store.get(id).cloned()
    }
    
    /// Delete entry
    pub fn delete(&mut self, id: &MemoryId) -> Result<bool> {
        let result = self.store.delete(id)?;
        self.maybe_auto_commit()?;
        Ok(result)
    }
    
    /// Get all entries
    pub fn all_entries(&self) -> Vec<MemoryEntry> {
        self.store.all_entries()
    }
    
    /// Get entry count
    pub fn len(&self) -> usize {
        self.store.len()
    }
    
    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.store.is_empty()
    }
    
    /// Load config for project
    pub fn load_config(&self, project_root: &Path) -> Result<LearnedConfig> {
        self.store.load_config(project_root)
    }
    
    /// Save config for project
    pub fn save_config(&self, config: &LearnedConfig) -> Result<()> {
        self.store.save_config(config)
    }
    
    /// Get inner store reference
    pub fn inner(&self) -> &MemoryStore {
        &self.store
    }
    
    /// Get mutable inner store reference
    pub fn inner_mut(&mut self) -> &mut MemoryStore {
        &mut self.store
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_snapshot_info_parse() {
        let info = SnapshotInfo {
            hash: "abc123".to_string(),
            message: "synward: snapshot".to_string(),
            author: "user".to_string(),
            timestamp: Utc::now(),
        };
        
        assert_eq!(info.hash, "abc123");
    }
}
