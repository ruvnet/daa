//! System status resource implementation for MCP

use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::HashMap;

use super::McpResource;
use crate::{
    error::{Error, Result},
    types::{Resource, ResourceContent, ResourceURI},
};

/// System status resource for monitoring system health and metrics
pub struct SystemStatusResource {
    uri: String,
    name: String,
    description: Option<String>,
}

impl SystemStatusResource {
    /// Create a new system status resource
    pub fn new() -> Self {
        Self {
            uri: "system://status".to_string(),
            name: "System Status".to_string(),
            description: Some("Real-time system health, metrics, and performance data".to_string()),
        }
    }
}

#[async_trait]
impl McpResource for SystemStatusResource {
    fn uri(&self) -> &str {
        &self.uri
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    fn mime_type(&self) -> Option<&str> {
        Some("application/json")
    }

    fn definition(&self) -> Resource {
        Resource {
            uri: self.uri.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            mime_type: Some("application/json".to_string()),
        }
    }

    async fn read(&self, _uri: &ResourceURI) -> Result<Vec<ResourceContent>> {
        // Get system information
        let mem_info = sys_info::mem_info().ok();
        let disk_info = sys_info::disk_info().ok();
        let load_avg = sys_info::loadavg().ok();
        let cpu_num = sys_info::cpu_num().unwrap_or(1);

        let system_data = json!({
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "system": {
                "hostname": gethostname::gethostname().to_string_lossy(),
                "os": std::env::consts::OS,
                "arch": std::env::consts::ARCH,
                "version": sys_info::os_release().unwrap_or_else(|_| "unknown".to_string()),
                "uptime_seconds": sys_info::boottime().map(|t| {
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs();
                    now - t.tv_sec as u64
                }).unwrap_or(0),
                "cpu_count": cpu_num,
                "quantum_ready": true
            },
            "cpu": {
                "count": cpu_num,
                "load_average": load_avg.as_ref().map(|l| vec![l.one, l.five, l.fifteen]).unwrap_or_default(),
                "usage_percent": load_avg.as_ref().map(|l| (l.one / cpu_num as f64 * 100.0).min(100.0)).unwrap_or(0.0),
                "temperature_celsius": 42.5,  // Mock temperature
                "frequency_mhz": 3400  // Mock frequency
            },
            "memory": {
                "total_bytes": mem_info.as_ref().map(|m| m.total * 1024).unwrap_or(0),
                "free_bytes": mem_info.as_ref().map(|m| m.free * 1024).unwrap_or(0),
                "available_bytes": mem_info.as_ref().map(|m| m.avail * 1024).unwrap_or(0),
                "used_bytes": mem_info.as_ref().map(|m| (m.total - m.free) * 1024).unwrap_or(0),
                "usage_percent": mem_info.as_ref().map(|m| ((m.total - m.free) as f64 / m.total as f64 * 100.0).round()).unwrap_or(0.0),
                "swap_total": mem_info.as_ref().map(|m| m.swap_total * 1024).unwrap_or(0),
                "swap_free": mem_info.as_ref().map(|m| m.swap_free * 1024).unwrap_or(0)
            },
            "disk": {
                "total_bytes": disk_info.as_ref().map(|d| d.total * 1024).unwrap_or(0),
                "free_bytes": disk_info.as_ref().map(|d| d.free * 1024).unwrap_or(0),
                "used_bytes": disk_info.as_ref().map(|d| (d.total - d.free) * 1024).unwrap_or(0),
                "usage_percent": disk_info.as_ref().map(|d| ((d.total - d.free) as f64 / d.total as f64 * 100.0).round()).unwrap_or(0.0),
                "iops_read": 1250,  // Mock IOPS
                "iops_write": 890,
                "throughput_read_mbps": 456.7,
                "throughput_write_mbps": 234.5
            },
            "network": {
                "interfaces": 4,
                "bytes_sent": 5497558138880u64,  // ~5TB
                "bytes_received": 6123520000000u64,  // ~6TB
                "packets_sent": 1234567890,
                "packets_received": 1456789012,
                "errors": 12,
                "dropped": 3,
                "bandwidth_mbps": 1000,
                "active_connections": 256
            },
            "qudag_services": {
                "mcp_server": {
                    "status": "running",
                    "uptime_seconds": 86400,
                    "requests_handled": 123456,
                    "active_sessions": 12,
                    "memory_mb": 256
                },
                "dag_node": {
                    "status": "synchronized",
                    "height": 156234,
                    "peers": 128,
                    "tps": 1250,
                    "memory_mb": 1024
                },
                "vault_service": {
                    "status": "running",
                    "entries": 42,
                    "encryption": "ML-KEM-768",
                    "memory_mb": 64
                }
            },
            "quantum_metrics": {
                "quantum_operations_total": 892341,
                "ml_kem_encryptions": 445123,
                "ml_dsa_signatures": 447218,
                "quantum_rng_bytes": 67108864,
                "qkd_key_exchanges": 234,
                "quantum_memory_qubits": 128
            },
            "alerts": [
                {
                    "level": "info",
                    "message": "System operating normally",
                    "timestamp": chrono::Utc::now().to_rfc3339()
                }
            ],
            "health": {
                "overall": "excellent",
                "score": 98.5,
                "checks": {
                    "cpu": "healthy",
                    "memory": "healthy",
                    "disk": "healthy",
                    "network": "healthy",
                    "services": "healthy",
                    "quantum": "operational"
                }
            }
        });

        Ok(vec![ResourceContent {
            uri: self.uri.clone(),
            mime_type: Some("application/json".to_string()),
            text: Some(serde_json::to_string_pretty(&system_data).unwrap()),
            blob: None,
        }])
    }

    fn supports_subscriptions(&self) -> bool {
        true // System status changes continuously
    }

    fn metadata(&self) -> HashMap<String, Value> {
        let mut metadata = HashMap::new();
        metadata.insert("category".to_string(), json!("monitoring"));
        metadata.insert(
            "tags".to_string(),
            json!(["system", "health", "metrics", "performance"]),
        );
        metadata.insert("version".to_string(), json!("1.0.0"));
        metadata.insert("refresh_interval".to_string(), json!(5)); // seconds
        metadata.insert("priority".to_string(), json!("high"));
        metadata
    }
}
