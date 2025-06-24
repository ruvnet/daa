//! QuDAG Exchange Core Library
//!
//! This crate provides the core functionality for the QuDAG Exchange system:
//! - rUv (Resource Utilization Voucher) token ledger
//! - Resource metering and cost calculations
//! - Transaction processing with quantum-resistant signatures
//! - Consensus integration with QR-Avalanche DAG
//! - Secure key management through QuDAG Vault
//!
//! The library is designed to be no_std compatible for WASM deployment.

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(unsafe_code)]
#![warn(missing_docs)]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::{collections::BTreeMap, string::String, vec::Vec};

#[cfg(feature = "std")]
use std::{collections::BTreeMap, string::String, vec::Vec};

// Public modules
pub mod account;
pub mod config;
pub mod consensus;
pub mod error;
pub mod fee_model;
pub mod immutable;
pub mod ledger;
pub mod metering;
pub mod payout;
pub mod state;
pub mod transaction;
pub mod types;

// Re-exports
pub use account::{Account, AccountId, Balance};
pub use config::{
    BusinessPlanConfig, BusinessPlanSummary, ConfigSummary, ExchangeConfig, ExchangeConfigBuilder,
    GovernanceConfig, NetworkConfig, SecurityConfig,
};
pub use consensus::ConsensusAdapter;
pub use error::{Error, Result};
pub use fee_model::{AgentStatus, FeeCalculator, FeeModel, FeeModelParams};
pub use immutable::{
    ImmutableConfig, ImmutableDeployment, ImmutableSignature, ImmutableStatus, LockableConfig,
};
pub use ledger::Ledger;
pub use metering::{OperationCost, ResourceMeter};
pub use payout::{
    ContributorInfo, ContributorRole, ContributorType, FeeRouter, PayoutConfig, PayoutEntry,
    PayoutSplit, PayoutSplitTemplates, PayoutTransaction,
};
pub use state::LedgerState;
pub use transaction::{Transaction, TransactionId, TransactionStatus};
pub use types::rUv;

/// Core version string
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Get the version of the QuDAG Exchange Core library
pub fn version() -> &'static str {
    VERSION
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!version().is_empty());
        assert!(version().contains('.'));
    }
}
