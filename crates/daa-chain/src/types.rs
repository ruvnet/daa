//! Common types for blockchain interactions

use serde::{Deserialize, Serialize};
use std::fmt;

/// A blockchain address (20 bytes for Ethereum, variable for others)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Address(Vec<u8>);

impl Address {
    pub fn new(bytes: Vec<u8>) -> Self {
        Address(bytes)
    }
    
    pub fn from_hex(hex_str: &str) -> Result<Self, hex::FromHexError> {
        let hex_str = hex_str.trim_start_matches("0x");
        let bytes = hex::decode(hex_str)?;
        Ok(Address(bytes))
    }
    
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
    
    pub fn to_hex(&self) -> String {
        format!("0x{}", hex::encode(&self.0))
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

/// Transaction hash
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TxHash(Vec<u8>);

impl TxHash {
    pub fn new(bytes: Vec<u8>) -> Self {
        TxHash(bytes)
    }
    
    pub fn from_hex(hex_str: &str) -> Result<Self, hex::FromHexError> {
        let hex_str = hex_str.trim_start_matches("0x");
        let bytes = hex::decode(hex_str)?;
        Ok(TxHash(bytes))
    }
    
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
    
    pub fn to_hex(&self) -> String {
        format!("0x{}", hex::encode(&self.0))
    }
}

impl fmt::Display for TxHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

/// Balance representation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Balance {
    /// Amount in the smallest unit (wei for Ethereum)
    pub amount: u128,
    /// Symbol of the currency/token
    pub symbol: String,
    /// Number of decimals
    pub decimals: u8,
}

impl Balance {
    pub fn new(amount: u128, symbol: String, decimals: u8) -> Self {
        Balance { amount, symbol, decimals }
    }
    
    /// Create balance from wei (for Ethereum)
    pub fn from_wei(wei: u128) -> Self {
        Balance {
            amount: wei,
            symbol: "ETH".to_string(),
            decimals: 18,
        }
    }
    
    /// Convert to human-readable format
    pub fn to_decimal(&self) -> f64 {
        self.amount as f64 / 10f64.powi(self.decimals as i32)
    }
}

impl fmt::Display for Balance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.to_decimal(), self.symbol)
    }
}

/// Block representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub number: u64,
    pub hash: TxHash,
    pub parent_hash: TxHash,
    pub timestamp: u64,
    pub transactions: Vec<TxHash>,
}

impl Block {
    pub fn new(
        number: u64,
        hash: TxHash,
        parent_hash: TxHash,
        timestamp: u64,
        transactions: Vec<TxHash>,
    ) -> Self {
        Block {
            number,
            hash,
            parent_hash,
            timestamp,
            transactions,
        }
    }
}

/// Transaction representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub from: Address,
    pub to: Option<Address>,
    pub value: u128,
    pub data: Vec<u8>,
    pub gas_limit: u64,
    pub gas_price: Option<u128>,
    pub max_fee_per_gas: Option<u128>,
    pub max_priority_fee_per_gas: Option<u128>,
    pub nonce: Option<u64>,
}

impl Transaction {
    pub fn new(from: Address, to: Option<Address>, value: u128) -> Self {
        Transaction {
            from,
            to,
            value,
            data: vec![],
            gas_limit: 21000, // Default for simple transfer
            gas_price: None,
            max_fee_per_gas: None,
            max_priority_fee_per_gas: None,
            nonce: None,
        }
    }
    
    pub fn with_data(mut self, data: Vec<u8>) -> Self {
        self.data = data;
        self
    }
    
    pub fn with_gas_limit(mut self, gas_limit: u64) -> Self {
        self.gas_limit = gas_limit;
        self
    }
    
    pub fn with_gas_price(mut self, gas_price: u128) -> Self {
        self.gas_price = Some(gas_price);
        self
    }
    
    pub fn with_eip1559_fees(mut self, max_fee: u128, max_priority_fee: u128) -> Self {
        self.max_fee_per_gas = Some(max_fee);
        self.max_priority_fee_per_gas = Some(max_priority_fee);
        self.gas_price = None; // Clear gas_price when using EIP-1559
        self
    }
    
    pub fn with_nonce(mut self, nonce: u64) -> Self {
        self.nonce = Some(nonce);
        self
    }
}

/// Chain configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainConfig {
    pub chain_id: u64,
    pub name: String,
    pub rpc_url: String,
    pub explorer_url: Option<String>,
    pub native_token_symbol: String,
    pub native_token_decimals: u8,
}

impl ChainConfig {
    pub fn ethereum_mainnet(rpc_url: String) -> Self {
        ChainConfig {
            chain_id: 1,
            name: "Ethereum Mainnet".to_string(),
            rpc_url,
            explorer_url: Some("https://etherscan.io".to_string()),
            native_token_symbol: "ETH".to_string(),
            native_token_decimals: 18,
        }
    }
    
    pub fn ethereum_goerli(rpc_url: String) -> Self {
        ChainConfig {
            chain_id: 5,
            name: "Ethereum Goerli".to_string(),
            rpc_url,
            explorer_url: Some("https://goerli.etherscan.io".to_string()),
            native_token_symbol: "ETH".to_string(),
            native_token_decimals: 18,
        }
    }
}