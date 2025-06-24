use crate::config::NodeConfigManager;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::process::{Child, Command};
use tokio::signal;
use tokio::sync::{Mutex, RwLock};
use tokio::time::interval;
use tracing::{error, info, warn};

/// Process ID file location
const PID_FILE: &str = "qudag.pid";
const LOG_FILE: &str = "qudag.log";
const CONFIG_FILE: &str = "config.toml";

/// Node state information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeState {
    pub pid: u32,
    pub started_at: u64,
    pub port: u16,
    pub data_dir: PathBuf,
    pub log_file: PathBuf,
    pub config_file: PathBuf,
}

/// Node manager configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeManagerConfig {
    /// Base directory for node data
    pub base_dir: PathBuf,
    /// Default network port
    pub default_port: u16,
    /// Log rotation size in MB
    pub log_rotation_size_mb: u64,
    /// Max log files to keep
    pub max_log_files: usize,
    /// Health check interval in seconds
    pub health_check_interval: u64,
    /// Graceful shutdown timeout in seconds
    pub shutdown_timeout: u64,
}

impl Default for NodeManagerConfig {
    fn default() -> Self {
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let base_dir = home_dir.join(".qudag");

        Self {
            base_dir,
            default_port: 8000,
            log_rotation_size_mb: 100,
            max_log_files: 5,
            health_check_interval: 60,
            shutdown_timeout: 30,
        }
    }
}

/// Node process manager
pub struct NodeManager {
    config: NodeManagerConfig,
    state: RwLock<Option<NodeState>>,
    process: Mutex<Option<Child>>,
    config_manager: NodeConfigManager,
}

impl NodeManager {
    /// Create a new node manager
    pub fn new(config: NodeManagerConfig) -> Result<Self> {
        // Create base directory if it doesn't exist
        fs::create_dir_all(&config.base_dir)?;

        // Initialize config manager
        let config_manager = NodeConfigManager::new(config.base_dir.join(CONFIG_FILE))?;

        // Load existing state if available
        let state = Self::load_state(&config.base_dir)?;

        Ok(Self {
            config,
            state: RwLock::new(state),
            process: Mutex::new(None),
            config_manager,
        })
    }

    /// Check if a node is currently running
    pub async fn is_running(&self) -> bool {
        if let Some(state) = self.state.read().await.as_ref() {
            // Check if process is still alive
            Self::check_process_alive(state.pid).await
        } else {
            false
        }
    }

    /// Start the node process
    pub async fn start_node(
        &self,
        port: Option<u16>,
        data_dir: Option<PathBuf>,
        peers: Vec<String>,
        foreground: bool,
    ) -> Result<()> {
        // Check if already running
        if self.is_running().await {
            return Err(anyhow!("Node is already running"));
        }

        let port = port.unwrap_or(self.config.default_port);
        let data_dir = data_dir.unwrap_or_else(|| self.config.base_dir.join("data"));

        // Create data directory
        fs::create_dir_all(&data_dir)?;

        // Update configuration
        self.config_manager
            .update_config(|config| {
                config.network_port = port;
                config.data_dir = data_dir.clone();
                if !peers.is_empty() {
                    config.initial_peers = peers.clone();
                }
                Ok(())
            })
            .await?;

        // Prepare log file
        let log_file = self.config.base_dir.join(LOG_FILE);
        let log_file_handle = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file)?;

        info!(
            "Starting QuDAG node on port {} with data dir {:?}",
            port, data_dir
        );

        // Build command
        let mut cmd = Command::new(std::env::current_exe()?);
        cmd.arg("run-node")
            .arg("--port")
            .arg(port.to_string())
            .arg("--data-dir")
            .arg(data_dir.display().to_string());

        for peer in &peers {
            cmd.arg("--peer").arg(peer);
        }

        // Configure process
        if !foreground {
            cmd.stdin(Stdio::null())
                .stdout(Stdio::from(log_file_handle.try_clone()?))
                .stderr(Stdio::from(log_file_handle));
        }

        // Spawn process
        let mut child = cmd.spawn()?;
        let pid = child
            .id()
            .ok_or_else(|| anyhow!("Failed to get process ID"))?;

        // Create node state
        let state = NodeState {
            pid,
            started_at: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            port,
            data_dir,
            log_file: log_file.clone(),
            config_file: self.config.base_dir.join(CONFIG_FILE),
        };

        // Save state
        Self::save_state(&self.config.base_dir, &state)?;
        *self.state.write().await = Some(state.clone());

        if foreground {
            // Run in foreground
            info!("Running node in foreground mode");

            // Set up signal handlers
            let mut sigterm = signal::unix::signal(signal::unix::SignalKind::terminate())?;
            let mut sigint = signal::unix::signal(signal::unix::SignalKind::interrupt())?;

            tokio::select! {
                _ = sigterm.recv() => {
                    info!("Received SIGTERM, shutting down...");
                }
                _ = sigint.recv() => {
                    info!("Received SIGINT, shutting down...");
                }
                status = child.wait() => {
                    match status {
                        Ok(status) => {
                            if status.success() {
                                info!("Node process exited successfully");
                            } else {
                                error!("Node process exited with status: {}", status);
                            }
                        }
                        Err(e) => {
                            error!("Failed to wait for node process: {}", e);
                        }
                    }
                }
            }

            // Clean up
            self.cleanup_state().await?;
        } else {
            // Store process handle for background mode
            *self.process.lock().await = Some(child);

            // Start health check task
            self.start_health_check().await;

            info!("Node started successfully in background (PID: {})", pid);
            info!("Log file: {:?}", log_file);
        }

        Ok(())
    }

    /// Stop the running node
    pub async fn stop_node(&self, force: bool) -> Result<()> {
        let state = match self.state.read().await.as_ref() {
            Some(s) => s.clone(),
            None => return Err(anyhow!("No node is currently running")),
        };

        info!("Stopping node (PID: {})", state.pid);

        // Try graceful shutdown first
        if !force {
            if let Err(e) = self.graceful_shutdown(&state).await {
                warn!("Graceful shutdown failed: {}, attempting force kill", e);
                self.force_kill(&state).await?;
            }
        } else {
            self.force_kill(&state).await?;
        }

        // Clean up state
        self.cleanup_state().await?;

        info!("Node stopped successfully");
        Ok(())
    }

    /// Restart the node
    pub async fn restart_node(&self, force: bool) -> Result<()> {
        info!("Restarting node...");

        // Get current configuration
        let (port, data_dir, peers) = if let Some(state) = self.state.read().await.as_ref() {
            let config = self.config_manager.load_config().await?;
            (
                Some(state.port),
                Some(state.data_dir.clone()),
                config.initial_peers,
            )
        } else {
            return Err(anyhow!("No node is currently running"));
        };

        // Stop the node
        self.stop_node(force).await?;

        // Wait a moment for cleanup
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Start the node with same configuration
        self.start_node(port, data_dir, peers, false).await?;

        Ok(())
    }

    /// Get node status
    pub async fn get_status(&self) -> Result<NodeStatus> {
        if let Some(state) = self.state.read().await.as_ref() {
            let is_running = Self::check_process_alive(state.pid).await;
            let uptime = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() - state.started_at;

            Ok(NodeStatus {
                is_running,
                pid: Some(state.pid),
                port: state.port,
                data_dir: state.data_dir.clone(),
                log_file: state.log_file.clone(),
                uptime_seconds: if is_running { Some(uptime) } else { None },
                last_health_check: None, // TODO: Implement health check tracking
            })
        } else {
            Ok(NodeStatus {
                is_running: false,
                pid: None,
                port: self.config.default_port,
                data_dir: self.config.base_dir.join("data"),
                log_file: self.config.base_dir.join(LOG_FILE),
                uptime_seconds: None,
                last_health_check: None,
            })
        }
    }

    /// Generate systemd service file
    pub async fn generate_systemd_service(&self, output_path: Option<PathBuf>) -> Result<String> {
        let exe_path = std::env::current_exe()?;
        let config = self.config_manager.load_config().await?;

        let service_content = format!(
            r#"[Unit]
Description=QuDAG Protocol Node
After=network.target

[Service]
Type=simple
ExecStart={} start --port {} --data-dir {}
ExecStop={} stop
Restart=on-failure
RestartSec=10
User={}
WorkingDirectory={}

# Security settings
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths={}

# Resource limits
LimitNOFILE=65536
MemoryLimit=2G

[Install]
WantedBy=multi-user.target
"#,
            exe_path.display(),
            config.network_port,
            config.data_dir.display(),
            exe_path.display(),
            whoami::username(),
            self.config.base_dir.display(),
            config.data_dir.display(),
        );

        if let Some(path) = output_path {
            fs::write(&path, &service_content)?;
            info!("Systemd service file written to: {:?}", path);
        }

        Ok(service_content)
    }

    /// Tail log file
    pub async fn tail_logs(&self, lines: usize, follow: bool) -> Result<()> {
        let log_file = self.config.base_dir.join(LOG_FILE);

        if !log_file.exists() {
            return Err(anyhow!("Log file not found: {:?}", log_file));
        }

        if follow {
            // Use tail -f equivalent
            let mut cmd = Command::new("tail");
            cmd.arg("-f")
                .arg("-n")
                .arg(lines.to_string())
                .arg(&log_file);

            let mut child = cmd.spawn()?;

            // Set up signal handler
            let mut sigint = signal::unix::signal(signal::unix::SignalKind::interrupt())?;

            tokio::select! {
                _ = sigint.recv() => {
                    child.kill().await?;
                }
                _ = child.wait() => {}
            }
        } else {
            // Read last N lines
            let content = fs::read_to_string(&log_file)?;
            let lines_vec: Vec<&str> = content.lines().collect();
            let start = lines_vec.len().saturating_sub(lines);

            for line in &lines_vec[start..] {
                println!("{}", line);
            }
        }

        Ok(())
    }

    /// Load node state from disk
    fn load_state(base_dir: &Path) -> Result<Option<NodeState>> {
        let pid_file = base_dir.join(PID_FILE);

        if !pid_file.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&pid_file)?;
        let state: NodeState = serde_json::from_str(&content)?;

        // Verify process is still alive
        if Self::check_process_alive_sync(state.pid) {
            Ok(Some(state))
        } else {
            // Clean up stale PID file
            let _ = fs::remove_file(&pid_file);
            Ok(None)
        }
    }

    /// Save node state to disk
    fn save_state(base_dir: &Path, state: &NodeState) -> Result<()> {
        let pid_file = base_dir.join(PID_FILE);
        let content = serde_json::to_string_pretty(state)?;
        fs::write(&pid_file, content)?;
        Ok(())
    }

    /// Check if a process is alive (async)
    async fn check_process_alive(pid: u32) -> bool {
        // Use kill -0 to check if process exists
        match Command::new("kill")
            .arg("-0")
            .arg(pid.to_string())
            .output()
            .await
        {
            Ok(output) => output.status.success(),
            Err(_) => false,
        }
    }

    /// Check if a process is alive (sync)
    fn check_process_alive_sync(pid: u32) -> bool {
        // Use nix crate or system call
        #[allow(unsafe_code)]
        unsafe {
            libc::kill(pid as i32, 0) == 0
        }
    }

    /// Graceful shutdown
    async fn graceful_shutdown(&self, state: &NodeState) -> Result<()> {
        info!("Attempting graceful shutdown of PID {}", state.pid);

        // Send SIGTERM
        Command::new("kill")
            .arg("-TERM")
            .arg(state.pid.to_string())
            .output()
            .await?;

        // Wait for process to exit
        let deadline =
            tokio::time::Instant::now() + Duration::from_secs(self.config.shutdown_timeout);
        let mut check_interval = interval(Duration::from_millis(100));

        while tokio::time::Instant::now() < deadline {
            check_interval.tick().await;

            if !Self::check_process_alive(state.pid).await {
                info!("Process {} exited gracefully", state.pid);
                return Ok(());
            }
        }

        Err(anyhow!("Process did not exit within timeout"))
    }

    /// Force kill the process
    async fn force_kill(&self, state: &NodeState) -> Result<()> {
        warn!("Force killing process {}", state.pid);

        Command::new("kill")
            .arg("-KILL")
            .arg(state.pid.to_string())
            .output()
            .await?;

        // Wait a moment for process to die
        tokio::time::sleep(Duration::from_millis(500)).await;

        if Self::check_process_alive(state.pid).await {
            return Err(anyhow!("Failed to kill process {}", state.pid));
        }

        Ok(())
    }

    /// Clean up state files
    async fn cleanup_state(&self) -> Result<()> {
        *self.state.write().await = None;
        *self.process.lock().await = None;

        let pid_file = self.config.base_dir.join(PID_FILE);
        if pid_file.exists() {
            fs::remove_file(&pid_file)?;
        }

        Ok(())
    }

    /// Start health check task
    async fn start_health_check(&self) {
        let interval_secs = self.config.health_check_interval;
        let base_dir = self.config.base_dir.clone();

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(interval_secs));

            loop {
                interval.tick().await;

                // Load current state
                if let Ok(Some(state)) = Self::load_state(&base_dir) {
                    if !Self::check_process_alive(state.pid).await {
                        warn!("Node process {} is no longer running", state.pid);
                        // Clean up stale PID file
                        let _ = fs::remove_file(base_dir.join(PID_FILE));
                        break;
                    }
                } else {
                    // No state, exit health check
                    break;
                }
            }
        });
    }

    /// Rotate log files if needed
    pub async fn rotate_logs(&self) -> Result<()> {
        let log_file = self.config.base_dir.join(LOG_FILE);

        if !log_file.exists() {
            return Ok(());
        }

        let metadata = fs::metadata(&log_file)?;
        let size_mb = metadata.len() / (1024 * 1024);

        if size_mb >= self.config.log_rotation_size_mb {
            info!("Rotating log file (size: {} MB)", size_mb);

            // Find next available rotation number
            let mut rotation_num = 1;
            while self
                .config
                .base_dir
                .join(format!("{}.{}", LOG_FILE, rotation_num))
                .exists()
            {
                rotation_num += 1;
            }

            // Rotate current log
            let rotated_file = self
                .config
                .base_dir
                .join(format!("{}.{}", LOG_FILE, rotation_num));
            fs::rename(&log_file, &rotated_file)?;

            // Clean up old logs
            self.cleanup_old_logs().await?;
        }

        Ok(())
    }

    /// Clean up old log files
    async fn cleanup_old_logs(&self) -> Result<()> {
        let mut log_files: Vec<(PathBuf, u64)> = Vec::new();

        // Find all rotated log files
        for entry in fs::read_dir(&self.config.base_dir)? {
            let entry = entry?;
            let path = entry.path();

            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with(LOG_FILE) && name != LOG_FILE {
                    let metadata = fs::metadata(&path)?;
                    let modified = metadata.modified()?.duration_since(UNIX_EPOCH)?.as_secs();
                    log_files.push((path, modified));
                }
            }
        }

        // Sort by modification time (oldest first)
        log_files.sort_by_key(|&(_, time)| time);

        // Remove oldest files if we exceed max_log_files
        while log_files.len() > self.config.max_log_files {
            if let Some((path, _)) = log_files.first() {
                info!("Removing old log file: {:?}", path);
                fs::remove_file(path)?;
                log_files.remove(0);
            }
        }

        Ok(())
    }
}

/// Node status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeStatus {
    pub is_running: bool,
    pub pid: Option<u32>,
    pub port: u16,
    pub data_dir: PathBuf,
    pub log_file: PathBuf,
    pub uptime_seconds: Option<u64>,
    pub last_health_check: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_node_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let config = NodeManagerConfig {
            base_dir: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let manager = NodeManager::new(config).unwrap();
        assert!(!manager.is_running().await);
    }

    #[test]
    fn test_process_alive_check() {
        // Check current process (should be alive)
        let current_pid = std::process::id();
        assert!(NodeManager::check_process_alive_sync(current_pid));

        // Check non-existent process
        assert!(!NodeManager::check_process_alive_sync(999999));
    }
}
