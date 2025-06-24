//! MCP (Model Context Protocol) server management module

use crate::CliError;
use std::path::PathBuf;

pub mod config;
pub mod server;

pub use config::*;
pub use server::*;

/// Handle MCP server start command
pub async fn handle_mcp_start(
    bind: String,
    transport: String,
    config: Option<PathBuf>,
    verbose: bool,
    background: bool,
) -> Result<(), CliError> {
    if background {
        start_mcp_server_background(bind, transport, config, verbose).await
    } else {
        start_mcp_server_foreground(bind, transport, config, verbose).await
    }
}

/// Handle MCP server stop command
pub async fn handle_mcp_stop(force: bool) -> Result<(), CliError> {
    stop_mcp_server(force).await
}

/// Handle MCP server status command
pub async fn handle_mcp_status() -> Result<(), CliError> {
    show_mcp_server_status().await
}

/// Handle MCP config show command
pub async fn handle_mcp_config_show() -> Result<(), CliError> {
    show_mcp_config().await
}

/// Handle MCP config init command
pub async fn handle_mcp_config_init(output: Option<PathBuf>, force: bool) -> Result<(), CliError> {
    init_mcp_config(output, force).await
}

/// Handle MCP config validate command
pub async fn handle_mcp_config_validate(config_path: PathBuf) -> Result<(), CliError> {
    validate_mcp_config(config_path).await
}

/// Handle MCP tools list command
pub async fn handle_mcp_tools() -> Result<(), CliError> {
    list_mcp_tools().await
}

/// Handle MCP resources list command
pub async fn handle_mcp_resources() -> Result<(), CliError> {
    list_mcp_resources().await
}

/// Handle MCP server test command
pub async fn handle_mcp_test(endpoint: String) -> Result<(), CliError> {
    test_mcp_server(endpoint).await
}
