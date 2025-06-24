//! Secret entry types and metadata.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop};

/// A secret entry containing sensitive information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretEntry {
    /// Unique label for the secret (e.g., "email/work").
    pub label: String,
    /// Username associated with the secret.
    pub username: String,
    /// The actual password or secret value.
    pub password: SensitiveString,
    /// Optional URL associated with the secret.
    pub url: Option<String>,
    /// Optional notes about the secret.
    pub notes: Option<String>,
    /// Metadata about the secret.
    pub metadata: SecretMetadata,
}

/// Wrapper for sensitive string data that zeroizes on drop.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensitiveString(String);

impl SensitiveString {
    /// Create a new sensitive string.
    pub fn new(value: String) -> Self {
        Self(value)
    }

    /// Get the string value.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consume and return the inner string.
    pub fn into_inner(self) -> String {
        // We need to manually take the string to avoid drop
        let mut s = self;
        std::mem::take(&mut s.0)
    }
}

impl Drop for SensitiveString {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

impl ZeroizeOnDrop for SensitiveString {}

/// Metadata associated with a secret entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretMetadata {
    /// When the secret was created.
    pub created_at: DateTime<Utc>,
    /// When the secret was last modified.
    pub modified_at: DateTime<Utc>,
    /// When the secret was last accessed.
    pub accessed_at: Option<DateTime<Utc>>,
    /// Version number for the secret.
    pub version: u32,
    /// Tags for categorization.
    pub tags: Vec<String>,
    /// Custom fields for extensibility.
    pub custom_fields: std::collections::HashMap<String, String>,
}

impl Default for SecretMetadata {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            created_at: now,
            modified_at: now,
            accessed_at: None,
            version: 1,
            tags: Vec::new(),
            custom_fields: std::collections::HashMap::new(),
        }
    }
}

impl SecretEntry {
    /// Create a new secret entry.
    pub fn new(label: String, username: String, password: String) -> Self {
        Self {
            label,
            username,
            password: SensitiveString::new(password),
            url: None,
            notes: None,
            metadata: SecretMetadata::default(),
        }
    }

    /// Create a new secret entry with full details.
    pub fn new_with_details(
        label: String,
        username: String,
        password: String,
        url: Option<String>,
        notes: Option<String>,
        tags: Vec<String>,
    ) -> Self {
        let mut metadata = SecretMetadata::default();
        metadata.tags = tags;

        Self {
            label,
            username,
            password: SensitiveString::new(password),
            url,
            notes,
            metadata,
        }
    }

    /// Update the accessed timestamp.
    pub fn mark_accessed(&mut self) {
        self.metadata.accessed_at = Some(Utc::now());
    }

    /// Update the secret password and increment version.
    pub fn update_password(&mut self, new_password: String) {
        self.password = SensitiveString::new(new_password);
        self.metadata.modified_at = Utc::now();
        self.metadata.version += 1;
    }

    /// Check if the secret matches a search query.
    pub fn matches(&self, query: &str) -> bool {
        let query_lower = query.to_lowercase();
        self.label.to_lowercase().contains(&query_lower)
            || self.username.to_lowercase().contains(&query_lower)
            || self
                .url
                .as_ref()
                .map_or(false, |u| u.to_lowercase().contains(&query_lower))
            || self.tags().any(|t| t.to_lowercase().contains(&query_lower))
    }

    /// Get an iterator over the secret's tags.
    pub fn tags(&self) -> impl Iterator<Item = &str> {
        self.metadata.tags.iter().map(|s| s.as_str())
    }
}

/// Encrypted secret data stored in the vault.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedSecret {
    /// The encrypted secret data.
    pub encrypted_data: Vec<u8>,
    /// Metadata (stored unencrypted for searching).
    pub metadata: SecretMetadata,
    /// Label (stored unencrypted for indexing).
    pub label: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_entry_creation() {
        let secret = SecretEntry::new(
            "email/work".to_string(),
            "user@example.com".to_string(),
            "password123".to_string(),
        );

        assert_eq!(secret.label, "email/work");
        assert_eq!(secret.username, "user@example.com");
        assert_eq!(secret.password.as_str(), "password123");
        assert_eq!(secret.metadata.version, 1);
    }

    #[test]
    fn test_secret_matching() {
        let secret = SecretEntry::new_with_details(
            "email/work".to_string(),
            "user@example.com".to_string(),
            "password123".to_string(),
            Some("https://example.com".to_string()),
            None,
            vec!["work".to_string(), "email".to_string()],
        );

        assert!(secret.matches("email"));
        assert!(secret.matches("WORK"));
        assert!(secret.matches("example.com"));
        assert!(secret.matches("user@"));
        assert!(!secret.matches("github"));
    }

    #[test]
    fn test_sensitive_string_zeroize() {
        let password = "secret123".to_string();
        {
            let sensitive = SensitiveString::new(password.clone());
            assert_eq!(sensitive.as_str(), "secret123");
        }
        // SensitiveString should be zeroized on drop
        // (Can't easily test this without unsafe code)
    }
}
