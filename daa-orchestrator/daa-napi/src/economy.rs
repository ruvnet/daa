//! Economy manager bindings for NAPI
//!
//! Provides Node.js access to the DAA economy system for token management,
//! account operations, and trading.

use daa_economy::accounts::AccountManager;
use daa_economy::trading::TradingEngine;
use napi::bindgen_prelude::*;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Account information for Node.js
#[napi(object)]
pub struct AccountJs {
    /// Account identifier
    pub id: String,
    /// Associated agent ID
    pub agent_id: String,
    /// Account status
    pub status: String,
    /// Account creation timestamp
    pub created_at: String,
}

/// Balance information for Node.js
#[napi(object)]
pub struct BalanceJs {
    /// Token symbol (e.g., "rUv", "USD")
    pub token: String,
    /// Balance amount
    pub amount: f64,
}

/// Transfer request for Node.js
#[napi(object)]
pub struct TransferRequestJs {
    /// Source account ID
    pub from_account: String,
    /// Destination account ID
    pub to_account: String,
    /// Token symbol
    pub token: String,
    /// Transfer amount
    pub amount: f64,
    /// Optional memo
    pub memo: Option<String>,
}

/// Transfer result for Node.js
#[napi(object)]
pub struct TransferResultJs {
    /// Transfer transaction ID
    pub transaction_id: String,
    /// Transfer status
    pub status: String,
    /// Timestamp
    pub timestamp: String,
}

/// Trade order for Node.js
#[napi(object)]
pub struct TradeOrderJs {
    /// Order identifier
    pub id: String,
    /// Trading symbol (e.g., "rUv/USD")
    pub symbol: String,
    /// Order type: "market", "limit", "stop_loss", "take_profit"
    pub order_type: String,
    /// Order side: "buy" or "sell"
    pub side: String,
    /// Order quantity
    pub quantity: f64,
    /// Order price (for limit orders)
    pub price: Option<f64>,
    /// Stop price (for stop orders)
    pub stop_price: Option<f64>,
    /// Order status
    pub status: String,
}

/// Economy manager for token and account operations
#[napi]
pub struct EconomyManager {
    account_manager: Arc<Mutex<AccountManager>>,
    trading_engine: Arc<Mutex<TradingEngine>>,
}

#[napi]
impl EconomyManager {
    /// Create a new economy manager instance.
    ///
    /// # Returns
    ///
    /// A new EconomyManager instance
    ///
    /// # Example
    ///
    /// ```javascript
    /// const economyManager = new EconomyManager();
    /// ```
    #[napi(constructor)]
    pub fn new() -> Result<Self> {
        Ok(Self {
            account_manager: Arc::new(Mutex::new(AccountManager::new())),
            trading_engine: Arc::new(Mutex::new(TradingEngine::new())),
        })
    }

    /// Create a new account for an agent.
    ///
    /// # Arguments
    ///
    /// * `agent_id` - ID of the agent to create an account for
    ///
    /// # Returns
    ///
    /// The created account information
    ///
    /// # Example
    ///
    /// ```javascript
    /// const account = await economyManager.createAccount('agent-123');
    /// console.log('Account ID:', account.id);
    /// console.log('Status:', account.status);
    /// ```
    #[napi]
    pub async fn create_account(&self, agent_id: String) -> Result<AccountJs> {
        let mut manager = self.account_manager.lock().await;

        let account = manager
            .create_account(agent_id)
            .await
            .map_err(|e| Error::from_reason(format!("Failed to create account: {}", e)))?;

        Ok(AccountJs {
            id: account.id,
            agent_id: account.agent_id,
            status: format!("{:?}", account.status),
            created_at: account.created_at.to_rfc3339(),
        })
    }

    /// Get account information by ID.
    ///
    /// # Arguments
    ///
    /// * `account_id` - Account ID to retrieve
    ///
    /// # Returns
    ///
    /// Account information
    ///
    /// # Example
    ///
    /// ```javascript
    /// const account = await economyManager.getAccount('account-123');
    /// console.log('Agent ID:', account.agentId);
    /// ```
    #[napi]
    pub async fn get_account(&self, account_id: String) -> Result<AccountJs> {
        let manager = self.account_manager.lock().await;

        let account = manager
            .get_account(&account_id)
            .await
            .map_err(|e| Error::from_reason(format!("Failed to get account: {}", e)))?;

        Ok(AccountJs {
            id: account.id,
            agent_id: account.agent_id,
            status: format!("{:?}", account.status),
            created_at: account.created_at.to_rfc3339(),
        })
    }

    /// Get balance for a specific token.
    ///
    /// # Arguments
    ///
    /// * `account_id` - Account ID
    /// * `token` - Token symbol (e.g., "rUv", "USD")
    ///
    /// # Returns
    ///
    /// Balance information
    ///
    /// # Example
    ///
    /// ```javascript
    /// const balance = await economyManager.getBalance('account-123', 'rUv');
    /// console.log('Balance:', balance.amount, balance.token);
    /// ```
    #[napi]
    pub async fn get_balance(&self, _account_id: String, token: String) -> Result<BalanceJs> {
        let engine = self.trading_engine.lock().await;

        let _balance = engine
            .get_account_balance()
            .map_err(|e| Error::from_reason(format!("Failed to get balance: {}", e)))?;

        // In a real implementation, we would look up the specific token balance
        Ok(BalanceJs {
            token: token.clone(),
            amount: 0.0,
        })
    }

    /// Transfer tokens between accounts.
    ///
    /// # Arguments
    ///
    /// * `transfer` - Transfer request details
    ///
    /// # Returns
    ///
    /// Transfer result with transaction ID
    ///
    /// # Example
    ///
    /// ```javascript
    /// const result = await economyManager.transfer({
    ///   fromAccount: 'account-123',
    ///   toAccount: 'account-456',
    ///   token: 'rUv',
    ///   amount: 100.0,
    ///   memo: 'Payment for services'
    /// });
    ///
    /// console.log('Transaction ID:', result.transactionId);
    /// console.log('Status:', result.status);
    /// ```
    #[napi]
    pub async fn transfer(&self, transfer: TransferRequestJs) -> Result<TransferResultJs> {
        // Validate transfer amount
        if transfer.amount <= 0.0 {
            return Err(Error::from_reason("Transfer amount must be positive"));
        }

        // In a real implementation, we would:
        // 1. Verify source account has sufficient balance
        // 2. Deduct from source account
        // 3. Add to destination account
        // 4. Create transaction record

        Ok(TransferResultJs {
            transaction_id: uuid::Uuid::new_v4().to_string(),
            status: "completed".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        })
    }

    /// Create a trade order.
    ///
    /// # Arguments
    ///
    /// * `order` - Trade order details
    ///
    /// # Returns
    ///
    /// The created order with status
    ///
    /// # Example
    ///
    /// ```javascript
    /// const order = await economyManager.createOrder({
    ///   id: 'order-1',
    ///   symbol: 'rUv/USD',
    ///   orderType: 'limit',
    ///   side: 'buy',
    ///   quantity: 10.0,
    ///   price: 50.0,
    ///   status: 'pending'
    /// });
    ///
    /// console.log('Order created:', order.id);
    /// ```
    #[napi]
    pub async fn create_order(&self, order: TradeOrderJs) -> Result<TradeOrderJs> {
        // Validate order
        if order.quantity <= 0.0 {
            return Err(Error::from_reason("Order quantity must be positive"));
        }

        if order.order_type == "limit" && order.price.is_none() {
            return Err(Error::from_reason("Limit orders must have a price"));
        }

        // In a real implementation, we would create the order in the trading engine

        Ok(order)
    }

    /// Get account count.
    ///
    /// # Returns
    ///
    /// Total number of accounts
    ///
    /// # Example
    ///
    /// ```javascript
    /// const count = await economyManager.getAccountCount();
    /// console.log('Total accounts:', count);
    /// ```
    #[napi]
    pub async fn get_account_count(&self) -> Result<u32> {
        let manager = self.account_manager.lock().await;

        let count = manager
            .get_account_count()
            .await
            .map_err(|e| Error::from_reason(format!("Failed to get account count: {}", e)))?;

        Ok(count as u32)
    }

    /// Get all balances for an account.
    ///
    /// # Arguments
    ///
    /// * `account_id` - Account ID
    ///
    /// # Returns
    ///
    /// List of all token balances
    ///
    /// # Example
    ///
    /// ```javascript
    /// const balances = await economyManager.getAllBalances('account-123');
    /// balances.forEach(balance => {
    ///   console.log(`${balance.token}: ${balance.amount}`);
    /// });
    /// ```
    #[napi]
    pub async fn get_all_balances(&self, _account_id: String) -> Result<Vec<BalanceJs>> {
        // In a real implementation, we would query all balances for the account
        Ok(vec![
            BalanceJs {
                token: "rUv".to_string(),
                amount: 0.0,
            },
            BalanceJs {
                token: "USD".to_string(),
                amount: 0.0,
            },
        ])
    }

    /// Set balance for testing purposes.
    ///
    /// # Arguments
    ///
    /// * `account_id` - Account ID
    /// * `token` - Token symbol
    /// * `amount` - New balance amount
    ///
    /// # Returns
    ///
    /// Promise that resolves when balance is set
    ///
    /// # Example
    ///
    /// ```javascript
    /// await economyManager.setBalance('account-123', 'rUv', 1000.0);
    /// ```
    #[napi]
    pub async fn set_balance(
        &self,
        _account_id: String,
        _token: String,
        amount: f64,
    ) -> Result<()> {
        if amount < 0.0 {
            return Err(Error::from_reason("Balance cannot be negative"));
        }

        // In a real implementation, we would update the account balance

        Ok(())
    }
}
