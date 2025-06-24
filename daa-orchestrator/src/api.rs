//! API server for monitoring and control

use tracing::{info, debug};

use crate::config::ApiConfig;
use crate::error::{OrchestratorError, Result};

/// API server for external monitoring and control
pub struct ApiServer {
    config: ApiConfig,
    running: bool,
    request_count: u64,
}

impl ApiServer {
    /// Create a new API server
    pub async fn new(config: ApiConfig) -> Result<Self> {
        Ok(Self {
            config,
            running: false,
            request_count: 0,
        })
    }

    /// Initialize the API server
    pub async fn initialize(&mut self) -> Result<()> {
        if !self.config.enabled {
            info!("API server is disabled");
            return Ok(());
        }

        info!("Initializing API server");
        debug!("API server will bind to {}:{}", self.config.bind_address, self.config.port);
        
        // Mock initialization
        info!("API server initialized");
        Ok(())
    }

    /// Start the API server
    pub async fn start(&mut self) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        info!("Starting API server on {}:{}", self.config.bind_address, self.config.port);
        
        // Mock server start
        self.running = true;
        self.request_count = 0;
        
        info!("API server started");
        Ok(())
    }

    /// Stop the API server
    pub async fn stop(&mut self) -> Result<()> {
        info!("Stopping API server");
        self.running = false;
        info!("API server stopped after handling {} requests", self.request_count);
        Ok(())
    }

    /// Health check for API server
    pub async fn health_check(&self) -> Result<bool> {
        if !self.config.enabled {
            return Ok(true);
        }

        debug!("API server health check");
        Ok(self.running)
    }

    /// Get server status
    pub fn get_status(&self) -> ApiServerStatus {
        ApiServerStatus {
            enabled: self.config.enabled,
            running: self.running,
            port: self.config.port,
            request_count: self.request_count,
            cors_enabled: self.config.enable_cors,
        }
    }

    /// Handle API request (mock implementation)
    pub async fn handle_request(&mut self, _path: &str, _method: &str) -> Result<ApiResponse> {
        if !self.running {
            return Err(OrchestratorError::ApiError("Server not running".to_string()));
        }

        self.request_count += 1;
        debug!("Handling API request #{}", self.request_count);

        // Mock response
        Ok(ApiResponse {
            status_code: 200,
            body: serde_json::json!({
                "status": "ok",
                "message": "Mock API response",
                "request_count": self.request_count
            }),
            headers: vec![("Content-Type".to_string(), "application/json".to_string())],
        })
    }
}

/// API response structure
#[derive(Debug, Clone)]
pub struct ApiResponse {
    pub status_code: u16,
    pub body: serde_json::Value,
    pub headers: Vec<(String, String)>,
}

/// API server status
#[derive(Debug, Clone)]
pub struct ApiServerStatus {
    pub enabled: bool,
    pub running: bool,
    pub port: u16,
    pub request_count: u64,
    pub cors_enabled: bool,
}

impl std::fmt::Display for ApiServerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.enabled {
            write!(
                f,
                "API Server: {} on port {} ({} requests, CORS: {})",
                if self.running { "Running" } else { "Stopped" },
                self.port,
                self.request_count,
                self.cors_enabled
            )
        } else {
            write!(f, "API Server: Disabled")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_api_server_creation() {
        let config = ApiConfig::default();
        let server = ApiServer::new(config).await;
        assert!(server.is_ok());
    }

    #[tokio::test]
    async fn test_api_server_lifecycle() {
        let config = ApiConfig::default();
        let mut server = ApiServer::new(config).await.unwrap();
        
        assert!(!server.get_status().running);
        
        server.initialize().await.unwrap();
        server.start().await.unwrap();
        
        assert!(server.get_status().running);
        assert!(server.health_check().await.unwrap());
        
        server.stop().await.unwrap();
        assert!(!server.get_status().running);
    }

    #[tokio::test]
    async fn test_disabled_server() {
        let mut config = ApiConfig::default();
        config.enabled = false;
        
        let mut server = ApiServer::new(config).await.unwrap();
        server.initialize().await.unwrap();
        server.start().await.unwrap();
        
        assert!(!server.get_status().running);
        assert!(server.health_check().await.unwrap()); // Should be healthy when disabled
    }

    #[tokio::test]
    async fn test_request_handling() {
        let config = ApiConfig::default();
        let mut server = ApiServer::new(config).await.unwrap();
        
        server.initialize().await.unwrap();
        server.start().await.unwrap();
        
        let response = server.handle_request("/status", "GET").await.unwrap();
        assert_eq!(response.status_code, 200);
        assert_eq!(server.get_status().request_count, 1);
    }
}