//! Transport layer implementations for MCP

use crate::error::{Error, Result};
use crate::protocol::MCPMessage;
use async_trait::async_trait;
use futures::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_util::codec::{FramedRead, FramedWrite, LinesCodec};

/// Trait for MCP transport implementations
#[async_trait]
pub trait Transport: Send + Sync {
    /// Send a message through the transport
    async fn send(&mut self, message: MCPMessage) -> Result<()>;

    /// Receive a message from the transport
    async fn receive(&mut self) -> Result<Option<MCPMessage>>;

    /// Close the transport connection
    async fn close(&mut self) -> Result<()>;

    /// Check if the transport is still connected
    fn is_connected(&self) -> bool;
}

/// Standard I/O transport for local subprocess communication
pub struct StdioTransport {
    sender: FramedWrite<tokio::io::Stdout, LinesCodec>,
    receiver: FramedRead<tokio::io::Stdin, LinesCodec>,
    connected: bool,
}

impl StdioTransport {
    pub fn new() -> Self {
        let stdin = tokio::io::stdin();
        let stdout = tokio::io::stdout();

        let receiver = FramedRead::new(stdin, LinesCodec::new());
        let sender = FramedWrite::new(stdout, LinesCodec::new());

        Self {
            sender,
            receiver,
            connected: true,
        }
    }
}

#[async_trait]
impl Transport for StdioTransport {
    async fn send(&mut self, message: MCPMessage) -> Result<()> {
        if !self.connected {
            return Err(Error::connection_lost());
        }

        let json = message.to_json()?;
        self.sender
            .send(json)
            .await
            .map_err(|e| Error::transport("stdio", format!("Failed to send message: {}", e)))?;

        // Ensure the message is flushed to stdout
        <FramedWrite<tokio::io::Stdout, LinesCodec> as SinkExt<String>>::flush(&mut self.sender)
            .await
            .map_err(|e| Error::transport("stdio", format!("Failed to flush message: {}", e)))?;

        Ok(())
    }

    async fn receive(&mut self) -> Result<Option<MCPMessage>> {
        if !self.connected {
            return Err(Error::connection_lost());
        }

        match self.receiver.next().await {
            Some(Ok(line)) => {
                let message = MCPMessage::from_json(&line)?;
                Ok(Some(message))
            }
            Some(Err(e)) => Err(Error::transport(
                "stdio",
                format!("Failed to receive message: {}", e),
            )),
            None => {
                // No more input available at the moment, but connection might still be open
                // Return None to indicate no message available rather than closing connection
                Ok(None)
            }
        }
    }

    async fn close(&mut self) -> Result<()> {
        self.connected = false;
        SinkExt::<String>::close(&mut self.sender)
            .await
            .map_err(|e| Error::transport("stdio", format!("Failed to close transport: {}", e)))?;
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }
}

/// HTTP transport for web-based communication
pub struct HttpTransport {
    client: reqwest::Client,
    server_url: String,
    session_id: Option<String>,
    connected: bool,
}

impl HttpTransport {
    pub fn new(server_url: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            server_url: server_url.into(),
            session_id: None,
            connected: false,
        }
    }

    pub async fn connect(&mut self) -> Result<()> {
        // Establish HTTP connection with server
        let connect_url = format!("{}/mcp/connect", self.server_url);

        let response = self
            .client
            .post(&connect_url)
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(|e| Error::transport("http", format!("Failed to connect to server: {}", e)))?;

        if response.status().is_success() {
            self.session_id = response
                .headers()
                .get("session-id")
                .and_then(|h| h.to_str().ok())
                .map(String::from);
            self.connected = true;
            Ok(())
        } else {
            Err(Error::transport(
                "http",
                format!("Server returned error: {}", response.status()),
            ))
        }
    }
}

#[async_trait]
impl Transport for HttpTransport {
    async fn send(&mut self, message: MCPMessage) -> Result<()> {
        if !self.connected {
            return Err(Error::connection_lost());
        }

        let json = message.to_json()?;
        let url = format!("{}/mcp/message", self.server_url);

        let mut request = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .body(json);

        if let Some(session_id) = &self.session_id {
            request = request.header("session-id", session_id);
        }

        let response = request
            .send()
            .await
            .map_err(|e| Error::transport("http", format!("Failed to send message: {}", e)))?;

        if !response.status().is_success() {
            return Err(Error::transport(
                "http",
                format!("Server returned error: {}", response.status()),
            ));
        }

        Ok(())
    }

    async fn receive(&mut self) -> Result<Option<MCPMessage>> {
        if !self.connected {
            return Err(Error::connection_lost());
        }

        let url = format!("{}/mcp/receive", self.server_url);
        let mut request = self.client.get(&url);

        if let Some(session_id) = &self.session_id {
            request = request.header("session-id", session_id);
        }

        let response = request
            .send()
            .await
            .map_err(|e| Error::transport("http", format!("Failed to receive message: {}", e)))?;

        if response.status().is_success() {
            let text = response
                .text()
                .await
                .map_err(|e| Error::transport("http", format!("Failed to read response: {}", e)))?;

            if text.is_empty() {
                Ok(None)
            } else {
                let message = MCPMessage::from_json(&text)?;
                Ok(Some(message))
            }
        } else if response.status() == 204 {
            // No content - no messages available
            Ok(None)
        } else {
            Err(Error::transport(
                "http",
                format!("Server returned error: {}", response.status()),
            ))
        }
    }

    async fn close(&mut self) -> Result<()> {
        if self.connected {
            let url = format!("{}/mcp/disconnect", self.server_url);
            let mut request = self.client.post(&url);

            if let Some(session_id) = &self.session_id {
                request = request.header("session-id", session_id);
            }

            let _ = request.send().await; // Ignore errors during disconnect
        }

        self.connected = false;
        self.session_id = None;
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }
}

/// WebSocket transport for real-time bidirectional communication
pub struct WebSocketTransport {
    sender: mpsc::UnboundedSender<MCPMessage>,
    receiver: mpsc::UnboundedReceiver<MCPMessage>,
    connected: Arc<std::sync::atomic::AtomicBool>,
}

impl WebSocketTransport {
    pub async fn connect(url: impl AsRef<str>) -> Result<Self> {
        let (ws_stream, _) = tokio_tungstenite::connect_async(url.as_ref())
            .await
            .map_err(|e| {
                Error::transport("websocket", format!("Failed to connect WebSocket: {}", e))
            })?;

        let (ws_sender, ws_receiver) = ws_stream.split();
        let (tx_out, rx_out) = mpsc::unbounded_channel();
        let (tx_in, rx_in) = mpsc::unbounded_channel();
        let connected = Arc::new(std::sync::atomic::AtomicBool::new(true));

        // Spawn task to handle outgoing messages
        let connected_out = connected.clone();
        tokio::spawn(async move {
            let mut ws_sender = ws_sender;
            let mut rx_out: mpsc::UnboundedReceiver<MCPMessage> = rx_out;

            while let Some(message) = rx_out.recv().await {
                if let Ok(json) = message.to_json() {
                    let ws_msg = tokio_tungstenite::tungstenite::Message::Text(json);
                    if ws_sender.send(ws_msg).await.is_err() {
                        break;
                    }
                }
            }

            connected_out.store(false, std::sync::atomic::Ordering::Relaxed);
        });

        // Spawn task to handle incoming messages
        let connected_in = connected.clone();
        tokio::spawn(async move {
            let mut ws_receiver = ws_receiver;
            let tx_in = tx_in;

            while let Some(msg_result) = ws_receiver.next().await {
                match msg_result {
                    Ok(tokio_tungstenite::tungstenite::Message::Text(text)) => {
                        if let Ok(message) = MCPMessage::from_json(&text) {
                            if tx_in.send(message).is_err() {
                                break;
                            }
                        }
                    }
                    Ok(tokio_tungstenite::tungstenite::Message::Close(_)) => {
                        break;
                    }
                    Err(_) => {
                        break;
                    }
                    _ => {
                        // Ignore other message types
                    }
                }
            }

            connected_in.store(false, std::sync::atomic::Ordering::Relaxed);
        });

        Ok(Self {
            sender: tx_out,
            receiver: rx_in,
            connected,
        })
    }
}

#[async_trait]
impl Transport for WebSocketTransport {
    async fn send(&mut self, message: MCPMessage) -> Result<()> {
        if !self.is_connected() {
            return Err(Error::connection_lost());
        }

        self.sender
            .send(message)
            .map_err(|_| Error::connection_lost())?;
        Ok(())
    }

    async fn receive(&mut self) -> Result<Option<MCPMessage>> {
        if !self.is_connected() {
            return Err(Error::connection_lost());
        }

        match self.receiver.recv().await {
            Some(message) => Ok(Some(message)),
            None => {
                self.connected
                    .store(false, std::sync::atomic::Ordering::Relaxed);
                Ok(None)
            }
        }
    }

    async fn close(&mut self) -> Result<()> {
        self.connected
            .store(false, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected.load(std::sync::atomic::Ordering::Relaxed)
    }
}

/// Transport configuration
#[derive(Debug, Clone)]
pub enum TransportConfig {
    Stdio,
    Http { server_url: String },
    WebSocket { url: String },
}

impl TransportConfig {
    pub async fn create_transport(&self) -> Result<Box<dyn Transport>> {
        match self {
            TransportConfig::Stdio => Ok(Box::new(StdioTransport::new())),
            TransportConfig::Http { server_url: _ } => {
                // For HTTP server mode, create a mock transport that indicates
                // the server should start an HTTP server instead
                Ok(Box::new(HttpServerTransport::new()))
            }
            TransportConfig::WebSocket { url: _ } => {
                // For WebSocket server mode, create a mock transport
                Ok(Box::new(WebSocketServerTransport::new()))
            }
        }
    }
}

/// Transport factory for creating transport instances
pub struct TransportFactory;

impl TransportFactory {
    pub fn stdio() -> TransportConfig {
        TransportConfig::Stdio
    }

    pub fn http(server_url: impl Into<String>) -> TransportConfig {
        TransportConfig::Http {
            server_url: server_url.into(),
        }
    }

    pub fn websocket(url: impl Into<String>) -> TransportConfig {
        TransportConfig::WebSocket { url: url.into() }
    }
}

/// Mock HTTP server transport for server mode
pub struct HttpServerTransport {
    connected: bool,
    message_queue: std::collections::VecDeque<MCPMessage>,
}

impl HttpServerTransport {
    pub fn new() -> Self {
        Self {
            connected: true,
            message_queue: std::collections::VecDeque::new(),
        }
    }
}

#[async_trait]
impl Transport for HttpServerTransport {
    async fn send(&mut self, _message: MCPMessage) -> Result<()> {
        // In server mode, sending is handled by HTTP responses
        Ok(())
    }

    async fn receive(&mut self) -> Result<Option<MCPMessage>> {
        // In server mode, keep the transport alive by returning None
        // Actual message handling is done by the HTTP server
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        Ok(None)
    }

    async fn close(&mut self) -> Result<()> {
        self.connected = false;
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }
}

/// Mock WebSocket server transport for server mode
pub struct WebSocketServerTransport {
    connected: bool,
}

impl WebSocketServerTransport {
    pub fn new() -> Self {
        Self { connected: true }
    }
}

#[async_trait]
impl Transport for WebSocketServerTransport {
    async fn send(&mut self, _message: MCPMessage) -> Result<()> {
        // In server mode, sending is handled by WebSocket connections
        Ok(())
    }

    async fn receive(&mut self) -> Result<Option<MCPMessage>> {
        // In server mode, keep the transport alive by returning None
        // Actual message handling is done by the WebSocket server
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        Ok(None)
    }

    async fn close(&mut self) -> Result<()> {
        self.connected = false;
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::MCPRequest;
    use crate::types::ClientInfo;
    use std::collections::HashMap;

    #[test]
    fn test_transport_config() {
        let stdio_config = TransportFactory::stdio();
        assert!(matches!(stdio_config, TransportConfig::Stdio));

        let http_config = TransportFactory::http("http://localhost:8080");
        assert!(matches!(http_config, TransportConfig::Http { .. }));

        let ws_config = TransportFactory::websocket("ws://localhost:8080/mcp");
        assert!(matches!(ws_config, TransportConfig::WebSocket { .. }));
    }

    #[tokio::test]
    async fn test_stdio_transport_creation() {
        let config = TransportFactory::stdio();

        // Note: We can't easily test actual stdio communication in unit tests
        // This test just verifies the transport can be created
        let transport_result = config.create_transport().await;
        assert!(transport_result.is_ok());

        let mut transport = transport_result.unwrap();
        assert!(transport.is_connected());
    }

    #[test]
    fn test_message_serialization_for_transport() {
        let request =
            MCPRequest::initialize(ClientInfo::new("test-client", "1.0.0"), HashMap::new());
        let message = MCPMessage::Request(request);

        let json = message.to_json().unwrap();
        let parsed = MCPMessage::from_json(&json).unwrap();

        assert!(parsed.is_request());
        assert_eq!(parsed.method(), Some("initialize"));
    }

    #[tokio::test]
    async fn test_transport_error_handling() {
        // Test connection lost error
        let mut transport = StdioTransport::new();
        transport.connected = false;

        let request = MCPRequest::initialize(ClientInfo::new("test", "1.0.0"), HashMap::new());
        let message = MCPMessage::Request(request);

        let result = transport.send(message).await;
        assert!(
            matches!(result, Err(Error::Transport { transport_type, .. }) if transport_type == "connection")
        );
    }
}
