//! Domain Mapping
//!
//! Classifies the technical domain of a request.

use std::collections::HashMap;

/// Result of domain mapping.
#[derive(Debug, Clone)]
pub struct DomainResult {
    /// Primary domain.
    pub primary: String,
    /// Secondary domains.
    pub secondary: Vec<String>,
    /// Domain tags.
    pub tags: Vec<String>,
    /// Confidence score.
    pub confidence: f32,
}

impl DomainResult {
    /// Create a new domain result.
    pub fn new(primary: impl Into<String>) -> Self {
        Self {
            primary: primary.into(),
            secondary: Vec::new(),
            tags: Vec::new(),
            confidence: 1.0,
        }
    }

    /// Add a secondary domain.
    pub fn with_secondary(mut self, domain: impl Into<String>) -> Self {
        self.secondary.push(domain.into());
        self
    }

    /// Add a tag.
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }
}

/// Maps prompts to technical domains.
pub struct DomainMapper {
    /// Domain keywords mapping.
    domain_keywords: HashMap<String, Vec<String>>,
    /// Tag keywords mapping.
    tag_keywords: HashMap<String, Vec<String>>,
}

impl DomainMapper {
    /// Create a new mapper with default keywords.
    pub fn new() -> Self {
        Self {
            domain_keywords: Self::default_domains(),
            tag_keywords: Self::default_tags(),
        }
    }

    /// Map a prompt to domains.
    pub fn map(&self, prompt: &str) -> DomainResult {
        let prompt_lower = prompt.to_lowercase();
        let mut scores: HashMap<String, f32> = HashMap::new();
        let mut all_tags: Vec<String> = Vec::new();

        // Score domains by keywords
        for (domain, keywords) in &self.domain_keywords {
            let mut score = 0.0f32;
            for keyword in keywords {
                if prompt_lower.contains(keyword) {
                    score += 1.0;
                }
            }
            if score > 0.0 {
                *scores.entry(domain.clone()).or_default() += score / keywords.len() as f32;
            }
        }

        // Extract tags
        for (tag, keywords) in &self.tag_keywords {
            for keyword in keywords {
                if prompt_lower.contains(keyword) {
                    all_tags.push(tag.clone());
                    break;
                }
            }
        }

        // Sort domains by score
        let mut sorted: Vec<_> = scores.into_iter().collect();
        sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        if sorted.is_empty() {
            return DomainResult::new("general")
                .with_tag("general");
        }

        let primary = sorted[0].0.clone();
        let confidence = sorted[0].1.min(1.0);
        let secondary: Vec<String> = sorted[1..]
            .iter()
            .filter(|(_, s)| *s > 0.1)
            .map(|(d, _)| d.clone())
            .take(3)
            .collect();

        DomainResult {
            primary,
            secondary,
            tags: all_tags,
            confidence,
        }
    }

    /// Default domain keywords.
    fn default_domains() -> HashMap<String, Vec<String>> {
        let mut map = HashMap::new();
        
        map.insert("gameplay".into(), vec![
            "enemy".into(), "player".into(), "weapon".into(),
            "damage".into(), "health".into(), "ai".into(),
            "pathfinding".into(), "behavior".into(), "npc".into(),
        ]);
        
        map.insert("ui".into(), vec![
            "menu".into(), "button".into(), "hud".into(),
            "ui".into(), "interface".into(), "click".into(),
            "input".into(), "widget".into(), "panel".into(),
        ]);
        
        map.insert("graphics".into(), vec![
            "render".into(), "shader".into(), "texture".into(),
            "mesh".into(), "particle".into(), "light".into(),
            "camera".into(), "animation".into(), "sprite".into(),
        ]);
        
        map.insert("audio".into(), vec![
            "sound".into(), "music".into(), "audio".into(),
            "sfx".into(), "volume".into(), "mixer".into(),
        ]);
        
        map.insert("networking".into(), vec![
            "multiplayer".into(), "server".into(), "client".into(),
            "sync".into(), "network".into(), "online".into(),
            "lobby".into(), "rpc".into(),
        ]);
        
        map.insert("data".into(), vec![
            "save".into(), "load".into(), "file".into(),
            "config".into(), "json".into(), "database".into(),
            "serialize".into(), "deserialize".into(),
        ]);
        
        map.insert("performance".into(), vec![
            "fast".into(), "slow".into(), "optimize".into(),
            "lag".into(), "fps".into(), "memory".into(),
            "profiler".into(), "benchmark".into(),
        ]);
        
        map.insert("security".into(), vec![
            "auth".into(), "login".into(), "password".into(),
            "encrypt".into(), "secure".into(), "token".into(),
        ]);
        
        map.insert("build".into(), vec![
            "build".into(), "compile".into(), "cmake".into(),
            "link".into(), "library".into(), "package".into(),
        ]);
        
        map.insert("testing".into(), vec![
            "test".into(), "spec".into(), "mock".into(),
            "assert".into(), "coverage".into(), "unit".into(),
        ]);
        
        map.insert("memory".into(), vec![
            "leak".into(), "allocation".into(), "deallocate".into(),
            "garbage".into(), "pool".into(), "arena".into(),
        ]);
        
        map.insert("concurrency".into(), vec![
            "thread".into(), "async".into(), "parallel".into(),
            "mutex".into(), "lock".into(), "atomic".into(),
            "channel".into(), "future".into(),
        ]);
        
        map
    }

    /// Default tag keywords.
    fn default_tags() -> HashMap<String, Vec<String>> {
        let mut map = HashMap::new();
        
        map.insert("rust".into(), vec!["rust".into(), "cargo".into(), "crate".into()]);
        map.insert("cpp".into(), vec!["c++".into(), "cpp".into(), "cxx".into(), "hpp".into()]);
        map.insert("javascript".into(), vec!["javascript".into(), "js".into(), "typescript".into(), "ts".into(), "node".into()]);
        map.insert("python".into(), vec!["python".into(), "py".into()]);
        map.insert("go".into(), vec!["golang".into(), "go".into()]);
        
        map.insert("bug".into(), vec!["bug".into(), "crash".into(), "error".into(), "fix".into()]);
        map.insert("feature".into(), vec!["add".into(), "create".into(), "implement".into(), "new".into()]);
        map.insert("refactor".into(), vec!["refactor".into(), "restructure".into(), "clean".into()]);
        map.insert("docs".into(), vec!["document".into(), "comment".into(), "readme".into()]);
        
        map
    }
}

impl Default for DomainMapper {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_gameplay() {
        let mapper = DomainMapper::new();
        let result = mapper.map("Add enemy patrol behavior");
        
        assert_eq!(result.primary, "gameplay");
        assert!(result.confidence > 0.0);
    }

    #[test]
    fn test_map_graphics() {
        let mapper = DomainMapper::new();
        let result = mapper.map("Fix the shader rendering issue");
        
        assert_eq!(result.primary, "graphics");
    }

    #[test]
    fn test_map_networking() {
        let mapper = DomainMapper::new();
        let result = mapper.map("Implement multiplayer lobby system");
        
        assert_eq!(result.primary, "networking");
        assert!(!result.tags.is_empty() || !result.secondary.is_empty());
    }

    #[test]
    fn test_map_with_secondary() {
        let mapper = DomainMapper::new();
        let result = mapper.map("Add sound effects for enemy attacks");
        
        // Should have both gameplay and audio
        assert!(!result.primary.is_empty());
        assert!(!result.secondary.is_empty() || result.tags.len() > 1);
    }

    #[test]
    fn test_map_general() {
        let mapper = DomainMapper::new();
        let result = mapper.map("Do something");
        
        assert_eq!(result.primary, "general");
    }

    #[test]
    fn test_tags_extraction() {
        let mapper = DomainMapper::new();
        let result = mapper.map("Fix the bug in the Rust code");
        
        // Check that relevant tags are extracted
        assert!(!result.tags.is_empty());
    }
}
