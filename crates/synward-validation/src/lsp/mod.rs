//! LSP Integration Module
//!
//! Provides integration with Language Server Protocol servers for deep semantic analysis.
//! Used on-demand when tree-sitter analysis is insufficient.

mod client;
mod types;
mod layer;
pub mod transport;
mod pool;

pub use client::LspClient;
pub use types::{LspError, LspDiagnostic, LspPosition};
pub use layer::LspAnalysisLayer;
pub use transport::{JsonRpcTransport, LspServerConfig, JsonRpcRequest, JsonRpcResponse, JsonRpcError};
pub use pool::LspClientPool;
