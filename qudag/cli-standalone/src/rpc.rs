use anyhow::{anyhow, Result};
use qudag_crypto::ml_dsa::MlDsaKeyPair;
use qudag_protocol::NodeConfig;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpStream, UnixStream};
use tokio::sync::Mutex;
use tokio::time::{sleep, timeout, Duration};
use tracing::{debug, warn};
use uuid::Uuid;

/// RPC request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcRequest {
    pub id: Uuid,
    pub method: String,
    pub params: serde_json::Value,
}

/// RPC response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcResponse {
    pub id: Uuid,
    pub result: Option<serde_json::Value>,
    pub error: Option<RpcError>,
}

/// RPC error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcError {
    pub code: i32,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

/// Node status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeStatus {
    pub node_id: String,
    pub state: String,
    pub uptime: u64,
    pub peers: Vec<PeerInfo>,
    pub network_stats: NetworkStats,
    pub dag_stats: DagStats,
    pub memory_usage: MemoryStats,
}

/// Peer information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub id: String,
    pub address: String,
    pub connected_duration: u64,
    pub messages_sent: u64,
    pub messages_received: u64,
    pub last_seen: u64,
    pub status: String,
    pub latency: Option<f64>,
}

/// Network statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    pub total_connections: usize,
    pub active_connections: usize,
    pub messages_sent: u64,
    pub messages_received: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub average_latency: f64,
    pub uptime: u64,
}

/// DAG statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DagStats {
    pub vertex_count: usize,
    pub edge_count: usize,
    pub tip_count: usize,
    pub finalized_height: u64,
    pub pending_transactions: usize,
}

/// Memory statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    pub total_allocated: usize,
    pub current_usage: usize,
    pub peak_usage: usize,
}

/// Wallet information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletInfo {
    pub public_key: String,
    pub balance: u64,
    pub address: String,
    pub key_type: String,
}

/// Network test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkTestResult {
    pub peer_id: String,
    pub address: String,
    pub reachable: bool,
    pub latency: Option<f64>,
    pub error: Option<String>,
}

/// Trait for async read/write operations
#[async_trait::async_trait]
trait AsyncReadWrite: Send + Sync {
    async fn read_u32(&mut self) -> Result<u32>;
    async fn read_exact(&mut self, buf: &mut [u8]) -> Result<()>;
    async fn write_u32(&mut self, val: u32) -> Result<()>;
    async fn write_all(&mut self, buf: &[u8]) -> Result<()>;
    async fn flush(&mut self) -> Result<()>;
}

#[async_trait::async_trait]
impl AsyncReadWrite for TcpStream {
    async fn read_u32(&mut self) -> Result<u32> {
        let mut buf = [0u8; 4];
        tokio::io::AsyncReadExt::read_exact(self, &mut buf).await?;
        Ok(u32::from_be_bytes(buf))
    }

    async fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
        tokio::io::AsyncReadExt::read_exact(self, buf).await?;
        Ok(())
    }

    async fn write_u32(&mut self, val: u32) -> Result<()> {
        AsyncWriteExt::write_all(self, &val.to_be_bytes()).await?;
        Ok(())
    }

    async fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        AsyncWriteExt::write_all(self, buf).await?;
        Ok(())
    }

    async fn flush(&mut self) -> Result<()> {
        AsyncWriteExt::flush(self).await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl AsyncReadWrite for UnixStream {
    async fn read_u32(&mut self) -> Result<u32> {
        let mut buf = [0u8; 4];
        tokio::io::AsyncReadExt::read_exact(self, &mut buf).await?;
        Ok(u32::from_be_bytes(buf))
    }

    async fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
        tokio::io::AsyncReadExt::read_exact(self, buf).await?;
        Ok(())
    }

    async fn write_u32(&mut self, val: u32) -> Result<()> {
        AsyncWriteExt::write_all(self, &val.to_be_bytes()).await?;
        Ok(())
    }

    async fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        AsyncWriteExt::write_all(self, buf).await?;
        Ok(())
    }

    async fn flush(&mut self) -> Result<()> {
        AsyncWriteExt::flush(self).await?;
        Ok(())
    }
}

/// Transport type for RPC client
#[derive(Debug, Clone)]
pub enum RpcTransport {
    /// TCP socket transport
    Tcp { host: String, port: u16 },
    /// Unix domain socket transport
    Unix { path: String },
}

/// Connection pool for RPC client
#[derive(Debug)]
struct ConnectionPool {
    transport: RpcTransport,
    connections: Arc<Mutex<Vec<TcpStream>>>,
    unix_connections: Arc<Mutex<Vec<UnixStream>>>,
    #[allow(dead_code)]
    max_connections: usize,
}

/// RPC client for communicating with QuDAG nodes
pub struct RpcClient {
    transport: RpcTransport,
    timeout: Duration,
    retry_attempts: u32,
    retry_delay: Duration,
    pool: Option<ConnectionPool>,
    auth_token: Option<String>,
    auth_key: Option<MlDsaKeyPair>,
    client_id: Option<String>,
}

impl RpcClient {
    /// Create new RPC client with TCP transport
    pub fn new_tcp(host: String, port: u16) -> Self {
        Self {
            transport: RpcTransport::Tcp { host, port },
            timeout: Duration::from_secs(30),
            retry_attempts: 3,
            retry_delay: Duration::from_millis(500),
            pool: None,
            auth_token: None,
            auth_key: None,
            client_id: None,
        }
    }

    /// Create new RPC client with Unix socket transport
    pub fn new_unix(path: String) -> Self {
        Self {
            transport: RpcTransport::Unix { path },
            timeout: Duration::from_secs(30),
            retry_attempts: 3,
            retry_delay: Duration::from_millis(500),
            pool: None,
            auth_token: None,
            auth_key: None,
            client_id: None,
        }
    }

    /// Set request timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set retry configuration
    pub fn with_retry(mut self, attempts: u32, delay: Duration) -> Self {
        self.retry_attempts = attempts;
        self.retry_delay = delay;
        self
    }

    /// Enable connection pooling
    pub fn with_pool(mut self, max_connections: usize) -> Self {
        self.pool = Some(ConnectionPool {
            transport: self.transport.clone(),
            connections: Arc::new(Mutex::new(Vec::new())),
            unix_connections: Arc::new(Mutex::new(Vec::new())),
            max_connections,
        });
        self
    }

    /// Set authentication token
    pub fn with_auth_token(mut self, token: String) -> Self {
        self.auth_token = Some(token);
        self
    }

    /// Set ML-DSA authentication
    pub fn with_ml_dsa_auth(mut self, client_id: String, keypair: MlDsaKeyPair) -> Self {
        self.client_id = Some(client_id);
        self.auth_key = Some(keypair);
        self
    }

    /// Connect to the RPC server
    async fn connect(&self) -> Result<Box<dyn AsyncReadWrite>> {
        match &self.transport {
            RpcTransport::Tcp { host, port } => {
                let stream = TcpStream::connect(format!("{}:{}", host, port)).await?;
                Ok(Box::new(stream))
            }
            RpcTransport::Unix { path } => {
                let stream = UnixStream::connect(path).await?;
                Ok(Box::new(stream))
            }
        }
    }

    /// Get connection from pool or create new one
    async fn get_connection(&self) -> Result<Box<dyn AsyncReadWrite>> {
        if let Some(pool) = &self.pool {
            match &pool.transport {
                RpcTransport::Tcp { host, port } => {
                    let mut conns = pool.connections.lock().await;
                    if let Some(conn) = conns.pop() {
                        // TODO: Check if connection is still alive
                        return Ok(Box::new(conn));
                    }
                    drop(conns);
                    // Create new connection
                    let stream = TcpStream::connect(format!("{}:{}", host, port)).await?;
                    Ok(Box::new(stream))
                }
                RpcTransport::Unix { path } => {
                    let mut conns = pool.unix_connections.lock().await;
                    if let Some(conn) = conns.pop() {
                        return Ok(Box::new(conn));
                    }
                    drop(conns);
                    let stream = UnixStream::connect(path).await?;
                    Ok(Box::new(stream))
                }
            }
        } else {
            self.connect().await
        }
    }

    /// Send RPC request with retry logic
    async fn send_request(
        &self,
        method: &str,
        mut params: serde_json::Value,
    ) -> Result<serde_json::Value> {
        // Add authentication to params if configured
        if let Some(token) = &self.auth_token {
            params["auth_token"] = serde_json::Value::String(token.clone());
        } else if let (Some(client_id), Some(keypair)) = (&self.client_id, &self.auth_key) {
            let request_id = Uuid::new_v4();
            let message = format!("{}:{}", method, request_id);
            let mut rng = rand::thread_rng();
            let signature = keypair.sign(message.as_bytes(), &mut rng)?;
            params["client_id"] = serde_json::Value::String(client_id.clone());
            params["signature"] = serde_json::Value::String(hex::encode(signature));
        }

        let mut last_error = None;

        for attempt in 0..self.retry_attempts {
            if attempt > 0 {
                sleep(self.retry_delay).await;
                debug!(
                    "Retrying RPC request, attempt {}/{}",
                    attempt + 1,
                    self.retry_attempts
                );
            }

            match self.send_request_once(method, params.clone()).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    warn!("RPC request failed: {}", e);
                    last_error = Some(e);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow!("All retry attempts failed")))
    }

    /// Send RPC request once (no retry)
    async fn send_request_once(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let request = RpcRequest {
            id: Uuid::new_v4(),
            method: method.to_string(),
            params,
        };

        let request_data = serde_json::to_vec(&request)?;

        // Get connection
        let mut stream = timeout(self.timeout, self.get_connection())
            .await
            .map_err(|_| anyhow!("Connection timeout"))??;

        // Send request
        timeout(self.timeout, async {
            stream.write_u32(request_data.len() as u32).await?;
            stream.write_all(&request_data).await?;
            stream.flush().await?;
            Ok::<(), anyhow::Error>(())
        })
        .await
        .map_err(|_| anyhow!("Request send timeout"))??;

        // Read response
        let response_len = timeout(self.timeout, stream.read_u32())
            .await
            .map_err(|_| anyhow!("Response read timeout"))??;

        if response_len > 10 * 1024 * 1024 {
            return Err(anyhow!("Response too large: {} bytes", response_len));
        }

        let mut response_data = vec![0u8; response_len as usize];
        timeout(self.timeout, stream.read_exact(&mut response_data))
            .await
            .map_err(|_| anyhow!("Response read timeout"))??;

        let response: RpcResponse = serde_json::from_slice(&response_data)?;

        if let Some(error) = response.error {
            return Err(anyhow!("RPC error {}: {}", error.code, error.message));
        }

        response.result.ok_or_else(|| anyhow!("Empty response"))
    }

    /// Get node status
    pub async fn get_status(&self) -> Result<NodeStatus> {
        let result = self
            .send_request("get_status", serde_json::Value::Null)
            .await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Start node
    pub async fn start_node(&self, config: NodeConfig) -> Result<()> {
        let params = serde_json::to_value(config)?;
        self.send_request("start", params).await?;
        Ok(())
    }

    /// Stop node
    pub async fn stop_node(&self) -> Result<()> {
        self.send_request("stop", serde_json::Value::Null).await?;
        Ok(())
    }

    /// Restart node
    pub async fn restart_node(&self) -> Result<()> {
        self.send_request("restart", serde_json::Value::Null)
            .await?;
        Ok(())
    }

    /// Add peer
    pub async fn add_peer(&self, address: String) -> Result<String> {
        let params = serde_json::json!({ "address": address });
        let result = self.send_request("add_peer", params).await?;
        Ok(serde_json::from_value::<serde_json::Value>(result)?
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("Peer added successfully")
            .to_string())
    }

    /// Remove peer
    pub async fn remove_peer(&self, peer_id: String) -> Result<String> {
        let params = serde_json::json!({ "peer_id": peer_id });
        let result = self.send_request("remove_peer", params).await?;
        Ok(serde_json::from_value::<serde_json::Value>(result)?
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("Peer removed successfully")
            .to_string())
    }

    /// List peers
    pub async fn list_peers(&self) -> Result<Vec<PeerInfo>> {
        let result = self
            .send_request("list_peers", serde_json::Value::Null)
            .await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Get peer information
    pub async fn get_peer_info(&self, peer_id: String) -> Result<PeerInfo> {
        let params = serde_json::json!({ "peer_id": peer_id });
        let result = self.send_request("get_peer_info", params).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Ban peer
    pub async fn ban_peer(&self, peer_id: String) -> Result<String> {
        let params = serde_json::json!({ "peer_id": peer_id });
        let result = self.send_request("ban_peer", params).await?;
        Ok(serde_json::from_value::<serde_json::Value>(result)?
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("Peer banned successfully")
            .to_string())
    }

    /// Unban peer
    pub async fn unban_peer(&self, address: String) -> Result<String> {
        let params = serde_json::json!({ "address": address });
        let result = self.send_request("unban_peer", params).await?;
        Ok(serde_json::from_value::<serde_json::Value>(result)?
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("Peer unbanned successfully")
            .to_string())
    }

    /// Get network statistics
    pub async fn get_network_stats(&self) -> Result<NetworkStats> {
        let result = self
            .send_request("get_network_stats", serde_json::Value::Null)
            .await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Test network connectivity
    pub async fn test_network(&self) -> Result<Vec<NetworkTestResult>> {
        let result = self
            .send_request("test_network", serde_json::Value::Null)
            .await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Get wallet information
    pub async fn get_wallet_info(&self) -> Result<WalletInfo> {
        let result = self
            .send_request("get_wallet_info", serde_json::Value::Null)
            .await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Create new wallet
    pub async fn create_wallet(&self, password: String) -> Result<String> {
        let params = serde_json::json!({ "password": password });
        let result = self.send_request("create_wallet", params).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Import wallet from seed
    pub async fn import_wallet(&self, seed: String, password: String) -> Result<()> {
        let params = serde_json::json!({ "seed": seed, "password": password });
        self.send_request("import_wallet", params).await?;
        Ok(())
    }

    /// Export wallet seed
    pub async fn export_wallet(&self, password: String) -> Result<String> {
        let params = serde_json::json!({ "password": password });
        let result = self.send_request("export_wallet", params).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Get DAG visualization data
    pub async fn get_dag_data(&self) -> Result<serde_json::Value> {
        self.send_request("get_dag_data", serde_json::Value::Null)
            .await
    }

    /// Debug network
    pub async fn debug_network(&self) -> Result<serde_json::Value> {
        self.send_request("debug_network", serde_json::Value::Null)
            .await
    }

    /// Debug consensus
    pub async fn debug_consensus(&self) -> Result<serde_json::Value> {
        self.send_request("debug_consensus", serde_json::Value::Null)
            .await
    }

    /// Debug performance
    pub async fn debug_performance(&self) -> Result<serde_json::Value> {
        self.send_request("debug_performance", serde_json::Value::Null)
            .await
    }

    /// Security audit
    pub async fn security_audit(&self) -> Result<serde_json::Value> {
        self.send_request("security_audit", serde_json::Value::Null)
            .await
    }

    /// Get configuration
    pub async fn get_config(&self) -> Result<serde_json::Value> {
        self.send_request("get_config", serde_json::Value::Null)
            .await
    }

    /// Update configuration
    pub async fn update_config(&self, config: serde_json::Value) -> Result<()> {
        self.send_request("update_config", config).await?;
        Ok(())
    }

    /// Validate configuration
    pub async fn validate_config(&self, config: serde_json::Value) -> Result<bool> {
        let params = serde_json::json!({ "config": config });
        let result = self.send_request("validate_config", params).await?;
        Ok(serde_json::from_value(result)?)
    }
}

/// Check if node is running
pub async fn is_node_running(port: u16) -> bool {
    TcpStream::connect(format!("127.0.0.1:{}", port))
        .await
        .is_ok()
}

/// Wait for node to start
pub async fn wait_for_node_start(port: u16, timeout_secs: u64) -> Result<()> {
    let start = std::time::Instant::now();
    let timeout_duration = Duration::from_secs(timeout_secs);

    while start.elapsed() < timeout_duration {
        if is_node_running(port).await {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    Err(anyhow!(
        "Node failed to start within {} seconds",
        timeout_secs
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rpc_request_serialization() {
        let request = RpcRequest {
            id: Uuid::new_v4(),
            method: "test_method".to_string(),
            params: serde_json::json!({"key": "value"}),
        };

        let serialized = serde_json::to_string(&request).unwrap();
        let deserialized: RpcRequest = serde_json::from_str(&serialized).unwrap();

        assert_eq!(request.method, deserialized.method);
    }

    #[test]
    fn test_rpc_response_serialization() {
        let response = RpcResponse {
            id: Uuid::new_v4(),
            result: Some(serde_json::json!({"status": "ok"})),
            error: None,
        };

        let serialized = serde_json::to_string(&response).unwrap();
        let deserialized: RpcResponse = serde_json::from_str(&serialized).unwrap();

        assert!(deserialized.result.is_some());
        assert!(deserialized.error.is_none());
    }
}
