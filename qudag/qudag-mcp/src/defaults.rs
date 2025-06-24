//! Default configuration values for QuDAG MCP

/// Default JWT expiration time in seconds (24 hours)
pub const JWT_EXPIRATION: u64 = 86400;

/// Default encryption key size in bits
pub const ENCRYPTION_KEY_SIZE: u32 = 256;

/// Default rate limit count per window
pub const RATE_LIMIT_COUNT: u32 = 1000;

/// Default rate limit window duration in seconds
pub const RATE_LIMIT_WINDOW: u64 = 60;

/// Default audit log retention in days
pub const AUDIT_RETENTION_DAYS: u64 = 30;

/// Default server port
pub const DEFAULT_SERVER_PORT: u16 = 8080;

/// Default maximum concurrent connections
pub const MAX_CONCURRENT_CONNECTIONS: usize = 1000;

/// Default request timeout in seconds
pub const REQUEST_TIMEOUT_SECONDS: u64 = 30;

/// Default buffer size for message handling
pub const MESSAGE_BUFFER_SIZE: usize = 1024;

/// Default maximum message size in bytes
pub const MAX_MESSAGE_SIZE: usize = 1024 * 1024; // 1MB
