#![no_main]

use libfuzzer_sys::fuzz_target;
use arbitrary::{Arbitrary, Unstructured};
use qudag_exchange_core::{Ledger, RuvAmount, AccountId};
use std::collections::HashMap;

/// Fuzzing operation types for ledger
#[derive(Debug, Clone, Copy, Arbitrary)]
enum LedgerOperation {
    Credit { account_idx: u8, amount: u64 },
    Debit { account_idx: u8, amount: u64 },
    Transfer { from_idx: u8, to_idx: u8, amount: u64 },
    CreateAccount { idx: u8, initial_balance: u64 },
    QueryBalance { idx: u8 },
    AtomicBatch { ops: u8 }, // Number of operations in batch
}

fuzz_target!(|data: &[u8]| {
    if let Ok(mut u) = Unstructured::new(data) {
        // Initialize ledger and shadow state for verification
        let mut ledger = Ledger::new();
        let mut shadow_balances: HashMap<u8, u128> = HashMap::new();
        let mut total_supply: u128 = 0;
        
        // Generate and apply random operations
        while !u.is_empty() {
            if let Ok(op) = LedgerOperation::arbitrary(&mut u) {
                apply_operation(&mut ledger, &mut shadow_balances, &mut total_supply, op);
                
                // Verify invariants after each operation
                verify_ledger_invariants(&ledger, &shadow_balances, total_supply);
            }
        }
        
        // Final comprehensive verification
        verify_final_state(&ledger, &shadow_balances, total_supply);
    }
});

fn apply_operation(
    ledger: &mut Ledger,
    shadow: &mut HashMap<u8, u128>,
    total_supply: &mut u128,
    op: LedgerOperation,
) {
    match op {
        LedgerOperation::CreateAccount { idx, initial_balance } => {
            let account_id = AccountId::from_index(idx);
            let amount = RuvAmount::from_raw(initial_balance.min(1_000_000_000)); // Cap for testing
            
            // Only create if doesn't exist
            if !shadow.contains_key(&idx) {
                let result = ledger.create_account(account_id.clone(), amount);
                if result.is_ok() {
                    shadow.insert(idx, amount.as_raw());
                    *total_supply += amount.as_raw();
                }
            }
        }
        
        LedgerOperation::Credit { account_idx, amount } => {
            let account_id = AccountId::from_index(account_idx);
            let credit_amount = RuvAmount::from_raw(amount.min(1_000_000_000));
            
            if shadow.contains_key(&account_idx) {
                let result = ledger.credit(&account_id, credit_amount);
                if result.is_ok() {
                    *shadow.get_mut(&account_idx).unwrap() += credit_amount.as_raw();
                    *total_supply += credit_amount.as_raw();
                }
            }
        }
        
        LedgerOperation::Debit { account_idx, amount } => {
            let account_id = AccountId::from_index(account_idx);
            let debit_amount = RuvAmount::from_raw(amount.min(1_000_000_000));
            
            if let Some(balance) = shadow.get_mut(&account_idx) {
                if *balance >= debit_amount.as_raw() {
                    let result = ledger.debit(&account_id, debit_amount);
                    if result.is_ok() {
                        *balance -= debit_amount.as_raw();
                        *total_supply -= debit_amount.as_raw();
                    }
                }
            }
        }
        
        LedgerOperation::Transfer { from_idx, to_idx, amount } => {
            if from_idx != to_idx {
                let from_id = AccountId::from_index(from_idx);
                let to_id = AccountId::from_index(to_idx);
                let transfer_amount = RuvAmount::from_raw(amount.min(1_000_000_000));
                
                // Both accounts must exist
                if shadow.contains_key(&from_idx) && shadow.contains_key(&to_idx) {
                    let from_balance = *shadow.get(&from_idx).unwrap();
                    if from_balance >= transfer_amount.as_raw() {
                        let result = ledger.transfer(&from_id, &to_id, transfer_amount);
                        if result.is_ok() {
                            *shadow.get_mut(&from_idx).unwrap() -= transfer_amount.as_raw();
                            *shadow.get_mut(&to_idx).unwrap() += transfer_amount.as_raw();
                            // Total supply remains unchanged in transfers
                        }
                    }
                }
            }
        }
        
        LedgerOperation::QueryBalance { idx } => {
            let account_id = AccountId::from_index(idx);
            let _ = ledger.get_balance(&account_id);
        }
        
        LedgerOperation::AtomicBatch { ops } => {
            // Test atomic batch operations
            let num_ops = (ops % 10) + 1; // 1-10 operations
            let mut batch_ops = Vec::new();
            
            for i in 0..num_ops {
                // Create simple transfer operations for the batch
                let from = i % 5;
                let to = (i + 1) % 5;
                let amount = RuvAmount::from_raw(100);
                
                batch_ops.push((
                    AccountId::from_index(from),
                    AccountId::from_index(to),
                    amount,
                ));
            }
            
            // Apply batch atomically
            let _ = ledger.apply_batch(batch_ops);
        }
    }
}

fn verify_ledger_invariants(
    ledger: &Ledger,
    shadow: &HashMap<u8, u128>,
    expected_total: u128,
) {
    // Invariant 1: All balances must be non-negative (enforced by type system with u128)
    
    // Invariant 2: Shadow state must match ledger state
    for (idx, expected_balance) in shadow.iter() {
        let account_id = AccountId::from_index(*idx);
        if let Ok(actual_balance) = ledger.get_balance(&account_id) {
            assert_eq!(
                actual_balance.as_raw(),
                *expected_balance,
                "Balance mismatch for account {}",
                idx
            );
        }
    }
    
    // Invariant 3: Total supply conservation
    let ledger_total = ledger.total_supply();
    assert_eq!(
        ledger_total.as_raw(),
        expected_total,
        "Total supply mismatch: ledger has {}, expected {}",
        ledger_total.as_raw(),
        expected_total
    );
    
    // Invariant 4: No phantom accounts
    assert_eq!(
        ledger.account_count(),
        shadow.len(),
        "Account count mismatch"
    );
}

fn verify_final_state(
    ledger: &Ledger,
    shadow: &HashMap<u8, u128>,
    total_supply: u128,
) {
    // Comprehensive final verification
    verify_ledger_invariants(ledger, shadow, total_supply);
    
    // Additional checks
    // 1. Verify sum of all balances equals total supply
    let sum_balances: u128 = shadow.values().sum();
    assert_eq!(
        sum_balances,
        total_supply,
        "Sum of balances doesn't match total supply"
    );
    
    // 2. Verify no overflow occurred
    assert!(
        total_supply <= qudag_exchange_core::MAX_RUV_SUPPLY,
        "Total supply exceeded maximum"
    );
    
    // 3. Verify ledger can be serialized/deserialized without data loss
    if let Ok(serialized) = ledger.serialize() {
        if let Ok(deserialized) = Ledger::deserialize(&serialized) {
            assert_eq!(
                ledger.total_supply(),
                deserialized.total_supply(),
                "Serialization roundtrip failed"
            );
        }
    }
}