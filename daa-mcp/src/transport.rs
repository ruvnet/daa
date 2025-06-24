//! Transport layer implementations for MCP communication
//! 
//! This module provides different transport mechanisms for MCP communication
//! including HTTP, WebSocket, and STDIO transports.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;
use tokio::sync::{broadcast, RwLock};
use tracing::{debug, error, info, warn};

use crate::{DaaMcpError, McpMessage, Result};

/// Transport trait for MCP communication
#[async_trait]
pub trait McpTransport: Send + Sync {
    /// Send a message through the transport
    async fn send_message(&self, message: McpMessage) -> Result<()>;
    
    /// Receive a message from the transport
    async fn receive_message(&self) -> Result<Option<McpMessage>>;
    
    /// Check if the transport is connected
    fn is_connected(&self) -> bool;
    
    /// Close the transport connection
    async fn close(&self) -> Result<()>;
    
    /// Get transport type identifier
    fn transport_type(&self) -> &'static str;
}

/// HTTP transport for MCP communication
pub struct HttpTransport {
    endpoint: String,
    client: reqwest::Client,
    connected: bool,
}

impl HttpTransport {
    pub fn new(endpoint: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            endpoint,
            client,
            connected: false,
        }
    }

    pub async fn connect(&mut self) -> Result<()> {
        // Test connection with a simple request
        let health_check = reqwest::get(&format!("{}/health", self.endpoint)).await;
        
        match health_check {
            Ok(response) if response.status().is_success() => {
                self.connected = true;
                info!("HTTP transport connected to {}", self.endpoint);
                Ok(())
            }
            Ok(response) => {
                error!("HTTP transport connection failed: {}", response.status());
                Err(DaaMcpError::Network(std::io::Error::new(
                    std::io::ErrorKind::ConnectionRefused,
                    format!("HTTP error: {}", response.status()),
                )))
            }
            Err(e) => {
                error!("HTTP transport connection error: {}", e);
                Err(DaaMcpError::Network(std::io::Error::new(
                    std::io::ErrorKind::ConnectionRefused,
                    e.to_string(),
                )))
            }
        }
    }
}

#[async_trait]
impl McpTransport for HttpTransport {
    async fn send_message(&self, message: McpMessage) -> Result<()> {
        if !self.connected {
            return Err(DaaMcpError::Network(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "HTTP transport not connected",
            )));
        }

        let url = format!("{}/mcp", self.endpoint);
        let response = self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&message)
            .send()
            .await
            .map_err(|e| DaaMcpError::Network(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        if !response.status().is_success() {
            return Err(DaaMcpError::Network(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("HTTP error: {}", response.status()),
            )));
        }

        debug!("HTTP message sent successfully");
        Ok(())
    }

    async fn receive_message(&self) -> Result<Option<McpMessage>> {
        // HTTP is request-response, so this would typically be handled differently
        // For now, return None to indicate no message available
        Ok(None)
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    async fn close(&self) -> Result<()> {
        info!("Closing HTTP transport connection");
        Ok(())
    }

    fn transport_type(&self) -> &'static str {
        "http"
    }
}

/// WebSocket transport for MCP communication
pub struct WebSocketTransport {
    endpoint: String,
    sender: Option<broadcast::Sender<McpMessage>>,
    receiver: Option<broadcast::Receiver<McpMessage>>,
    connected: Arc<RwLock<bool>>,
}

impl WebSocketTransport {
    pub fn new(endpoint: String) -> Self {
        let (sender, receiver) = broadcast::channel(100);
        
        Self {
            endpoint,
            sender: Some(sender),
            receiver: Some(receiver),
            connected: Arc::new(RwLock::new(false)),
        }
    }

    pub async fn connect(&mut self) -> Result<()> {
        use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

        let ws_url = if self.endpoint.starts_with("http://") {
            self.endpoint.replace("http://", "ws://")
        } else if self.endpoint.starts_with("https://") {
            self.endpoint.replace("https://", "wss://")
        } else {
            format!("ws://{}", self.endpoint)
        };

        let url = format!("{}/mcp/ws", ws_url);
        
        match connect_async(&url).await {
            Ok((ws_stream, _)) => {
                let (ws_sender, mut ws_receiver) = ws_stream.split();
                let sender = self.sender.take().unwrap();
                let connected = self.connected.clone();
                
                // Set connected status
                *connected.write().await = true;
                
                // Spawn receiver task
                let receiver_connected = connected.clone();
                tokio::spawn(async move {
                    while let Some(msg) = ws_receiver.next().await {
                        match msg {
                            Ok(Message::Text(text)) => {
                                match serde_json::from_str::<McpMessage>(&text) {
                                    Ok(mcp_message) => {
                                        if sender.send(mcp_message).is_err() {
                                            break;
                                        }
                                    }
                                    Err(e) => {
                                        error!("Failed to parse WebSocket message: {}", e);
                                    }
                                }
                            }
                            Ok(Message::Close(_)) => {
                                info!("WebSocket connection closed by server");
                                *receiver_connected.write().await = false;
                                break;
                            }
                            Err(e) => {
                                error!("WebSocket receive error: {}", e);
                                *receiver_connected.write().await = false;
                                break;
                            }
                            _ => {}
                        }
                    }
                });

                info!("WebSocket transport connected to {}", url);
                Ok(())
            }
            Err(e) => {
                error!("WebSocket connection failed: {}", e);
                Err(DaaMcpError::Network(std::io::Error::new(
                    std::io::ErrorKind::ConnectionRefused,
                    e.to_string(),
                )))
            }
        }
    }
}

#[async_trait]
impl McpTransport for WebSocketTransport {
    async fn send_message(&self, message: McpMessage) -> Result<()> {
        // In a real implementation, this would send through the WebSocket
        // For now, we'll just log the message
        debug!("WebSocket sending message: {:?}", message);
        Ok(())
    }

    async fn receive_message(&self) -> Result<Option<McpMessage>> {
        if let Some(ref mut receiver) = self.receiver.as_ref() {
            match receiver.try_recv() {
                Ok(message) => Ok(Some(message)),
                Err(broadcast::error::TryRecvError::Empty) => Ok(None),
                Err(broadcast::error::TryRecvError::Closed) => {
                    Err(DaaMcpError::Network(std::io::Error::new(
                        std::io::ErrorKind::ConnectionAborted,
                        "WebSocket channel closed",
                    )))
                }
                Err(broadcast::error::TryRecvError::Lagged(_)) => {
                    warn!("WebSocket receiver lagged, some messages may be lost");
                    Ok(None)
                }
            }
        } else {
            Ok(None)
        }
    }

    fn is_connected(&self) -> bool {
        // Use try_read to avoid blocking
        self.connected.try_read().map(|guard| *guard).unwrap_or(false)
    }

    async fn close(&self) -> Result<()> {
        *self.connected.write().await = false;
        info!("WebSocket transport connection closed");
        Ok(())
    }

    fn transport_type(&self) -> &'static str {
        "websocket"
    }
}

/// STDIO transport for MCP communication (for local processes)
pub struct StdioTransport {
    connected: bool,
    message_queue: Arc<RwLock<Vec<McpMessage>>>,
}

impl StdioTransport {
    pub fn new() -> Self {
        Self {
            connected: false,
            message_queue: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn connect(&mut self) -> Result<()> {
        // For STDIO, we just mark as connected
        self.connected = true;
        info!("STDIO transport connected");
        Ok(())
    }
}

#[async_trait]
impl McpTransport for StdioTransport {
    async fn send_message(&self, message: McpMessage) -> Result<()> {
        if !self.connected {
            return Err(DaaMcpError::Network(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "STDIO transport not connected",
            )));
        }

        // In a real implementation, this would write to stdout
        let json = serde_json::to_string(&message)?;
        println!("{}", json);
        debug!("STDIO message sent: {}", json);
        Ok(())
    }

    async fn receive_message(&self) -> Result<Option<McpMessage>> {
        // In a real implementation, this would read from stdin
        // For now, check the message queue
        let mut queue = self.message_queue.write().await;
        Ok(queue.pop())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    async fn close(&self) -> Result<()> {
        info!("STDIO transport connection closed");
        Ok(())
    }

    fn transport_type(&self) -> &'static str {
        "stdio"
    }
}

/// Transport factory for creating different transport types
pub struct TransportFactory;

impl TransportFactory {
    /// Create a transport based on the endpoint URL
    pub fn create_transport(endpoint: &str) -> Result<Box<dyn McpTransport>> {
        if endpoint.starts_with("http://") || endpoint.starts_with("https://") {
            Ok(Box::new(HttpTransport::new(endpoint.to_string())))
        } else if endpoint.starts_with("ws://") || endpoint.starts_with("wss://") {
            Ok(Box::new(WebSocketTransport::new(endpoint.to_string())))
        } else if endpoint == "stdio" {
            Ok(Box::new(StdioTransport::new()))
        } else {
            Err(DaaMcpError::Protocol(format!("Unsupported transport: {}", endpoint)))
        }
    }

    /// Create an HTTP transport
    pub fn create_http(endpoint: String) -> HttpTransport {
        HttpTransport::new(endpoint)
    }

    /// Create a WebSocket transport
    pub fn create_websocket(endpoint: String) -> WebSocketTransport {
        WebSocketTransport::new(endpoint)
    }

    /// Create a STDIO transport
    pub fn create_stdio() -> StdioTransport {
        StdioTransport::new()
    }
}

/// Transport manager for handling multiple transports
pub struct TransportManager {
    transports: HashMap<String, Box<dyn McpTransport>>,
    default_transport: Option<String>,
}

impl TransportManager {
    pub fn new() -> Self {
        Self {
            transports: HashMap::new(),
            default_transport: None,
        }
    }

    /// Add a transport with a given name
    pub fn add_transport(&mut self, name: String, transport: Box<dyn McpTransport>) {
        if self.default_transport.is_none() {
            self.default_transport = Some(name.clone());
        }
        self.transports.insert(name, transport);
    }

    /// Get a transport by name
    pub fn get_transport(&self, name: &str) -> Option<&dyn McpTransport> {
        self.transports.get(name).map(|t| t.as_ref())
    }

    /// Get the default transport
    pub fn get_default_transport(&self) -> Option<&dyn McpTransport> {
        if let Some(ref name) = self.default_transport {
            self.get_transport(name)
        } else {
            None
        }
    }

    /// Send a message through a specific transport
    pub async fn send_message(&self, transport_name: &str, message: McpMessage) -> Result<()> {
        if let Some(transport) = self.transports.get(transport_name) {
            transport.send_message(message).await
        } else {
            Err(DaaMcpError::Protocol(format!("Transport not found: {}", transport_name)))
        }
    }

    /// Send a message through the default transport
    pub async fn send_message_default(&self, message: McpMessage) -> Result<()> {
        if let Some(ref name) = self.default_transport {
            self.send_message(name, message).await
        } else {
            Err(DaaMcpError::Protocol("No default transport configured".to_string()))
        }
    }

    /// Get status of all transports
    pub fn get_transport_status(&self) -> HashMap<String, TransportStatus> {
        self.transports.iter()
            .map(|(name, transport)| {
                (name.clone(), TransportStatus {
                    name: name.clone(),
                    transport_type: transport.transport_type().to_string(),
                    connected: transport.is_connected(),
                })
            })
            .collect()
    }

    /// Close all transports
    pub async fn close_all(&self) -> Result<()> {
        for (name, transport) in &self.transports {
            if let Err(e) = transport.close().await {
                error!("Failed to close transport {}: {}", name, e);
            }
        }
        Ok(())
    }
}

/// Transport status information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TransportStatus {
    pub name: String,
    pub transport_type: String,
    pub connected: bool,
}

/// Configuration for transport creation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TransportConfig {
    pub transport_type: String,
    pub endpoint: String,
    pub timeout: Option<u64>,
    pub retry_attempts: Option<u32>,
    pub retry_delay: Option<u64>,
}

impl TransportConfig {
    pub fn http(endpoint: String) -> Self {
        Self {
            transport_type: "http".to_string(),
            endpoint,
            timeout: Some(30),
            retry_attempts: Some(3),
            retry_delay: Some(1000),
        }
    }

    pub fn websocket(endpoint: String) -> Self {
        Self {
            transport_type: "websocket".to_string(),
            endpoint,
            timeout: Some(30),
            retry_attempts: Some(5),
            retry_delay: Some(2000),
        }
    }

    pub fn stdio() -> Self {
        Self {
            transport_type: "stdio".to_string(),
            endpoint: "stdio".to_string(),
            timeout: None,
            retry_attempts: None,
            retry_delay: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_transport_factory() {
        let http_transport = TransportFactory::create_transport("http://localhost:3001").unwrap();
        assert_eq!(http_transport.transport_type(), "http");

        let ws_transport = TransportFactory::create_transport("ws://localhost:3001").unwrap();
        assert_eq!(ws_transport.transport_type(), "websocket");

        let stdio_transport = TransportFactory::create_transport("stdio").unwrap();
        assert_eq!(stdio_transport.transport_type(), "stdio");
    }

    #[tokio::test]
    async fn test_transport_manager() {
        let mut manager = TransportManager::new();
        
        let http_transport = TransportFactory::create_http("http://localhost:3001".to_string());
        manager.add_transport("http".to_string(), Box::new(http_transport));

        let status = manager.get_transport_status();
        assert_eq!(status.len(), 1);
        assert!(status.contains_key("http"));
    }

    #[test]
    fn test_transport_config() {
        let http_config = TransportConfig::http("http://localhost:3001".to_string());
        assert_eq!(http_config.transport_type, "http");
        assert_eq!(http_config.timeout, Some(30));

        let ws_config = TransportConfig::websocket("ws://localhost:3001".to_string());
        assert_eq!(ws_config.transport_type, "websocket");

        let stdio_config = TransportConfig::stdio();
        assert_eq!(stdio_config.transport_type, "stdio");
        assert_eq!(stdio_config.endpoint, "stdio");
    }
}