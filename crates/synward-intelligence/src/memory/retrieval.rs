//! Memory Retrieval - Advanced retrieval strategies

use crate::memory::{MemoryEntry, MemoryStore, MemoryType, SimilarityScore};
use crate::Result;

/// Advanced memory retrieval with different strategies
pub struct MemoryRetriever {
    store: MemoryStore,
    default_threshold: f32,
}

impl MemoryRetriever {
    /// Create a new retriever
    pub fn new(store: MemoryStore) -> Self {
        Self {
            store,
            default_threshold: 0.3,
        }
    }

    /// Recall similar code entries
    pub fn recall_similar(&self, code: &str, limit: usize) -> Result<Vec<(MemoryEntry, SimilarityScore)>> {
        let entries = self.store.recall(code, limit * 2)?; // Get more to filter

        let filtered: Vec<_> = entries
            .into_iter()
            .filter_map(|entry| {
                let score = SimilarityScore::new(crate::memory::store::similarity(code, &entry.code));
                if score.is_similar(self.default_threshold) {
                    Some((entry, score))
                } else {
                    None
                }
            })
            .take(limit)
            .collect();

        Ok(filtered)
    }

    /// Recall entries of a specific type
    pub fn recall_by_type(&self, memory_type: MemoryType, limit: usize) -> Result<Vec<MemoryEntry>> {
        let all = self.store.all();

        let filtered: Vec<_> = all
            .into_iter()
            .filter(|e| e.memory_type == memory_type)
            .take(limit)
            .cloned()
            .collect();

        Ok(filtered)
    }

    /// Recall entries for a specific project
    pub fn recall_for_project(&self, project_name: &str, limit: usize) -> Result<Vec<MemoryEntry>> {
        let all = self.store.all();

        let filtered: Vec<_> = all
            .into_iter()
            .filter(|e| e.context.project_name.as_deref() == Some(project_name))
            .take(limit)
            .cloned()
            .collect();

        Ok(filtered)
    }

    /// Get the most frequently recalled entries (popular)
    pub fn get_popular(&self, limit: usize) -> Result<Vec<MemoryEntry>> {
        let mut all = self.store.all().to_vec();
        all.sort_by(|a, b| b.recall_count.cmp(&a.recall_count));

        Ok(all.into_iter().take(limit).cloned().collect())
    }

    /// Get recent entries
    pub fn get_recent(&self, limit: usize) -> Result<Vec<MemoryEntry>> {
        let mut all = self.store.all().to_vec();
        all.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        Ok(all.into_iter().rev().take(limit).cloned().collect())
    }
}
