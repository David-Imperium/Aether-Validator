//! Intent Inference via External API

use crate::error::{Error, Result};
use crate::intent::Intent;

/// Intent inferrer using external LLM API
pub struct IntentInferrer {
    /// API endpoint
    endpoint: Option<String>,

    /// HTTP client
    client: Option<reqwest::Client>,
}

impl IntentInferrer {
    /// Create a new inferrer
    pub fn new(endpoint: Option<String>) -> Self {
        let client = endpoint.as_ref().map(|_| reqwest::Client::new());

        Self { endpoint, client }
    }

    /// Check if endpoint is configured
    pub fn is_configured(&self) -> bool {
        self.endpoint.is_some()
    }

    /// Infer the intent of a code block
    pub async fn infer(&self, code: &str) -> Result<Intent> {
        let endpoint = self.endpoint.as_ref().ok_or_else(|| {
            Error::Config("LLM API endpoint not configured".to_string())
        })?;

        let client = self.client.as_ref().ok_or_else(|| {
            Error::Config("HTTP client not initialized".to_string())
        })?;

        let prompt = format!(
            r#"Analyze this code and infer its intent. Respond in JSON:
{{
  "summary": "one-line description",
  "purpose": "what it achieves",
  "invariants": ["conditions maintained"],
  "side_effects": ["external effects"],
  "dependencies": ["what it relies on"],
  "confidence": 0.0-1.0
}}

Code:
```
{}
```"#,
            code
        );

        let response = client
            .post(endpoint)
            .json(&serde_json::json!({ "prompt": prompt }))
            .send()
            .await
            .map_err(|e| Error::Config(format!("API request failed: {}", e)))?;

        let text = response
            .text()
            .await
            .map_err(|e| Error::Config(format!("Failed to read response: {}", e)))?;

        // Try to parse as Intent
        let intent: Intent = serde_json::from_str(&text)
            .unwrap_or_else(|_| Intent {
                summary: text.lines().next().unwrap_or("").to_string(),
                purpose: text,
                invariants: vec![],
                side_effects: vec![],
                dependencies: vec![],
                confidence: 0.5,
            });

        Ok(intent)
    }

    /// Infer with context
    pub async fn infer_with_context(&self, code: &str, context: &str) -> Result<Intent> {
        let endpoint = self.endpoint.as_ref().ok_or_else(|| {
            Error::Config("LLM API endpoint not configured".to_string())
        })?;

        let client = self.client.as_ref().ok_or_else(|| {
            Error::Config("HTTP client not initialized".to_string())
        })?;

        let prompt = format!(
            r#"Context: {}

Analyze this code and infer its intent. Respond in JSON.

Code:
```
{}
```"#,
            context, code
        );

        let response = client
            .post(endpoint)
            .json(&serde_json::json!({ "prompt": prompt }))
            .send()
            .await
            .map_err(|e| Error::Config(format!("API request failed: {}", e)))?;

        let text = response
            .text()
            .await
            .map_err(|e| Error::Config(format!("Failed to read response: {}", e)))?;

        let intent: Intent = serde_json::from_str(&text).unwrap_or_else(|_| Intent {
            summary: "Could not parse response".to_string(),
            purpose: text,
            invariants: vec![],
            side_effects: vec![],
            dependencies: vec![],
            confidence: 0.3,
        });

        Ok(intent)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inferrer_new_without_endpoint() {
        let inferrer = IntentInferrer::new(None);
        assert!(!inferrer.is_configured());
    }

    #[test]
    fn test_inferrer_new_with_endpoint() {
        let inferrer = IntentInferrer::new(Some("http://localhost:8080/api".to_string()));
        assert!(inferrer.is_configured());
    }

    #[test]
    fn test_intent_default() {
        let intent = Intent::default();
        assert!(intent.summary.is_empty());
        assert!(intent.purpose.is_empty());
        assert!(intent.invariants.is_empty());
        assert_eq!(intent.confidence, 0.0);
    }

    #[test]
    fn test_intent_serialization() {
        let intent = Intent {
            summary: "Test function".to_string(),
            purpose: "Testing".to_string(),
            invariants: vec!["x > 0".to_string()],
            side_effects: vec![],
            dependencies: vec!["std".to_string()],
            confidence: 0.9,
        };

        let json = serde_json::to_string(&intent).unwrap();
        assert!(json.contains("Test function"));
        assert!(json.contains("0.9"));
    }

    #[tokio::test]
    async fn test_infer_without_endpoint_fails() {
        let inferrer = IntentInferrer::new(None);
        let result = inferrer.infer("fn main() {}").await;
        assert!(result.is_err());
    }
}
