//! Account management for DAA Economy

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{Result, EconomyError};

/// Account information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    /// Account ID
    pub id: String,
    
    /// Associated agent ID
    pub agent_id: String,
    
    /// Account creation timestamp
    pub created_at: DateTime<Utc>,
    
    /// Account status
    pub status: AccountStatus,
    
    /// Account metadata
    pub metadata: HashMap<String, String>,
}

/// Account status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccountStatus {
    Active,
    Suspended,
    Closed,
}

/// Account manager
pub struct AccountManager {
    /// Account storage
    accounts: Arc<RwLock<HashMap<String, Account>>>,
}

impl AccountManager {
    /// Create new account manager
    pub fn new() -> Self {
        Self {
            accounts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new account
    pub async fn create_account(&mut self, agent_id: String) -> Result<Account> {
        let account = Account {
            id: Uuid::new_v4().to_string(),
            agent_id,
            created_at: Utc::now(),
            status: AccountStatus::Active,
            metadata: HashMap::new(),
        };

        self.accounts.write().await.insert(account.id.clone(), account.clone());
        Ok(account)
    }

    /// Get account by ID
    pub async fn get_account(&self, account_id: &str) -> Result<Account> {
        self.accounts
            .read()
            .await
            .get(account_id)
            .cloned()
            .ok_or_else(|| EconomyError::AccountNotFound(account_id.to_string()))
    }

    /// Get account count
    pub async fn get_account_count(&self) -> Result<u64> {
        Ok(self.accounts.read().await.len() as u64)
    }
}

impl Default for AccountManager {
    fn default() -> Self {
        Self::new()
    }
}