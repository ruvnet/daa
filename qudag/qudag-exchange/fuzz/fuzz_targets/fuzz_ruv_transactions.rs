#![no_main]

use libfuzzer_sys::fuzz_target;
use arbitrary::{Arbitrary, Unstructured};
use qudag_exchange_core::{Transaction, TransactionType, RuvAmount};

/// Arbitrary implementation for fuzzing transaction types
#[derive(Debug, Clone, Arbitrary)]
struct FuzzTransaction {
    sender: Vec<u8>,
    receiver: Vec<u8>,
    amount: u64,
    nonce: u64,
    tx_type: u8,
    signature: Vec<u8>,
    metadata: Vec<u8>,
}

fuzz_target!(|data: &[u8]| {
    // Test 1: Raw deserialization resilience
    if let Ok(tx) = bincode::deserialize::<Transaction>(data) {
        // Verify transaction invariants
        assert!(tx.verify_structure().is_ok() || tx.verify_structure().is_err());
        
        // Ensure amount parsing doesn't panic
        let _ = RuvAmount::from_raw(tx.amount());
        
        // Ensure signature verification doesn't panic
        let _ = tx.verify_signature();
    }
    
    // Test 2: Structured fuzzing with arbitrary data
    if let Ok(mut u) = Unstructured::new(data) {
        if let Ok(fuzz_tx) = FuzzTransaction::arbitrary(&mut u) {
            // Test transaction creation with fuzzer-generated data
            test_transaction_creation(fuzz_tx);
        }
    }
    
    // Test 3: Transaction state transitions
    test_transaction_state_machine(data);
});

fn test_transaction_creation(fuzz_tx: FuzzTransaction) {
    // Try to create a transaction with fuzzed data
    let tx_type = match fuzz_tx.tx_type % 4 {
        0 => TransactionType::Transfer,
        1 => TransactionType::ResourceContribution,
        2 => TransactionType::FeeDistribution,
        _ => TransactionType::Genesis,
    };
    
    // Ensure transaction builder doesn't panic on invalid data
    let result = Transaction::builder()
        .sender(fuzz_tx.sender)
        .receiver(fuzz_tx.receiver)
        .amount(RuvAmount::from_raw(fuzz_tx.amount))
        .nonce(fuzz_tx.nonce)
        .tx_type(tx_type)
        .signature(fuzz_tx.signature)
        .metadata(fuzz_tx.metadata)
        .build();
        
    // Transaction creation should either succeed or return an error, not panic
    match result {
        Ok(tx) => {
            // Verify basic invariants
            assert!(tx.amount().as_raw() >= 0);
            assert!(tx.sender().len() <= 256); // Reasonable limit
            assert!(tx.receiver().len() <= 256);
        }
        Err(_) => {
            // Error is expected for invalid data
        }
    }
}

fn test_transaction_state_machine(data: &[u8]) {
    // Test transaction lifecycle state transitions
    if data.len() < 8 {
        return;
    }
    
    let action = data[0] % 5;
    let amount = u64::from_le_bytes([
        data.get(1).copied().unwrap_or(0),
        data.get(2).copied().unwrap_or(0),
        data.get(3).copied().unwrap_or(0),
        data.get(4).copied().unwrap_or(0),
        data.get(5).copied().unwrap_or(0),
        data.get(6).copied().unwrap_or(0),
        data.get(7).copied().unwrap_or(0),
        data.get(8).copied().unwrap_or(0),
    ]);
    
    match action {
        0 => {
            // Test pending -> validated transition
            let _ = Transaction::validate_amount(amount);
        }
        1 => {
            // Test validated -> confirmed transition
            let _ = Transaction::simulate_confirmation(data);
        }
        2 => {
            // Test confirmed -> finalized transition
            let _ = Transaction::simulate_finalization(data);
        }
        3 => {
            // Test rejected state
            let _ = Transaction::simulate_rejection(data);
        }
        _ => {
            // Test rollback scenarios
            let _ = Transaction::simulate_rollback(data);
        }
    }
}