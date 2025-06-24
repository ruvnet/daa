//! Token management for rUv and other tokens in the DAA economy

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{Result, EconomyError, CurrencyConfig};

/// Token information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    /// Token symbol (e.g., "rUv")
    pub symbol: String,
    
    /// Token name
    pub name: String,
    
    /// Number of decimal places
    pub decimals: u8,
    
    /// Total supply
    pub total_supply: Decimal,
    
    /// Maximum supply (None for unlimited)
    pub max_supply: Option<Decimal>,
    
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    
    /// Token metadata
    pub metadata: HashMap<String, String>,
}

/// Account balance information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Balance {
    /// Account ID
    pub account_id: String,
    
    /// Token symbol
    pub token_symbol: String,
    
    /// Available balance
    pub available: Decimal,
    
    /// Locked balance (in orders, staking, etc.)
    pub locked: Decimal,
    
    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,
}

impl Balance {
    /// Get total balance (available + locked)
    pub fn total(&self) -> Decimal {
        self.available + self.locked
    }
}

/// Transaction record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenTransaction {
    /// Transaction ID
    pub id: String,
    
    /// Transaction type
    pub transaction_type: TransactionType,
    
    /// Token symbol
    pub token_symbol: String,
    
    /// From account (None for minting)
    pub from_account: Option<String>,
    
    /// To account (None for burning)
    pub to_account: Option<String>,
    
    /// Amount transferred
    pub amount: Decimal,
    
    /// Fee paid
    pub fee: Decimal,
    
    /// Transaction timestamp
    pub timestamp: DateTime<Utc>,
    
    /// Transaction metadata
    pub metadata: HashMap<String, String>,
}

/// Types of token transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionType {
    /// Transfer between accounts
    Transfer,
    
    /// Mint new tokens
    Mint,
    
    /// Burn existing tokens
    Burn,
    
    /// Lock tokens (for staking, orders, etc.)
    Lock,
    
    /// Unlock previously locked tokens
    Unlock,
    
    /// Reward distribution
    Reward,
    
    /// Fee payment
    Fee,
}

/// Token manager handling all token operations
pub struct TokenManager {
    /// Token definitions
    tokens: Arc<RwLock<HashMap<String, Token>>>,
    
    /// Account balances (account_id -> token_symbol -> balance)
    balances: Arc<RwLock<HashMap<String, HashMap<String, Balance>>>>,
    
    /// Transaction history
    transactions: Arc<RwLock<Vec<TokenTransaction>>>,
    
    /// Base currency configuration
    base_currency: CurrencyConfig,
}

impl TokenManager {
    /// Create a new token manager
    pub fn new(base_currency: &CurrencyConfig) -> Result<Self> {
        Ok(Self {
            tokens: Arc::new(RwLock::new(HashMap::new())),
            balances: Arc::new(RwLock::new(HashMap::new())),
            transactions: Arc::new(RwLock::new(Vec::new())),
            base_currency: base_currency.clone(),
        })
    }

    /// Initialize the token manager with base currency
    pub async fn initialize(&mut self) -> Result<()> {
        // Create the base rUv token
        let ruv_token = Token {
            symbol: self.base_currency.symbol.clone(),
            name: self.base_currency.name.clone(),
            decimals: self.base_currency.decimals,
            total_supply: self.base_currency.initial_supply,
            max_supply: self.base_currency.max_supply,
            created_at: Utc::now(),
            metadata: HashMap::new(),
        };

        self.tokens.write().await.insert(ruv_token.symbol.clone(), ruv_token);
        
        tracing::info!(
            "Initialized token manager with {} ({}) - Initial supply: {}",
            self.base_currency.name,
            self.base_currency.symbol,
            self.base_currency.initial_supply
        );
        
        Ok(())
    }

    /// Create a new token
    pub async fn create_token(
        &mut self,
        symbol: String,
        name: String,
        decimals: u8,
        initial_supply: Decimal,
        max_supply: Option<Decimal>,
    ) -> Result<Token> {
        let mut tokens = self.tokens.write().await;
        
        if tokens.contains_key(&symbol) {
            return Err(EconomyError::InvalidTransaction(
                format!("Token {} already exists", symbol)
            ));
        }

        let token = Token {
            symbol: symbol.clone(),
            name,
            decimals,
            total_supply: initial_supply,
            max_supply,
            created_at: Utc::now(),
            metadata: HashMap::new(),
        };

        tokens.insert(symbol, token.clone());
        
        tracing::info!("Created new token: {} ({})", token.name, token.symbol);
        
        Ok(token)
    }

    /// Get token information
    pub async fn get_token(&self, symbol: &str) -> Result<Token> {
        self.tokens
            .read()
            .await
            .get(symbol)
            .cloned()
            .ok_or_else(|| EconomyError::TokenNotFound(symbol.to_string()))
    }

    /// Create balance for an account
    pub async fn create_balance(&mut self, account_id: &str, initial_amount: Decimal) -> Result<()> {
        let mut balances = self.balances.write().await;
        
        let account_balances = balances
            .entry(account_id.to_string())
            .or_insert_with(HashMap::new);

        let balance = Balance {
            account_id: account_id.to_string(),
            token_symbol: self.base_currency.symbol.clone(),
            available: initial_amount,
            locked: Decimal::ZERO,
            updated_at: Utc::now(),
        };

        account_balances.insert(self.base_currency.symbol.clone(), balance);
        
        tracing::debug!(
            "Created balance for account {}: {} {}",
            account_id,
            initial_amount,
            self.base_currency.symbol
        );
        
        Ok(())
    }

    /// Get account balance for specific token
    pub async fn get_balance(&self, account_id: &str) -> Result<Decimal> {
        self.get_token_balance(account_id, &self.base_currency.symbol).await
    }

    /// Get account balance for specific token
    pub async fn get_token_balance(&self, account_id: &str, token_symbol: &str) -> Result<Decimal> {
        let balances = self.balances.read().await;
        
        if let Some(account_balances) = balances.get(account_id) {
            if let Some(balance) = account_balances.get(token_symbol) {
                Ok(balance.available)
            } else {
                Ok(Decimal::ZERO)
            }
        } else {
            Ok(Decimal::ZERO)
        }
    }

    /// Get all balances for an account
    pub async fn get_all_balances(&self, account_id: &str) -> Result<Vec<Balance>> {
        let balances = self.balances.read().await;
        
        if let Some(account_balances) = balances.get(account_id) {
            Ok(account_balances.values().cloned().collect())
        } else {
            Ok(Vec::new())
        }
    }

    /// Transfer tokens between accounts
    pub async fn transfer(
        &mut self,
        from_account: &str,
        to_account: &str,
        amount: Decimal,
        fee: Decimal,
    ) -> Result<String> {
        self.transfer_token(
            from_account,
            to_account,
            &self.base_currency.symbol,
            amount,
            fee,
        ).await
    }

    /// Transfer specific token between accounts
    pub async fn transfer_token(
        &mut self,
        from_account: &str,
        to_account: &str,
        token_symbol: &str,
        amount: Decimal,
        fee: Decimal,
    ) -> Result<String> {
        if amount <= Decimal::ZERO {
            return Err(EconomyError::InvalidTransaction("Amount must be positive".to_string()));
        }

        let mut balances = self.balances.write().await;
        
        // Check sender balance
        let from_balances = balances
            .get_mut(from_account)
            .ok_or_else(|| EconomyError::AccountNotFound(from_account.to_string()))?;
        
        let from_balance = from_balances
            .get_mut(token_symbol)
            .ok_or_else(|| EconomyError::TokenNotFound(token_symbol.to_string()))?;

        let total_amount = amount + fee;
        if from_balance.available < total_amount {
            return Err(EconomyError::InsufficientBalance {
                account_id: from_account.to_string(),
                required: total_amount,
                available: from_balance.available,
            });
        }

        // Deduct from sender
        from_balance.available -= total_amount;
        from_balance.updated_at = Utc::now();

        // Add to receiver
        let to_balances = balances
            .entry(to_account.to_string())
            .or_insert_with(HashMap::new);
        
        let to_balance = to_balances
            .entry(token_symbol.to_string())
            .or_insert_with(|| Balance {
                account_id: to_account.to_string(),
                token_symbol: token_symbol.to_string(),
                available: Decimal::ZERO,
                locked: Decimal::ZERO,
                updated_at: Utc::now(),
            });

        to_balance.available += amount;
        to_balance.updated_at = Utc::now();

        // Record transaction
        let transaction_id = Uuid::new_v4().to_string();
        let transaction = TokenTransaction {
            id: transaction_id.clone(),
            transaction_type: TransactionType::Transfer,
            token_symbol: token_symbol.to_string(),
            from_account: Some(from_account.to_string()),
            to_account: Some(to_account.to_string()),
            amount,
            fee,
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        };

        self.transactions.write().await.push(transaction);

        tracing::info!(
            "Transfer completed: {} {} from {} to {}, fee: {} {}",
            amount, token_symbol, from_account, to_account, fee, token_symbol
        );

        Ok(transaction_id)
    }

    /// Mint new tokens to an account
    pub async fn mint(&mut self, account_id: &str, amount: Decimal) -> Result<String> {
        self.mint_token(account_id, &self.base_currency.symbol, amount).await
    }

    /// Mint specific tokens to an account
    pub async fn mint_token(&mut self, account_id: &str, token_symbol: &str, amount: Decimal) -> Result<String> {
        if amount <= Decimal::ZERO {
            return Err(EconomyError::InvalidTransaction("Amount must be positive".to_string()));
        }

        // Check token exists and supply limits
        {
            let mut tokens = self.tokens.write().await;
            let token = tokens
                .get_mut(token_symbol)
                .ok_or_else(|| EconomyError::TokenNotFound(token_symbol.to_string()))?;

            if let Some(max_supply) = token.max_supply {
                if token.total_supply + amount > max_supply {
                    return Err(EconomyError::InvalidTransaction(
                        format!("Minting would exceed max supply of {}", max_supply)
                    ));
                }
            }

            token.total_supply += amount;
        }

        // Add to account balance
        let mut balances = self.balances.write().await;
        let account_balances = balances
            .entry(account_id.to_string())
            .or_insert_with(HashMap::new);
        
        let balance = account_balances
            .entry(token_symbol.to_string())
            .or_insert_with(|| Balance {
                account_id: account_id.to_string(),
                token_symbol: token_symbol.to_string(),
                available: Decimal::ZERO,
                locked: Decimal::ZERO,
                updated_at: Utc::now(),
            });

        balance.available += amount;
        balance.updated_at = Utc::now();

        // Record transaction
        let transaction_id = Uuid::new_v4().to_string();
        let transaction = TokenTransaction {
            id: transaction_id.clone(),
            transaction_type: TransactionType::Mint,
            token_symbol: token_symbol.to_string(),
            from_account: None,
            to_account: Some(account_id.to_string()),
            amount,
            fee: Decimal::ZERO,
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        };

        self.transactions.write().await.push(transaction);

        tracing::info!("Minted {} {} to account {}", amount, token_symbol, account_id);

        Ok(transaction_id)
    }

    /// Burn tokens from an account
    pub async fn burn(&mut self, account_id: &str, amount: Decimal) -> Result<String> {
        self.burn_token(account_id, &self.base_currency.symbol, amount).await
    }

    /// Burn specific tokens from an account
    pub async fn burn_token(&mut self, account_id: &str, token_symbol: &str, amount: Decimal) -> Result<String> {
        if amount <= Decimal::ZERO {
            return Err(EconomyError::InvalidTransaction("Amount must be positive".to_string()));
        }

        let mut balances = self.balances.write().await;
        
        // Check account balance
        let account_balances = balances
            .get_mut(account_id)
            .ok_or_else(|| EconomyError::AccountNotFound(account_id.to_string()))?;
        
        let balance = account_balances
            .get_mut(token_symbol)
            .ok_or_else(|| EconomyError::TokenNotFound(token_symbol.to_string()))?;

        if balance.available < amount {
            return Err(EconomyError::InsufficientBalance {
                account_id: account_id.to_string(),
                required: amount,
                available: balance.available,
            });
        }

        // Deduct from account
        balance.available -= amount;
        balance.updated_at = Utc::now();

        // Reduce total supply
        {
            let mut tokens = self.tokens.write().await;
            let token = tokens
                .get_mut(token_symbol)
                .ok_or_else(|| EconomyError::TokenNotFound(token_symbol.to_string()))?;

            token.total_supply -= amount;
        }

        // Record transaction
        let transaction_id = Uuid::new_v4().to_string();
        let transaction = TokenTransaction {
            id: transaction_id.clone(),
            transaction_type: TransactionType::Burn,
            token_symbol: token_symbol.to_string(),
            from_account: Some(account_id.to_string()),
            to_account: None,
            amount,
            fee: Decimal::ZERO,
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        };

        self.transactions.write().await.push(transaction);

        tracing::info!("Burned {} {} from account {}", amount, token_symbol, account_id);

        Ok(transaction_id)
    }

    /// Lock tokens in an account
    pub async fn lock_tokens(
        &mut self,
        account_id: &str,
        token_symbol: &str,
        amount: Decimal,
    ) -> Result<String> {
        if amount <= Decimal::ZERO {
            return Err(EconomyError::InvalidTransaction("Amount must be positive".to_string()));
        }

        let mut balances = self.balances.write().await;
        
        let account_balances = balances
            .get_mut(account_id)
            .ok_or_else(|| EconomyError::AccountNotFound(account_id.to_string()))?;
        
        let balance = account_balances
            .get_mut(token_symbol)
            .ok_or_else(|| EconomyError::TokenNotFound(token_symbol.to_string()))?;

        if balance.available < amount {
            return Err(EconomyError::InsufficientBalance {
                account_id: account_id.to_string(),
                required: amount,
                available: balance.available,
            });
        }

        // Move from available to locked
        balance.available -= amount;
        balance.locked += amount;
        balance.updated_at = Utc::now();

        // Record transaction
        let transaction_id = Uuid::new_v4().to_string();
        let transaction = TokenTransaction {
            id: transaction_id.clone(),
            transaction_type: TransactionType::Lock,
            token_symbol: token_symbol.to_string(),
            from_account: Some(account_id.to_string()),
            to_account: Some(account_id.to_string()),
            amount,
            fee: Decimal::ZERO,
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        };

        self.transactions.write().await.push(transaction);

        tracing::info!("Locked {} {} for account {}", amount, token_symbol, account_id);

        Ok(transaction_id)
    }

    /// Unlock tokens in an account
    pub async fn unlock_tokens(
        &mut self,
        account_id: &str,
        token_symbol: &str,
        amount: Decimal,
    ) -> Result<String> {
        if amount <= Decimal::ZERO {
            return Err(EconomyError::InvalidTransaction("Amount must be positive".to_string()));
        }

        let mut balances = self.balances.write().await;
        
        let account_balances = balances
            .get_mut(account_id)
            .ok_or_else(|| EconomyError::AccountNotFound(account_id.to_string()))?;
        
        let balance = account_balances
            .get_mut(token_symbol)
            .ok_or_else(|| EconomyError::TokenNotFound(token_symbol.to_string()))?;

        if balance.locked < amount {
            return Err(EconomyError::InvalidTransaction(
                format!("Insufficient locked balance: {} < {}", balance.locked, amount)
            ));
        }

        // Move from locked to available
        balance.locked -= amount;
        balance.available += amount;
        balance.updated_at = Utc::now();

        // Record transaction
        let transaction_id = Uuid::new_v4().to_string();
        let transaction = TokenTransaction {
            id: transaction_id.clone(),
            transaction_type: TransactionType::Unlock,
            token_symbol: token_symbol.to_string(),
            from_account: Some(account_id.to_string()),
            to_account: Some(account_id.to_string()),
            amount,
            fee: Decimal::ZERO,
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        };

        self.transactions.write().await.push(transaction);

        tracing::info!("Unlocked {} {} for account {}", amount, token_symbol, account_id);

        Ok(transaction_id)
    }

    /// Get total supply of base currency
    pub async fn get_total_supply(&self) -> Result<Decimal> {
        self.get_token_total_supply(&self.base_currency.symbol).await
    }

    /// Get total supply of specific token
    pub async fn get_token_total_supply(&self, token_symbol: &str) -> Result<Decimal> {
        let token = self.get_token(token_symbol).await?;
        Ok(token.total_supply)
    }

    /// Get transaction history
    pub async fn get_transactions(&self, account_id: Option<&str>) -> Result<Vec<TokenTransaction>> {
        let transactions = self.transactions.read().await;
        
        if let Some(account) = account_id {
            Ok(transactions
                .iter()
                .filter(|tx| {
                    tx.from_account.as_ref() == Some(&account.to_string()) ||
                    tx.to_account.as_ref() == Some(&account.to_string())
                })
                .cloned()
                .collect())
        } else {
            Ok(transactions.clone())
        }
    }

    /// Get transaction by ID
    pub async fn get_transaction(&self, transaction_id: &str) -> Result<Option<TokenTransaction>> {
        let transactions = self.transactions.read().await;
        Ok(transactions.iter().find(|tx| tx.id == transaction_id).cloned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_token_manager_creation() {
        let config = CurrencyConfig::default();
        let manager = TokenManager::new(&config);
        assert!(manager.is_ok());
    }

    #[tokio::test]
    async fn test_token_initialization() {
        let config = CurrencyConfig::default();
        let mut manager = TokenManager::new(&config).unwrap();
        
        manager.initialize().await.unwrap();
        
        let token = manager.get_token("rUv").await.unwrap();
        assert_eq!(token.symbol, "rUv");
        assert_eq!(token.total_supply, config.initial_supply);
    }

    #[tokio::test]
    async fn test_balance_operations() {
        let config = CurrencyConfig::default();
        let mut manager = TokenManager::new(&config).unwrap();
        manager.initialize().await.unwrap();
        
        // Create balance
        manager.create_balance("test-account", Decimal::from(100)).await.unwrap();
        
        // Check balance
        let balance = manager.get_balance("test-account").await.unwrap();
        assert_eq!(balance, Decimal::from(100));
    }

    #[tokio::test]
    async fn test_token_transfer() {
        let config = CurrencyConfig::default();
        let mut manager = TokenManager::new(&config).unwrap();
        manager.initialize().await.unwrap();
        
        // Create accounts
        manager.create_balance("account1", Decimal::from(100)).await.unwrap();
        manager.create_balance("account2", Decimal::ZERO).await.unwrap();
        
        // Transfer
        let tx_id = manager.transfer("account1", "account2", Decimal::from(50), Decimal::from(1)).await.unwrap();
        assert!(!tx_id.is_empty());
        
        // Check balances
        let balance1 = manager.get_balance("account1").await.unwrap();
        let balance2 = manager.get_balance("account2").await.unwrap();
        
        assert_eq!(balance1, Decimal::from(49)); // 100 - 50 - 1 (fee)
        assert_eq!(balance2, Decimal::from(50));
    }

    #[tokio::test]
    async fn test_mint_and_burn() {
        let config = CurrencyConfig::default();
        let mut manager = TokenManager::new(&config).unwrap();
        manager.initialize().await.unwrap();
        
        manager.create_balance("test-account", Decimal::ZERO).await.unwrap();
        
        // Mint tokens
        manager.mint("test-account", Decimal::from(100)).await.unwrap();
        let balance = manager.get_balance("test-account").await.unwrap();
        assert_eq!(balance, Decimal::from(100));
        
        // Burn tokens
        manager.burn("test-account", Decimal::from(30)).await.unwrap();
        let balance = manager.get_balance("test-account").await.unwrap();
        assert_eq!(balance, Decimal::from(70));
    }
}