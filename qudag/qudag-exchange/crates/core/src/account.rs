//! Account management for QuDAG Exchange

use serde::{Deserialize, Serialize};
use std::fmt;

/// Unique identifier for an account
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AccountId(pub String);

impl fmt::Display for AccountId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Account information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: AccountId,
    pub created_at: u64,
    pub public_key: Vec<u8>,
}

impl Account {
    pub fn new(id: AccountId, public_key: Vec<u8>) -> Self {
        Self {
            id,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            public_key,
        }
    }
}