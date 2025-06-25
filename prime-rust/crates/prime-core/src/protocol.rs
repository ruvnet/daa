//! Protocol definitions for Prime distributed ML

use crate::types::*;
use serde::{Deserialize, Serialize};

/// Protocol version
pub const PROTOCOL_VERSION: u32 = 1;

/// Protocol message wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolMessage {
    pub version: u32,
    pub message_id: String,
    pub sender: NodeId,
    pub recipient: Option<NodeId>,
    pub message_type: MessageType,
    pub timestamp: u64,
    pub signature: Option<Vec<u8>>,
}

impl ProtocolMessage {
    pub fn new(sender: NodeId, message_type: MessageType) -> Self {
        Self {
            version: PROTOCOL_VERSION,
            message_id: generate_message_id(),
            sender,
            recipient: None,
            message_type,
            timestamp: current_timestamp(),
            signature: None,
        }
    }

    pub fn with_recipient(mut self, recipient: NodeId) -> Self {
        self.recipient = Some(recipient);
        self
    }

    pub fn sign(&mut self, _private_key: &[u8]) {
        // TODO: Implement actual signing
        self.signature = Some(vec![0; 64]);
    }

    pub fn verify(&self, _public_key: &[u8]) -> bool {
        // TODO: Implement actual verification
        self.signature.is_some()
    }
}

fn generate_message_id() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    format!("msg_{:016x}", rng.gen::<u64>())
}

fn current_timestamp() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// Protocol handler trait
#[async_trait::async_trait]
pub trait ProtocolHandler: Send + Sync {
    async fn handle_message(&self, message: ProtocolMessage) -> crate::Result<Option<ProtocolMessage>>;
    async fn validate_message(&self, message: &ProtocolMessage) -> crate::Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use quickcheck::{Arbitrary, Gen, QuickCheck};

    #[test]
    fn test_protocol_message_creation() {
        let sender = NodeId::new("node1");
        let msg = ProtocolMessage::new(sender.clone(), MessageType::Ping);
        
        assert_eq!(msg.version, PROTOCOL_VERSION);
        assert_eq!(msg.sender, sender);
        assert!(msg.recipient.is_none());
        assert!(msg.message_id.starts_with("msg_"));
    }

    #[test]
    fn test_message_with_recipient() {
        let sender = NodeId::new("node1");
        let recipient = NodeId::new("node2");
        let msg = ProtocolMessage::new(sender, MessageType::Ping)
            .with_recipient(recipient.clone());
        
        assert_eq!(msg.recipient, Some(recipient));
    }

    #[test]
    fn test_message_signing() {
        let sender = NodeId::new("node1");
        let mut msg = ProtocolMessage::new(sender, MessageType::Ping);
        
        assert!(msg.signature.is_none());
        msg.sign(&[]);
        assert!(msg.signature.is_some());
        assert!(msg.verify(&[]));
    }

    // QuickCheck arbitrary implementation for protocol testing
    impl Arbitrary for ProtocolMessage {
        fn arbitrary(g: &mut Gen) -> Self {
            let sender = NodeId::new(format!("node_{}", u32::arbitrary(g)));
            let msg_types = vec![
                MessageType::Ping,
                MessageType::Pong,
                MessageType::DhtGet { key: vec![1, 2, 3] },
            ];
            let msg_type = g.choose(&msg_types).unwrap().clone();
            
            ProtocolMessage::new(sender, msg_type)
        }
    }

    #[quickcheck]
    fn test_protocol_message_roundtrip(msg: ProtocolMessage) -> bool {
        let serialized = serde_json::to_string(&msg).unwrap();
        let deserialized: ProtocolMessage = serde_json::from_str(&serialized).unwrap();
        
        msg.message_id == deserialized.message_id &&
        msg.sender == deserialized.sender &&
        msg.version == deserialized.version
    }

    proptest! {
        #[test]
        fn test_message_id_uniqueness(
            count in 10..100
        ) {
            let mut ids = std::collections::HashSet::new();
            
            for _ in 0..count {
                let id = generate_message_id();
                assert!(ids.insert(id), "Duplicate message ID generated");
            }
        }

        #[test]
        fn test_timestamp_monotonic(
            delays in prop::collection::vec(0u64..100u64, 5..10)
        ) {
            let mut timestamps = Vec::new();
            
            for delay in delays {
                std::thread::sleep(std::time::Duration::from_millis(delay));
                timestamps.push(current_timestamp());
            }
            
            // Check timestamps are non-decreasing
            for window in timestamps.windows(2) {
                assert!(window[0] <= window[1]);
            }
        }
    }
}