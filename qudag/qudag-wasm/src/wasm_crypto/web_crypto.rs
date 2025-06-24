//! WebCrypto API provider for WASM
//!
//! This module provides cryptographic operations using the browser's WebCrypto API
//! for better performance and security isolation.

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{CryptoKey, SubtleCrypto};
use js_sys::{ArrayBuffer, Object, Uint8Array, Array};
use anyhow::{Result, anyhow};

/// WebCrypto provider that uses the browser's SubtleCrypto API
pub struct WebCryptoProvider {
    crypto: web_sys::Crypto,
    subtle: SubtleCrypto,
}

impl WebCryptoProvider {
    /// Try to create a new WebCrypto provider
    pub async fn try_new() -> Result<Self> {
        let window = web_sys::window()
            .ok_or_else(|| anyhow!("No window object available"))?;
        
        let crypto = window.crypto()
            .map_err(|_| anyhow!("Crypto API not available"))?;
        
        let subtle = crypto.subtle();
        
        // Test if SubtleCrypto is available by trying a simple operation
        let test_data = Uint8Array::new_with_length(16);
        crypto.get_random_values(&test_data)
            .map_err(|_| anyhow!("Failed to get random values"))?;
        
        Ok(Self { crypto, subtle })
    }
    
    /// Generate random bytes using crypto.getRandomValues()
    pub fn random_bytes(&self, len: usize) -> Result<Vec<u8>> {
        let array = Uint8Array::new_with_length(len as u32);
        self.crypto.get_random_values(&array)
            .map_err(|_| anyhow!("Failed to generate random bytes"))?;
        
        Ok(array.to_vec())
    }
    
    /// AES-GCM encryption using SubtleCrypto
    pub async fn encrypt_aes_gcm(
        &self,
        key: &[u8],
        plaintext: &[u8],
        nonce: &[u8],
    ) -> Result<Vec<u8>> {
        // Import the key
        let key_data = Uint8Array::from(key);
        let key_algorithm = create_aes_key_algorithm();
        let key_usages = Array::new();
        key_usages.push(&JsValue::from_str("encrypt"));
        
        let crypto_key = JsFuture::from(
            self.subtle.import_key_with_str(
                "raw",
                &key_data,
                &key_algorithm,
                false,
                &key_usages,
            )?
        ).await?;
        
        let crypto_key: CryptoKey = crypto_key.dyn_into()
            .map_err(|_| anyhow!("Failed to import key"))?;
        
        // Set up encryption algorithm
        let algorithm = create_aes_gcm_params(nonce);
        let plaintext_array = Uint8Array::from(plaintext);
        
        // Perform encryption
        let encrypted = JsFuture::from(
            self.subtle.encrypt_with_object_and_u8_array(
                &algorithm,
                &crypto_key,
                &plaintext_array,
            )?
        ).await?;
        
        let array_buffer: ArrayBuffer = encrypted.dyn_into()
            .map_err(|_| anyhow!("Failed to encrypt"))?;
        
        let result = Uint8Array::new(&array_buffer);
        Ok(result.to_vec())
    }
    
    /// AES-GCM decryption using SubtleCrypto
    pub async fn decrypt_aes_gcm(
        &self,
        key: &[u8],
        ciphertext: &[u8],
        nonce: &[u8],
    ) -> Result<Vec<u8>> {
        // Import the key
        let key_data = Uint8Array::from(key);
        let key_algorithm = create_aes_key_algorithm();
        let key_usages = Array::new();
        key_usages.push(&JsValue::from_str("decrypt"));
        
        let crypto_key = JsFuture::from(
            self.subtle.import_key_with_str(
                "raw",
                &key_data,
                &key_algorithm,
                false,
                &key_usages,
            )?
        ).await?;
        
        let crypto_key: CryptoKey = crypto_key.dyn_into()
            .map_err(|_| anyhow!("Failed to import key"))?;
        
        // Set up decryption algorithm
        let algorithm = create_aes_gcm_params(nonce);
        let ciphertext_array = Uint8Array::from(ciphertext);
        
        // Perform decryption
        let decrypted = JsFuture::from(
            self.subtle.decrypt_with_object_and_u8_array(
                &algorithm,
                &crypto_key,
                &ciphertext_array,
            )?
        ).await?;
        
        let array_buffer: ArrayBuffer = decrypted.dyn_into()
            .map_err(|_| anyhow!("Failed to decrypt"))?;
        
        let result = Uint8Array::new(&array_buffer);
        Ok(result.to_vec())
    }
    
    /// SHA-256 hashing using SubtleCrypto
    pub async fn sha256(&self, data: &[u8]) -> Result<Vec<u8>> {
        let data_array = Uint8Array::from(data);
        
        let hash = JsFuture::from(
            self.subtle.digest_with_str_and_u8_array("SHA-256", &data_array)?
        ).await?;
        
        let array_buffer: ArrayBuffer = hash.dyn_into()
            .map_err(|_| anyhow!("Failed to compute hash"))?;
        
        let result = Uint8Array::new(&array_buffer);
        Ok(result.to_vec())
    }
    
    /// SHA-512 hashing using SubtleCrypto
    pub async fn sha512(&self, data: &[u8]) -> Result<Vec<u8>> {
        let data_array = Uint8Array::from(data);
        
        let hash = JsFuture::from(
            self.subtle.digest_with_str_and_u8_array("SHA-512", &data_array)?
        ).await?;
        
        let array_buffer: ArrayBuffer = hash.dyn_into()
            .map_err(|_| anyhow!("Failed to compute hash"))?;
        
        let result = Uint8Array::new(&array_buffer);
        Ok(result.to_vec())
    }
}

/// Create AES key import algorithm
fn create_aes_key_algorithm() -> Object {
    let algorithm = Object::new();
    js_sys::Reflect::set(
        &algorithm,
        &JsValue::from_str("name"),
        &JsValue::from_str("AES-GCM"),
    ).unwrap();
    js_sys::Reflect::set(
        &algorithm,
        &JsValue::from_str("length"),
        &JsValue::from_f64(256.0),
    ).unwrap();
    algorithm
}

/// Create AES-GCM algorithm parameters
fn create_aes_gcm_params(nonce: &[u8]) -> Object {
    let params = Object::new();
    js_sys::Reflect::set(
        &params,
        &JsValue::from_str("name"),
        &JsValue::from_str("AES-GCM"),
    ).unwrap();
    
    let iv = Uint8Array::from(nonce);
    js_sys::Reflect::set(
        &params,
        &JsValue::from_str("iv"),
        &iv,
    ).unwrap();
    
    // Tag length (128 bits)
    js_sys::Reflect::set(
        &params,
        &JsValue::from_str("tagLength"),
        &JsValue::from_f64(128.0),
    ).unwrap();
    
    params
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;
    
    #[wasm_bindgen_test]
    async fn test_web_crypto_random() {
        if let Ok(provider) = WebCryptoProvider::try_new().await {
            let random = provider.random_bytes(32).unwrap();
            assert_eq!(random.len(), 32);
            
            // Check that bytes are not all zeros
            assert!(random.iter().any(|&b| b != 0));
        }
    }
    
    #[wasm_bindgen_test]
    async fn test_web_crypto_sha256() {
        if let Ok(provider) = WebCryptoProvider::try_new().await {
            let data = b"Hello, WebCrypto!";
            let hash = provider.sha256(data).await.unwrap();
            assert_eq!(hash.len(), 32); // SHA-256 produces 32 bytes
        }
    }
}