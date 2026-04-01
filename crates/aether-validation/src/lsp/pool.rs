//! LSP Client Pool
//!
//! Pool for caching and reusing LSP clients per language.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use super::client::LspClient;
use super::types::LspError;

/// Pool for managing LSP clients per language.
///
/// Provides lazy initialization and caching of LSP clients.
/// Each language gets its own cached client.
pub struct LspClientPool {
    /// Cached clients by language name.
    clients: HashMap<String, Arc<Mutex<LspClient>>>,
    /// Root URI for the project.
    root_uri: Option<String>,
}

impl LspClientPool {
    /// Create a new empty LSP client pool.
    ///
    /// # Arguments
    /// * `root_uri` - Optional root URI for the project (passed to LSP clients)
    pub fn new(root_uri: Option<String>) -> Self {
        Self {
            clients: HashMap::new(),
            root_uri,
        }
    }

    /// Get or create an LSP client for the given language.
    ///
    /// Lazy initialization: creates and initializes the client on first call.
    /// Subsequent calls return the cached client.
    ///
    /// # Arguments
    /// * `language` - Language identifier (e.g., "rust", "python", "typescript")
    ///
    /// # Returns
    /// * `Ok(Arc<Mutex<LspClient>>)` - The cached or newly created client
    /// * `Err(LspError)` - If client creation or initialization fails
    pub fn get(&mut self, language: &str) -> Result<Arc<Mutex<LspClient>>, LspError> {
        // Check if we already have a cached client
        if let Some(client) = self.clients.get(language) {
            return Ok(Arc::clone(client));
        }

        // Create new client using for_language convenience method
        let mut client = LspClient::for_language(language).ok_or_else(|| LspError {
            code: -1,
            message: format!("No LSP server available for language: {}", language),
            data: None,
        })?;

        // Initialize the client with root_uri
        let root_uri = self.root_uri.clone().unwrap_or_else(|| "file:///".to_string());
        client.initialize(&root_uri)?;

        // Wrap in Arc<Mutex> and cache
        let client_arc = Arc::new(Mutex::new(client));
        self.clients.insert(language.to_string(), Arc::clone(&client_arc));

        Ok(client_arc)
    }

    /// Shutdown all cached LSP clients.
    ///
    /// Calls shutdown on each client and clears the pool.
    pub fn shutdown_all(&mut self) {
        for (language, client_arc) in self.clients.drain() {
            if let Ok(mut client) = client_arc.lock() {
                if let Err(e) = client.shutdown() {
                    tracing::warn!("Failed to shutdown LSP client for {}: {}", language, e.message);
                }
            }
        }
    }

    /// Check if a client for the given language is cached.
    pub fn has_client(&self, language: &str) -> bool {
        self.clients.contains_key(language)
    }

    /// Get the number of cached clients.
    pub fn client_count(&self) -> usize {
        self.clients.len()
    }

    /// Get the root URI.
    pub fn root_uri(&self) -> Option<&str> {
        self.root_uri.as_deref()
    }
}

impl Drop for LspClientPool {
    fn drop(&mut self) {
        self.shutdown_all();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_new() {
        let pool = LspClientPool::new(Some("file:///test".to_string()));
        assert_eq!(pool.client_count(), 0);
        assert_eq!(pool.root_uri(), Some("file:///test"));
    }

    #[test]
    fn test_pool_empty_root_uri() {
        let pool = LspClientPool::new(None);
        assert_eq!(pool.root_uri(), None);
    }

    #[test]
    fn test_pool_has_client() {
        let pool = LspClientPool::new(None);
        assert!(!pool.has_client("rust"));
    }

    #[test]
    fn test_pool_shutdown_empty() {
        let mut pool = LspClientPool::new(None);
        pool.shutdown_all();
        assert_eq!(pool.client_count(), 0);
    }
}
