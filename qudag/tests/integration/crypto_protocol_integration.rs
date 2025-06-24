//! Integration tests for cryptographic operations within the protocol

use qudag_crypto::{
    ml_kem::{MlKem768, KeyPair as KemKeyPair},
    ml_dsa::{MlDsa65, SigningKey, VerifyingKey},
    hqc::Hqc256,
    fingerprint::Fingerprint,
};
use qudag_protocol::{Coordinator, ProtocolConfig, Message};
use qudag_network::{NetworkManager, P2PMessage};
use tokio::time::{sleep, Duration};
use tracing::{info, error};

#[tokio::test]
async fn test_encrypted_message_exchange() {
    // Create two nodes with ML-KEM encryption
    let config1 = ProtocolConfig {
        network_port: 9001,
        bootstrap_nodes: vec![],
        max_peers: 10,
        validation_timeout: 1000,
    };
    
    let config2 = ProtocolConfig {
        network_port: 9002,
        bootstrap_nodes: vec!["127.0.0.1:9001".to_string()],
        max_peers: 10,
        validation_timeout: 1000,
    };
    
    let mut node1 = Coordinator::new(config1).await.unwrap();
    let mut node2 = Coordinator::new(config2).await.unwrap();
    
    // Start both nodes
    node1.start().await.unwrap();
    node2.start().await.unwrap();
    
    // Allow time for connection
    sleep(Duration::from_millis(500)).await;
    
    // Generate ML-KEM keypairs for both nodes
    let (pk1, sk1) = MlKem768::generate_keypair();
    let (pk2, sk2) = MlKem768::generate_keypair();
    
    // Exchange public keys (simulate key exchange protocol)
    node1.register_peer_key("node2", pk2.clone()).await.unwrap();
    node2.register_peer_key("node1", pk1.clone()).await.unwrap();
    
    // Create encrypted message from node1 to node2
    let plaintext = b"Secret quantum-resistant message";
    let (ciphertext, shared_secret1) = pk2.encapsulate();
    
    // Send encrypted message
    let encrypted_msg = Message::encrypted(plaintext.to_vec(), ciphertext);
    node1.send_encrypted_message("node2", encrypted_msg).await.unwrap();
    
    // Allow time for message delivery
    sleep(Duration::from_millis(200)).await;
    
    // Verify node2 received and decrypted the message
    let received_messages = node2.get_received_messages().await.unwrap();
    assert!(!received_messages.is_empty());
    
    // Decrypt on node2 side
    let shared_secret2 = sk2.decapsulate(&ciphertext).unwrap();
    assert_eq!(shared_secret1, shared_secret2);
    
    // Stop nodes
    node1.stop().await.unwrap();
    node2.stop().await.unwrap();
}

#[tokio::test]
async fn test_signed_message_verification() {
    // Create three nodes for signature verification test
    let mut nodes = Vec::new();
    for i in 0..3 {
        let config = ProtocolConfig {
            network_port: 9100 + i as u16,
            bootstrap_nodes: if i == 0 { 
                vec![] 
            } else { 
                vec![format!("127.0.0.1:{}", 9100)]
            },
            max_peers: 10,
            validation_timeout: 1000,
        };
        
        let node = Coordinator::new(config).await.unwrap();
        nodes.push(node);
    }
    
    // Start all nodes
    for node in nodes.iter_mut() {
        node.start().await.unwrap();
    }
    
    // Allow network formation
    sleep(Duration::from_millis(500)).await;
    
    // Generate ML-DSA signing key for node 0
    let signing_key = MlDsa65::generate_signing_key();
    let verifying_key = signing_key.verifying_key();
    
    // Register verifying key with all nodes
    for node in nodes.iter_mut() {
        node.register_verifying_key("node0", verifying_key.clone()).await.unwrap();
    }
    
    // Create and sign a message
    let message = b"Authenticated protocol message";
    let signature = signing_key.sign(message);
    
    // Broadcast signed message
    let signed_msg = Message::signed(message.to_vec(), signature);
    nodes[0].broadcast_signed_message(signed_msg).await.unwrap();
    
    // Allow propagation
    sleep(Duration::from_millis(300)).await;
    
    // Verify all nodes received and validated the signed message
    for (i, node) in nodes.iter().enumerate() {
        let dag = node.dag_manager().unwrap();
        assert!(dag.contains_signed_message(message), "Node {} didn't receive signed message", i);
        
        // Verify signature validation occurred
        let validation_log = node.get_validation_log().await.unwrap();
        assert!(validation_log.contains(&("node0".to_string(), true)));
    }
    
    // Test invalid signature rejection
    let invalid_signature = vec![0u8; signature.len()];
    let invalid_msg = Message::signed(b"Fake message".to_vec(), invalid_signature);
    
    match nodes[1].broadcast_signed_message(invalid_msg).await {
        Err(e) => info!("Invalid signature correctly rejected: {}", e),
        Ok(_) => panic!("Invalid signature should have been rejected"),
    }
    
    // Stop all nodes
    for node in nodes.iter_mut() {
        node.stop().await.unwrap();
    }
}

#[tokio::test]
async fn test_hqc_encrypted_broadcast() {
    // Test HQC encryption for broadcast messages
    let mut nodes = Vec::new();
    for i in 0..4 {
        let config = ProtocolConfig {
            network_port: 9200 + i as u16,
            bootstrap_nodes: if i == 0 { 
                vec![] 
            } else { 
                vec![format!("127.0.0.1:{}", 9200)]
            },
            max_peers: 10,
            validation_timeout: 1000,
        };
        
        let node = Coordinator::new(config).await.unwrap();
        nodes.push(node);
    }
    
    // Start all nodes
    for node in nodes.iter_mut() {
        node.start().await.unwrap();
    }
    
    sleep(Duration::from_millis(500)).await;
    
    // Generate HQC keypairs for all nodes
    let mut keypairs = Vec::new();
    for _ in 0..4 {
        let (pk, sk) = Hqc256::generate_keypair();
        keypairs.push((pk, sk));
    }
    
    // Register all public keys with all nodes
    for (i, node) in nodes.iter_mut().enumerate() {
        for (j, (pk, _)) in keypairs.iter().enumerate() {
            if i != j {
                node.register_hqc_key(&format!("node{}", j), pk.clone()).await.unwrap();
            }
        }
    }
    
    // Create group encryption key
    let group_message = b"Broadcast to all nodes securely";
    let public_keys: Vec<_> = keypairs.iter().map(|(pk, _)| pk.clone()).collect();
    
    // Encrypt message for the group (simulate group encryption)
    let encrypted_broadcasts = nodes[0].create_group_broadcast(
        group_message.to_vec(),
        public_keys
    ).await.unwrap();
    
    // Broadcast encrypted message
    nodes[0].send_group_broadcast(encrypted_broadcasts).await.unwrap();
    
    // Allow propagation
    sleep(Duration::from_millis(300)).await;
    
    // Verify all nodes can decrypt their portion
    for (i, node) in nodes.iter().enumerate() {
        if i == 0 { continue; } // Skip sender
        
        let decrypted_messages = node.get_decrypted_broadcasts().await.unwrap();
        assert!(
            decrypted_messages.iter().any(|msg| msg == group_message),
            "Node {} couldn't decrypt broadcast", i
        );
    }
    
    // Stop all nodes
    for node in nodes.iter_mut() {
        node.stop().await.unwrap();
    }
}

#[tokio::test]
async fn test_fingerprint_based_routing() {
    // Test fingerprint-based anonymous routing
    let mut nodes = Vec::new();
    for i in 0..5 {
        let config = ProtocolConfig {
            network_port: 9300 + i as u16,
            bootstrap_nodes: if i == 0 { 
                vec![] 
            } else { 
                vec![format!("127.0.0.1:{}", 9300)]
            },
            max_peers: 10,
            validation_timeout: 1000,
        };
        
        let node = Coordinator::new(config).await.unwrap();
        nodes.push(node);
    }
    
    // Start all nodes
    for node in nodes.iter_mut() {
        node.start().await.unwrap();
    }
    
    sleep(Duration::from_millis(500)).await;
    
    // Generate fingerprints for all nodes
    let mut fingerprints = Vec::new();
    for (i, node) in nodes.iter().enumerate() {
        let fp = Fingerprint::from_node_id(&format!("node{}", i));
        fingerprints.push(fp);
        node.set_fingerprint(fp).await.unwrap();
    }
    
    // Test anonymous routing from node 0 to node 4
    let source_fp = fingerprints[0].clone();
    let dest_fp = fingerprints[4].clone();
    
    let anonymous_msg = b"Anonymous routed message";
    nodes[0].send_anonymous_message(
        dest_fp,
        anonymous_msg.to_vec()
    ).await.unwrap();
    
    // Allow routing time
    sleep(Duration::from_millis(500)).await;
    
    // Verify destination received message
    let received = nodes[4].get_anonymous_messages().await.unwrap();
    assert!(
        received.iter().any(|(_, msg)| msg == anonymous_msg),
        "Anonymous message not received at destination"
    );
    
    // Verify intermediate nodes don't have plaintext
    for i in 1..4 {
        let intermediate_msgs = nodes[i].get_anonymous_messages().await.unwrap();
        assert!(
            !intermediate_msgs.iter().any(|(_, msg)| msg == anonymous_msg),
            "Intermediate node {} has access to plaintext", i
        );
    }
    
    // Stop all nodes
    for node in nodes.iter_mut() {
        node.stop().await.unwrap();
    }
}

#[tokio::test]
async fn test_crypto_performance_under_load() {
    // Test cryptographic operations performance
    let config = ProtocolConfig::default();
    let mut coordinator = Coordinator::new(config).await.unwrap();
    coordinator.start().await.unwrap();
    
    // Generate keypairs
    let (kem_pk, kem_sk) = MlKem768::generate_keypair();
    let signing_key = MlDsa65::generate_signing_key();
    let verifying_key = signing_key.verifying_key();
    
    // Measure encryption performance
    let num_operations = 100;
    let message = vec![42u8; 1024]; // 1KB message
    
    let start = std::time::Instant::now();
    for _ in 0..num_operations {
        let (ciphertext, _) = kem_pk.encapsulate();
        let _ = kem_sk.decapsulate(&ciphertext).unwrap();
    }
    let kem_duration = start.elapsed();
    
    info!(
        "ML-KEM operations: {} ops in {:?} ({:.2} ops/sec)",
        num_operations,
        kem_duration,
        num_operations as f64 / kem_duration.as_secs_f64()
    );
    
    // Measure signing performance
    let start = std::time::Instant::now();
    for _ in 0..num_operations {
        let signature = signing_key.sign(&message);
        assert!(verifying_key.verify(&message, &signature).is_ok());
    }
    let dsa_duration = start.elapsed();
    
    info!(
        "ML-DSA operations: {} ops in {:?} ({:.2} ops/sec)",
        num_operations,
        dsa_duration,
        num_operations as f64 / dsa_duration.as_secs_f64()
    );
    
    // Assert performance meets requirements
    assert!(kem_duration < Duration::from_secs(10), "KEM operations too slow");
    assert!(dsa_duration < Duration::from_secs(10), "DSA operations too slow");
    
    coordinator.stop().await.unwrap();
}

#[tokio::test]
async fn test_key_rotation() {
    // Test key rotation during active communication
    let config1 = ProtocolConfig {
        network_port: 9400,
        bootstrap_nodes: vec![],
        max_peers: 10,
        validation_timeout: 1000,
    };
    
    let config2 = ProtocolConfig {
        network_port: 9401,
        bootstrap_nodes: vec!["127.0.0.1:9400".to_string()],
        max_peers: 10,
        validation_timeout: 1000,
    };
    
    let mut node1 = Coordinator::new(config1).await.unwrap();
    let mut node2 = Coordinator::new(config2).await.unwrap();
    
    node1.start().await.unwrap();
    node2.start().await.unwrap();
    
    sleep(Duration::from_millis(300)).await;
    
    // Initial key exchange
    let (pk1_old, sk1_old) = MlKem768::generate_keypair();
    let (pk2_old, sk2_old) = MlKem768::generate_keypair();
    
    node1.register_peer_key("node2", pk2_old.clone()).await.unwrap();
    node2.register_peer_key("node1", pk1_old.clone()).await.unwrap();
    
    // Send some messages with old keys
    for i in 0..5 {
        let msg = format!("Message {} with old key", i);
        node1.send_encrypted_to_peer("node2", msg.as_bytes().to_vec()).await.unwrap();
    }
    
    sleep(Duration::from_millis(200)).await;
    
    // Rotate keys
    let (pk1_new, sk1_new) = MlKem768::generate_keypair();
    let (pk2_new, sk2_new) = MlKem768::generate_keypair();
    
    node1.rotate_keys(sk1_new, pk1_new.clone()).await.unwrap();
    node2.rotate_keys(sk2_new, pk2_new.clone()).await.unwrap();
    
    // Update peer keys
    node1.update_peer_key("node2", pk2_new).await.unwrap();
    node2.update_peer_key("node1", pk1_new).await.unwrap();
    
    // Send messages with new keys
    for i in 0..5 {
        let msg = format!("Message {} with new key", i);
        node1.send_encrypted_to_peer("node2", msg.as_bytes().to_vec()).await.unwrap();
    }
    
    sleep(Duration::from_millis(200)).await;
    
    // Verify all messages were received
    let received = node2.get_received_messages().await.unwrap();
    assert_eq!(received.len(), 10, "Not all messages received after key rotation");
    
    // Stop nodes
    node1.stop().await.unwrap();
    node2.stop().await.unwrap();
}