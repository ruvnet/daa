//! Shared health monitoring structures for QuDAG services

use serde::{Deserialize, Serialize};

/// Standard health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResponse {
    /// Service status: "healthy", "degraded", or "unhealthy"
    pub status: String,
    /// Service version
    pub version: String,
    /// Current timestamp (Unix epoch seconds)
    pub timestamp: u64,
    /// Additional service-specific details
    pub details: HealthDetails,
}

/// Service-specific health details
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum HealthDetails {
    /// QuDAG node health details
    Node {
        p2p_status: String,
        dag_status: String,
        peer_count: usize,
        vertex_count: usize,
    },
    /// QuDAG Exchange health details
    Exchange {
        api_status: String,
        database_status: String,
        pending_transactions: usize,
    },
    /// Generic service health details
    Generic {
        service_name: String,
        additional_info: serde_json::Value,
    },
}

/// Standard metrics response (Prometheus format)
pub struct MetricsResponse {
    pub metrics: Vec<Metric>,
}

#[derive(Debug, Clone)]
pub struct Metric {
    pub name: String,
    pub help: String,
    pub metric_type: MetricType,
    pub value: f64,
    pub labels: Vec<(String, String)>,
}

#[derive(Debug, Clone)]
pub enum MetricType {
    Counter,
    Gauge,
    Histogram,
    Summary,
}

impl MetricsResponse {
    /// Format metrics in Prometheus text format
    pub fn to_prometheus_format(&self) -> String {
        let mut output = String::new();
        
        for metric in &self.metrics {
            // Add HELP line
            output.push_str(&format!("# HELP {} {}\n", metric.name, metric.help));
            
            // Add TYPE line
            let type_str = match metric.metric_type {
                MetricType::Counter => "counter",
                MetricType::Gauge => "gauge",
                MetricType::Histogram => "histogram",
                MetricType::Summary => "summary",
            };
            output.push_str(&format!("# TYPE {} {}\n", metric.name, type_str));
            
            // Add metric value with labels
            if metric.labels.is_empty() {
                output.push_str(&format!("{} {}\n", metric.name, metric.value));
            } else {
                let labels = metric.labels
                    .iter()
                    .map(|(k, v)| format!("{}=\"{}\"", k, v))
                    .collect::<Vec<_>>()
                    .join(",");
                output.push_str(&format!("{}{{{}}} {}\n", metric.name, labels, metric.value));
            }
            output.push('\n');
        }
        
        output
    }
}

/// Helper to get current Unix timestamp
pub fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// Helper to determine service health status based on conditions
pub fn determine_health_status(
    critical_checks: &[bool],
    warning_checks: &[bool],
) -> String {
    if critical_checks.iter().any(|&check| !check) {
        "unhealthy".to_string()
    } else if warning_checks.iter().any(|&check| !check) {
        "degraded".to_string()
    } else {
        "healthy".to_string()
    }
}