//! API route definitions

use axum::{
    routing::{get, post},
    Router,
};
use crate::{handlers, state::AppState};

/// Build the API routes
pub fn api_routes() -> Router<AppState> {
    Router::new()
        // Account management
        .route("/accounts", post(handlers::create_account))
        .route("/accounts/:id/balance", get(handlers::get_balance))
        
        // Transactions
        .route("/transactions", post(handlers::submit_transaction))
        .route("/transactions/:id", get(handlers::get_transaction))
        
        // Resource monitoring
        .route("/resources/status", get(handlers::resource_status))
        .route("/resources/costs", get(handlers::resource_costs))
        
        // Consensus information
        .route("/consensus/info", get(handlers::consensus_info))
        .route("/consensus/peers", get(handlers::list_peers))
        
        // Health check
        .route("/health", get(handlers::health_check))
}