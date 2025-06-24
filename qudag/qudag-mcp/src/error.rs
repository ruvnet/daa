//! Error types for the QuDAG MCP server
//!
//! This module provides comprehensive error handling for all MCP operations,
//! including proper error propagation from QuDAG components and MCP-specific errors.

use std::fmt;
use thiserror::Error;

/// Result type alias for QuDAG MCP operations
pub type Result<T> = std::result::Result<T, Error>;

/// Main error type for QuDAG MCP operations
#[derive(Error, Debug)]
pub enum Error {
    /// Configuration errors
    #[error("Configuration error: {message}")]
    Config {
        /// The error message
        message: String,
        /// Optional source error
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Transport layer errors
    #[error("Transport error: {transport_type}: {message}")]
    Transport {
        /// Type of transport that failed
        transport_type: String,
        /// Error message
        message: String,
        /// Optional source error
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// MCP protocol errors
    #[error("Protocol error: {message}")]
    Protocol {
        /// Error message
        message: String,
        /// Optional error code
        code: Option<i32>,
        /// Optional source error
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Authentication and authorization errors
    #[error("Authentication error: {message}")]
    Auth {
        /// Error message
        message: String,
        /// Optional source error
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Vault operation errors
    #[error("Vault error: {operation}: {message}")]
    Vault {
        /// The vault operation that failed
        operation: String,
        /// Error message
        message: String,
        /// Optional source error
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// QuDAG integration errors
    #[error("QuDAG error: {component}: {message}")]
    QuDAG {
        /// QuDAG component that failed
        component: String,
        /// Error message
        message: String,
        /// Optional source error
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Resource management errors
    #[error("Resource error: {resource_type}: {message}")]
    Resource {
        /// Type of resource
        resource_type: String,
        /// Error message
        message: String,
        /// Optional source error
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Tool execution errors
    #[error("Tool error: {tool_name}: {message}")]
    Tool {
        /// Name of the tool that failed
        tool_name: String,
        /// Error message
        message: String,
        /// Optional source error
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Serialization/deserialization errors
    #[error("Serialization error: {message}")]
    Serialization {
        /// Error message
        message: String,
        /// Optional source error
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Network-related errors
    #[error("Network error: {message}")]
    Network {
        /// Error message
        message: String,
        /// Optional source error
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// I/O errors
    #[error("I/O error: {message}")]
    Io {
        /// Error message
        message: String,
        /// Optional source error
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Internal server errors
    #[error("Internal error: {message}")]
    Internal {
        /// Error message
        message: String,
        /// Optional source error
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

impl Error {
    /// Create a new configuration error
    pub fn config<S: Into<String>>(message: S) -> Self {
        Self::Config {
            message: message.into(),
            source: None,
        }
    }

    /// Create a new configuration error with source
    pub fn config_with_source<S: Into<String>, E>(message: S, source: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::Config {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    /// Create a new transport error
    pub fn transport<T: Into<String>, M: Into<String>>(transport_type: T, message: M) -> Self {
        Self::Transport {
            transport_type: transport_type.into(),
            message: message.into(),
            source: None,
        }
    }

    /// Create a new transport error with source
    pub fn transport_with_source<T: Into<String>, M: Into<String>, E>(
        transport_type: T,
        message: M,
        source: E,
    ) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::Transport {
            transport_type: transport_type.into(),
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    /// Create a new protocol error
    pub fn protocol<S: Into<String>>(message: S) -> Self {
        Self::Protocol {
            message: message.into(),
            code: None,
            source: None,
        }
    }

    /// Create a new protocol error with code
    pub fn protocol_with_code<S: Into<String>>(message: S, code: i32) -> Self {
        Self::Protocol {
            message: message.into(),
            code: Some(code),
            source: None,
        }
    }

    /// Create a new authentication error
    pub fn auth<S: Into<String>>(message: S) -> Self {
        Self::Auth {
            message: message.into(),
            source: None,
        }
    }

    /// Create a new vault error
    pub fn vault<O: Into<String>, M: Into<String>>(operation: O, message: M) -> Self {
        Self::Vault {
            operation: operation.into(),
            message: message.into(),
            source: None,
        }
    }

    /// Create a new vault error with source
    pub fn vault_with_source<O: Into<String>, M: Into<String>, E>(
        operation: O,
        message: M,
        source: E,
    ) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::Vault {
            operation: operation.into(),
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    /// Create a new QuDAG error
    pub fn qudag<C: Into<String>, M: Into<String>>(component: C, message: M) -> Self {
        Self::QuDAG {
            component: component.into(),
            message: message.into(),
            source: None,
        }
    }

    /// Create a new resource error
    pub fn resource<R: Into<String>, M: Into<String>>(resource_type: R, message: M) -> Self {
        Self::Resource {
            resource_type: resource_type.into(),
            message: message.into(),
            source: None,
        }
    }

    /// Create a new tool error
    pub fn tool<T: Into<String>, M: Into<String>>(tool_name: T, message: M) -> Self {
        Self::Tool {
            tool_name: tool_name.into(),
            message: message.into(),
            source: None,
        }
    }

    /// Create a new internal error
    pub fn internal<S: Into<String>>(message: S) -> Self {
        Self::Internal {
            message: message.into(),
            source: None,
        }
    }

    /// Create a connection lost error
    pub fn connection_lost() -> Self {
        Self::Transport {
            transport_type: "connection".to_string(),
            message: "Connection lost".to_string(),
            source: None,
        }
    }

    /// Create a method not found error
    pub fn method_not_found<S: Into<String>>(method: S) -> Self {
        Self::Protocol {
            message: format!("Method not found: {}", method.into()),
            code: Some(-32601),
            source: None,
        }
    }

    /// Create an invalid params error
    pub fn invalid_params<S: Into<String>>(message: S) -> Self {
        Self::Protocol {
            message: format!("Invalid params: {}", message.into()),
            code: Some(-32602),
            source: None,
        }
    }

    /// Create an invalid request error
    pub fn invalid_request<S: Into<String>>(message: S) -> Self {
        Self::Protocol {
            message: format!("Invalid request: {}", message.into()),
            code: Some(-32600),
            source: None,
        }
    }

    /// Create a parse error
    pub fn parse_error<S: Into<String>>(message: S) -> Self {
        Self::Serialization {
            message: format!("Parse error: {}", message.into()),
            source: None,
        }
    }

    /// Create a serialization error
    pub fn serialization_error<S: Into<String>>(message: S) -> Self {
        Self::Serialization {
            message: message.into(),
            source: None,
        }
    }

    /// Create a tool not found error
    pub fn tool_not_found<S: Into<String>>(tool_name: S) -> Self {
        Self::Tool {
            tool_name: tool_name.into(),
            message: "Tool not found".to_string(),
            source: None,
        }
    }

    /// Create a prompt not found error
    pub fn prompt_not_found<S: Into<String>>(name: S) -> Self {
        Self::Protocol {
            message: format!("Prompt not found: {}", name.into()),
            code: Some(-32601),
            source: None,
        }
    }

    /// Create an unsupported protocol version error
    pub fn unsupported_protocol_version<S: Into<String>>(version: S) -> Self {
        Self::Protocol {
            message: format!("Unsupported protocol version: {}", version.into()),
            code: Some(-32600),
            source: None,
        }
    }

    /// Get the error code for MCP protocol responses
    pub fn error_code(&self) -> i32 {
        match self {
            Self::Config { .. } => -32600,    // Invalid Request
            Self::Transport { .. } => -32603, // Internal Error
            Self::Protocol {
                code: Some(code), ..
            } => *code,
            Self::Protocol { .. } => -32602,      // Invalid Params
            Self::Auth { .. } => -32001,          // Unauthorized
            Self::Vault { .. } => -32002,         // Vault Error
            Self::QuDAG { .. } => -32003,         // QuDAG Error
            Self::Resource { .. } => -32004,      // Resource Error
            Self::Tool { .. } => -32005,          // Tool Error
            Self::Serialization { .. } => -32700, // Parse Error
            Self::Network { .. } => -32603,       // Internal Error
            Self::Io { .. } => -32603,            // Internal Error
            Self::Internal { .. } => -32603,      // Internal Error
        }
    }

    /// Check if this error should be retried
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::Network { .. } | Self::Transport { .. } | Self::Io { .. }
        )
    }

    /// Check if this error is related to authentication
    pub fn is_auth_error(&self) -> bool {
        matches!(self, Self::Auth { .. })
    }

    /// Check if this error is a client error (4xx equivalent)
    pub fn is_client_error(&self) -> bool {
        matches!(
            self,
            Self::Config { .. } | Self::Protocol { .. } | Self::Auth { .. }
        )
    }

    /// Check if this error is a server error (5xx equivalent)
    pub fn is_server_error(&self) -> bool {
        matches!(
            self,
            Self::Transport { .. }
                | Self::Vault { .. }
                | Self::QuDAG { .. }
                | Self::Resource { .. }
                | Self::Tool { .. }
                | Self::Network { .. }
                | Self::Io { .. }
                | Self::Internal { .. }
        )
    }

    /// Convert to JSON-RPC error format
    pub fn to_json_rpc_error(&self) -> serde_json::Value {
        serde_json::json!({
            "code": self.error_code(),
            "message": self.to_string(),
            "data": {
                "error_type": self.error_type_name()
            }
        })
    }

    /// Get the error type name as a string
    fn error_type_name(&self) -> &'static str {
        match self {
            Self::Config { .. } => "Config",
            Self::Transport { .. } => "Transport",
            Self::Protocol { .. } => "Protocol",
            Self::Auth { .. } => "Auth",
            Self::Vault { .. } => "Vault",
            Self::QuDAG { .. } => "QuDAG",
            Self::Resource { .. } => "Resource",
            Self::Tool { .. } => "Tool",
            Self::Serialization { .. } => "Serialization",
            Self::Network { .. } => "Network",
            Self::Io { .. } => "Io",
            Self::Internal { .. } => "Internal",
        }
    }
}

// Standard library error conversions
impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::Io {
            message: err.to_string(),
            source: Some(Box::new(err)),
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::Serialization {
            message: format!("JSON serialization error: {}", err),
            source: Some(Box::new(err)),
        }
    }
}

impl From<toml::de::Error> for Error {
    fn from(err: toml::de::Error) -> Self {
        Self::Serialization {
            message: format!("TOML deserialization error: {}", err),
            source: Some(Box::new(err)),
        }
    }
}

impl From<toml::ser::Error> for Error {
    fn from(err: toml::ser::Error) -> Self {
        Self::Serialization {
            message: format!("TOML serialization error: {}", err),
            source: Some(Box::new(err)),
        }
    }
}

impl From<url::ParseError> for Error {
    fn from(err: url::ParseError) -> Self {
        Self::Config {
            message: format!("URL parse error: {}", err),
            source: Some(Box::new(err)),
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Self::Network {
            message: format!("HTTP request error: {}", err),
            source: Some(Box::new(err)),
        }
    }
}

impl From<tungstenite::Error> for Error {
    fn from(err: tungstenite::Error) -> Self {
        Self::Transport {
            transport_type: "WebSocket".to_string(),
            message: format!("WebSocket error: {}", err),
            source: Some(Box::new(err)),
        }
    }
}

// QuDAG vault error integration
impl From<qudag_vault_core::error::VaultError> for Error {
    fn from(err: qudag_vault_core::error::VaultError) -> Self {
        Self::Vault {
            operation: "vault_operation".to_string(),
            message: err.to_string(),
            source: Some(Box::new(err)),
        }
    }
}

impl From<std::time::SystemTimeError> for Error {
    fn from(err: std::time::SystemTimeError) -> Self {
        Self::Internal {
            message: format!("System time error: {}", err),
            source: Some(Box::new(err)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = Error::config("test configuration error");
        assert!(matches!(err, Error::Config { .. }));
        assert_eq!(err.error_code(), -32600);
        assert!(err.is_client_error());
        assert!(!err.is_server_error());
    }

    #[test]
    fn test_error_with_source() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err = Error::config_with_source("config file missing", io_err);
        assert!(err.source().is_some());
    }

    #[test]
    fn test_retryable_errors() {
        let network_err = Error::Network {
            message: "connection timeout".to_string(),
            source: None,
        };
        assert!(network_err.is_retryable());

        let config_err = Error::config("invalid config");
        assert!(!config_err.is_retryable());
    }

    #[test]
    fn test_error_codes() {
        assert_eq!(Error::config("test").error_code(), -32600);
        assert_eq!(Error::auth("test").error_code(), -32001);
        assert_eq!(Error::vault("op", "test").error_code(), -32002);
    }

    #[test]
    fn test_error_conversions() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
        let mcp_err: Error = io_err.into();
        assert!(matches!(mcp_err, Error::Io { .. }));
    }
}
