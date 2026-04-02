//! Intent Classification
//!
//! Determines what action the user wants to perform.

use std::collections::HashMap;

/// User intent type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Intent {
    /// Create new code.
    Create,
    /// Change existing code.
    Modify,
    /// Fix a bug or error.
    Fix,
    /// Restructure without behavior change.
    Refactor,
    /// Remove code.
    Delete,
    /// Understand code.
    Explain,
    /// Find something.
    Search,
    /// Create or run tests.
    Test,
    /// Add documentation.
    Document,
    /// Unknown intent.
    Unknown,
}

impl std::fmt::Display for Intent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Create => write!(f, "CREATE"),
            Self::Modify => write!(f, "MODIFY"),
            Self::Fix => write!(f, "FIX"),
            Self::Refactor => write!(f, "REFACTOR"),
            Self::Delete => write!(f, "DELETE"),
            Self::Explain => write!(f, "EXPLAIN"),
            Self::Search => write!(f, "SEARCH"),
            Self::Test => write!(f, "TEST"),
            Self::Document => write!(f, "DOCUMENT"),
            Self::Unknown => write!(f, "UNKNOWN"),
        }
    }
}

/// Result of intent classification.
#[derive(Debug, Clone)]
pub struct IntentResult {
    /// Primary intent.
    pub primary: Intent,
    /// Confidence score (0.0-1.0).
    pub confidence: f32,
    /// Alternative intents with lower confidence.
    pub alternatives: Vec<(Intent, f32)>,
}

/// Classifies user intent from prompts.
pub struct IntentClassifier {
    /// Keywords for each intent.
    keywords: HashMap<Intent, Vec<String>>,
    /// Pattern phrases for each intent.
    patterns: HashMap<Intent, Vec<String>>,
}

impl IntentClassifier {
    /// Create a new classifier with default keywords.
    pub fn new() -> Self {
        Self {
            keywords: Self::default_keywords(),
            patterns: Self::default_patterns(),
        }
    }

    /// Classify the intent of a prompt.
    pub fn classify(&self, prompt: &str) -> IntentResult {
        let prompt_lower = prompt.to_lowercase();
        let mut scores: HashMap<Intent, f32> = HashMap::new();

        // Score based on keywords
        for (intent, keywords) in &self.keywords {
            let mut score = 0.0f32;
            for keyword in keywords {
                if prompt_lower.contains(keyword) {
                    score += 1.0;
                }
            }
            if score > 0.0 {
                *scores.entry(*intent).or_default() += score / keywords.len() as f32;
            }
        }

        // Score based on patterns (higher weight)
        for (intent, patterns) in &self.patterns {
            for pattern in patterns {
                if prompt_lower.contains(pattern) {
                    *scores.entry(*intent).or_default() += 0.5;
                }
            }
        }

        // Find best match
        let mut sorted: Vec<_> = scores.into_iter().collect();
        sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        if sorted.is_empty() {
            return IntentResult {
                primary: Intent::Unknown,
                confidence: 0.0,
                alternatives: Vec::new(),
            };
        }

        let primary = sorted[0].0;
        let confidence = sorted[0].1.min(1.0);
        let alternatives: Vec<(Intent, f32)> = sorted[1..]
            .iter()
            .filter(|(_, s)| *s > 0.1)
            .map(|(i, s)| (*i, s.min(1.0)))
            .collect();

        IntentResult {
            primary,
            confidence,
            alternatives,
        }
    }

    /// Default keyword mappings.
    fn default_keywords() -> HashMap<Intent, Vec<String>> {
        let mut map = HashMap::new();
        map.insert(Intent::Create, vec![
            "add".into(), "create".into(), "new".into(),
            "implement".into(), "build".into(), "write".into(),
        ]);
        map.insert(Intent::Modify, vec![
            "change".into(), "update".into(), "modify".into(),
            "alter".into(), "set".into(), "adjust".into(),
        ]);
        map.insert(Intent::Fix, vec![
            "fix".into(), "bug".into(), "error".into(),
            "crash".into(), "issue".into(), "broken".into(),
        ]);
        map.insert(Intent::Refactor, vec![
            "refactor".into(), "restructure".into(), "reorganize".into(),
            "extract".into(), "move".into(), "rename".into(),
        ]);
        map.insert(Intent::Delete, vec![
            "delete".into(), "remove".into(), "eliminate".into(),
        ]);
        map.insert(Intent::Explain, vec![
            "how".into(), "what".into(), "why".into(),
            "explain".into(), "describe".into(),
        ]);
        map.insert(Intent::Search, vec![
            "where".into(), "find".into(), "search".into(),
            "locate".into(), "show me".into(),
        ]);
        map.insert(Intent::Test, vec![
            "test".into(), "spec".into(), "coverage".into(),
            "assert".into(), "verify".into(),
        ]);
        map.insert(Intent::Document, vec![
            "document".into(), "comment".into(), "doc".into(),
            "readme".into(),
        ]);
        map
    }

    /// Default pattern mappings.
    fn default_patterns() -> HashMap<Intent, Vec<String>> {
        let mut map = HashMap::new();
        map.insert(Intent::Create, vec![
            "add a".into(), "create a".into(), "implement a".into(),
            "new function".into(), "new class".into(),
        ]);
        map.insert(Intent::Fix, vec![
            "fix the".into(), "fix a bug".into(), "fix crash".into(),
            "fix error".into(), "fix issue".into(),
        ]);
        map.insert(Intent::Refactor, vec![
            "refactor the".into(), "extract method".into(),
            "move to".into(), "rename the".into(),
        ]);
        map
    }
}

impl Default for IntentClassifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_create() {
        let classifier = IntentClassifier::new();
        let result = classifier.classify("Add a new enemy class");
        
        assert_eq!(result.primary, Intent::Create);
        assert!(result.confidence > 0.0);
    }

    #[test]
    fn test_classify_fix() {
        let classifier = IntentClassifier::new();
        let result = classifier.classify("Fix the crash in enemy update");
        
        assert_eq!(result.primary, Intent::Fix);
        assert!(result.confidence > 0.0);
    }

    #[test]
    fn test_classify_explain() {
        let classifier = IntentClassifier::new();
        let result = classifier.classify("How does the AI work?");
        
        assert_eq!(result.primary, Intent::Explain);
        assert!(result.confidence > 0.0);
    }

    #[test]
    fn test_classify_unknown() {
        let classifier = IntentClassifier::new();
        let result = classifier.classify("xyz abc def");
        
        assert_eq!(result.primary, Intent::Unknown);
    }

    #[test]
    fn test_intent_display() {
        assert_eq!(Intent::Create.to_string(), "CREATE");
        assert_eq!(Intent::Fix.to_string(), "FIX");
    }
}
