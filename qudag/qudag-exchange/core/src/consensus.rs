//! Consensus integration for QuDAG Exchange
//!
//! Provides adapter layer for QR-Avalanche DAG consensus

#[cfg(not(feature = "std"))]
use alloc::{collections::BTreeMap, string::String, vec::Vec};

#[cfg(feature = "std")]
use std::collections::BTreeMap;

use crate::{
    transaction::{Transaction, TransactionId, TransactionStatus},
    types::{Hash, Timestamp},
    Error, Result,
};
use serde::{Deserialize, Serialize};

/// Consensus adapter for integrating with QR-Avalanche DAG
#[derive(Debug, Clone)]
pub struct ConsensusAdapter {
    /// Configuration for consensus
    config: ConsensusConfig,

    /// Pending transactions awaiting consensus
    #[cfg(not(feature = "std"))]
    pending_transactions: BTreeMap<TransactionId, PendingTransaction>,

    #[cfg(feature = "std")]
    pending_transactions: dashmap::DashMap<TransactionId, PendingTransaction>,

    /// Node ID for this instance
    node_id: String,

    /// Current consensus round
    current_round: u64,
}

/// Configuration for consensus adapter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusConfig {
    /// Confidence threshold for transaction finalization
    pub confidence_threshold: f64,

    /// Maximum rounds before transaction expires
    pub max_rounds: u64,

    /// Sample size for consensus queries
    pub sample_size: usize,

    /// Timeout for consensus queries (milliseconds)
    pub query_timeout_ms: u64,

    /// Enable fast finality for high-confidence transactions
    pub fast_finality: bool,
}

impl Default for ConsensusConfig {
    fn default() -> Self {
        Self {
            confidence_threshold: 0.95,
            max_rounds: 100,
            sample_size: 20,
            query_timeout_ms: 5000,
            fast_finality: true,
        }
    }
}

/// Transaction pending consensus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingTransaction {
    /// The transaction
    pub transaction: Transaction,

    /// Current status
    pub status: TransactionStatus,

    /// Submission timestamp
    pub submitted_at: Timestamp,

    /// Current confidence level (0.0 to 1.0)
    pub confidence: f64,

    /// Number of consensus rounds
    pub rounds: u64,

    /// Votes received (for simulation)
    pub votes_for: u32,
    pub votes_against: u32,
}

/// DAG vertex containing one or more transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionVertex {
    /// Vertex ID (hash of contents)
    pub id: VertexId,

    /// Transactions in this vertex
    pub transactions: Vec<Transaction>,

    /// Parent vertex IDs
    pub parents: Vec<VertexId>,

    /// Creation timestamp
    pub timestamp: Timestamp,

    /// Node that created this vertex
    pub creator: String,

    /// Quantum-resistant signature
    pub signature: Vec<u8>,
}

/// Vertex identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VertexId(Hash);

impl VertexId {
    /// Create from hash
    pub fn from_hash(hash: Hash) -> Self {
        Self(hash)
    }

    /// Get the underlying hash
    pub fn hash(&self) -> &Hash {
        &self.0
    }
}

/// Consensus event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsensusEvent {
    /// Transaction submitted for consensus
    TransactionSubmitted(TransactionId),

    /// Transaction confidence updated
    ConfidenceUpdated {
        tx_id: TransactionId,
        confidence: f64,
    },

    /// Transaction finalized
    TransactionFinalized {
        tx_id: TransactionId,
        accepted: bool,
    },

    /// New vertex added to DAG
    VertexAdded(VertexId),

    /// Consensus round completed
    RoundCompleted(u64),
}

impl ConsensusAdapter {
    /// Create a new consensus adapter
    pub fn new(node_id: impl Into<String>) -> Self {
        Self::with_config(node_id, ConsensusConfig::default())
    }

    /// Create with custom configuration
    pub fn with_config(node_id: impl Into<String>, config: ConsensusConfig) -> Self {
        Self {
            config,
            #[cfg(not(feature = "std"))]
            pending_transactions: BTreeMap::new(),
            #[cfg(feature = "std")]
            pending_transactions: dashmap::DashMap::new(),
            node_id: node_id.into(),
            current_round: 0,
        }
    }

    /// Submit a transaction for consensus
    pub fn submit_transaction(&mut self, transaction: Transaction) -> Result<TransactionId> {
        // Verify transaction signature if present
        #[cfg(feature = "std")]
        if transaction.signature.is_some() && !transaction.verify_signature()? {
            return Err(Error::SignatureVerificationFailed);
        }

        let tx_id = transaction.id()?;
        let pending = PendingTransaction {
            transaction,
            status: TransactionStatus::Pending,
            submitted_at: Timestamp::now(),
            confidence: 0.0,
            rounds: 0,
            votes_for: 0,
            votes_against: 0,
        };

        #[cfg(not(feature = "std"))]
        self.pending_transactions.insert(tx_id, pending);

        #[cfg(feature = "std")]
        self.pending_transactions.insert(tx_id, pending);

        Ok(tx_id)
    }

    /// Get transaction status
    pub fn get_transaction_status(&self, tx_id: &TransactionId) -> Option<TransactionStatus> {
        #[cfg(not(feature = "std"))]
        return self.pending_transactions.get(tx_id).map(|tx| tx.status);

        #[cfg(feature = "std")]
        return self
            .pending_transactions
            .get(tx_id)
            .map(|entry| entry.status);
    }

    /// Get transaction confidence
    pub fn get_confidence(&self, tx_id: &TransactionId) -> Option<f64> {
        #[cfg(not(feature = "std"))]
        return self.pending_transactions.get(tx_id).map(|tx| tx.confidence);

        #[cfg(feature = "std")]
        return self
            .pending_transactions
            .get(tx_id)
            .map(|entry| entry.confidence);
    }

    /// Simulate a consensus round (for testing without actual DAG)
    pub fn simulate_consensus_round(&mut self) -> Vec<ConsensusEvent> {
        let mut events = Vec::new();
        self.current_round += 1;

        // Process each pending transaction
        let tx_ids: Vec<_> = {
            #[cfg(not(feature = "std"))]
            let ids = self.pending_transactions.keys().cloned().collect();

            #[cfg(feature = "std")]
            let ids = self
                .pending_transactions
                .iter()
                .map(|entry| *entry.key())
                .collect();

            ids
        };

        for tx_id in tx_ids {
            self.process_transaction_round(&tx_id, &mut events);
        }

        events.push(ConsensusEvent::RoundCompleted(self.current_round));
        events
    }

    /// Process a single transaction in the consensus round
    fn process_transaction_round(
        &mut self,
        tx_id: &TransactionId,
        events: &mut Vec<ConsensusEvent>,
    ) {
        #[cfg(not(feature = "std"))]
        let mut pending_tx = match self.pending_transactions.get_mut(tx_id) {
            Some(tx) => tx.clone(),
            None => return,
        };

        #[cfg(feature = "std")]
        let mut pending_tx = match self.pending_transactions.get(tx_id) {
            Some(entry) => entry.clone(),
            None => return,
        };

        // Skip if already finalized
        if pending_tx.status == TransactionStatus::Confirmed
            || pending_tx.status == TransactionStatus::Rejected
        {
            return;
        }

        // Update status to processing
        pending_tx.status = TransactionStatus::Processing;
        pending_tx.rounds += 1;

        // Simulate voting (in real implementation, this would query other nodes)
        let vote_for = self.simulate_vote(&pending_tx.transaction);
        if vote_for {
            pending_tx.votes_for += 1;
        } else {
            pending_tx.votes_against += 1;
        }

        // Update confidence
        let total_votes = pending_tx.votes_for + pending_tx.votes_against;
        if total_votes > 0 {
            pending_tx.confidence = pending_tx.votes_for as f64 / total_votes as f64;

            events.push(ConsensusEvent::ConfidenceUpdated {
                tx_id: *tx_id,
                confidence: pending_tx.confidence,
            });
        }

        // Check for finalization
        if pending_tx.confidence >= self.config.confidence_threshold
            && total_votes >= self.config.sample_size as u32
        {
            // Transaction accepted
            pending_tx.status = TransactionStatus::Confirmed;
            events.push(ConsensusEvent::TransactionFinalized {
                tx_id: *tx_id,
                accepted: true,
            });
        } else if (1.0 - pending_tx.confidence) >= self.config.confidence_threshold
            && total_votes >= self.config.sample_size as u32
        {
            // Transaction rejected
            pending_tx.status = TransactionStatus::Rejected;
            events.push(ConsensusEvent::TransactionFinalized {
                tx_id: *tx_id,
                accepted: false,
            });
        } else if pending_tx.rounds >= self.config.max_rounds {
            // Transaction expired
            pending_tx.status = TransactionStatus::Expired;
            events.push(ConsensusEvent::TransactionFinalized {
                tx_id: *tx_id,
                accepted: false,
            });
        }

        // Update the pending transaction
        #[cfg(not(feature = "std"))]
        self.pending_transactions.insert(*tx_id, pending_tx);

        #[cfg(feature = "std")]
        self.pending_transactions.insert(*tx_id, pending_tx);
    }

    /// Simulate a vote (for testing)
    fn simulate_vote(&self, transaction: &Transaction) -> bool {
        // Simple validation rules for simulation
        // In real implementation, this would validate against ledger state

        // Check if fee is sufficient
        if transaction.fee < crate::types::rUv::new(1) {
            return false;
        }

        // Check if not expired
        if let Some(expires_at) = transaction.expires_at {
            if transaction.is_expired(Timestamp::now()) {
                return false;
            }
        }

        // 90% chance of accepting valid transactions (for simulation)
        true
    }

    /// Create a new vertex containing transactions
    pub fn create_vertex(
        &self,
        transactions: Vec<Transaction>,
        parent_ids: Vec<VertexId>,
    ) -> Result<TransactionVertex> {
        let vertex = TransactionVertex {
            id: self.compute_vertex_id(&transactions, &parent_ids)?,
            transactions,
            parents: parent_ids,
            timestamp: Timestamp::now(),
            creator: self.node_id.clone(),
            signature: Vec::new(), // Would be signed in real implementation
        };

        Ok(vertex)
    }

    /// Compute vertex ID from contents
    fn compute_vertex_id(
        &self,
        transactions: &[Transaction],
        parents: &[VertexId],
    ) -> Result<VertexId> {
        use blake3::Hasher;

        let mut hasher = Hasher::new();

        // Hash transactions
        for tx in transactions {
            let tx_bytes = tx.to_bytes()?;
            hasher.update(&tx_bytes);
        }

        // Hash parent IDs
        for parent in parents {
            hasher.update(parent.hash().as_bytes());
        }

        let hash_bytes = hasher.finalize();
        Ok(VertexId::from_hash(Hash::from_bytes(
            *hash_bytes.as_bytes(),
        )))
    }

    /// Get all pending transactions
    pub fn pending_transactions(&self) -> Vec<(TransactionId, PendingTransaction)> {
        #[cfg(not(feature = "std"))]
        return self
            .pending_transactions
            .iter()
            .map(|(k, v)| (*k, v.clone()))
            .collect();

        #[cfg(feature = "std")]
        return self
            .pending_transactions
            .iter()
            .map(|entry| (*entry.key(), entry.value().clone()))
            .collect();
    }

    /// Get finalized transactions
    pub fn finalized_transactions(&self) -> Vec<(TransactionId, Transaction)> {
        self.pending_transactions()
            .into_iter()
            .filter(|(_, pending)| pending.status == TransactionStatus::Confirmed)
            .map(|(id, pending)| (id, pending.transaction))
            .collect()
    }

    /// Clear finalized transactions (for memory management)
    pub fn clear_finalized(&mut self) {
        #[cfg(not(feature = "std"))]
        {
            self.pending_transactions.retain(|_, pending| {
                pending.status != TransactionStatus::Confirmed
                    && pending.status != TransactionStatus::Rejected
                    && pending.status != TransactionStatus::Expired
            });
        }

        #[cfg(feature = "std")]
        {
            self.pending_transactions.retain(|_, pending| {
                pending.status != TransactionStatus::Confirmed
                    && pending.status != TransactionStatus::Rejected
                    && pending.status != TransactionStatus::Expired
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        account::AccountId,
        types::{rUv, Nonce},
    };

    #[test]
    fn test_consensus_adapter_creation() {
        let adapter = ConsensusAdapter::new("node1");
        assert_eq!(adapter.node_id, "node1");
        assert_eq!(adapter.current_round, 0);
    }

    #[test]
    fn test_submit_transaction() {
        let mut adapter = ConsensusAdapter::new("node1");

        let tx = Transaction::transfer(
            AccountId::new("alice"),
            AccountId::new("bob"),
            rUv::new(100),
            Nonce::new(1),
            rUv::new(5),
        );

        let tx_id = adapter.submit_transaction(tx).unwrap();
        assert_eq!(
            adapter.get_transaction_status(&tx_id),
            Some(TransactionStatus::Pending)
        );
    }

    #[test]
    fn test_consensus_simulation() {
        let mut adapter = ConsensusAdapter::new("node1");

        // Submit a transaction
        let tx = Transaction::transfer(
            AccountId::new("alice"),
            AccountId::new("bob"),
            rUv::new(100),
            Nonce::new(1),
            rUv::new(5),
        );
        let tx_id = adapter.submit_transaction(tx).unwrap();

        // Run consensus rounds
        for _ in 0..25 {
            let events = adapter.simulate_consensus_round();

            // Check if transaction is finalized
            if let Some(status) = adapter.get_transaction_status(&tx_id) {
                if status == TransactionStatus::Confirmed || status == TransactionStatus::Rejected {
                    break;
                }
            }
        }

        // Should be finalized by now
        let status = adapter.get_transaction_status(&tx_id).unwrap();
        assert!(status == TransactionStatus::Confirmed || status == TransactionStatus::Rejected);
    }

    #[test]
    fn test_vertex_creation() {
        let adapter = ConsensusAdapter::new("node1");

        let tx1 = Transaction::transfer(
            AccountId::new("alice"),
            AccountId::new("bob"),
            rUv::new(50),
            Nonce::new(1),
            rUv::new(1),
        );

        let tx2 = Transaction::transfer(
            AccountId::new("bob"),
            AccountId::new("charlie"),
            rUv::new(25),
            Nonce::new(1),
            rUv::new(1),
        );

        let vertex = adapter.create_vertex(vec![tx1, tx2], vec![]).unwrap();
        assert_eq!(vertex.transactions.len(), 2);
        assert_eq!(vertex.creator, "node1");
    }

    #[test]
    fn test_confidence_updates() {
        let mut config = ConsensusConfig::default();
        config.sample_size = 5; // Lower for testing
        config.confidence_threshold = 0.8;

        let mut adapter = ConsensusAdapter::with_config("node1", config);

        let tx = Transaction::transfer(
            AccountId::new("alice"),
            AccountId::new("bob"),
            rUv::new(100),
            Nonce::new(1),
            rUv::new(5),
        );
        let tx_id = adapter.submit_transaction(tx).unwrap();

        // Run rounds and check confidence
        for _ in 0..10 {
            adapter.simulate_consensus_round();

            if let Some(confidence) = adapter.get_confidence(&tx_id) {
                assert!(confidence >= 0.0 && confidence <= 1.0);
            }
        }
    }
}
