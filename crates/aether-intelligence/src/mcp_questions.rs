//! MCP Question Protocol for Dubbioso Mode
//!
//! Enables interactive questioning when confidence is low.
//! Questions are sent via MCP (Model Context Protocol) to the client.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// MCP question message sent to client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpQuestion {
    /// Question ID for tracking
    pub id: String,
    /// Type of question
    pub question_type: QuestionType,
    /// The actual question text
    pub message: String,
    /// Available response options
    pub options: Vec<QuestionOption>,
    /// Context for the question (violation details, etc.)
    pub context: QuestionContext,
    /// Timeout in seconds (0 = no timeout)
    pub timeout_secs: u32,
}

/// Type of question being asked
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum QuestionType {
    /// Ask for confirmation (y/n)
    Confirm,
    /// Ask to choose from options
    Choice,
    /// Ask for pattern learning
    PatternLearn,
    /// Ask for context clarification
    Clarification,
}

/// Available response option
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionOption {
    /// Option key (single char for quick response)
    pub key: String,
    /// Display label
    pub label: String,
    /// Description of what this option does
    pub description: String,
    /// Impact on memory if selected
    pub memory_impact: MemoryImpact,
}

/// Impact on memory when option is selected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryImpact {
    /// No memory change
    None,
    /// Boost confidence for this case
    BoostCase(f64),
    /// Boost confidence for this pattern
    BoostPattern(f64),
    /// Add to whitelist
    Whitelist,
    /// Record as violation
    RecordViolation,
    /// Skip, don't remember
    Skip,
}

/// Context for the question
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionContext {
    /// File path
    pub file: String,
    /// Line number
    pub line: u32,
    /// Violation type
    pub violation: String,
    /// Confidence score
    pub confidence: f64,
    /// Graph context summary
    pub graph_summary: Option<String>,
    /// Semantic context summary
    pub semantic_summary: Option<String>,
}

/// Response from client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResponse {
    /// Question ID being answered
    pub question_id: String,
    /// Selected option key
    pub selected: String,
    /// Optional explanation from user
    pub explanation: Option<String>,
    /// Timestamp
    pub timestamp: u64,
}

/// Result of processing a response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseResult {
    /// Whether the violation is accepted
    pub accepted: bool,
    /// Memory update to apply
    pub memory_update: Option<MemoryUpdate>,
    /// Message to display
    pub message: String,
}

/// Memory update from response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryUpdate {
    /// Pattern to remember
    pub pattern: String,
    /// Confidence adjustment
    pub confidence_adjustment: f64,
    /// Whether to whitelist
    pub whitelist: bool,
    /// Whether to make permanent
    pub permanent: bool,
}

/// MCP Question Manager
pub struct McpQuestionManager {
    /// Pending questions waiting for response
    pending: HashMap<String, McpQuestion>,
    /// Response history
    history: Vec<(McpQuestion, McpResponse)>,
    /// Pattern acceptance counts
    pattern_counts: HashMap<String, u32>,
    /// Permanent after N acceptances
    permanent_after: u32,
}

impl McpQuestionManager {
    /// Create new question manager
    pub fn new(permanent_after: u32) -> Self {
        Self {
            pending: HashMap::new(),
            history: Vec::new(),
            pattern_counts: HashMap::new(),
            permanent_after,
        }
    }

    /// Create a question for a violation
    #[allow(clippy::too_many_arguments)]
    pub fn create_question(
        &mut self,
        violation: &str,
        file: &str,
        line: u32,
        confidence: f64,
        graph_summary: Option<String>,
        semantic_summary: Option<String>,
        questions: &[String],
    ) -> McpQuestion {
        let id = format!("q_{}_{}", file.replace('/', "_"), line);

        let question_type = if questions.len() == 1 {
            QuestionType::Confirm
        } else {
            QuestionType::Choice
        };

        let message = if questions.is_empty() {
            "Is this violation acceptable?".to_string()
        } else {
            questions.join("\n")
        };

        let options = vec![
            QuestionOption {
                key: "y".to_string(),
                label: "Yes".to_string(),
                description: "Accept this time".to_string(),
                memory_impact: MemoryImpact::BoostCase(0.05),
            },
            QuestionOption {
                key: "n".to_string(),
                label: "No".to_string(),
                description: "Reject, it's an error".to_string(),
                memory_impact: MemoryImpact::RecordViolation,
            },
            QuestionOption {
                key: "a".to_string(),
                label: "Always".to_string(),
                description: "Always accept this pattern".to_string(),
                memory_impact: MemoryImpact::Whitelist,
            },
            QuestionOption {
                key: "s".to_string(),
                label: "Skip".to_string(),
                description: "Skip, don't remember".to_string(),
                memory_impact: MemoryImpact::Skip,
            },
            QuestionOption {
                key: "e".to_string(),
                label: "Explain".to_string(),
                description: "Show full context".to_string(),
                memory_impact: MemoryImpact::None,
            },
        ];

        let question = McpQuestion {
            id: id.clone(),
            question_type,
            message,
            options,
            context: QuestionContext {
                file: file.to_string(),
                line,
                violation: violation.to_string(),
                confidence,
                graph_summary,
                semantic_summary,
            },
            timeout_secs: 300, // 5 minutes default
        };

        self.pending.insert(id, question.clone());
        question
    }

    /// Process a response
    pub fn process_response(&mut self, response: McpResponse) -> ResponseResult {
        let question = match self.pending.remove(&response.question_id) {
            Some(q) => q,
            None => {
                return ResponseResult {
                    accepted: false,
                    memory_update: None,
                    message: "Unknown question ID".to_string(),
                };
            }
        };

        // Find selected option
        let selected_option = question
            .options
            .iter()
            .find(|o| o.key == response.selected)
            .cloned();

        let Some(option) = selected_option else {
            return ResponseResult {
                accepted: false,
                memory_update: None,
                message: "Invalid option selected".to_string(),
            };
        };

        // Record in history
        self.history.push((question.clone(), response.clone()));

        // Process based on memory impact
        let (accepted, memory_update, message) = match option.memory_impact {
            MemoryImpact::None => {
                // Explain mode - don't accept/reject, just return context
                let msg = format_context_explanation(&question);
                (false, None, msg)
            }
            MemoryImpact::Skip => {
                (false, None, "Skipped, no memory change".to_string())
            }
            MemoryImpact::BoostCase(amount) => {
                let update = MemoryUpdate {
                    pattern: question.context.violation.clone(),
                    confidence_adjustment: amount,
                    whitelist: false,
                    permanent: false,
                };
                (true, Some(update), "Accepted this case, confidence boosted".to_string())
            }
            MemoryImpact::BoostPattern(amount) => {
                *self.pattern_counts.entry(question.context.violation.clone()).or_insert(0) += 1;
                let count = self.pattern_counts[&question.context.violation];
                let permanent = count >= self.permanent_after;

                let update = MemoryUpdate {
                    pattern: question.context.violation.clone(),
                    confidence_adjustment: amount,
                    whitelist: false,
                    permanent,
                };
                let msg = if permanent {
                    "Pattern now permanent after multiple acceptances".to_string()
                } else {
                    format!("Pattern accepted {} times", count)
                };
                (true, Some(update), msg)
            }
            MemoryImpact::Whitelist => {
                *self.pattern_counts.entry(question.context.violation.clone()).or_insert(0) += 1;
                let count = self.pattern_counts[&question.context.violation];
                let permanent = count >= self.permanent_after;

                let update = MemoryUpdate {
                    pattern: question.context.violation.clone(),
                    confidence_adjustment: 0.2,
                    whitelist: true,
                    permanent,
                };
                let msg = if permanent {
                    "Pattern now permanent after multiple acceptances".to_string()
                } else {
                    "Added to whitelist".to_string()
                };
                (true, Some(update), msg)
            }
            MemoryImpact::RecordViolation => {
                let update = MemoryUpdate {
                    pattern: question.context.violation.clone(),
                    confidence_adjustment: -0.05,
                    whitelist: false,
                    permanent: false,
                };
                (false, Some(update), "Recorded as violation".to_string())
            }
        };

        ResponseResult {
            accepted,
            memory_update,
            message,
        }
    }

    /// Get pending question count
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    /// Get pattern acceptance count
    pub fn pattern_count(&self, pattern: &str) -> u32 {
        self.pattern_counts.get(pattern).copied().unwrap_or(0)
    }

    /// Check if pattern should be permanent
    pub fn should_make_permanent(&self, pattern: &str) -> bool {
        self.pattern_counts.get(pattern).copied().unwrap_or(0) >= self.permanent_after
    }
}

impl Default for McpQuestionManager {
    fn default() -> Self {
        Self::new(5) // Default: permanent after 5 acceptances
    }
}

/// Format context explanation for 'e' option
fn format_context_explanation(question: &McpQuestion) -> String {
    let mut output = String::new();

    output.push_str(&format!("{}\n\n", "=".repeat(60)));
    output.push_str(&format!("VIOLATION: {}\n", question.context.violation));
    output.push_str(&format!("File: {}:{}\n", question.context.file, question.context.line));
    output.push_str(&format!("Confidence: {:.0}%\n\n", question.context.confidence * 100.0));

    if let Some(ref graph) = question.context.graph_summary {
        output.push_str(&format!("Graph Context:\n{}\n\n", graph));
    }

    if let Some(ref semantic) = question.context.semantic_summary {
        output.push_str(&format!("Semantic Context:\n{}\n\n", semantic));
    }

    output.push_str(&format!("{}\n", "=".repeat(60)));
    output
}

/// Format MCP question as JSON for transmission
pub fn format_question_json(question: &McpQuestion) -> String {
    serde_json::to_string_pretty(question).unwrap_or_default()
}

/// Parse MCP response from JSON
pub fn parse_response_json(json: &str) -> Option<McpResponse> {
    serde_json::from_str(json).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_question() {
        let mut manager = McpQuestionManager::default();

        let question = manager.create_question(
            "unwrap() in production",
            "src/main.rs",
            42,
            0.65,
            Some("Called by run()".to_string()),
            Some("No error handling".to_string()),
            &["Is this acceptable?".to_string()],
        );

        assert_eq!(question.question_type, QuestionType::Confirm);
        assert_eq!(question.options.len(), 5);
        assert!(manager.pending.contains_key(&question.id));
    }

    #[test]
    fn test_process_response_whitelist() {
        let mut manager = McpQuestionManager::default();

        let question = manager.create_question(
            "unwrap() in test",
            "test/main.rs",
            10,
            0.70,
            None,
            None,
            &[],
        );

        let response = McpResponse {
            question_id: question.id.clone(),
            selected: "a".to_string(),
            explanation: None,
            timestamp: 0,
        };

        let result = manager.process_response(response);
        assert!(result.accepted);
        assert!(result.memory_update.unwrap().whitelist);
    }

    #[test]
    fn test_permanent_after_n() {
        let mut manager = McpQuestionManager::new(3);

        // Accept same pattern 3 times
        for i in 0..3 {
            let question = manager.create_question(
                "unwrap() in test",
                "test/main.rs",
                10 + i,
                0.70,
                None,
                None,
                &[],
            );

            let response = McpResponse {
                question_id: question.id.clone(),
                selected: "a".to_string(),
                explanation: None,
                timestamp: i as u64,
            };

            let result = manager.process_response(response);
            if i == 2 {
                assert!(result.memory_update.unwrap().permanent);
            }
        }

        assert!(manager.should_make_permanent("unwrap() in test"));
    }
}
