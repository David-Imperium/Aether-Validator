//! Layer 2B: Decision Log (Knowledge Graph)
//!
//! Tracks architectural decisions, intent, and user feedback.
//! Solves the "Context Rot" problem by maintaining a persistent
//! knowledge graph of *why* code exists and decisions made.
//!
//! ## v3.0 Features
//!
//! - **Temporal Decision Log**: Track decision evolution with `superseded_by`/`supersedes`
//! - **Multi-Signal Scoring**: Composite score from relevance, recency, importance
//! - **Audit Trail**: Complete history for compliance

use crate::error::{Error, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::fs;

use super::MemoryId;
use super::scope::MemoryPath;

/// Multi-signal score for v3.0
///
/// Composite score combining multiple signals:
/// ```text
/// score = α*relevance + β*recency + γ*importance
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiSignalScore {
    /// Relevance to current context (0.0-1.0)
    pub relevance: f32,

    /// Recency score with decay (0.0-1.0)
    pub recency: f32,

    /// Importance weight (0.0-1.0)
    pub importance: f32,

    /// Composite score (0.0-1.0)
    pub score: f32,

    /// When this score was computed
    pub computed_at: DateTime<Utc>,
}

impl Default for MultiSignalScore {
    fn default() -> Self {
        Self {
            relevance: 0.5,
            recency: 1.0,
            importance: 0.5,
            score: 0.67,
            computed_at: Utc::now(),
        }
    }
}

impl MultiSignalScore {
    /// Compute multi-signal score with configurable weights
    ///
    /// Default weights: α=0.4, β=0.3, γ=0.3
    pub fn compute(
        relevance: f32,
        timestamp: DateTime<Utc>,
        importance: f32,
        now: DateTime<Utc>,
    ) -> Self {
        let recency = Self::decay(timestamp, now);

        // Default weights
        let alpha = 0.4;
        let beta = 0.3;
        let gamma = 0.3;

        let score = alpha * relevance + beta * recency + gamma * importance;

        Self {
            relevance,
            recency,
            importance,
            score: score.clamp(0.0, 1.0),
            computed_at: now,
        }
    }

    /// Decay function for recency (half-life 30 days)
    ///
    /// ```text
    /// recency = 0.5^((now - timestamp) / half_life_days)
    /// ```
    fn decay(timestamp: DateTime<Utc>, now: DateTime<Utc>) -> f32 {
        let half_life_days = 30.0;
        let age_days = (now - timestamp).num_days() as f32;

        if age_days <= 0.0 {
            1.0
        } else {
            0.5_f32.powf(age_days / half_life_days)
        }
    }

    /// Update score with new weights
    pub fn with_weights(mut self, alpha: f32, beta: f32, gamma: f32) -> Self {
        self.score = (alpha * self.relevance + beta * self.recency + gamma * self.importance)
            .clamp(0.0, 1.0);
        self.computed_at = Utc::now();
        self
    }
}

/// A decision node in the knowledge graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionNode {
    /// Unique identifier
    pub id: MemoryId,

    /// Type of decision
    pub decision_type: DecisionType,

    /// The decision content (e.g., "Use unwrap here because...")
    pub content: String,

    /// Code location this decision applies to
    pub location: CodeLocation,

    /// When this decision was made
    pub timestamp: DateTime<Utc>,

    /// Who made this decision (user, ai, auto)
    pub author: DecisionAuthor,

    /// Current status
    pub status: DecisionStatus,

    /// Related decisions (edges in the graph)
    pub related: Vec<MemoryId>,

    /// User confidence in this decision (0.0-1.0)
    pub confidence: f32,

    /// Tags for categorization
    pub tags: Vec<String>,

    // ===== v3.0 Temporal Fields =====

    /// ID of the decision that supersedes this one (if superseded)
    #[serde(default)]
    pub superseded_by: Option<MemoryId>,

    /// IDs of decisions that this one supersedes
    #[serde(default)]
    pub supersedes: Vec<MemoryId>,

    /// Multi-signal score (v3.0)
    #[serde(default)]
    pub multi_signal: Option<MultiSignalScore>,

    /// Version of this decision (incremented on supersede)
    #[serde(default)]
    pub version: u32,
}

impl DecisionNode {
    /// Create a new decision node
    pub fn new(decision_type: DecisionType, content: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: MemoryId::default(),
            decision_type,
            content: content.into(),
            location: CodeLocation::default(),
            timestamp: now,
            author: DecisionAuthor::Auto,
            status: DecisionStatus::Active,
            related: Vec::new(),
            confidence: 0.5,
            tags: Vec::new(),
            // v3.0 fields
            superseded_by: None,
            supersedes: Vec::new(),
            multi_signal: Some(MultiSignalScore::compute(0.5, now, 0.5, now)),
            version: 1,
        }
    }

    /// Create a decision that supersedes an existing one
    ///
    /// This is the main entry point for temporal decision evolution.
    /// The new decision will:
    /// - Have `supersedes` set to the old decision
    /// - Have `version = old.version + 1`
    /// - Mark the old decision as `Superseded` with `superseded_by` set
    pub fn supersede(old: &DecisionNode, new_content: impl Into<String>) -> Self {
        let now = Utc::now();
        let new_id = MemoryId::default();

        // Compute score for new decision (inherit importance from old)
        let importance = old.multi_signal.as_ref()
            .map(|s| s.importance)
            .unwrap_or(0.5);
        let multi_signal = MultiSignalScore::compute(0.8, now, importance, now);

        Self {
            id: new_id.clone(),
            decision_type: old.decision_type,
            content: new_content.into(),
            location: old.location.clone(),
            timestamp: now,
            author: DecisionAuthor::User, // Usually user-initiated
            status: DecisionStatus::Active,
            related: old.related.clone(),
            confidence: 0.7,
            tags: old.tags.clone(),
            // v3.0 temporal fields
            superseded_by: None,
            supersedes: vec![old.id.clone()],
            multi_signal: Some(multi_signal),
            version: old.version + 1,
        }
    }

    /// Get the full audit trail (supersession chain)
    ///
    /// Returns the chain of decisions from oldest to newest.
    /// The last element is always `self`.
    pub fn audit_trail<'a>(&'a self, log: &'a DecisionLog) -> Vec<&'a DecisionNode> {
        let mut trail = Vec::new();

        // First, collect all superseded decisions (recursively)
        fn collect_superseded<'a>(
            node: &'a DecisionNode,
            log: &'a DecisionLog,
            trail: &mut Vec<&'a DecisionNode>,
        ) {
            for old_id in &node.supersedes {
                if let Some(old_node) = log.get(old_id) {
                    collect_superseded(old_node, log, trail);
                    trail.push(old_node);
                }
            }
        }

        collect_superseded(self, log, &mut trail);
        trail.push(self);
        trail
    }

    /// Update the multi-signal score
    pub fn update_score(&mut self, relevance: f32, importance: f32) {
        self.multi_signal = Some(MultiSignalScore::compute(
            relevance,
            self.timestamp,
            importance,
            Utc::now(),
        ));
    }

    /// Set the code location
    pub fn at(mut self, file: impl Into<String>, line: usize) -> Self {
        self.location = CodeLocation {
            file: file.into(),
            line,
            column: 0,
        };
        self
    }

    /// Set the author
    pub fn by(mut self, author: DecisionAuthor) -> Self {
        self.author = author;
        self
    }

    /// Set confidence
    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }

    /// Add a related decision
    pub fn related_to(mut self, id: MemoryId) -> Self {
        self.related.push(id);
        self
    }

    /// Add tags
    pub fn with_tags(mut self, tags: &[&str]) -> Self {
        self.tags.extend(tags.iter().map(|s| s.to_string()));
        self
    }

    /// Set the status
    pub fn with_status(mut self, status: DecisionStatus) -> Self {
        self.status = status;
        self
    }

    /// Check if this decision accepts a violation
    pub fn accepts(&self, violation_id: &str) -> bool {
        self.status == DecisionStatus::Accepted
            && self.decision_type == DecisionType::AcceptViolation
            && self.tags.contains(&violation_id.to_string())
    }
}

/// Type of decision recorded
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DecisionType {
    /// Architectural decision (e.g., "Use async for I/O")
    Architectural,

    /// Design pattern choice
    PatternChoice,

    /// Accepting a violation with justification
    AcceptViolation,

    /// Code intent explanation
    IntentExplanation,

    /// Refactoring decision
    Refactoring,

    /// Dependency choice
    Dependency,

    /// Performance decision
    Performance,

    /// Security decision
    Security,
}

/// Who made the decision
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DecisionAuthor {
    /// Human user
    User,

    /// AI assistant
    Ai,

    /// Auto-detected from code
    Auto,
}

/// Status of a decision
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DecisionStatus {
    /// Active and valid
    Active,

    /// Accepted by user
    Accepted,

    /// Rejected by user
    Rejected,

    /// Superseded by another decision
    Superseded,

    /// Under review
    Pending,
}

/// Code location for a decision
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct CodeLocation {
    pub file: String,
    pub line: usize,
    pub column: usize,
}


/// Edge between decisions (relationship)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionEdge {
    pub from: MemoryId,
    pub to: MemoryId,
    pub relation: DecisionRelation,
}

/// Type of relationship between decisions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DecisionRelation {
    /// One decision caused another
    Causes,

    /// One decision conflicts with another
    Conflicts,

    /// One decision supersedes another
    Supersedes,

    /// Decisions are related (same topic)
    Related,

    /// One decision depends on another
    DependsOn,
}

/// The decision log (knowledge graph)
#[derive(Debug)]
pub struct DecisionLog {
    /// All decision nodes
    nodes: HashMap<MemoryId, DecisionNode>,

    /// Edges between decisions
    edges: Vec<DecisionEdge>,

    /// Persistent storage path
    path: PathBuf,

    /// RAG store for semantic queries (optional)
    rag_store: Option<super::MemoryStore>,
}

impl DecisionLog {
    /// Create a new decision log
    pub fn new(path: Option<PathBuf>) -> Result<Self> {
        let path = path.unwrap_or_else(|| {
            MemoryPath::global_base()
                .join("decisions.json")
        });

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(Error::Io)?;
        }

        let mut log = Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
            path,
            rag_store: None,
        };

        log.load()?;
        Ok(log)
    }

    /// Attach a RAG store for semantic queries
    pub fn with_rag(mut self, store: super::MemoryStore) -> Self {
        self.rag_store = Some(store);
        self
    }

    /// Record a new decision
    pub fn record(&mut self, node: DecisionNode) -> Result<MemoryId> {
        let id = node.id.clone();
        self.nodes.insert(id.clone(), node);
        self.persist()?;
        Ok(id)
    }

    /// Query: Why does this code exist?
    pub fn why_exists(&self, file: &str, line: usize) -> Vec<&DecisionNode> {
        self.nodes
            .values()
            .filter(|n| n.location.file == file && n.location.line == line)
            .filter(|n| n.status == DecisionStatus::Active || n.status == DecisionStatus::Accepted)
            .collect()
    }

    /// Query: Is this violation accepted?
    pub fn is_accepted(&self, violation_id: &str) -> Option<&DecisionNode> {
        self.nodes
            .values()
            .find(|n| n.accepts(violation_id))
    }

    /// Query: Get all decisions for a file
    pub fn for_file(&self, file: &str) -> Vec<&DecisionNode> {
        self.nodes
            .values()
            .filter(|n| n.location.file == file)
            .collect()
    }

    /// Query: Semantic search for related decisions
    pub fn recall_semantic(&self, query: &str, limit: usize) -> Vec<&DecisionNode> {
        // If RAG store is available, use semantic search
        if let Some(ref _rag) = self.rag_store {
            // TODO: Implement semantic search via RAG
            // For now, fall back to keyword search
        }

        // Keyword-based search
        let query_lower = query.to_lowercase();
        let mut scored: Vec<_> = self.nodes
            .values()
            .filter(|n| n.status == DecisionStatus::Active || n.status == DecisionStatus::Accepted)
            .map(|n| {
                let content_lower = n.content.to_lowercase();
                let score = if content_lower.contains(&query_lower) {
                    1.0
                } else {
                    n.tags.iter()
                        .filter(|t| t.to_lowercase().contains(&query_lower))
                        .count() as f32 * 0.5
                };
                (score, n)
            })
            .filter(|(score, _)| *score > 0.0)
            .collect();

        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        scored.into_iter().take(limit).map(|(_, n)| n).collect()
    }

    /// Update decision status
    pub fn set_status(&mut self, id: &MemoryId, status: DecisionStatus) -> Result<()> {
        if let Some(node) = self.nodes.get_mut(id) {
            node.status = status;
            self.persist()?;
        }
        Ok(())
    }

    /// Add a relationship between decisions
    pub fn relate(&mut self, from: MemoryId, to: MemoryId, relation: DecisionRelation) -> Result<()> {
        self.edges.push(DecisionEdge { from, to, relation });
        self.persist()
    }

    // ===== v3.0 Temporal Methods =====

    /// Supersede a decision with a new one
    ///
    /// This creates a temporal link between decisions:
    /// 1. Creates new decision with `supersedes` pointing to old
    /// 2. Updates old decision with `superseded_by` pointing to new
    /// 3. Marks old decision as `Superseded`
    /// 4. Adds edge with `Supersedes` relation
    ///
    /// Returns the ID of the new decision.
    pub fn supersede_decision(
        &mut self,
        old_id: &MemoryId,
        new_content: impl Into<String>,
    ) -> Result<MemoryId> {
        // Get old decision
        let old_node = self.nodes.get(old_id).cloned()
            .ok_or_else(|| Error::NotFound(format!("Decision {:?} not found", old_id)))?;

        // Create new decision that supersedes the old one
        let new_node = DecisionNode::supersede(&old_node, new_content);
        let new_id = new_node.id.clone();

        // Update old decision
        if let Some(old) = self.nodes.get_mut(old_id) {
            old.superseded_by = Some(new_id.clone());
            old.status = DecisionStatus::Superseded;
        }

        // Record new decision
        self.nodes.insert(new_id.clone(), new_node);

        // Add edge
        self.edges.push(DecisionEdge {
            from: new_id.clone(),
            to: old_id.clone(),
            relation: DecisionRelation::Supersedes,
        });

        self.persist()?;

        tracing::info!(
            "Decision {:?} superseded by {:?} (v{})",
            old_id,
            new_id,
            self.nodes.get(&new_id).map(|n| n.version).unwrap_or(1)
        );

        Ok(new_id)
    }

    /// Get a decision by ID
    pub fn get(&self, id: &MemoryId) -> Option<&DecisionNode> {
        self.nodes.get(id)
    }

    /// Get the latest (non-superseded) decision in a chain
    ///
    /// Follows `superseded_by` links to find the current decision.
    pub fn get_latest(&self, id: &MemoryId) -> Option<&DecisionNode> {
        let node = self.nodes.get(id)?;

        match &node.superseded_by {
            Some(newer_id) => self.get_latest(newer_id),
            None => Some(node),
        }
    }

    /// Get audit trail for a decision
    ///
    /// Returns the full history from oldest to newest.
    pub fn audit_trail(&self, id: &MemoryId) -> Vec<&DecisionNode> {
        let node = match self.get_latest(id) {
            Some(n) => n,
            None => return Vec::new(),
        };

        node.audit_trail(self)
    }

    /// Get all superseded decisions (for reporting)
    pub fn superseded(&self) -> Vec<&DecisionNode> {
        self.nodes
            .values()
            .filter(|n| n.status == DecisionStatus::Superseded)
            .collect()
    }

    /// Get all active decisions (not superseded)
    pub fn active(&self) -> Vec<&DecisionNode> {
        self.nodes
            .values()
            .filter(|n| n.status == DecisionStatus::Active || n.status == DecisionStatus::Accepted)
            .collect()
    }

    /// Update scores for all decisions
    ///
    /// Recomputes multi-signal scores based on current time.
    pub fn update_all_scores(&mut self) -> Result<()> {
        let now = Utc::now();

        for node in self.nodes.values_mut() {
            let relevance = node.multi_signal.as_ref()
                .map(|s| s.relevance)
                .unwrap_or(0.5);
            let importance = node.multi_signal.as_ref()
                .map(|s| s.importance)
                .unwrap_or(0.5);

            node.multi_signal = Some(MultiSignalScore::compute(
                relevance,
                node.timestamp,
                importance,
                now,
            ));
        }

        self.persist()
    }

    /// Query with multi-signal ranking (v3.0)
    ///
    /// Returns decisions sorted by composite score.
    pub fn recall_ranked(&self, query: &str, limit: usize) -> Vec<&DecisionNode> {
        let query_lower = query.to_lowercase();

        let mut scored: Vec<_> = self.nodes
            .values()
            .filter(|n| n.status == DecisionStatus::Active || n.status == DecisionStatus::Accepted)
            .filter_map(|n| {
                // Keyword match
                let content_lower = n.content.to_lowercase();
                let keyword_score = if content_lower.contains(&query_lower) {
                    1.0
                } else {
                    n.tags.iter()
                        .filter(|t| t.to_lowercase().contains(&query_lower))
                        .count() as f32 * 0.5
                };

                if keyword_score > 0.0 {
                    // Use multi-signal score if available
                    let score = n.multi_signal.as_ref()
                        .map(|s| s.score * keyword_score)
                        .unwrap_or(keyword_score);
                    Some((score, n))
                } else {
                    None
                }
            })
            .collect();

        // Sort by score descending
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        scored.into_iter().take(limit).map(|(_, n)| n).collect()
    }

    /// Get related decisions
    pub fn get_related(&self, id: &MemoryId) -> Vec<(&DecisionNode, DecisionRelation)> {
        self.edges
            .iter()
            .filter(|e| &e.from == id || &e.to == id)
            .filter_map(|e| {
                if &e.from == id {
                    self.nodes.get(&e.to).map(|n| (n, e.relation))
                } else {
                    self.nodes.get(&e.from).map(|n| (n, e.relation))
                }
            })
            .collect()
    }

    /// Get all decisions
    pub fn all(&self) -> Vec<&DecisionNode> {
        self.nodes.values().collect()
    }

    /// Count decisions
    pub fn count(&self) -> usize {
        self.nodes.len()
    }

    /// Clear all decisions
    pub fn clear(&mut self) -> Result<()> {
        self.nodes.clear();
        self.edges.clear();
        self.persist()
    }

    /// Load from disk
    fn load(&mut self) -> Result<()> {
        if !self.path.exists() {
            return Ok(());
        }

        #[derive(Deserialize)]
        struct Serialized {
            nodes: Vec<DecisionNode>,
            edges: Vec<DecisionEdge>,
        }

        let content = fs::read_to_string(&self.path).map_err(Error::Io)?;
        let data: Serialized = serde_json::from_str(&content)?;

        for node in data.nodes {
            self.nodes.insert(node.id.clone(), node);
        }
        self.edges = data.edges;

        tracing::info!("Loaded {} decision nodes from {:?}", self.nodes.len(), self.path);
        Ok(())
    }

    /// Persist to disk
    fn persist(&self) -> Result<()> {
        #[derive(Serialize)]
        struct Serialized<'a> {
            nodes: Vec<&'a DecisionNode>,
            edges: &'a [DecisionEdge],
        }

        let nodes: Vec<_> = self.nodes.values().collect();
        let data = Serialized {
            nodes,
            edges: &self.edges,
        };

        let content = serde_json::to_string_pretty(&data)?;
        fs::write(&self.path, content).map_err(Error::Io)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decision_node_creation() {
        let node = DecisionNode::new(
            DecisionType::Architectural,
            "Use async for all I/O operations"
        )
        .at("src/main.rs", 42)
        .by(DecisionAuthor::User)
        .with_confidence(0.9);

        assert_eq!(node.decision_type, DecisionType::Architectural);
        assert_eq!(node.location.file, "src/main.rs");
        assert_eq!(node.location.line, 42);
        assert_eq!(node.author, DecisionAuthor::User);
    }

    #[test]
    fn test_accept_violation() {
        let node = DecisionNode::new(
            DecisionType::AcceptViolation,
            "Allow unwrap here because value is guaranteed by config"
        )
        .at("src/config.rs", 15)
        .with_tags(&["UNWRAP001"]);

        let accepted = DecisionNode {
            status: DecisionStatus::Accepted,
            ..node
        };

        assert!(accepted.accepts("UNWRAP001"));
        assert!(!accepted.accepts("OTHER001"));
    }

    #[test]
    fn test_decision_log_roundtrip() {
        use std::env::temp_dir;
        let temp_path = temp_dir().join(format!("aether_test_decisions_{}.json", std::process::id()));

        let mut log = DecisionLog::new(Some(temp_path.clone())).unwrap();

        let node = DecisionNode::new(
            DecisionType::IntentExplanation,
            "This function handles user authentication"
        )
        .at("src/auth.rs", 10);

        log.record(node).unwrap();

        let found = log.why_exists("src/auth.rs", 10);
        assert_eq!(found.len(), 1);

        // Cleanup
        std::fs::remove_file(&temp_path).ok();
    }

    #[test]
    fn test_semantic_recall() {
        // Use a temporary path to avoid loading persistent decisions
        let temp_dir = std::env::temp_dir();
        let temp_path = temp_dir.join("aether_test_decisions_semantic.json");
        let mut log = DecisionLog::new(Some(temp_path)).unwrap();
        log.clear().unwrap(); // Ensure clean state

        log.record(DecisionNode::new(
            DecisionType::Architectural,
            "Use PostgreSQL for persistence"
        ).with_tags(&["database", "persistence"])).unwrap();

        log.record(DecisionNode::new(
            DecisionType::Security,
            "Encrypt sensitive data before storage"
        ).with_tags(&["encryption", "security"])).unwrap();

        let found = log.recall_semantic("database", 5);
        assert_eq!(found.len(), 1);
        assert!(found[0].content.contains("PostgreSQL"));
    }

    // ===== v3.0 Temporal Tests =====

    #[test]
    fn test_multi_signal_score_decay() {
        let now = Utc::now();
        let one_hour_ago = now - chrono::Duration::hours(1);
        let thirty_days_ago = now - chrono::Duration::days(30);
        let sixty_days_ago = now - chrono::Duration::days(60);

        // Fresh decision
        let fresh = MultiSignalScore::compute(1.0, one_hour_ago, 0.5, now);
        assert!(fresh.recency > 0.99);

        // 30 days old (half-life)
        let half_life = MultiSignalScore::compute(1.0, thirty_days_ago, 0.5, now);
        assert!((half_life.recency - 0.5).abs() < 0.01);

        // 60 days old
        let old = MultiSignalScore::compute(1.0, sixty_days_ago, 0.5, now);
        assert!(old.recency < 0.3);
    }

    #[test]
    fn test_multi_signal_composite() {
        let now = Utc::now();
        let score = MultiSignalScore::compute(0.8, now, 0.6, now);

        // Default weights: α=0.4, β=0.3, γ=0.3
        // score = 0.4*0.8 + 0.3*1.0 + 0.3*0.6 = 0.32 + 0.3 + 0.18 = 0.8
        assert!((score.score - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_supersede_decision() {
        let temp_dir = std::env::temp_dir();
        let temp_path = temp_dir.join("aether_test_supersede.json");
        let mut log = DecisionLog::new(Some(temp_path)).unwrap();
        log.clear().unwrap();

        // Create initial decision
        let old_node = DecisionNode::new(
            DecisionType::Architectural,
            "Use synchronous I/O for simplicity"
        )
        .at("src/main.rs", 42)
        .by(DecisionAuthor::User);

        let old_id = log.record(old_node).unwrap();

        // Supersede it
        let new_id = log.supersede_decision(
            &old_id,
            "Actually, use async I/O for scalability"
        ).unwrap();

        // Verify old decision is superseded
        let old = log.get(&old_id).unwrap();
        assert_eq!(old.status, DecisionStatus::Superseded);
        assert_eq!(old.superseded_by, Some(new_id.clone()));

        // Verify new decision has correct links
        let new = log.get(&new_id).unwrap();
        assert_eq!(new.supersedes, vec![old_id.clone()]);
        assert_eq!(new.version, 2);
        assert_eq!(new.status, DecisionStatus::Active);
    }

    #[test]
    fn test_supersede_chain() {
        let temp_dir = std::env::temp_dir();
        let temp_path = temp_dir.join("aether_test_chain.json");
        let mut log = DecisionLog::new(Some(temp_path)).unwrap();
        log.clear().unwrap();

        // Create chain: v1 -> v2 -> v3
        let v1 = DecisionNode::new(DecisionType::Security, "Use basic auth")
            .at("src/auth.rs", 10);
        let v1_id = log.record(v1).unwrap();

        let v2_id = log.supersede_decision(&v1_id, "Use JWT tokens").unwrap();
        let v3_id = log.supersede_decision(&v2_id, "Use JWT with refresh tokens").unwrap();

        // Verify get_latest follows chain
        let latest = log.get_latest(&v1_id).unwrap();
        assert_eq!(latest.id, v3_id);
        assert_eq!(latest.version, 3);

        // Verify audit trail
        let trail = log.audit_trail(&v1_id);
        assert_eq!(trail.len(), 3);
        assert_eq!(trail[0].version, 1);
        assert_eq!(trail[1].version, 2);
        assert_eq!(trail[2].version, 3);
    }

    #[test]
    fn test_recall_ranked() {
        let temp_dir = std::env::temp_dir();
        let temp_path = temp_dir.join("aether_test_ranked.json");
        let mut log = DecisionLog::new(Some(temp_path)).unwrap();
        log.clear().unwrap();

        // Create decisions with different ages
        let old_node = DecisionNode::new(DecisionType::Architectural, "Use database")
            .with_tags(&["persistence"]);
        // Manually set old timestamp
        let mut old = old_node;
        old.timestamp = Utc::now() - chrono::Duration::days(60);
        old.multi_signal = Some(MultiSignalScore::compute(0.8, old.timestamp, 0.5, Utc::now()));
        log.record(old).unwrap();

        let new_node = DecisionNode::new(DecisionType::Architectural, "Use PostgreSQL database")
            .with_tags(&["persistence"]);
        log.record(new_node).unwrap();

        // Search should rank newer higher due to recency
        let results = log.recall_ranked("database", 5);
        assert_eq!(results.len(), 2);
        // Newer should be first (higher recency score)
        assert!(results[0].content.contains("PostgreSQL"));
    }
}
