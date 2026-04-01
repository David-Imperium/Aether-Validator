//! LSP Types
//!
//! Types for Language Server Protocol communication.

use serde::{Deserialize, Serialize};

/// LSP position in a document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LspPosition {
    pub line: u32,
    pub character: u32,
}

/// LSP range in a document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LspRange {
    pub start: LspPosition,
    pub end: LspPosition,
}

/// LSP diagnostic severity.
/// LSP uses integers: 1=Error, 2=Warning, 3=Information, 4=Hint
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LspDiagnosticSeverity {
    Error,
    Warning,
    Information,
    Hint,
}

impl Serialize for LspDiagnosticSeverity {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let value = match self {
            LspDiagnosticSeverity::Error => 1,
            LspDiagnosticSeverity::Warning => 2,
            LspDiagnosticSeverity::Information => 3,
            LspDiagnosticSeverity::Hint => 4,
        };
        serializer.serialize_i32(value)
    }
}

impl<'de> Deserialize<'de> for LspDiagnosticSeverity {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        match value {
            serde_json::Value::Number(n) => {
                match n.as_i64() {
                    Some(1) => Ok(LspDiagnosticSeverity::Error),
                    Some(2) => Ok(LspDiagnosticSeverity::Warning),
                    Some(3) => Ok(LspDiagnosticSeverity::Information),
                    Some(4) => Ok(LspDiagnosticSeverity::Hint),
                    _ => Err(serde::de::Error::custom(format!("Invalid severity: {}", n))),
                }
            }
            serde_json::Value::String(s) => {
                match s.to_lowercase().as_str() {
                    "error" => Ok(LspDiagnosticSeverity::Error),
                    "warning" => Ok(LspDiagnosticSeverity::Warning),
                    "information" => Ok(LspDiagnosticSeverity::Information),
                    "hint" => Ok(LspDiagnosticSeverity::Hint),
                    _ => Err(serde::de::Error::custom(format!("Invalid severity: {}", s))),
                }
            }
            _ => Err(serde::de::Error::custom("Expected number or string for severity")),
        }
    }
}

/// LSP diagnostic from language server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspDiagnostic {
    pub range: LspRange,
    pub severity: Option<LspDiagnosticSeverity>,
    pub code: Option<serde_json::Value>,
    pub source: Option<String>,
    pub message: String,
    pub related_information: Option<Vec<LspDiagnosticRelated>>,
}

/// Related diagnostic information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspDiagnosticRelated {
    pub location: LspLocation,
    pub message: String,
}

/// LSP location (URI + range).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspLocation {
    pub uri: String,
    pub range: LspRange,
}

/// LSP error response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspError {
    pub code: i32,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

/// LSP initialize params.
#[derive(Debug, Clone, Serialize)]
#[allow(dead_code)]
pub struct LspInitializeParams {
    pub process_id: Option<u32>,
    pub root_uri: Option<String>,
    pub capabilities: LspClientCapabilities,
}

/// LSP client capabilities.
#[derive(Debug, Clone, Serialize)]
#[allow(dead_code)]
pub struct LspClientCapabilities {
    pub text_document: TextDocumentClientCapabilities,
}

/// Text document client capabilities.
#[derive(Debug, Clone, Serialize)]
#[allow(dead_code)]
pub struct TextDocumentClientCapabilities {
    pub publish_diagnostics: PublishDiagnosticsCapabilities,
}

/// Publish diagnostics capabilities.
#[derive(Debug, Clone, Serialize)]
#[allow(dead_code)]
pub struct PublishDiagnosticsCapabilities {
    pub related_information: bool,
}

/// LSP text document identifier.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct LspTextDocumentIdentifier {
    pub uri: String,
}

/// LSP text document item.
#[derive(Debug, Clone, Serialize)]
#[allow(dead_code)]
pub struct LspTextDocumentItem {
    pub uri: String,
    pub language_id: String,
    pub version: i32,
    pub text: String,
}

/// LSP did open text document params.
#[derive(Debug, Clone, Serialize)]
#[allow(dead_code)]
pub struct LspDidOpenTextDocumentParams {
    pub text_document: LspTextDocumentItem,
}

/// LSP did change text document params.
#[derive(Debug, Clone, Serialize)]
#[allow(dead_code)]
pub struct LspDidChangeTextDocumentParams {
    pub text_document: LspVersionedTextDocumentIdentifier,
    pub content_changes: Vec<LspTextDocumentContentChangeEvent>,
}

/// LSP versioned text document identifier.
#[derive(Debug, Clone, Serialize)]
#[allow(dead_code)]
pub struct LspVersionedTextDocumentIdentifier {
    pub uri: String,
    pub version: i32,
}

/// LSP text document content change event.
#[derive(Debug, Clone, Serialize)]
#[allow(dead_code)]
pub struct LspTextDocumentContentChangeEvent {
    pub text: String,
}

impl Default for LspClientCapabilities {
    fn default() -> Self {
        Self {
            text_document: TextDocumentClientCapabilities {
                publish_diagnostics: PublishDiagnosticsCapabilities {
                    related_information: true,
                },
            },
        }
    }
}

impl LspInitializeParams {
    #[allow(dead_code)]
    pub fn new(root_uri: Option<String>) -> Self {
        Self {
            process_id: Some(std::process::id()),
            root_uri,
            capabilities: LspClientCapabilities::default(),
        }
    }
}
