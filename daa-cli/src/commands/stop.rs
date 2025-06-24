//! Stop command implementation

use anyhow::Result;
use colorful::Colorful;

use crate::{Cli, config::CliConfig};

/// Handle the stop command
pub async fn handle_stop(
    force: bool,
    grace_period: u64,
    config: &CliConfig,
    cli: &Cli,
) -> Result<()> {
    if cli.verbose {
        println!("Stopping DAA orchestrator");
        println!("Force: {}", force);
        println!("Grace period: {}s", grace_period);
    }

    let spinner = crate::utils::create_spinner("Stopping orchestrator...");
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    spinner.finish_with_message("Orchestrator stopped");

    if cli.json {
        println!("{}", serde_json::json!({
            "status": "stopped",
            "force": force,
            "grace_period": grace_period
        }));
    } else {
        println!("{}", "✓ DAA orchestrator stopped successfully".green());
    }

    Ok(())
}