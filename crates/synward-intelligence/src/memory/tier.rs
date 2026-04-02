//! Memory Tier Trait - Abstract interface for memory hierarchy
//!
//! Defines the contract for STM, MTM, and LTM implementations.

use std::sync::Arc;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

use super::DecisionNode;
use super::decision_log::MultiSignalScore;

/// Unique identifier for a decision entry
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct DecisionId(pub String);

impl Default for DecisionId {
    fn default() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
}

impl std::fmt::Display for DecisionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A scored decision entry ready for storage in memory hierarchy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionEntry {
    /// Unique identifier
    pub id: DecisionId,
    
    /// The decision node from DecisionLog
    pub decision: DecisionNode,
    
    /// Multi-signal score (computed in executor)
    pub score: MultiSignalScore,
    
    /// When this entry was created (for TTL)
    #[serde(with = "instant_ser")]
    pub created_at: Instant,
    
    /// When this entry was last accessed (for LRU)
    #[serde(with = "instant_ser")]
    pub last_accessed: Instant,
}

impl DecisionEntry {
    /// Create a new entry from a decision node and score
    pub fn new(decision: DecisionNode, score: MultiSignalScore) -> Self {
        let now = Instant::now();
        Self {
            id: DecisionId::default(),
            decision,
            score,
            created_at: now,
            last_accessed: now,
        }
    }
    
    /// Check if this entry has expired given a TTL
    pub fn is_expired(&self, ttl: Duration) -> bool {
        Instant::now().duration_since(self.created_at) > ttl
    }
    
    /// Update last accessed timestamp
    pub fn touch(&mut self) {
        self.last_accessed = Instant::now();
    }
}

/// Core trait for all memory tiers
pub trait MemoryTier: Send + Sync {
    /// Store a new entry in this tier
    fn store(&mut self, entry: DecisionEntry) -> Result<(), TierError>;
    
    /// Retrieve an entry by ID, updates last_accessed
    fn retrieve(&mut self, id: &DecisionId) -> Option<Arc<DecisionEntry>>;
    
    /// Move entry to another tier (promotion)
    fn promote(&mut self, id: &DecisionId, target: &mut dyn MemoryTier) -> Result<(), TierError>;
    
    /// Evict expired entries, returns evicted IDs
    fn evict_expired(&mut self) -> Vec<DecisionId>;
    
    /// Get this tier's TTL
    fn ttl(&self) -> Duration;
    
    /// Count entries in this tier
    fn len(&self) -> usize;
    
    /// Check if tier is empty
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    
    /// Check if entry exists
    fn contains(&self, id: &DecisionId) -> bool;
}

/// Errors for tier operations
#[derive(Debug, thiserror::Error)]
pub enum TierError {
    #[error("Entry not found: {0}")]
    NotFound(DecisionId),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Tier capacity exceeded")]
    CapacityExceeded,
    
    #[error("LTM is terminal tier - cannot promote out")]
    TerminalTier,
    
    #[error("Memory store error: {0}")]
    MemoryStore(#[from] crate::error::Error),

    #[error("Tier not available: {0}")]
    NotAvailable(String),
}

/// Serialization helper for Instant
mod instant_ser {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::{Duration, Instant};
    
    pub fn serialize<S>(instant: &Instant, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Store as duration since an epoch (relative)
        let duration = instant.elapsed();
        duration.as_secs_f64().serialize(serializer)
    }
    
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Instant, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs: f64 = Deserialize::deserialize(deserializer)?;
        // Reconstruct as Instant::now() - duration
        Ok(Instant::now() - Duration::from_secs_f64(secs))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::DecisionType;
    
    fn make_test_node() -> DecisionNode {
        DecisionNode::new(DecisionType::Architectural, "test decision")
    }
    
    #[test]
    fn test_decision_entry_expiration() {
        let entry = DecisionEntry::new(
            make_test_node(),
            MultiSignalScore::default(),
        );
        
        // Fresh entry should not be expired
        assert!(!entry.is_expired(Duration::from_secs(60)));
        
        // Simulate aging (can't actually wait in tests)
        // In real use, Instant::now() advances naturally
    }
}
