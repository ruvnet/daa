#![no_main]
use libfuzzer_sys::fuzz_target;
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant, SystemTime};
use serde::{Serialize, Deserialize};
use qudag_protocol::{
    validation::{Validator, ValidationResult, ValidationRule, ValidationContext},
    transaction::{Transaction, TransactionType},
    message::{ProtocolMessage, MessageType, MessageHeader},
    node::{Node, NodeId, NodeStatus},
    types::{Hash, Timestamp, Nonce, Amount},
    config::SecurityConfig,
};
use qudag_crypto::{
    signature::{DigitalSignature, Signature, PublicKey, PrivateKey},
    ml_dsa::MlDsa65,
    fingerprint::QuantumFingerprint,
};

/// Test validation context
#[derive(Debug, Clone)]
pub struct TestValidationContext {
    pub current_time: u64,
    pub block_height: u64,
    pub network_id: u8,
    pub known_nodes: HashSet<NodeId>,
    pub transaction_history: HashMap<Hash, Transaction>,
}

/// Test transaction with validation data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestTransaction {
    pub hash: Hash,
    pub from: NodeId,
    pub to: NodeId,
    pub amount: u64,
    pub fee: u64,
    pub nonce: u64,
    pub timestamp: u64,
    pub signature: Vec<u8>,
    pub transaction_type: TransactionType,
}

/// Test signature data
#[derive(Debug, Clone)]
pub struct TestSignatureData {
    pub message: Vec<u8>,
    pub signature: Vec<u8>,
    pub public_key: Vec<u8>,
    pub is_valid: bool,
}

/// Helper to create test transaction from fuzz data
fn create_test_transaction(data: &[u8], tx_id: u64) -> Option<TestTransaction> {
    if data.len() < 64 {
        return None;
    }
    
    let hash = Hash::from_bytes(&data[..32]);
    let from = NodeId::from_bytes(&data[32..40]);
    let to = NodeId::from_bytes(&data[40..48]);
    let amount = u64::from_le_bytes([
        data[48], data[49], data[50], data[51],
        data[52], data[53], data[54], data[55]
    ]);
    let fee = u64::from_le_bytes([
        data[56], data[57], data[58], data[59],
        data[60], data[61], data[62], data[63]
    ]);
    
    let nonce = tx_id; // Use tx_id as nonce for uniqueness
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let signature = if data.len() > 64 {
        data[64..std::cmp::min(data.len(), 128)].to_vec()
    } else {
        vec![0u8; 32]
    };
    
    let transaction_type = match data[32] % 4 {
        0 => TransactionType::Transfer,
        1 => TransactionType::Stake,
        2 => TransactionType::Unstake,
        _ => TransactionType::Contract,
    };
    
    Some(TestTransaction {
        hash,
        from,
        to,
        amount,
        fee,
        nonce,
        timestamp,
        signature,
        transaction_type,
    })
}

/// Helper to create test validation context
fn create_test_validation_context(data: &[u8]) -> TestValidationContext {
    let current_time = if data.len() >= 8 {
        u64::from_le_bytes([
            data[0], data[1], data[2], data[3],
            data[4], data[5], data[6], data[7]
        ])
    } else {
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    };
    
    let block_height = if data.len() >= 16 {
        u64::from_le_bytes([
            data[8], data[9], data[10], data[11],
            data[12], data[13], data[14], data[15]
        ])
    } else {
        1000
    };
    
    let network_id = if data.len() > 16 {
        data[16]
    } else {
        1
    };
    
    let mut known_nodes = HashSet::new();
    let max_nodes = std::cmp::min(data.len() / 8, 20);
    
    for i in 0..max_nodes {
        let start_idx = i * 8;
        let end_idx = std::cmp::min(start_idx + 8, data.len());
        
        if end_idx > start_idx && end_idx - start_idx >= 8 {
            let node_id = NodeId::from_bytes(&data[start_idx..start_idx + 8]);
            known_nodes.insert(node_id);
        }
    }
    
    TestValidationContext {
        current_time,
        block_height,
        network_id,
        known_nodes,
        transaction_history: HashMap::new(),
    }
}

/// Helper to create test signature data
fn create_test_signature_data(data: &[u8]) -> Option<TestSignatureData> {
    if data.len() < 64 {
        return None;
    }
    
    let message = data[..32].to_vec();
    let signature = data[32..64].to_vec();
    let public_key = if data.len() >= 96 {
        data[64..96].to_vec()
    } else {
        vec![0u8; 32]
    };
    
    // Randomly determine if signature should be valid (for testing purposes)
    let is_valid = data[0] % 2 == 0;
    
    Some(TestSignatureData {
        message,
        signature,
        public_key,
        is_valid,
    })
}

/// Test basic transaction validation
fn test_transaction_validation(
    transactions: &[TestTransaction],
    context: &TestValidationContext
) -> Result<(), String> {
    let security_config = SecurityConfig::default();
    let mut validator = Validator::new(security_config);
    
    for tx in transactions {
        // Convert to protocol transaction
        let protocol_tx = Transaction::new(
            tx.hash.clone(),
            tx.from.clone(),
            tx.to.clone(),
            tx.amount,
            Nonce::from(tx.nonce),
        );
        
        // Create validation context
        let validation_context = ValidationContext::new(
            context.current_time,
            context.block_height,
            context.network_id,
        );
        
        match validator.validate_transaction(&protocol_tx, &validation_context) {
            Ok(ValidationResult::Valid) => {
                // Transaction is valid
            },
            Ok(ValidationResult::Invalid(reason)) => {
                // Transaction is invalid - expected with fuzz data
                continue;
            },
            Err(e) => {
                // Validation error - also expected with fuzz data
                continue;
            }
        }
    }
    
    Ok(())
}

/// Test signature validation
fn test_signature_validation(signature_data: &[TestSignatureData]) -> Result<(), String> {
    let dsa = MlDsa65::new();
    
    // Generate a real keypair for testing
    let (real_pk, real_sk) = dsa.generate_keypair()
        .map_err(|e| format!("Failed to generate keypair: {:?}", e))?;
    
    for sig_data in signature_data {
        // Test with real signature
        if !sig_data.message.is_empty() {
            let real_signature = dsa.sign(&real_sk, &sig_data.message)
                .map_err(|e| format!("Failed to create real signature: {:?}", e))?;
            
            // This should verify successfully
            match dsa.verify(&real_pk, &sig_data.message, &real_signature) {
                Ok(_) => {}, // Expected success
                Err(e) => return Err(format!("Real signature verification failed: {:?}", e)),
            }
        }
        
        // Test with fuzz signature data
        if sig_data.signature.len() >= 32 && sig_data.public_key.len() >= 32 {
            // Try to create signature and public key from fuzz data
            if let (Ok(fuzz_sig), Ok(fuzz_pk)) = (
                qudag_crypto::signature::Signature::from_bytes(&sig_data.signature),
                qudag_crypto::signature::PublicKey::from_bytes(&sig_data.public_key)
            ) {
                // This should mostly fail with fuzz data
                let _ = dsa.verify(&fuzz_pk, &sig_data.message, &fuzz_sig);
                // Don't assert failure - just ensure it doesn't crash
            }
        }
    }
    
    Ok(())
}

/// Test message validation
fn test_message_validation(data: &[u8]) -> Result<(), String> {
    if data.len() < 32 {
        return Ok(());
    }
    
    let security_config = SecurityConfig::default();
    let mut validator = Validator::new(security_config);
    
    // Create test messages from fuzz data
    let max_messages = std::cmp::min(data.len() / 32, 10);
    
    for i in 0..max_messages {
        let start_idx = i * 32;
        let end_idx = std::cmp::min(start_idx + 32, data.len());
        
        if end_idx > start_idx {
            let msg_data = &data[start_idx..end_idx];
            
            let sender = NodeId::from_bytes(&msg_data[..8]);
            let recipient = NodeId::from_bytes(&msg_data[8..16]);
            let sequence = u64::from_le_bytes([
                msg_data[16], msg_data[17], msg_data[18], msg_data[19],
                msg_data[20], msg_data[21], msg_data[22], msg_data[23]
            ]);
            
            let header = MessageHeader::new(sender, recipient, sequence);
            let message_type = match msg_data[24] % 4 {
                0 => MessageType::Consensus,
                1 => MessageType::Transaction,
                2 => MessageType::Heartbeat,
                _ => MessageType::Discovery,
            };
            
            let payload = msg_data[25..].to_vec();
            let protocol_msg = ProtocolMessage::new(header, message_type, payload);
            
            // Test message validation
            match validator.validate_message(&protocol_msg) {
                Ok(ValidationResult::Valid) => {},
                Ok(ValidationResult::Invalid(_)) => continue,
                Err(_) => continue,
            }
        }
    }
    
    Ok(())
}

/// Test node validation
fn test_node_validation(data: &[u8]) -> Result<(), String> {
    if data.len() < 16 {
        return Ok(());
    }
    
    let security_config = SecurityConfig::default();
    let mut validator = Validator::new(security_config);
    
    // Create test nodes from fuzz data
    let max_nodes = std::cmp::min(data.len() / 16, 15);
    
    for i in 0..max_nodes {
        let start_idx = i * 16;
        let end_idx = std::cmp::min(start_idx + 16, data.len());
        
        if end_idx > start_idx && end_idx - start_idx >= 16 {
            let node_data = &data[start_idx..end_idx];
            
            let node_id = NodeId::from_bytes(&node_data[..8]);
            let stake = u64::from_le_bytes([
                node_data[8], node_data[9], node_data[10], node_data[11],
                node_data[12], node_data[13], node_data[14], node_data[15]
            ]);
            
            let is_validator = node_data[8] % 2 == 0;
            let node = Node::new(node_id, stake, is_validator);
            
            // Test node validation
            match validator.validate_node(&node) {
                Ok(ValidationResult::Valid) => {},
                Ok(ValidationResult::Invalid(_)) => continue,
                Err(_) => continue,
            }
        }
    }
    
    Ok(())
}

/// Test quantum fingerprint validation
fn test_fingerprint_validation(data: &[u8]) -> Result<(), String> {
    if data.is_empty() {
        return Ok(());
    }
    
    let fingerprint = QuantumFingerprint::new();
    
    // Test fingerprint generation and validation
    let chunks = data.chunks(32);
    
    for chunk in chunks {
        if chunk.is_empty() {
            continue;
        }
        
        // Generate fingerprint
        let fp = fingerprint.generate(chunk)
            .map_err(|e| format!("Failed to generate fingerprint: {:?}", e))?;
        
        // Test verification with correct data
        match fingerprint.verify(chunk, &fp) {
            Ok(_) => {}, // Expected success
            Err(e) => return Err(format!("Fingerprint verification failed: {:?}", e)),
        }
        
        // Test verification with modified data
        if chunk.len() > 1 {
            let mut modified_chunk = chunk.to_vec();
            modified_chunk[0] ^= 1; // Flip one bit
            
            match fingerprint.verify(&modified_chunk, &fp) {
                Ok(_) => return Err("Modified data should not verify".to_string()),
                Err(_) => {}, // Expected failure
            }
        }
    }
    
    Ok(())
}

/// Test validation rule combinations
fn test_validation_rules(
    transactions: &[TestTransaction],
    context: &TestValidationContext
) -> Result<(), String> {
    let security_config = SecurityConfig::default();
    let mut validator = Validator::new(security_config);
    
    // Define test validation rules
    let rules = vec![
        ValidationRule::NonceOrder,
        ValidationRule::BalanceSufficiency,
        ValidationRule::SignatureValid,
        ValidationRule::TimestampRecent,
        ValidationRule::FeeMinimum,
    ];
    
    for tx in transactions {
        let protocol_tx = Transaction::new(
            tx.hash.clone(),
            tx.from.clone(),
            tx.to.clone(),
            tx.amount,
            Nonce::from(tx.nonce),
        );
        
        let validation_context = ValidationContext::new(
            context.current_time,
            context.block_height,
            context.network_id,
        );
        
        // Test each validation rule individually
        for rule in &rules {
            match validator.validate_with_rule(&protocol_tx, &validation_context, rule) {
                Ok(ValidationResult::Valid) => {},
                Ok(ValidationResult::Invalid(_)) => continue,
                Err(_) => continue,
            }
        }
        
        // Test all rules combined
        match validator.validate_with_all_rules(&protocol_tx, &validation_context, &rules) {
            Ok(ValidationResult::Valid) => {},
            Ok(ValidationResult::Invalid(_)) => continue,
            Err(_) => continue,
        }
    }
    
    Ok(())
}

/// Test edge cases and malformed data
fn test_validation_edge_cases(data: &[u8]) -> Result<(), String> {
    let security_config = SecurityConfig::default();
    let mut validator = Validator::new(security_config);
    
    // Test with empty transaction
    let empty_tx = Transaction::new(
        Hash::from_bytes(&[0u8; 32]),
        NodeId::from_bytes(&[0u8; 8]),
        NodeId::from_bytes(&[0u8; 8]),
        0,
        Nonce::from(0),
    );
    
    let context = ValidationContext::new(0, 0, 0);
    let _ = validator.validate_transaction(&empty_tx, &context);
    
    // Test with maximum values
    let max_tx = Transaction::new(
        Hash::from_bytes(&[0xFFu8; 32]),
        NodeId::from_bytes(&[0xFFu8; 8]),
        NodeId::from_bytes(&[0xFFu8; 8]),
        u64::MAX,
        Nonce::from(u64::MAX),
    );
    
    let max_context = ValidationContext::new(u64::MAX, u64::MAX, u8::MAX);
    let _ = validator.validate_transaction(&max_tx, &max_context);
    
    // Test with self-transactions (from == to)
    if data.len() >= 8 {
        let node_id = NodeId::from_bytes(&data[..8]);
        let self_tx = Transaction::new(
            Hash::from_bytes(&data[..32]),
            node_id.clone(),
            node_id,
            100,
            Nonce::from(1),
        );
        
        let _ = validator.validate_transaction(&self_tx, &context);
    }
    
    // Test with zero amounts and fees
    let zero_tx = Transaction::new(
        Hash::from_bytes(&data[..std::cmp::min(32, data.len())]),
        NodeId::from_bytes(&data[..std::cmp::min(8, data.len())]),
        NodeId::from_bytes(&data[std::cmp::min(8, data.len())..std::cmp::min(16, data.len())]),
        0,
        Nonce::from(0),
    );
    
    let _ = validator.validate_transaction(&zero_tx, &context);
    
    Ok(())
}

/// Test concurrent validation operations
fn test_concurrent_validation(
    transactions: &[TestTransaction],
    context: &TestValidationContext
) -> Result<(), String> {
    let security_config = SecurityConfig::default();
    let mut validator = Validator::new(security_config);
    
    // Simulate concurrent validation of same transaction
    for tx in transactions {
        let protocol_tx = Transaction::new(
            tx.hash.clone(),
            tx.from.clone(),
            tx.to.clone(),
            tx.amount,
            Nonce::from(tx.nonce),
        );
        
        let validation_context = ValidationContext::new(
            context.current_time,
            context.block_height,
            context.network_id,
        );
        
        // Multiple concurrent validations
        let result1 = validator.validate_transaction(&protocol_tx, &validation_context);
        let result2 = validator.validate_transaction(&protocol_tx, &validation_context);
        let result3 = validator.validate_transaction(&protocol_tx, &validation_context);
        
        // Results should be consistent
        if let (Ok(r1), Ok(r2), Ok(r3)) = (result1, result2, result3) {
            match (r1, r2, r3) {
                (ValidationResult::Valid, ValidationResult::Valid, ValidationResult::Valid) => {},
                (ValidationResult::Invalid(_), ValidationResult::Invalid(_), ValidationResult::Invalid(_)) => {},
                _ => return Err("Inconsistent validation results".to_string()),
            }
        }
    }
    
    Ok(())
}

/// Test timing consistency for validation operations
fn measure_validation_timing<F>(op: F) -> bool
where
    F: Fn() -> Result<(), String>
{
    let iterations = 20;
    let mut timings = Vec::with_capacity(iterations);
    
    for _ in 0..iterations {
        let start = Instant::now();
        let _ = op();
        timings.push(start.elapsed());
    }
    
    if timings.is_empty() {
        return false;
    }
    
    let mean = timings.iter().sum::<Duration>() / iterations as u32;
    let variance = timings.iter()
        .map(|t| {
            let diff = t.as_nanos() as i128 - mean.as_nanos() as i128;
            diff * diff
        })
        .sum::<i128>() / iterations as i128;
    
    // Allow reasonable variance for validation operations
    variance < 1500000
}

fuzz_target!(|data: &[u8]| {
    // Set panic hook to prevent information leaks
    std::panic::set_hook(Box::new(|_| {}));
    
    if data.is_empty() {
        return;
    }
    
    // Create test validation context
    let context = create_test_validation_context(data);
    
    // Create test transactions
    let mut transactions = Vec::new();
    let max_txs = std::cmp::min(data.len() / 64, 15);
    
    for i in 0..max_txs {
        let start_idx = i * 64;
        let end_idx = std::cmp::min(start_idx + 64, data.len());
        
        if end_idx > start_idx {
            if let Some(tx) = create_test_transaction(&data[start_idx..end_idx], i as u64) {
                transactions.push(tx);
            }
        }
    }
    
    // Create test signature data
    let mut signature_data = Vec::new();
    let max_sigs = std::cmp::min(data.len() / 64, 10);
    
    for i in 0..max_sigs {
        let start_idx = i * 64;
        let end_idx = std::cmp::min(start_idx + 64, data.len());
        
        if end_idx > start_idx {
            if let Some(sig) = create_test_signature_data(&data[start_idx..end_idx]) {
                signature_data.push(sig);
            }
        }
    }
    
    // Test transaction validation with timing
    if !transactions.is_empty() {
        let _tx_timing = measure_validation_timing(|| {
            test_transaction_validation(&transactions, &context)
        });
    }
    
    // Test signature validation with timing
    if !signature_data.is_empty() {
        let _sig_timing = measure_validation_timing(|| {
            test_signature_validation(&signature_data)
        });
    }
    
    // Test message validation
    if data.len() >= 32 {
        let _msg_timing = measure_validation_timing(|| {
            test_message_validation(data)
        });
    }
    
    // Test node validation
    if data.len() >= 16 {
        let _node_timing = measure_validation_timing(|| {
            test_node_validation(data)
        });
    }
    
    // Test fingerprint validation
    let _fp_timing = measure_validation_timing(|| {
        test_fingerprint_validation(data)
    });
    
    // Test validation rules
    if !transactions.is_empty() {
        let _rules_timing = measure_validation_timing(|| {
            test_validation_rules(&transactions, &context)
        });
    }
    
    // Test edge cases
    let _edge_timing = measure_validation_timing(|| {
        test_validation_edge_cases(data)
    });
    
    // Test concurrent validation
    if !transactions.is_empty() {
        let _concurrent_timing = measure_validation_timing(|| {
            test_concurrent_validation(&transactions, &context)
        });
    }
    
    // Test with various malformed inputs
    if data.len() >= 128 {
        test_malformed_validation_inputs(data);
    }
    
    // Test memory management
    test_validation_memory_management(&transactions, &signature_data);
});

/// Test malformed validation inputs
fn test_malformed_validation_inputs(data: &[u8]) {
    let security_config = SecurityConfig::default();
    let mut validator = Validator::new(security_config);
    
    // Test with truncated data
    for i in 1..std::cmp::min(data.len(), 64) {
        let truncated = &data[..i];
        
        // Try to create transaction from truncated data
        let _ = create_test_transaction(truncated, 999);
        
        // Try to create signature data from truncated data
        let _ = create_test_signature_data(truncated);
    }
    
    // Test with bit flipping
    if data.len() >= 32 {
        let mut mutated = data[..32].to_vec();
        for i in 0..mutated.len() {
            mutated[i] ^= 1;
            
            // Test validation with mutated data
            let _ = create_test_transaction(&mutated, 998);
            let _ = create_test_signature_data(&mutated);
            
            mutated[i] ^= 1; // Restore
        }
    }
    
    // Test with extreme values
    let extreme_patterns = vec![
        vec![0x00; 64], // All zeros
        vec![0xFF; 64], // All ones
        (0..64).map(|i| i as u8).collect::<Vec<u8>>(), // Sequential
        (0..64).map(|i| (i as u8) ^ 0xAA).collect::<Vec<u8>>(), // XOR pattern
    ];
    
    for pattern in extreme_patterns {
        let _ = create_test_transaction(&pattern, 997);
        let _ = create_test_signature_data(&pattern);
    }
}

/// Test validation memory management
fn test_validation_memory_management(
    transactions: &[TestTransaction],
    signature_data: &[TestSignatureData]
) {
    // Test with many validators
    let mut validators = Vec::new();
    for _ in 0..10 {
        let security_config = SecurityConfig::default();
        let validator = Validator::new(security_config);
        validators.push(validator);
    }
    
    // Test each validator with transactions
    for validator in &mut validators {
        for tx in transactions {
            let protocol_tx = Transaction::new(
                tx.hash.clone(),
                tx.from.clone(),
                tx.to.clone(),
                tx.amount,
                Nonce::from(tx.nonce),
            );
            
            let context = ValidationContext::new(0, 0, 0);
            let _ = validator.validate_transaction(&protocol_tx, &context);
        }
    }
    
    // Test cleanup
    drop(validators);
    
    // Test signature validation memory management
    let dsa = MlDsa65::new();
    
    // Generate multiple keypairs and signatures
    for sig_data in signature_data {
        if !sig_data.message.is_empty() {
            if let Ok((pk, sk)) = dsa.generate_keypair() {
                if let Ok(signature) = dsa.sign(&sk, &sig_data.message) {
                    let _ = dsa.verify(&pk, &sig_data.message, &signature);
                }
            }
        }
    }
    
    // Memory should be cleaned up automatically
}