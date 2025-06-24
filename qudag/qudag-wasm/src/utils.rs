//! Utility functions for WASM
//!
//! Provides helper functions and utilities for WASM operations

use wasm_bindgen::prelude::*;
use web_sys::console;

/// Set up panic hook for better error messages
pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// Log a message to the browser console
#[wasm_bindgen]
pub fn log(message: &str) {
    console::log_1(&message.into());
}

/// Log an error to the browser console
#[wasm_bindgen]
pub fn log_error(message: &str) {
    console::error_1(&message.into());
}

/// Log a warning to the browser console
#[wasm_bindgen]
pub fn log_warn(message: &str) {
    console::warn_1(&message.into());
}

/// Performance utilities
#[wasm_bindgen]
pub struct Performance;

#[wasm_bindgen]
impl Performance {
    /// Get current timestamp in milliseconds
    #[wasm_bindgen(js_name = "now")]
    pub fn now() -> f64 {
        js_sys::Date::now()
    }

    /// Measure execution time of a function
    #[wasm_bindgen(js_name = "measure")]
    pub fn measure(name: &str, start: f64) -> f64 {
        let duration = Self::now() - start;
        log(&format!("{}: {}ms", name, duration));
        duration
    }
}

/// Memory utilities
#[wasm_bindgen]
pub struct Memory;

#[wasm_bindgen]
impl Memory {
    /// Get current WASM memory usage
    #[wasm_bindgen(js_name = "getUsage")]
    pub fn get_usage() -> JsValue {
        if let Ok(memory) = js_sys::Reflect::get(&wasm_bindgen::memory(), &"buffer".into()) {
            if let Some(buffer) = memory.dyn_ref::<js_sys::ArrayBuffer>() {
                let obj = js_sys::Object::new();
                js_sys::Reflect::set(&obj, &"bytes".into(), &buffer.byte_length().into()).unwrap();
                return obj.into();
            }
        }
        JsValue::NULL
    }
}

/// Encoding utilities
#[wasm_bindgen]
pub struct Encoding;

#[wasm_bindgen]
impl Encoding {
    /// Convert bytes to hex string
    #[wasm_bindgen(js_name = "bytesToHex")]
    pub fn bytes_to_hex(bytes: &[u8]) -> String {
        hex::encode(bytes)
    }

    /// Convert hex string to bytes
    #[wasm_bindgen(js_name = "hexToBytes")]
    pub fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, JsError> {
        hex::decode(hex).map_err(|e| JsError::new(&format!("Invalid hex string: {}", e)))
    }

    /// Convert string to bytes (UTF-8)
    #[wasm_bindgen(js_name = "stringToBytes")]
    pub fn string_to_bytes(s: &str) -> Vec<u8> {
        s.as_bytes().to_vec()
    }

    /// Convert bytes to string (UTF-8)
    #[wasm_bindgen(js_name = "bytesToString")]
    pub fn bytes_to_string(bytes: &[u8]) -> Result<String, JsError> {
        std::string::String::from_utf8(bytes.to_vec())
            .map_err(|e| JsError::new(&format!("Invalid UTF-8: {}", e)))
    }
}

/// Validation utilities
#[wasm_bindgen]
pub struct Validation;

#[wasm_bindgen]
impl Validation {
    /// Validate a dark domain name
    #[wasm_bindgen(js_name = "isDarkDomain")]
    pub fn is_dark_domain(domain: &str) -> bool {
        domain.ends_with(".dark") && domain.len() > 5
    }

    /// Validate a peer address
    #[wasm_bindgen(js_name = "isPeerAddress")]
    pub fn is_peer_address(address: &str) -> bool {
        // Simple validation - check if it looks like multiaddr
        address.starts_with("/ip4/") || address.starts_with("/ip6/")
    }

    /// Validate a hex string
    #[wasm_bindgen(js_name = "isValidHex")]
    pub fn is_valid_hex(hex: &str) -> bool {
        hex.chars().all(|c| c.is_ascii_hexdigit())
    }
}

/// Random utilities
#[wasm_bindgen]
pub struct Random;

#[wasm_bindgen]
impl Random {
    /// Generate random bytes
    #[wasm_bindgen(js_name = "getBytes")]
    pub fn get_bytes(length: usize) -> Result<Vec<u8>, JsError> {
        let mut bytes = vec![0u8; length];
        getrandom::getrandom(&mut bytes)
            .map_err(|e| JsError::new(&format!("Failed to generate random bytes: {}", e)))?;
        Ok(bytes)
    }

    /// Generate a random ID
    #[wasm_bindgen(js_name = "getId")]
    pub fn get_id() -> String {
        let bytes = Self::get_bytes(16).unwrap_or_else(|_| vec![0u8; 16]);
        hex::encode(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn test_encoding_roundtrip() {
        let original = "Hello, QuDAG!";
        let bytes = Encoding::string_to_bytes(original);
        let hex = Encoding::bytes_to_hex(&bytes);
        let decoded_bytes = Encoding::hex_to_bytes(&hex).unwrap();
        let decoded = Encoding::bytes_to_string(&decoded_bytes).unwrap();
        assert_eq!(original, decoded);
    }

    #[wasm_bindgen_test]
    fn test_validation() {
        assert!(Validation::is_dark_domain("test.dark"));
        assert!(!Validation::is_dark_domain("test.com"));
        assert!(Validation::is_peer_address("/ip4/127.0.0.1/tcp/8000"));
        assert!(Validation::is_valid_hex("deadbeef"));
        assert!(!Validation::is_valid_hex("not-hex"));
    }
}
