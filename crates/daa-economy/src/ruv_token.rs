//! rUv Token Management with QuDAG Integration
//!
//! This module handles the native rUv token operations, including minting, burning,
//! transfers, and staking mechanisms built on top of QuDAG infrastructure.

use crate::{Result, EconomyError};
use async_trait::async_trait;
use daa_chain::{Address, TxHash, BlockchainAdapter, Transaction};
use qudag_crypto::CryptoProvider;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use log::{info, debug, warn};

/// rUv token information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuvToken {
    pub address: Address,
    pub symbol: String,
    pub decimals: u8,
    pub total_supply: Decimal,
    pub circulating_supply: Decimal,
}

impl RuvToken {
    pub fn new() -> Self {
        RuvToken {
            address: Address::from_hex("0x0000000000000000000000000000000000000001").unwrap(),
            symbol: "rUv".to_string(),
            decimals: 18,
            total_supply: Decimal::new(1_000_000_000, 18), // 1B max supply
            circulating_supply: Decimal::ZERO,
        }
    }
    
    /// Convert amount to smallest unit (wei equivalent)
    pub fn to_wei(&self, amount: Decimal) -> u128 {
        let multiplier = Decimal::new(10_i64.pow(self.decimals as u32), 0);
        (amount * multiplier).to_u128().unwrap_or(0)
    }
    
    /// Convert from smallest unit to decimal
    pub fn from_wei(&self, wei: u128) -> Decimal {
        let divisor = Decimal::new(10_i64.pow(self.decimals as u32), 0);
        Decimal::from(wei) / divisor
    }
}

/// rUv token manager handles all token operations
pub struct RuvTokenManager {
    token: RuvToken,
    blockchain_adapter: Arc<dyn BlockchainAdapter>,
    crypto_provider: Arc<dyn CryptoProvider>,
    balances: Arc<RwLock<HashMap<Address, Decimal>>>,
    locked_balances: Arc<RwLock<HashMap<Address, Decimal>>>,
    allowances: Arc<RwLock<HashMap<Address, HashMap<Address, Decimal>>>>,
    total_staked: Arc<RwLock<Decimal>>,
    minters: Arc<RwLock<Vec<Address>>>,
    burners: Arc<RwLock<Vec<Address>>>,
}

impl RuvTokenManager {
    /// Create a new rUv token manager
    pub async fn new(
        token: RuvToken,
        blockchain_adapter: Arc<dyn BlockchainAdapter>,
        crypto_provider: Arc<dyn CryptoProvider>,
    ) -> Result<Self> {
        info!("Initializing rUv Token Manager");
        
        Ok(RuvTokenManager {
            token,
            blockchain_adapter,
            crypto_provider,
            balances: Arc::new(RwLock::new(HashMap::new())),
            locked_balances: Arc::new(RwLock::new(HashMap::new())),
            allowances: Arc::new(RwLock::new(HashMap::new())),
            total_staked: Arc::new(RwLock::new(Decimal::ZERO)),
            minters: Arc::new(RwLock::new(Vec::new())),
            burners: Arc::new(RwLock::new(Vec::new())),
        })
    }
    
    /// Get token information
    pub fn get_token_info(&self) -> &RuvToken {
        &self.token
    }
    
    /// Get balance for an address
    pub async fn get_balance(&self, address: &Address) -> Result<Decimal> {
        let balances = self.balances.read().unwrap();
        Ok(balances.get(address).copied().unwrap_or(Decimal::ZERO))
    }
    
    /// Get locked balance for an address
    pub async fn get_locked_balance(&self, address: &Address) -> Result<Decimal> {
        let locked_balances = self.locked_balances.read().unwrap();
        Ok(locked_balances.get(address).copied().unwrap_or(Decimal::ZERO))
    }
    
    /// Get available balance (total - locked)
    pub async fn get_available_balance(&self, address: &Address) -> Result<Decimal> {
        let total = self.get_balance(address).await?;
        let locked = self.get_locked_balance(address).await?;
        Ok(total - locked)
    }
    
    /// Mint new rUv tokens
    pub async fn mint(&self, to: &Address, amount: Decimal) -> Result<TxHash> {
        info!("Minting {} rUv to {}", amount, to);
        
        // Check if minting would exceed total supply
        let current_supply = {
            let balances = self.balances.read().unwrap();
            balances.values().sum::<Decimal>()
        };
        
        if current_supply + amount > self.token.total_supply {
            return Err(EconomyError::InvalidToken(
                "Minting would exceed total supply".to_string()
            ));
        }
        
        // Create mint transaction
        let mint_data = self.create_mint_transaction_data(to, amount)?;
        let mint_tx = Transaction {
            from: self.token.address.clone(),
            to: Some(to.clone()),
            value: 0,
            data: mint_data,
            gas_limit: 100000,
            gas_price: Some(self.blockchain_adapter.get_gas_price().await?),
            max_fee_per_gas: None,
            max_priority_fee_per_gas: None,
            nonce: Some(self.blockchain_adapter.get_nonce(&self.token.address).await?),
        };
        
        // Send transaction
        let tx_hash = self.blockchain_adapter.send_transaction(mint_tx).await
            .map_err(|e| EconomyError::ExchangeError(e.to_string()))?;
        
        // Update local balance
        {
            let mut balances = self.balances.write().unwrap();
            let current_balance = balances.get(to).copied().unwrap_or(Decimal::ZERO);
            balances.insert(to.clone(), current_balance + amount);
        }
        
        info!("Minted {} rUv to {} with tx hash: {}", amount, to, tx_hash);
        Ok(tx_hash)
    }
    
    /// Burn rUv tokens
    pub async fn burn(&self, from: &Address, amount: Decimal) -> Result<TxHash> {
        info!("Burning {} rUv from {}", amount, from);
        
        // Check balance
        let balance = self.get_available_balance(from).await?;
        if balance < amount {
            return Err(EconomyError::InsufficientBalance {
                required: amount,
                available: balance,
            });
        }
        
        // Create burn transaction
        let burn_data = self.create_burn_transaction_data(from, amount)?;
        let burn_tx = Transaction {
            from: from.clone(),
            to: Some(self.token.address.clone()),
            value: 0,
            data: burn_data,
            gas_limit: 100000,
            gas_price: Some(self.blockchain_adapter.get_gas_price().await?),
            max_fee_per_gas: None,
            max_priority_fee_per_gas: None,
            nonce: Some(self.blockchain_adapter.get_nonce(from).await?),
        };
        
        // Send transaction
        let tx_hash = self.blockchain_adapter.send_transaction(burn_tx).await
            .map_err(|e| EconomyError::ExchangeError(e.to_string()))?;
        
        // Update local balance
        {
            let mut balances = self.balances.write().unwrap();
            let current_balance = balances.get(from).copied().unwrap_or(Decimal::ZERO);
            balances.insert(from.clone(), current_balance - amount);
        }
        
        info!("Burned {} rUv from {} with tx hash: {}", amount, from, tx_hash);
        Ok(tx_hash)
    }
    
    /// Transfer rUv tokens
    pub async fn transfer(&self, from: &Address, to: &Address, amount: Decimal) -> Result<TxHash> {
        debug!("Transferring {} rUv from {} to {}", amount, from, to);
        
        // Check available balance
        let available = self.get_available_balance(from).await?;
        if available < amount {
            return Err(EconomyError::InsufficientBalance {
                required: amount,
                available,
            });
        }
        
        // Create transfer transaction
        let transfer_data = self.create_transfer_transaction_data(from, to, amount)?;
        let transfer_tx = Transaction {
            from: from.clone(),
            to: Some(self.token.address.clone()),
            value: 0,
            data: transfer_data,
            gas_limit: 60000,
            gas_price: Some(self.blockchain_adapter.get_gas_price().await?),
            max_fee_per_gas: None,
            max_priority_fee_per_gas: None,
            nonce: Some(self.blockchain_adapter.get_nonce(from).await?),
        };
        
        // Send transaction
        let tx_hash = self.blockchain_adapter.send_transaction(transfer_tx).await
            .map_err(|e| EconomyError::ExchangeError(e.to_string()))?;
        
        // Update local balances
        {
            let mut balances = self.balances.write().unwrap();
            
            // Decrease sender balance
            let from_balance = balances.get(from).copied().unwrap_or(Decimal::ZERO);
            balances.insert(from.clone(), from_balance - amount);
            
            // Increase receiver balance
            let to_balance = balances.get(to).copied().unwrap_or(Decimal::ZERO);
            balances.insert(to.clone(), to_balance + amount);
        }
        
        debug!("Transferred {} rUv from {} to {} with tx hash: {}", amount, from, to, tx_hash);
        Ok(tx_hash)
    }
    
    /// Lock tokens for staking
    pub async fn lock(&self, address: &Address, amount: Decimal) -> Result<TxHash> {
        info!("Locking {} rUv for address {}", amount, address);
        
        // Check available balance
        let available = self.get_available_balance(address).await?;
        if available < amount {
            return Err(EconomyError::InsufficientBalance {
                required: amount,
                available,
            });
        }
        
        // Create lock transaction
        let lock_data = self.create_lock_transaction_data(address, amount)?;
        let lock_tx = Transaction {
            from: address.clone(),
            to: Some(self.token.address.clone()),
            value: 0,
            data: lock_data,
            gas_limit: 80000,
            gas_price: Some(self.blockchain_adapter.get_gas_price().await?),
            max_fee_per_gas: None,
            max_priority_fee_per_gas: None,
            nonce: Some(self.blockchain_adapter.get_nonce(address).await?),
        };
        
        // Send transaction
        let tx_hash = self.blockchain_adapter.send_transaction(lock_tx).await
            .map_err(|e| EconomyError::ExchangeError(e.to_string()))?;
        
        // Update locked balance
        {
            let mut locked_balances = self.locked_balances.write().unwrap();
            let current_locked = locked_balances.get(address).copied().unwrap_or(Decimal::ZERO);
            locked_balances.insert(address.clone(), current_locked + amount);
        }
        
        // Update total staked
        {
            let mut total_staked = self.total_staked.write().unwrap();
            *total_staked += amount;
        }
        
        info!("Locked {} rUv for address {} with tx hash: {}", amount, address, tx_hash);
        Ok(tx_hash)
    }
    
    /// Unlock tokens from staking
    pub async fn unlock(&self, address: &Address, amount: Decimal) -> Result<TxHash> {
        info!("Unlocking {} rUv for address {}", amount, address);
        
        // Check locked balance
        let locked = self.get_locked_balance(address).await?;
        if locked < amount {
            return Err(EconomyError::InsufficientBalance {
                required: amount,
                available: locked,
            });
        }
        
        // Create unlock transaction
        let unlock_data = self.create_unlock_transaction_data(address, amount)?;
        let unlock_tx = Transaction {
            from: address.clone(),
            to: Some(self.token.address.clone()),
            value: 0,
            data: unlock_data,
            gas_limit: 80000,
            gas_price: Some(self.blockchain_adapter.get_gas_price().await?),
            max_fee_per_gas: None,
            max_priority_fee_per_gas: None,
            nonce: Some(self.blockchain_adapter.get_nonce(address).await?),
        };
        
        // Send transaction
        let tx_hash = self.blockchain_adapter.send_transaction(unlock_tx).await
            .map_err(|e| EconomyError::ExchangeError(e.to_string()))?;
        
        // Update locked balance
        {
            let mut locked_balances = self.locked_balances.write().unwrap();
            let current_locked = locked_balances.get(address).copied().unwrap_or(Decimal::ZERO);
            locked_balances.insert(address.clone(), current_locked - amount);
        }
        
        // Update total staked
        {
            let mut total_staked = self.total_staked.write().unwrap();
            *total_staked -= amount;
        }
        
        info!("Unlocked {} rUv for address {} with tx hash: {}", amount, address, tx_hash);
        Ok(tx_hash)
    }
    
    /// Get total staked amount across all addresses
    pub async fn get_total_staked(&self) -> Decimal {
        let total_staked = self.total_staked.read().unwrap();
        *total_staked
    }
    
    /// Get circulating supply (total balances)
    pub async fn get_circulating_supply(&self) -> Decimal {
        let balances = self.balances.read().unwrap();
        balances.values().sum()
    }
    
    /// Set allowance for spending
    pub async fn approve(&self, owner: &Address, spender: &Address, amount: Decimal) -> Result<TxHash> {
        debug!("Setting allowance: {} allows {} to spend {} rUv", owner, spender, amount);
        
        // Create approve transaction
        let approve_data = self.create_approve_transaction_data(owner, spender, amount)?;
        let approve_tx = Transaction {
            from: owner.clone(),
            to: Some(self.token.address.clone()),
            value: 0,
            data: approve_data,
            gas_limit: 50000,
            gas_price: Some(self.blockchain_adapter.get_gas_price().await?),
            max_fee_per_gas: None,
            max_priority_fee_per_gas: None,
            nonce: Some(self.blockchain_adapter.get_nonce(owner).await?),
        };
        
        // Send transaction
        let tx_hash = self.blockchain_adapter.send_transaction(approve_tx).await
            .map_err(|e| EconomyError::ExchangeError(e.to_string()))?;
        
        // Update allowance
        {
            let mut allowances = self.allowances.write().unwrap();
            let owner_allowances = allowances.entry(owner.clone()).or_insert_with(HashMap::new);
            owner_allowances.insert(spender.clone(), amount);
        }
        
        debug!("Set allowance with tx hash: {}", tx_hash);
        Ok(tx_hash)
    }
    
    /// Get allowance amount
    pub async fn get_allowance(&self, owner: &Address, spender: &Address) -> Decimal {
        let allowances = self.allowances.read().unwrap();
        allowances.get(owner)
            .and_then(|owner_allowances| owner_allowances.get(spender))
            .copied()
            .unwrap_or(Decimal::ZERO)
    }
    
    /// Transfer from another account (requires allowance)
    pub async fn transfer_from(
        &self,
        spender: &Address,
        from: &Address,
        to: &Address,
        amount: Decimal,
    ) -> Result<TxHash> {
        debug!("Transfer from: {} spending {} rUv from {} to {}", spender, amount, from, to);
        
        // Check allowance
        let allowance = self.get_allowance(from, spender).await;
        if allowance < amount {
            return Err(EconomyError::InsufficientBalance {
                required: amount,
                available: allowance,
            });
        }
        
        // Check balance
        let available = self.get_available_balance(from).await?;
        if available < amount {
            return Err(EconomyError::InsufficientBalance {
                required: amount,
                available,
            });
        }
        
        // Execute transfer
        let tx_hash = self.transfer(from, to, amount).await?;
        
        // Update allowance
        {
            let mut allowances = self.allowances.write().unwrap();
            if let Some(owner_allowances) = allowances.get_mut(from) {
                owner_allowances.insert(spender.clone(), allowance - amount);
            }
        }
        
        debug!("Transfer from completed with tx hash: {}", tx_hash);
        Ok(tx_hash)
    }
    
    // Transaction data creation methods
    
    fn create_mint_transaction_data(&self, to: &Address, amount: Decimal) -> Result<Vec<u8>> {
        let data = format!("MINT:{}:{}", to.to_hex(), amount.to_string());
        Ok(data.into_bytes())
    }
    
    fn create_burn_transaction_data(&self, from: &Address, amount: Decimal) -> Result<Vec<u8>> {
        let data = format!("BURN:{}:{}", from.to_hex(), amount.to_string());
        Ok(data.into_bytes())
    }
    
    fn create_transfer_transaction_data(&self, from: &Address, to: &Address, amount: Decimal) -> Result<Vec<u8>> {
        let data = format!("TRANSFER:{}:{}:{}", from.to_hex(), to.to_hex(), amount.to_string());
        Ok(data.into_bytes())
    }
    
    fn create_lock_transaction_data(&self, address: &Address, amount: Decimal) -> Result<Vec<u8>> {
        let data = format!("LOCK:{}:{}", address.to_hex(), amount.to_string());
        Ok(data.into_bytes())
    }
    
    fn create_unlock_transaction_data(&self, address: &Address, amount: Decimal) -> Result<Vec<u8>> {
        let data = format!("UNLOCK:{}:{}", address.to_hex(), amount.to_string());
        Ok(data.into_bytes())
    }
    
    fn create_approve_transaction_data(&self, owner: &Address, spender: &Address, amount: Decimal) -> Result<Vec<u8>> {
        let data = format!("APPROVE:{}:{}:{}", owner.to_hex(), spender.to_hex(), amount.to_string());
        Ok(data.into_bytes())
    }
}

/// rUv token trait for external integration
#[async_trait]
pub trait RuvTokenInterface: Send + Sync {
    async fn get_balance(&self, address: &Address) -> Result<Decimal>;
    async fn transfer(&self, from: &Address, to: &Address, amount: Decimal) -> Result<TxHash>;
    async fn mint(&self, to: &Address, amount: Decimal) -> Result<TxHash>;
    async fn burn(&self, from: &Address, amount: Decimal) -> Result<TxHash>;
    async fn lock(&self, address: &Address, amount: Decimal) -> Result<TxHash>;
    async fn unlock(&self, address: &Address, amount: Decimal) -> Result<TxHash>;
}

#[async_trait]
impl RuvTokenInterface for RuvTokenManager {
    async fn get_balance(&self, address: &Address) -> Result<Decimal> {
        self.get_balance(address).await
    }
    
    async fn transfer(&self, from: &Address, to: &Address, amount: Decimal) -> Result<TxHash> {
        self.transfer(from, to, amount).await
    }
    
    async fn mint(&self, to: &Address, amount: Decimal) -> Result<TxHash> {
        self.mint(to, amount).await
    }
    
    async fn burn(&self, from: &Address, amount: Decimal) -> Result<TxHash> {
        self.burn(from, amount).await
    }
    
    async fn lock(&self, address: &Address, amount: Decimal) -> Result<TxHash> {
        self.lock(address, amount).await
    }
    
    async fn unlock(&self, address: &Address, amount: Decimal) -> Result<TxHash> {
        self.unlock(address, amount).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use daa_chain::{QuDAGAdapter, QuDAGConfig};
    
    #[tokio::test]
    async fn test_ruv_token_creation() {
        let token = RuvToken::new();
        assert_eq!(token.symbol, "rUv");
        assert_eq!(token.decimals, 18);
        assert_eq!(token.total_supply, Decimal::new(1_000_000_000, 18));
    }
    
    #[tokio::test]
    async fn test_wei_conversions() {
        let token = RuvToken::new();
        let amount = Decimal::new(1, 0); // 1 rUv
        let wei = token.to_wei(amount);
        assert_eq!(wei, 1_000_000_000_000_000_000); // 1e18
        
        let back_to_decimal = token.from_wei(wei);
        assert_eq!(back_to_decimal, amount);
    }
}