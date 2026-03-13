//! Aether API Server Binary

use aether_api::ApiServer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Get address from environment or use default
    let addr = std::env::var("AETHER_API_ADDR")
        .unwrap_or_else(|_| aether_api::DEFAULT_ADDR.to_string());

    tracing::info!("Starting Aether API server on {}", addr);

    // Create and start server
    let server = ApiServer::new(&addr);
    server.start().await?;

    Ok(())
}
