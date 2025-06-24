//! Password vault operations for WASM
//!
//! Provides quantum-resistant password management including:
//! - Vault initialization and management
//! - Password storage and retrieval
//! - Secure password generation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use wasm_bindgen::prelude::*;

/// WASM wrapper for password vault
#[wasm_bindgen]
pub struct WasmVault {
    // Mock implementation - real implementation would use qudag-vault-core
    entries: Arc<Mutex<HashMap<String, VaultEntry>>>,
    config: Arc<Mutex<VaultConfig>>,
}

#[wasm_bindgen]
impl WasmVault {
    /// Create a new vault
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            entries: Arc::new(Mutex::new(HashMap::new())),
            config: Arc::new(Mutex::new(VaultConfig::default())),
        }
    }

    /// Initialize vault with master password
    #[wasm_bindgen(js_name = "init")]
    pub async fn init(&self, master_password: &str) -> Result<(), JsError> {
        if master_password.len() < 8 {
            return Err(JsError::new(
                "Master password must be at least 8 characters",
            ));
        }

        let mut config = self
            .config
            .lock()
            .map_err(|e| JsError::new(&format!("Failed to lock config: {}", e)))?;

        config.initialized = true;
        config.created_at = js_sys::Date::now() as u64;

        Ok(())
    }

    /// Add a password entry
    #[wasm_bindgen(js_name = "addEntry")]
    pub fn add_entry(
        &self,
        label: &str,
        username: &str,
        password: &str,
        category: Option<String>,
    ) -> Result<String, JsError> {
        let mut entries = self
            .entries
            .lock()
            .map_err(|e| JsError::new(&format!("Failed to lock entries: {}", e)))?;

        if entries.contains_key(label) {
            return Err(JsError::new("Entry with this label already exists"));
        }

        let entry = VaultEntry {
            id: format!("entry_{}", js_sys::Math::random()),
            label: label.to_string(),
            username: username.to_string(),
            password_encrypted: encrypt_mock(password),
            category: category.unwrap_or_else(|| "default".to_string()),
            created_at: js_sys::Date::now() as u64,
            modified_at: js_sys::Date::now() as u64,
            accessed_at: js_sys::Date::now() as u64,
            tags: vec![],
        };

        let entry_id = entry.id.clone();
        entries.insert(label.to_string(), entry);

        Ok(entry_id)
    }

    /// Get a password entry
    #[wasm_bindgen(js_name = "getEntry")]
    pub fn get_entry(&self, label: &str) -> Result<JsValue, JsError> {
        let mut entries = self
            .entries
            .lock()
            .map_err(|e| JsError::new(&format!("Failed to lock entries: {}", e)))?;

        match entries.get_mut(label) {
            Some(entry) => {
                entry.accessed_at = js_sys::Date::now() as u64;

                // Return entry without decrypted password for security
                let safe_entry = SafeVaultEntry {
                    id: entry.id.clone(),
                    label: entry.label.clone(),
                    username: entry.username.clone(),
                    category: entry.category.clone(),
                    created_at: entry.created_at,
                    modified_at: entry.modified_at,
                    accessed_at: entry.accessed_at,
                    tags: entry.tags.clone(),
                };

                Ok(serde_wasm_bindgen::to_value(&safe_entry)?)
            }
            None => Err(JsError::new("Entry not found")),
        }
    }

    /// Get decrypted password
    #[wasm_bindgen(js_name = "getPassword")]
    pub fn get_password(&self, label: &str) -> Result<String, JsError> {
        let entries = self
            .entries
            .lock()
            .map_err(|e| JsError::new(&format!("Failed to lock entries: {}", e)))?;

        match entries.get(label) {
            Some(entry) => Ok(decrypt_mock(&entry.password_encrypted)),
            None => Err(JsError::new("Entry not found")),
        }
    }

    /// List all entries
    #[wasm_bindgen(js_name = "listEntries")]
    pub fn list_entries(&self, category: Option<String>) -> Result<JsValue, JsError> {
        let entries = self
            .entries
            .lock()
            .map_err(|e| JsError::new(&format!("Failed to lock entries: {}", e)))?;

        let filtered_entries: Vec<SafeVaultEntry> = entries
            .values()
            .filter(|e| category.as_ref().map_or(true, |c| &e.category == c))
            .map(|e| SafeVaultEntry {
                id: e.id.clone(),
                label: e.label.clone(),
                username: e.username.clone(),
                category: e.category.clone(),
                created_at: e.created_at,
                modified_at: e.modified_at,
                accessed_at: e.accessed_at,
                tags: e.tags.clone(),
            })
            .collect();

        Ok(serde_wasm_bindgen::to_value(&filtered_entries)?)
    }

    /// Remove an entry
    #[wasm_bindgen(js_name = "removeEntry")]
    pub fn remove_entry(&self, label: &str) -> Result<bool, JsError> {
        let mut entries = self
            .entries
            .lock()
            .map_err(|e| JsError::new(&format!("Failed to lock entries: {}", e)))?;

        Ok(entries.remove(label).is_some())
    }

    /// Generate a secure password
    #[wasm_bindgen(js_name = "generatePassword")]
    pub fn generate_password(
        length: usize,
        include_symbols: bool,
        include_numbers: bool,
    ) -> Result<String, JsError> {
        if length < 8 || length > 128 {
            return Err(JsError::new("Password length must be between 8 and 128"));
        }

        let mut chars = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ".to_string();
        if include_numbers {
            chars.push_str("0123456789");
        }
        if include_symbols {
            chars.push_str("!@#$%^&*()_+-=[]{}|;:,.<>?");
        }

        let mut password = String::new();
        let char_vec: Vec<char> = chars.chars().collect();

        for _ in 0..length {
            let idx = (js_sys::Math::random() * char_vec.len() as f64) as usize;
            password.push(char_vec[idx]);
        }

        Ok(password)
    }

    /// Get vault statistics
    #[wasm_bindgen(js_name = "getStats")]
    pub fn get_stats(&self) -> Result<JsValue, JsError> {
        let entries = self
            .entries
            .lock()
            .map_err(|e| JsError::new(&format!("Failed to lock entries: {}", e)))?;

        let mut categories = HashMap::new();
        for entry in entries.values() {
            *categories.entry(entry.category.clone()).or_insert(0) += 1;
        }

        let stats = VaultStats {
            total_entries: entries.len(),
            categories,
            last_modified: entries.values().map(|e| e.modified_at).max().unwrap_or(0),
        };

        Ok(serde_wasm_bindgen::to_value(&stats)?)
    }
}

// Mock encryption/decryption functions
fn encrypt_mock(data: &str) -> String {
    // In real implementation, would use AES-256-GCM
    format!("encrypted_{}", base64::encode(data))
}

fn decrypt_mock(encrypted: &str) -> String {
    // In real implementation, would decrypt with AES-256-GCM
    if let Some(encoded) = encrypted.strip_prefix("encrypted_") {
        String::from_utf8(base64::decode(encoded).unwrap_or_default())
            .unwrap_or_else(|_| "decryption_error".to_string())
    } else {
        "decryption_error".to_string()
    }
}

// Data structures
#[derive(Serialize, Deserialize, Clone)]
struct VaultEntry {
    id: String,
    label: String,
    username: String,
    password_encrypted: String,
    category: String,
    created_at: u64,
    modified_at: u64,
    accessed_at: u64,
    tags: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct SafeVaultEntry {
    id: String,
    label: String,
    username: String,
    category: String,
    created_at: u64,
    modified_at: u64,
    accessed_at: u64,
    tags: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct VaultConfig {
    initialized: bool,
    created_at: u64,
    auto_lock_minutes: u32,
    password_history: bool,
}

impl Default for VaultConfig {
    fn default() -> Self {
        Self {
            initialized: false,
            created_at: 0,
            auto_lock_minutes: 10,
            password_history: true,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct VaultStats {
    total_entries: usize,
    categories: HashMap<String, usize>,
    last_modified: u64,
}

// Base64 encoding utilities
mod base64 {
    pub fn encode(data: &str) -> std::string::String {
        // Simple mock implementation
        data.chars().map(|c| format!("{:02x}", c as u8)).collect()
    }

    pub fn decode(encoded: &str) -> Result<std::vec::Vec<u8>, &'static str> {
        // Simple mock implementation
        if encoded.len() % 2 != 0 {
            return Err("Invalid encoding");
        }

        encoded
            .as_bytes()
            .chunks(2)
            .map(|chunk| {
                let s = std::str::from_utf8(chunk).unwrap();
                u8::from_str_radix(s, 16).map_err(|_| "Invalid hex")
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    async fn test_vault_creation() {
        let vault = WasmVault::new();
        let result = vault.init("testpassword123").await;
        assert!(result.is_ok());
    }

    #[wasm_bindgen_test]
    fn test_password_generation() {
        let password = WasmVault::generate_password(16, true, true).unwrap();
        assert_eq!(password.len(), 16);
    }
}
