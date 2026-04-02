//! Layer 4: Intent Inference
//!
//! External LLM API for understanding code purpose.

mod inference;

pub use inference::IntentInferrer;

use serde::{Deserialize, Serialize};

/// Inferred intent of a code block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Intent {
    /// Brief description of what the code does
    pub summary: String,

    /// What the code is supposed to achieve
    pub purpose: String,

    /// Invariants that should be maintained
    pub invariants: Vec<String>,

    /// Side effects
    pub side_effects: Vec<String>,

    /// Dependencies (what the code relies on)
    pub dependencies: Vec<String>,

    /// Confidence level
    pub confidence: f32,
}

impl Default for Intent {
    fn default() -> Self {
        Self {
            summary: String::new(),
            purpose: String::new(),
            invariants: Vec::new(),
            side_effects: Vec::new(),
            dependencies: Vec::new(),
            confidence: 0.0,
        }
    }
}
