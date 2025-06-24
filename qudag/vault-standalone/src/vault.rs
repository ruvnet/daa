//! Main vault implementation combining all components.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

use crate::{
    crypto::{VaultCrypto, VaultKeyPair},
    dag_storage::{SerializedVaultDag, VaultDag},
    error::{VaultError, VaultResult},
    kdf::{self, KdfContext, Password},
    secret::SecretEntry,
    utils::{self, CharacterSet},
};

/// Version of the vault format.
const VAULT_VERSION: u32 = 1;

/// Main vault structure managing encrypted secrets.
pub struct Vault {
    /// Path to the vault file.
    path: PathBuf,
    /// Cryptographic operations handler.
    crypto: VaultCrypto,
    /// DAG storage for secrets.
    dag: VaultDag,
    /// Vault metadata.
    metadata: VaultMetadata,
}

/// Metadata about the vault.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultMetadata {
    /// Version of the vault format.
    pub version: u32,
    /// When the vault was created.
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// When the vault was last modified.
    pub modified_at: chrono::DateTime<chrono::Utc>,
    /// Number of secrets in the vault.
    pub secret_count: usize,
    /// Optional description of the vault.
    pub description: Option<String>,
}

/// Vault file format containing all encrypted data.
#[derive(Debug, Serialize, Deserialize)]
struct VaultFile {
    /// Version of the vault format.
    version: u32,
    /// KDF context for password derivation.
    kdf_context: KdfContext,
    /// Encrypted vault key.
    encrypted_vault_key: Vec<u8>,
    /// Optional quantum-resistant key pair.
    keypair: Option<VaultKeyPair>,
    /// Serialized and encrypted DAG.
    encrypted_dag: Vec<u8>,
    /// Vault metadata.
    metadata: VaultMetadata,
}

impl Vault {
    /// Create a new vault at the specified path.
    pub fn create(path: impl AsRef<Path>, master_password: &str) -> VaultResult<Self> {
        let path = path.as_ref().to_path_buf();
        
        // Check if vault already exists
        if path.exists() {
            return Err(VaultError::VaultExists(path.display().to_string()));
        }

        info!("Creating new vault at: {}", path.display());

        // Ensure directory exists
        utils::ensure_directory_exists(&path)?;

        // Create crypto components
        let crypto = VaultCrypto::new()?;
        let password = Password::new(master_password.to_string());
        
        // Derive key from password and encrypt vault key
        let (encrypted_vault_key, kdf_context) = 
            kdf::encrypt_vault_key(crypto.get_key(), &password)?;

        // Generate quantum-resistant key pair
        let keypair = crypto.generate_keypair()?;

        // Create empty DAG
        let dag = VaultDag::new();
        
        // Create metadata
        let metadata = VaultMetadata {
            version: VAULT_VERSION,
            created_at: chrono::Utc::now(),
            modified_at: chrono::Utc::now(),
            secret_count: 0,
            description: None,
        };

        // Create vault instance
        let vault = Self {
            path: path.clone(),
            crypto,
            dag,
            metadata,
        };

        // Save the vault
        vault.save_internal(encrypted_vault_key, kdf_context, Some(keypair))?;

        info!("Vault created successfully");
        Ok(vault)
    }

    /// Open an existing vault.
    pub fn open(path: impl AsRef<Path>, master_password: &str) -> VaultResult<Self> {
        let path = path.as_ref().to_path_buf();
        
        if !path.exists() {
            return Err(VaultError::VaultNotFound(path.display().to_string()));
        }

        info!("Opening vault at: {}", path.display());

        // Read vault file
        let data = std::fs::read(&path)?;
        let vault_file: VaultFile = serde_json::from_slice(&data)?;

        // Check version compatibility
        if vault_file.version > VAULT_VERSION {
            return Err(VaultError::InvalidFormat(format!(
                "Unsupported vault version: {}",
                vault_file.version
            )));
        }

        // Decrypt vault key
        let password = Password::new(master_password.to_string());
        let vault_key = kdf::decrypt_vault_key(
            &vault_file.encrypted_vault_key,
            &password,
            &vault_file.kdf_context,
        )?;

        // Create crypto with decrypted key
        let crypto = VaultCrypto::from_key(vault_key);

        // Decrypt and deserialize DAG
        let dag_data = crypto.decrypt(&vault_file.encrypted_dag)?;
        let serialized_dag: SerializedVaultDag = bincode::deserialize(&dag_data)?;
        let dag = VaultDag::deserialize(serialized_dag)?;

        debug!("Vault opened with {} secrets", vault_file.metadata.secret_count);

        Ok(Self {
            path,
            crypto,
            dag,
            metadata: vault_file.metadata,
        })
    }

    /// Add a new secret to the vault.
    pub fn add_secret(
        &mut self,
        label: &str,
        username: &str,
        password: Option<&str>,
    ) -> VaultResult<()> {
        let password = match password {
            Some(p) => p.to_string(),
            None => utils::generate_password(16, CharacterSet::All),
        };

        let secret = SecretEntry::new(label.to_string(), username.to_string(), password);
        
        // Parse categories from label (e.g., "email/work" -> category "email")
        let categories = self.parse_categories(label);
        
        self.dag.add_secret(secret, &self.crypto, categories)?;
        self.metadata.secret_count += 1;
        self.metadata.modified_at = chrono::Utc::now();
        
        // Save changes
        self.save()?;
        
        info!("Added secret: {}", label);
        Ok(())
    }

    /// Get a secret by its label.
    pub fn get_secret(&self, label: &str) -> VaultResult<SecretEntry> {
        debug!("Retrieving secret: {}", label);
        self.dag.get_secret(label, &self.crypto)
    }

    /// List all secrets or those in a specific category.
    pub fn list_secrets(&self, category: Option<&str>) -> VaultResult<Vec<String>> {
        self.dag.list_secrets(category)
    }

    /// Update an existing secret.
    pub fn update_secret(
        &mut self,
        label: &str,
        username: Option<&str>,
        password: Option<&str>,
    ) -> VaultResult<()> {
        let mut secret = self.get_secret(label)?;
        
        if let Some(u) = username {
            secret.username = u.to_string();
        }
        
        if let Some(p) = password {
            secret.update_password(p.to_string());
        }
        
        self.dag.update_secret(label, secret, &self.crypto)?;
        self.metadata.modified_at = chrono::Utc::now();
        
        // Save changes
        self.save()?;
        
        info!("Updated secret: {}", label);
        Ok(())
    }

    /// Delete a secret from the vault.
    pub fn delete_secret(&mut self, label: &str) -> VaultResult<()> {
        self.dag.delete_secret(label)?;
        self.metadata.secret_count = self.metadata.secret_count.saturating_sub(1);
        self.metadata.modified_at = chrono::Utc::now();
        
        // Save changes
        self.save()?;
        
        info!("Deleted secret: {}", label);
        Ok(())
    }

    /// Export the vault to a file.
    pub fn export(&self, output_path: impl AsRef<Path>) -> VaultResult<()> {
        let output_path = output_path.as_ref();
        info!("Exporting vault to: {}", output_path.display());
        
        // Read current vault file
        let data = std::fs::read(&self.path)?;
        
        // Write to output (vault file is already encrypted)
        std::fs::write(output_path, data)?;
        
        info!("Vault exported successfully");
        Ok(())
    }

    /// Import secrets from another vault file.
    pub fn import(&mut self, input_path: impl AsRef<Path>, master_password: &str) -> VaultResult<()> {
        let input_path = input_path.as_ref();
        info!("Importing vault from: {}", input_path.display());
        
        // Open the other vault
        let other_vault = Self::open(input_path, master_password)?;
        
        // Import all secrets
        let secrets = other_vault.list_secrets(None)?;
        let mut imported = 0;
        
        for label in secrets {
            if self.dag.get_secret(&label, &self.crypto).is_ok() {
                warn!("Skipping duplicate secret: {}", label);
                continue;
            }
            
            let secret = other_vault.get_secret(&label)?;
            let categories = self.parse_categories(&label);
            self.dag.add_secret(secret, &self.crypto, categories)?;
            imported += 1;
        }
        
        self.metadata.secret_count += imported;
        self.metadata.modified_at = chrono::Utc::now();
        
        // Save changes
        self.save()?;
        
        info!("Imported {} secrets", imported);
        Ok(())
    }

    /// Generate a random password.
    pub fn generate_password(&self, length: usize, charset: CharacterSet) -> String {
        utils::generate_password(length, charset)
    }

    /// Save the vault to disk.
    fn save(&self) -> VaultResult<()> {
        // Re-read the file to get current KDF context and keypair
        let data = std::fs::read(&self.path)?;
        let current_file: VaultFile = serde_json::from_slice(&data)?;
        
        self.save_internal(
            current_file.encrypted_vault_key,
            current_file.kdf_context,
            current_file.keypair,
        )
    }

    /// Internal save method.
    fn save_internal(
        &self,
        encrypted_vault_key: Vec<u8>,
        kdf_context: KdfContext,
        keypair: Option<VaultKeyPair>,
    ) -> VaultResult<()> {
        // Serialize and encrypt the DAG
        let serialized_dag = self.dag.serialize()?;
        let dag_data = bincode::serialize(&serialized_dag)?;
        let encrypted_dag = self.crypto.encrypt(&dag_data)?;
        
        // Create vault file
        let vault_file = VaultFile {
            version: VAULT_VERSION,
            kdf_context,
            encrypted_vault_key,
            keypair,
            encrypted_dag,
            metadata: self.metadata.clone(),
        };
        
        // Serialize to JSON
        let data = serde_json::to_vec_pretty(&vault_file)?;
        
        // Write atomically (write to temp file then rename)
        let temp_path = self.path.with_extension("tmp");
        std::fs::write(&temp_path, data)?;
        std::fs::rename(&temp_path, &self.path)?;
        
        debug!("Vault saved successfully");
        Ok(())
    }

    /// Parse categories from a label (e.g., "email/work" -> ["email"]).
    fn parse_categories(&self, label: &str) -> Vec<String> {
        if label.contains('/') {
            let parts: Vec<&str> = label.split('/').collect();
            if parts.len() > 1 {
                return parts[..parts.len() - 1]
                    .iter()
                    .map(|s| s.to_string())
                    .collect();
            }
        }
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile;

    #[test]
    fn test_vault_create_open() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("test_vault.qdag");
        
        // Create vault
        let vault = Vault::create(&path, "test_password").unwrap();
        drop(vault);
        
        // Open vault
        let vault = Vault::open(&path, "test_password").unwrap();
        assert_eq!(vault.metadata.version, VAULT_VERSION);
        
        // Wrong password should fail
        assert!(Vault::open(&path, "wrong_password").is_err());
    }

    #[test]
    fn test_vault_secrets() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("test_vault.qdag");
        
        let mut vault = Vault::create(&path, "test_password").unwrap();
        
        // Add secret
        vault.add_secret("test/secret", "user", Some("password")).unwrap();
        
        // Get secret
        let secret = vault.get_secret("test/secret").unwrap();
        assert_eq!(secret.username, "user");
        assert_eq!(secret.password.as_str(), "password");
        
        // List secrets
        let secrets = vault.list_secrets(None).unwrap();
        assert_eq!(secrets.len(), 2); // "test/secret" and "test" category
        assert!(secrets.contains(&"test/secret".to_string()));
        assert!(secrets.contains(&"test".to_string()));
        
        // Update secret
        vault.update_secret("test/secret", Some("new_user"), None).unwrap();
        let updated = vault.get_secret("test/secret").unwrap();
        assert_eq!(updated.username, "new_user");
        
        // Delete secret
        vault.delete_secret("test/secret").unwrap();
        vault.get_secret("test/secret").unwrap_err();
    }
}