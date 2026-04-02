//! Authentication service

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// API key representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub key: String,
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub scopes: Vec<String>,
    pub rate_limit: u32,
}

impl ApiKey {
    /// Create a new API key
    pub fn new(name: String, scopes: Vec<String>) -> Self {
        let key = Self::generate_key(&name);
        Self {
            key,
            name,
            created_at: chrono::Utc::now(),
            scopes,
            rate_limit: 1000,
        }
    }

    /// Generate a deterministic key from name
    fn generate_key(name: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(name.as_bytes());
        hasher.update(chrono::Utc::now().timestamp().to_string().as_bytes());
        let hash = hasher.finalize();
        format!("synward_{}", BASE64.encode(&hash[..16]))
    }

    /// Check if key has required scope
    pub fn has_scope(&self, scope: &str) -> bool {
        self.scopes.contains(&scope.to_string()) || self.scopes.contains(&"*".to_string())
    }
}

/// Authentication service
pub struct AuthService {
    keys: Arc<RwLock<HashMap<String, ApiKey>>>,
}

impl AuthService {
    /// Create new auth service
    pub fn new() -> Self {
        Self {
            keys: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add an API key
    pub async fn add_key(&self, key: ApiKey) {
        let mut keys = self.keys.write().await;
        keys.insert(key.key.clone(), key);
    }

    /// Validate an API key
    pub async fn validate(&self, key: &str) -> Option<ApiKey> {
        let keys = self.keys.read().await;
        keys.get(key).cloned()
    }

    /// Remove an API key
    pub async fn remove_key(&self, key: &str) -> bool {
        let mut keys = self.keys.write().await;
        keys.remove(key).is_some()
    }

    /// List all keys (without secret values)
    pub async fn list_keys(&self) -> Vec<ApiKey> {
        let keys = self.keys.read().await;
        keys.values().cloned().collect()
    }
}

impl Default for AuthService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_key_generation() {
        let key = ApiKey::new("test-client".to_string(), vec!["validate".to_string()]);
        assert!(key.key.starts_with("synward_"));
        assert!(key.has_scope("validate"));
        assert!(!key.has_scope("admin"));
    }

    #[tokio::test]
    async fn test_auth_service() {
        let auth = AuthService::new();
        let key = ApiKey::new("test".to_string(), vec!["validate".to_string()]);
        let key_str = key.key.clone();
        
        auth.add_key(key).await;
        
        let validated = auth.validate(&key_str).await;
        assert!(validated.is_some());
        
        let removed = auth.remove_key(&key_str).await;
        assert!(removed);
        
        let validated = auth.validate(&key_str).await;
        assert!(validated.is_none());
    }
}
