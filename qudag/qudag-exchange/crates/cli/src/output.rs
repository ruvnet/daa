//! Output formatting utilities

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

/// Output format options
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Serialize, Deserialize)]
pub enum OutputFormat {
    /// Human-readable text output
    Text,
    
    /// JSON output for programmatic use
    Json,
}