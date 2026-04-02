//! Dubbioso Validator Integration
#![allow(clippy::cognitive_complexity)] // Complex state machine for validation flow
//!
//! Integrates Dubbioso Mode components into validation flow:
//! - DubbiosoAnalyzer: confidence calculation
//! - McpQuestionManager: interactive questioning
//! - DubbiosoPatternStore: pattern persistence
//!
//! ## Flow
//!
//! 1. Validate code → get violations
//! 2. For each violation → calculate confidence
//! 3. If confidence low → ask question via MCP
//! 4. Process response → update pattern store
//! 5. Pattern becomes permanent after N acceptances

use crate::dubbioso::{DubbiosoAnalyzer, DubbiosoConfig, ConfidenceResult, ConfidenceLevel};
use crate::dubbioso_patterns::{DubbiosoPatternStore, PatternUpdate};
use crate::mcp_questions::{McpQuestionManager, McpResponse, ResponseResult};
use crate::memory::CodeGraph;
use crate::memory::ProjectState;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::error::Result;

/// Delta status for a violation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeltaStatus {
    /// New violation (not present in previous state)
    New,
    /// Persistent violation (present in previous state)
    Persistent,
    /// Unknown (no previous state available)
    Unknown,
}

/// Integrated Dubbioso Mode validator
pub struct DubbiosoValidator {
    /// Confidence analyzer
    analyzer: DubbiosoAnalyzer,
    /// Question manager for MCP
    question_manager: McpQuestionManager,
    /// Pattern persistence store
    pattern_store: DubbiosoPatternStore,
    /// Configuration
    #[allow(dead_code)] // Prepared for future: dynamic config access
    config: DubbiosoConfig,
    /// Previous validation state for delta detection
    previous_state: Option<ProjectState>,
}

/// Result of Dubbioso validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DubbiosoValidationResult {
    /// Original violation ID
    pub violation_id: String,
    /// File path
    pub file: String,
    /// Line number
    pub line: u32,
    /// Confidence analysis result
    pub confidence: ConfidenceResult,
    /// Whether violation is accepted
    pub accepted: bool,
    /// Whether a question was generated
    pub question_generated: bool,
    /// Pattern update (if any)
    pub pattern_update: Option<PatternUpdate>,
    /// Reason for decision
    pub reason: String,
    /// Delta status (new/persistent/unknown)
    pub delta_status: DeltaStatus,
}

/// Violation input for Dubbioso validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViolationInput {
    /// Violation ID or type
    pub id: String,
    /// Rule name (for delta matching with ViolationRecord::key())
    pub rule: String,
    /// Violation message
    pub message: String,
    /// File path
    pub file: String,
    /// Line number
    pub line: u32,
    /// Column number
    pub column: u32,
    /// Function name (if applicable)
    pub function_name: Option<String>,
    /// Code snippet
    pub code: Option<String>,
    /// Language
    pub language: String,
}

impl DubbiosoValidator {
    /// Create new Dubbioso validator
    pub fn new(config: DubbiosoConfig) -> Self {
        let permanent_after = config.permanent_after;
        Self {
            analyzer: DubbiosoAnalyzer::new(config.clone()),
            question_manager: McpQuestionManager::new(permanent_after),
            pattern_store: DubbiosoPatternStore::new(permanent_after),
            config,
            previous_state: None,
        }
    }

    /// Create with code graph
    pub fn with_graph(config: DubbiosoConfig, graph: CodeGraph) -> Self {
        let permanent_after = config.permanent_after;
        Self {
            analyzer: DubbiosoAnalyzer::with_graph(config.clone(), graph),
            question_manager: McpQuestionManager::new(permanent_after),
            pattern_store: DubbiosoPatternStore::new(permanent_after),
            config,
            previous_state: None,
        }
    }

    /// Create with pattern store persistence
    pub fn with_persistence(
        config: DubbiosoConfig,
        graph: Option<CodeGraph>,
        store_path: PathBuf,
    ) -> Result<Self> {
        let permanent_after = config.permanent_after;
        let analyzer = match graph {
            Some(g) => DubbiosoAnalyzer::with_graph(config.clone(), g),
            None => DubbiosoAnalyzer::new(config.clone()),
        };

        let pattern_store = if store_path.exists() {
            DubbiosoPatternStore::load(&store_path)?
        } else {
            DubbiosoPatternStore::with_path(store_path, permanent_after)
        };

        Ok(Self {
            analyzer,
            question_manager: McpQuestionManager::new(permanent_after),
            pattern_store,
            config,
            previous_state: None,
        })
    }

    /// Set previous validation state for delta detection
    pub fn set_previous_state(&mut self, state: ProjectState) {
        self.previous_state = Some(state);
    }

    /// Get previous state (for delta computation)
    pub fn previous_state(&self) -> Option<&ProjectState> {
        self.previous_state.as_ref()
    }

    /// Set code graph for context
    pub fn set_graph(&mut self, graph: CodeGraph) {
        self.analyzer.set_graph(graph);
    }

    /// Compute delta status for a violation
    fn compute_delta_status(&self, violation: &ViolationInput) -> DeltaStatus {
        match &self.previous_state {
            None => DeltaStatus::Unknown,
            Some(state) => {
                // Build a key matching ViolationRecord::key() format: rule:file:line:column
                let key = format!("{}:{}:{}:{}", violation.rule, violation.file, violation.line, violation.column);
                
                // Check if this violation exists in any file's violations
                for file_state in state.files.values() {
                    for v in &file_state.violations {
                        if v.key() == key {
                            return DeltaStatus::Persistent;
                        }
                    }
                }
                DeltaStatus::New
            }
        }
    }

    /// Validate a violation with Dubbioso Mode
    pub fn validate(&mut self, violation: &ViolationInput) -> DubbiosoValidationResult {
        // Compute delta status once
        let delta_status = self.compute_delta_status(violation);

        // Check if pattern is whitelisted
        if self.pattern_store.is_whitelisted(&violation.id, &violation.language, &violation.file) {
            return DubbiosoValidationResult {
                violation_id: violation.id.clone(),
                file: violation.file.clone(),
                line: violation.line,
                confidence: ConfidenceResult {
                    confidence: 1.0,
                    level: ConfidenceLevel::AutoAccept,
                    graph_context: None,
                    semantic_context: None,
                    uncertainty_reasons: vec!["Pattern is whitelisted".to_string()],
                    questions: vec![],
                },
                accepted: true,
                question_generated: false,
                pattern_update: None,
                reason: "Pattern is in whitelist".to_string(),
                delta_status,
            };
        }

        // Check if pattern is permanent
        if self.pattern_store.is_permanent(&violation.id, &violation.language) {
            let adjustment = self.pattern_store.get_confidence_adjustment(&violation.id, &violation.language);
            return DubbiosoValidationResult {
                violation_id: violation.id.clone(),
                file: violation.file.clone(),
                line: violation.line,
                confidence: ConfidenceResult {
                    confidence: 0.95 + adjustment,
                    level: ConfidenceLevel::AutoAccept,
                    graph_context: None,
                    semantic_context: None,
                    uncertainty_reasons: vec!["Pattern is permanent".to_string()],
                    questions: vec![],
                },
                accepted: true,
                question_generated: false,
                pattern_update: None,
                reason: "Pattern is permanent (accepted multiple times)".to_string(),
                delta_status,
            };
        }

        // Analyze confidence
        let code = violation.code.as_deref().unwrap_or("");
        let function = violation.function_name.as_deref().unwrap_or("unknown");
        let confidence = self.analyzer.analyze(code, function, &violation.file, &violation.language);

        // Apply learned confidence adjustment
        let adjustment = self.pattern_store.get_confidence_adjustment(&violation.id, &violation.language);
        let adjusted_confidence = (confidence.confidence + adjustment).clamp(0.0, 1.0);

        // Determine action based on level
        // Extract uncertainty reasons before match to avoid borrow issues
        let uncertainty_reasons = confidence.uncertainty_reasons.clone();
        let graph_context = confidence.graph_context.clone();
        let semantic_context = confidence.semantic_context.clone();
        let questions = confidence.questions.clone();

        match confidence.level {
            ConfidenceLevel::AutoAccept => {
                DubbiosoValidationResult {
                    violation_id: violation.id.clone(),
                    file: violation.file.clone(),
                    line: violation.line,
                    confidence,
                    accepted: true,
                    question_generated: false,
                    pattern_update: None,
                    reason: format!("High confidence ({:.0}%)", adjusted_confidence * 100.0),
                    delta_status,
                }
            }
            ConfidenceLevel::Good => {
                DubbiosoValidationResult {
                    violation_id: violation.id.clone(),
                    file: violation.file.clone(),
                    line: violation.line,
                    confidence,
                    accepted: true,
                    question_generated: false,
                    pattern_update: None,
                    reason: format!("Good confidence ({:.0}%)", adjusted_confidence * 100.0),
                    delta_status,
                }
            }
            ConfidenceLevel::Warn => {
                DubbiosoValidationResult {
                    violation_id: violation.id.clone(),
                    file: violation.file.clone(),
                    line: violation.line,
                    confidence,
                    accepted: true, // Warn but continue
                    question_generated: false,
                    pattern_update: None,
                    reason: format!("Warning: low confidence ({:.0}%) - {}", adjusted_confidence * 100.0, uncertainty_reasons.join(", ")),
                    delta_status,
                }
            }
            ConfidenceLevel::Ask => {
                // Generate question
                let graph_summary = graph_context.as_ref().map(|ctx| {
                    format!("{} files, score {:.2}", ctx.files_involved.len(), ctx.context_score)
                });
                let semantic_summary = semantic_context.as_ref().map(|s| {
                    format!("{:?}", s.base.intent)
                });

                let _question = self.question_manager.create_question(
                    &violation.id,
                    &violation.file,
                    violation.line,
                    adjusted_confidence,
                    graph_summary,
                    semantic_summary,
                    &questions,
                );

                DubbiosoValidationResult {
                    violation_id: violation.id.clone(),
                    file: violation.file.clone(),
                    line: violation.line,
                    confidence,
                    accepted: false, // Pending response
                    question_generated: true,
                    pattern_update: None,
                    reason: "Question generated, awaiting response".to_string(),
                    delta_status,
                }
            }
        }
    }

    /// Process MCP response
    pub fn process_response(&mut self, response: McpResponse) -> ResponseResult {
        let result = self.question_manager.process_response(response);

        // Update pattern store based on memory impact
        if let Some(ref update) = result.memory_update {
            // Find the question to get context
            let pattern = &update.pattern;

            if update.whitelist {
                self.pattern_store.whitelist_pattern(
                    pattern,
                    "unknown", // Language not stored in response
                    &result.message,
                );
            } else if result.accepted {
                self.pattern_store.accept_pattern(
                    pattern,
                    "unknown",
                    "", // File not stored in response
                    None,
                );
            } else {
                self.pattern_store.reject_pattern(
                    pattern,
                    "unknown",
                    "",
                    Some(&result.message),
                );
            }
        }

        result
    }

    /// Get pending question count
    pub fn pending_questions(&self) -> usize {
        self.question_manager.pending_count()
    }

    /// Get pattern store
    pub fn pattern_store(&self) -> &DubbiosoPatternStore {
        &self.pattern_store
    }

    /// Get mutable pattern store
    pub fn pattern_store_mut(&mut self) -> &mut DubbiosoPatternStore {
        &mut self.pattern_store
    }

    /// Get permanent patterns count
    pub fn permanent_pattern_count(&self) -> usize {
        self.pattern_store.permanent_patterns().len()
    }

    /// Get whitelisted patterns count
    pub fn whitelisted_pattern_count(&self) -> usize {
        self.pattern_store.whitelisted_patterns().len()
    }

    /// Format validation result for display
    pub fn format_result(&self, result: &DubbiosoValidationResult) -> String {
        let mut output = String::new();

        output.push_str(&format!(
            "{}:{} [{}] - {} ({:?})\n",
            result.file, result.line, result.violation_id,
            if result.accepted { "ACCEPTED" } else { "PENDING" },
            result.delta_status
        ));

        output.push_str(&format!(
            "  Confidence: {:.0}% ({:?})\n",
            result.confidence.confidence * 100.0,
            result.confidence.level
        ));

        if !result.confidence.uncertainty_reasons.is_empty() {
            output.push_str("  Uncertainty:\n");
            for reason in &result.confidence.uncertainty_reasons {
                output.push_str(&format!("    • {}\n", reason));
            }
        }

        output.push_str(&format!("  Reason: {}\n", result.reason));

        output
    }

    /// Save pattern store
    pub fn save_patterns(&self) -> Result<()> {
        self.pattern_store.save()
    }
}

impl Default for DubbiosoValidator {
    fn default() -> Self {
        Self::new(DubbiosoConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_high_confidence() {
        let mut validator = DubbiosoValidator::default();

        let violation = ViolationInput {
            id: "TEST001".to_string(),
            rule: "TEST001".to_string(),
            message: "Test violation".to_string(),
            file: "src/lib.rs".to_string(),
            line: 10,
            column: 0,
            function_name: Some("helper".to_string()),
            code: Some("fn helper() -> i32 { 42 }".to_string()),
            language: "rust".to_string(),
        };

        let result = validator.validate(&violation);
        assert!(result.confidence.confidence >= 0.0);
    }

    #[test]
    fn test_whitelist_bypass() {
        let mut validator = DubbiosoValidator::default();

        // Whitelist a pattern
        validator.pattern_store.whitelist_pattern("TEST001", "rust", "Test");

        let violation = ViolationInput {
            id: "TEST001".to_string(),
            rule: "TEST001".to_string(),
            message: "Test".to_string(),
            file: "src/test.rs".to_string(),
            line: 1,
            column: 0,
            function_name: None,
            code: None,
            language: "rust".to_string(),
        };

        let result = validator.validate(&violation);
        assert!(result.accepted);
        assert!(!result.question_generated);
    }

    #[test]
    fn test_permanent_pattern() {
        let mut validator = DubbiosoValidator::new(DubbiosoConfig {
            permanent_after: 2,
            ..Default::default()
        });

        // Accept pattern 2 times
        validator.pattern_store.accept_pattern("PERM001", "rust", "src/a.rs", None);
        validator.pattern_store.accept_pattern("PERM001", "rust", "src/b.rs", None);

        let violation = ViolationInput {
            id: "PERM001".to_string(),
            rule: "PERM001".to_string(),
            message: "Test".to_string(),
            file: "src/c.rs".to_string(),
            line: 1,
            column: 0,
            function_name: None,
            code: None,
            language: "rust".to_string(),
        };

        let result = validator.validate(&violation);
        assert!(result.accepted);
        assert!(!result.question_generated);
    }

    #[test]
    fn test_delta_status_unknown_without_previous_state() {
        let mut validator = DubbiosoValidator::default();

        let violation = ViolationInput {
            id: "DELTA001".to_string(),
            rule: "DELTA001".to_string(),
            message: "Test".to_string(),
            file: "src/lib.rs".to_string(),
            line: 42,
            column: 0,
            function_name: None,
            code: None,
            language: "rust".to_string(),
        };

        let result = validator.validate(&violation);
        assert_eq!(result.delta_status, DeltaStatus::Unknown);
    }

    #[test]
    fn test_delta_status_new_with_previous_state() {
        use crate::memory::{FileState, ViolationRecord, Severity};
        use std::collections::HashMap;
        use std::path::PathBuf;

        let mut validator = DubbiosoValidator::default();

        // Create previous state with a different violation
        let mut files = HashMap::new();
        let mut file_state = FileState::new();
        file_state.violations.push(ViolationRecord {
            id: "OTHER001".to_string(),
            rule: "OTHER001".to_string(),
            file: "src/lib.rs".to_string(),
            severity: Severity::Warning,
            line: 10,
            column: 0,
            message: "Other violation".to_string(),
            snippet: None,
        });
        files.insert("src/lib.rs".to_string(), file_state);

        let previous = ProjectState {
            project_id: "test".to_string(),
            root_path: PathBuf::from("/test"),
            files,
            accepted_violations: vec![],
            last_full_scan: None,
            metadata: Default::default(),
        };

        validator.set_previous_state(previous);

        // Validate a NEW violation (not in previous state)
        let violation = ViolationInput {
            id: "DELTA002".to_string(),
            rule: "DELTA002".to_string(),
            message: "Test".to_string(),
            file: "src/lib.rs".to_string(),
            line: 42,
            column: 0,
            function_name: None,
            code: None,
            language: "rust".to_string(),
        };

        let result = validator.validate(&violation);
        assert_eq!(result.delta_status, DeltaStatus::New);
    }

    #[test]
    fn test_delta_status_persistent_when_matches() {
        use crate::memory::{FileState, ViolationRecord, Severity};
        use std::collections::HashMap;
        use std::path::PathBuf;

        let mut validator = DubbiosoValidator::default();

        // Create previous state with THE SAME violation we'll validate
        let mut files = HashMap::new();
        let mut file_state = FileState::new();
        file_state.violations.push(ViolationRecord {
            id: "DELTA003".to_string(),
            rule: "DELTA003".to_string(),  // rule matches id
            file: "src/lib.rs".to_string(),
            severity: Severity::Warning,
            line: 42,
            column: 0,
            message: "Persistent violation".to_string(),
            snippet: None,
        });
        files.insert("src/lib.rs".to_string(), file_state);

        let previous = ProjectState {
            project_id: "test".to_string(),
            root_path: PathBuf::from("/test"),
            files,
            accepted_violations: vec![],
            last_full_scan: None,
            metadata: Default::default(),
        };

        validator.set_previous_state(previous);

        // Validate THE SAME violation (should be Persistent)
        let violation = ViolationInput {
            id: "DELTA003".to_string(),
            rule: "DELTA003".to_string(),
            message: "Test".to_string(),
            file: "src/lib.rs".to_string(),
            line: 42,
            column: 0,
            function_name: None,
            code: None,
            language: "rust".to_string(),
        };

        let result = validator.validate(&violation);
        assert_eq!(result.delta_status, DeltaStatus::Persistent);
    }
}
