//! Utility functions for the CLI

use anyhow::{Context, Result};
use std::path::PathBuf;

/// Get the default configuration file path
pub fn get_default_config_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .context("Unable to determine config directory")?;
    
    let daa_config_dir = config_dir.join("daa");
    std::fs::create_dir_all(&daa_config_dir)
        .with_context(|| format!("Failed to create config directory: {}", daa_config_dir.display()))?;
    
    Ok(daa_config_dir.join("config.toml"))
}

/// Get the default data directory path
pub fn get_default_data_path() -> Result<PathBuf> {
    let data_dir = dirs::data_dir()
        .context("Unable to determine data directory")?;
    
    let daa_data_dir = data_dir.join("daa");
    std::fs::create_dir_all(&daa_data_dir)
        .with_context(|| format!("Failed to create data directory: {}", daa_data_dir.display()))?;
    
    Ok(daa_data_dir)
}

/// Get the default orchestrator configuration path
pub fn get_default_orchestrator_config_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .context("Unable to determine config directory")?;
    
    let daa_config_dir = config_dir.join("daa");
    std::fs::create_dir_all(&daa_config_dir)
        .with_context(|| format!("Failed to create config directory: {}", daa_config_dir.display()))?;
    
    Ok(daa_config_dir.join("orchestrator.toml"))
}

/// Format duration in human-readable format
pub fn format_duration(duration: std::time::Duration) -> String {
    let total_seconds = duration.as_secs();
    let days = total_seconds / 86400;
    let hours = (total_seconds % 86400) / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    if days > 0 {
        format!("{}d {}h {}m {}s", days, hours, minutes, seconds)
    } else if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    }
}

/// Format bytes in human-readable format
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.2} {}", size, UNITS[unit_index])
    }
}

/// Truncate string to specified length with ellipsis
pub fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

/// Check if a process is running by PID
pub fn is_process_running(pid: u32) -> bool {
    use std::process::Command;
    
    #[cfg(target_os = "windows")]
    {
        Command::new("tasklist")
            .args(&["/FI", &format!("PID eq {}", pid)])
            .output()
            .map(|output| String::from_utf8_lossy(&output.stdout).contains(&pid.to_string()))
            .unwrap_or(false)
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        Command::new("kill")
            .args(&["-0", &pid.to_string()])
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
}

/// Read PID from file
pub fn read_pid_file(path: &std::path::Path) -> Result<u32> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read PID file: {}", path.display()))?;
    
    content.trim().parse::<u32>()
        .with_context(|| format!("Invalid PID in file: {}", path.display()))
}

/// Write PID to file
pub fn write_pid_file(path: &std::path::Path, pid: u32) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }
    
    std::fs::write(path, pid.to_string())
        .with_context(|| format!("Failed to write PID file: {}", path.display()))
}

/// Remove PID file
pub fn remove_pid_file(path: &std::path::Path) -> Result<()> {
    if path.exists() {
        std::fs::remove_file(path)
            .with_context(|| format!("Failed to remove PID file: {}", path.display()))?;
    }
    Ok(())
}

/// Create a spinner for long-running operations
pub fn create_spinner(message: &str) -> indicatif::ProgressBar {
    let pb = indicatif::ProgressBar::new_spinner();
    pb.set_style(
        indicatif::ProgressStyle::default_spinner()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{spinner:.green} {msg}")
            .unwrap()
    );
    pb.set_message(message.to_string());
    pb.enable_steady_tick(std::time::Duration::from_millis(80));
    pb
}

/// Convert JSON value to human-readable table
pub fn json_to_table(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Object(map) => {
            let mut table = String::new();
            for (key, val) in map {
                table.push_str(&format!("{}: {}\n", key, json_value_to_string(val)));
            }
            table
        }
        _ => json_value_to_string(value),
    }
}

fn json_value_to_string(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Null => "null".to_string(),
        serde_json::Value::Array(arr) => {
            format!("[{}]", arr.iter().map(json_value_to_string).collect::<Vec<_>>().join(", "))
        }
        serde_json::Value::Object(_) => serde_json::to_string_pretty(value).unwrap_or_else(|_| "{}".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(std::time::Duration::from_secs(30)), "30s");
        assert_eq!(format_duration(std::time::Duration::from_secs(90)), "1m 30s");
        assert_eq!(format_duration(std::time::Duration::from_secs(3661)), "1h 1m 1s");
        assert_eq!(format_duration(std::time::Duration::from_secs(90061)), "1d 1h 1m 1s");
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1536), "1.50 KB");
        assert_eq!(format_bytes(1048576), "1.00 MB");
    }

    #[test]
    fn test_truncate_string() {
        assert_eq!(truncate_string("hello", 10), "hello");
        assert_eq!(truncate_string("hello world", 8), "hello...");
        assert_eq!(truncate_string("hi", 8), "hi");
    }

    #[test]
    fn test_json_to_table() {
        let json = serde_json::json!({
            "name": "test",
            "count": 42,
            "active": true
        });
        
        let table = json_to_table(&json);
        assert!(table.contains("name: test"));
        assert!(table.contains("count: 42"));
        assert!(table.contains("active: true"));
    }
}