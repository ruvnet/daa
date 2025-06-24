//! Status command implementation

use anyhow::Result;
use colorful::Colorful;

use crate::{Cli, config::CliConfig};

/// Handle the status command
pub async fn handle_status(
    detailed: bool,
    watch: bool,
    interval: u64,
    config: &CliConfig,
    cli: &Cli,
) -> Result<()> {
    if watch {
        return handle_watch_status(detailed, interval, config, cli).await;
    }

    let status = get_orchestrator_status(config).await?;
    
    if cli.json {
        println!("{}", serde_json::to_string_pretty(&status)?);
    } else {
        display_status(&status, detailed);
    }

    Ok(())
}

async fn handle_watch_status(
    detailed: bool,
    interval: u64,
    config: &CliConfig,
    cli: &Cli,
) -> Result<()> {
    println!("Watching DAA status (press Ctrl+C to exit)...");
    
    loop {
        let status = get_orchestrator_status(config).await?;
        
        // Clear screen
        print!("\x1B[2J\x1B[1;1H");
        
        if cli.json {
            println!("{}", serde_json::to_string_pretty(&status)?);
        } else {
            display_status(&status, detailed);
        }
        
        tokio::time::sleep(std::time::Duration::from_secs(interval)).await;
    }
}

async fn get_orchestrator_status(config: &CliConfig) -> Result<OrchestratorStatus> {
    // Mock status - in real implementation, this would query the orchestrator
    Ok(OrchestratorStatus {
        name: "daa-orchestrator".to_string(),
        state: "Running".to_string(),
        uptime: "2h 15m 30s".to_string(),
        autonomy_status: "Active".to_string(),
        qudag_status: "Connected".to_string(),
        mcp_enabled: true,
        mcp_port: 3001,
        api_enabled: true,
        api_port: 3000,
        agents_count: 3,
        active_rules: 5,
        network_peers: 4,
    })
}

fn display_status(status: &OrchestratorStatus, detailed: bool) {
    println!("{}", "DAA Orchestrator Status".blue().bold());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    let state_color = match status.state.as_str() {
        "Running" => status.state.green(),
        "Starting" => status.state.yellow(),
        "Stopping" => status.state.yellow(),
        "Stopped" => status.state.red(),
        _ => status.state.white(),
    };
    
    println!("Name:     {}", status.name);
    println!("State:    {}", state_color);
    println!("Uptime:   {}", status.uptime);
    
    if detailed {
        println!();
        println!("{}", "Components".blue().bold());
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("Autonomy Loop:   {}", status.autonomy_status.green());
        println!("QuDAG Network:   {}", status.qudag_status.green());
        
        if status.mcp_enabled {
            println!("MCP Server:      {} (port {})", "Enabled".green(), status.mcp_port);
        } else {
            println!("MCP Server:      {}", "Disabled".red());
        }
        
        if status.api_enabled {
            println!("API Server:      {} (port {})", "Enabled".green(), status.api_port);
        } else {
            println!("API Server:      {}", "Disabled".red());
        }
        
        println!();
        println!("{}", "Statistics".blue().bold());
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("Active Agents:   {}", status.agents_count);
        println!("Active Rules:    {}", status.active_rules);
        println!("Network Peers:   {}", status.network_peers);
    }
}

#[derive(serde::Serialize)]
struct OrchestratorStatus {
    name: String,
    state: String,
    uptime: String,
    autonomy_status: String,
    qudag_status: String,
    mcp_enabled: bool,
    mcp_port: u16,
    api_enabled: bool,
    api_port: u16,
    agents_count: u32,
    active_rules: u32,
    network_peers: u32,
}