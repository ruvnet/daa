//! System information and monitoring tool for MCP

use async_trait::async_trait;
use serde_json::{json, Value};
use std::env;

use super::{get_optional_bool_arg, get_required_string_arg, McpTool};
use crate::error::{Error, Result};

/// System tool for system information and monitoring
pub struct SystemTool {
    name: String,
    description: String,
}

impl SystemTool {
    /// Create a new system tool
    pub fn new() -> Self {
        Self {
            name: "system".to_string(),
            description: "System information, monitoring, and management operations including CPU, memory, disk, network stats, and process management.".to_string(),
        }
    }

    /// Get system information
    async fn get_info(&self, args: &Value) -> Result<Value> {
        let detailed = get_optional_bool_arg(args, "detailed").unwrap_or(false);

        // Get basic system info
        let info = json!({
            "success": true,
            "hostname": gethostname::gethostname().to_string_lossy(),
            "os": env::consts::OS,
            "arch": env::consts::ARCH,
            "family": env::consts::FAMILY,
            "version": sys_info::os_release().unwrap_or_else(|_| "unknown".to_string()),
            "uptime": sys_info::boottime().map(|t| {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                now - t.tv_sec as u64
            }).unwrap_or(0),
            "cpuCount": sys_info::cpu_num().unwrap_or(0),
            "timestamp": chrono::Utc::now().to_rfc3339()
        });

        if detailed {
            // Add more detailed information
            Ok(json!({
                "basic": info,
                "cpu": {
                    "model": sys_info::cpu_speed().unwrap_or(0),
                    "cores": sys_info::cpu_num().unwrap_or(0),
                    "loadAverage": sys_info::loadavg().map(|l| vec![l.one, l.five, l.fifteen]).unwrap_or_default()
                },
                "memory": self.get_memory_stats(),
                "disk": self.get_disk_stats(),
                "network": {
                    "hostname": sys_info::hostname().unwrap_or_else(|_| "unknown".to_string()),
                }
            }))
        } else {
            Ok(info)
        }
    }

    /// Get memory statistics
    fn get_memory_stats(&self) -> Value {
        if let Ok(mem) = sys_info::mem_info() {
            json!({
                "total": mem.total * 1024,  // Convert to bytes
                "free": mem.free * 1024,
                "available": mem.avail * 1024,
                "used": (mem.total - mem.free) * 1024,
                "usagePercent": ((mem.total - mem.free) as f64 / mem.total as f64 * 100.0).round()
            })
        } else {
            json!({
                "error": "Unable to get memory info"
            })
        }
    }

    /// Get disk statistics
    fn get_disk_stats(&self) -> Value {
        if let Ok(disk) = sys_info::disk_info() {
            json!({
                "total": disk.total * 1024,  // Convert to bytes
                "free": disk.free * 1024,
                "used": (disk.total - disk.free) * 1024,
                "usagePercent": ((disk.total - disk.free) as f64 / disk.total as f64 * 100.0).round()
            })
        } else {
            json!({
                "error": "Unable to get disk info"
            })
        }
    }

    /// Get process list
    async fn list_processes(&self, args: &Value) -> Result<Value> {
        let filter = get_optional_bool_arg(args, "filter").unwrap_or(false);

        // Mock implementation - in real implementation would use sysinfo crate
        let processes = vec![
            json!({
                "pid": 1234,
                "name": "qudag",
                "cpu": 2.5,
                "memory": 150 * 1024 * 1024,  // 150MB
                "status": "running"
            }),
            json!({
                "pid": 5678,
                "name": "qudag-mcp",
                "cpu": 0.5,
                "memory": 50 * 1024 * 1024,  // 50MB
                "status": "running"
            }),
        ];

        let filtered_processes = if filter {
            processes
                .iter()
                .filter(|p| p["name"].as_str().unwrap_or("").contains("qudag"))
                .cloned()
                .collect()
        } else {
            processes.clone()
        };

        let count = filtered_processes.len();

        Ok(json!({
            "success": true,
            "processes": filtered_processes,
            "count": count,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
    }

    /// Get environment variables
    async fn get_env(&self, args: &Value) -> Result<Value> {
        let key = get_optional_string_arg(args, "key");
        let pattern = get_optional_string_arg(args, "pattern");

        if let Some(key) = key {
            // Get specific environment variable
            match env::var(&key) {
                Ok(value) => Ok(json!({
                    "success": true,
                    "key": key,
                    "value": value,
                    "exists": true
                })),
                Err(_) => Ok(json!({
                    "success": true,
                    "key": key,
                    "exists": false
                })),
            }
        } else if let Some(pattern) = pattern {
            // Get environment variables matching pattern
            let vars: Vec<(String, String)> =
                env::vars().filter(|(k, _)| k.contains(&pattern)).collect();

            let count = vars.len();
            let var_list = vars
                .into_iter()
                .map(|(k, v)| {
                    json!({
                        "key": k,
                        "value": v
                    })
                })
                .collect::<Vec<_>>();

            Ok(json!({
                "success": true,
                "pattern": pattern,
                "variables": var_list,
                "count": count
            }))
        } else {
            // Get all environment variables (limited for security)
            let safe_keys = ["PATH", "HOME", "USER", "SHELL", "LANG", "PWD", "QUDAG_HOME"];
            let vars: Vec<_> = safe_keys
                .iter()
                .filter_map(|&k| env::var(k).ok().map(|v| (k.to_string(), v)))
                .collect();

            Ok(json!({
                "success": true,
                "variables": vars.into_iter().map(|(k, v)| json!({
                    "key": k,
                    "value": v
                })).collect::<Vec<_>>(),
                "note": "Limited to safe environment variables"
            }))
        }
    }

    /// Execute system command (limited for security)
    async fn execute_command(&self, args: &Value) -> Result<Value> {
        let command = get_required_string_arg(args, "command")?;
        let allowed_commands = ["uptime", "date", "whoami", "pwd", "hostname"];

        if !allowed_commands.contains(&command.as_str()) {
            return Err(Error::invalid_params(format!(
                "Command '{}' not allowed. Allowed commands: {:?}",
                command, allowed_commands
            )));
        }

        // Mock implementation
        let current_date = chrono::Utc::now().to_rfc3339();
        let output = match command.as_str() {
            "uptime" => "up 5 days, 3:45, 2 users, load average: 0.15, 0.20, 0.18",
            "date" => current_date.as_str(),
            "whoami" => "qudag-user",
            "pwd" => "/home/qudag",
            "hostname" => "qudag-node-01",
            _ => "Unknown command",
        };

        Ok(json!({
            "success": true,
            "command": command,
            "output": output,
            "exitCode": 0,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
    }
}

use super::get_optional_string_arg;

#[async_trait]
impl McpTool for SystemTool {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "operation": {
                    "type": "string",
                    "description": "The system operation to perform",
                    "enum": ["info", "processes", "env", "command"]
                },
                "detailed": {
                    "type": "boolean",
                    "description": "Return detailed information"
                },
                "filter": {
                    "type": "boolean",
                    "description": "Filter processes to QuDAG-related only"
                },
                "key": {
                    "type": "string",
                    "description": "Environment variable key to retrieve"
                },
                "pattern": {
                    "type": "string",
                    "description": "Pattern to filter environment variables"
                },
                "command": {
                    "type": "string",
                    "description": "System command to execute",
                    "enum": ["uptime", "date", "whoami", "pwd", "hostname"]
                }
            },
            "required": ["operation"]
        })
    }

    async fn execute(&self, arguments: Option<Value>) -> Result<Value> {
        let args = arguments
            .as_ref()
            .ok_or_else(|| Error::invalid_params("Arguments required"))?;
        let operation = get_required_string_arg(args, "operation")?;

        match operation.as_str() {
            "info" => self.get_info(args).await,
            "processes" => self.list_processes(args).await,
            "env" => self.get_env(args).await,
            "command" => self.execute_command(args).await,
            _ => Err(Error::invalid_params(format!(
                "Unknown operation: {}",
                operation
            ))),
        }
    }
}
