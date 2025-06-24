//! Authentication and authorization for QuDAG MCP.

use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Authentication manager
#[derive(Debug, Clone)]
pub struct AuthManager {
    /// Authentication configuration
    config: AuthConfig,
    /// Active sessions
    sessions: HashMap<String, AuthSession>,
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// Whether authentication is required
    pub required: bool,
    /// Supported authentication methods
    pub methods: Vec<AuthMethod>,
    /// Session timeout in seconds
    pub session_timeout: u64,
    /// Maximum concurrent sessions
    pub max_sessions: usize,
}

/// Authentication method
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthMethod {
    /// API key authentication
    #[serde(rename = "api_key")]
    ApiKey,
    /// OAuth2 authentication
    #[serde(rename = "oauth2")]
    OAuth2,
    /// Vault token authentication
    #[serde(rename = "vault_token")]
    VaultToken,
    /// No authentication
    #[serde(rename = "none")]
    None,
}

/// Authentication session
#[derive(Debug, Clone)]
pub struct AuthSession {
    /// Session ID
    pub id: String,
    /// User ID
    pub user_id: String,
    /// Authentication method used
    pub method: AuthMethod,
    /// Session creation time
    pub created_at: SystemTime,
    /// Session last activity
    pub last_activity: SystemTime,
    /// Session permissions
    pub permissions: Vec<Permission>,
}

/// Permission type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Permission {
    /// Read DAG data
    DagRead,
    /// Write DAG data
    DagWrite,
    /// Read vault data
    VaultRead,
    /// Write vault data
    VaultWrite,
    /// Network operations
    NetworkAccess,
    /// Crypto operations
    CryptoAccess,
    /// Admin operations
    Admin,
}

/// Authentication request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthRequest {
    /// Authentication method
    pub method: AuthMethod,
    /// Credentials
    pub credentials: HashMap<String, String>,
    /// Client information
    pub client_info: Option<crate::types::ClientInfo>,
}

/// Authentication response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    /// Whether authentication was successful
    pub success: bool,
    /// Session token (if successful)
    pub session_token: Option<String>,
    /// Error message (if failed)
    pub error: Option<String>,
    /// User permissions
    pub permissions: Option<Vec<String>>,
}

impl AuthManager {
    /// Create new authentication manager
    pub fn new(config: AuthConfig) -> Self {
        Self {
            config,
            sessions: HashMap::new(),
        }
    }

    /// Authenticate a request
    pub async fn authenticate(&mut self, request: AuthRequest) -> Result<AuthResponse> {
        if !self.config.required || request.method == AuthMethod::None {
            // No authentication required or explicitly disabled
            return Ok(AuthResponse {
                success: true,
                session_token: None,
                error: None,
                permissions: Some(vec!["read".to_string(), "write".to_string()]),
            });
        }

        match request.method {
            AuthMethod::ApiKey => self.authenticate_api_key(&request.credentials).await,
            AuthMethod::OAuth2 => self.authenticate_oauth2(&request.credentials).await,
            AuthMethod::VaultToken => self.authenticate_vault_token(&request.credentials).await,
            AuthMethod::None => Ok(AuthResponse {
                success: true,
                session_token: None,
                error: None,
                permissions: Some(vec!["read".to_string()]),
            }),
        }
    }

    /// Validate a session token
    pub async fn validate_session(&mut self, token: &str) -> Result<Option<&AuthSession>> {
        if let Some(session) = self.sessions.get(token) {
            // Check if session is expired
            let now = SystemTime::now();
            let age = now.duration_since(session.created_at)?;

            if age.as_secs() > self.config.session_timeout {
                // Session expired, remove it
                self.sessions.remove(token);
                return Ok(None);
            }

            // Update last activity
            if let Some(session) = self.sessions.get_mut(token) {
                session.last_activity = now;
            }

            Ok(self.sessions.get(token))
        } else {
            Ok(None)
        }
    }

    /// Check if user has permission
    pub fn has_permission(&self, session_token: &str, permission: Permission) -> bool {
        if let Some(session) = self.sessions.get(session_token) {
            session.permissions.contains(&permission)
        } else {
            false
        }
    }

    /// Create a new session
    fn create_session(
        &mut self,
        user_id: String,
        method: AuthMethod,
        permissions: Vec<Permission>,
    ) -> Result<String> {
        // Check max sessions
        if self.sessions.len() >= self.config.max_sessions {
            return Err(Error::auth("Maximum concurrent sessions reached"));
        }

        let session_id = uuid::Uuid::new_v4().to_string();
        let now = SystemTime::now();

        let session = AuthSession {
            id: session_id.clone(),
            user_id,
            method,
            created_at: now,
            last_activity: now,
            permissions,
        };

        self.sessions.insert(session_id.clone(), session);
        Ok(session_id)
    }

    /// Authenticate with API key
    async fn authenticate_api_key(
        &mut self,
        credentials: &HashMap<String, String>,
    ) -> Result<AuthResponse> {
        let api_key = credentials
            .get("api_key")
            .ok_or_else(|| Error::auth("API key not provided"))?;

        // In a real implementation, you would validate the API key against a database
        // For now, we'll accept any non-empty key
        if api_key.is_empty() {
            return Ok(AuthResponse {
                success: false,
                session_token: None,
                error: Some("Invalid API key".to_string()),
                permissions: None,
            });
        }

        let user_id = format!(
            "api_key_user_{}",
            api_key.chars().take(8).collect::<String>()
        );
        let permissions = vec![
            Permission::DagRead,
            Permission::DagWrite,
            Permission::VaultRead,
            Permission::NetworkAccess,
            Permission::CryptoAccess,
        ];

        let session_token = self.create_session(user_id, AuthMethod::ApiKey, permissions)?;

        Ok(AuthResponse {
            success: true,
            session_token: Some(session_token),
            error: None,
            permissions: Some(vec![
                "dag_read".to_string(),
                "dag_write".to_string(),
                "vault_read".to_string(),
                "network_access".to_string(),
                "crypto_access".to_string(),
            ]),
        })
    }

    /// Authenticate with OAuth2
    async fn authenticate_oauth2(
        &mut self,
        credentials: &HashMap<String, String>,
    ) -> Result<AuthResponse> {
        let access_token = credentials
            .get("access_token")
            .ok_or_else(|| Error::auth("Access token not provided"))?;

        // In a real implementation, you would validate the token with the OAuth2 provider
        // For now, we'll accept any non-empty token
        if access_token.is_empty() {
            return Ok(AuthResponse {
                success: false,
                session_token: None,
                error: Some("Invalid access token".to_string()),
                permissions: None,
            });
        }

        let user_id = format!(
            "oauth2_user_{}",
            access_token.chars().take(8).collect::<String>()
        );
        let permissions = vec![
            Permission::DagRead,
            Permission::VaultRead,
            Permission::NetworkAccess,
            Permission::CryptoAccess,
        ];

        let session_token = self.create_session(user_id, AuthMethod::OAuth2, permissions)?;

        Ok(AuthResponse {
            success: true,
            session_token: Some(session_token),
            error: None,
            permissions: Some(vec![
                "dag_read".to_string(),
                "vault_read".to_string(),
                "network_access".to_string(),
                "crypto_access".to_string(),
            ]),
        })
    }

    /// Authenticate with vault token
    async fn authenticate_vault_token(
        &mut self,
        credentials: &HashMap<String, String>,
    ) -> Result<AuthResponse> {
        let vault_token = credentials
            .get("vault_token")
            .ok_or_else(|| Error::auth("Vault token not provided"))?;

        // In a real implementation, you would validate the token with the vault
        // For now, we'll accept any non-empty token
        if vault_token.is_empty() {
            return Ok(AuthResponse {
                success: false,
                session_token: None,
                error: Some("Invalid vault token".to_string()),
                permissions: None,
            });
        }

        let user_id = format!(
            "vault_user_{}",
            vault_token.chars().take(8).collect::<String>()
        );
        let permissions = vec![
            Permission::DagRead,
            Permission::DagWrite,
            Permission::VaultRead,
            Permission::VaultWrite,
            Permission::NetworkAccess,
            Permission::CryptoAccess,
            Permission::Admin,
        ];

        let session_token = self.create_session(user_id, AuthMethod::VaultToken, permissions)?;

        Ok(AuthResponse {
            success: true,
            session_token: Some(session_token),
            error: None,
            permissions: Some(vec![
                "dag_read".to_string(),
                "dag_write".to_string(),
                "vault_read".to_string(),
                "vault_write".to_string(),
                "network_access".to_string(),
                "crypto_access".to_string(),
                "admin".to_string(),
            ]),
        })
    }

    /// Clean up expired sessions
    pub fn cleanup_expired_sessions(&mut self) {
        let now = SystemTime::now();
        let timeout = std::time::Duration::from_secs(self.config.session_timeout);

        self.sessions.retain(|_, session| {
            now.duration_since(session.created_at)
                .map(|age| age < timeout)
                .unwrap_or(false)
        });
    }
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            required: false,
            methods: vec![AuthMethod::None],
            session_timeout: 3600, // 1 hour
            max_sessions: 100,
        }
    }
}

impl std::fmt::Display for Permission {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Permission::DagRead => write!(f, "dag_read"),
            Permission::DagWrite => write!(f, "dag_write"),
            Permission::VaultRead => write!(f, "vault_read"),
            Permission::VaultWrite => write!(f, "vault_write"),
            Permission::NetworkAccess => write!(f, "network_access"),
            Permission::CryptoAccess => write!(f, "crypto_access"),
            Permission::Admin => write!(f, "admin"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_auth_disabled() {
        let config = AuthConfig {
            required: false,
            ..Default::default()
        };
        let mut auth = AuthManager::new(config);

        let request = AuthRequest {
            method: AuthMethod::None,
            credentials: HashMap::new(),
            client_info: None,
        };

        let response = auth.authenticate(request).await.unwrap();
        assert!(response.success);
    }

    #[tokio::test]
    async fn test_api_key_auth() {
        let config = AuthConfig {
            required: true,
            methods: vec![AuthMethod::ApiKey],
            ..Default::default()
        };
        let mut auth = AuthManager::new(config);

        let mut credentials = HashMap::new();
        credentials.insert("api_key".to_string(), "test_key_123".to_string());

        let request = AuthRequest {
            method: AuthMethod::ApiKey,
            credentials,
            client_info: None,
        };

        let response = auth.authenticate(request).await.unwrap();
        assert!(response.success);
        assert!(response.session_token.is_some());
    }

    #[tokio::test]
    async fn test_session_validation() {
        let config = AuthConfig {
            required: true,
            methods: vec![AuthMethod::ApiKey],
            session_timeout: 60, // 1 minute
            ..Default::default()
        };
        let mut auth = AuthManager::new(config);

        let mut credentials = HashMap::new();
        credentials.insert("api_key".to_string(), "test_key_123".to_string());

        let request = AuthRequest {
            method: AuthMethod::ApiKey,
            credentials,
            client_info: None,
        };

        let response = auth.authenticate(request).await.unwrap();
        let token = response.session_token.unwrap();

        // Validate the session
        let session = auth.validate_session(&token).await.unwrap();
        assert!(session.is_some());

        // Validate with invalid token
        let invalid_session = auth.validate_session("invalid_token").await.unwrap();
        assert!(invalid_session.is_none());
    }
}
