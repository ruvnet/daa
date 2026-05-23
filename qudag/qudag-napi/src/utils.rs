//! Utility functions for QuDAG native bindings

use napi::bindgen_prelude::*;
use napi_derive::napi;

/// Convert hex string to bytes
#[napi]
pub fn hex_to_bytes(hex: String) -> Result<Buffer> {
    let bytes = hex::decode(hex.trim_start_matches("0x"))
        .map_err(|e| Error::from_reason(format!("Invalid hex string: {}", e)))?;
    Ok(bytes.into())
}

/// Convert bytes to hex string
#[napi]
pub fn bytes_to_hex(bytes: Buffer) -> Result<String> {
    Ok(hex::encode(bytes.as_ref()))
}

/// Generate random bytes
#[napi]
pub fn random_bytes(length: u32) -> Result<Buffer> {
    use rand::RngCore;
    let mut rng = rand::thread_rng();
    let mut bytes = vec![0u8; length as usize];
    rng.fill_bytes(&mut bytes);
    Ok(bytes.into())
}

/// Constant-time comparison of buffers
#[napi]
pub fn constant_time_compare(a: Buffer, b: Buffer) -> Result<bool> {
    if a.len() != b.len() {
        return Ok(false);
    }

    let mut result = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        result |= x ^ y;
    }

    Ok(result == 0)
}
