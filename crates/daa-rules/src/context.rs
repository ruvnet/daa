//! State context for rule evaluation

use crate::error::{Result, RuleError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Context for rule evaluation containing system state and transaction details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateContext {
    /// Current account balances
    pub balances: HashMap<String, u128>,
    
    /// Current transaction being evaluated
    pub current_transaction: Option<TransactionContext>,
    
    /// System-wide state variables
    pub system_state: SystemState,
    
    /// QuDAG-specific context
    pub qudag_context: QuDAGContext,
    
    /// Custom context data
    pub custom_data: HashMap<String, serde_json::Value>,
}

/// Transaction context for rule evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionContext {
    pub from: String,
    pub to: String,
    pub amount: u128,
    pub transaction_type: TransactionType,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// System state information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemState {
    pub current_time: chrono::DateTime<chrono::Utc>,
    pub block_height: u64,
    pub network_id: String,
    pub operational_mode: OperationalMode,
}

/// QuDAG-specific context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuDAGContext {
    /// Current rUv (recoverable Utility value) balances
    pub ruv_balances: HashMap<String, u128>,
    
    /// DAG node information
    pub dag_state: DAGState,
    
    /// Consensus state
    pub consensus_state: ConsensusState,
}

/// DAG state information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DAGState {
    pub current_height: u64,
    pub total_nodes: u64,
    pub confirmed_nodes: u64,
}

/// Consensus state information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusState {
    pub active_validators: u32,
    pub consensus_round: u64,
    pub threshold: u32,
}

/// Transaction types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TransactionType {
    Transfer,
    Mint,
    Burn,
    Stake,
    Unstake,
    Delegate,
    Undelegate,
    Governance,
    Custom(String),
}

/// System operational modes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OperationalMode {
    Normal,
    Maintenance,
    Emergency,
    Paused,
}

impl StateContext {
    /// Create a new empty state context
    pub fn new() -> Self {
        Self {
            balances: HashMap::new(),
            current_transaction: None,
            system_state: SystemState::default(),
            qudag_context: QuDAGContext::default(),
            custom_data: HashMap::new(),
        }
    }
    
    /// Get balance for an account
    pub fn get_balance(&self, account: &str) -> u128 {
        self.balances.get(account).copied().unwrap_or(0)
    }
    
    /// Get rUv balance for an account
    pub fn get_ruv_balance(&self, account: &str) -> u128 {
        self.qudag_context.ruv_balances.get(account).copied().unwrap_or(0)
    }
    
    /// Set balance for an account
    pub fn set_balance(&mut self, account: String, balance: u128) {
        self.balances.insert(account, balance);
    }
    
    /// Set rUv balance for an account
    pub fn set_ruv_balance(&mut self, account: String, balance: u128) {
        self.qudag_context.ruv_balances.insert(account, balance);
    }
    
    /// Get custom data value
    pub fn get_custom<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        match self.custom_data.get(key) {
            Some(value) => Ok(Some(serde_json::from_value(value.clone())?)),
            None => Ok(None),
        }
    }
    
    /// Set custom data value
    pub fn set_custom<T>(&mut self, key: String, value: T) -> Result<()>
    where
        T: Serialize,
    {
        let json_value = serde_json::to_value(value)?;
        self.custom_data.insert(key, json_value);
        Ok(())
    }
    
    /// Validate context has required fields
    pub fn validate_required_fields(&self, fields: &[&str]) -> Result<()> {
        for field in fields {
            match *field {
                "current_transaction" => {
                    if self.current_transaction.is_none() {
                        return Err(RuleError::MissingContext(field.to_string()));
                    }
                }
                "system_state" => {
                    // Always present, but could validate specific fields
                }
                "qudag_context" => {
                    // Always present, but could validate specific fields
                }
                key => {
                    if !self.custom_data.contains_key(key) {
                        return Err(RuleError::MissingContext(key.to_string()));
                    }
                }
            }
        }
        Ok(())
    }
}

impl Default for StateContext {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for SystemState {
    fn default() -> Self {
        Self {
            current_time: chrono::Utc::now(),
            block_height: 0,
            network_id: "daa-testnet".to_string(),
            operational_mode: OperationalMode::Normal,
        }
    }
}

impl Default for QuDAGContext {
    fn default() -> Self {
        Self {
            ruv_balances: HashMap::new(),
            dag_state: DAGState::default(),
            consensus_state: ConsensusState::default(),
        }
    }
}

impl Default for DAGState {
    fn default() -> Self {
        Self {
            current_height: 0,
            total_nodes: 0,
            confirmed_nodes: 0,
        }
    }
}

impl Default for ConsensusState {
    fn default() -> Self {
        Self {
            active_validators: 0,
            consensus_round: 0,
            threshold: 0,
        }
    }
}