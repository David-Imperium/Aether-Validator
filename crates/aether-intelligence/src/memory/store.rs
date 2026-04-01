//! Memory Store - Persistent storage for validation memories

use crate::error::{Error, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;

use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

use super::{MemoryId, ProjectContext};
use super::learned_config::LearnedConfig;
use super::scope::MemoryPath;

/// A single memory entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    /// Unique identifier
    pub id: MemoryId,

    /// The code snippet
    pub code: String,

    /// Programming language
    pub language: String,

    /// Type of memory
    pub memory_type: MemoryType,

    /// Validation errors found
    pub errors: Vec<String>,

    /// Context (project, file, etc.)
    pub context: ProjectContext,

    /// When this was stored
    pub created_at: DateTime<Utc>,

    /// How many times this was recalled
    pub recall_count: u32,

    /// User feedback (if any)
    pub feedback: Option<MemoryFeedback>,
}

impl MemoryEntry {
    /// Create a new memory entry
    pub fn new(code: impl Into<String>, language: impl Into<String>) -> Self {
        Self {
            id: MemoryId::default(),
            code: code.into(),
            language: language.into(),
            memory_type: MemoryType::Code,
            errors: Vec::new(),
            context: ProjectContext::default(),
            created_at: Utc::now(),
            recall_count: 0,
            feedback: None,
        }
    }

    /// Add an error to this entry
    pub fn with_error(mut self, error: impl Into<String>) -> Self {
        self.errors.push(error.into());
        self
    }

    /// Set the memory type
    pub fn with_type(mut self, memory_type: MemoryType) -> Self {
        self.memory_type = memory_type;
        self
    }

    /// Set the project context
    pub fn with_context(mut self, context: ProjectContext) -> Self {
        self.context = context;
        self
    }

    /// Mark as recalled (increment counter)
    pub fn mark_recalled(&mut self) {
        self.recall_count += 1;
    }
}

/// Type of memory
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MemoryType {
    /// Code snippet with validation result
    Code,

    /// Pattern discovered
    Pattern,

    /// Fix applied
    Fix,

    /// User preference
    Preference,

    /// Project-wide context
    ProjectContext,
}

/// User feedback on a memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryFeedback {
    pub helpful: bool,
    pub comment: Option<String>,
    pub timestamp: DateTime<Utc>,
}

/// In-memory and persistent store for memories
#[derive(Debug)]
pub struct MemoryStore {
    /// In-memory cache
    entries: HashMap<MemoryId, MemoryEntry>,

    /// Hash index for real-time dedup (code hash -> MemoryId)
    hash_index: HashMap<u64, MemoryId>,

    /// Persistent storage path
    path: PathBuf,

    /// Maximum entries in memory
    max_entries: usize,
}

impl MemoryStore {
    /// Create a new memory store
    pub fn new(path: Option<PathBuf>) -> Result<Self> {
        let path = path.unwrap_or_else(|| {
            MemoryPath::global_base()
                .join("memory.toml")
        });

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(Error::Io)?;
        }

        let mut store = Self {
            entries: HashMap::with_capacity(100),
            hash_index: HashMap::with_capacity(100),
            path,
            max_entries: 10_000,
        };

        // Load existing entries
        store.load()?;

        Ok(store)
    }
    
    /// Compute hash for code (for dedup)
    fn compute_hash(code: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        code.hash(&mut hasher);
        hasher.finish()
    }

    /// Save a memory entry (with real-time hash-based dedup)
    pub fn save(&mut self, entry: MemoryEntry) -> Result<()> {
        // Check capacity
        if self.entries.len() >= self.max_entries {
            // Remove oldest/least recalled entries
            self.prune()?;
        }

        // Real-time dedup: check if exact duplicate exists
        let hash = Self::compute_hash(&entry.code);
        if let Some(existing_id) = self.hash_index.get(&hash) {
            // Merge with existing entry instead of creating duplicate
            if let Some(existing) = self.entries.get_mut(existing_id) {
                // Merge: keep higher recall_count, combine errors
                existing.recall_count = existing.recall_count.max(entry.recall_count);
                for err in entry.errors {
                    if !existing.errors.contains(&err) {
                        existing.errors.push(err);
                    }
                }
                // Keep earlier created_at
                if entry.created_at < existing.created_at {
                    existing.created_at = entry.created_at;
                }
                return self.persist();
            }
        }

        // No duplicate: insert new entry
        let id = entry.id.clone();
        self.hash_index.insert(hash, id.clone());
        self.entries.insert(id, entry);
        self.persist()
    }

    /// Recall entries similar to the given code
    pub fn recall(&self, code: &str, limit: usize) -> Result<Vec<MemoryEntry>> {
        let mut scored: Vec<_> = self.entries
            .values()
            .map(|entry| {
                let score = similarity(code, &entry.code);
                (score, entry.clone())
            })
            .collect();

        // Sort by similarity (descending)
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        // Take top N
        let results: Vec<_> = scored
            .into_iter()
            .take(limit)
            .map(|(_, mut entry)| {
                entry.mark_recalled();
                entry
            })
            .collect();

        Ok(results)
    }

    /// Get a specific entry by ID
    pub fn get(&self, id: &MemoryId) -> Option<&MemoryEntry> {
        self.entries.get(id)
    }

    /// Delete an entry
    pub fn delete(&mut self, id: &MemoryId) -> Result<bool> {
        if let Some(entry) = self.entries.remove(id) {
            // Clean up hash index
            let hash = Self::compute_hash(&entry.code);
            self.hash_index.remove(&hash);
            self.persist()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
    
    /// Get all entries (for deduplication)
    pub fn all_entries(&self) -> Vec<MemoryEntry> {
        self.entries.values().cloned().collect()
    }

    /// Get entry count
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Delete multiple entries by ID (for deduplication cleanup)
    pub fn delete_multiple(&mut self, ids: &[MemoryId]) -> Result<usize> {
        let mut removed = 0;
        for id in ids {
            if self.entries.remove(id).is_some() {
                removed += 1;
            }
        }
        if removed > 0 {
            self.persist()?;
        }
        Ok(removed)
    }

    /// Load LearnedConfig for a project (Memory-Driven Core)
    ///
    /// This is the main entry point for memory-driven validation.
    /// Returns the learned configuration that should be applied to validation layers.
    ///
    /// Uses TOML format for human-readability and editability.
    pub fn load_config(&self, project_root: &Path) -> Result<LearnedConfig> {
        let config_path = project_root
            .join(".aether")
            .join("learned_config.toml");

        if config_path.exists() {
            let content = fs::read_to_string(&config_path).map_err(Error::Io)?;
            let config: LearnedConfig = toml::from_str(&content)
                .map_err(|e| Error::Toml(e.to_string()))?;
            tracing::info!(
                "Loaded LearnedConfig for {:?} (confidence: {:.2})",
                project_root,
                config.confidence
            );
            Ok(config)
        } else {
            // Return default config for new projects
            let defaults = LearnedConfig::defaults();
            tracing::info!(
                "No LearnedConfig found for {:?}, using defaults",
                project_root
            );
            Ok(defaults)
        }
    }

    /// Save LearnedConfig for a project
    ///
    /// Uses TOML format for human-readability and editability.
    pub fn save_config(&self, config: &LearnedConfig) -> Result<()> {
        let config_path = config
            .project_root
            .join(".aether")
            .join("learned_config.toml");

        // Ensure .aether directory exists
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent).map_err(Error::Io)?;
        }

        let content = toml::to_string_pretty(config)
            .map_err(|e| Error::Toml(e.to_string()))?;
        fs::write(&config_path, content).map_err(Error::Io)?;

        tracing::info!(
            "Saved LearnedConfig to {:?} (confidence: {:.2})",
            config_path,
            config.confidence
        );
        Ok(())
    }

    /// Update LearnedConfig based on validation feedback
    ///
    /// This implements the feedback loop: validation results → config evolution
    pub fn update_config_from_feedback(
        &self,
        config: &mut LearnedConfig,
        violations: &[super::validation_state::ViolationRecord],
        accepted_ids: &[String],
    ) -> Result<()> {
        // Increment sample count
        config.stats.sample_count += 1;
        config.last_updated = Utc::now();

        // Process accepted violations → whitelist
        for violation in violations {
            if accepted_ids.contains(&violation.id) {
                // Add to whitelist if not already present
                let whitelist_entry = super::learned_config::WhitelistedPattern {
                    pattern_id: violation.rule.clone(),
                    file_pattern: violation.file.clone(),
                    reason: "Accepted by user".to_string(),
                    approved_by: "user".to_string(),
                    approved_at: Utc::now(),
                    expires_at: None,
                };

                if !config.security_whitelist.iter().any(|w| {
                    w.pattern_id == whitelist_entry.pattern_id
                        && w.file_pattern == whitelist_entry.file_pattern
                }) {
                    config.security_whitelist.push(whitelist_entry);
                }
            }
        }

        // Update violation rates
        let total_violations = violations.len() as f32;
        if config.stats.sample_count > 0 {
            let alpha = 0.1; // Exponential moving average factor
            config.stats.current_violation_rate = alpha * total_violations
                + (1.0 - alpha) * config.stats.current_violation_rate;
        }

        // Update confidence
        config.update_confidence();

        // Persist updated config
        self.save_config(config)?;

        tracing::info!(
            "Updated LearnedConfig: {} samples, {:.2} avg violations",
            config.stats.sample_count,
            config.stats.current_violation_rate
        );
        Ok(())
    }

    /// Get all entries
    pub fn all(&self) -> Vec<&MemoryEntry> {
        self.entries.values().collect()
    }

    /// Count entries
    pub fn count(&self) -> usize {
        self.entries.len()
    }

    /// Clear all entries
    pub fn clear(&mut self) -> Result<()> {
        self.entries.clear();
        self.persist()?;
        tracing::info!("Cleared all memory entries");
        Ok(())
    }

    /// Load entries from disk (TOML format)
    fn load(&mut self) -> Result<()> {
        if !self.path.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(&self.path).map_err(Error::Io)?;

        // TOML needs a wrapper struct for arrays
        #[derive(Deserialize)]
        struct MemoryFile {
            entries: Vec<MemoryEntry>,
        }

        let file: MemoryFile = toml::from_str(&content)
            .map_err(|e| Error::Toml(e.to_string()))?;

        for entry in file.entries {
            // Populate hash index for dedup
            let hash = Self::compute_hash(&entry.code);
            self.hash_index.insert(hash, entry.id.clone());
            self.entries.insert(entry.id.clone(), entry);
        }

        tracing::info!("Loaded {} memory entries from {:?}", self.entries.len(), self.path);
        Ok(())
    }

    /// Persist entries to disk (TOML format)
    fn persist(&self) -> Result<()> {
        let entries: Vec<_> = self.entries.values().collect();

        // TOML needs a wrapper struct for arrays
        #[derive(Serialize)]
        struct MemoryFile<'a> {
            entries: Vec<&'a MemoryEntry>,
        }

        let file = MemoryFile { entries };
        let content = toml::to_string_pretty(&file)
            .map_err(|e| Error::Toml(e.to_string()))?;

        fs::write(&self.path, content).map_err(Error::Io)?;
        tracing::debug!("Persisted {} entries to {:?}", file.entries.len(), self.path);
        Ok(())
    }

    /// Remove old/unused entries
    fn prune(&mut self) -> Result<()> {
        // Keep entries with:
        // - Recent creation (last 30 days)
        // - High recall count
        // - Positive feedback

        let now = Utc::now();
        let threshold = chrono::Duration::days(30);

        let to_keep: Vec<_> = self.entries
            .values()
            .filter(|entry| {
                let age = now - entry.created_at;
                let is_recent = age < threshold;
                let is_useful = entry.recall_count > MIN_RECALL_KEEP;
                let is_helpful = entry.feedback.as_ref().map(|f| f.helpful).unwrap_or(false);

                is_recent || is_useful || is_helpful
            })
            .cloned()
            .collect();

        self.entries.clear();
        for entry in to_keep {
            self.entries.insert(entry.id.clone(), entry);
        }

        tracing::info!("Pruned memory store, {} entries remaining", self.entries.len());
        Ok(())
    }
}

/// Minimum recall count to keep entries during pruning
const MIN_RECALL_KEEP: u32 = 5;

/// Calculate similarity between two code snippets
///
/// Uses a simple token-based similarity for now.
/// Future: Use embeddings for semantic similarity.
pub fn similarity(code1: &str, code2: &str) -> f32 {
    // Normalize: lowercase, remove extra whitespace
    let normalize = |s: &str| -> Vec<String> {
        s.to_lowercase()
            .split_whitespace()
            .map(|w| w.to_string())
            .collect()
    };

    let tokens1 = normalize(code1);
    let tokens2 = normalize(code2);

    if tokens1.is_empty() || tokens2.is_empty() {
        return 0.0;
    }

    // Jaccard similarity
    let set1: std::collections::HashSet<_> = tokens1.iter().collect();
    let set2: std::collections::HashSet<_> = tokens2.iter().collect();

    let intersection = set1.intersection(&set2).count();
    let union = set1.union(&set2).count();

    if union == 0 {
        0.0
    } else {
        intersection as f32 / union as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_similarity_identical() {
        let code = "fn main() { println!(\"hello\"); }";
        assert!((similarity(code, code) - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_similarity_different() {
        let code1 = "fn main() { }";
        let code2 = "struct Point { x: i32 }";
        assert!(similarity(code1, code2) < 0.5);
    }

    #[test]
    fn test_similarity_similar() {
        let code1 = "fn main() { println!(\"hello\"); }";
        let code2 = "fn main() { println!(\"world\"); }";
        // Jaccard: 4/6 tokens match = 0.67
        assert!(similarity(code1, code2) > 0.6);
    }

    #[test]
    fn test_memory_store_roundtrip() {
        use std::env::temp_dir;
        let temp_path = temp_dir().join(format!("aether_test_memory_{}.toml", std::process::id()));

        let mut store = MemoryStore::new(Some(temp_path.clone())).unwrap();
        store.clear().ok(); // Clean any previous test data

        let entry = MemoryEntry::new("fn test() {}", "rust")
            .with_error("LOGIC001: Test error");

        store.save(entry.clone()).unwrap();

        let recalled = store.recall("fn test() {}", 5).unwrap();
        assert_eq!(recalled.len(), 1);
        assert_eq!(recalled[0].code, entry.code);

        // Cleanup
        std::fs::remove_file(&temp_path).ok();
    }
}
