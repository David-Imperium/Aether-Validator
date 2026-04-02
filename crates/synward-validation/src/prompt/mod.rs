//! Prompt Analysis Module
//!
//! Analyzes user prompts to extract:
//! - Intent (CREATE, MODIFY, FIX, etc.)
//! - Scope (FILE, FUNCTION, CLASS, MODULE)
//! - Domain (gameplay, ui, graphics, etc.)
//! - Ambiguities (underspecified parts)
//! - Context (relevant code)

mod intent;
mod scope;
mod domain;
mod ambiguity;
mod analyzer;

pub use intent::{Intent, IntentResult, IntentClassifier};
pub use scope::{ScopeLevel, ScopeEntity, ScopeResult, ScopeExtractor};
pub use domain::{DomainResult, DomainMapper};
pub use ambiguity::{Ambiguity, AmbiguityType, AmbiguityDetector, ClarificationRequest};
pub use analyzer::{PromptAnalysis, PromptAnalyzer};
