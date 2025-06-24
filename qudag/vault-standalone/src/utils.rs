//! Utility functions for the vault library.

use rand::{distributions::Alphanumeric, Rng};
use std::path::Path;

use crate::error::VaultResult;

/// Character sets for password generation.
#[derive(Debug, Clone, Copy)]
pub enum CharacterSet {
    /// Lowercase letters only.
    Lowercase,
    /// Uppercase letters only.
    Uppercase,
    /// Digits only.
    Digits,
    /// Special characters only.
    Special,
    /// All alphanumeric characters.
    Alphanumeric,
    /// All characters including special.
    All,
}

impl CharacterSet {
    /// Get the character set as a string.
    pub fn as_str(&self) -> &'static str {
        match self {
            CharacterSet::Lowercase => "abcdefghijklmnopqrstuvwxyz",
            CharacterSet::Uppercase => "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
            CharacterSet::Digits => "0123456789",
            CharacterSet::Special => "!@#$%^&*()-_=+[]{}|;:'\",.<>?/~`",
            CharacterSet::Alphanumeric => {
                "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
            }
            CharacterSet::All => {
                "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*()-_=+[]{}|;:'\",.<>?/~`"
            }
        }
    }
}

/// Generate a random password with the specified length and character set.
pub fn generate_password(length: usize, charset: CharacterSet) -> String {
    let chars = charset.as_str();
    let mut rng = rand::thread_rng();
    
    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..chars.len());
            chars.chars().nth(idx).unwrap()
        })
        .collect()
}

/// Generate a random alphanumeric password.
pub fn generate_alphanumeric_password(length: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

/// Ensure a directory exists, creating it if necessary.
pub fn ensure_directory_exists(path: &Path) -> VaultResult<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    Ok(())
}

/// Get the default vault path in the user's home directory.
pub fn default_vault_path() -> Option<std::path::PathBuf> {
    std::env::var("HOME").ok().map(|home| {
        let mut path = std::path::PathBuf::from(home);
        path.push(".qudag");
        path.push("vault.qdag");
        path
    })
}

/// Validate that a password meets minimum security requirements.
pub fn validate_password(password: &str) -> Result<(), String> {
    if password.len() < 8 {
        return Err("Password must be at least 8 characters long".to_string());
    }
    
    let has_lowercase = password.chars().any(|c| c.is_lowercase());
    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_digit = password.chars().any(|c| c.is_numeric());
    
    if !has_lowercase {
        return Err("Password must contain at least one lowercase letter".to_string());
    }
    
    if !has_uppercase {
        return Err("Password must contain at least one uppercase letter".to_string());
    }
    
    if !has_digit {
        return Err("Password must contain at least one digit".to_string());
    }
    
    Ok(())
}

/// Securely compare two byte slices in constant time.
pub fn secure_compare(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    
    let mut result = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        result |= x ^ y;
    }
    
    result == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_password() {
        let password = generate_password(16, CharacterSet::Alphanumeric);
        assert_eq!(password.len(), 16);
        assert!(password.chars().all(|c| c.is_alphanumeric()));
        
        let password2 = generate_password(16, CharacterSet::Alphanumeric);
        assert_ne!(password, password2); // Should be random
    }

    #[test]
    fn test_generate_password_special() {
        let password = generate_password(20, CharacterSet::All);
        assert_eq!(password.len(), 20);
    }

    #[test]
    fn test_validate_password() {
        assert!(validate_password("Test1234").is_ok());
        assert!(validate_password("test").is_err()); // Too short
        assert!(validate_password("testtest").is_err()); // No uppercase or digit
        assert!(validate_password("TESTTEST").is_err()); // No lowercase or digit
        assert!(validate_password("TestTest").is_err()); // No digit
    }

    #[test]
    fn test_secure_compare() {
        let a = b"hello";
        let b = b"hello";
        let c = b"world";
        
        assert!(secure_compare(a, b));
        assert!(!secure_compare(a, c));
        assert!(!secure_compare(a, &b"hell"[..])); // Different lengths
    }
}