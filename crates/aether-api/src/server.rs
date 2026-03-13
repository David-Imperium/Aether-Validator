//! API Server

use std::net::SocketAddr;
use std::sync::Arc;
use tokio::signal;
use tower_http::trace::TraceLayer;
use tracing::info;

use crate::auth::AuthService;
use crate::routes;
use crate::state::AppState;

/// API Server
pub struct ApiServer {
    addr: SocketAddr,
    state: Arc<AppState>,
}

impl ApiServer {
    /// Create new API server with default pipeline
    pub fn new(addr: impl Into<String>) -> Self {
        let addr_str = addr.into();
        let addr: SocketAddr = addr_str.parse()
            .unwrap_or_else(|_| crate::DEFAULT_ADDR.parse().expect("DEFAULT_ADDR must be valid"));
        
        Self {
            addr,
            state: Arc::new(AppState::new()),
        }
    }

    /// Create server with custom state
    pub fn with_state(addr: impl Into<String>, state: Arc<AppState>) -> Self {
        let addr_str = addr.into();
        let addr: SocketAddr = addr_str.parse()
            .unwrap_or_else(|_| crate::DEFAULT_ADDR.parse().expect("DEFAULT_ADDR must be valid"));
        
        Self { addr, state }
    }

    /// Get auth service
    pub fn auth(&self) -> Arc<AuthService> {
        self.state.auth.clone()
    }

    /// Get app state
    pub fn state(&self) -> Arc<AppState> {
        self.state.clone()
    }

    /// Start the server
    pub async fn start(self) -> Result<(), Box<dyn std::error::Error>> {
        let router = routes::create_router(self.state.clone())
            .layer(TraceLayer::new_for_http());

        let listener = tokio::net::TcpListener::bind(self.addr).await?;
        info!("API server listening on {}", self.addr);

        axum::serve(listener, router)
            .with_graceful_shutdown(shutdown_signal())
            .await?;

        Ok(())
    }
}

/// Graceful shutdown signal
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("Shutdown signal received");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_creation() {
        let server = ApiServer::new("127.0.0.1:8080");
        assert!(server.addr.port() == 8080);
    }
}
