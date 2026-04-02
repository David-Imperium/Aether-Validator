//! Exemption Store - Manages learned exemptions for violations
//!
//! Stores and retrieves exemptions based on:
//! - Project patterns
//! - User decisions
//! - Context-specific rules

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use chrono::{DateTime, Utc};
use crate::error::Result;

/// Scope for an exemption
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExemptionScope {
    /// Applies to a specific file
    File { path: String },
    /// Applies to a directory
    Directory { path: String },
    /// Applies to a pattern (e.g., all test files)
    Pattern { pattern: String },
    /// Applies project-wide
    Project,
}

/// An exemption for a specific violation type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Exemption {
    /// Unique ID for this exemption
    pub id: String,
    /// Rule ID this exempts
    pub rule_id: String,
    /// Scope of the exemption
    pub scope: ExemptionScope,
    /// Reason for the exemption
    pub reason: String,
    /// How many times this exemption has been applied
    pub application_count: u32,
    /// When this exemption was created
    pub created_at: DateTime<Utc>,
    /// When this exemption was last applied
    pub last_applied: Option<DateTime<Utc>>,
    /// Confidence in this exemption (from learning)
    pub confidence: f64,
    /// Source of this exemption
    pub source: ExemptionSource,
}

/// How an exemption was created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExemptionSource {
    /// User explicitly created
    UserCreated,
    /// Learned from repeated patterns
    LearnedPattern,
    /// Project configuration
    ProjectConfig,
    /// Precedent-based
    Precedent { original_decision_id: String },
}

/// Store for exemptions
pub struct ExemptionStore {
    /// Exemptions by rule ID
    by_rule: HashMap<String, Vec<Exemption>>,
    /// Storage path
    path: Option<PathBuf>,
}

impl Default for ExemptionStore {
    fn default() -> Self {
        Self::new()
    }
}

impl ExemptionStore {
    /// Create a new exemption store
    pub fn new() -> Self {
        Self {
            by_rule: HashMap::new(),
            path: None,
        }
    }
    
    /// Create with storage path
    pub fn with_path(path: PathBuf) -> Self {
        let mut store = Self::new();
        store.path = Some(path.clone());
        store.load().ok();
        store
    }
    
    /// Add an exemption
    pub fn add(&mut self, exemption: Exemption) {
        self.by_rule
            .entry(exemption.rule_id.clone())
            .or_default()
            .push(exemption);
        self.save().ok();
    }
    
    /// Find matching exemption for a rule and context
    pub fn find(
        &self,
        rule_id: &str,
        file_path: &str,
    ) -> Option<&Exemption> {
        self.by_rule
            .get(rule_id)?
            .iter()
            .find(|e| self.matches_scope(&e.scope, file_path))
    }
    
    /// Check if a scope matches a file path
    fn matches_scope(&self, scope: &ExemptionScope, file_path: &str) -> bool {
        match scope {
            ExemptionScope::File { path } => file_path == path,
            ExemptionScope::Directory { path } => file_path.starts_with(path),
            ExemptionScope::Pattern { pattern } => {
                // Simple glob-like matching
                if pattern.contains('*') {
                    let parts: Vec<&str> = pattern.split('*').collect();
                    if parts.len() == 2 {
                        file_path.starts_with(parts[0]) && file_path.ends_with(parts[1])
                    } else {
                        file_path.contains(&pattern.replace('*', ""))
                    }
                } else {
                    file_path.contains(pattern)
                }
            }
            ExemptionScope::Project => true,
        }
    }
    
    /// Record an application of an exemption
    pub fn record_application(&mut self, exemption_id: &str) {
        for exemptions in self.by_rule.values_mut() {
            if let Some(exemption) = exemptions.iter_mut().find(|e| e.id == exemption_id) {
                exemption.application_count += 1;
                exemption.last_applied = Some(Utc::now());
                self.save().ok();
                return;
            }
        }
    }
    
    /// Get all exemptions for a rule
    pub fn get_for_rule(&self, rule_id: &str) -> Vec<&Exemption> {
        self.by_rule.get(rule_id).map(|v| v.iter().collect()).unwrap_or_default()
    }
    
    /// Get statistics
    pub fn stats(&self) -> ExemptionStats {
        let total: usize = self.by_rule.values().map(|v| v.len()).sum();
        let learned = self.by_rule.values()
            .flat_map(|v| v.iter())
            .filter(|e| matches!(e.source, ExemptionSource::LearnedPattern))
            .count();
        let user_created = self.by_rule.values()
            .flat_map(|v| v.iter())
            .filter(|e| matches!(e.source, ExemptionSource::UserCreated))
            .count();
        
        ExemptionStats {
            total,
            learned,
            user_created,
            by_rule_count: self.by_rule.len(),
        }
    }
    
    /// Load from disk
    fn load(&mut self) -> Result<()> {
        if let Some(ref path) = self.path {
            if path.exists() {
                let data = std::fs::read_to_string(path)?;
                let loaded: HashMap<String, Vec<Exemption>> = serde_json::from_str(&data)?;
                self.by_rule = loaded;
            }
        }
        Ok(())
    }
    
    /// Save to disk
    fn save(&self) -> Result<()> {
        if let Some(ref path) = self.path {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let data = serde_json::to_string_pretty(&self.by_rule)?;
            std::fs::write(path, data)?;
        }
        Ok(())
    }
    
    /// Clear all exemptions
    pub fn clear(&mut self) {
        self.by_rule.clear();
        self.save().ok();
    }
}

/// Statistics about exemptions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExemptionStats {
    pub total: usize,
    pub learned: usize,
    pub user_created: usize,
    pub by_rule_count: usize,
}

impl Exemption {
    /// Create a new exemption
    pub fn new(
        rule_id: String,
        scope: ExemptionScope,
        reason: String,
        source: ExemptionSource,
    ) -> Self {
        Self {
            id: format!("EXM-{}", uuid::Uuid::new_v4().to_string()[..8].to_string()),
            rule_id,
            scope,
            reason,
            application_count: 0,
            created_at: Utc::now(),
            last_applied: None,
            confidence: 0.5,
            source,
        }
    }
    
    /// Create a learned exemption
    pub fn learned(rule_id: String, scope: ExemptionScope, confidence: f64) -> Self {
        Self {
            id: format!("EXM-{}", uuid::Uuid::new_v4().to_string()[..8].to_string()),
            rule_id,
            scope,
            reason: "Learned from project patterns".into(),
            application_count: 0,
            created_at: Utc::now(),
            last_applied: None,
            confidence,
            source: ExemptionSource::LearnedPattern,
        }
    }
}
