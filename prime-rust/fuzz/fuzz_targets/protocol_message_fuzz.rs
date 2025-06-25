#![no_main]

use libfuzzer_sys::fuzz_target;
use arbitrary::Arbitrary;
use prime_core::{protocol::*, types::*};
use std::collections::HashMap;

/// Fuzz input for protocol messages
#[derive(Arbitrary, Debug)]
struct FuzzInput {
    sender_id: String,
    recipient_id: Option<String>,
    message_variant: u8,
    data_size: usize,
    corrupt_signature: bool,
}

fuzz_target!(|input: FuzzInput| {
    // Limit data size to prevent excessive memory usage
    if input.data_size > 10000 || input.sender_id.len() > 100 {
        return;
    }

    let sender = NodeId::new(input.sender_id);
    let recipient = input.recipient_id.map(NodeId::new);
    
    // Create different message types based on variant
    let message_type = match input.message_variant % 10 {
        0 => MessageType::Ping,
        1 => MessageType::Pong,
        2 => MessageType::DhtGet { key: vec![0u8; input.data_size.min(1000)] },
        3 => MessageType::DhtPut { 
            key: vec![1u8; input.data_size.min(100)], 
            value: vec![2u8; input.data_size.min(1000)] 
        },
        4 => MessageType::ConsensusProposal { 
            round: input.data_size as u64, 
            value: vec![3u8; input.data_size.min(100)] 
        },
        5 => MessageType::ConsensusVote { 
            round: input.data_size as u64, 
            accept: input.data_size % 2 == 0 
        },
        6 => MessageType::ConsensusCommit { 
            round: input.data_size as u64, 
            value: vec![4u8; input.data_size.min(100)] 
        },
        7 => MessageType::JoinRequest { 
            capabilities: vec!["test".to_string(); input.data_size.min(10)] 
        },
        8 => MessageType::JoinResponse { 
            accepted: input.data_size % 2 == 0 
        },
        _ => {
            // Create a gradient update message
            let mut gradients = HashMap::new();
            for i in 0..input.data_size.min(10) {
                gradients.insert(
                    format!("layer_{}", i),
                    vec![i as f32; input.data_size.min(100)]
                );
            }
            
            MessageType::GradientUpdate(GradientUpdate {
                node_id: sender.clone(),
                model_version: input.data_size as u64,
                round: input.data_size as u64,
                gradients,
                metrics: TrainingMetrics {
                    loss: input.data_size as f32,
                    accuracy: (input.data_size % 100) as f32 / 100.0,
                    samples_processed: input.data_size,
                    computation_time_ms: input.data_size as u64,
                },
                timestamp: input.data_size as u64,
            })
        }
    };
    
    // Create protocol message
    let mut msg = ProtocolMessage::new(sender, message_type);
    
    if let Some(recipient) = recipient {
        msg = msg.with_recipient(recipient);
    }
    
    // Test signing
    if input.corrupt_signature {
        msg.sign(&[0u8; 32]);
        // Corrupt the signature
        if let Some(ref mut sig) = msg.signature {
            if !sig.is_empty() {
                sig[0] = sig[0].wrapping_add(1);
            }
        }
    } else {
        msg.sign(&[1u8; 32]);
    }
    
    // Test serialization and deserialization
    if let Ok(serialized) = serde_json::to_string(&msg) {
        // Don't panic on deserialization errors, just continue
        let _: Result<ProtocolMessage, _> = serde_json::from_str(&serialized);
    }
    
    // Test verification
    let _verify_result = msg.verify(&[1u8; 32]);
});