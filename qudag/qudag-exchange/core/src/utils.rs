//! Utility functions for QuDAG Exchange

use chrono::{DateTime, Utc};

/// Get current timestamp in seconds
pub fn current_timestamp() -> u64 {
    Utc::now().timestamp() as u64
}

/// Format timestamp to human readable string
pub fn format_timestamp(timestamp: u64) -> String {
    let datetime = DateTime::<Utc>::from_timestamp(timestamp as i64, 0)
        .unwrap_or_else(|| Utc::now());
    datetime.format("%Y-%m-%d %H:%M:%S UTC").to_string()
}

/// Generate a random nonce
pub fn generate_nonce() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timestamp_functions() {
        let ts = current_timestamp();
        assert!(ts > 0);
        
        let formatted = format_timestamp(ts);
        assert!(formatted.contains("UTC"));
    }

    #[test]
    fn test_generate_nonce() {
        let n1 = generate_nonce();
        let n2 = generate_nonce();
        assert_ne!(n1, n2);
    }
}