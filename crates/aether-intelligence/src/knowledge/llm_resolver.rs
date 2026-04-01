//! LLM API Resolver - Fallback to LLM for unknown API signatures

use crate::error::Result;
use crate::knowledge::ApiSignature;
use lru::LruCache;
use std::num::NonZeroUsize;

/// LLM-based resolver for unknown API signatures
///
/// Currently a placeholder that only uses cache.
/// Full LLM integration requires the `intent-api` feature.
pub struct LlmApiResolver {
    /// Cache of resolved signatures
    cache: LruCache<String, ApiSignature>,
}

impl LlmApiResolver {
    /// Create a new resolver (cache only for now)
    pub fn new_cache_only() -> Self {
        Self {
            cache: LruCache::new(NonZeroUsize::new(1000).unwrap()),
        }
    }

    /// Resolve an API signature
    ///
    /// Currently returns None for unknown APIs (placeholder).
    /// With `intent-api` feature, would query external LLM.
    pub async fn resolve(&mut self, module: &str, function: &str) -> Result<Option<ApiSignature>> {
        let cache_key = format!("{}.{}", module, function);

        // Check cache first
        if let Some(sig) = self.cache.get(&cache_key) {
            tracing::debug!("Cache hit for {}", cache_key);
            return Ok(Some(sig.clone()));
        }

        // Placeholder: no LLM integration yet
        tracing::debug!("No signature for {}", cache_key);
        Ok(None)
    }

    /// Add a signature to the cache manually
    pub fn cache_signature(&mut self, sig: ApiSignature) {
        let key = format!("{}.{}", sig.module, sig.function);
        self.cache.put(key, sig);
    }

    /// Get cache size
    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }
}

impl Default for LlmApiResolver {
    fn default() -> Self {
        Self::new_cache_only()
    }
}

#[cfg(feature = "intent-api")]
mod api_provider {
    use crate::error::{Error, Result};
    use crate::knowledge::ApiSignature;
    use lru::LruCache;
    use std::num::NonZeroUsize;

    /// HTTP-based LLM provider using external API
    pub struct HttpLlmResolver {
        cache: LruCache<String, ApiSignature>,
        client: reqwest::Client,
        endpoint: String,
        api_key: Option<String>,
    }

    impl HttpLlmResolver {
        pub fn new(endpoint: impl Into<String>, api_key: Option<String>) -> Self {
            Self {
                cache: LruCache::new(NonZeroUsize::new(1000).unwrap()),
                client: reqwest::Client::new(),
                endpoint: endpoint.into(),
                api_key,
            }
        }

        pub async fn resolve(&mut self, module: &str, function: &str) -> Result<Option<ApiSignature>> {
            let cache_key = format!("{}.{}", module, function);

            if let Some(sig) = self.cache.get(&cache_key) {
                return Ok(Some(sig.clone()));
            }

            let prompt = format!(
                "What are the function signature and parameter order for {}::{}? Output JSON.",
                module, function
            );

            let mut request = self.client.post(&self.endpoint);
            if let Some(key) = &self.api_key {
                request = request.bearer_auth(key);
            }

            let response = request
                .json(&serde_json::json!({ "prompt": prompt }))
                .send()
                .await
                .map_err(|e| Error::Config(e.to_string()))?;

            let text = response.text().await
                .map_err(|e| Error::Config(e.to_string()))?;

            if text.contains("unknown") {
                return Ok(None);
            }

            let sig: ApiSignature = serde_json::from_str(&text)
                .map_err(|e| Error::Config(format!("Failed to parse LLM response: {}", e)))?;

            self.cache.put(cache_key, sig.clone());
            Ok(Some(sig))
        }
    }
}

#[cfg(feature = "intent-api")]
pub use api_provider::HttpLlmResolver;
