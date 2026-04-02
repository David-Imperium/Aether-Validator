//! Deduplication System for Memory Entries
//!
//! Phase 5: Detect and merge duplicate entries.
//!
//! ## Strategy
//!
//! - **Exact duplicates**: Hash-based detection (fast)
//! - **Near duplicates**: Semantic similarity via embeddings
//! - **Merge policy**: Keep most recent, combine metadata
//!
//! ## Integration
//!
//! - Works with MemoryStore (LTM tier)
//! - Uses VectorSearch from Phase 4 for semantic matching
//! - Runs during maintenance cycle

use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};

use super::{MemoryId, MemoryEntry};
use crate::memory::store::similarity;

/// Deduplication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DedupConfig {
    /// Threshold for semantic similarity (0.0-1.0)
    /// Entries above this are considered duplicates
    pub semantic_threshold: f32,
    
    /// Minimum entries before dedup runs
    pub min_entries: usize,
    
    /// Maximum entries to compare in one pass
    pub batch_size: usize,
    
    /// Whether to merge metadata from duplicates
    pub merge_metadata: bool,
}

impl Default for DedupConfig {
    fn default() -> Self {
        Self {
            semantic_threshold: 0.85,
            min_entries: 10,
            batch_size: 100,
            merge_metadata: true,
        }
    }
}

/// Result of deduplication pass
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DedupReport {
    /// Number of exact duplicates found
    pub exact_duplicates: usize,
    
    /// Number of semantic duplicates found
    pub semantic_duplicates: usize,
    
    /// Number of entries merged
    pub entries_merged: usize,
    
    /// Space saved (bytes estimate)
    pub bytes_saved: usize,
    
    /// Pairs that were compared
    pub pairs_compared: usize,
}

/// Duplicate detection result
#[derive(Debug, Clone)]
pub struct DuplicatePair {
    /// First entry ID
    pub id1: MemoryId,
    
    /// Second entry ID
    pub id2: MemoryId,
    
    /// Similarity score
    pub similarity: f32,
    
    /// Type of duplicate
    pub dup_type: DuplicateType,
}

/// Type of duplicate detected
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DuplicateType {
    /// Exact match (hash identical)
    Exact,
    
    /// Semantic match (meaning similar)
    Semantic,
}

/// Deduplication engine
pub struct DedupEngine {
    config: DedupConfig,
    /// Hash index for exact duplicates
    hash_index: HashMap<u64, MemoryId>,
    /// Cache of entry hashes
    entry_hashes: HashMap<MemoryId, u64>,
}

impl DedupEngine {
    /// Create new dedup engine with config
    pub fn new(config: DedupConfig) -> Self {
        Self {
            config,
            hash_index: HashMap::new(),
            entry_hashes: HashMap::new(),
        }
    }
    
    /// Create with default config
    pub fn default_config() -> Self {
        Self::new(DedupConfig::default())
    }
    
    /// Compute hash for an entry's code
    fn compute_hash(code: &str) -> u64 {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;
        
        let mut hasher = DefaultHasher::new();
        code.hash(&mut hasher);
        hasher.finish()
    }
    
    /// Index an entry for dedup detection
    pub fn index(&mut self, entry: &MemoryEntry) {
        let hash = Self::compute_hash(&entry.code);
        
        // Update hash index
        if let Some(_existing_id) = self.hash_index.insert(hash, entry.id.clone()) {
            // Hash collision or exact duplicate
            // Keep the more recent one
            self.entry_hashes.insert(entry.id.clone(), hash);
        } else {
            self.entry_hashes.insert(entry.id.clone(), hash);
        }
    }
    
    /// Remove an entry from index
    pub fn remove(&mut self, id: &MemoryId) {
        if let Some(hash) = self.entry_hashes.remove(id) {
            // Only remove from hash_index if it points to this entry
            if let Some(existing_id) = self.hash_index.get(&hash) {
                if existing_id == id {
                    self.hash_index.remove(&hash);
                }
            }
        }
    }
    
    /// Find exact duplicates using hash
    pub fn find_exact_duplicate(&self, entry: &MemoryEntry) -> Option<&MemoryId> {
        let hash = Self::compute_hash(&entry.code);
        self.hash_index.get(&hash).filter(|id| **id != entry.id)
    }
    
    /// Find all duplicates in a set of entries
    pub fn find_duplicates(&self, entries: &[MemoryEntry]) -> Vec<DuplicatePair> {
        let mut duplicates = Vec::new();
        let mut seen = HashSet::new();
        
        // Exact duplicates via hash - compare each pair once
        for (i, entry) in entries.iter().enumerate() {
            let hash = Self::compute_hash(&entry.code);
            
            for other in entries.iter().skip(i + 1) {
                let other_hash = Self::compute_hash(&other.code);
                
                if hash == other_hash {
                    let (id1, id2) = if entry.id.0 < other.id.0 {
                        (entry.id.clone(), other.id.clone())
                    } else {
                        (other.id.clone(), entry.id.clone())
                    };
                    
                    let key = format!("{}:{}", id1.0, id2.0);
                    if seen.insert(key) {
                        duplicates.push(DuplicatePair {
                            id1,
                            id2,
                            similarity: 1.0,
                            dup_type: DuplicateType::Exact,
                        });
                    }
                }
            }
        }
        
        duplicates
    }
    
    /// Find semantic duplicates using similarity
    pub fn find_semantic_duplicates(&self, entries: &[MemoryEntry]) -> Vec<DuplicatePair> {
        let mut duplicates = Vec::new();
        let mut seen = HashSet::new();
        
        for (i, entry) in entries.iter().enumerate() {
            for other in entries.iter().skip(i + 1) {
                let sim = similarity(&entry.code, &other.code);
                
                if sim >= self.config.semantic_threshold {
                    let (id1, id2) = if entry.id.0 < other.id.0 {
                        (entry.id.clone(), other.id.clone())
                    } else {
                        (other.id.clone(), entry.id.clone())
                    };
                    
                    let key = format!("{}:{}", id1.0, id2.0);
                    if seen.insert(key) {
                        duplicates.push(DuplicatePair {
                            id1,
                            id2,
                            similarity: sim,
                            dup_type: DuplicateType::Semantic,
                        });
                    }
                }
            }
        }
        
        duplicates
    }
    
    /// Merge two entries, keeping the best attributes
    pub fn merge_entries(keep: MemoryEntry, discard: &MemoryEntry) -> MemoryEntry {
        let mut merged = keep;
        
        // Keep higher recall count
        merged.recall_count = merged.recall_count.max(discard.recall_count);
        
        // Keep earlier creation time
        if discard.created_at < merged.created_at {
            merged.created_at = discard.created_at;
        }
        
        // Merge feedback if present
        if merged.feedback.is_none() && discard.feedback.is_some() {
            merged.feedback = discard.feedback.clone();
        }
        
        // Merge errors
        for err in &discard.errors {
            if !merged.errors.contains(err) {
                merged.errors.push(err.clone());
            }
        }
        
        merged
    }
    
    /// Run deduplication on a set of entries
    /// Returns IDs to remove and updated entries
    pub fn deduplicate(&self, entries: Vec<MemoryEntry>) -> (Vec<MemoryEntry>, Vec<MemoryId>, DedupReport) {
        let mut report = DedupReport::default();
        let mut to_remove = Vec::new();
        let mut keep_map: HashMap<MemoryId, MemoryEntry> = entries
            .into_iter()
            .map(|e| (e.id.clone(), e))
            .collect();
        
        if keep_map.len() < self.config.min_entries {
            return (keep_map.into_values().collect(), to_remove, report);
        }
        
        // Find exact duplicates
        let exact_dups = self.find_duplicates(
            &keep_map.values().cloned().collect::<Vec<_>>()
        );
        
        report.exact_duplicates = exact_dups.len();
        
        for dup in exact_dups {
            if let (Some(e1), Some(e2)) = (keep_map.get(&dup.id1).cloned(), keep_map.get(&dup.id2)) {
                // Keep more recent, discard older
                let (keep, discard_id) = if e1.created_at >= e2.created_at {
                    (Self::merge_entries(e1, e2), dup.id2.clone())
                } else {
                    let e2 = e2.clone();
                    (Self::merge_entries(e2, &keep_map[&dup.id1].clone()), dup.id1.clone())
                };
                
                keep_map.insert(keep.id.clone(), keep);
                to_remove.push(discard_id);
            }
        }
        
        // Remove discarded from map
        for id in &to_remove {
            keep_map.remove(id);
        }
        
        // Find semantic duplicates among remaining
        let remaining: Vec<_> = keep_map.values().cloned().collect();
        let semantic_dups = self.find_semantic_duplicates(&remaining);
        
        report.semantic_duplicates = semantic_dups.len();
        report.pairs_compared = remaining.len() * (remaining.len() - 1) / 2;
        
        for dup in semantic_dups {
            if to_remove.contains(&dup.id1) || to_remove.contains(&dup.id2) {
                continue;
            }
            
            if let (Some(e1), Some(e2)) = (keep_map.get(&dup.id1).cloned(), keep_map.get(&dup.id2)) {
                let (keep, discard_id) = if e1.created_at >= e2.created_at {
                    (Self::merge_entries(e1, e2), dup.id2.clone())
                } else {
                    let e2 = e2.clone();
                    (Self::merge_entries(e2, &keep_map[&dup.id1].clone()), dup.id1.clone())
                };
                
                keep_map.insert(keep.id.clone(), keep);
                to_remove.push(discard_id);
            }
        }
        
        report.entries_merged = to_remove.len();
        report.bytes_saved = to_remove.len() * 500; // Rough estimate
        
        (keep_map.into_values().collect(), to_remove, report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn make_entry(code: &str) -> MemoryEntry {
        MemoryEntry::new(code, "rust")
    }
    
    fn make_entry_with_id(code: &str, id: &str) -> MemoryEntry {
        let mut entry = MemoryEntry::new(code, "rust");
        entry.id = MemoryId(id.to_string());
        entry
    }
    
    fn test_config() -> DedupConfig {
        DedupConfig {
            semantic_threshold: 0.95, // High threshold - only exact or near-exact matches
            min_entries: 1,
            batch_size: 100,
            merge_metadata: true,
        }
    }
    
    #[test]
    fn test_exact_duplicate_detection() {
        let engine = DedupEngine::default_config();
        
        let e1 = make_entry("fn foo() { }");
        let e2 = make_entry("fn foo() { }");
        let e3 = make_entry("fn bar() { }");
        
        let dups = engine.find_duplicates(&[e1.clone(), e2.clone(), e3]);
        
        assert_eq!(dups.len(), 1);
        assert_eq!(dups[0].dup_type, DuplicateType::Exact);
        assert!((dups[0].similarity - 1.0).abs() < 0.01);
    }
    
    #[test]
    fn test_semantic_duplicate_detection() {
        let config = DedupConfig {
            semantic_threshold: 0.4, // Lower threshold for Jaccard
            ..Default::default()
        };
        let engine = DedupEngine::new(config);
        
        // Use code with significant token overlap
        let e1 = make_entry("fn calculate_age(year: i32) -> i32 { 2024 - year }");
        let e2 = make_entry("fn calculate_age(birth: i32) -> i32 { 2024 - birth }");
        let e3 = make_entry("fn process_data(items: Vec<i32>) { }");
        
        let dups = engine.find_semantic_duplicates(&[e1, e2, e3]);
        
        // e1 and e2 should be semantically similar (same tokens)
        assert!(!dups.is_empty());
        assert_eq!(dups[0].dup_type, DuplicateType::Semantic);
    }
    
    #[test]
    fn test_merge_entries() {
        let mut e1 = make_entry("fn foo() { }");
        e1.recall_count = 5;
        
        let mut e2 = make_entry("fn foo() { }");
        e2.recall_count = 10;
        e2.errors.push("test error".to_string());
        
        let merged = DedupEngine::merge_entries(e1, &e2);
        
        // Should keep higher recall count
        assert_eq!(merged.recall_count, 10);
        // Should merge errors
        assert!(merged.errors.contains(&"test error".to_string()));
    }
    
    #[test]
    fn test_deduplicate_removes_duplicates() {
        let engine = DedupEngine::new(test_config());
        
        let entries = vec![
            make_entry_with_id("fn foo() { }", "1"),
            make_entry_with_id("fn foo() { }", "2"),  // Exact duplicate
            make_entry_with_id("fn bar() { }", "3"),
        ];
        
        let (kept, removed, report) = engine.deduplicate(entries);
        
        assert_eq!(removed.len(), 1);
        assert_eq!(kept.len(), 2);
        assert_eq!(report.exact_duplicates, 1);
    }
    
    #[test]
    fn test_hash_index() {
        let mut engine = DedupEngine::default_config();
        
        let e1 = make_entry("fn foo() { }");
        engine.index(&e1);
        
        let e2 = make_entry("fn foo() { }");
        let dup = engine.find_exact_duplicate(&e2);
        
        assert!(dup.is_some());
    }
    
    #[test]
    fn test_no_duplicates_different_code() {
        let engine = DedupEngine::new(test_config());
        
        let entries = vec![
            make_entry("fn foo() { }"),
            make_entry("fn bar() { }"),
            make_entry("fn baz() { }"),
        ];
        
        let (_, removed, report) = engine.deduplicate(entries);
        
        assert_eq!(removed.len(), 0);
        assert_eq!(report.exact_duplicates, 0);
        assert_eq!(report.semantic_duplicates, 0);
    }
}
