//! Composite Patterns — AND, OR, NOT pattern composition
//!
//! This module provides pattern composition for complex rule matching:
//! - AndPattern: All sub-patterns must match
//! - OrPattern: Any sub-pattern must match
//! - NotPattern: Pattern must NOT match

use regex::Regex;
use std::collections::HashMap;
use crate::error::{ContractError, ContractResult};

/// Pattern match result with context
#[derive(Debug, Clone)]
pub struct PatternMatch {
    /// The matched text
    pub matched: String,
    /// Start position in source
    pub start: usize,
    /// End position in source
    pub end: usize,
}

/// Trait for pattern matching
pub trait Pattern: Send + Sync {
    /// Check if pattern matches in source
    fn matches(&self, source: &str) -> ContractResult<Vec<PatternMatch>>;
    
    /// Check if pattern matches at least once
    fn matches_any(&self, source: &str) -> ContractResult<bool> {
        Ok(!self.matches(source)?.is_empty())
    }
}

/// Simple text pattern
pub struct TextPattern {
    pattern: String,
}

impl TextPattern {
    pub fn new(pattern: impl Into<String>) -> Self {
        Self { pattern: pattern.into() }
    }
}

impl Pattern for TextPattern {
    fn matches(&self, source: &str) -> ContractResult<Vec<PatternMatch>> {
        let mut results = Vec::new();
        let mut start = 0;
        
        while let Some(pos) = source[start..].find(&self.pattern) {
            let abs_start = start + pos;
            let abs_end = abs_start + self.pattern.len();
            
            results.push(PatternMatch {
                matched: self.pattern.clone(),
                start: abs_start,
                end: abs_end,
            });
            
            start = abs_end;
        }
        
        Ok(results)
    }
}

/// Regex pattern
pub struct RegexPattern {
    #[allow(dead_code)]
    pattern: String,
    regex: Regex,
}

impl RegexPattern {
    pub fn new(pattern: &str) -> ContractResult<Self> {
        let regex = Regex::new(pattern)
            .map_err(|e| ContractError::ParseError(
                format!("Invalid regex: {}", pattern),
                e.to_string()
            ))?;
        
        Ok(Self {
            pattern: pattern.to_string(),
            regex,
        })
    }
}

impl Pattern for RegexPattern {
    fn matches(&self, source: &str) -> ContractResult<Vec<PatternMatch>> {
        Ok(self.regex
            .find_iter(source)
            .map(|m| PatternMatch {
                matched: m.as_str().to_string(),
                start: m.start(),
                end: m.end(),
            })
            .collect())
    }
}

/// AND pattern — all sub-patterns must match
pub struct AndPattern {
    patterns: Vec<Box<dyn Pattern>>,
}

impl AndPattern {
    pub fn new(patterns: Vec<Box<dyn Pattern>>) -> Self {
        Self { patterns }
    }
    
    pub fn from_text(patterns: Vec<String>) -> ContractResult<Self> {
        let patterns: Vec<Box<dyn Pattern>> = patterns
            .into_iter()
            .map(|p| Box::new(TextPattern::new(p)) as Box<dyn Pattern>)
            .collect();
        
        Ok(Self { patterns })
    }
}

impl Pattern for AndPattern {
    fn matches(&self, source: &str) -> ContractResult<Vec<PatternMatch>> {
        // For AND, we return matches only if ALL patterns match
        // We return the intersection of all matches
        
        if self.patterns.is_empty() {
            return Ok(Vec::new());
        }
        
        // Get matches from first pattern
        let result = self.patterns[0].matches(source)?;
        
        // Check that all other patterns match
        for pattern in &self.patterns[1..] {
            if !pattern.matches_any(source)? {
                // If any pattern doesn't match, AND fails
                return Ok(Vec::new());
            }
        }
        
        Ok(result)
    }
}

/// OR pattern — any sub-pattern must match
pub struct OrPattern {
    patterns: Vec<Box<dyn Pattern>>,
}

impl OrPattern {
    pub fn new(patterns: Vec<Box<dyn Pattern>>) -> Self {
        Self { patterns }
    }
    
    pub fn from_text(patterns: Vec<String>) -> ContractResult<Self> {
        let patterns: Vec<Box<dyn Pattern>> = patterns
            .into_iter()
            .map(|p| Box::new(TextPattern::new(p)) as Box<dyn Pattern>)
            .collect();
        
        Ok(Self { patterns })
    }
}

impl Pattern for OrPattern {
    fn matches(&self, source: &str) -> ContractResult<Vec<PatternMatch>> {
        let mut all_matches = Vec::new();
        
        for pattern in &self.patterns {
            all_matches.extend(pattern.matches(source)?);
        }
        
        // Sort by position
        all_matches.sort_by_key(|m| m.start);
        
        Ok(all_matches)
    }
}

/// NOT pattern — pattern must NOT match
pub struct NotPattern {
    pattern: Box<dyn Pattern>,
}

impl NotPattern {
    pub fn new(pattern: Box<dyn Pattern>) -> Self {
        Self { pattern }
    }
    
    pub fn from_text(pattern: String) -> Self {
        Self {
            pattern: Box::new(TextPattern::new(pattern)),
        }
    }
}

impl Pattern for NotPattern {
    fn matches(&self, source: &str) -> ContractResult<Vec<PatternMatch>> {
        // NOT pattern matches the entire source if the inner pattern doesn't match
        if self.pattern.matches_any(source)? {
            Ok(Vec::new())
        } else {
            Ok(vec![PatternMatch {
                matched: source.to_string(),
                start: 0,
                end: source.len(),
            }])
        }
    }
}

/// Pattern factory for creating patterns from definitions
pub struct PatternFactory {
    regex_cache: HashMap<String, Regex>,
}

impl PatternFactory {
    pub fn new() -> Self {
        Self {
            regex_cache: HashMap::new(),
        }
    }
    
    /// Create a pattern from a definition string
    /// 
    /// Supported formats:
    /// - `text:pattern` — Simple text match
    /// - `regex:pattern` — Regex match
    /// - `and:[p1, p2, ...]` — All patterns must match
    /// - `or:[p1, p2, ...]` — Any pattern must match
    /// - `not:pattern` — Pattern must NOT match
    pub fn create(&mut self, definition: &str) -> ContractResult<Box<dyn Pattern>> {
        // Handle composite patterns
        if let Some(inner) = definition.strip_prefix("and:") {
            let patterns = self.parse_pattern_list(inner)?;
            Ok(Box::new(AndPattern::new(patterns)))
        } else if let Some(inner) = definition.strip_prefix("or:") {
            let patterns = self.parse_pattern_list(inner)?;
            Ok(Box::new(OrPattern::new(patterns)))
        } else if let Some(inner) = definition.strip_prefix("not:") {
            let pattern = self.create(inner)?;
            Ok(Box::new(NotPattern::new(pattern)))
        } else if let Some(regex_pattern) = definition.strip_prefix("regex:") {
            let pattern = self.create_regex_pattern(regex_pattern)?;
            Ok(Box::new(pattern))
        } else if let Some(text_pattern) = definition.strip_prefix("text:") {
            Ok(Box::new(TextPattern::new(text_pattern)))
        } else {
            // Default to text pattern
            Ok(Box::new(TextPattern::new(definition)))
        }
    }
    
    fn parse_pattern_list(&mut self, inner: &str) -> ContractResult<Vec<Box<dyn Pattern>>> {
        // Parse format: [pattern1, pattern2, ...]
        let inner = inner.trim();
        
        if !inner.starts_with('[') || !inner.ends_with(']') {
            return Err(ContractError::ParseError(
                "Invalid pattern list format".to_string(),
                "Expected [pattern1, pattern2, ...]".to_string()
            ));
        }
        
        let inner = &inner[1..inner.len()-1];
        let patterns: Vec<Box<dyn Pattern>> = inner
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| self.create(s))
            .collect::<ContractResult<Vec<_>>>()?;
        
        Ok(patterns)
    }
    
    fn create_regex_pattern(&mut self, pattern: &str) -> ContractResult<RegexPattern> {
        if let Some(cached) = self.regex_cache.get(pattern) {
            Ok(RegexPattern {
                pattern: pattern.to_string(),
                regex: cached.clone(),
            })
        } else {
            let regex = Regex::new(pattern)
                .map_err(|e| ContractError::ParseError(
                    format!("Invalid regex: {}", pattern),
                    e.to_string()
                ))?;
            
            self.regex_cache.insert(pattern.to_string(), regex.clone());
            Ok(RegexPattern {
                pattern: pattern.to_string(),
                regex,
            })
        }
    }
}

impl Default for PatternFactory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_pattern() {
        let pattern = TextPattern::new("unwrap()");
        let matches = pattern.matches("let x = opt.unwrap();").unwrap();
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].matched, "unwrap()");
    }
    
    #[test]
    fn test_text_pattern_multiple() {
        let pattern = TextPattern::new("unwrap()");
        let matches = pattern.matches("let a = x.unwrap(); let b = y.unwrap();").unwrap();
        assert_eq!(matches.len(), 2);
    }
    
    #[test]
    fn test_regex_pattern() {
        let pattern = RegexPattern::new(r"\bpanic!\(").unwrap();
        let matches = pattern.matches("fn main() { panic!(\"error\"); }").unwrap();
        assert_eq!(matches.len(), 1);
    }
    
    #[test]
    fn test_and_pattern() {
        let patterns: Vec<Box<dyn Pattern>> = vec![
            Box::new(TextPattern::new("unwrap()")),
            Box::new(TextPattern::new("Result")),
        ];
        let and_pattern = AndPattern::new(patterns);
        
        // Both patterns match
        let matches = and_pattern.matches("let x: Result<T, E> = opt.unwrap();").unwrap();
        assert!(!matches.is_empty());
        
        // Only one pattern matches
        let matches = and_pattern.matches("let x = opt.unwrap();").unwrap();
        assert!(matches.is_empty());
    }
    
    #[test]
    fn test_or_pattern() {
        let patterns: Vec<Box<dyn Pattern>> = vec![
            Box::new(TextPattern::new("unwrap()")),
            Box::new(TextPattern::new("expect(")),
        ];
        let or_pattern = OrPattern::new(patterns);
        
        let matches = or_pattern.matches("let x = opt.unwrap();").unwrap();
        assert_eq!(matches.len(), 1);
        
        let matches = or_pattern.matches("let x = opt.expect(\"msg\");").unwrap();
        assert_eq!(matches.len(), 1);
        
        let matches = or_pattern.matches("let x = opt?;").unwrap();
        assert!(matches.is_empty());
    }
    
    #[test]
    fn test_not_pattern() {
        let pattern = NotPattern::new(Box::new(TextPattern::new("unsafe")));
        
        let matches = pattern.matches("fn safe_function() {}").unwrap();
        assert_eq!(matches.len(), 1);
        
        let matches = pattern.matches("unsafe { }").unwrap();
        assert!(matches.is_empty());
    }
    
    #[test]
    fn test_pattern_factory_text() {
        let mut factory = PatternFactory::new();
        let pattern = factory.create("unwrap()").unwrap();
        let matches = pattern.matches("x.unwrap()").unwrap();
        assert_eq!(matches.len(), 1);
    }
    
    #[test]
    fn test_pattern_factory_regex() {
        let mut factory = PatternFactory::new();
        let pattern = factory.create("regex:\\bpanic!").unwrap();
        let matches = pattern.matches("panic!(\"error\")").unwrap();
        assert_eq!(matches.len(), 1);
    }
    
    #[test]
    fn test_pattern_factory_and() {
        let mut factory = PatternFactory::new();
        let pattern = factory.create("and:[unwrap, Result]").unwrap();
        let matches = pattern.matches("let x: Result<T> = opt.unwrap();").unwrap();
        assert!(!matches.is_empty());
    }
    
    #[test]
    fn test_pattern_factory_or() {
        let mut factory = PatternFactory::new();
        let pattern = factory.create("or:[unwrap, expect]").unwrap();
        let matches = pattern.matches("x.unwrap()").unwrap();
        assert_eq!(matches.len(), 1);
    }
    
    #[test]
    fn test_pattern_factory_not() {
        let mut factory = PatternFactory::new();
        let pattern = factory.create("not:unsafe").unwrap();
        let matches = pattern.matches("fn safe() {}").unwrap();
        assert_eq!(matches.len(), 1);
    }
}
