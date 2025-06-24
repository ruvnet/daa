//! Encryption management for QuDAG MCP security.

use std::sync::Arc;
use aes_gcm::{Aes256Gcm, Key, Nonce, aead::{Aead, KeyInit, OsRng}};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{SaltString, rand_core::RngCore};
use blake3;
use rand::RngCore as _;
use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop};
use tracing::{debug, warn, error};

use crate::error::{McpError, McpResult};

/// Encryption manager for data protection
pub struct EncryptionManager {
    /// Primary encryption cipher
    cipher: Aes256Gcm,
    
    /// Configuration
    config: EncryptionConfig,
    
    /// Key derivation function
    kdf: Argon2<'static>,
}

/// Encryption configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    /// Encryption key size in bytes
    pub key_size: usize,
    
    /// Nonce size in bytes
    pub nonce_size: usize,
    
    /// Key derivation parameters
    pub kdf_params: KdfParams,
    
    /// Enable compression before encryption
    pub enable_compression: bool,
    
    /// Compression level (1-9)
    pub compression_level: u32,
    
    /// Enable key rotation
    pub enable_key_rotation: bool,
    
    /// Key rotation interval in seconds
    pub key_rotation_interval: u64,
}

/// Key derivation function parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KdfParams {
    /// Memory cost (KB)
    pub memory_cost: u32,
    
    /// Time cost (iterations)
    pub time_cost: u32,
    
    /// Parallelism factor
    pub parallelism: u32,
    
    /// Output length in bytes
    pub output_length: usize,
}

/// Encrypted data container
#[derive(Debug, Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
pub struct EncryptedData {
    /// Encrypted data
    pub data: Vec<u8>,
    
    /// Nonce used for encryption
    pub nonce: Vec<u8>,
    
    /// Key derivation salt (if applicable)
    pub salt: Option<Vec<u8>>,
    
    /// Encryption algorithm identifier
    pub algorithm: String,
    
    /// Additional authenticated data
    pub aad: Option<Vec<u8>>,
    
    /// Compression flag
    pub compressed: bool,
    
    /// Key version for rotation
    pub key_version: u32,
    
    /// Encryption timestamp
    pub timestamp: std::time::SystemTime,
}

/// Encryption key material
#[derive(Debug, Clone, Zeroize, ZeroizeOnDrop)]
pub struct EncryptionKey {
    /// Key data
    key: Vec<u8>,
    
    /// Key version
    version: u32,
    
    /// Key creation time
    created_at: std::time::SystemTime,
    
    /// Key derivation salt
    salt: Option<Vec<u8>>,
}

/// Key derivation context
#[derive(Debug, Clone)]
pub struct KeyDerivationContext {
    /// Password or passphrase
    pub password: String,
    
    /// Salt for key derivation
    pub salt: Vec<u8>,
    
    /// Additional context information
    pub context: Option<Vec<u8>>,
    
    /// Key derivation parameters
    pub params: KdfParams,
}

impl EncryptionManager {
    /// Create new encryption manager
    pub fn new(config: EncryptionConfig) -> McpResult<Self> {
        // Generate a random encryption key
        let mut key_bytes = vec![0u8; config.key_size];
        OsRng.fill_bytes(&mut key_bytes);
        
        let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
        let cipher = Aes256Gcm::new(key);
        
        let kdf = Argon2::new(
            argon2::Algorithm::Argon2id,
            argon2::Version::V0x13,
            argon2::Params::new(
                config.kdf_params.memory_cost,
                config.kdf_params.time_cost,
                config.kdf_params.parallelism,
                Some(config.kdf_params.output_length),
            ).map_err(|e| McpError::crypto(format!("Invalid KDF parameters: {}", e)))?,
        );
        
        debug!("Encryption manager initialized with AES-256-GCM");
        Ok(Self {
            cipher,
            config,
            kdf,
        })
    }
    
    /// Create encryption manager with derived key
    pub fn with_derived_key(config: EncryptionConfig, context: KeyDerivationContext) -> McpResult<Self> {
        let key = Self::derive_key(&context)?;
        let cipher_key = Key::<Aes256Gcm>::from_slice(&key.key);
        let cipher = Aes256Gcm::new(cipher_key);
        
        let kdf = Argon2::new(
            argon2::Algorithm::Argon2id,
            argon2::Version::V0x13,
            argon2::Params::new(
                config.kdf_params.memory_cost,
                config.kdf_params.time_cost,
                config.kdf_params.parallelism,
                Some(config.kdf_params.output_length),
            ).map_err(|e| McpError::crypto(format!("Invalid KDF parameters: {}", e)))?,
        );
        
        debug!("Encryption manager initialized with derived key");
        Ok(Self {
            cipher,
            config,
            kdf,
        })
    }
    
    /// Derive encryption key from password
    pub fn derive_key(context: &KeyDerivationContext) -> McpResult<EncryptionKey> {
        let salt_string = SaltString::from_b64(&base64::encode(&context.salt))
            .map_err(|e| McpError::crypto(format!("Invalid salt: {}", e)))?;
        
        let argon2 = Argon2::new(
            argon2::Algorithm::Argon2id,
            argon2::Version::V0x13,
            argon2::Params::new(
                context.params.memory_cost,
                context.params.time_cost,
                context.params.parallelism,
                Some(context.params.output_length),
            ).map_err(|e| McpError::crypto(format!("Invalid KDF parameters: {}", e)))?,
        );
        
        let password_hash = argon2
            .hash_password(context.password.as_bytes(), &salt_string)
            .map_err(|e| McpError::crypto(format!("Key derivation failed: {}", e)))?;
        
        let key_bytes = password_hash.hash.unwrap().as_bytes().to_vec();
        
        Ok(EncryptionKey {
            key: key_bytes,
            version: 1,
            created_at: std::time::SystemTime::now(),
            salt: Some(context.salt.clone()),
        })
    }
    
    /// Encrypt data
    pub async fn encrypt(&self, data: &[u8]) -> McpResult<EncryptedData> {
        let start_time = std::time::Instant::now();
        
        // Optionally compress data before encryption
        let input_data = if self.config.enable_compression {
            self.compress_data(data)?
        } else {
            data.to_vec()
        };
        
        // Generate random nonce
        let mut nonce_bytes = vec![0u8; self.config.nonce_size];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        // Encrypt data
        let encrypted = self.cipher
            .encrypt(nonce, input_data.as_ref())
            .map_err(|e| McpError::crypto(format!("Encryption failed: {}", e)))?;
        
        let encryption_time = start_time.elapsed();
        debug!("Encrypted {} bytes in {:?}", data.len(), encryption_time);
        
        Ok(EncryptedData {
            data: encrypted,
            nonce: nonce_bytes,
            salt: None,
            algorithm: "AES-256-GCM".to_string(),
            aad: None,
            compressed: self.config.enable_compression,
            key_version: 1,
            timestamp: std::time::SystemTime::now(),
        })
    }
    
    /// Decrypt data
    pub async fn decrypt(&self, encrypted_data: &EncryptedData) -> McpResult<Vec<u8>> {
        let start_time = std::time::Instant::now();
        
        // Verify algorithm compatibility
        if encrypted_data.algorithm != "AES-256-GCM" {
            return Err(McpError::crypto(format!(
                "Unsupported encryption algorithm: {}",
                encrypted_data.algorithm
            )));
        }
        
        let nonce = Nonce::from_slice(&encrypted_data.nonce);
        
        // Decrypt data
        let decrypted = self.cipher
            .decrypt(nonce, encrypted_data.data.as_ref())
            .map_err(|e| McpError::crypto(format!("Decryption failed: {}", e)))?;
        
        // Optionally decompress data after decryption
        let output_data = if encrypted_data.compressed {
            self.decompress_data(&decrypted)?
        } else {
            decrypted
        };
        
        let decryption_time = start_time.elapsed();
        debug!("Decrypted {} bytes in {:?}", encrypted_data.data.len(), decryption_time);
        
        Ok(output_data)
    }
    
    /// Encrypt data with additional authenticated data (AAD)
    pub async fn encrypt_with_aad(&self, data: &[u8], aad: &[u8]) -> McpResult<EncryptedData> {
        // For AES-GCM with AAD, we would need to modify the encryption process
        // For now, we'll implement a basic version that stores AAD separately
        let mut encrypted = self.encrypt(data).await?;
        encrypted.aad = Some(aad.to_vec());
        Ok(encrypted)
    }
    
    /// Decrypt data with additional authenticated data verification
    pub async fn decrypt_with_aad(&self, encrypted_data: &EncryptedData, expected_aad: &[u8]) -> McpResult<Vec<u8>> {
        // Verify AAD matches
        if let Some(stored_aad) = &encrypted_data.aad {
            if stored_aad != expected_aad {
                return Err(McpError::crypto("AAD verification failed"));
            }
        } else {
            return Err(McpError::crypto("No AAD present in encrypted data"));
        }
        
        self.decrypt(encrypted_data).await
    }
    
    /// Compress data using DEFLATE
    fn compress_data(&self, data: &[u8]) -> McpResult<Vec<u8>> {
        use flate2::{Compression, write::DeflateEncoder};
        use std::io::Write;
        
        let mut encoder = DeflateEncoder::new(Vec::new(), Compression::new(self.config.compression_level));
        encoder.write_all(data)
            .map_err(|e| McpError::crypto(format!("Compression failed: {}", e)))?;
        
        let compressed = encoder.finish()
            .map_err(|e| McpError::crypto(format!("Compression finalization failed: {}", e)))?;
        
        debug!("Compressed {} bytes to {} bytes", data.len(), compressed.len());
        Ok(compressed)
    }
    
    /// Decompress data using DEFLATE
    fn decompress_data(&self, data: &[u8]) -> McpResult<Vec<u8>> {
        use flate2::read::DeflateDecoder;
        use std::io::Read;
        
        let mut decoder = DeflateDecoder::new(data);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)
            .map_err(|e| McpError::crypto(format!("Decompression failed: {}", e)))?;
        
        debug!("Decompressed {} bytes to {} bytes", data.len(), decompressed.len());
        Ok(decompressed)
    }
    
    /// Generate cryptographically secure random bytes
    pub fn generate_random_bytes(length: usize) -> Vec<u8> {
        let mut bytes = vec![0u8; length];
        OsRng.fill_bytes(&mut bytes);
        bytes
    }
    
    /// Hash data using BLAKE3
    pub fn hash_data(data: &[u8]) -> Vec<u8> {
        blake3::hash(data).as_bytes().to_vec()
    }
    
    /// Hash data with salt using BLAKE3
    pub fn hash_data_with_salt(data: &[u8], salt: &[u8]) -> Vec<u8> {
        let mut hasher = blake3::Hasher::new();
        hasher.update(salt);
        hasher.update(data);
        hasher.finalize().as_bytes().to_vec()
    }
    
    /// Verify password against hash
    pub fn verify_password(&self, password: &str, hash: &str) -> McpResult<bool> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| McpError::crypto(format!("Invalid password hash: {}", e)))?;
        
        Ok(self.kdf.verify_password(password.as_bytes(), &parsed_hash).is_ok())
    }
    
    /// Hash password for storage
    pub fn hash_password(&self, password: &str) -> McpResult<String> {
        let salt = SaltString::generate(&mut OsRng);
        let password_hash = self.kdf
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| McpError::crypto(format!("Password hashing failed: {}", e)))?;
        
        Ok(password_hash.to_string())
    }
    
    /// Rotate encryption key
    pub async fn rotate_key(&mut self) -> McpResult<()> {
        if !self.config.enable_key_rotation {
            return Err(McpError::crypto("Key rotation is disabled"));
        }
        
        // Generate new key
        let mut new_key_bytes = vec![0u8; self.config.key_size];
        OsRng.fill_bytes(&mut new_key_bytes);
        
        let new_key = Key::<Aes256Gcm>::from_slice(&new_key_bytes);
        self.cipher = Aes256Gcm::new(new_key);
        
        debug!("Encryption key rotated successfully");
        Ok(())
    }
    
    /// Get encryption configuration
    pub fn get_config(&self) -> &EncryptionConfig {
        &self.config
    }
    
    /// Update encryption configuration
    pub fn update_config(&mut self, config: EncryptionConfig) -> McpResult<()> {
        // Validate new configuration
        if config.key_size < 32 {
            return Err(McpError::crypto("Key size must be at least 32 bytes"));
        }
        
        if config.nonce_size < 12 {
            return Err(McpError::crypto("Nonce size must be at least 12 bytes"));
        }
        
        self.config = config;
        debug!("Encryption configuration updated");
        Ok(())
    }
}

impl EncryptedData {
    /// Create plaintext data container (for when encryption is disabled)
    pub fn plaintext(data: Vec<u8>) -> Self {
        Self {
            data,
            nonce: Vec::new(),
            salt: None,
            algorithm: "NONE".to_string(),
            aad: None,
            compressed: false,
            key_version: 0,
            timestamp: std::time::SystemTime::now(),
        }
    }
    
    /// Check if data is actually encrypted
    pub fn is_encrypted(&self) -> bool {
        self.algorithm != "NONE"
    }
    
    /// Get data size
    pub fn size(&self) -> usize {
        self.data.len()
    }
    
    /// Serialize to bytes
    pub fn to_bytes(&self) -> McpResult<Vec<u8>> {
        bincode::serialize(self)
            .map_err(|e| McpError::crypto(format!("Serialization failed: {}", e)))
    }
    
    /// Deserialize from bytes
    pub fn from_bytes(data: &[u8]) -> McpResult<Self> {
        bincode::deserialize(data)
            .map_err(|e| McpError::crypto(format!("Deserialization failed: {}", e)))
    }
}

impl Default for EncryptionConfig {
    fn default() -> Self {
        Self {
            key_size: 32,      // 256 bits for AES-256
            nonce_size: 12,    // 96 bits for GCM
            kdf_params: KdfParams::default(),
            enable_compression: false,
            compression_level: 6,
            enable_key_rotation: false,
            key_rotation_interval: 86400, // 24 hours
        }
    }
}

impl Default for KdfParams {
    fn default() -> Self {
        Self {
            memory_cost: 65536,    // 64 MB
            time_cost: 3,          // 3 iterations
            parallelism: 1,        // Single threaded
            output_length: 32,     // 256 bits
        }
    }
}

/// Encryption utilities
pub mod utils {
    use super::*;
    
    /// Secure comparison of byte arrays (constant time)
    pub fn secure_compare(a: &[u8], b: &[u8]) -> bool {
        use constant_time_eq::constant_time_eq;
        a.len() == b.len() && constant_time_eq(a, b)
    }
    
    /// Generate secure random string
    pub fn generate_random_string(length: usize) -> String {
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        let mut rng = OsRng;
        (0..length)
            .map(|_| {
                let idx = (rng.next_u32() as usize) % CHARSET.len();
                CHARSET[idx] as char
            })
            .collect()
    }
    
    /// Generate secure random hex string
    pub fn generate_random_hex(length: usize) -> String {
        let bytes = EncryptionManager::generate_random_bytes(length / 2);
        hex::encode(bytes)
    }
    
    /// Convert bytes to base64
    pub fn bytes_to_base64(data: &[u8]) -> String {
        base64::encode(data)
    }
    
    /// Convert base64 to bytes
    pub fn base64_to_bytes(data: &str) -> McpResult<Vec<u8>> {
        base64::decode(data)
            .map_err(|e| McpError::crypto(format!("Base64 decode failed: {}", e)))
    }
    
    /// Wipe memory securely
    pub fn secure_wipe(data: &mut [u8]) {
        use zeroize::Zeroize;
        data.zeroize();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;
    
    #[tokio::test]
    async fn test_encryption_decryption() {
        let config = EncryptionConfig::default();
        let manager = EncryptionManager::new(config).unwrap();
        
        let plaintext = b"Hello, World! This is a test message.";
        
        // Encrypt
        let encrypted = manager.encrypt(plaintext).await.unwrap();
        assert!(encrypted.is_encrypted());
        assert_ne!(encrypted.data, plaintext);
        
        // Decrypt
        let decrypted = manager.decrypt(&encrypted).await.unwrap();
        assert_eq!(decrypted, plaintext);
    }
    
    #[tokio::test]
    async fn test_encryption_with_compression() {
        let mut config = EncryptionConfig::default();
        config.enable_compression = true;
        
        let manager = EncryptionManager::new(config).unwrap();
        
        // Use larger, compressible data
        let plaintext = b"This is a test message that should compress well due to repetition. This is a test message that should compress well due to repetition. This is a test message that should compress well due to repetition.";
        
        let encrypted = manager.encrypt(plaintext).await.unwrap();
        assert!(encrypted.compressed);
        
        let decrypted = manager.decrypt(&encrypted).await.unwrap();
        assert_eq!(decrypted, plaintext);
    }
    
    #[tokio::test]
    async fn test_key_derivation() {
        let context = KeyDerivationContext {
            password: "test_password_123".to_string(),
            salt: EncryptionManager::generate_random_bytes(32),
            context: None,
            params: KdfParams::default(),
        };
        
        let key1 = EncryptionManager::derive_key(&context).unwrap();
        let key2 = EncryptionManager::derive_key(&context).unwrap();
        
        // Same input should produce same key
        assert_eq!(key1.key, key2.key);
        
        // Different salt should produce different key
        let mut different_context = context.clone();
        different_context.salt = EncryptionManager::generate_random_bytes(32);
        let key3 = EncryptionManager::derive_key(&different_context).unwrap();
        
        assert_ne!(key1.key, key3.key);
    }
    
    #[tokio::test]
    async fn test_password_hashing() {
        let config = EncryptionConfig::default();
        let manager = EncryptionManager::new(config).unwrap();
        
        let password = "test_password_123";
        let hash = manager.hash_password(password).unwrap();
        
        // Verify correct password
        assert!(manager.verify_password(password, &hash).unwrap());
        
        // Verify incorrect password
        assert!(!manager.verify_password("wrong_password", &hash).unwrap());
    }
    
    #[tokio::test]
    async fn test_encrypted_data_serialization() {
        let config = EncryptionConfig::default();
        let manager = EncryptionManager::new(config).unwrap();
        
        let plaintext = b"Test data for serialization";
        let encrypted = manager.encrypt(plaintext).await.unwrap();
        
        // Serialize and deserialize
        let serialized = encrypted.to_bytes().unwrap();
        let deserialized = EncryptedData::from_bytes(&serialized).unwrap();
        
        // Should be able to decrypt after serialization round-trip
        let decrypted = manager.decrypt(&deserialized).await.unwrap();
        assert_eq!(decrypted, plaintext);
    }
    
    #[test]
    fn test_secure_comparison() {
        let data1 = b"test_data";
        let data2 = b"test_data";
        let data3 = b"different";
        
        assert!(utils::secure_compare(data1, data2));
        assert!(!utils::secure_compare(data1, data3));
        assert!(!utils::secure_compare(data1, b"test_dat")); // Different length
    }
    
    #[test]
    fn test_random_generation() {
        let bytes1 = EncryptionManager::generate_random_bytes(32);
        let bytes2 = EncryptionManager::generate_random_bytes(32);
        
        assert_eq!(bytes1.len(), 32);
        assert_eq!(bytes2.len(), 32);
        assert_ne!(bytes1, bytes2); // Should be different
        
        let string1 = utils::generate_random_string(16);
        let string2 = utils::generate_random_string(16);
        
        assert_eq!(string1.len(), 16);
        assert_eq!(string2.len(), 16);
        assert_ne!(string1, string2); // Should be different
        
        let hex1 = utils::generate_random_hex(32);
        let hex2 = utils::generate_random_hex(32);
        
        assert_eq!(hex1.len(), 32);
        assert_eq!(hex2.len(), 32);
        assert_ne!(hex1, hex2); // Should be different
    }
    
    #[test]
    fn test_base64_conversion() {
        let data = b"Hello, World!";
        let encoded = utils::bytes_to_base64(data);
        let decoded = utils::base64_to_bytes(&encoded).unwrap();
        
        assert_eq!(decoded, data);
    }
}