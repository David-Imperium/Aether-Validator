//! Aether HTTP API
//!
//! REST endpoints for validation, certification, and analysis.

mod server;
mod routes;
mod handlers;
mod error;
mod auth;
mod state;

pub use server::ApiServer;
pub use error::{ApiError, ApiResult};
pub use auth::{AuthService, ApiKey};
pub use state::AppState;

/// API version
pub const API_VERSION: &str = "v1";

/// Default server address
pub const DEFAULT_ADDR: &str = "127.0.0.1:3000";
