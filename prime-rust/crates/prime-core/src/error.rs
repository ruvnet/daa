//! Error types for Prime distributed ML framework

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Network error: {0}")]
    Network(String),

    #[error("Protocol error: {0}")]
    Protocol(String),

    #[error("Consensus error: {0}")]
    Consensus(String),

    #[error("Training error: {0}")]
    Training(String),

    #[error("DHT error: {0}")]
    Dht(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("Other error: {0}")]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_error_display() {
        let err = Error::Network("connection failed".to_string());
        assert_eq!(err.to_string(), "Network error: connection failed");
    }

    #[test]
    fn test_error_conversion() {
        let json_err = serde_json::from_str::<String>("invalid").unwrap_err();
        let err: Error = json_err.into();
        assert!(matches!(err, Error::Serialization(_)));
    }

    proptest! {
        #[test]
        fn test_error_messages_not_empty(s in "\\PC+") {
            let errors = vec![
                Error::Network(s.clone()),
                Error::Protocol(s.clone()),
                Error::Consensus(s.clone()),
                Error::Training(s.clone()),
            ];

            for err in errors {
                assert!(!err.to_string().is_empty());
            }
        }
    }
}