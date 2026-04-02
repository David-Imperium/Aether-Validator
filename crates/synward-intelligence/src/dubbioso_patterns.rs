//! Dubbioso Pattern Persistence
//!
//! Stores accepted patterns from Dubbioso Mode for permanent learning.
//! When a pattern is accepted N times (configurable), it becomes permanent.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use chrono::{DateTime, Utc};
use crate::error::{Error, Result};

/// A learned pattern from Dubbioso Mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DubbiosoPattern {
    /// Pattern identifier (e.g., "unwrap() in test", "expect() in main")
    pub id: String,

    /// The violation pattern
    pub pattern: String,

    /// Language this applies to
    pub language: String,

    /// How many times accepted
    pub accept_count: u32,

    /// How many times rejected
    pub reject_count: u32,

    /// First seen
    pub first_seen: DateTime<Utc>,

    /// Last seen
    pub last_seen: DateTime<Utc>,

    /// Confidence adjustment (computed from accept/reject ratio)
    pub confidence_adjustment: f64,

    /// Whether this pattern is whitelisted
    pub is_whitelisted: bool,

    /// Whether this pattern is permanent (accepted N times)
    pub is_permanent: bool,

    /// File patterns where this applies (empty = all files)
    pub file_patterns: Vec<String>,

    /// Notes from user responses
    pub notes: Vec<String>,
}

/// Dubbioso Pattern Store - persistence for learned patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DubbiosoPatternStore {
    /// Patterns by ID
    patterns: HashMap<String, DubbiosoPattern>,

    /// Path to persistence file
    #[serde(skip)]
    path: Option<PathBuf>,

    /// Make permanent after N acceptances
    permanent_after: u32,

    /// Last save timestamp
    last_saved: Option<DateTime<Utc>>,
}

impl DubbiosoPatternStore {
    /// Create new pattern store
    pub fn new(permanent_after: u32) -> Self {
        Self {
            patterns: HashMap::new(),
            path: None,
            permanent_after,
            last_saved: None,
        }
    }

    /// Create with persistence path
    pub fn with_path(path: PathBuf, permanent_after: u32) -> Self {
        Self {
            patterns: HashMap::new(),
            path: Some(path),
            permanent_after,
            last_saved: None,
        }
    }

    /// Load from file
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::with_path(path.to_path_buf(), 5));
        }

        let content = fs::read_to_string(path).map_err(Error::Io)?;
        let mut store: Self = serde_json::from_str(&content)
            .map_err(Error::Serialization)?;

        store.path = Some(path.to_path_buf());
        Ok(store)
    }

    /// Save to file
    pub fn save(&self) -> Result<()> {
        let Some(ref path) = self.path else {
            return Ok(());
        };

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).map_err(Error::Io)?;
            }
        }

        let content = serde_json::to_string_pretty(self)
            .map_err(Error::Serialization)?;

        fs::write(path, content).map_err(Error::Io)?;
        Ok(())
    }

    /// Record a pattern acceptance
    pub fn accept_pattern(
        &mut self,
        pattern: &str,
        language: &str,
        file: &str,
        note: Option<&str>,
    ) -> PatternUpdate {
        let id = format!("{}_{}", language, pattern.replace(' ', "_").to_lowercase());
        let now = Utc::now();

        // Get or create entry
        let entry = self.patterns.entry(id.clone()).or_insert_with(|| DubbiosoPattern {
            id: id.clone(),
            pattern: pattern.to_string(),
            language: language.to_string(),
            accept_count: 0,
            reject_count: 0,
            first_seen: now,
            last_seen: now,
            confidence_adjustment: 0.0,
            is_whitelisted: false,
            is_permanent: false,
            file_patterns: Vec::new(),
            notes: Vec::new(),
        });

        entry.accept_count += 1;
        entry.last_seen = now;

        // Add file pattern if not already present
        if !entry.file_patterns.iter().any(|p| file.contains(p)) {
            if let Some(file_dir) = Path::new(file).parent().and_then(|p| p.to_str()) {
                if !file_dir.is_empty() && !entry.file_patterns.iter().any(|p| p == file_dir) {
                    entry.file_patterns.push(format!("{}/**", file_dir));
                }
            }
        }

        // Add note if provided
        if let Some(n) = note {
            entry.notes.push(format!("[{}] {}", now.format("%Y-%m-%d"), n));
        }

        // Compute confidence adjustment
        let total = entry.accept_count + entry.reject_count;
        if total > 0 {
            entry.confidence_adjustment = (entry.accept_count as f64 / total as f64) * 0.3;
        }

        // Check for permanence
        let became_permanent = !entry.is_permanent && entry.accept_count >= self.permanent_after;
        if became_permanent {
            entry.is_permanent = true;
        }

        // Extract result before saving
        let result = PatternUpdate {
            pattern_id: id,
            accept_count: entry.accept_count,
            is_permanent: entry.is_permanent,
            became_permanent,
            confidence_adjustment: entry.confidence_adjustment,
        };

        let _ = self.save();
        result
    }

    /// Record a pattern rejection
    pub fn reject_pattern(
        &mut self,
        pattern: &str,
        language: &str,
        _file: &str,
        note: Option<&str>,
    ) -> PatternUpdate {
        let id = format!("{}_{}", language, pattern.replace(' ', "_").to_lowercase());
        let now = Utc::now();

        let entry = self.patterns.entry(id.clone()).or_insert_with(|| DubbiosoPattern {
            id: id.clone(),
            pattern: pattern.to_string(),
            language: language.to_string(),
            accept_count: 0,
            reject_count: 0,
            first_seen: now,
            last_seen: now,
            confidence_adjustment: 0.0,
            is_whitelisted: false,
            is_permanent: false,
            file_patterns: Vec::new(),
            notes: Vec::new(),
        });

        entry.reject_count += 1;
        entry.last_seen = now;

        if let Some(n) = note {
            entry.notes.push(format!("[{}] REJECTED: {}", now.format("%Y-%m-%d"), n));
        }

        // Compute confidence adjustment (negative for rejections)
        let total = entry.accept_count + entry.reject_count;
        if total > 0 {
            entry.confidence_adjustment = -((entry.reject_count as f64 / total as f64) * 0.3);
        }

        // Extract result before saving
        let result = PatternUpdate {
            pattern_id: id,
            accept_count: entry.accept_count,
            is_permanent: entry.is_permanent,
            became_permanent: false,
            confidence_adjustment: entry.confidence_adjustment,
        };

        let _ = self.save();
        result
    }

    /// Whitelist a pattern
    pub fn whitelist_pattern(
        &mut self,
        pattern: &str,
        language: &str,
        reason: &str,
    ) {
        let id = format!("{}_{}", language, pattern.replace(' ', "_").to_lowercase());

        let now = Utc::now();
        let entry = self.patterns.entry(id.clone()).or_insert_with(|| DubbiosoPattern {
            id: id.clone(),
            pattern: pattern.to_string(),
            language: language.to_string(),
            accept_count: 0,
            reject_count: 0,
            first_seen: now,
            last_seen: now,
            confidence_adjustment: 0.3, // Whitelist gives confidence boost
            is_whitelisted: false,
            is_permanent: false,
            file_patterns: Vec::new(),
            notes: Vec::new(),
        });

        entry.is_whitelisted = true;
        entry.is_permanent = true; // Whitelist is always permanent
        entry.notes.push(format!("[{}] WHITELISTED: {}", now.format("%Y-%m-%d"), reason));

        let _ = self.save();
    }

    /// Check if pattern is whitelisted
    pub fn is_whitelisted(&self, pattern: &str, language: &str, file: &str) -> bool {
        let id = format!("{}_{}", language, pattern.replace(' ', "_").to_lowercase());

        if let Some(entry) = self.patterns.get(&id) {
            if entry.is_whitelisted {
                // Check file pattern match
                if entry.file_patterns.is_empty() {
                    return true;
                }
                return entry.file_patterns.iter().any(|p| file.contains(p));
            }
        }
        false
    }

    /// Check if pattern is permanent
    pub fn is_permanent(&self, pattern: &str, language: &str) -> bool {
        let id = format!("{}_{}", language, pattern.replace(' ', "_").to_lowercase());
        self.patterns.get(&id).map(|e| e.is_permanent).unwrap_or(false)
    }

    /// Get confidence adjustment for a pattern
    pub fn get_confidence_adjustment(&self, pattern: &str, language: &str) -> f64 {
        let id = format!("{}_{}", language, pattern.replace(' ', "_").to_lowercase());
        self.patterns.get(&id).map(|e| e.confidence_adjustment).unwrap_or(0.0)
    }

    /// Get all permanent patterns
    pub fn permanent_patterns(&self) -> Vec<&DubbiosoPattern> {
        self.patterns.values().filter(|p| p.is_permanent).collect()
    }

    /// Get all whitelisted patterns
    pub fn whitelisted_patterns(&self) -> Vec<&DubbiosoPattern> {
        self.patterns.values().filter(|p| p.is_whitelisted).collect()
    }

    /// Get pattern by ID
    pub fn get_pattern(&self, id: &str) -> Option<&DubbiosoPattern> {
        self.patterns.get(id)
    }

    /// Get total pattern count
    pub fn pattern_count(&self) -> usize {
        self.patterns.len()
    }

    /// Remove a pattern
    pub fn remove_pattern(&mut self, id: &str) -> bool {
        if self.patterns.remove(id).is_some() {
            let _ = self.save();
            true
        } else {
            false
        }
    }

    /// Clear all non-permanent patterns
    pub fn clear_non_permanent(&mut self) -> usize {
        let before = self.patterns.len();
        self.patterns.retain(|_, p| p.is_permanent);
        let removed = before - self.patterns.len();
        if removed > 0 {
            let _ = self.save();
        }
        removed
    }
}

impl Default for DubbiosoPatternStore {
    fn default() -> Self {
        Self::new(5)
    }
}

/// Result of a pattern update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternUpdate {
    /// Pattern ID
    pub pattern_id: String,
    /// Accept count after update
    pub accept_count: u32,
    /// Whether pattern is now permanent
    pub is_permanent: bool,
    /// Whether pattern just became permanent
    pub became_permanent: bool,
    /// Confidence adjustment from this pattern
    pub confidence_adjustment: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_accept_pattern() {
        let mut store = DubbiosoPatternStore::new(3);

        let update = store.accept_pattern("unwrap() in test", "rust", "src/test.rs", None);
        assert_eq!(update.accept_count, 1);
        assert!(!update.is_permanent);

        store.accept_pattern("unwrap() in test", "rust", "src/test2.rs", None);
        let update2 = store.accept_pattern("unwrap() in test", "rust", "src/test3.rs", None);

        assert!(update2.is_permanent);
        assert!(update2.became_permanent);
    }

    #[test]
    fn test_reject_pattern() {
        let mut store = DubbiosoPatternStore::new(3);

        store.accept_pattern("expect()", "rust", "src/main.rs", None);
        let update = store.reject_pattern("expect()", "rust", "src/main.rs", Some("Not safe here"));

        assert!(update.confidence_adjustment < 0.0);
    }

    #[test]
    fn test_whitelist() {
        let mut store = DubbiosoPatternStore::new(3);

        store.whitelist_pattern("unwrap()", "rust", "Test code");
        assert!(store.is_whitelisted("unwrap()", "rust", "test/foo.rs"));
        assert!(store.is_permanent("unwrap()", "rust"));
    }

    #[test]
    fn test_confidence_adjustment() {
        let mut store = DubbiosoPatternStore::new(3);

        // 3 accepts = positive adjustment
        for _ in 0..3 {
            store.accept_pattern("pattern_a", "rust", "src/lib.rs", None);
        }

        // 1 accept, 3 rejects = negative adjustment
        store.accept_pattern("pattern_b", "rust", "src/lib.rs", None);
        for _ in 0..3 {
            store.reject_pattern("pattern_b", "rust", "src/lib.rs", None);
        }

        let adj_a = store.get_confidence_adjustment("pattern_a", "rust");
        let adj_b = store.get_confidence_adjustment("pattern_b", "rust");

        assert!(adj_a > 0.0);
        assert!(adj_b < 0.0);
    }
}
