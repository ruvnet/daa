//! Logs command implementation

use anyhow::Result;
use colorful::Colorful;

use crate::{Cli, config::CliConfig};

/// Handle the logs command
pub async fn handle_logs(
    lines: usize,
    follow: bool,
    level: Option<String>,
    component: Option<String>,
    config: &CliConfig,
    cli: &Cli,
) -> Result<()> {
    if cli.verbose {
        println!("Showing logs: {} lines", lines);
        println!("Follow: {}", follow);
        println!("Level filter: {:?}", level);
        println!("Component filter: {:?}", component);
    }

    if follow {
        return handle_follow_logs(lines, level, component, config, cli).await;
    }

    let logs = get_logs(lines, level, component).await?;
    
    if cli.json {
        println!("{}", serde_json::json!({ "logs": logs }));
    } else {
        display_logs(&logs);
    }

    Ok(())
}

async fn handle_follow_logs(
    lines: usize,
    level: Option<String>,
    component: Option<String>,
    config: &CliConfig,
    cli: &Cli,
) -> Result<()> {
    println!("Following logs (press Ctrl+C to exit)...");
    
    // Show initial logs
    let initial_logs = get_logs(lines, level.clone(), component.clone()).await?;
    display_logs(&initial_logs);
    
    // Mock follow functionality
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        
        // Mock new log entry
        let new_log = LogEntry {
            timestamp: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            level: "INFO".to_string(),
            component: "orchestrator".to_string(),
            message: "Autonomy loop iteration completed".to_string(),
        };
        
        if should_include_log(&new_log, &level, &component) {
            if cli.json {
                println!("{}", serde_json::to_string(&new_log)?);
            } else {
                display_log_entry(&new_log);
            }
        }
    }
}

async fn get_logs(
    lines: usize,
    level: Option<String>,
    component: Option<String>,
) -> Result<Vec<LogEntry>> {
    // Mock log entries
    let mut logs = vec![
        LogEntry {
            timestamp: "2024-06-24 10:30:15".to_string(),
            level: "INFO".to_string(),
            component: "orchestrator".to_string(),
            message: "DAA Orchestrator started successfully".to_string(),
        },
        LogEntry {
            timestamp: "2024-06-24 10:30:16".to_string(),
            level: "INFO".to_string(),
            component: "qudag".to_string(),
            message: "Connected to QuDAG network".to_string(),
        },
        LogEntry {
            timestamp: "2024-06-24 10:30:17".to_string(),
            level: "INFO".to_string(),
            component: "mcp".to_string(),
            message: "MCP server started on port 3001".to_string(),
        },
        LogEntry {
            timestamp: "2024-06-24 10:30:18".to_string(),
            level: "WARN".to_string(),
            component: "autonomy".to_string(),
            message: "No tasks in queue, entering idle state".to_string(),
        },
        LogEntry {
            timestamp: "2024-06-24 10:30:20".to_string(),
            level: "ERROR".to_string(),
            component: "rules".to_string(),
            message: "Rule evaluation failed: insufficient data".to_string(),
        },
    ];

    // Filter by level
    if let Some(ref filter_level) = level {
        logs.retain(|log| log.level.to_lowercase() == filter_level.to_lowercase());
    }

    // Filter by component
    if let Some(ref filter_component) = component {
        logs.retain(|log| log.component.to_lowercase() == filter_component.to_lowercase());
    }

    // Take only requested number of lines
    logs.truncate(lines);

    Ok(logs)
}

fn should_include_log(log: &LogEntry, level: &Option<String>, component: &Option<String>) -> bool {
    if let Some(ref filter_level) = level {
        if log.level.to_lowercase() != filter_level.to_lowercase() {
            return false;
        }
    }

    if let Some(ref filter_component) = component {
        if log.component.to_lowercase() != filter_component.to_lowercase() {
            return false;
        }
    }

    true
}

fn display_logs(logs: &[LogEntry]) {
    for log in logs {
        display_log_entry(log);
    }
}

fn display_log_entry(log: &LogEntry) {
    let level_color = match log.level.as_str() {
        "ERROR" => log.level.red(),
        "WARN" => log.level.yellow(),
        "INFO" => log.level.green(),
        "DEBUG" => log.level.blue(),
        _ => log.level.white(),
    };

    println!("{} [{}] {}: {}", 
             log.timestamp, 
             level_color, 
             log.component.cyan(), 
             log.message);
}

#[derive(serde::Serialize, serde::Deserialize)]
struct LogEntry {
    timestamp: String,
    level: String,
    component: String,
    message: String,
}