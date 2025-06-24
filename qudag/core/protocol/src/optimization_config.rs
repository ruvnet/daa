//! Optimization configuration management
//!
//! This module provides configuration loading and management for all QuDAG optimizations.

use serde::{Deserialize, Serialize};
use std::env;
use std::path::Path;
use thiserror::Error;
use toml;

/// Optimization configuration errors
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Failed to read configuration file: {0}")]
    ReadError(#[from] std::io::Error),

    #[error("Failed to parse configuration: {0}")]
    ParseError(#[from] toml::de::Error),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

/// Main optimization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationConfig {
    /// Global settings
    pub global: GlobalConfig,
    /// Network optimizations
    pub network: NetworkOptimizations,
    /// DAG optimizations
    pub dag: DagOptimizations,
    /// Swarm optimizations
    pub swarm: SwarmOptimizations,
}

/// Global configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    /// Enable all optimizations
    pub enable_optimizations: bool,
}

/// Network optimization settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkOptimizations {
    /// Message chunking configuration
    pub message_chunking: MessageChunkingConfig,
    /// Adaptive batching configuration
    pub adaptive_batching: AdaptiveBatchingConfig,
}

/// Message chunking configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageChunkingConfig {
    pub enabled: bool,
    pub max_chunk_size: usize,
    pub max_chunks: usize,
    pub chunk_timeout: u64,
    pub enable_compression: bool,
    pub compression_threshold: usize,
    pub compression_level: i32,
    pub cache_size: usize,
}

/// Adaptive batching configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveBatchingConfig {
    pub enabled: bool,
    pub max_batch_size: usize,
    pub batch_timeout: u64,
    pub algorithm: String,
}

/// DAG optimization settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DagOptimizations {
    /// Validation cache configuration
    pub validation_cache: ValidationCacheConfig,
    /// Traversal index configuration
    pub traversal_index: TraversalIndexConfig,
}

/// Validation cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationCacheConfig {
    pub enabled: bool,
    pub max_entries: usize,
    pub ttl: u64,
    pub enable_batch_validation: bool,
    pub batch_size: usize,
    pub cache_parent_validation: bool,
}

/// Traversal index configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraversalIndexConfig {
    pub enabled: bool,
    pub common_ancestor_cache_size: usize,
    pub path_cache_size: usize,
    pub enable_graph_algorithms: bool,
}

/// Swarm optimization settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmOptimizations {
    /// Async coordination configuration
    pub async_coordination: AsyncCoordinationConfig,
}

/// Async coordination configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsyncCoordinationConfig {
    pub enabled: bool,
    pub max_agents_per_coordinator: usize,
    pub max_hierarchy_depth: usize,
    pub communication_timeout: u64,
    pub distribution_strategy: String,
    pub enable_work_stealing: bool,
    pub heartbeat_interval: u64,
}

impl OptimizationConfig {
    /// Load configuration from file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let contents = std::fs::read_to_string(path)?;
        let mut config: Self = toml::from_str(&contents)?;

        // Apply environment variable overrides
        config.apply_env_overrides();

        Ok(config)
    }

    /// Load default configuration
    pub fn default() -> Self {
        Self {
            global: GlobalConfig {
                enable_optimizations: true,
            },
            network: NetworkOptimizations {
                message_chunking: MessageChunkingConfig {
                    enabled: true,
                    max_chunk_size: 65536,
                    max_chunks: 10000,
                    chunk_timeout: 30,
                    enable_compression: true,
                    compression_threshold: 1024,
                    compression_level: 3,
                    cache_size: 1000,
                },
                adaptive_batching: AdaptiveBatchingConfig {
                    enabled: true,
                    max_batch_size: 100,
                    batch_timeout: 50,
                    algorithm: "exponential_backoff".to_string(),
                },
            },
            dag: DagOptimizations {
                validation_cache: ValidationCacheConfig {
                    enabled: true,
                    max_entries: 100000,
                    ttl: 3600,
                    enable_batch_validation: true,
                    batch_size: 100,
                    cache_parent_validation: true,
                },
                traversal_index: TraversalIndexConfig {
                    enabled: true,
                    common_ancestor_cache_size: 10000,
                    path_cache_size: 1000,
                    enable_graph_algorithms: true,
                },
            },
            swarm: SwarmOptimizations {
                async_coordination: AsyncCoordinationConfig {
                    enabled: true,
                    max_agents_per_coordinator: 10,
                    max_hierarchy_depth: 3,
                    communication_timeout: 5,
                    distribution_strategy: "load_balanced".to_string(),
                    enable_work_stealing: true,
                    heartbeat_interval: 10,
                },
            },
        }
    }

    /// Apply environment variable overrides
    fn apply_env_overrides(&mut self) {
        // Global
        if let Ok(val) = env::var("QUDAG_ENABLE_OPTIMIZATIONS") {
            self.global.enable_optimizations = val.parse().unwrap_or(true);
        }

        // Network - Message Chunking
        if let Ok(val) = env::var("QUDAG_NETWORK_MESSAGE_CHUNKING_ENABLED") {
            self.network.message_chunking.enabled = val.parse().unwrap_or(true);
        }
        if let Ok(val) = env::var("QUDAG_NETWORK_MESSAGE_CHUNKING_MAX_CHUNK_SIZE") {
            if let Ok(size) = val.parse() {
                self.network.message_chunking.max_chunk_size = size;
            }
        }

        // DAG - Validation Cache
        if let Ok(val) = env::var("QUDAG_DAG_VALIDATION_CACHE_ENABLED") {
            self.dag.validation_cache.enabled = val.parse().unwrap_or(true);
        }
        if let Ok(val) = env::var("QUDAG_DAG_VALIDATION_CACHE_MAX_ENTRIES") {
            if let Ok(entries) = val.parse() {
                self.dag.validation_cache.max_entries = entries;
            }
        }

        // Swarm - Async Coordination
        if let Ok(val) = env::var("QUDAG_SWARM_ASYNC_COORDINATION_ENABLED") {
            self.swarm.async_coordination.enabled = val.parse().unwrap_or(true);
        }
        if let Ok(val) = env::var("QUDAG_SWARM_ASYNC_COORDINATION_MAX_AGENTS") {
            if let Ok(agents) = val.parse() {
                self.swarm.async_coordination.max_agents_per_coordinator = agents;
            }
        }
    }

    /// Check if a specific optimization is enabled
    pub fn is_enabled(&self, optimization: &str) -> bool {
        if !self.global.enable_optimizations {
            return false;
        }

        match optimization {
            "message_chunking" => self.network.message_chunking.enabled,
            "adaptive_batching" => self.network.adaptive_batching.enabled,
            "validation_cache" => self.dag.validation_cache.enabled,
            "traversal_index" => self.dag.traversal_index.enabled,
            "async_coordination" => self.swarm.async_coordination.enabled,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = OptimizationConfig::default();
        assert!(config.global.enable_optimizations);
        assert!(config.network.message_chunking.enabled);
        assert_eq!(config.network.message_chunking.max_chunk_size, 65536);
    }

    #[test]
    fn test_is_enabled() {
        let config = OptimizationConfig::default();
        assert!(config.is_enabled("message_chunking"));
        assert!(config.is_enabled("validation_cache"));
        assert!(!config.is_enabled("unknown_optimization"));
    }
}
