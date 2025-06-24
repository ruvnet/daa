use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::OnceCell;
use tracing::{debug, info, warn};

use crate::config::NodeConfig;
use crate::performance::PerformanceTracker;

/// Lazy-initialized CLI resources
pub struct CliResources {
    pub config: Arc<NodeConfig>,
    pub performance_tracker: Arc<PerformanceTracker>,
}

/// Global CLI resources singleton
static CLI_RESOURCES: OnceCell<CliResources> = OnceCell::const_new();

/// Fast startup optimization
pub struct StartupOptimizer {
    startup_time: Instant,
}

impl Default for StartupOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

impl StartupOptimizer {
    /// Create new startup optimizer
    pub fn new() -> Self {
        Self {
            startup_time: Instant::now(),
        }
    }

    /// Initialize CLI with optimized startup
    pub async fn initialize(&self) -> Result<Arc<CliResources>, StartupError> {
        let init_start = Instant::now();

        // Use OnceCell for singleton initialization
        let resources = CLI_RESOURCES
            .get_or_init(|| async { self.initialize_resources().await })
            .await;

        let init_duration = init_start.elapsed();
        if init_duration > Duration::from_millis(100) {
            warn!(
                "CLI initialization took {:.2}ms",
                init_duration.as_secs_f64() * 1000.0
            );
        } else {
            debug!(
                "CLI initialized in {:.2}ms",
                init_duration.as_secs_f64() * 1000.0
            );
        }

        Ok(Arc::new(CliResources {
            config: resources.config.clone(),
            performance_tracker: resources.performance_tracker.clone(),
        }))
    }

    /// Initialize resources with lazy loading
    async fn initialize_resources(&self) -> CliResources {
        let config_start = Instant::now();

        // Initialize config with defaults (fast path)
        let config = Arc::new(NodeConfig::default());

        debug!(
            "Config loaded in {:.2}ms",
            config_start.elapsed().as_secs_f64() * 1000.0
        );

        // Initialize performance tracker
        let perf_start = Instant::now();
        let performance_tracker = Arc::new(PerformanceTracker::new());

        debug!(
            "Performance tracker initialized in {:.2}ms",
            perf_start.elapsed().as_secs_f64() * 1000.0
        );

        CliResources {
            config,
            performance_tracker,
        }
    }

    /// Fast logging setup with minimal overhead
    pub fn setup_logging(&self) -> Result<(), StartupError> {
        let log_start = Instant::now();

        // Use compact format for better performance
        tracing_subscriber::fmt()
            .compact()
            .with_target(false) // Reduce log overhead
            .with_thread_ids(false) // Reduce log overhead
            .with_file(false) // Reduce log overhead for CLI
            .init();

        debug!(
            "Logging setup in {:.2}ms",
            log_start.elapsed().as_secs_f64() * 1000.0
        );
        Ok(())
    }

    /// Pre-warm commonly used components
    pub async fn pre_warm(&self) -> Result<(), StartupError> {
        let warm_start = Instant::now();

        // Pre-allocate common data structures
        let _temp_hashmap: std::collections::HashMap<String, String> =
            std::collections::HashMap::with_capacity(16);

        // Pre-warm tokio runtime
        tokio::task::yield_now().await;

        debug!(
            "Pre-warming completed in {:.2}ms",
            warm_start.elapsed().as_secs_f64() * 1000.0
        );
        Ok(())
    }

    /// Get total startup time
    pub fn get_startup_time(&self) -> Duration {
        self.startup_time.elapsed()
    }
}

/// Optimized command execution with caching
pub struct CommandExecutor {
    resources: Arc<CliResources>,
    command_cache: std::sync::Mutex<lru::LruCache<String, CachedResult>>,
}

#[derive(Clone)]
struct CachedResult {
    result: String,
    timestamp: Instant,
    ttl: Duration,
}

impl CommandExecutor {
    /// Create new command executor
    pub fn new(resources: Arc<CliResources>) -> Self {
        Self {
            resources,
            command_cache: std::sync::Mutex::new(lru::LruCache::new(
                std::num::NonZeroUsize::new(32).unwrap(),
            )),
        }
    }

    /// Execute command with caching
    pub async fn execute_cached<F, R>(
        &self,
        command_key: &str,
        ttl: Duration,
        executor: F,
    ) -> Result<R, Box<dyn std::error::Error + Send + Sync>>
    where
        F: std::future::Future<Output = Result<R, Box<dyn std::error::Error + Send + Sync>>>,
        R: Clone + std::fmt::Debug + serde::Serialize + serde::de::DeserializeOwned,
    {
        let cache_key = command_key.to_string();

        // Check cache first
        if let Some(cached) = self.get_from_cache(&cache_key) {
            if cached.timestamp.elapsed() < cached.ttl {
                debug!("Cache hit for command: {}", command_key);
                return serde_json::from_str(&cached.result)
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>);
            }
        }

        // Execute command
        let cmd_tracker = self
            .resources
            .performance_tracker
            .start_command(command_key);

        let result = match executor.await {
            Ok(result) => {
                cmd_tracker.complete(true).await;

                // Cache the result
                self.cache_result(&cache_key, &result, ttl)?;

                Ok(result)
            }
            Err(e) => {
                cmd_tracker.complete_with_error("execution_error").await;
                Err(e)
            }
        };

        result
    }

    /// Get result from cache
    fn get_from_cache(&self, key: &str) -> Option<CachedResult> {
        let mut cache = self.command_cache.lock().ok()?;
        cache.get(key).cloned()
    }

    /// Cache execution result
    fn cache_result<R>(
        &self,
        key: &str,
        result: &R,
        ttl: Duration,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
    where
        R: serde::Serialize,
    {
        let serialized = serde_json::to_string(result)?;
        let cached_result = CachedResult {
            result: serialized,
            timestamp: Instant::now(),
            ttl,
        };

        if let Ok(mut cache) = self.command_cache.lock() {
            cache.put(key.to_string(), cached_result);
        }

        Ok(())
    }

    /// Clear cache
    pub fn clear_cache(&self) {
        if let Ok(mut cache) = self.command_cache.lock() {
            cache.clear();
        }
    }
}

/// Resource management for efficient cleanup
pub struct ResourceManager {
    cleanup_tasks: Vec<tokio::task::JoinHandle<()>>,
}

impl Default for ResourceManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ResourceManager {
    /// Create new resource manager
    pub fn new() -> Self {
        Self {
            cleanup_tasks: Vec::new(),
        }
    }

    /// Register cleanup task
    pub fn register_cleanup<F>(&mut self, cleanup: F)
    where
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        let handle = tokio::spawn(cleanup);
        self.cleanup_tasks.push(handle);
    }

    /// Shutdown all resources gracefully
    pub async fn shutdown(self) {
        info!("Shutting down CLI resources...");

        // Cancel all cleanup tasks
        for handle in self.cleanup_tasks {
            handle.abort();
        }

        // Give tasks time to clean up
        tokio::time::sleep(Duration::from_millis(100)).await;

        info!("CLI resources shutdown complete");
    }
}

/// Startup error types
#[derive(Debug, thiserror::Error)]
pub enum StartupError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Logging setup error: {0}")]
    Logging(String),

    #[error("Resource initialization error: {0}")]
    Resource(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Async runtime optimization
pub fn optimize_runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2) // Limit threads for CLI
        .thread_name("qudag-cli")
        .thread_stack_size(2 * 1024 * 1024) // 2MB stack
        .enable_all()
        .build()
        .expect("Failed to create tokio runtime")
}

/// Memory-efficient command parsing
pub fn optimize_clap_parser() -> clap::Command {
    use clap::{Arg, ArgAction, Command};

    // Pre-allocate command structure for better performance
    Command::new("qudag")
        .version(env!("CARGO_PKG_VERSION"))
        .about("QuDAG node operation and management CLI")
        .arg_required_else_help(true)
        .subcommand_required(true)
        .disable_help_subcommand(true) // Reduce memory usage
        .disable_version_flag(false)
        .subcommands([
            Command::new("start").about("Start the QuDAG node").args([
                Arg::new("data-dir")
                    .long("data-dir")
                    .help("Data directory")
                    .value_name("DIR"),
                Arg::new("port")
                    .long("port")
                    .help("Network port")
                    .value_name("PORT")
                    .value_parser(clap::value_parser!(u16)),
                Arg::new("peers")
                    .long("peers")
                    .help("Initial peers")
                    .value_name("PEERS")
                    .action(ArgAction::Append),
            ]),
            Command::new("stop").about("Stop the QuDAG node"),
            Command::new("status").about("Show node status"),
            Command::new("peer")
                .about("Peer management commands")
                .subcommand_required(true)
                .subcommands([
                    Command::new("list").about("List all peers"),
                    Command::new("add").about("Add a new peer").arg(
                        Arg::new("address")
                            .help("Peer address")
                            .required(true)
                            .value_name("ADDRESS"),
                    ),
                    Command::new("remove").about("Remove a peer").arg(
                        Arg::new("address")
                            .help("Peer address")
                            .required(true)
                            .value_name("ADDRESS"),
                    ),
                ]),
            Command::new("network")
                .about("Network management commands")
                .subcommand_required(true)
                .subcommands([
                    Command::new("stats").about("Display network statistics"),
                    Command::new("test").about("Test network connectivity"),
                ]),
            Command::new("dag").about("DAG visualization").args([
                Arg::new("output")
                    .long("output")
                    .help("Output file")
                    .value_name("FILE"),
                Arg::new("format")
                    .long("format")
                    .help("Output format")
                    .value_name("FORMAT"),
            ]),
        ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_startup_optimizer() {
        let optimizer = StartupOptimizer::new();
        let resources = optimizer.initialize().await.unwrap();
        assert!(optimizer.get_startup_time() < Duration::from_millis(500));
    }

    #[tokio::test]
    async fn test_command_executor() {
        let optimizer = StartupOptimizer::new();
        let resources = optimizer.initialize().await.unwrap();
        let executor = CommandExecutor::new(resources);

        let result = executor
            .execute_cached("test_command", Duration::from_secs(60), async {
                Ok::<String, Box<dyn std::error::Error + Send + Sync>>("test_result".to_string())
            })
            .await
            .unwrap();

        assert_eq!(result, "test_result");
    }

    #[test]
    fn test_optimized_clap_parser() {
        let cmd = optimize_clap_parser();
        assert_eq!(cmd.get_name(), "qudag");
        assert!(cmd.get_subcommands().count() > 0);
    }
}
