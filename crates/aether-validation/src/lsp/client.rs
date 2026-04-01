//! LSP Client
//!
//! Client for communicating with Language Server Protocol servers.
//! Wraps JsonRpcTransport for LSP-specific protocol handling.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use serde_json::{json, Value};

use super::transport::{JsonRpcTransport, LspServerConfig};
use super::types::*;

/// Server capabilities received from initialize response.
#[derive(Debug, Clone, Default)]
pub struct ServerCapabilities {
    /// Server supports textDocument synchronization.
    pub text_document_sync: bool,
    /// Server provides diagnostics.
    pub diagnostics: bool,
    /// Server name.
    pub server_name: Option<String>,
    /// Server version.
    pub server_version: Option<String>,
}

/// LSP client for a specific language server.
pub struct LspClient {
    /// Server configuration.
    config: LspServerConfig,
    /// Transport layer.
    transport: Option<JsonRpcTransport>,
    /// Server capabilities (cached after initialize).
    capabilities: Option<ServerCapabilities>,
    /// Root URI for the project.
    root_uri: Option<String>,
    /// Cached diagnostics per URI.
    diagnostics_cache: Arc<Mutex<HashMap<String, Vec<LspDiagnostic>>>>,
    /// Initialized flag.
    initialized: bool,
}

impl LspClient {
    /// Create a new LSP client with the given configuration.
    pub fn new(config: LspServerConfig) -> Self {
        Self {
            config,
            transport: None,
            capabilities: None,
            root_uri: None,
            diagnostics_cache: Arc::new(Mutex::new(HashMap::new())),
            initialized: false,
        }
    }

    /// Create a new LSP client for a language (convenience method).
    pub fn for_language(language: &str) -> Option<Self> {
        LspServerConfig::for_language(language).map(Self::new)
    }

    /// Check if the LSP server is available.
    pub fn is_available(&self) -> bool {
        self.config.is_available()
    }

    /// Get the language name.
    pub fn language(&self) -> &str {
        &self.config.language
    }

    /// Get the root URI.
    pub fn root_uri(&self) -> Option<&str> {
        self.root_uri.as_deref()
    }

    /// Get server capabilities (after initialize).
    pub fn capabilities(&self) -> Option<&ServerCapabilities> {
        self.capabilities.as_ref()
    }

    /// Initialize the language server.
    ///
    /// Starts the server process and sends the initialize request.
    /// Must be called before any other LSP operations.
    ///
    /// # Panics
    ///
    /// Panics if the transport fails to start after being created internally.
    /// This should not happen in normal operation.
    pub fn initialize(&mut self, root_uri: &str) -> Result<ServerCapabilities, LspError> {
        // Start the transport if not already started
        if self.transport.is_none() {
            let transport = JsonRpcTransport::start(&self.config)?;
            self.transport = Some(transport);
        }

        // Safe: we just created the transport above if it was None
        let transport = self.transport.as_mut()
            .expect("transport should be initialized after start()");

        // Build initialize params per LSP spec
        let params = json!({
            "processId": null,
            "rootUri": root_uri,
            "capabilities": {
                "textDocument": {
                    "publishDiagnostics": {}
                }
            }
        });

        let result = transport.request("initialize", params)?;

        // Parse server capabilities from response
        let capabilities = Self::parse_capabilities(&result);
        self.capabilities = Some(capabilities.clone());
        self.root_uri = Some(root_uri.to_string());

        // Send initialized notification (required by LSP spec)
        transport.notify("initialized", json!({}))?;

        self.initialized = true;
        Ok(capabilities)
    }

    /// Parse server capabilities from initialize response.
    fn parse_capabilities(result: &Value) -> ServerCapabilities {
        let caps = result.get("capabilities").cloned().unwrap_or(json!({}));

        ServerCapabilities {
            text_document_sync: caps.get("textDocumentSync").is_some(),
            diagnostics: caps.get("diagnosticProvider").is_some()
                || caps.get("textDocumentSync").is_some(),
            server_name: result.get("serverInfo")
                .and_then(|info| info.get("name"))
                .and_then(|n| n.as_str())
                .map(|s| s.to_string()),
            server_version: result.get("serverInfo")
                .and_then(|info| info.get("version"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
        }
    }

    /// Open a text document in the language server.
    ///
    /// Sends textDocument/didOpen notification.
    ///
    /// # Panics
    ///
    /// Panics if the client is not initialized (transport is None).
    /// Call `initialize()` before this method.
    pub fn did_open(
        &mut self,
        uri: &str,
        language_id: &str,
        content: &str,
    ) -> Result<(), LspError> {
        self.require_initialized()?;
        let transport = self.transport.as_mut()
            .ok_or_else(|| LspError {
                code: -32002,
                message: "Transport not initialized".to_string(),
                data: None,
            })?;

        let params = json!({
            "textDocument": {
                "uri": uri,
                "languageId": language_id,
                "version": 1,
                "text": content
            }
        });

        transport.notify("textDocument/didOpen", params)
    }

    /// Request diagnostics for a document.
    ///
    /// For servers that support pull diagnostics (textDocument/diagnostic),
    /// sends a request. Otherwise, waits for publishDiagnostics notification.
    ///
    /// Returns cached diagnostics if available.
    ///
    /// # Panics
    ///
    /// Panics if the diagnostics cache lock is poisoned.
    /// Panics if the client is not initialized when calling pull_diagnostics.
    pub fn diagnostics(&mut self, uri: &str) -> Result<Vec<LspDiagnostic>, LspError> {
        self.require_initialized()?;

        // Check cache first
        {
            let cache = self.diagnostics_cache.lock().unwrap();
            if let Some(diags) = cache.get(uri) {
                return Ok(diags.clone());
            }
        }

        // Try pull diagnostics if server supports it
        if let Some(ref caps) = self.capabilities {
            if caps.diagnostics {
                return self.pull_diagnostics(uri);
            }
        }

        // Fall back to waiting for publishDiagnostics
        self.wait_for_diagnostics(uri)
    }

    /// Pull diagnostics using textDocument/diagnostic request.
    ///
    /// # Panics
    ///
    /// Panics if the transport is None (client not initialized).
    /// Panics if the diagnostics cache lock is poisoned.
    fn pull_diagnostics(&mut self, uri: &str) -> Result<Vec<LspDiagnostic>, LspError> {
        let transport = self.transport.as_mut()
            .ok_or_else(|| LspError {
                code: -32002,
                message: "Transport not initialized".to_string(),
                data: None,
            })?;

        let params = json!({
            "textDocument": {
                "uri": uri
            }
        });

        let result = transport.request("textDocument/diagnostic", params)?;

        // Parse diagnostics from response
        let diags = Self::parse_diagnostics_response(&result);

        // Cache the diagnostics
        {
            let mut cache = self.diagnostics_cache.lock().unwrap();
            cache.insert(uri.to_string(), diags.clone());
        }

        Ok(diags)
    }

    /// Wait for publishDiagnostics notification.
    fn wait_for_diagnostics(&mut self, _uri: &str) -> Result<Vec<LspDiagnostic>, LspError> {
        // For now, return empty vec - real implementation would poll for notifications
        // This requires async runtime for proper handling
        Ok(vec![])
    }

    /// Parse diagnostics from response.
    fn parse_diagnostics_response(result: &Value) -> Vec<LspDiagnostic> {
        // Handle both full and partial result
        let items = if let Some(items) = result.get("items") {
            items.as_array()
        } else { result.as_array() };

        match items {
            Some(arr) => arr
                .iter()
                .filter_map(|item| serde_json::from_value(item.clone()).ok())
                .collect(),
            None => vec![],
        }
    }

    /// Shutdown the language server gracefully.
    ///
    /// Sends shutdown request, then exit notification.
    pub fn shutdown(&mut self) -> Result<(), LspError> {
        if !self.initialized {
            return Ok(());
        }

        if let Some(ref mut transport) = self.transport {
            // Send shutdown request
            let _ = transport.request("shutdown", json!({}));

            // Send exit notification
            let _ = transport.notify("exit", json!({}));

            // Kill the process
            transport.kill();
        }

        self.transport = None;
        self.initialized = false;
        Ok(())
    }

    /// Check if client is initialized and transport is available.
    fn require_initialized(&self) -> Result<(), LspError> {
        if !self.initialized || self.transport.is_none() {
            Err(LspError {
                code: -32002,
                message: "Client not initialized".to_string(),
                data: None,
            })
        } else {
            Ok(())
        }
    }

    /// Update diagnostics cache (called when publishDiagnostics is received).
    ///
    /// # Panics
    ///
    /// Panics if the diagnostics cache lock is poisoned.
    pub fn update_diagnostics(&mut self, uri: &str, diagnostics: Vec<LspDiagnostic>) {
        let mut cache = self.diagnostics_cache.lock().unwrap();
        cache.insert(uri.to_string(), diagnostics);
    }

    /// Get cached diagnostics for a URI.
    ///
    /// # Panics
    ///
    /// Panics if the diagnostics cache lock is poisoned.
    pub fn cached_diagnostics(&self, uri: &str) -> Option<Vec<LspDiagnostic>> {
        let cache = self.diagnostics_cache.lock().unwrap();
        cache.get(uri).cloned()
    }
}

impl Drop for LspClient {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let config = LspServerConfig::for_language("rust").unwrap();
        let client = LspClient::new(config);
        assert_eq!(client.language(), "rust");
    }

    #[test]
    fn test_for_language() {
        let client = LspClient::for_language("typescript");
        assert!(client.is_some());
        assert_eq!(client.unwrap().language(), "typescript");
    }

    #[test]
    fn test_for_unknown_language() {
        let client = LspClient::for_language("unknown");
        assert!(client.is_none());
    }

    #[test]
    fn test_not_initialized_error() {
        let config = LspServerConfig::for_language("rust").unwrap();
        let mut client = LspClient::new(config);

        let result = client.did_open("file:///test.ts", "typescript", "const x = 1;");
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("not initialized"));
    }

    #[test]
    fn test_parse_capabilities() {
        let result = json!({
            "capabilities": {
                "textDocumentSync": 1
            },
            "serverInfo": {
                "name": "test-server",
                "version": "1.0.0"
            }
        });

        let caps = LspClient::parse_capabilities(&result);
        assert!(caps.text_document_sync);
        assert_eq!(caps.server_name, Some("test-server".to_string()));
        assert_eq!(caps.server_version, Some("1.0.0".to_string()));
    }

    #[test]
    fn test_parse_diagnostics() {
        let result = json!({
            "items": [
                {
                    "range": {
                        "start": { "line": 0, "character": 0 },
                        "end": { "line": 0, "character": 5 }
                    },
                    "severity": 1,
                    "message": "Test error"
                }
            ]
        });

        let diags = LspClient::parse_diagnostics_response(&result);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].message, "Test error");
    }
}
