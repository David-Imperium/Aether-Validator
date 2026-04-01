//! RAG integration for learning from validation corrections
//!
//! This module provides:
//! - Storage of validation corrections for future reference
//! - Search for similar issues and their solutions
//! - Suggestions based on historical patterns
//!
//! Status: Future feature - not yet integrated with CLI

#![allow(dead_code)]

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// RAG storage for Aether validation learning
pub struct AetherRag {
    /// Path to the RAG storage directory
    storage_path: PathBuf,
}

/// A learned correction entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrectionEntry {
    /// Unique ID
    pub id: String,
    /// Original error ID
    pub error_id: String,
    /// Language
    pub language: String,
    /// Original problematic code
    pub original_code: String,
    /// Corrected code
    pub corrected_code: String,
    /// Error message
    pub message: String,
    /// How it was fixed
    pub fix_description: String,
    /// Timestamp
    pub timestamp: i64,
    /// Tags for categorization
    pub tags: Vec<String>,
}

/// Search result from RAG
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Matching entry
    pub entry: CorrectionEntry,
    /// Relevance score (0.0 - 1.0)
    pub score: f32,
    /// What matched
    pub match_type: String,
}

impl AetherRag {
    /// Create a new RAG instance
    pub fn new() -> Result<Self> {
        let storage_path = Self::get_storage_path()?;

        // Create storage directory if needed
        fs::create_dir_all(&storage_path)
            .with_context(|| format!("Failed to create RAG storage: {:?}", storage_path))?;

        Ok(Self { storage_path })
    }

    /// Get the default storage path
    fn get_storage_path() -> Result<PathBuf> {
        // Use XDG data directory on Linux, AppData on Windows, etc.
        if let Ok(data_dir) = std::env::var("AETHER_DATA_DIR") {
            return Ok(PathBuf::from(data_dir).join("rag"));
        }

        #[cfg(target_os = "windows")]
        {
            let app_data = std::env::var("APPDATA")
                .context("APPDATA not set")?;
            Ok(PathBuf::from(app_data).join("aether").join("rag"))
        }

        #[cfg(target_os = "macos")]
        {
            let home = std::env::var("HOME")
                .context("HOME not set")?;
            Ok(PathBuf::from(home)
                .join("Library")
                .join("Application Support")
                .join("aether")
                .join("rag"))
        }

        #[cfg(target_os = "linux")]
        {
            let data_dir = std::env::var("XDG_DATA_HOME")
                .unwrap_or_else(|_| {
                    let home = std::env::var("HOME").unwrap_or_default();
                    format!("{}/.local/share", home)
                });
            Ok(PathBuf::from(data_dir).join("aether").join("rag"))
        }

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            let home = std::env::var("HOME")
                .context("HOME not set")?;
            Ok(PathBuf::from(home).join(".aether").join("rag"))
        }
    }

    /// Store a correction for future reference
    pub fn store_correction(&self, entry: CorrectionEntry) -> Result<String> {
        let filename = format!("{}.json", entry.id);
        let path = self.storage_path.join(&filename);

        let json = serde_json::to_string_pretty(&entry)
            .context("Failed to serialize correction")?;

        fs::write(&path, json)
            .with_context(|| format!("Failed to write correction: {:?}", path))?;

        // Update index
        self.update_index(&entry)?;

        Ok(entry.id)
    }

    /// Search for similar corrections
    pub fn search(&self, query: &str, language: Option<&str>, limit: usize) -> Result<Vec<SearchResult>> {
        let mut results = Vec::new();
        let query_lower = query.to_lowercase();

        // Read all entries
        for entry in fs::read_dir(&self.storage_path)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().map(|e| e == "json").unwrap_or(false) && !path.file_name().expect("path from read_dir has filename").to_string_lossy().starts_with("index") {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(correction) = serde_json::from_str::<CorrectionEntry>(&content) {
                        // Filter by language if specified
                        if let Some(lang) = language {
                            if correction.language != lang {
                                continue;
                            }
                        }

                        // Calculate relevance score
                        let score = self.calculate_relevance(&query_lower, &correction);

                        if score > 0.0 {
                            results.push(SearchResult {
                                entry: correction,
                                score,
                                match_type: "keyword".to_string(),
                            });
                        }
                    }
                }
            }
        }

        // Sort by score and limit
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(limit);

        Ok(results)
    }

    /// Calculate relevance score for a query against an entry
    fn calculate_relevance(&self, query: &str, entry: &CorrectionEntry) -> f32 {
        let mut score = 0.0f32;

        // Check error ID match
        if entry.error_id.to_lowercase().contains(query) {
            score += 0.5;
        }

        // Check message match
        if entry.message.to_lowercase().contains(query) {
            score += 0.3;
        }

        // Check fix description match
        if entry.fix_description.to_lowercase().contains(query) {
            score += 0.2;
        }

        // Check tags
        for tag in &entry.tags {
            if tag.to_lowercase().contains(query) {
                score += 0.1;
            }
        }

        // Normalize
        if score > 1.0 {
            score = 1.0;
        }

        score
    }

    /// Update the search index
    fn update_index(&self, entry: &CorrectionEntry) -> Result<()> {
        let index_path = self.storage_path.join("index.json");

        let mut index: Vec<IndexEntry> = if index_path.exists() {
            let content = fs::read_to_string(&index_path)?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Vec::new()
        };

        // Add or update entry
        let new_index_entry = IndexEntry {
            id: entry.id.clone(),
            error_id: entry.error_id.clone(),
            language: entry.language.clone(),
            tags: entry.tags.clone(),
        };

        // Remove old if exists
        index.retain(|e| e.id != entry.id);
        index.push(new_index_entry);

        // Write index
        let json = serde_json::to_string(&index)?;
        fs::write(&index_path, json)?;

        Ok(())
    }

    /// Get statistics about stored corrections
    pub fn stats(&self) -> Result<RagStats> {
        let mut stats = RagStats::default();

        for entry in fs::read_dir(&self.storage_path)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().map(|e| e == "json").unwrap_or(false) && !path.file_name().expect("path from read_dir has filename").to_string_lossy().starts_with("index") {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(correction) = serde_json::from_str::<CorrectionEntry>(&content) {
                        stats.total_entries += 1;
                        stats.by_language.entry(correction.language).or_insert(0).add_assign(1);
                        stats.by_error_id.entry(correction.error_id).or_insert(0).add_assign(1);
                    }
                }
            }
        }

        Ok(stats)
    }

    /// Clear all stored corrections
    pub fn clear(&self) -> Result<()> {
        for entry in fs::read_dir(&self.storage_path)? {
            let entry = entry?;
            if entry.path().extension().map(|e| e == "json").unwrap_or(false) {
                fs::remove_file(entry.path())?;
            }
        }
        Ok(())
    }
}

impl Default for AetherRag {
    fn default() -> Self {
        Self::new().expect("Failed to initialize RAG")
    }
}

/// Index entry for fast searching
#[derive(Debug, Clone, Serialize, Deserialize)]
struct IndexEntry {
    id: String,
    error_id: String,
    language: String,
    tags: Vec<String>,
}

/// Statistics about the RAG storage
#[derive(Debug, Default, Serialize)]
pub struct RagStats {
    pub total_entries: usize,
    pub by_language: std::collections::HashMap<String, usize>,
    pub by_error_id: std::collections::HashMap<String, usize>,
}

use std::ops::AddAssign;

/// Generate a unique ID for a correction
pub fn generate_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis();
    format!("correction-{}", timestamp)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_correction_entry_serialization() {
        let entry = CorrectionEntry {
            id: "test-1".to_string(),
            error_id: "LOGIC001".to_string(),
            language: "rust".to_string(),
            original_code: "let x = option.unwrap();".to_string(),
            corrected_code: "let x = option.ok_or(Error::Missing)?;".to_string(),
            message: "Implicit unwrap".to_string(),
            fix_description: "Use explicit error handling with ok_or".to_string(),
            timestamp: 1234567890,
            tags: vec!["error-handling".to_string()],
        };

        let json = serde_json::to_string(&entry).unwrap();
        let parsed: CorrectionEntry = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.id, "test-1");
        assert_eq!(parsed.error_id, "LOGIC001");
    }
}
