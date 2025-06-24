//! Test suite for QuDAG Exchange Core
//! 
//! This module organizes all tests following TDD methodology.
//! Tests are written before implementation to ensure comprehensive coverage.

// Core data structure tests
mod test_ledger;

// Cryptographic integration tests
mod test_vault_integration;

// Transaction processing tests
mod test_transactions;

// Resource management tests
mod test_resource_metering;

// Consensus protocol tests
mod test_consensus_integration;

// End-to-end integration tests
mod test_integration;

// Re-export test utilities for use in integration tests
pub(crate) mod test_utils {
    use qudag_exchange_core::ledger::{AccountId, Balance};
    use qudag_exchange_core::transaction::Transaction;
    use std::time::{SystemTime, UNIX_EPOCH};
    
    /// Generate a deterministic AccountId for testing
    pub fn test_account_id(name: &str) -> AccountId {
        AccountId::from_string(name)
    }
    
    /// Create a test transaction with default values
    pub fn test_transaction(
        sender: &str,
        recipient: &str,
        amount: u64,
        nonce: u64,
    ) -> Transaction {
        Transaction::new(
            test_account_id(sender),
            test_account_id(recipient),
            Balance::from_ruv(amount),
            nonce,
        )
    }
    
    /// Get current timestamp for testing
    pub fn test_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
    
    /// Create a test balance
    pub fn ruv(amount: u64) -> Balance {
        Balance::from_ruv(amount)
    }
}

// Common test fixtures
pub(crate) mod fixtures {
    use super::test_utils::*;
    use qudag_exchange_core::ledger::Ledger;
    use qudag_exchange_core::vault::{VaultManager, VaultConfig, KeyType};
    use tempfile::TempDir;
    
    /// Create a test ledger with some accounts
    pub fn create_test_ledger() -> Ledger {
        let mut ledger = Ledger::new();
        
        // Create test accounts
        let alice = test_account_id("alice");
        let bob = test_account_id("bob");
        let charlie = test_account_id("charlie");
        
        ledger.create_account(alice.clone()).unwrap();
        ledger.create_account(bob.clone()).unwrap();
        ledger.create_account(charlie.clone()).unwrap();
        
        // Give Alice some initial balance
        ledger.credit(&alice, ruv(10000)).unwrap();
        
        ledger
    }
    
    /// Create a test vault manager
    pub fn create_test_vault() -> (VaultManager, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let config = VaultConfig {
            path: temp_dir.path().join("test_vault.db"),
            auto_lock_timeout: None,
            use_hardware_security: false,
        };
        
        let mut manager = VaultManager::new(config).unwrap();
        manager.initialize("test_password").unwrap();
        manager.unlock("test_password").unwrap();
        
        (manager, temp_dir)
    }
    
    /// Generate test keypairs
    pub fn generate_test_keys(vault: &mut VaultManager) -> Vec<(String, Vec<u8>)> {
        let mut keys = Vec::new();
        
        for name in &["alice", "bob", "charlie"] {
            let keypair = vault
                .generate_key_pair(name, KeyType::MlDsa65)
                .unwrap();
            keys.push((name.to_string(), keypair.public_key()));
        }
        
        keys
    }
}

#[cfg(test)]
mod test_suite_validation {
    use super::*;
    
    #[test]
    fn test_all_modules_compile() {
        // This test ensures all test modules compile correctly
        // It will fail at compile time if any module has issues
        println!("All test modules compiled successfully");
    }
    
    #[test]
    fn test_fixtures_work() {
        // Verify test fixtures are functional
        let ledger = fixtures::create_test_ledger();
        assert_eq!(ledger.total_accounts(), 3);
        
        let (mut vault, _temp) = fixtures::create_test_vault();
        let keys = fixtures::generate_test_keys(&mut vault);
        assert_eq!(keys.len(), 3);
    }
}
