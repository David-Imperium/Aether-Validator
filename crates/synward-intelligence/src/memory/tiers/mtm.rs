//! Medium-Term Memory (MTM) - JSON File Storage
//!
//! Intermediate storage between STM and LTM.
//! TTL: 1 hour, persisted to JSON file.

use std::sync::Arc;
use std::time::Duration;
use std::collections::HashMap;
use std::path::PathBuf;
use std::fs;

use crate::memory::tier::{DecisionEntry, DecisionId, MemoryTier, TierError};

const MTM_TTL: Duration = Duration::from_secs(60 * 60); // 1 hour

/// Medium-Term Memory: File-based intermediate storage
pub struct MTM {
    path: PathBuf,
    entries: HashMap<DecisionId, Arc<DecisionEntry>>,
    ttl: Duration,
}

impl MTM {
    /// Create a new MTM with default TTL
    pub fn new(path: PathBuf) -> Self {
        let entries = Self::load_from_disk(&path).unwrap_or_default();
        Self {
            path,
            entries,
            ttl: MTM_TTL,
        }
    }
    
    /// Create MTM with custom TTL
    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.ttl = ttl;
        self
    }
    
    /// Get all entry IDs
    pub fn ids(&self) -> Vec<DecisionId> {
        self.entries.keys().cloned().collect()
    }
    
    fn load_from_disk(path: &PathBuf) -> Result<HashMap<DecisionId, Arc<DecisionEntry>>, TierError> {
        if !path.exists() {
            return Ok(HashMap::new());
        }
        
        let data = fs::read(path)?;
        let entries: Vec<DecisionEntry> = serde_json::from_slice(&data)?;
        
        Ok(entries
            .into_iter()
            .map(|e| (e.id.clone(), Arc::new(e)))
            .collect())
    }
    
    fn save_to_disk(&self) -> Result<(), TierError> {
        // Ensure parent directory exists
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let entries: Vec<_> = self.entries.values().map(|e| (**e).clone()).collect();
        let data = serde_json::to_vec_pretty(&entries)?;
        fs::write(&self.path, data)?;
        
        Ok(())
    }
}

impl MemoryTier for MTM {
    fn store(&mut self, entry: DecisionEntry) -> Result<(), TierError> {
        let id = entry.id.clone();
        self.entries.insert(id, Arc::new(entry));
        self.save_to_disk()?;
        Ok(())
    }
    
    fn retrieve(&mut self, id: &DecisionId) -> Option<Arc<DecisionEntry>> {
        self.entries.get_mut(id).map(|entry| {
            Arc::make_mut(entry).touch();
            Arc::clone(entry)
        })
    }
    
    fn promote(&mut self, id: &DecisionId, target: &mut dyn MemoryTier) -> Result<(), TierError> {
        let entry = self.entries.remove(id)
            .ok_or_else(|| TierError::NotFound(id.clone()))?;
        
        target.store((*entry).clone())?;
        self.save_to_disk()?;
        
        Ok(())
    }
    
    fn evict_expired(&mut self) -> Vec<DecisionId> {
        let now = std::time::Instant::now();
        
        let expired: Vec<_> = self.entries
            .iter()
            .filter(|(_, entry)| now.duration_since(entry.created_at) > self.ttl)
            .map(|(id, _)| id.clone())
            .collect();
        
        for id in &expired {
            self.entries.remove(id);
        }
        
        if !expired.is_empty() {
            let _ = self.save_to_disk();
        }
        
        expired
    }
    
    fn ttl(&self) -> Duration {
        self.ttl
    }
    
    fn len(&self) -> usize {
        self.entries.len()
    }
    
    fn contains(&self, id: &DecisionId) -> bool {
        self.entries.contains_key(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::DecisionNode;
    use crate::memory::DecisionType;
    use crate::memory::decision_log::MultiSignalScore;
    use tempfile::tempdir;
    
    fn make_entry() -> DecisionEntry {
        DecisionEntry::new(
            DecisionNode::new(DecisionType::Architectural, "test decision"),
            MultiSignalScore::default(),
        )
    }
    
    #[test]
    fn test_mtm_store_and_retrieve() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("mtm.json");
        
        let mut mtm = MTM::new(path.clone());
        let entry = make_entry();
        let id = entry.id.clone();
        
        mtm.store(entry).unwrap();
        assert_eq!(mtm.len(), 1);
        
        // Verify persistence
        let mtm2 = MTM::new(path);
        assert!(mtm2.contains(&id));
    }
    
    #[test]
    fn test_mtm_eviction() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("mtm.json");
        
        let mut mtm = MTM::new(path).with_ttl(Duration::from_millis(10));
        let entry = make_entry();
        let id = entry.id.clone();
        
        mtm.store(entry).unwrap();
        
        // Wait for TTL
        std::thread::sleep(Duration::from_millis(20));
        
        let evicted = mtm.evict_expired();
        assert_eq!(evicted.len(), 1);
        assert_eq!(evicted[0], id);
    }
    
    #[test]
    fn test_mtm_promote() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("mtm.json");
        
        let mut mtm = MTM::new(path);
        let mut stm = crate::memory::STM::new();
        
        let entry = make_entry();
        let id = entry.id.clone();
        
        mtm.store(entry).unwrap();
        mtm.promote(&id, &mut stm).unwrap();
        
        assert_eq!(mtm.len(), 0);
        assert_eq!(stm.len(), 1);
    }
}
