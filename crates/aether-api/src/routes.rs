//! API routes

use axum::{
    routing::{get, post},
    Router,
};
use tower_http::cors::{Any, CorsLayer};
use std::sync::Arc;

use crate::state::AppState;
use crate::handlers;

/// Create the API router
pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        // Health check
        .route("/api/v1/health", get(handlers::health))
        
        // Validation endpoints
        .route("/api/v1/validate", post(handlers::validate))
        .route("/api/v1/certify", post(handlers::certify))
        .route("/api/v1/verify", post(handlers::verify))
        .route("/api/v1/analyze", post(handlers::analyze))
        
        // Key management
        .route("/api/v1/keys", get(handlers::list_keys))
        
        // CORS
        .layer(CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any)
        )
        
        // App state
        .with_state(state)
}

/// Create public router (no auth required for health)
#[allow(dead_code)]
pub fn create_public_router() -> Router {
    Router::new()
        .route("/api/v1/health", get(handlers::health))
        .layer(CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any)
        )
}
