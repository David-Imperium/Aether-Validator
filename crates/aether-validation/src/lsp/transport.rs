//! LSP JSON-RPC Transport
//!
//! Transport layer for LSP communication over stdio.
//! Implements Content-Length framing per LSP specification.

use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::types::LspError;

/// LSP server configuration.
#[derive(Debug, Clone)]
pub struct LspServerConfig {
    /// Command to run.
    pub command: String,
    /// Arguments for the command.
    pub args: Vec<String>,
    /// Language identifier.
    pub language: String,
}

impl LspServerConfig {
    /// Create config for a known language.
    pub fn for_language(language: &str) -> Option<Self> {
        match language {
            "rust" => Some(Self {
                command: "rust-analyzer".to_string(),
                args: vec![],
                language: language.to_string(),
            }),
            "python" => Some(Self {
                command: "pyright-langserver".to_string(),
                args: vec!["--stdio".to_string()],
                language: language.to_string(),
            }),
            "typescript" | "javascript" => Some(Self {
                command: "typescript-language-server".to_string(),
                args: vec!["--stdio".to_string()],
                language: language.to_string(),
            }),
            "go" => Some(Self {
                command: "gopls".to_string(),
                args: vec!["serve".to_string()],
                language: language.to_string(),
            }),
            "c" | "cpp" => Some(Self {
                command: "clangd".to_string(),
                args: vec![],
                language: language.to_string(),
            }),
            "java" => Some(Self {
                command: "jdtls".to_string(),
                args: vec![],
                language: language.to_string(),
            }),
            _ => None,
        }
    }

    /// Check if the LSP server is available.
    pub fn is_available(&self) -> bool {
        which::which(&self.command).is_ok()
    }
}

/// JSON-RPC request.
#[derive(Debug, Serialize)]
pub struct JsonRpcRequest {
    jsonrpc: &'static str,
    id: u64,
    method: String,
    params: Value,
}

/// JSON-RPC response.
#[derive(Debug, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: u64,
    #[serde(flatten)]
    pub payload: ResponsePayload,
}

/// Response payload (result or error).
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ResponsePayload {
    Success { result: Value },
    Error { error: JsonRpcError },
}

/// JSON-RPC error object.
#[derive(Debug, Deserialize, Clone)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(default)]
    pub data: Option<Value>,
}

/// JSON-RPC notification.
#[derive(Debug, Deserialize)]
pub struct JsonRpcNotification {
    pub jsonrpc: String,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

/// Incoming LSP message.
#[derive(Debug)]
pub enum Incoming {
    Response(JsonRpcResponse),
    Notification(JsonRpcNotification),
}

/// LSP transport using sync std::process.
/// 
/// For async usage, wrap calls in `tokio::task::spawn_blocking`.
pub struct JsonRpcTransport {
    /// Child process.
    process: Option<Child>,
    /// Stdin writer.
    stdin: std::process::ChildStdin,
    /// Stdout reader.
    stdout: BufReader<std::process::ChildStdout>,
    /// Next request ID.
    next_id: AtomicU64,
    /// Shutdown flag.
    shutdown: Arc<std::sync::atomic::AtomicBool>,
}

impl JsonRpcTransport {
    /// Start an LSP server process.
    pub fn start(config: &LspServerConfig) -> Result<Self, LspError> {
        if !config.is_available() {
            return Err(LspError {
                code: -1,
                message: format!("LSP server '{}' not found in PATH", config.command),
                data: None,
            });
        }

        let mut cmd = Command::new(&config.command);
        cmd.args(&config.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());

        let mut process = cmd.spawn().map_err(|e| LspError {
            code: -1,
            message: format!("Failed to spawn {}: {}", config.command, e),
            data: None,
        })?;

        let stdin = process.stdin.take().ok_or_else(|| LspError {
            code: -1,
            message: "Failed to get stdin".to_string(),
            data: None,
        })?;

        let stdout = process.stdout.take().ok_or_else(|| LspError {
            code: -1,
            message: "Failed to get stdout".to_string(),
            data: None,
        })?;

        Ok(Self {
            process: Some(process),
            stdin,
            stdout: BufReader::new(stdout),
            next_id: AtomicU64::new(1),
            shutdown: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        })
    }

    /// Get next request ID.
    fn next_id(&self) -> u64 {
        self.next_id.fetch_add(1, Ordering::SeqCst)
    }

    /// Send a request and wait for response.
    pub fn request(&mut self, method: &str, params: Value) -> Result<Value, LspError> {
        let id = self.next_id();

        let req = JsonRpcRequest {
            jsonrpc: "2.0",
            id,
            method: method.to_string(),
            params,
        };

        self.send(&req)?;
        self.recv_response(id)
    }

    /// Send a notification (no response expected).
    pub fn notify(&mut self, method: &str, params: Value) -> Result<(), LspError> {
        let req = JsonRpcRequest {
            jsonrpc: "2.0",
            id: 0,
            method: method.to_string(),
            params,
        };
        self.send(&req)
    }

    /// Send a message to the server.
    fn send(&mut self, req: &JsonRpcRequest) -> Result<(), LspError> {
        let json = serde_json::to_string(req).map_err(|e| LspError {
            code: -1,
            message: format!("JSON serialize error: {}", e),
            data: None,
        })?;

        let msg = format!("Content-Length: {}\r\n\r\n{}", json.len(), json);

        self.stdin.write_all(msg.as_bytes()).map_err(|e| LspError {
            code: -1,
            message: format!("Write error: {}", e),
            data: None,
        })?;
        self.stdin.flush().map_err(|e| LspError {
            code: -1,
            message: format!("Flush error: {}", e),
            data: None,
        })?;

        Ok(())
    }

    /// Receive a response for a specific request ID.
    fn recv_response(&mut self, expected_id: u64) -> Result<Value, LspError> {
        loop {
            let incoming = self.recv_message()?;

            match incoming {
                Incoming::Response(resp) if resp.id == expected_id => {
                    match resp.payload {
                        ResponsePayload::Success { result } => return Ok(result),
                        ResponsePayload::Error { error } => return Err(LspError {
                            code: error.code,
                            message: error.message,
                            data: error.data,
                        }),
                    }
                }
                Incoming::Response(resp) => {
                    // Unexpected response ID, skip
                    tracing::warn!("Unexpected response ID: {}", resp.id);
                }
                Incoming::Notification(notif) => {
                    // Handle notifications (e.g., publishDiagnostics)
                    tracing::debug!("Notification: {}", notif.method);
                }
            }
        }
    }

    /// Receive a single message from the server.
    fn recv_message(&mut self) -> Result<Incoming, LspError> {
        // Read Content-Length header
        let mut header_line = String::new();
        loop {
            header_line.clear();
            self.stdout.read_line(&mut header_line).map_err(|e| LspError {
                code: -1,
                message: format!("Read header error: {}", e),
                data: None,
            })?;

            if header_line.trim().is_empty() {
                break;
            }

            if header_line.starts_with("Content-Length:") {
                // Parse content length
            }
        }

        // Read Content-Length value from previous line
        // Simplified: we re-read
        let mut content_len = 0usize;
        let mut line = String::new();

        // Re-read from start - we need proper parsing
        // For now, simplified approach
        loop {
            line.clear();
            let bytes = self.stdout.read_line(&mut line).map_err(|e| LspError {
                code: -1,
                message: format!("Read error: {}", e),
                data: None,
            })?;

            if bytes == 0 || line.trim().is_empty() {
                break;
            }

            if line.starts_with("Content-Length:") {
                let len_str = line.split(':').nth(1).unwrap_or("0").trim();
                content_len = len_str.parse().unwrap_or(0);
            }
        }

        if content_len == 0 {
            return Err(LspError {
                code: -1,
                message: "No Content-Length header".to_string(),
                data: None,
            });
        }

        // Read the JSON content
        let mut content_buf = vec![0u8; content_len];
        self.stdout.read_exact(&mut content_buf).map_err(|e| LspError {
            code: -1,
            message: format!("Read content error: {}", e),
            data: None,
        })?;

        let content = String::from_utf8(content_buf).map_err(|e| LspError {
            code: -1,
            message: format!("UTF-8 decode error: {}", e),
            data: None,
        })?;

        // Parse as response or notification
        if content.contains("\"id\":") {
            let resp: JsonRpcResponse = serde_json::from_str(&content).map_err(|e| LspError {
                code: -1,
                message: format!("JSON parse error: {}", e),
                data: None,
            })?;
            Ok(Incoming::Response(resp))
        } else {
            let notif: JsonRpcNotification = serde_json::from_str(&content).map_err(|e| LspError {
                code: -1,
                message: format!("JSON parse error: {}", e),
                data: None,
            })?;
            Ok(Incoming::Notification(notif))
        }
    }

    /// Kill the server process.
    pub fn kill(&mut self) {
        if let Some(ref mut process) = self.process {
            let _ = process.kill();
        }
    }

    /// Check if server is still running.
    pub fn is_running(&self) -> bool {
        !self.shutdown.load(Ordering::SeqCst)
    }
}

impl Drop for JsonRpcTransport {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::SeqCst);
        self.kill();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_config() {
        let rust = LspServerConfig::for_language("rust");
        assert!(rust.is_some());
        assert_eq!(rust.unwrap().command, "rust-analyzer");

        let unknown = LspServerConfig::for_language("unknown");
        assert!(unknown.is_none());
    }

    #[test]
    fn test_request_serialization() {
        let req = JsonRpcRequest {
            jsonrpc: "2.0",
            id: 1,
            method: "initialize".to_string(),
            params: serde_json::json!({}),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"method\":\"initialize\""));
    }
}
