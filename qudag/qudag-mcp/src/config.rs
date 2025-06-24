//! Configuration management for QuDAG MCP integration.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

use crate::error::{Error, Result};

/// Main configuration for MCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    /// Server binding configuration
    pub server: ServerConfig,

    /// Authentication configuration
    pub auth: AuthConfig,

    /// Security configuration
    pub security: SecurityConfig,

    /// Rate limiting configuration
    pub rate_limit: RateLimitConfig,

    /// Audit logging configuration
    pub audit: AuditConfig,

    /// Storage configuration
    pub storage: StorageConfig,
}

/// Server binding and network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Server bind address
    pub host: String,

    /// Server port
    pub port: u16,

    /// Maximum concurrent connections
    pub max_connections: usize,

    /// Request timeout
    pub request_timeout: Duration,

    /// Keep-alive timeout
    pub keep_alive_timeout: Duration,

    /// Enable TLS
    pub tls_enabled: bool,

    /// TLS certificate path
    pub tls_cert_path: Option<PathBuf>,

    /// TLS private key path
    pub tls_key_path: Option<PathBuf>,
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// Vault file path for credential storage
    pub vault_path: PathBuf,

    /// JWT secret key (base64 encoded)
    pub jwt_secret: String,

    /// JWT token expiration time
    pub jwt_expiration: Duration,

    /// Enable multi-factor authentication
    pub mfa_enabled: bool,

    /// MFA token validity duration
    pub mfa_token_duration: Duration,

    /// Enable role-based access control
    pub rbac_enabled: bool,

    /// Session timeout
    pub session_timeout: Duration,

    /// Maximum failed login attempts
    pub max_login_attempts: u32,

    /// Login attempt lockout duration
    pub lockout_duration: Duration,
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Encryption key size in bytes
    pub encryption_key_size: usize,

    /// Enable request signing
    pub request_signing: bool,

    /// Enable response encryption
    pub response_encryption: bool,

    /// Minimum password strength score (0-100)
    pub min_password_strength: u8,

    /// Enable secure headers
    pub secure_headers: bool,

    /// CORS allowed origins
    pub cors_origins: Vec<String>,

    /// Content Security Policy
    pub csp_policy: Option<String>,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Enable rate limiting
    pub enabled: bool,

    /// Maximum requests per window
    pub max_requests: usize,

    /// Time window for rate limiting
    pub window_duration: Duration,

    /// Burst allowance
    pub burst_size: usize,

    /// Rate limit storage backend
    pub storage_backend: RateLimitStorage,
}

/// Rate limiting storage backend options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RateLimitStorage {
    /// In-memory storage (not persistent)
    Memory,
    /// Redis storage (requires Redis connection)
    Redis { url: String },
    /// Database storage
    Database,
}

/// Audit logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfig {
    /// Enable audit logging
    pub enabled: bool,

    /// Audit log file path
    pub log_file: PathBuf,

    /// Log rotation size in bytes
    pub rotation_size: u64,

    /// Number of rotated files to keep
    pub rotation_count: u32,

    /// Log retention duration
    pub retention_duration: Duration,

    /// Events to audit
    pub audit_events: Vec<AuditEvent>,

    /// Include request/response bodies in logs
    pub include_bodies: bool,

    /// Structured logging format
    pub structured_format: bool,
}

/// Types of events to audit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditEvent {
    /// Authentication attempts
    Authentication,
    /// Authorization decisions
    Authorization,
    /// MCP resource access
    ResourceAccess,
    /// Configuration changes
    ConfigChange,
    /// Security events
    Security,
    /// All events
    All,
}

/// Storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Base directory for storage
    pub base_dir: PathBuf,

    /// Database URL for persistent storage
    pub database_url: Option<String>,

    /// Enable encryption at rest
    pub encrypt_at_rest: bool,

    /// Backup configuration
    pub backup: BackupConfig,
}

/// Backup configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupConfig {
    /// Enable automatic backups
    pub enabled: bool,

    /// Backup interval
    pub interval: Duration,

    /// Backup directory
    pub directory: PathBuf,

    /// Number of backups to retain
    pub retain_count: u32,

    /// Compress backup files
    pub compress: bool,
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            auth: AuthConfig::default(),
            security: SecurityConfig::default(),
            rate_limit: RateLimitConfig::default(),
            audit: AuditConfig::default(),
            storage: StorageConfig::default(),
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            max_connections: 1000,
            request_timeout: Duration::from_secs(30),
            keep_alive_timeout: Duration::from_secs(60),
            tls_enabled: false,
            tls_cert_path: None,
            tls_key_path: None,
        }
    }
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            vault_path: PathBuf::from("mcp_vault.qdag"),
            jwt_secret: "your-256-bit-secret".to_string(),
            jwt_expiration: Duration::from_secs(crate::defaults::JWT_EXPIRATION),
            mfa_enabled: false,
            mfa_token_duration: Duration::from_secs(300), // 5 minutes
            rbac_enabled: true,
            session_timeout: Duration::from_secs(3600), // 1 hour
            max_login_attempts: 5,
            lockout_duration: Duration::from_secs(900), // 15 minutes
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            encryption_key_size: crate::defaults::ENCRYPTION_KEY_SIZE as usize,
            request_signing: true,
            response_encryption: true,
            min_password_strength: 70,
            secure_headers: true,
            cors_origins: vec!["http://localhost:3000".to_string()],
            csp_policy: Some("default-src 'self'".to_string()),
        }
    }
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_requests: crate::defaults::RATE_LIMIT_COUNT as usize,
            window_duration: Duration::from_secs(crate::defaults::RATE_LIMIT_WINDOW),
            burst_size: 10,
            storage_backend: RateLimitStorage::Memory,
        }
    }
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            log_file: PathBuf::from("mcp_audit.log"),
            rotation_size: 100 * 1024 * 1024, // 100MB
            rotation_count: 10,
            retention_duration: Duration::from_secs(
                crate::defaults::AUDIT_RETENTION_DAYS * 24 * 3600,
            ),
            audit_events: vec![
                AuditEvent::Authentication,
                AuditEvent::Authorization,
                AuditEvent::Security,
            ],
            include_bodies: false,
            structured_format: true,
        }
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            base_dir: PathBuf::from("mcp_data"),
            database_url: None,
            encrypt_at_rest: true,
            backup: BackupConfig::default(),
        }
    }
}

impl Default for BackupConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval: Duration::from_secs(24 * 3600), // Daily
            directory: PathBuf::from("mcp_backups"),
            retain_count: 7,
            compress: true,
        }
    }
}

impl McpConfig {
    /// Load configuration from file
    pub fn from_file(path: impl AsRef<std::path::Path>) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)
            .map_err(|e| Error::config(format!("Failed to parse config: {}", e)))?;
        config.validate()?;
        Ok(config)
    }

    /// Save configuration to file
    pub fn save_to_file(&self, path: impl AsRef<std::path::Path>) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| Error::config(format!("Failed to serialize config: {}", e)))?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        // Validate server config
        if self.server.port == 0 {
            return Err(Error::config("Invalid server port"));
        }

        // Validate JWT secret
        if self.auth.jwt_secret.is_empty() {
            return Err(Error::config("JWT secret cannot be empty"));
        }

        // Validate vault path
        if !self.auth.vault_path.parent().map_or(true, |p| p.is_dir()) {
            return Err(Error::config("Vault directory does not exist"));
        }

        // Validate storage directory
        if !self.storage.base_dir.exists() {
            std::fs::create_dir_all(&self.storage.base_dir)?;
        }

        // Validate backup directory if backups are enabled
        if self.storage.backup.enabled && !self.storage.backup.directory.exists() {
            std::fs::create_dir_all(&self.storage.backup.directory)?;
        }

        Ok(())
    }

    /// Create configuration with builder pattern
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::new()
    }

    /// Enable TLS with certificate paths
    pub fn with_tls(mut self, cert_path: PathBuf, key_path: PathBuf) -> Self {
        self.server.tls_enabled = true;
        self.server.tls_cert_path = Some(cert_path);
        self.server.tls_key_path = Some(key_path);
        self
    }

    /// Set vault path
    pub fn with_vault_path(mut self, path: PathBuf) -> Self {
        self.auth.vault_path = path;
        self
    }

    /// Set JWT secret
    pub fn with_jwt_secret(mut self, secret: String) -> Self {
        self.auth.jwt_secret = secret;
        self
    }

    /// Enable/disable MFA
    pub fn with_mfa(mut self, enabled: bool) -> Self {
        self.auth.mfa_enabled = enabled;
        self
    }

    /// Set rate limiting
    pub fn with_rate_limiting(mut self, max_requests: usize, window: Duration) -> Self {
        self.rate_limit.enabled = true;
        self.rate_limit.max_requests = max_requests;
        self.rate_limit.window_duration = window;
        self
    }

    /// Enable/disable audit logging
    pub fn with_audit_logging(mut self, enabled: bool) -> Self {
        self.audit.enabled = enabled;
        self
    }
}

/// Configuration builder for fluent API
pub struct ConfigBuilder {
    config: McpConfig,
}

impl ConfigBuilder {
    /// Create new builder with default config
    pub fn new() -> Self {
        Self {
            config: McpConfig::default(),
        }
    }

    /// Set server bind address
    pub fn bind(mut self, host: impl Into<String>, port: u16) -> Self {
        self.config.server.host = host.into();
        self.config.server.port = port;
        self
    }

    /// Set vault path
    pub fn vault_path(mut self, path: PathBuf) -> Self {
        self.config.auth.vault_path = path;
        self
    }

    /// Set JWT configuration
    pub fn jwt(mut self, secret: String, expiration: Duration) -> Self {
        self.config.auth.jwt_secret = secret;
        self.config.auth.jwt_expiration = expiration;
        self
    }

    /// Enable TLS
    pub fn tls(mut self, cert_path: PathBuf, key_path: PathBuf) -> Self {
        self.config.server.tls_enabled = true;
        self.config.server.tls_cert_path = Some(cert_path);
        self.config.server.tls_key_path = Some(key_path);
        self
    }

    /// Configure rate limiting
    pub fn rate_limit(mut self, max_requests: usize, window: Duration) -> Self {
        self.config.rate_limit.enabled = true;
        self.config.rate_limit.max_requests = max_requests;
        self.config.rate_limit.window_duration = window;
        self
    }

    /// Enable MFA
    pub fn mfa(mut self, enabled: bool) -> Self {
        self.config.auth.mfa_enabled = enabled;
        self
    }

    /// Configure audit logging
    pub fn audit(mut self, enabled: bool, log_file: PathBuf) -> Self {
        self.config.audit.enabled = enabled;
        self.config.audit.log_file = log_file;
        self
    }

    /// Build the configuration
    pub fn build(self) -> Result<McpConfig> {
        self.config.validate()?;
        Ok(self.config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile;

    #[test]
    fn test_default_config() {
        let config = McpConfig::default();
        assert_eq!(config.server.port, 8080);
        assert!(config.rate_limit.enabled);
        assert!(config.audit.enabled);
    }

    #[test]
    fn test_config_builder() {
        let config = McpConfig::builder()
            .bind("0.0.0.0", 9090)
            .vault_path(PathBuf::from("test.qdag"))
            .jwt("test-secret".to_string(), Duration::from_secs(7200))
            .rate_limit(50, Duration::from_secs(60))
            .mfa(true)
            .build()
            .unwrap();

        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 9090);
        assert_eq!(config.auth.vault_path, PathBuf::from("test.qdag"));
        assert_eq!(config.auth.jwt_secret, "test-secret");
        assert_eq!(config.auth.jwt_expiration, Duration::from_secs(7200));
        assert_eq!(config.rate_limit.max_requests, 50);
        assert!(config.auth.mfa_enabled);
    }

    #[test]
    fn test_config_file_operations() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("test_config.toml");

        let config = McpConfig::default();
        config.save_to_file(&config_path).unwrap();

        let loaded_config = McpConfig::from_file(&config_path).unwrap();
        assert_eq!(config.server.port, loaded_config.server.port);
    }

    #[test]
    fn test_config_validation() {
        let mut config = McpConfig::default();
        config.server.port = 0;

        assert!(config.validate().is_err());

        config.server.port = 8080;
        config.auth.jwt_secret.clear();

        assert!(config.validate().is_err());
    }
}
