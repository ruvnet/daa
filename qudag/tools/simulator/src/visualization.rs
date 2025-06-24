//! Visualization module for network topology and simulation metrics.

use crate::metrics::{LatencyMetrics, ThroughputMetrics};
use anyhow::Result;
use plotters::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Visualization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationConfig {
    /// Output directory for visualizations
    pub output_dir: String,
    /// Image width in pixels
    pub width: u32,
    /// Image height in pixels
    pub height: u32,
    /// Chart theme
    pub theme: ChartTheme,
    /// Whether to generate interactive charts
    pub interactive: bool,
}

/// Chart theme options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChartTheme {
    /// Light theme with bright background
    Light,
    /// Dark theme with dark background
    Dark,
    /// Colorful theme with vibrant colors
    Colorful,
    /// Monochrome theme with grayscale colors
    Monochrome,
}

/// Network topology for visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkTopology {
    /// Network nodes
    pub nodes: Vec<NetworkNode>,
    /// Network connections
    pub connections: Vec<NetworkConnection>,
    /// Attack information
    pub attacks: Vec<AttackVisualization>,
}

/// Network node for visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkNode {
    /// Node identifier
    pub id: String,
    /// Node position (x, y)
    pub position: (f64, f64),
    /// Node type
    pub node_type: NodeType,
    /// Node status
    pub status: NodeStatus,
    /// Node metrics
    pub metrics: NodeMetrics,
}

/// Network connection for visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConnection {
    /// Source node
    pub from: String,
    /// Destination node
    pub to: String,
    /// Connection strength/quality
    pub strength: f64,
    /// Connection latency
    pub latency: Duration,
    /// Whether connection is active
    pub active: bool,
}

/// Node types for visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeType {
    /// Honest node following protocol correctly
    Honest,
    /// Malicious node with harmful intent
    Malicious,
    /// Byzantine node with arbitrary behavior
    Byzantine,
    /// Sybil node with fake identity
    Sybil,
    /// Offline node not participating
    Offline,
}

/// Node status for visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeStatus {
    /// Node is active and participating
    Active,
    /// Node is inactive but available
    Inactive,
    /// Node has been compromised
    Compromised,
    /// Node is isolated from network
    Isolated,
    /// Node is recovering from failure
    Recovering,
}

/// Node-specific metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetrics {
    /// Messages sent
    pub messages_sent: u64,
    /// Messages received
    pub messages_received: u64,
    /// CPU usage
    pub cpu_usage: f64,
    /// Memory usage
    pub memory_usage: f64,
    /// Network bandwidth usage
    pub bandwidth_usage: f64,
}

/// Attack visualization data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttackVisualization {
    /// Attack identifier
    pub id: String,
    /// Attack type name
    pub attack_type: String,
    /// Affected nodes
    pub affected_nodes: Vec<String>,
    /// Attack start time
    pub start_time: SystemTime,
    /// Attack duration
    pub duration: Duration,
    /// Attack severity (0.0-1.0)
    pub severity: f64,
}

/// Time series data for metrics visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesData {
    /// Timestamp
    pub timestamp: SystemTime,
    /// Data points
    pub values: HashMap<String, f64>,
}

/// Network visualizer
pub struct NetworkVisualizer {
    config: VisualizationConfig,
}

impl NetworkVisualizer {
    /// Create a new network visualizer
    pub fn new(config: VisualizationConfig) -> Self {
        Self { config }
    }

    /// Create with default configuration
    pub fn default() -> Self {
        Self {
            config: VisualizationConfig {
                output_dir: "visualizations".to_string(),
                width: 1200,
                height: 800,
                theme: ChartTheme::Light,
                interactive: false,
            },
        }
    }

    /// Visualize network topology
    pub fn visualize_topology(&self, topology: &NetworkTopology) -> Result<String> {
        let output_path = format!("{}/network_topology.png", self.config.output_dir);

        // Create output directory if it doesn't exist
        std::fs::create_dir_all(&self.config.output_dir)?;

        let output_path_clone = output_path.clone();
        let root = BitMapBackend::new(&output_path_clone, (self.config.width, self.config.height))
            .into_drawing_area();
        root.fill(&WHITE)?;

        let mut chart = ChartBuilder::on(&root)
            .caption("Network Topology", ("sans-serif", 40))
            .margin(10)
            .x_label_area_size(50)
            .y_label_area_size(50)
            .build_cartesian_2d(-10.0f64..10.0f64, -10.0f64..10.0f64)?;

        chart.configure_mesh().draw()?;

        // Draw connections first (so they appear behind nodes)
        for connection in &topology.connections {
            if !connection.active {
                continue;
            }

            let from_node = topology.nodes.iter().find(|n| n.id == connection.from);
            let to_node = topology.nodes.iter().find(|n| n.id == connection.to);

            if let (Some(from), Some(to)) = (from_node, to_node) {
                let color = match connection.strength {
                    s if s > 0.8 => &GREEN,
                    s if s > 0.5 => &YELLOW,
                    _ => &RED,
                };

                chart.draw_series(LineSeries::new(
                    vec![from.position, to.position],
                    color.stroke_width(2),
                ))?;
            }
        }

        // Draw nodes
        for node in &topology.nodes {
            let color = match node.node_type {
                NodeType::Honest => &BLUE,
                NodeType::Malicious => &RED,
                NodeType::Byzantine => &MAGENTA,
                NodeType::Sybil => &BLACK,
                NodeType::Offline => &plotters::style::colors::full_palette::GREY,
            };

            let size = match node.status {
                NodeStatus::Active => 8,
                NodeStatus::Inactive => 4,
                NodeStatus::Compromised => 10,
                NodeStatus::Isolated => 6,
                NodeStatus::Recovering => 7,
            };

            chart.draw_series(PointSeries::of_element(
                vec![node.position],
                size,
                color,
                &|c, s, st| Circle::new(c, s, st.filled()),
            ))?;

            // Add node labels
            chart.draw_series(std::iter::once(Text::new(
                node.id.clone(),
                (node.position.0, node.position.1 + 0.5),
                ("sans-serif", 12),
            )))?;
        }

        root.present()?;
        Ok(output_path)
    }

    /// Visualize network metrics over time
    pub fn visualize_metrics_timeline(&self, metrics_history: &[TimeSeriesData]) -> Result<String> {
        if metrics_history.is_empty() {
            return Err(anyhow::anyhow!("No metrics data provided"));
        }

        let output_path = format!("{}/metrics_timeline.png", self.config.output_dir);
        std::fs::create_dir_all(&self.config.output_dir)?;

        let output_path_clone = output_path.clone();
        let root = BitMapBackend::new(&output_path_clone, (self.config.width, self.config.height))
            .into_drawing_area();
        root.fill(&WHITE)?;

        // Convert timestamps to seconds for plotting
        let start_time = metrics_history[0].timestamp;
        let time_series: Vec<_> = metrics_history
            .iter()
            .map(|data| {
                let elapsed = data
                    .timestamp
                    .duration_since(start_time)
                    .unwrap_or_default();
                elapsed.as_secs_f64()
            })
            .collect();

        let max_time = time_series.last().copied().unwrap_or(0.0);

        let mut chart = ChartBuilder::on(&root)
            .caption("Network Metrics Timeline", ("sans-serif", 40))
            .margin(10)
            .x_label_area_size(50)
            .y_label_area_size(80)
            .build_cartesian_2d(0.0f64..max_time, 0.0f64..100.0f64)?;

        chart
            .configure_mesh()
            .x_desc("Time (seconds)")
            .y_desc("Value")
            .draw()?;

        // Plot different metrics
        let metrics_keys: Vec<_> = metrics_history[0].values.keys().cloned().collect();
        let colors = [&RED, &BLUE, &GREEN, &MAGENTA, &CYAN];

        for (i, metric_key) in metrics_keys.iter().enumerate() {
            let color = colors[i % colors.len()];
            let data_points: Vec<_> = metrics_history
                .iter()
                .zip(time_series.iter())
                .filter_map(|(data, &time)| data.values.get(metric_key).map(|&value| (time, value)))
                .collect();

            chart
                .draw_series(LineSeries::new(data_points, color.stroke_width(2)))?
                .label(metric_key)
                .legend(move |(x, y)| {
                    Rectangle::new([(x - 5, y - 5), (x + 5, y + 5)], color.filled())
                });
        }

        chart.configure_series_labels().draw()?;
        root.present()?;
        Ok(output_path)
    }

    /// Visualize latency distribution
    pub fn visualize_latency_distribution(
        &self,
        latency_metrics: &LatencyMetrics,
    ) -> Result<String> {
        let output_path = format!("{}/latency_distribution.png", self.config.output_dir);
        std::fs::create_dir_all(&self.config.output_dir)?;

        let output_path_clone = output_path.clone();
        let root = BitMapBackend::new(&output_path_clone, (self.config.width, self.config.height))
            .into_drawing_area();
        root.fill(&WHITE)?;

        let latencies = vec![
            ("Average", latency_metrics.avg_latency.as_millis() as f64),
            (
                "95th Percentile",
                latency_metrics.p95_latency.as_millis() as f64,
            ),
            (
                "99th Percentile",
                latency_metrics.p99_latency.as_millis() as f64,
            ),
            ("Maximum", latency_metrics.max_latency.as_millis() as f64),
        ];

        let max_latency = latencies.iter().map(|(_, v)| *v).fold(0.0f64, f64::max);

        let mut chart = ChartBuilder::on(&root)
            .caption("Latency Distribution", ("sans-serif", 40))
            .margin(10)
            .x_label_area_size(50)
            .y_label_area_size(80)
            .build_cartesian_2d(0f64..latencies.len() as f64, 0f64..max_latency * 1.1)?;

        chart
            .configure_mesh()
            .x_desc("Latency Type")
            .y_desc("Latency (ms)")
            .draw()?;

        chart.draw_series(latencies.iter().enumerate().map(|(i, (_name, value))| {
            Rectangle::new(
                [(i as f64 + 0.1, 0.0), (i as f64 + 0.9, *value)],
                BLUE.filled(),
            )
        }))?;

        // Add value labels on bars
        for (i, (name, value)) in latencies.iter().enumerate() {
            chart.draw_series(std::iter::once(Text::new(
                format!("{:.1} ms", value),
                (i as f64 + 0.5, *value + max_latency * 0.02),
                ("sans-serif", 12),
            )))?;

            chart.draw_series(std::iter::once(Text::new(
                name.to_string(),
                (i as f64 + 0.5, -max_latency * 0.05),
                ("sans-serif", 12),
            )))?;
        }

        root.present()?;
        Ok(output_path)
    }

    /// Visualize throughput over time
    pub fn visualize_throughput(
        &self,
        throughput_history: &[(SystemTime, ThroughputMetrics)],
    ) -> Result<String> {
        if throughput_history.is_empty() {
            return Err(anyhow::anyhow!("No throughput data provided"));
        }

        let output_path = format!("{}/throughput.png", self.config.output_dir);
        std::fs::create_dir_all(&self.config.output_dir)?;

        let output_path_clone = output_path.clone();
        let root = BitMapBackend::new(&output_path_clone, (self.config.width, self.config.height))
            .into_drawing_area();
        root.fill(&WHITE)?;

        let start_time = throughput_history[0].0;
        let time_series: Vec<_> = throughput_history
            .iter()
            .map(|(time, metrics)| {
                let elapsed = time.duration_since(start_time).unwrap_or_default();
                (elapsed.as_secs_f64(), metrics.msgs_per_sec)
            })
            .collect();

        let max_time = time_series.last().map(|(t, _)| *t).unwrap_or(0.0);
        let max_throughput = time_series.iter().map(|(_, t)| *t).fold(0.0f64, f64::max);

        let mut chart = ChartBuilder::on(&root)
            .caption("Network Throughput", ("sans-serif", 40))
            .margin(10)
            .x_label_area_size(50)
            .y_label_area_size(80)
            .build_cartesian_2d(0.0f64..max_time, 0.0f64..max_throughput * 1.1)?;

        chart
            .configure_mesh()
            .x_desc("Time (seconds)")
            .y_desc("Messages per Second")
            .draw()?;

        chart.draw_series(LineSeries::new(time_series, BLUE.stroke_width(2)))?;

        root.present()?;
        Ok(output_path)
    }

    /// Visualize attack timeline
    pub fn visualize_attack_timeline(&self, attacks: &[AttackVisualization]) -> Result<String> {
        if attacks.is_empty() {
            return Err(anyhow::anyhow!("No attack data provided"));
        }

        let output_path = format!("{}/attack_timeline.png", self.config.output_dir);
        std::fs::create_dir_all(&self.config.output_dir)?;

        let output_path_clone = output_path.clone();
        let root = BitMapBackend::new(&output_path_clone, (self.config.width, self.config.height))
            .into_drawing_area();
        root.fill(&WHITE)?;

        // Find time range
        let start_time = attacks
            .iter()
            .map(|a| a.start_time)
            .min()
            .unwrap_or_else(|| UNIX_EPOCH);

        let end_time = attacks
            .iter()
            .map(|a| a.start_time + a.duration)
            .max()
            .unwrap_or_else(|| UNIX_EPOCH);

        let total_duration = end_time.duration_since(start_time).unwrap_or_default();

        let mut chart = ChartBuilder::on(&root)
            .caption("Attack Timeline", ("sans-serif", 40))
            .margin(10)
            .x_label_area_size(50)
            .y_label_area_size(80)
            .build_cartesian_2d(
                0.0f64..total_duration.as_secs_f64(),
                0f64..attacks.len() as f64,
            )?;

        chart
            .configure_mesh()
            .x_desc("Time (seconds)")
            .y_desc("Attacks")
            .draw()?;

        // Draw attack bars
        for (i, attack) in attacks.iter().enumerate() {
            let attack_start = attack
                .start_time
                .duration_since(start_time)
                .unwrap_or_default()
                .as_secs_f64();
            let attack_end = attack_start + attack.duration.as_secs_f64();

            let color = match attack.severity {
                s if s > 0.8 => &RED,
                s if s > 0.5 => &YELLOW,
                _ => &GREEN,
            };

            chart.draw_series(std::iter::once(Rectangle::new(
                [(attack_start, i as f64 + 0.1), (attack_end, i as f64 + 0.9)],
                color.filled(),
            )))?;

            // Add attack type label
            chart.draw_series(std::iter::once(Text::new(
                attack.attack_type.clone(),
                (
                    attack_start + (attack_end - attack_start) / 2.0,
                    i as f64 + 0.5,
                ),
                ("sans-serif", 10),
            )))?;
        }

        root.present()?;
        Ok(output_path)
    }

    /// Generate comprehensive dashboard
    pub fn generate_dashboard(
        &self,
        topology: &NetworkTopology,
        metrics_history: &[TimeSeriesData],
        latency_metrics: &LatencyMetrics,
        throughput_history: &[(SystemTime, ThroughputMetrics)],
        attacks: &[AttackVisualization],
    ) -> Result<Vec<String>> {
        let mut generated_files = Vec::new();

        // Generate all visualizations
        if let Ok(file) = self.visualize_topology(topology) {
            generated_files.push(file);
        }

        if !metrics_history.is_empty() {
            if let Ok(file) = self.visualize_metrics_timeline(metrics_history) {
                generated_files.push(file);
            }
        }

        if let Ok(file) = self.visualize_latency_distribution(latency_metrics) {
            generated_files.push(file);
        }

        if !throughput_history.is_empty() {
            if let Ok(file) = self.visualize_throughput(throughput_history) {
                generated_files.push(file);
            }
        }

        if !attacks.is_empty() {
            if let Ok(file) = self.visualize_attack_timeline(attacks) {
                generated_files.push(file);
            }
        }

        // Generate HTML dashboard
        let dashboard_html = self.generate_html_dashboard(&generated_files)?;
        let dashboard_path = format!("{}/dashboard.html", self.config.output_dir);
        std::fs::write(&dashboard_path, dashboard_html)?;
        generated_files.push(dashboard_path);

        Ok(generated_files)
    }

    /// Generate HTML dashboard
    fn generate_html_dashboard(&self, image_files: &[String]) -> Result<String> {
        let mut html = format!(
            r#"
<!DOCTYPE html>
<html>
<head>
    <title>QuDAG Network Simulation Dashboard</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; }}
        .header {{ background-color: #f0f0f0; padding: 20px; margin-bottom: 20px; }}
        .visualization {{ margin: 20px 0; text-align: center; }}
        .visualization img {{ max-width: 100%; height: auto; border: 1px solid #ddd; }}
        .stats {{ display: grid; grid-template-columns: repeat(3, 1fr); gap: 20px; margin: 20px 0; }}
        .stat-box {{ background-color: #f9f9f9; padding: 15px; border-radius: 5px; }}
    </style>
</head>
<body>
    <div class="header">
        <h1>QuDAG Network Simulation Dashboard</h1>
        <p>Generated on: {}</p>
    </div>
"#,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        );

        for image_file in image_files {
            if let Some(filename) = Path::new(image_file).file_name() {
                if let Some(name) = filename.to_str() {
                    if name.ends_with(".png") {
                        html.push_str(&format!(
                            r#"
    <div class="visualization">
        <h2>{}</h2>
        <img src="{}" alt="{}">
    </div>
"#,
                            name.replace("_", " ").replace(".png", ""),
                            name,
                            name
                        ));
                    }
                }
            }
        }

        html.push_str(
            r#"
</body>
</html>
"#,
        );

        Ok(html)
    }
}

impl Default for VisualizationConfig {
    fn default() -> Self {
        Self {
            output_dir: "visualizations".to_string(),
            width: 1200,
            height: 800,
            theme: ChartTheme::Light,
            interactive: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, UNIX_EPOCH};

    #[test]
    fn test_visualization_config_creation() {
        let config = VisualizationConfig::default();
        assert_eq!(config.width, 1200);
        assert_eq!(config.height, 800);
        assert_eq!(config.output_dir, "visualizations");
    }

    #[test]
    fn test_network_topology_creation() {
        let topology = NetworkTopology {
            nodes: vec![NetworkNode {
                id: "node1".to_string(),
                position: (0.0, 0.0),
                node_type: NodeType::Honest,
                status: NodeStatus::Active,
                metrics: NodeMetrics {
                    messages_sent: 100,
                    messages_received: 95,
                    cpu_usage: 0.5,
                    memory_usage: 0.3,
                    bandwidth_usage: 0.7,
                },
            }],
            connections: vec![],
            attacks: vec![],
        };

        assert_eq!(topology.nodes.len(), 1);
        assert_eq!(topology.nodes[0].id, "node1");
    }

    #[test]
    fn test_attack_visualization_creation() {
        let attack = AttackVisualization {
            id: "attack1".to_string(),
            attack_type: "DoS".to_string(),
            affected_nodes: vec!["node1".to_string()],
            start_time: UNIX_EPOCH,
            duration: Duration::from_secs(10),
            severity: 0.8,
        };

        assert_eq!(attack.id, "attack1");
        assert_eq!(attack.attack_type, "DoS");
        assert_eq!(attack.severity, 0.8);
    }
}
