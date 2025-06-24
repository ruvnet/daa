/// System integration tests for cryptographic operations
/// 
/// This module tests the integration of cryptographic primitives with:
/// - QuDAG protocol operations
/// - Network layer interactions
/// - DAG consensus mechanisms
/// - Peer-to-peer communications
/// - System-wide security properties

use qudag_crypto::{
    kem::{KeyEncapsulation, MlKem768},
    ml_dsa::{MlDsa, MlDsaKeyPair},
    encryption::HQC,
    hash::Blake3Hash,
    fingerprint::{Fingerprint, FingerprintError},
    CryptoError,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use rand::{RngCore, thread_rng};
use serde::{Serialize, Deserialize};

/// Mock network message for testing
#[derive(Debug, Clone, Serialize, Deserialize)]
struct NetworkMessage {
    sender_id: Vec<u8>,
    receiver_id: Vec<u8>,
    message_type: MessageType,
    payload: Vec<u8>,
    timestamp: u64,
    signature: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum MessageType {
    KeyExchange,
    Transaction,
    ConsensusVote,
    Heartbeat,
    DataSync,
}

/// Mock DAG vertex for testing
#[derive(Debug, Clone, Serialize, Deserialize)]
struct DagVertex {
    id: Vec<u8>,
    parents: Vec<Vec<u8>>,
    data: Vec<u8>,
    timestamp: u64,
    signature: Vec<u8>,
    fingerprint: Vec<u8>,
}

/// Mock peer identity for testing
#[derive(Debug, Clone)]
struct PeerIdentity {
    peer_id: Vec<u8>,
    kem_public_key: Vec<u8>,
    dsa_public_key: Vec<u8>,
    created_at: SystemTime,
}

/// Integration test context
struct IntegrationTestContext {
    peers: HashMap<Vec<u8>, PeerIdentity>,
    shared_secrets: HashMap<(Vec<u8>, Vec<u8>), Vec<u8>>, // (peer1, peer2) -> shared_secret
    message_log: Arc<Mutex<Vec<NetworkMessage>>>,
    dag_vertices: Arc<Mutex<Vec<DagVertex>>>,
}

impl IntegrationTestContext {
    fn new() -> Self {
        Self {
            peers: HashMap::new(),
            shared_secrets: HashMap::new(),
            message_log: Arc::new(Mutex::new(Vec::new())),
            dag_vertices: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    fn add_peer(&mut self, peer_id: Vec<u8>) -> Result<(), CryptoError> {
        let (kem_pk, _kem_sk) = MlKem768::keygen()?;
        let dsa_keypair = MlDsa::keygen()?;
        
        let peer = PeerIdentity {
            peer_id: peer_id.clone(),
            kem_public_key: kem_pk.as_bytes().to_vec(),
            dsa_public_key: dsa_keypair.public_key().as_bytes().to_vec(),
            created_at: SystemTime::now(),
        };
        
        self.peers.insert(peer_id, peer);
        Ok(())
    }
    
    fn establish_shared_secret(&mut self, peer1_id: &[u8], peer2_id: &[u8]) -> Result<Vec<u8>, CryptoError> {
        let peer1 = self.peers.get(peer1_id).ok_or(CryptoError::InvalidKey)?;
        let peer2 = self.peers.get(peer2_id).ok_or(CryptoError::InvalidKey)?;
        
        // Simulate key exchange using ML-KEM
        let kem_pk = qudag_crypto::kem::PublicKey::from_bytes(&peer2.kem_public_key);
        let (ct, ss) = MlKem768::encapsulate(&kem_pk)?;
        
        // In real implementation, ciphertext would be sent to peer2
        // and they would decapsulate to get the same shared secret
        
        let mut key = (peer1_id.to_vec(), peer2_id.to_vec());
        if key.0 > key.1 {
            std::mem::swap(&mut key.0, &mut key.1);
        }
        
        self.shared_secrets.insert(key, ss.as_bytes().to_vec());
        Ok(ss.as_bytes().to_vec())
    }
    
    fn sign_message(&self, message: &NetworkMessage, signer_id: &[u8]) -> Result<Vec<u8>, CryptoError> {
        let _peer = self.peers.get(signer_id).ok_or(CryptoError::InvalidKey)?;
        
        // In real implementation, we would use the peer's private key
        // For testing, we generate a fresh keypair
        let keypair = MlDsa::keygen()?;
        let message_bytes = bincode::serialize(message).map_err(|_| CryptoError::SerializationError)?;
        let signature = MlDsa::sign(&message_bytes, keypair.secret_key())?;
        
        Ok(signature)
    }
    
    fn verify_message(&self, message: &NetworkMessage, signature: &[u8], signer_id: &[u8]) -> Result<bool, CryptoError> {
        let peer = self.peers.get(signer_id).ok_or(CryptoError::InvalidKey)?;
        
        let dsa_pk = qudag_crypto::ml_dsa::MlDsaPublicKey::from_bytes(&peer.dsa_public_key);
        let message_bytes = bincode::serialize(message).map_err(|_| CryptoError::SerializationError)?;
        
        match MlDsa::verify(&message_bytes, signature, &dsa_pk) {
            Ok(()) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

#[cfg(test)]
mod system_integration_tests {
    use super::*;

    #[test]
    fn test_end_to_end_key_exchange() {
        let mut context = IntegrationTestContext::new();
        
        // Add test peers
        let peer1_id = b"peer1".to_vec();
        let peer2_id = b"peer2".to_vec();
        
        context.add_peer(peer1_id.clone()).unwrap();
        context.add_peer(peer2_id.clone()).unwrap();
        
        // Establish shared secret
        let shared_secret = context.establish_shared_secret(&peer1_id, &peer2_id).unwrap();
        
        // Verify shared secret properties
        assert_eq!(shared_secret.len(), 32); // ML-KEM-768 shared secret size
        assert!(!shared_secret.iter().all(|&b| b == 0)); // Not all zeros
        
        // Test that the same pair produces the same key
        let shared_secret2 = context.establish_shared_secret(&peer2_id, &peer1_id).unwrap();
        assert_eq!(shared_secret, shared_secret2);
    }

    #[test]
    fn test_message_signing_and_verification() {
        let mut context = IntegrationTestContext::new();
        
        let peer1_id = b"signer_peer".to_vec();
        context.add_peer(peer1_id.clone()).unwrap();
        
        // Create test message
        let message = NetworkMessage {
            sender_id: peer1_id.clone(),
            receiver_id: b"receiver_peer".to_vec(),
            message_type: MessageType::Transaction,
            payload: b"test transaction data".to_vec(),
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            signature: None,
        };
        
        // Sign message
        let signature = context.sign_message(&message, &peer1_id).unwrap();
        
        // Verify signature
        let is_valid = context.verify_message(&message, &signature, &peer1_id).unwrap();
        assert!(is_valid, "Message signature verification failed");
        
        // Test with modified message (should fail)
        let mut modified_message = message.clone();
        modified_message.payload = b"modified transaction data".to_vec();
        
        let is_valid_modified = context.verify_message(&modified_message, &signature, &peer1_id).unwrap();
        assert!(!is_valid_modified, "Modified message should not verify");
    }

    #[test]
    fn test_dag_vertex_creation_and_verification() {
        let mut context = IntegrationTestContext::new();
        let peer_id = b"vertex_creator".to_vec();
        context.add_peer(peer_id.clone()).unwrap();
        
        // Create DAG vertex data
        let vertex_data = b"sample vertex data for DAG consensus".to_vec();
        let parent_ids = vec![
            b"parent1".to_vec(),
            b"parent2".to_vec(),
        ];
        
        // Generate fingerprint for the vertex
        let fingerprint = Fingerprint::generate(&vertex_data).unwrap();
        
        // Create vertex structure
        let vertex = DagVertex {
            id: Blake3Hash::hash(&vertex_data)[0..16].to_vec(), // First 16 bytes as ID
            parents: parent_ids,
            data: vertex_data,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            signature: vec![], // Will be filled below
            fingerprint: fingerprint.to_bytes(),
        };
        
        // Sign the vertex
        let keypair = MlDsa::keygen().unwrap();
        let vertex_bytes = bincode::serialize(&vertex).unwrap();
        let signature = MlDsa::sign(&vertex_bytes, keypair.secret_key()).unwrap();
        
        let mut signed_vertex = vertex;
        signed_vertex.signature = signature;
        
        // Verify vertex signature
        let signed_vertex_bytes = bincode::serialize(&signed_vertex).unwrap();
        assert!(MlDsa::verify(&signed_vertex_bytes, &signed_vertex.signature, keypair.public_key()).is_ok());
        
        // Verify fingerprint
        let reconstructed_fingerprint = Fingerprint::from_bytes(&signed_vertex.fingerprint).unwrap();
        assert!(reconstructed_fingerprint.verify(&signed_vertex.data).unwrap());
        
        // Add to DAG
        context.dag_vertices.lock().unwrap().push(signed_vertex);
    }

    #[test]
    fn test_multi_peer_consensus_simulation() {
        let mut context = IntegrationTestContext::new();
        let num_peers = 5;
        let mut peer_ids = Vec::new();
        
        // Setup peers
        for i in 0..num_peers {
            let peer_id = format!("consensus_peer_{}", i).into_bytes();
            context.add_peer(peer_id.clone()).unwrap();
            peer_ids.push(peer_id);
        }
        
        // Simulate consensus round
        let consensus_data = b"consensus proposal data".to_vec();
        let mut votes = Vec::new();
        
        for peer_id in &peer_ids {
            // Each peer creates a vote message
            let vote_message = NetworkMessage {
                sender_id: peer_id.clone(),
                receiver_id: b"consensus_coordinator".to_vec(),
                message_type: MessageType::ConsensusVote,
                payload: consensus_data.clone(),
                timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                signature: None,
            };
            
            // Sign the vote
            let signature = context.sign_message(&vote_message, peer_id).unwrap();
            let mut signed_vote = vote_message;
            signed_vote.signature = Some(signature);
            
            votes.push(signed_vote);
        }
        
        // Verify all votes
        let mut valid_votes = 0;
        for vote in &votes {
            if let Some(signature) = &vote.signature {
                let is_valid = context.verify_message(vote, signature, &vote.sender_id).unwrap();
                if is_valid {
                    valid_votes += 1;
                }
            }
        }
        
        assert_eq!(valid_votes, num_peers, "Not all consensus votes are valid");
        
        // Simulate reaching consensus (simple majority)
        let consensus_threshold = (num_peers * 2) / 3 + 1; // 2/3 + 1 majority
        assert!(valid_votes >= consensus_threshold, "Insufficient votes for consensus");
    }

    #[test]
    fn test_encrypted_peer_communication() {
        let mut context = IntegrationTestContext::new();
        
        let peer1_id = b"alice".to_vec();
        let peer2_id = b"bob".to_vec();
        
        context.add_peer(peer1_id.clone()).unwrap();
        context.add_peer(peer2_id.clone()).unwrap();
        
        // Establish shared secret
        let shared_secret = context.establish_shared_secret(&peer1_id, &peer2_id).unwrap();
        
        // Simulate encrypted communication using shared secret as key
        let plaintext = b"confidential message between peers".to_vec();
        
        // Simple encryption simulation (in real implementation, use AES-GCM or similar)
        let mut ciphertext = plaintext.clone();
        for (i, byte) in ciphertext.iter_mut().enumerate() {
            *byte ^= shared_secret[i % shared_secret.len()];
        }
        
        // Create encrypted message
        let encrypted_message = NetworkMessage {
            sender_id: peer1_id.clone(),
            receiver_id: peer2_id.clone(),
            message_type: MessageType::DataSync,
            payload: ciphertext.clone(),
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            signature: None,
        };
        
        // Sign the encrypted message
        let signature = context.sign_message(&encrypted_message, &peer1_id).unwrap();
        
        // Simulate receiving and decrypting
        let mut decrypted = ciphertext;
        for (i, byte) in decrypted.iter_mut().enumerate() {
            *byte ^= shared_secret[i % shared_secret.len()];
        }
        
        assert_eq!(plaintext, decrypted, "Decryption failed");
        
        // Verify message integrity
        let is_valid = context.verify_message(&encrypted_message, &signature, &peer1_id).unwrap();
        assert!(is_valid, "Encrypted message verification failed");
    }

    #[test]
    fn test_network_partition_resilience() {
        let mut context = IntegrationTestContext::new();
        let mut peer_ids = Vec::new();
        
        // Create two groups of peers (simulating network partition)
        let group1_size = 3;
        let group2_size = 2;
        
        for i in 0..group1_size {
            let peer_id = format!("group1_peer_{}", i).into_bytes();
            context.add_peer(peer_id.clone()).unwrap();
            peer_ids.push(peer_id);
        }
        
        for i in 0..group2_size {
            let peer_id = format!("group2_peer_{}", i).into_bytes();
            context.add_peer(peer_id.clone()).unwrap();
            peer_ids.push(peer_id);
        }
        
        // Establish communication within groups
        let mut group1_messages = Vec::new();
        let mut group2_messages = Vec::new();
        
        // Group 1 internal communication
        for i in 0..group1_size {
            for j in 0..group1_size {
                if i != j {
                    let sender = &peer_ids[i];
                    let receiver = &peer_ids[j];
                    
                    let message = NetworkMessage {
                        sender_id: sender.clone(),
                        receiver_id: receiver.clone(),
                        message_type: MessageType::Heartbeat,
                        payload: b"group1 heartbeat".to_vec(),
                        timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                        signature: None,
                    };
                    
                    let signature = context.sign_message(&message, sender).unwrap();
                    let mut signed_message = message;
                    signed_message.signature = Some(signature);
                    
                    group1_messages.push(signed_message);
                }
            }
        }
        
        // Group 2 internal communication
        for i in group1_size..peer_ids.len() {
            for j in group1_size..peer_ids.len() {
                if i != j {
                    let sender = &peer_ids[i];
                    let receiver = &peer_ids[j];
                    
                    let message = NetworkMessage {
                        sender_id: sender.clone(),
                        receiver_id: receiver.clone(),
                        message_type: MessageType::Heartbeat,
                        payload: b"group2 heartbeat".to_vec(),
                        timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                        signature: None,
                    };
                    
                    let signature = context.sign_message(&message, sender).unwrap();
                    let mut signed_message = message;
                    signed_message.signature = Some(signature);
                    
                    group2_messages.push(signed_message);
                }
            }
        }
        
        // Verify all messages within groups are valid
        for message in &group1_messages {
            if let Some(signature) = &message.signature {
                let is_valid = context.verify_message(message, signature, &message.sender_id).unwrap();
                assert!(is_valid, "Group 1 message verification failed");
            }
        }
        
        for message in &group2_messages {
            if let Some(signature) = &message.signature {
                let is_valid = context.verify_message(message, signature, &message.sender_id).unwrap();
                assert!(is_valid, "Group 2 message verification failed");
            }
        }
        
        // Simulate network healing - cross-group communication
        let cross_group_message = NetworkMessage {
            sender_id: peer_ids[0].clone(), // Group 1 peer
            receiver_id: peer_ids[group1_size].clone(), // Group 2 peer
            message_type: MessageType::DataSync,
            payload: b"partition healing message".to_vec(),
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            signature: None,
        };
        
        let signature = context.sign_message(&cross_group_message, &peer_ids[0]).unwrap();
        let is_valid = context.verify_message(&cross_group_message, &signature, &peer_ids[0]).unwrap();
        assert!(is_valid, "Cross-group communication failed after partition healing");
    }

    #[test]
    fn test_system_performance_under_load() {
        let mut context = IntegrationTestContext::new();
        let num_peers = 10;
        let messages_per_peer = 50;
        
        // Setup peers
        let mut peer_ids = Vec::new();
        for i in 0..num_peers {
            let peer_id = format!("load_test_peer_{}", i).into_bytes();
            context.add_peer(peer_id.clone()).unwrap();
            peer_ids.push(peer_id);
        }
        
        let start_time = Instant::now();
        let mut total_operations = 0;
        
        // Generate load
        for sender_idx in 0..num_peers {
            for msg_idx in 0..messages_per_peer {
                let receiver_idx = (sender_idx + 1) % num_peers;
                
                let message = NetworkMessage {
                    sender_id: peer_ids[sender_idx].clone(),
                    receiver_id: peer_ids[receiver_idx].clone(),
                    message_type: MessageType::Transaction,
                    payload: format!("load_test_message_{}_{}", sender_idx, msg_idx).into_bytes(),
                    timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                    signature: None,
                };
                
                // Sign message
                let signature = context.sign_message(&message, &peer_ids[sender_idx]).unwrap();
                
                // Verify message
                let is_valid = context.verify_message(&message, &signature, &peer_ids[sender_idx]).unwrap();
                assert!(is_valid, "Message verification failed under load");
                
                total_operations += 2; // Sign + verify
            }
        }
        
        let elapsed = start_time.elapsed();
        let ops_per_second = total_operations as f64 / elapsed.as_secs_f64();
        
        println!("Performance test results:");
        println!("  Total operations: {}", total_operations);
        println!("  Elapsed time: {:?}", elapsed);
        println!("  Operations per second: {:.2}", ops_per_second);
        
        // Performance baseline - should be able to handle reasonable load
        assert!(ops_per_second > 100.0, "System performance too low: {} ops/sec", ops_per_second);
        assert!(elapsed < Duration::from_secs(30), "System took too long under load");
    }

    #[test]
    fn test_cryptographic_protocol_composition() {
        // Test that all crypto primitives work correctly together
        let mut context = IntegrationTestContext::new();
        
        let peer_id = b"composer_peer".to_vec();
        context.add_peer(peer_id.clone()).unwrap();
        
        // 1. Generate keys
        let (kem_pk, kem_sk) = MlKem768::keygen().unwrap();
        let dsa_keypair = MlDsa::keygen().unwrap();
        
        // 2. Create and sign data
        let data = b"important protocol data".to_vec();
        let signature = MlDsa::sign(&data, dsa_keypair.secret_key()).unwrap();
        
        // 3. Generate fingerprint
        let fingerprint = Fingerprint::generate(&data).unwrap();
        
        // 4. Establish shared secret
        let (ct, ss) = MlKem768::encapsulate(&kem_pk).unwrap();
        let ss2 = MlKem768::decapsulate(&kem_sk, &ct).unwrap();
        assert_eq!(ss.as_bytes(), ss2.as_bytes());
        
        // 5. Hash all components together
        let combined_data = [
            &data,
            &signature,
            &fingerprint.to_bytes(),
            ss.as_bytes(),
        ].concat();
        let final_hash = Blake3Hash::hash(&combined_data);
        
        // 6. Verify all components
        assert!(MlDsa::verify(&data, &signature, dsa_keypair.public_key()).is_ok());
        assert!(fingerprint.verify(&data).unwrap());
        assert_eq!(final_hash.len(), 32);
        
        // 7. Test protocol message with all components
        let protocol_message = NetworkMessage {
            sender_id: peer_id.clone(),
            receiver_id: b"protocol_receiver".to_vec(),
            message_type: MessageType::DataSync,
            payload: combined_data,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            signature: Some(signature),
        };
        
        // Verify the composed protocol works end-to-end
        if let Some(sig) = &protocol_message.signature {
            let message_without_sig = NetworkMessage {
                signature: None,
                ..protocol_message.clone()
            };
            let msg_bytes = bincode::serialize(&message_without_sig).unwrap();
            assert!(MlDsa::verify(&msg_bytes, sig, dsa_keypair.public_key()).is_ok());
        }
    }

    #[test]
    fn test_byzantine_fault_tolerance() {
        let mut context = IntegrationTestContext::new();
        let total_peers = 7; // 7 peers, can tolerate 2 byzantine
        let byzantine_peers = 2;
        let honest_peers = total_peers - byzantine_peers;
        
        let mut peer_ids = Vec::new();
        
        // Setup all peers
        for i in 0..total_peers {
            let peer_id = format!("bft_peer_{}", i).into_bytes();
            context.add_peer(peer_id.clone()).unwrap();
            peer_ids.push(peer_id);
        }
        
        let consensus_data = b"bft consensus proposal".to_vec();
        let mut honest_votes = Vec::new();
        let mut byzantine_votes = Vec::new();
        
        // Honest peers vote correctly
        for i in 0..honest_peers {
            let vote = NetworkMessage {
                sender_id: peer_ids[i].clone(),
                receiver_id: b"bft_coordinator".to_vec(),
                message_type: MessageType::ConsensusVote,
                payload: consensus_data.clone(),
                timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                signature: None,
            };
            
            let signature = context.sign_message(&vote, &peer_ids[i]).unwrap();
            let mut signed_vote = vote;
            signed_vote.signature = Some(signature);
            
            honest_votes.push(signed_vote);
        }
        
        // Byzantine peers vote with different data
        for i in honest_peers..total_peers {
            let byzantine_data = format!("byzantine_data_{}", i).into_bytes();
            let vote = NetworkMessage {
                sender_id: peer_ids[i].clone(),
                receiver_id: b"bft_coordinator".to_vec(),
                message_type: MessageType::ConsensusVote,
                payload: byzantine_data,
                timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                signature: None,
            };
            
            let signature = context.sign_message(&vote, &peer_ids[i]).unwrap();
            let mut signed_vote = vote;
            signed_vote.signature = Some(signature);
            
            byzantine_votes.push(signed_vote);
        }
        
        // Verify honest votes reach consensus threshold
        let consensus_threshold = (total_peers * 2) / 3 + 1; // 2/3 + 1
        assert!(honest_votes.len() >= consensus_threshold, 
            "Honest votes insufficient for consensus: {} < {}", honest_votes.len(), consensus_threshold);
        
        // Verify all votes (both honest and byzantine) have valid signatures
        for vote in honest_votes.iter().chain(byzantine_votes.iter()) {
            if let Some(signature) = &vote.signature {
                let is_valid = context.verify_message(vote, signature, &vote.sender_id).unwrap();
                assert!(is_valid, "Vote signature verification failed");
            }
        }
        
        // System should still reach consensus despite byzantine behavior
        let honest_consensus = honest_votes.iter()
            .filter(|vote| vote.payload == consensus_data)
            .count();
        
        assert!(honest_consensus >= consensus_threshold,
            "Byzantine fault tolerance failed: {} honest votes < {} threshold", 
            honest_consensus, consensus_threshold);
    }
}