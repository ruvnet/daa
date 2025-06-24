//! Key derivation functions for password-based encryption.

use argon2::{
    password_hash::{PasswordHasher, SaltString},
    Argon2, Params,
};
use rand::rngs::OsRng;
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::{
    crypto::{VaultCrypto, VAULT_KEY_SIZE},
    error::{VaultError, VaultResult},
};

/// Argon2id parameters for key derivation.
/// These are tuned for security while maintaining reasonable performance.
const ARGON2_MEMORY: u32 = 64 * 1024; // 64 MB
const ARGON2_ITERATIONS: u32 = 3;
const ARGON2_PARALLELISM: u32 = 4;

/// Password wrapper that zeroizes on drop.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct Password(String);

impl Password {
    /// Create a new password wrapper.
    pub fn new(password: String) -> Self {
        Self(password)
    }

    /// Get the password as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Key derivation context containing salt and parameters.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct KdfContext {
    /// Salt for the KDF.
    pub salt: String,
    /// Argon2 parameters encoded as a string.
    pub params: String,
}

/// Derive a vault key from a password using Argon2id.
pub fn derive_key(password: &Password) -> VaultResult<([u8; VAULT_KEY_SIZE], KdfContext)> {
    let salt = SaltString::generate(&mut OsRng);
    
    let params = Params::new(
        ARGON2_MEMORY,
        ARGON2_ITERATIONS,
        ARGON2_PARALLELISM,
        Some(VAULT_KEY_SIZE),
    )
    .map_err(|e| VaultError::KeyDerivation(format!("Invalid Argon2 params: {}", e)))?;
    
    let argon2 = Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        params.clone(),
    );
    
    let password_hash = argon2
        .hash_password(password.as_str().as_bytes(), &salt)
        .map_err(|e| VaultError::KeyDerivation(format!("Failed to hash password: {}", e)))?;
    
    let hash_bytes = password_hash
        .hash
        .ok_or_else(|| VaultError::KeyDerivation("No hash output".to_string()))?;
    
    let mut key = [0u8; VAULT_KEY_SIZE];
    key.copy_from_slice(&hash_bytes.as_bytes()[..VAULT_KEY_SIZE]);
    
    let context = KdfContext {
        salt: salt.to_string(),
        params: format!("m={},t={},p={}", ARGON2_MEMORY, ARGON2_ITERATIONS, ARGON2_PARALLELISM),
    };
    
    Ok((key, context))
}

/// Derive a vault key from a password using a saved context.
pub fn derive_key_with_context(
    password: &Password,
    context: &KdfContext,
) -> VaultResult<[u8; VAULT_KEY_SIZE]> {
    let salt = SaltString::from_b64(&context.salt)
        .map_err(|e| VaultError::KeyDerivation(format!("Invalid salt: {}", e)))?;
    
    // Parse params from string format "m=65536,t=3,p=4"
    let params = Params::new(
        ARGON2_MEMORY,
        ARGON2_ITERATIONS,
        ARGON2_PARALLELISM,
        Some(VAULT_KEY_SIZE),
    )
    .map_err(|e| VaultError::KeyDerivation(format!("Invalid params: {}", e)))?;
    
    let argon2 = Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        params,
    );
    
    // We don't need to create a dummy hash for verification
    // Just re-derive the key with the same parameters
    
    // Re-derive the key
    let hash_output = argon2
        .hash_password(password.as_str().as_bytes(), &salt)
        .map_err(|e| VaultError::KeyDerivation(format!("Failed to hash password: {}", e)))?;
    
    let hash_bytes = hash_output
        .hash
        .ok_or_else(|| VaultError::KeyDerivation("No hash output".to_string()))?;
    
    let mut key = [0u8; VAULT_KEY_SIZE];
    key.copy_from_slice(&hash_bytes.as_bytes()[..VAULT_KEY_SIZE]);
    
    Ok(key)
}

/// Encrypt the vault key with a password-derived key.
pub fn encrypt_vault_key(
    vault_key: &[u8; VAULT_KEY_SIZE],
    password: &Password,
) -> VaultResult<(Vec<u8>, KdfContext)> {
    let (derived_key, context) = derive_key(password)?;
    let crypto = VaultCrypto::from_key(derived_key);
    let encrypted = crypto.encrypt(vault_key)?;
    
    Ok((encrypted, context))
}

/// Decrypt the vault key with a password-derived key.
pub fn decrypt_vault_key(
    encrypted_key: &[u8],
    password: &Password,
    context: &KdfContext,
) -> VaultResult<[u8; VAULT_KEY_SIZE]> {
    let derived_key = derive_key_with_context(password, context)?;
    let crypto = VaultCrypto::from_key(derived_key);
    let decrypted = crypto.decrypt(encrypted_key)?;
    
    if decrypted.len() != VAULT_KEY_SIZE {
        return Err(VaultError::InvalidFormat(
            "Decrypted key has wrong size".to_string(),
        ));
    }
    
    let mut key = [0u8; VAULT_KEY_SIZE];
    key.copy_from_slice(&decrypted);
    
    Ok(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_key() {
        let password = Password::new("test_password".to_string());
        let (key1, context1) = derive_key(&password).unwrap();
        let (key2, context2) = derive_key(&password).unwrap();
        
        // Different salts should produce different keys
        assert_ne!(key1, key2);
        assert_ne!(context1.salt, context2.salt);
    }

    #[test]
    fn test_derive_key_with_context() {
        let password = Password::new("test_password".to_string());
        let (key1, context) = derive_key(&password).unwrap();
        let key2 = derive_key_with_context(&password, &context).unwrap();
        
        // Same password and context should produce same key
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_encrypt_decrypt_vault_key() {
        let mut vault_key = [0u8; VAULT_KEY_SIZE];
        getrandom::getrandom(&mut vault_key).unwrap();
        
        let password = Password::new("secure_password".to_string());
        let (encrypted, context) = encrypt_vault_key(&vault_key, &password).unwrap();
        
        let decrypted = decrypt_vault_key(&encrypted, &password, &context).unwrap();
        assert_eq!(vault_key, decrypted);
        
        // Wrong password should fail
        let wrong_password = Password::new("wrong_password".to_string());
        decrypt_vault_key(&encrypted, &wrong_password, &context).unwrap_err();
    }
}