use serde::Serialize;
use std::io;

/// Output format
#[derive(Debug, Clone, Copy)]
pub enum OutputFormat {
    /// Plain text output
    Text,
    /// JSON output
    Json,
    /// Table output
    Table,
}

/// Output formatter
pub struct Formatter {
    format: OutputFormat,
}

impl Formatter {
    /// Create new formatter
    pub fn new(format: OutputFormat) -> Self {
        Self { format }
    }

    /// Write formatted output
    pub fn write<T: Serialize>(&self, value: &T) -> io::Result<()> {
        match self.format {
            OutputFormat::Text => {
                println!("{}", serde_json::to_string_pretty(value)?);
            }
            OutputFormat::Json => {
                println!("{}", serde_json::to_string(value)?);
            }
            OutputFormat::Table => {
                // TODO: Implement table formatting
                println!("{}", serde_json::to_string_pretty(value)?);
            }
        }
        Ok(())
    }
}

/// Status output
#[derive(Debug, Serialize)]
pub struct StatusOutput {
    /// Node ID
    pub node_id: String,
    /// Node version
    pub version: String,
    /// Uptime in seconds
    pub uptime: u64,
    /// Connected peers
    pub peers: Vec<PeerOutput>,
    /// DAG statistics
    pub dag_stats: DagStats,
}

/// Peer information
#[derive(Debug, Serialize)]
pub struct PeerOutput {
    /// Peer ID
    pub id: String,
    /// Peer address
    pub address: String,
    /// Connection duration
    pub connected: u64,
}

/// DAG statistics
#[derive(Debug, Serialize)]
pub struct DagStats {
    /// Total vertices
    pub vertex_count: usize,
    /// Number of tips
    pub tip_count: usize,
    /// Finalized height
    pub finalized_height: u64,
}
