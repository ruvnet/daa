//! Command implementations for the DAA CLI

use anyhow::Result;
use std::path::PathBuf;

use crate::{Cli, ConfigAction, NetworkAction, AgentAction};
use crate::config::CliConfig;

pub mod init;
pub mod start;
pub mod status;
pub mod stop;
pub mod rules;
pub mod config;
pub mod network;
pub mod agent;
pub mod logs;

// Re-export command handlers
pub use init::handle_init;
pub use start::handle_start;
pub use status::handle_status;
pub use stop::handle_stop;
pub use rules::handle_add_rule;
pub use config::handle_config;
pub use network::handle_network;
pub use agent::handle_agent;
pub use logs::handle_logs;