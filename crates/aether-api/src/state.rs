//! App State — Shared state for API handlers

use std::sync::Arc;
use aether_validation::ValidationPipeline;
use aether_certification::Keypair;
use crate::auth::AuthService;

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    /// Validation pipeline
    pub pipeline: Arc<ValidationPipeline>,
    /// Auth service for API keys
    pub auth: Arc<AuthService>,
    /// Signing keypair for certificates (wrapped in Arc for Clone)
    pub keypair: Option<Arc<Keypair>>,
}

impl AppState {
    /// Create new state with default validation pipeline
    pub fn new() -> Self {
        Self {
            pipeline: Arc::new(ValidationPipeline::new()),
            auth: Arc::new(AuthService::new()),
            keypair: None,
        }
    }

    /// Create state with custom pipeline
    pub fn with_pipeline(pipeline: ValidationPipeline) -> Self {
        Self {
            pipeline: Arc::new(pipeline),
            auth: Arc::new(AuthService::new()),
            keypair: None,
        }
    }

    /// Set signing keypair
    pub fn with_keypair(mut self, keypair: Keypair) -> Self {
        self.keypair = Some(Arc::new(keypair));
        self
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
