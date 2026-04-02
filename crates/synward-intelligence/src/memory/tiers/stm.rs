//! Short-Term Memory (STM) - LRU Cache
//!
//! Fast in-memory cache for recent decisions.
//! TTL: 5 minutes, Capacity: 1000 entries.

use std::sync::Arc;
use std::time::Duration;
use std::num::NonZeroUsize;
use lru::LruCache;

use crate::memory::tier::{DecisionEntry, DecisionId, MemoryTier, TierError};

const STM_CAPACITY: usize = 1000;
const STM_TTL: Duration = Duration::from_secs(5 * 60); // 5 minutes

/// Short-Term Memory: Fast LRU cache
pub struct STM {
    cache: LruCache<DecisionId, Arc<DecisionEntry>>,
    ttl: Duration,
}

impl STM {
    /// Create a new STM with default capacity and TTL
    pub fn new() -> Self {
        Self {
            cache: LruCache::new(NonZeroUsize::new(STM_CAPACITY).unwrap()),
            ttl: STM_TTL,
        }
    }
    
    /// Create STM with custom capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            cache: LruCache::new(NonZeroUsize::new(capacity).unwrap()),
            ttl: STM_TTL,
        }
    }
    
    /// Create STM with custom TTL
    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.ttl = ttl;
        self
    }
    
    /// Get all entry IDs (for debugging)
    pub fn ids(&self) -> Vec<DecisionId> {
        self.cache.iter().map(|(id, _)| id.clone()).collect()
    }
}

impl Default for STM {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryTier for STM {
    fn store(&mut self, entry: DecisionEntry) -> Result<(), TierError> {
        let id = entry.id.clone();
        self.cache.put(id, Arc::new(entry));
        Ok(())
    }
    
    fn retrieve(&mut self, id: &DecisionId) -> Option<Arc<DecisionEntry>> {
        self.cache.get_mut(id).map(|entry| {
            // Update last_accessed on hit
            Arc::make_mut(entry).touch();
            Arc::clone(entry)
        })
    }
    
    fn promote(&mut self, id: &DecisionId, target: &mut dyn MemoryTier) -> Result<(), TierError> {
        let entry = self.cache.pop(id).ok_or_else(|| TierError::NotFound(id.clone()))?;
        target.store((*entry).clone())?;
        Ok(())
    }
    
    fn evict_expired(&mut self) -> Vec<DecisionId> {
        let now = std::time::Instant::now();
        let expired: Vec<_> = self.cache
            .iter()
            .filter(|(_, entry)| now.duration_since(entry.created_at) > self.ttl)
            .map(|(id, _)| id.clone())
            .collect();
        
        for id in &expired {
            self.cache.pop(id);
        }
        expired
    }
    
    fn ttl(&self) -> Duration {
        self.ttl
    }
    
    fn len(&self) -> usize {
        self.cache.len()
    }
    
    fn contains(&self, id: &DecisionId) -> bool {
        self.cache.contains(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::DecisionNode;
    use crate::memory::DecisionType;
    use crate::memory::decision_log::MultiSignalScore;
    
    fn make_entry() -> DecisionEntry {
        DecisionEntry::new(
            DecisionNode::new(DecisionType::Architectural, "test decision"),
            MultiSignalScore::default(),
        )
    }
    
    #[test]
    fn test_stm_store_and_retrieve() {
        let mut stm = STM::new();
        let entry = make_entry();
        let id = entry.id.clone();
        
        stm.store(entry).unwrap();
        assert_eq!(stm.len(), 1);
        
        let retrieved = stm.retrieve(&id);
        assert!(retrieved.is_some());
    }
    
    #[test]
    fn test_stm_lru_eviction() {
        let mut stm = STM::with_capacity(2);
        
        let e1 = make_entry();
        let e2 = make_entry();
        let e3 = make_entry();
        
        let id1 = e1.id.clone();
        let id2 = e2.id.clone();
        
        stm.store(e1).unwrap();
        stm.store(e2).unwrap();
        assert_eq!(stm.len(), 2);
        
        // Adding third should evict least recently used (e1)
        stm.store(e3).unwrap();
        assert_eq!(stm.len(), 2);
        assert!(!stm.contains(&id1));
        assert!(stm.contains(&id2));
    }
    
    #[test]
    fn test_stm_ttl_expiry() {
        let mut stm = STM::new().with_ttl(Duration::from_millis(10));
        let entry = make_entry();
        let id = entry.id.clone();
        
        stm.store(entry).unwrap();
        
        // Wait for TTL
        std::thread::sleep(Duration::from_millis(20));
        
        let evicted = stm.evict_expired();
        assert_eq!(evicted.len(), 1);
        assert_eq!(evicted[0], id);
        assert_eq!(stm.len(), 0);
    }
    
    #[test]
    fn test_stm_promote() {
        let mut stm = STM::new();
        let mut target = STM::new();
        
        let entry = make_entry();
        let id = entry.id.clone();
        
        stm.store(entry).unwrap();
        assert_eq!(stm.len(), 1);
        
        stm.promote(&id, &mut target).unwrap();
        assert_eq!(stm.len(), 0);
        assert_eq!(target.len(), 1);
    }
}
