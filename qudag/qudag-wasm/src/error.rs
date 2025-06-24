//! Error types for WASM bindings

use thiserror::Error;
use wasm_bindgen::JsError;

#[derive(Error, Debug)]
pub enum WasmError {
    #[error("Crypto error: {0}")]
    CryptoError(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Operation failed: {0}")]
    OperationFailed(String),

    #[error("Not implemented: {0}")]
    NotImplemented(String),
}

impl WasmError {
    pub fn to_js_error(self) -> JsError {
        JsError::new(&self.to_string())
    }
}
