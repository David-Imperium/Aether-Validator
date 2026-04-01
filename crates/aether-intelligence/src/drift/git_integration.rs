//! Git Integration - Load code history from git

use crate::error::{Error, Result};
use std::path::PathBuf;

/// Git integration for loading history
pub struct GitIntegration {
    repo_path: PathBuf,
    #[cfg(feature = "git2")]
    repo: Option<git2::Repository>,
}

impl GitIntegration {
    /// Create a new integration
    pub fn new(repo_path: PathBuf) -> Result<Self> {
        if !repo_path.exists() {
            return Err(Error::Git("Repository path does not exist".to_string()));
        }

        #[cfg(feature = "git2")]
        {
            let repo = git2::Repository::discover(&repo_path).ok();
            Ok(Self { repo_path, repo })
        }

        #[cfg(not(feature = "git2"))]
        Ok(Self { repo_path })
    }

    /// Get commit history
    #[cfg(feature = "git2")]
    pub fn get_commits(&self, limit: usize) -> Result<Vec<CommitInfo>> {
        let repo = self.repo.as_ref().ok_or_else(|| {
            Error::Git("Not a git repository".to_string())
        })?;

        let mut revwalk = repo.revwalk().map_err(|e| Error::Git(e.to_string()))?;
        revwalk.push_head().map_err(|e| Error::Git(e.to_string()))?;
        revwalk.set_sorting(git2::Sort::TIME).map_err(|e| Error::Git(e.to_string()))?;

        let mut commits = Vec::new();

        for (i, oid_result) in revwalk.enumerate() {
            if i >= limit {
                break;
            }

            if let Ok(oid) = oid_result {
                if let Ok(commit) = repo.find_commit(oid) {
                    let hash = commit.id().to_string();
                    let author = commit.author().name().unwrap_or("unknown").to_string();
                    let message = commit.message().unwrap_or("").to_string();
                    let timestamp = chrono::DateTime::from_timestamp(
                        commit.time().seconds(),
                        0
                    ).unwrap_or_else(|| chrono::Utc::now());

                    // Get files changed
                    let mut files_changed = Vec::new();
                    if let Ok(tree) = commit.tree() {
                        if let Ok(parent) = commit.parent(0) {
                            if let Ok(parent_tree) = parent.tree() {
                                if let Ok(diff) = repo.diff_tree_to_tree(
                                    Some(&parent_tree),
                                    Some(&tree),
                                    None
                                ) {
                                    diff.foreach(
                                        &mut |delta, _| {
                                            if let Some(path) = delta.new_file().path() {
                                                files_changed.push(path.to_string_lossy().to_string());
                                            }
                                            true
                                        },
                                        None,
                                        None,
                                        None
                                    ).ok();
                                }
                            }
                        }
                    }

                    commits.push(CommitInfo {
                        hash,
                        author,
                        message,
                        timestamp,
                        files_changed,
                    });
                }
            }
        }

        Ok(commits)
    }

    /// Get commit history (stub without git2)
    #[cfg(not(feature = "git2"))]
    pub fn get_commits(&self, _limit: usize) -> Result<Vec<CommitInfo>> {
        Ok(vec![])
    }

    /// Get file content at a specific commit
    #[cfg(feature = "git2")]
    pub fn get_file_at_commit(&self, commit_hash: &str, file_path: &str) -> Result<String> {
        let repo = self.repo.as_ref().ok_or_else(|| {
            Error::Git("Not a git repository".to_string())
        })?;

        let oid = git2::Oid::from_str(commit_hash)
            .map_err(|e| Error::Git(format!("Invalid commit hash: {}", e)))?;

        let commit = repo.find_commit(oid)
            .map_err(|e| Error::Git(format!("Commit not found: {}", e)))?;

        let tree = commit.tree()
            .map_err(|e| Error::Git(format!("Failed to get tree: {}", e)))?;

        let entry = tree.get_path(std::path::Path::new(file_path))
            .map_err(|e| Error::Git(format!("File not found in commit: {}", e)))?;

        let blob = repo.find_blob(entry.id())
            .map_err(|e| Error::Git(format!("Failed to get blob: {}", e)))?;

        let content = std::str::from_utf8(blob.content())
            .map_err(|e| Error::Git(format!("Invalid UTF-8 content: {}", e)))?;

        Ok(content.to_string())
    }

    /// Get file content at commit (stub without git2)
    #[cfg(not(feature = "git2"))]
    pub fn get_file_at_commit(&self, _commit: &str, _file: &str) -> Result<String> {
        Err(Error::Git("git2 feature not enabled".to_string()))
    }

    /// Check if repository is valid
    pub fn is_valid(&self) -> bool {
        #[cfg(feature = "git2")]
        return self.repo.is_some();

        #[cfg(not(feature = "git2"))]
        false
    }
}

/// Information about a commit
#[derive(Debug, Clone)]
pub struct CommitInfo {
    /// Commit hash
    pub hash: String,

    /// Author
    pub author: String,

    /// Message
    pub message: String,

    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Files changed
    pub files_changed: Vec<String>,
}
