//! Property-based tests for QuDAG Exchange ledger invariants
//!
//! These tests verify critical properties that must hold for all possible
//! sequences of ledger operations to ensure rUv conservation and consistency.

use proptest::prelude::*;
use proptest::test_runner::{Config, TestRunner};
use qudag_exchange_core::{Ledger, RuvAmount, AccountId, Error};
use std::collections::{HashMap, HashSet};

/// Maximum number of accounts for property testing
const MAX_ACCOUNTS: usize = 100;

/// Maximum rUv amount for individual operations
const MAX_OPERATION_AMOUNT: u64 = 1_000_000_000;

/// Strategy for generating account IDs
fn account_id_strategy() -> impl Strategy<Value = AccountId> {
    (0u8..MAX_ACCOUNTS as u8).prop_map(AccountId::from_index)
}

/// Strategy for generating rUv amounts
fn ruv_amount_strategy() -> impl Strategy<Value = RuvAmount> {
    (0u64..=MAX_OPERATION_AMOUNT).prop_map(RuvAmount::from_raw)
}

/// Ledger operation types for property testing
#[derive(Debug, Clone)]
enum LedgerOp {
    CreateAccount { id: AccountId, initial_balance: RuvAmount },
    Credit { account: AccountId, amount: RuvAmount },
    Debit { account: AccountId, amount: RuvAmount },
    Transfer { from: AccountId, to: AccountId, amount: RuvAmount },
    Burn { account: AccountId, amount: RuvAmount },
    Mint { account: AccountId, amount: RuvAmount },
}

/// Strategy for generating ledger operations
fn ledger_op_strategy() -> impl Strategy<Value = LedgerOp> {
    prop_oneof![
        // CreateAccount (20% probability)
        (account_id_strategy(), ruv_amount_strategy())
            .prop_map(|(id, balance)| LedgerOp::CreateAccount { 
                id, 
                initial_balance: balance 
            }),
        
        // Credit (20% probability)
        (account_id_strategy(), ruv_amount_strategy())
            .prop_map(|(account, amount)| LedgerOp::Credit { account, amount }),
        
        // Debit (20% probability)
        (account_id_strategy(), ruv_amount_strategy())
            .prop_map(|(account, amount)| LedgerOp::Debit { account, amount }),
        
        // Transfer (30% probability)
        (account_id_strategy(), account_id_strategy(), ruv_amount_strategy())
            .prop_map(|(from, to, amount)| LedgerOp::Transfer { from, to, amount }),
        
        // Burn (5% probability - for testing supply reduction)
        (account_id_strategy(), ruv_amount_strategy())
            .prop_map(|(account, amount)| LedgerOp::Burn { account, amount }),
        
        // Mint (5% probability - for testing supply increase)
        (account_id_strategy(), ruv_amount_strategy())
            .prop_map(|(account, amount)| LedgerOp::Mint { account, amount })
    ]
}

/// Property: Total supply conservation
/// The sum of all account balances must equal the total supply at all times
#[test]
fn prop_total_supply_conservation() {
    let mut config = Config::default();
    config.cases = 1000; // Run 1000 test cases
    
    proptest!(config, |(ops in prop::collection::vec(ledger_op_strategy(), 1..100))| {
        let mut ledger = Ledger::new();
        let mut expected_total: u128 = 0;
        let mut account_balances: HashMap<AccountId, u128> = HashMap::new();
        
        for op in ops {
            match op {
                LedgerOp::CreateAccount { id, initial_balance } => {
                    if !account_balances.contains_key(&id) {
                        if ledger.create_account(id.clone(), initial_balance).is_ok() {
                            account_balances.insert(id, initial_balance.as_raw());
                            expected_total += initial_balance.as_raw();
                        }
                    }
                }
                
                LedgerOp::Credit { account, amount } => {
                    if account_balances.contains_key(&account) {
                        if ledger.credit(&account, amount).is_ok() {
                            *account_balances.get_mut(&account).unwrap() += amount.as_raw();
                            expected_total += amount.as_raw();
                        }
                    }
                }
                
                LedgerOp::Debit { account, amount } => {
                    if let Some(balance) = account_balances.get_mut(&account) {
                        if *balance >= amount.as_raw() {
                            if ledger.debit(&account, amount).is_ok() {
                                *balance -= amount.as_raw();
                                expected_total -= amount.as_raw();
                            }
                        }
                    }
                }
                
                LedgerOp::Transfer { from, to, amount } => {
                    if from != to && 
                       account_balances.contains_key(&from) && 
                       account_balances.contains_key(&to) {
                        let from_balance = *account_balances.get(&from).unwrap();
                        if from_balance >= amount.as_raw() {
                            if ledger.transfer(&from, &to, amount).is_ok() {
                                *account_balances.get_mut(&from).unwrap() -= amount.as_raw();
                                *account_balances.get_mut(&to).unwrap() += amount.as_raw();
                                // Total remains unchanged in transfers
                            }
                        }
                    }
                }
                
                LedgerOp::Burn { account, amount } => {
                    if let Some(balance) = account_balances.get_mut(&account) {
                        if *balance >= amount.as_raw() {
                            if ledger.burn(&account, amount).is_ok() {
                                *balance -= amount.as_raw();
                                expected_total -= amount.as_raw();
                            }
                        }
                    }
                }
                
                LedgerOp::Mint { account, amount } => {
                    if account_balances.contains_key(&account) {
                        // Check if minting would exceed max supply
                        if expected_total + amount.as_raw() <= qudag_exchange_core::MAX_RUV_SUPPLY {
                            if ledger.mint(&account, amount).is_ok() {
                                *account_balances.get_mut(&account).unwrap() += amount.as_raw();
                                expected_total += amount.as_raw();
                            }
                        }
                    }
                }
            }
            
            // Verify total supply matches expected
            prop_assert_eq!(
                ledger.total_supply().as_raw(),
                expected_total,
                "Total supply mismatch after operation"
            );
            
            // Verify sum of balances equals total supply
            let sum_balances: u128 = account_balances.values().sum();
            prop_assert_eq!(
                sum_balances,
                expected_total,
                "Sum of balances doesn't match total supply"
            );
        }
    });
}

/// Property: No negative balances
/// All account balances must be non-negative (enforced by u128 type)
#[test]
fn prop_no_negative_balances() {
    let config = Config::with_cases(500);
    
    proptest!(config, |(ops in prop::collection::vec(ledger_op_strategy(), 1..200))| {
        let mut ledger = Ledger::new();
        let mut accounts = HashSet::new();
        
        for op in ops {
            match op {
                LedgerOp::CreateAccount { id, initial_balance } => {
                    let _ = ledger.create_account(id.clone(), initial_balance);
                    accounts.insert(id);
                }
                
                LedgerOp::Debit { account, amount } => {
                    // Attempt to debit more than available should fail
                    if accounts.contains(&account) {
                        if let Ok(balance) = ledger.get_balance(&account) {
                            if amount.as_raw() > balance.as_raw() {
                                // This should fail
                                prop_assert!(
                                    ledger.debit(&account, amount).is_err(),
                                    "Debit exceeding balance should fail"
                                );
                                
                                // Balance should remain unchanged
                                prop_assert_eq!(
                                    ledger.get_balance(&account).unwrap(),
                                    balance,
                                    "Balance changed after failed debit"
                                );
                            }
                        }
                    }
                }
                
                _ => {
                    // Apply other operations normally
                    apply_operation(&mut ledger, &op);
                }
            }
            
            // Verify all balances are valid (type system ensures non-negative)
            for account in &accounts {
                if let Ok(balance) = ledger.get_balance(account) {
                    // Balance is u128, so always >= 0
                    prop_assert!(balance.as_raw() <= qudag_exchange_core::MAX_RUV_SUPPLY);
                }
            }
        }
    });
}

/// Property: Transfer atomicity
/// Transfers must be atomic - either both accounts are updated or neither
#[test]
fn prop_transfer_atomicity() {
    let config = Config::with_cases(500);
    
    proptest!(config, |(
        from_idx in 0u8..10,
        to_idx in 0u8..10,
        amount in 1u64..1000,
        initial_balance in 1000u64..10000
    )| {
        let mut ledger = Ledger::new();
        let from = AccountId::from_index(from_idx);
        let to = AccountId::from_index(to_idx);
        let transfer_amount = RuvAmount::from_raw(amount);
        let initial = RuvAmount::from_raw(initial_balance);
        
        // Setup accounts
        ledger.create_account(from.clone(), initial).unwrap();
        ledger.create_account(to.clone(), RuvAmount::from_raw(0)).unwrap();
        
        let from_balance_before = ledger.get_balance(&from).unwrap();
        let to_balance_before = ledger.get_balance(&to).unwrap();
        let total_before = ledger.total_supply();
        
        // Attempt transfer
        let result = ledger.transfer(&from, &to, transfer_amount);
        
        if from == to {
            // Self-transfer should fail
            prop_assert!(result.is_err());
            // Balances unchanged
            prop_assert_eq!(ledger.get_balance(&from).unwrap(), from_balance_before);
        } else if transfer_amount.as_raw() > from_balance_before.as_raw() {
            // Insufficient balance should fail
            prop_assert!(result.is_err());
            // Both balances unchanged
            prop_assert_eq!(ledger.get_balance(&from).unwrap(), from_balance_before);
            prop_assert_eq!(ledger.get_balance(&to).unwrap(), to_balance_before);
        } else {
            // Valid transfer should succeed
            prop_assert!(result.is_ok());
            // Verify atomic update
            prop_assert_eq!(
                ledger.get_balance(&from).unwrap().as_raw(),
                from_balance_before.as_raw() - transfer_amount.as_raw()
            );
            prop_assert_eq!(
                ledger.get_balance(&to).unwrap().as_raw(),
                to_balance_before.as_raw() + transfer_amount.as_raw()
            );
        }
        
        // Total supply must remain unchanged
        prop_assert_eq!(ledger.total_supply(), total_before);
    });
}

/// Property: Concurrent operation safety
/// Simulate concurrent operations and verify consistency
#[test]
fn prop_concurrent_operation_safety() {
    use std::sync::{Arc, Mutex};
    use std::thread;
    
    let config = Config::with_cases(100);
    
    proptest!(config, |(
        ops1 in prop::collection::vec(ledger_op_strategy(), 10..50),
        ops2 in prop::collection::vec(ledger_op_strategy(), 10..50),
        ops3 in prop::collection::vec(ledger_op_strategy(), 10..50)
    )| {
        let ledger = Arc::new(Mutex::new(Ledger::new()));
        
        // Pre-create some accounts
        for i in 0..10 {
            let account = AccountId::from_index(i);
            let _ = ledger.lock().unwrap()
                .create_account(account, RuvAmount::from_raw(1000));
        }
        
        let initial_total = ledger.lock().unwrap().total_supply();
        
        // Spawn threads to apply operations concurrently
        let ledger1 = Arc::clone(&ledger);
        let handle1 = thread::spawn(move || {
            for op in ops1 {
                apply_operation_safe(&ledger1, &op);
            }
        });
        
        let ledger2 = Arc::clone(&ledger);
        let handle2 = thread::spawn(move || {
            for op in ops2 {
                apply_operation_safe(&ledger2, &op);
            }
        });
        
        let ledger3 = Arc::clone(&ledger);
        let handle3 = thread::spawn(move || {
            for op in ops3 {
                apply_operation_safe(&ledger3, &op);
            }
        });
        
        // Wait for all threads
        handle1.join().unwrap();
        handle2.join().unwrap();
        handle3.join().unwrap();
        
        // Verify consistency
        let final_ledger = ledger.lock().unwrap();
        
        // Total supply should be conserved (assuming no mint/burn in ops)
        let mut total_from_accounts = 0u128;
        for i in 0..MAX_ACCOUNTS {
            let account = AccountId::from_index(i as u8);
            if let Ok(balance) = final_ledger.get_balance(&account) {
                total_from_accounts += balance.as_raw();
            }
        }
        
        // The sum of all balances should equal total supply
        prop_assert_eq!(
            total_from_accounts,
            final_ledger.total_supply().as_raw(),
            "Concurrent operations broke supply conservation"
        );
    });
}

/// Property: Serialization round-trip preserves state
#[test]
fn prop_serialization_preserves_state() {
    let config = Config::with_cases(200);
    
    proptest!(config, |(ops in prop::collection::vec(ledger_op_strategy(), 1..100))| {
        let mut ledger = Ledger::new();
        
        // Apply operations
        for op in ops {
            apply_operation(&mut ledger, &op);
        }
        
        // Serialize
        let serialized = ledger.serialize().expect("Serialization should succeed");
        
        // Deserialize
        let restored = Ledger::deserialize(&serialized).expect("Deserialization should succeed");
        
        // Verify state preservation
        prop_assert_eq!(
            ledger.total_supply(),
            restored.total_supply(),
            "Total supply not preserved"
        );
        
        prop_assert_eq!(
            ledger.account_count(),
            restored.account_count(),
            "Account count not preserved"
        );
        
        // Verify all account balances match
        for i in 0..MAX_ACCOUNTS {
            let account = AccountId::from_index(i as u8);
            match (ledger.get_balance(&account), restored.get_balance(&account)) {
                (Ok(balance1), Ok(balance2)) => {
                    prop_assert_eq!(balance1, balance2, "Balance mismatch for account {}", i);
                }
                (Err(_), Err(_)) => {
                    // Both don't have the account - OK
                }
                _ => {
                    prop_assert!(false, "Account existence mismatch for account {}", i);
                }
            }
        }
    });
}

// Helper functions

fn apply_operation(ledger: &mut Ledger, op: &LedgerOp) {
    match op {
        LedgerOp::CreateAccount { id, initial_balance } => {
            let _ = ledger.create_account(id.clone(), *initial_balance);
        }
        LedgerOp::Credit { account, amount } => {
            let _ = ledger.credit(account, *amount);
        }
        LedgerOp::Debit { account, amount } => {
            let _ = ledger.debit(account, *amount);
        }
        LedgerOp::Transfer { from, to, amount } => {
            let _ = ledger.transfer(from, to, *amount);
        }
        LedgerOp::Burn { account, amount } => {
            let _ = ledger.burn(account, *amount);
        }
        LedgerOp::Mint { account, amount } => {
            let _ = ledger.mint(account, *amount);
        }
    }
}

fn apply_operation_safe(ledger: &Arc<Mutex<Ledger>>, op: &LedgerOp) {
    let mut ledger = ledger.lock().unwrap();
    apply_operation(&mut ledger, op);
}