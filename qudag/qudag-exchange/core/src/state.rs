//! State management for QuDAG Exchange
//!
//! Handles ledger state persistence, snapshots, and recovery

#[cfg(not(feature = "std"))]
use alloc::{collections::BTreeMap, string::String, vec::Vec};

#[cfg(feature = "std")]
use std::collections::BTreeMap;

use crate::{
    account::AccountId,
    consensus::ConsensusAdapter,
    ledger::Ledger,
    metering::ResourceMeter,
    transaction::{Transaction, TransactionId},
    types::{rUv, Hash, Timestamp},
    Error, Result,
};
use serde::{Deserialize, Serialize};

/// Main ledger state containing all exchange data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerState {
    /// The ledger with account balances
    pub ledger: Ledger,

    /// Resource meter for tracking costs
    pub meter: ResourceMeter,

    /// State metadata
    pub metadata: StateMetadata,

    /// Transaction history (limited to recent)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub recent_transactions: Vec<TransactionRecord>,

    /// Checkpoint history
    pub checkpoints: Vec<StateCheckpoint>,
}

/// State metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateMetadata {
    /// State version for migrations
    pub version: u32,

    /// Chain ID
    pub chain_id: u64,

    /// Genesis timestamp
    pub genesis_time: Timestamp,

    /// Last update timestamp
    pub last_updated: Timestamp,

    /// Current block/round height
    pub height: u64,

    /// State root hash
    pub state_root: Option<Hash>,
}

/// Record of a processed transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionRecord {
    /// Transaction ID
    pub id: TransactionId,

    /// The transaction
    pub transaction: Transaction,

    /// Execution timestamp
    pub executed_at: Timestamp,

    /// Execution result
    pub result: TransactionResult,

    /// Resource cost
    pub cost: rUv,
}

/// Result of transaction execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionResult {
    /// Transaction succeeded
    Success,
    /// Transaction failed with error
    Failed(String),
    /// Transaction reverted (state unchanged)
    Reverted(String),
}

/// State checkpoint for recovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateCheckpoint {
    /// Checkpoint height
    pub height: u64,

    /// Checkpoint timestamp
    pub timestamp: Timestamp,

    /// State root at this checkpoint
    pub state_root: Hash,

    /// Serialized state snapshot
    #[serde(with = "base64")]
    pub snapshot: Vec<u8>,
}

/// State manager for coordinating state operations
pub struct StateManager {
    /// Current state
    state: LedgerState,

    /// Consensus adapter
    consensus: ConsensusAdapter,

    /// Configuration
    config: StateConfig,
}

/// State management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateConfig {
    /// Maximum transactions to keep in history
    pub max_transaction_history: usize,

    /// Maximum checkpoints to retain
    pub max_checkpoints: usize,

    /// Checkpoint interval (in blocks/rounds)
    pub checkpoint_interval: u64,

    /// Enable state pruning
    pub enable_pruning: bool,

    /// State file path (for persistence)
    #[cfg(feature = "std")]
    pub state_file: Option<String>,
}

impl Default for StateConfig {
    fn default() -> Self {
        Self {
            max_transaction_history: 1000,
            max_checkpoints: 10,
            checkpoint_interval: 100,
            enable_pruning: true,
            #[cfg(feature = "std")]
            state_file: Some("ledger_state.dat".to_string()),
        }
    }
}

// Base64 encoding helper for serde
mod base64 {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(bytes: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use base64::Engine;
        let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
        encoded.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        use base64::Engine;
        let encoded = String::deserialize(deserializer)?;
        base64::engine::general_purpose::STANDARD
            .decode(&encoded)
            .map_err(serde::de::Error::custom)
    }
}

impl LedgerState {
    /// Create a new ledger state
    pub fn new(chain_id: u64) -> Self {
        let now = Timestamp::now();
        Self {
            ledger: Ledger::new(),
            meter: ResourceMeter::new(),
            metadata: StateMetadata {
                version: 1,
                chain_id,
                genesis_time: now,
                last_updated: now,
                height: 0,
                state_root: None,
            },
            recent_transactions: Vec::new(),
            checkpoints: Vec::new(),
        }
    }

    /// Create genesis state with initial accounts
    pub fn genesis(chain_id: u64, initial_accounts: Vec<(AccountId, rUv)>) -> Result<Self> {
        let mut state = Self::new(chain_id);

        // Create initial accounts
        for (account_id, balance) in initial_accounts {
            state.ledger.create_account(account_id.clone())?;
            if balance > rUv::ZERO {
                state.ledger.mint(&account_id, balance)?;
            }
        }

        // Compute initial state root
        state.update_state_root()?;

        Ok(state)
    }

    /// Apply a transaction to the state
    pub fn apply_transaction(&mut self, transaction: &Transaction) -> Result<TransactionRecord> {
        let tx_id = transaction.id()?;
        let start_time = Timestamp::now();

        // Calculate resource cost
        let cost = self.calculate_transaction_cost(transaction)?;

        // Execute transaction
        let result = self.execute_transaction(transaction, cost);

        // Create transaction record
        let record = TransactionRecord {
            id: tx_id,
            transaction: transaction.clone(),
            executed_at: start_time,
            result: match &result {
                Ok(()) => TransactionResult::Success,
                Err(e) => TransactionResult::Failed(e.to_string()),
            },
            cost,
        };

        // Add to history
        self.add_transaction_record(record.clone());

        // Update metadata
        self.metadata.last_updated = Timestamp::now();
        self.metadata.height += 1;

        result?;
        Ok(record)
    }

    /// Execute a transaction
    fn execute_transaction(&mut self, transaction: &Transaction, cost: rUv) -> Result<()> {
        use crate::transaction::TransactionType;

        // Deduct fee from sender (if applicable)
        if let Some(sender) = transaction.sender() {
            self.ledger.burn(sender, cost)?;
        }

        // Execute based on transaction type
        match &transaction.tx_type {
            TransactionType::Transfer { from, to, amount } => {
                self.ledger.transfer(from, to, *amount)?;
            }
            TransactionType::Mint { to, amount } => {
                self.ledger.mint(to, *amount)?;
            }
            TransactionType::Burn { from, amount } => {
                self.ledger.burn(from, *amount)?;
            }
            TransactionType::CreateAccount {
                account,
                initial_balance,
            } => {
                self.ledger.create_account(account.clone())?;
                if *initial_balance > rUv::ZERO {
                    self.ledger.mint(account, *initial_balance)?;
                }
            }
            TransactionType::UpdateAccount {
                account,
                public_key,
            } => {
                let mut acc = self.ledger.get_account(account)?;
                if let Some(pk) = public_key {
                    acc.set_public_key(pk.clone());
                }
                // In real implementation, would update the account in ledger
            }
        }

        Ok(())
    }

    /// Calculate transaction cost
    fn calculate_transaction_cost(&self, transaction: &Transaction) -> Result<rUv> {
        use crate::metering::OperationType;
        use crate::transaction::TransactionType;

        let op_type = match &transaction.tx_type {
            TransactionType::Transfer { .. } => OperationType::Transfer,
            TransactionType::Mint { .. } => OperationType::Mint,
            TransactionType::Burn { .. } => OperationType::Burn,
            TransactionType::CreateAccount { .. } => OperationType::CreateAccount,
            TransactionType::UpdateAccount { .. } => OperationType::UpdateAccount,
        };

        // Calculate units based on transaction size
        let units = ResourceMeter::estimate_transfer_units(transaction.metadata.memo.as_deref());

        let metered_cost = self.meter.calculate_cost(op_type, units)?;

        // Use the higher of metered cost or transaction fee
        Ok(if transaction.fee > metered_cost {
            transaction.fee
        } else {
            metered_cost
        })
    }

    /// Add transaction record to history
    fn add_transaction_record(&mut self, record: TransactionRecord) {
        self.recent_transactions.push(record);

        // Prune old transactions if needed
        let max_history = 1000; // Could be configurable
        if self.recent_transactions.len() > max_history {
            self.recent_transactions
                .drain(0..self.recent_transactions.len() - max_history);
        }
    }

    /// Update state root hash
    pub fn update_state_root(&mut self) -> Result<()> {
        let state_bytes = self.to_bytes()?;
        let hash = blake3::hash(&state_bytes);
        self.metadata.state_root = Some(Hash::from_bytes(*hash.as_bytes()));
        Ok(())
    }

    /// Serialize state to bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        bincode::serialize(self).map_err(|e| Error::SerializationError(e.to_string()))
    }

    /// Deserialize state from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        bincode::deserialize(bytes).map_err(|e| Error::SerializationError(e.to_string()))
    }

    /// Create a checkpoint
    pub fn create_checkpoint(&mut self) -> Result<StateCheckpoint> {
        self.update_state_root()?;

        let checkpoint = StateCheckpoint {
            height: self.metadata.height,
            timestamp: Timestamp::now(),
            state_root: self
                .metadata
                .state_root
                .ok_or_else(|| Error::Other("No state root".into()))?,
            snapshot: self.to_bytes()?,
        };

        self.checkpoints.push(checkpoint.clone());

        // Prune old checkpoints
        let max_checkpoints = 10; // Could be configurable
        if self.checkpoints.len() > max_checkpoints {
            self.checkpoints
                .drain(0..self.checkpoints.len() - max_checkpoints);
        }

        Ok(checkpoint)
    }

    /// Restore from checkpoint
    pub fn restore_from_checkpoint(checkpoint: &StateCheckpoint) -> Result<Self> {
        Self::from_bytes(&checkpoint.snapshot)
    }

    /// Verify state integrity
    pub fn verify_integrity(&self) -> Result<()> {
        // Verify ledger invariants
        self.ledger.check_invariants()?;

        // Verify state root if present
        if let Some(expected_root) = self.metadata.state_root {
            let mut state_copy = self.clone();
            state_copy.metadata.state_root = None; // Exclude root from hash
            let computed_bytes = state_copy.to_bytes()?;
            let computed_hash = blake3::hash(&computed_bytes);
            let computed_root = Hash::from_bytes(*computed_hash.as_bytes());

            if computed_root != expected_root {
                return Err(Error::StateCorruption("State root mismatch".into()));
            }
        }

        Ok(())
    }
}

impl StateManager {
    /// Create a new state manager
    pub fn new(state: LedgerState, consensus: ConsensusAdapter) -> Self {
        Self::with_config(state, consensus, StateConfig::default())
    }

    /// Create with custom configuration
    pub fn with_config(
        state: LedgerState,
        consensus: ConsensusAdapter,
        config: StateConfig,
    ) -> Self {
        Self {
            state,
            consensus,
            config,
        }
    }

    /// Process finalized transactions from consensus
    pub fn process_consensus_transactions(&mut self) -> Result<Vec<TransactionRecord>> {
        let finalized = self.consensus.finalized_transactions();
        let mut records = Vec::new();

        for (_, transaction) in finalized {
            match self.state.apply_transaction(&transaction) {
                Ok(record) => records.push(record),
                Err(e) => {
                    // Log error but continue processing other transactions
                    eprintln!("Failed to apply transaction: {:?}", e);
                }
            }
        }

        // Clear processed transactions from consensus
        self.consensus.clear_finalized();

        // Check if we should create a checkpoint
        if self.state.metadata.height % self.config.checkpoint_interval == 0 {
            self.state.create_checkpoint()?;
        }

        Ok(records)
    }

    /// Save state to persistent storage
    #[cfg(feature = "std")]
    pub fn save_state(&self) -> Result<()> {
        if let Some(path) = &self.config.state_file {
            use std::fs;
            let bytes = self.state.to_bytes()?;
            fs::write(path, bytes)
                .map_err(|e| Error::Other(format!("Failed to save state: {}", e)))?;
        }
        Ok(())
    }

    /// Load state from persistent storage
    #[cfg(feature = "std")]
    pub fn load_state(path: &str) -> Result<LedgerState> {
        use std::fs;
        let bytes =
            fs::read(path).map_err(|e| Error::Other(format!("Failed to load state: {}", e)))?;
        LedgerState::from_bytes(&bytes)
    }

    /// Get current state
    pub fn state(&self) -> &LedgerState {
        &self.state
    }

    /// Get mutable state
    pub fn state_mut(&mut self) -> &mut LedgerState {
        &mut self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        transaction::Transaction,
        types::{rUv, Nonce},
    };

    #[test]
    fn test_ledger_state_creation() {
        let state = LedgerState::new(1);
        assert_eq!(state.metadata.chain_id, 1);
        assert_eq!(state.metadata.height, 0);
        assert!(state.recent_transactions.is_empty());
    }

    #[test]
    fn test_genesis_state() {
        let initial_accounts = vec![
            (AccountId::new("alice"), rUv::new(1000)),
            (AccountId::new("bob"), rUv::new(500)),
        ];

        let state = LedgerState::genesis(1, initial_accounts).unwrap();
        assert_eq!(state.ledger.account_count(), 2);
        assert_eq!(state.ledger.total_supply(), rUv::new(1500));
        assert!(state.metadata.state_root.is_some());
    }

    #[test]
    fn test_apply_transaction() {
        let mut state = LedgerState::new(1);

        // Create accounts
        state
            .ledger
            .create_account(AccountId::new("alice"))
            .unwrap();
        state.ledger.create_account(AccountId::new("bob")).unwrap();
        state
            .ledger
            .mint(&AccountId::new("alice"), rUv::new(1000))
            .unwrap();

        // Create and apply transfer
        let tx = Transaction::transfer(
            AccountId::new("alice"),
            AccountId::new("bob"),
            rUv::new(100),
            Nonce::new(1),
            rUv::new(5),
        );

        let record = state.apply_transaction(&tx).unwrap();

        // Check results
        assert!(matches!(record.result, TransactionResult::Success));
        assert_eq!(
            state.ledger.get_balance(&AccountId::new("alice")).unwrap(),
            rUv::new(895)
        ); // 1000 - 100 - 5
        assert_eq!(
            state.ledger.get_balance(&AccountId::new("bob")).unwrap(),
            rUv::new(100)
        );
        assert_eq!(state.metadata.height, 1);
    }

    #[test]
    fn test_checkpoint_creation() {
        let mut state = LedgerState::new(1);
        state.metadata.height = 100;

        let checkpoint = state.create_checkpoint().unwrap();
        assert_eq!(checkpoint.height, 100);
        assert!(!checkpoint.snapshot.is_empty());

        // Restore from checkpoint
        let restored = LedgerState::restore_from_checkpoint(&checkpoint).unwrap();
        assert_eq!(restored.metadata.height, 100);
    }

    #[test]
    fn test_state_serialization() {
        let state = LedgerState::new(1);
        let bytes = state.to_bytes().unwrap();
        let restored = LedgerState::from_bytes(&bytes).unwrap();

        assert_eq!(state.metadata.chain_id, restored.metadata.chain_id);
        assert_eq!(state.metadata.version, restored.metadata.version);
    }

    #[test]
    fn test_state_integrity() {
        let mut state = LedgerState::new(1);
        state.update_state_root().unwrap();

        // Should pass integrity check
        state.verify_integrity().unwrap();

        // Corrupt the height
        state.metadata.height = 999;

        // Should fail integrity check
        assert!(state.verify_integrity().is_err());
    }
}
