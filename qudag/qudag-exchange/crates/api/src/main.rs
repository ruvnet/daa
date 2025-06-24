//! QuDAG Exchange API Server
//!
//! HTTP API interface for the QuDAG Exchange system.

use anyhow::Result;
use axum::{Router, Server};
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use tracing::info;

mod routes;
mod handlers;
mod auth;
mod state;
mod error;

use routes::api_routes;
use state::AppState;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "qudag_exchange_api=debug,tower_http=debug".into()),
        )
        .init();

    info!("Starting QuDAG Exchange API Server");

    // Initialize application state
    let state = AppState::new().await?;

    // Build the application
    let app = Router::new()
        .nest("/api/v1", api_routes())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // Define the address to listen on
    let addr = SocketAddr::from(([0, 0, 0, 0], 8585));
    info!("Listening on {}", addr);

    // Run the server
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}