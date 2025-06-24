use crate::ProtocolError;
use qudag_crypto::ml_dsa::MlDsaPublicKey;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream, UnixListener, UnixStream};
use tokio::sync::{mpsc, oneshot, Mutex, RwLock};
use tokio::time::{timeout, Duration};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Extension trait for reading u32 from streams
trait ReadU32Ext: AsyncReadExt + Unpin {
    async fn read_u32(&mut self) -> std::io::Result<u32> {
        let mut buf = [0u8; 4];
        self.read_exact(&mut buf).await?;
        Ok(u32::from_be_bytes(buf))
    }
}

impl<T: AsyncReadExt + Unpin> ReadU32Ext for T {}

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

/// RPC command types
#[derive(Debug, Clone)]
pub enum RpcCommand {
    Stop,
    GetStatus,
    ListPeers,
    AddPeer(String),
    RemovePeer(String),
    GetPeerInfo(String),
    BanPeer(String),
    UnbanPeer(String),
    GetNetworkStats,
    TestNetwork,
}

/// Peer information for RPC responses
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

/// Network test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkTestResult {
    pub peer_id: String,
    pub address: String,
    pub reachable: bool,
    pub latency: Option<f64>,
    pub error: Option<String>,
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

/// Node status with all metrics
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

/// Transport type for RPC server
#[derive(Debug, Clone)]
pub enum RpcTransport {
    /// TCP socket transport
    Tcp(String),
    /// Unix domain socket transport
    Unix(String),
}

/// Forward declaration of NodeRunner to avoid circular imports
type NodeRunnerHandle = Arc<RwLock<dyn NodeRunnerTrait + Send + Sync>>;

/// Trait for NodeRunner operations that RPC server can call
pub trait NodeRunnerTrait: Send + Sync + std::fmt::Debug {
    fn get_status(
        &self,
    ) -> Pin<
        Box<
            dyn std::future::Future<
                    Output = Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>>,
                > + Send,
        >,
    >;
    fn get_connected_peers(
        &self,
    ) -> Pin<Box<dyn std::future::Future<Output = Vec<PeerInfo>> + Send>>;
    fn dial_peer(
        &self,
        address: String,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send>>;
    fn disconnect_peer(
        &self,
        peer_id: &str,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send>>;
    fn get_network_stats(&self) -> Pin<Box<dyn std::future::Future<Output = NetworkStats> + Send>>;
    fn shutdown(
        &self,
    ) -> Pin<
        Box<
            dyn std::future::Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>>
                + Send,
        >,
    >;
}

/// RPC server for handling remote commands
pub struct RpcServer {
    transport: RpcTransport,
    shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
    command_tx: mpsc::Sender<(RpcCommand, tokio::sync::oneshot::Sender<serde_json::Value>)>,
    network_manager: Arc<RwLock<NetworkManager>>,
    /// Handle to the running node for real operations
    node_handle: Option<NodeRunnerHandle>,
    /// Channel to send shutdown signal to the node
    node_shutdown_tx: Option<oneshot::Sender<()>>,
    auth_token: Option<String>,
    rate_limiter: Arc<Mutex<RateLimiter>>,
    auth_keys: Arc<RwLock<HashMap<String, MlDsaPublicKey>>>,
    #[allow(dead_code)]
    start_time: SystemTime,
}

/// Network manager for peer operations that can work with or without a real P2P node
#[derive(Debug)]
pub struct NetworkManager {
    /// Mock peers for when no real node is connected
    mock_peers: HashMap<String, PeerInfo>,
    /// Banned peer addresses
    banned_peers: std::collections::HashSet<String>,
    /// Network statistics
    network_stats: NetworkStats,
    /// Start time for uptime calculation
    start_time: SystemTime,
    /// Handle to real node (if available)
    node_handle: Option<NodeRunnerHandle>,
}

/// Rate limiter for RPC requests
#[derive(Debug)]
struct RateLimiter {
    requests: HashMap<String, Vec<SystemTime>>,
    max_requests_per_minute: usize,
}

impl NetworkManager {
    fn new() -> Self {
        Self {
            mock_peers: HashMap::new(),
            banned_peers: std::collections::HashSet::new(),
            network_stats: NetworkStats {
                total_connections: 0,
                active_connections: 0,
                messages_sent: 0,
                messages_received: 0,
                bytes_sent: 0,
                bytes_received: 0,
                average_latency: 0.0,
                uptime: 0,
            },
            start_time: SystemTime::now(),
            node_handle: None,
        }
    }

    /// Set the handle to the real node for actual operations
    pub fn set_node_handle(&mut self, handle: NodeRunnerHandle) {
        self.node_handle = Some(handle);
    }

    async fn add_peer(&mut self, address: String) -> Result<(), String> {
        if self.banned_peers.contains(&address) {
            return Err("Peer is banned".to_string());
        }

        // Try to use real node if available
        if let Some(node) = &self.node_handle {
            let node_guard = node.read().await;
            return node_guard.dial_peer(address).await;
        }

        // Fall back to mock behavior
        let peer_id = format!("peer_{}", &uuid::Uuid::new_v4().to_string()[..8]);
        let peer_info = PeerInfo {
            id: peer_id.clone(),
            address: address.clone(),
            connected_duration: 0,
            messages_sent: 0,
            messages_received: 0,
            last_seen: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            status: "Connected".to_string(),
            latency: None,
        };

        self.mock_peers.insert(peer_id, peer_info);
        self.network_stats.total_connections += 1;
        self.network_stats.active_connections += 1;
        Ok(())
    }

    async fn remove_peer(&mut self, peer_id: &str) -> Result<(), String> {
        // Try to use real node if available
        if let Some(node) = &self.node_handle {
            let node_guard = node.read().await;
            return node_guard.disconnect_peer(peer_id).await;
        }

        // Fall back to mock behavior
        if self.mock_peers.remove(peer_id).is_some() {
            self.network_stats.active_connections =
                self.network_stats.active_connections.saturating_sub(1);
            Ok(())
        } else {
            Err("Peer not found".to_string())
        }
    }

    async fn get_peer_info(&self, peer_id: &str) -> Option<PeerInfo> {
        // If we have a real node, get peer info from it
        if let Some(node) = &self.node_handle {
            let node_guard = node.read().await;
            let connected_peers = node_guard.get_connected_peers().await;
            return connected_peers.into_iter().find(|p| p.id == peer_id);
        }

        // Fall back to mock peers
        self.mock_peers.get(peer_id).cloned()
    }

    async fn list_peers(&self) -> Vec<PeerInfo> {
        // If we have a real node, get peers from it
        if let Some(node) = &self.node_handle {
            let node_guard = node.read().await;
            return node_guard.get_connected_peers().await;
        }

        // Fall back to mock peers
        self.mock_peers.values().cloned().collect()
    }

    async fn ban_peer(&mut self, peer_id: &str) -> Result<(), String> {
        // Get peer address before removing
        let peer_address = if let Some(node) = &self.node_handle {
            let node_guard = node.read().await;
            let connected_peers = node_guard.get_connected_peers().await;
            connected_peers
                .into_iter()
                .find(|p| p.id == peer_id)
                .map(|p| p.address)
        } else {
            self.mock_peers.get(peer_id).map(|p| p.address.clone())
        };

        if let Some(address) = peer_address {
            self.banned_peers.insert(address);
            self.remove_peer(peer_id).await?;
            Ok(())
        } else {
            Err("Peer not found".to_string())
        }
    }

    fn unban_peer(&mut self, address: &str) -> Result<(), String> {
        if self.banned_peers.remove(address) {
            Ok(())
        } else {
            Err("Peer is not banned".to_string())
        }
    }

    async fn get_network_stats(&mut self) -> NetworkStats {
        // If we have a real node, get stats from it
        if let Some(node) = &self.node_handle {
            let node_guard = node.read().await;
            return node_guard.get_network_stats().await;
        }

        // Fall back to mock stats
        self.network_stats.uptime = self.start_time.elapsed().unwrap_or_default().as_secs();
        self.network_stats.clone()
    }

    async fn test_network(&self) -> Vec<NetworkTestResult> {
        let mut results = Vec::new();

        let peers = if let Some(node) = &self.node_handle {
            let node_guard = node.read().await;
            node_guard.get_connected_peers().await
        } else {
            self.mock_peers.values().cloned().collect()
        };

        for peer in peers {
            let result = self.test_peer_connectivity(&peer).await;
            results.push(result);
        }

        results
    }

    async fn test_peer_connectivity(&self, peer: &PeerInfo) -> NetworkTestResult {
        // Simulate network test - in a real implementation this would do actual connectivity testing
        let start = std::time::Instant::now();

        // Try to parse address and test connectivity
        match peer.address.parse::<std::net::SocketAddr>() {
            Ok(addr) => {
                match timeout(Duration::from_secs(5), tokio::net::TcpStream::connect(addr)).await {
                    Ok(Ok(_)) => NetworkTestResult {
                        peer_id: peer.id.clone(),
                        address: peer.address.clone(),
                        reachable: true,
                        latency: Some(start.elapsed().as_millis() as f64),
                        error: None,
                    },
                    Ok(Err(e)) => NetworkTestResult {
                        peer_id: peer.id.clone(),
                        address: peer.address.clone(),
                        reachable: false,
                        latency: None,
                        error: Some(e.to_string()),
                    },
                    Err(_) => NetworkTestResult {
                        peer_id: peer.id.clone(),
                        address: peer.address.clone(),
                        reachable: false,
                        latency: None,
                        error: Some("Connection timeout".to_string()),
                    },
                }
            }
            Err(e) => NetworkTestResult {
                peer_id: peer.id.clone(),
                address: peer.address.clone(),
                reachable: false,
                latency: None,
                error: Some(format!("Invalid address: {}", e)),
            },
        }
    }
}

impl RateLimiter {
    fn new(max_requests_per_minute: usize) -> Self {
        Self {
            requests: HashMap::new(),
            max_requests_per_minute,
        }
    }

    fn check_rate_limit(&mut self, client_ip: &str) -> bool {
        let now = SystemTime::now();
        let requests = self.requests.entry(client_ip.to_string()).or_default();

        // Remove requests older than 1 minute
        requests.retain(|&time| now.duration_since(time).unwrap_or_default().as_secs() < 60);

        if requests.len() >= self.max_requests_per_minute {
            false
        } else {
            requests.push(now);
            true
        }
    }
}

impl RpcServer {
    /// Create new RPC server with TCP transport
    pub fn new_tcp(
        port: u16,
    ) -> (
        Self,
        mpsc::Receiver<(RpcCommand, tokio::sync::oneshot::Sender<serde_json::Value>)>,
    ) {
        let (command_tx, command_rx) = mpsc::channel(100);

        let server = Self {
            transport: RpcTransport::Tcp(format!("127.0.0.1:{}", port)),
            shutdown_tx: None,
            command_tx,
            network_manager: Arc::new(RwLock::new(NetworkManager::new())),
            node_handle: None,
            node_shutdown_tx: None,
            auth_token: std::env::var("RPC_AUTH_TOKEN").ok(),
            rate_limiter: Arc::new(Mutex::new(RateLimiter::new(60))), // 60 requests per minute
            auth_keys: Arc::new(RwLock::new(HashMap::new())),
            start_time: SystemTime::now(),
        };

        (server, command_rx)
    }

    /// Create new RPC server with Unix socket transport
    pub fn new_unix(
        socket_path: String,
    ) -> (
        Self,
        mpsc::Receiver<(RpcCommand, tokio::sync::oneshot::Sender<serde_json::Value>)>,
    ) {
        let (command_tx, command_rx) = mpsc::channel(100);

        let server = Self {
            transport: RpcTransport::Unix(socket_path),
            shutdown_tx: None,
            command_tx,
            network_manager: Arc::new(RwLock::new(NetworkManager::new())),
            node_handle: None,
            node_shutdown_tx: None,
            auth_token: std::env::var("RPC_AUTH_TOKEN").ok(),
            rate_limiter: Arc::new(Mutex::new(RateLimiter::new(60))),
            auth_keys: Arc::new(RwLock::new(HashMap::new())),
            start_time: SystemTime::now(),
        };

        (server, command_rx)
    }

    /// Create new RPC server with authentication
    pub fn with_auth(
        transport: RpcTransport,
        auth_token: String,
    ) -> (
        Self,
        mpsc::Receiver<(RpcCommand, tokio::sync::oneshot::Sender<serde_json::Value>)>,
    ) {
        let (command_tx, command_rx) = mpsc::channel(100);

        let server = Self {
            transport,
            shutdown_tx: None,
            command_tx,
            network_manager: Arc::new(RwLock::new(NetworkManager::new())),
            node_handle: None,
            node_shutdown_tx: None,
            auth_token: Some(auth_token),
            rate_limiter: Arc::new(Mutex::new(RateLimiter::new(60))),
            auth_keys: Arc::new(RwLock::new(HashMap::new())),
            start_time: SystemTime::now(),
        };

        (server, command_rx)
    }

    /// Set the node handle for real operations
    pub async fn set_node_handle(&mut self, handle: NodeRunnerHandle) {
        self.node_handle = Some(handle.clone());
        let mut manager = self.network_manager.write().await;
        manager.set_node_handle(handle);
    }

    /// Set the shutdown channel for stopping the node
    pub fn set_shutdown_channel(&mut self, tx: oneshot::Sender<()>) {
        self.node_shutdown_tx = Some(tx);
    }

    /// Add an authorized public key for ML-DSA authentication
    pub async fn add_auth_key(&self, client_id: String, public_key: MlDsaPublicKey) {
        let mut keys = self.auth_keys.write().await;
        keys.insert(client_id, public_key);
    }

    /// Start RPC server
    pub async fn start(&mut self) -> Result<(), ProtocolError> {
        let (shutdown_tx, mut shutdown_rx) = tokio::sync::oneshot::channel();
        self.shutdown_tx = Some(shutdown_tx);

        let command_tx = self.command_tx.clone();
        let network_manager = Arc::clone(&self.network_manager);
        // let node = self.node.clone();
        // let dag = self.dag.clone();
        let auth_token = self.auth_token.clone();
        let auth_keys = Arc::clone(&self.auth_keys);
        let rate_limiter = Arc::clone(&self.rate_limiter);
        let transport = self.transport.clone();

        tokio::spawn(async move {
            match transport {
                RpcTransport::Tcp(addr) => {
                    let listener = match TcpListener::bind(&addr).await {
                        Ok(l) => l,
                        Err(e) => {
                            error!("Failed to bind TCP listener: {}", e);
                            return;
                        }
                    };

                    info!(
                        "RPC server listening on TCP: {}",
                        listener.local_addr().unwrap()
                    );

                    loop {
                        tokio::select! {
                            Ok((stream, addr)) = listener.accept() => {
                                debug!("New RPC connection from {}", addr);
                                let command_tx = command_tx.clone();
                                let network_manager = Arc::clone(&network_manager);
                                // let node = node.clone();
                                // let dag = dag.clone();
                                let auth_token = auth_token.clone();
                                let auth_keys = Arc::clone(&auth_keys);
                                let rate_limiter = Arc::clone(&rate_limiter);
                                let client_ip = addr.ip().to_string();

                                tokio::spawn(async move {
                                    // Check rate limit
                                    {
                                        let mut limiter = rate_limiter.lock().await;
                                        if !limiter.check_rate_limit(&client_ip) {
                                            warn!("Rate limit exceeded for client: {}", client_ip);
                                            return;
                                        }
                                    }

                                    if let Err(e) = handle_tcp_connection(
                                        stream, command_tx, network_manager, auth_token, auth_keys
                                    ).await {
                                        error!("Error handling RPC connection: {}", e);
                                    }
                                });
                            }
                            _ = &mut shutdown_rx => {
                                info!("RPC server shutting down");
                                break;
                            }
                        }
                    }
                }
                RpcTransport::Unix(path) => {
                    // Remove existing socket file if it exists
                    let _ = std::fs::remove_file(&path);

                    let listener = match UnixListener::bind(&path) {
                        Ok(l) => l,
                        Err(e) => {
                            error!("Failed to bind Unix listener: {}", e);
                            return;
                        }
                    };

                    info!("RPC server listening on Unix socket: {}", path);

                    loop {
                        tokio::select! {
                            Ok((stream, _)) = listener.accept() => {
                                debug!("New RPC connection on Unix socket");
                                let command_tx = command_tx.clone();
                                let network_manager = Arc::clone(&network_manager);
                                // let node = node.clone();
                                // let dag = dag.clone();
                                let auth_token = auth_token.clone();
                                let auth_keys = Arc::clone(&auth_keys);

                                tokio::spawn(async move {
                                    if let Err(e) = handle_unix_connection(
                                        stream, command_tx, network_manager, auth_token, auth_keys
                                    ).await {
                                        error!("Error handling RPC connection: {}", e);
                                    }
                                });
                            }
                            _ = &mut shutdown_rx => {
                                info!("RPC server shutting down");
                                break;
                            }
                        }
                    }
                }
            }
        });

        Ok(())
    }

    /// Stop RPC server
    pub async fn stop(&mut self) -> Result<(), ProtocolError> {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
        Ok(())
    }
}

/// Handle TCP RPC connection
async fn handle_tcp_connection(
    mut stream: TcpStream,
    command_tx: mpsc::Sender<(RpcCommand, tokio::sync::oneshot::Sender<serde_json::Value>)>,
    network_manager: Arc<RwLock<NetworkManager>>,
    auth_token: Option<String>,
    auth_keys: Arc<RwLock<HashMap<String, MlDsaPublicKey>>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Read request with timeout
    let request_len = timeout(Duration::from_secs(30), ReadU32Ext::read_u32(&mut stream))
        .await??
        .min(10 * 1024 * 1024); // Max 10MB request

    let mut request_data = vec![0u8; request_len as usize];
    timeout(
        Duration::from_secs(30),
        stream.read_exact(&mut request_data),
    )
    .await??;

    let request: RpcRequest = serde_json::from_slice(&request_data)?;

    let response =
        handle_request(request, command_tx, network_manager, auth_token, auth_keys).await;

    let response_data = serde_json::to_vec(&response)?;
    stream
        .write_all(&(response_data.len() as u32).to_be_bytes())
        .await?;
    stream.write_all(&response_data).await?;
    stream.flush().await?;

    Ok(())
}

/// Handle Unix socket RPC connection
async fn handle_unix_connection(
    mut stream: UnixStream,
    command_tx: mpsc::Sender<(RpcCommand, tokio::sync::oneshot::Sender<serde_json::Value>)>,
    network_manager: Arc<RwLock<NetworkManager>>,
    auth_token: Option<String>,
    auth_keys: Arc<RwLock<HashMap<String, MlDsaPublicKey>>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let request_len = timeout(Duration::from_secs(30), ReadU32Ext::read_u32(&mut stream))
        .await??
        .min(10 * 1024 * 1024);

    let mut request_data = vec![0u8; request_len as usize];
    timeout(
        Duration::from_secs(30),
        stream.read_exact(&mut request_data),
    )
    .await??;

    let request: RpcRequest = serde_json::from_slice(&request_data)?;

    let response =
        handle_request(request, command_tx, network_manager, auth_token, auth_keys).await;

    let response_data = serde_json::to_vec(&response)?;
    stream
        .write_all(&(response_data.len() as u32).to_be_bytes())
        .await?;
    stream.write_all(&response_data).await?;
    stream.flush().await?;

    Ok(())
}

/// Authenticate RPC request
async fn authenticate_request(
    request: &RpcRequest,
    auth_token: &Option<String>,
    auth_keys: &Arc<RwLock<HashMap<String, MlDsaPublicKey>>>,
) -> bool {
    // Check token-based auth first
    if let Some(expected_token) = auth_token {
        if let Some(provided_token) = request.params.get("auth_token").and_then(|v| v.as_str()) {
            if provided_token == expected_token {
                return true;
            }
        }
    } else if auth_token.is_none() && auth_keys.read().await.is_empty() {
        // No authentication required if both are empty
        return true;
    }

    // Check ML-DSA signature-based auth
    if let (Some(client_id), Some(signature)) = (
        request.params.get("client_id").and_then(|v| v.as_str()),
        request.params.get("signature").and_then(|v| v.as_str()),
    ) {
        let keys = auth_keys.read().await;
        if let Some(public_key) = keys.get(client_id) {
            // Verify signature over the request method and ID
            let message = format!("{}:{}", request.method, request.id);
            if let Ok(sig_bytes) = hex::decode(signature) {
                if public_key.verify(message.as_bytes(), &sig_bytes).is_ok() {
                    return true;
                }
            }
        }
    }

    false
}

/// Handle RPC request
async fn handle_request(
    request: RpcRequest,
    command_tx: mpsc::Sender<(RpcCommand, tokio::sync::oneshot::Sender<serde_json::Value>)>,
    network_manager: Arc<RwLock<NetworkManager>>,
    auth_token: Option<String>,
    auth_keys: Arc<RwLock<HashMap<String, MlDsaPublicKey>>>,
) -> RpcResponse {
    // Authenticate request if auth is enabled
    if !authenticate_request(&request, &auth_token, &auth_keys).await {
        return RpcResponse {
            id: request.id,
            result: None,
            error: Some(RpcError {
                code: -32001,
                message: "Authentication required".to_string(),
                data: None,
            }),
        };
    }
    match request.method.as_str() {
        "list_peers" => {
            let manager = network_manager.read().await;
            let peers = manager.list_peers().await;
            RpcResponse {
                id: request.id,
                result: Some(serde_json::to_value(peers).unwrap()),
                error: None,
            }
        }
        "add_peer" => {
            let address = match request.params.get("address").and_then(|v| v.as_str()) {
                Some(addr) => addr.to_string(),
                None => {
                    return RpcResponse {
                        id: request.id,
                        result: None,
                        error: Some(RpcError {
                            code: -32602,
                            message: "Invalid params: address required".to_string(),
                            data: None,
                        }),
                    };
                }
            };

            let mut manager = network_manager.write().await;
            match manager.add_peer(address.clone()).await {
                Ok(()) => RpcResponse {
                    id: request.id,
                    result: Some(
                        serde_json::json!({"status": "success", "message": format!("Peer {} added", address)}),
                    ),
                    error: None,
                },
                Err(e) => RpcResponse {
                    id: request.id,
                    result: None,
                    error: Some(RpcError {
                        code: -32003,
                        message: format!("Failed to add peer: {}", e),
                        data: None,
                    }),
                },
            }
        }
        "remove_peer" => {
            let peer_id = match request.params.get("peer_id").and_then(|v| v.as_str()) {
                Some(id) => id,
                None => {
                    return RpcResponse {
                        id: request.id,
                        result: None,
                        error: Some(RpcError {
                            code: -32602,
                            message: "Invalid params: peer_id required".to_string(),
                            data: None,
                        }),
                    };
                }
            };

            let mut manager = network_manager.write().await;
            match manager.remove_peer(peer_id).await {
                Ok(()) => RpcResponse {
                    id: request.id,
                    result: Some(
                        serde_json::json!({"status": "success", "message": format!("Peer {} removed", peer_id)}),
                    ),
                    error: None,
                },
                Err(e) => RpcResponse {
                    id: request.id,
                    result: None,
                    error: Some(RpcError {
                        code: -32003,
                        message: format!("Failed to remove peer: {}", e),
                        data: None,
                    }),
                },
            }
        }
        "get_peer_info" => {
            let peer_id = match request.params.get("peer_id").and_then(|v| v.as_str()) {
                Some(id) => id,
                None => {
                    return RpcResponse {
                        id: request.id,
                        result: None,
                        error: Some(RpcError {
                            code: -32602,
                            message: "Invalid params: peer_id required".to_string(),
                            data: None,
                        }),
                    };
                }
            };

            let manager = network_manager.read().await;
            match manager.get_peer_info(peer_id).await {
                Some(peer_info) => RpcResponse {
                    id: request.id,
                    result: Some(serde_json::to_value(peer_info).unwrap()),
                    error: None,
                },
                None => RpcResponse {
                    id: request.id,
                    result: None,
                    error: Some(RpcError {
                        code: -32004,
                        message: "Peer not found".to_string(),
                        data: None,
                    }),
                },
            }
        }
        "ban_peer" => {
            let peer_id = match request.params.get("peer_id").and_then(|v| v.as_str()) {
                Some(id) => id,
                None => {
                    return RpcResponse {
                        id: request.id,
                        result: None,
                        error: Some(RpcError {
                            code: -32602,
                            message: "Invalid params: peer_id required".to_string(),
                            data: None,
                        }),
                    };
                }
            };

            let mut manager = network_manager.write().await;
            match manager.ban_peer(peer_id).await {
                Ok(()) => RpcResponse {
                    id: request.id,
                    result: Some(
                        serde_json::json!({"status": "success", "message": format!("Peer {} banned", peer_id)}),
                    ),
                    error: None,
                },
                Err(e) => RpcResponse {
                    id: request.id,
                    result: None,
                    error: Some(RpcError {
                        code: -32003,
                        message: format!("Failed to ban peer: {}", e),
                        data: None,
                    }),
                },
            }
        }
        "unban_peer" => {
            let address = match request.params.get("address").and_then(|v| v.as_str()) {
                Some(addr) => addr,
                None => {
                    return RpcResponse {
                        id: request.id,
                        result: None,
                        error: Some(RpcError {
                            code: -32602,
                            message: "Invalid params: address required".to_string(),
                            data: None,
                        }),
                    };
                }
            };

            let mut manager = network_manager.write().await;
            match manager.unban_peer(address) {
                Ok(()) => RpcResponse {
                    id: request.id,
                    result: Some(
                        serde_json::json!({"status": "success", "message": format!("Peer {} unbanned", address)}),
                    ),
                    error: None,
                },
                Err(e) => RpcResponse {
                    id: request.id,
                    result: None,
                    error: Some(RpcError {
                        code: -32003,
                        message: format!("Failed to unban peer: {}", e),
                        data: None,
                    }),
                },
            }
        }
        "get_network_stats" => {
            let mut manager = network_manager.write().await;
            let stats = manager.get_network_stats().await;
            RpcResponse {
                id: request.id,
                result: Some(serde_json::to_value(stats).unwrap()),
                error: None,
            }
        }
        "test_network" => {
            let manager = network_manager.read().await;
            let results = manager.test_network().await;
            RpcResponse {
                id: request.id,
                result: Some(serde_json::to_value(results).unwrap()),
                error: None,
            }
        }
        "stop" => {
            info!("Received stop request via RPC");

            // Try to shutdown the node gracefully through command channel
            let (tx, rx) = tokio::sync::oneshot::channel();
            if let Err(_) = command_tx.send((RpcCommand::Stop, tx)).await {
                return RpcResponse {
                    id: request.id,
                    result: None,
                    error: Some(RpcError {
                        code: -1,
                        message: "Failed to send stop command".to_string(),
                        data: None,
                    }),
                };
            }

            match rx.await {
                Ok(result) => RpcResponse {
                    id: request.id,
                    result: Some(result),
                    error: None,
                },
                Err(_) => RpcResponse {
                    id: request.id,
                    result: None,
                    error: Some(RpcError {
                        code: -1,
                        message: "Command execution failed".to_string(),
                        data: None,
                    }),
                },
            }
        }
        "get_status" => {
            // Try to get status from real node if available
            let mut manager = network_manager.write().await;

            // Check if we have a node handle to get real status from
            let real_status = if let Some(node) = &manager.node_handle {
                let node_guard = node.read().await;
                match node_guard.get_status().await {
                    Ok(status) => Some(status),
                    Err(e) => {
                        warn!("Failed to get real node status: {}", e);
                        None
                    }
                }
            } else {
                None
            };

            // If we got real status, use it; otherwise build mock status
            let result = if let Some(status) = real_status {
                status
            } else {
                // Build mock status
                let mut status = NodeStatus {
                    node_id: "node_mock".to_string(),
                    state: "Mock".to_string(),
                    uptime: 0,
                    peers: vec![],
                    network_stats: NetworkStats {
                        total_connections: 0,
                        active_connections: 0,
                        messages_sent: 0,
                        messages_received: 0,
                        bytes_sent: 0,
                        bytes_received: 0,
                        average_latency: 0.0,
                        uptime: 0,
                    },
                    dag_stats: DagStats {
                        vertex_count: 0,
                        edge_count: 0,
                        tip_count: 0,
                        finalized_height: 0,
                        pending_transactions: 0,
                    },
                    memory_usage: MemoryStats {
                        total_allocated: 0,
                        current_usage: 0,
                        peak_usage: 0,
                    },
                };

                // Get mock network stats
                status.peers = manager.list_peers().await;
                status.network_stats = manager.get_network_stats().await;
                status.uptime = manager.start_time.elapsed().unwrap_or_default().as_secs();

                // Get memory stats
                #[cfg(target_os = "linux")]
                {
                    if let Ok(contents) = std::fs::read_to_string("/proc/self/status") {
                        for line in contents.lines() {
                            if line.starts_with("VmRSS:") {
                                if let Some(kb_str) = line.split_whitespace().nth(1) {
                                    if let Ok(kb) = kb_str.parse::<usize>() {
                                        status.memory_usage.current_usage = kb * 1024;
                                    }
                                }
                            } else if line.starts_with("VmPeak:") {
                                if let Some(kb_str) = line.split_whitespace().nth(1) {
                                    if let Ok(kb) = kb_str.parse::<usize>() {
                                        status.memory_usage.peak_usage = kb * 1024;
                                    }
                                }
                            }
                        }
                    }
                }

                serde_json::to_value(status).unwrap()
            };

            RpcResponse {
                id: request.id,
                result: Some(result),
                error: None,
            }
        }
        _ => RpcResponse {
            id: request.id,
            result: None,
            error: Some(RpcError {
                code: -32601,
                message: format!("Method '{}' not found", request.method),
                data: None,
            }),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rpc_request_serialization() {
        let request = RpcRequest {
            id: Uuid::new_v4(),
            method: "stop".to_string(),
            params: serde_json::Value::Null,
        };

        let serialized = serde_json::to_string(&request).unwrap();
        let deserialized: RpcRequest = serde_json::from_str(&serialized).unwrap();

        assert_eq!(request.method, deserialized.method);
    }

    #[tokio::test]
    async fn test_network_manager_peer_operations() {
        let mut manager = NetworkManager::new();

        // Test adding peer
        assert!(manager.add_peer("127.0.0.1:8001".to_string()).await.is_ok());
        assert_eq!(manager.list_peers().await.len(), 1);

        // Test adding duplicate peer (should work)
        assert!(manager.add_peer("127.0.0.1:8002".to_string()).await.is_ok());
        assert_eq!(manager.list_peers().await.len(), 2);

        // Test getting peer info
        let peers = manager.list_peers().await;
        let peer_id = peers[0].id.clone();
        assert!(manager.get_peer_info(&peer_id).await.is_some());

        // Test removing peer
        assert!(manager.remove_peer(&peer_id).await.is_ok());
        assert_eq!(manager.list_peers().await.len(), 1);

        // Test removing non-existent peer
        assert!(manager.remove_peer("invalid_id").await.is_err());
    }

    #[tokio::test]
    async fn test_network_manager_ban_operations() {
        let mut manager = NetworkManager::new();

        // Add a peer
        manager
            .add_peer("127.0.0.1:8001".to_string())
            .await
            .unwrap();
        let peer_id = manager.list_peers().await[0].id.clone();

        // Ban the peer
        assert!(manager.ban_peer(&peer_id).await.is_ok());
        assert_eq!(manager.list_peers().await.len(), 0); // Should be removed

        // Try to add the same address again (should fail)
        assert!(manager
            .add_peer("127.0.0.1:8001".to_string())
            .await
            .is_err());

        // Unban the peer
        assert!(manager.unban_peer("127.0.0.1:8001").is_ok());

        // Now adding should work again
        assert!(manager.add_peer("127.0.0.1:8001".to_string()).await.is_ok());
    }

    #[test]
    fn test_rate_limiter() {
        let mut limiter = RateLimiter::new(2); // 2 requests per minute

        // First two requests should pass
        assert!(limiter.check_rate_limit("127.0.0.1"));
        assert!(limiter.check_rate_limit("127.0.0.1"));

        // Third request should fail
        assert!(!limiter.check_rate_limit("127.0.0.1"));

        // Different IP should work
        assert!(limiter.check_rate_limit("127.0.0.2"));
    }

    #[tokio::test]
    async fn test_authenticate_request() {
        let request_with_token = RpcRequest {
            id: Uuid::new_v4(),
            method: "test".to_string(),
            params: serde_json::json!({ "auth_token": "secret123" }),
        };

        let request_without_token = RpcRequest {
            id: Uuid::new_v4(),
            method: "test".to_string(),
            params: serde_json::Value::Null,
        };

        let auth_keys = Arc::new(RwLock::new(HashMap::new()));

        // Test with auth enabled
        let auth_token = Some("secret123".to_string());
        assert!(authenticate_request(&request_with_token, &auth_token, &auth_keys).await);
        assert!(!authenticate_request(&request_without_token, &auth_token, &auth_keys).await);

        // Test with auth disabled
        let no_auth = None;
        assert!(authenticate_request(&request_with_token, &no_auth, &auth_keys).await);
        assert!(authenticate_request(&request_without_token, &no_auth, &auth_keys).await);
    }

    #[tokio::test]
    async fn test_rpc_server_creation() {
        let (server, _rx) = RpcServer::new_tcp(0); // Port 0 for automatic assignment
        match server.transport {
            RpcTransport::Tcp(addr) => assert!(addr.contains(":0")),
            _ => panic!("Expected TCP transport"),
        }
    }

    #[tokio::test]
    async fn test_rpc_server_with_auth() {
        let (server, _rx) = RpcServer::with_auth(
            RpcTransport::Tcp("127.0.0.1:0".to_string()),
            "secret123".to_string(),
        );
        assert_eq!(server.auth_token, Some("secret123".to_string()));
    }

    #[tokio::test]
    async fn test_network_test_functionality() {
        let manager = NetworkManager::new();
        let results = manager.test_network().await;
        assert!(results.is_empty()); // No peers to test
    }

    #[tokio::test]
    async fn test_network_stats() {
        let mut manager = NetworkManager::new();
        let stats = manager.get_network_stats().await;

        assert_eq!(stats.total_connections, 0);
        assert_eq!(stats.active_connections, 0);
        assert_eq!(stats.messages_sent, 0);
        assert_eq!(stats.messages_received, 0);

        // Add a peer and check stats update
        manager
            .add_peer("127.0.0.1:8001".to_string())
            .await
            .unwrap();
        let updated_stats = manager.get_network_stats().await;
        assert_eq!(updated_stats.total_connections, 1);
        assert_eq!(updated_stats.active_connections, 1);
    }

    #[test]
    fn test_peer_info_serialization() {
        let peer_info = PeerInfo {
            id: "test_peer".to_string(),
            address: "127.0.0.1:8001".to_string(),
            connected_duration: 300,
            messages_sent: 10,
            messages_received: 15,
            last_seen: 1234567890,
            status: "Connected".to_string(),
            latency: Some(25.5),
        };

        let serialized = serde_json::to_string(&peer_info).unwrap();
        let deserialized: PeerInfo = serde_json::from_str(&serialized).unwrap();

        assert_eq!(peer_info.id, deserialized.id);
        assert_eq!(peer_info.address, deserialized.address);
        assert_eq!(peer_info.status, deserialized.status);
        assert_eq!(peer_info.latency, deserialized.latency);
    }

    #[test]
    fn test_network_stats_serialization() {
        let stats = NetworkStats {
            total_connections: 5,
            active_connections: 3,
            messages_sent: 100,
            messages_received: 95,
            bytes_sent: 1024,
            bytes_received: 2048,
            average_latency: 15.7,
            uptime: 3600,
        };

        let serialized = serde_json::to_string(&stats).unwrap();
        let deserialized: NetworkStats = serde_json::from_str(&serialized).unwrap();

        assert_eq!(stats.total_connections, deserialized.total_connections);
        assert_eq!(stats.active_connections, deserialized.active_connections);
        assert_eq!(stats.uptime, deserialized.uptime);
    }

    #[test]
    fn test_rpc_error_codes() {
        // Test standard JSON-RPC error codes
        let method_not_found = RpcError {
            code: -32601,
            message: "Method not found".to_string(),
            data: None,
        };

        let invalid_params = RpcError {
            code: -32602,
            message: "Invalid params".to_string(),
            data: None,
        };

        let auth_required = RpcError {
            code: -32001,
            message: "Authentication required".to_string(),
            data: None,
        };

        assert_eq!(method_not_found.code, -32601);
        assert_eq!(invalid_params.code, -32602);
        assert_eq!(auth_required.code, -32001);
    }
}
