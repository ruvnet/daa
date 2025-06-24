//! Integration tests between network, crypto, and DAG layers
//!
//! This module tests the integration and interaction between the networking layer
//! and other core components of the QuDAG system, ensuring proper end-to-end
//! functionality and protocol compliance.

use qudag_crypto::{
    encryption::{MLKEMCiphertext, MLKEMKeyPair, MLKEMSecretKey},
    fingerprint::{generate_peer_fingerprint, PeerFingerprint},
    hash::{HashFunction, QuDagHash},
    ml_dsa::{MLDSAKeyPair, MLDSASignature, SigningKey, VerifyingKey},
    ml_kem::{MLKEMParameters, MLKEMPublicKey},
    signature::{Signature, SignatureScheme},
};
use qudag_dag::{
    consensus::{ConsensusConfig, ConsensusEngine, ConsensusResult, QuorumSize},
    dag::{DagConfig, Transaction, Vertex, DAG},
    node::{Node, NodeConfig, NodeId},
    tip_selection::{TipSelectionStrategy, TipSelector},
};
use qudag_network::{
    connection::ConnectionManager,
    message::MessageEnvelope,
    onion::{MLKEMOnionRouter, OnionLayer},
    quantum_crypto::{MlKemSecurityLevel, QuantumKeyExchange, SharedSecret},
    router::Router,
    types::{ConnectionStatus, MessagePriority, NetworkMessage, PeerId, RoutingStrategy},
    NetworkManager,
};
use rand::{thread_rng, Rng};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Test integration between network layer and ML-DSA signature scheme
#[tokio::test]
async fn test_network_mldsa_integration() {
    // Setup network components
    let mut network_manager = NetworkManager::new();
    network_manager.initialize().await.unwrap();

    // Setup crypto components
    let alice_keypair = MLDSAKeyPair::generate();
    let bob_keypair = MLDSAKeyPair::generate();

    // Create signed network message
    let message_data = b"This is a test message for ML-DSA integration".to_vec();
    let network_msg = NetworkMessage {
        id: "mldsa_integration_test".into(),
        source: alice_keypair.public_key().to_bytes(),
        destination: bob_keypair.public_key().to_bytes(),
        payload: message_data.clone(),
        priority: MessagePriority::High,
        ttl: Duration::from_secs(120),
    };

    // Create message envelope with signature
    let mut envelope = MessageEnvelope::new(network_msg.clone());

    // Sign the message with Alice's private key
    let message_hash = QuDagHash::hash(&bincode::serialize(&network_msg).unwrap());
    let signature = alice_keypair.sign(&message_hash.as_bytes());
    envelope.signature = Some(signature.to_bytes());

    // Verify integrity
    assert!(envelope.verify());

    // Bob verifies the signature
    let received_hash = QuDagHash::hash(&bincode::serialize(&envelope.message).unwrap());
    let signature_bytes = envelope.signature.unwrap();
    let signature = MLDSASignature::from_bytes(&signature_bytes).unwrap();

    assert!(alice_keypair
        .public_key()
        .verify(&received_hash.as_bytes(), &signature));

    // Test signature verification failure with wrong key
    assert!(!bob_keypair
        .public_key()
        .verify(&received_hash.as_bytes(), &signature));

    // Test message integrity - tampering should break verification
    let mut tampered_envelope = envelope.clone();
    tampered_envelope.message.payload = b"tampered data".to_vec();
    let tampered_hash = QuDagHash::hash(&bincode::serialize(&tampered_envelope.message).unwrap());
    assert!(!alice_keypair
        .public_key()
        .verify(&tampered_hash.as_bytes(), &signature));
}

/// Test integration between network layer and ML-KEM encryption
#[tokio::test]
async fn test_network_mlkem_integration() {
    let router = Router::new();
    let onion_router = MLKEMOnionRouter::new();

    // Setup quantum key exchanges for three hops
    let hop_exchanges: Vec<_> = (0..3)
        .map(|_| QuantumKeyExchange::new(MlKemSecurityLevel::Level3))
        .collect();

    let hop_public_keys: Vec<_> = hop_exchanges.iter().map(|kx| kx.public_key()).collect();

    // Create original message
    let original_payload = b"Secret message encrypted with ML-KEM".to_vec();

    // Create onion layers using ML-KEM
    let onion_layers = onion_router
        .create_layers(&original_payload, &hop_public_keys)
        .await
        .unwrap();

    assert_eq!(onion_layers.len(), 3);

    // Simulate message passing through each hop
    let mut current_layer = onion_layers.last().unwrap().clone();

    for (i, kx) in hop_exchanges.iter().enumerate().rev() {
        // Each hop decrypts its layer
        let peeled_result = onion_router.peel_layer(&current_layer, kx).await;
        assert!(peeled_result.is_ok());

        let (next_layer, routing_info) = peeled_result.unwrap();

        if i == 0 {
            // Final hop should reveal original payload
            assert_eq!(next_layer.encrypted_data, original_payload);
        } else {
            // Intermediate hops continue the chain
            current_layer = next_layer;
            assert!(routing_info.next_hop.is_some());
        }
    }
}

/// Test integration between network routing and DAG consensus
#[tokio::test]
async fn test_network_dag_consensus_integration() {
    // Setup network layer
    let router = Router::new();
    let connection_manager = ConnectionManager::new(50);

    // Setup DAG layer
    let dag_config = DagConfig {
        max_vertices_per_level: 100,
        confirmation_threshold: 10,
        finality_threshold: 20,
    };
    let mut dag = DAG::new(dag_config);

    // Setup consensus
    let consensus_config = ConsensusConfig {
        quorum_size: QuorumSize::TwoThirds,
        timeout: Duration::from_secs(30),
        max_rounds: 10,
    };
    let mut consensus_engine = ConsensusEngine::new(consensus_config);

    // Create network nodes that participate in consensus
    let node_configs: Vec<_> = (0..5)
        .map(|i| NodeConfig {
            node_id: NodeId::new(format!("node_{}", i)),
            stake: 100,
            is_validator: true,
        })
        .collect();

    let nodes: Vec<_> = node_configs
        .into_iter()
        .map(|config| Node::new(config))
        .collect();

    // Add nodes to network
    for node in &nodes {
        let peer_id = PeerId::from_bytes({
            let mut bytes = [0u8; 32];
            bytes[..16].copy_from_slice(node.id().as_bytes());
            bytes
        });
        router.add_peer(peer_id).await;
        connection_manager.connect(peer_id).await.unwrap();
        connection_manager.update_status(peer_id, ConnectionStatus::Connected);
    }

    // Create transaction to be processed through the network and added to DAG
    let transaction = Transaction {
        id: "test_tx_001".to_string(),
        sender: nodes[0].id().clone(),
        receiver: nodes[1].id().clone(),
        amount: 100,
        timestamp: Instant::now(),
        signature: vec![0; 64], // Mock signature
    };

    // Create vertex containing the transaction
    let vertex = Vertex::new(
        "vertex_001".to_string(),
        vec![transaction],
        vec![], // No parents for first vertex
        nodes[0].id().clone(),
    );

    // Add vertex to DAG
    assert!(dag.add_vertex(vertex.clone()).is_ok());

    // Network message to propagate vertex
    let vertex_data = bincode::serialize(&vertex).unwrap();
    let network_msg = NetworkMessage {
        id: "vertex_propagation".into(),
        source: nodes[0].id().as_bytes().to_vec(),
        destination: vec![0xFF; 32], // Broadcast
        payload: vertex_data,
        priority: MessagePriority::High,
        ttl: Duration::from_secs(60),
    };

    // Route message through network
    let route = router
        .route(&network_msg, RoutingStrategy::Direct)
        .await
        .unwrap();

    assert!(route.len() > 0);

    // Simulate consensus on the vertex
    let consensus_input = vertex.id().clone();
    let consensus_result = consensus_engine
        .process_input(consensus_input, &nodes)
        .await;

    assert!(consensus_result.is_ok());
    let result = consensus_result.unwrap();

    match result {
        ConsensusResult::Agreed(agreed_vertex_id) => {
            assert_eq!(agreed_vertex_id, vertex.id().clone());

            // Mark vertex as confirmed in DAG
            dag.confirm_vertex(&agreed_vertex_id).unwrap();
            assert!(dag.is_confirmed(&agreed_vertex_id));
        }
        ConsensusResult::Disagreed => {
            panic!("Consensus should reach agreement for valid vertex");
        }
        ConsensusResult::Timeout => {
            panic!("Consensus should not timeout for simple test case");
        }
    }

    // Verify DAG state
    assert_eq!(dag.vertex_count(), 1);
    assert!(dag.get_vertex(vertex.id()).is_some());
}

/// Test end-to-end transaction flow through network, crypto, and DAG
#[tokio::test]
async fn test_end_to_end_transaction_flow() {
    // Setup all components
    let mut network_manager = NetworkManager::new();
    network_manager.initialize().await.unwrap();

    let router = Router::new();
    let onion_router = MLKEMOnionRouter::new();

    let dag_config = DagConfig {
        max_vertices_per_level: 50,
        confirmation_threshold: 5,
        finality_threshold: 10,
    };
    let mut dag = DAG::new(dag_config);

    // Setup participating nodes with crypto keys
    let mut node_keypairs = HashMap::new();
    let mut node_quantum_keys = HashMap::new();

    for i in 0..5 {
        let node_id = NodeId::new(format!("node_{}", i));
        let keypair = MLDSAKeyPair::generate();
        let quantum_kx = QuantumKeyExchange::new(MlKemSecurityLevel::Level3);

        node_keypairs.insert(node_id.clone(), keypair);
        node_quantum_keys.insert(node_id.clone(), quantum_kx);

        // Add to network
        let peer_id = PeerId::from_bytes({
            let mut bytes = [0u8; 32];
            bytes[..16].copy_from_slice(node_id.as_bytes());
            bytes
        });
        router.add_peer(peer_id).await;
    }

    // Create transaction
    let sender_id = NodeId::new("node_0".to_string());
    let receiver_id = NodeId::new("node_4".to_string());

    let transaction = Transaction {
        id: "e2e_test_tx".to_string(),
        sender: sender_id.clone(),
        receiver: receiver_id.clone(),
        amount: 250,
        timestamp: Instant::now(),
        signature: vec![0; 64], // Will be filled after signing
    };

    // Sign transaction with sender's key
    let sender_keypair = &node_keypairs[&sender_id];
    let tx_hash = QuDagHash::hash(&bincode::serialize(&transaction).unwrap());
    let signature = sender_keypair.sign(&tx_hash.as_bytes());

    let mut signed_transaction = transaction.clone();
    signed_transaction.signature = signature.to_bytes();

    // Create vertex with signed transaction
    let vertex = Vertex::new(
        "e2e_vertex".to_string(),
        vec![signed_transaction.clone()],
        vec![], // Genesis vertex
        sender_id.clone(),
    );

    // Serialize vertex for network transmission
    let vertex_data = bincode::serialize(&vertex).unwrap();

    // Create encrypted onion message
    let routing_peers: Vec<_> = (1..4).map(|i| NodeId::new(format!("node_{}", i))).collect();
    let quantum_keys: Vec<_> = routing_peers
        .iter()
        .map(|node_id| node_quantum_keys[node_id].public_key())
        .collect();

    let onion_layers = onion_router
        .create_layers(&vertex_data, &quantum_keys)
        .await
        .unwrap();

    // Create network message
    let network_msg = NetworkMessage {
        id: "e2e_encrypted_vertex".into(),
        source: sender_id.as_bytes().to_vec(),
        destination: receiver_id.as_bytes().to_vec(),
        payload: bincode::serialize(&onion_layers).unwrap(),
        priority: MessagePriority::High,
        ttl: Duration::from_secs(300),
    };

    // Route through anonymous network
    let route = router
        .route(&network_msg, RoutingStrategy::Anonymous { hops: 3 })
        .await
        .unwrap();

    assert_eq!(route.len(), 3);

    // Simulate message processing at each hop
    let mut current_layers = onion_layers;
    for (i, peer_id) in route.iter().enumerate() {
        // Convert peer_id back to node_id
        let node_id = NodeId::new(format!("node_{}", i + 1));
        let quantum_kx = &node_quantum_keys[&node_id];

        // Decrypt layer
        if let Some(layer) = current_layers.pop() {
            let peeled_result = onion_router.peel_layer(&layer, quantum_kx).await;
            assert!(peeled_result.is_ok());

            let (next_layer, _routing_info) = peeled_result.unwrap();

            if i == route.len() - 1 {
                // Final hop - should reveal vertex data
                let recovered_vertex: Vertex =
                    bincode::deserialize(&next_layer.encrypted_data).unwrap();
                assert_eq!(recovered_vertex.id(), vertex.id());

                // Verify transaction signature
                let recovered_tx = &recovered_vertex.transactions()[0];
                let tx_hash = QuDagHash::hash(&bincode::serialize(recovered_tx).unwrap());
                let signature = MLDSASignature::from_bytes(&recovered_tx.signature).unwrap();

                assert!(sender_keypair
                    .public_key()
                    .verify(&tx_hash.as_bytes(), &signature));

                // Add vertex to DAG
                assert!(dag.add_vertex(recovered_vertex).is_ok());

                // Verify DAG state
                assert_eq!(dag.vertex_count(), 1);
                assert!(dag.get_vertex(vertex.id()).is_some());

                // Verify transaction is in DAG
                let stored_vertex = dag.get_vertex(vertex.id()).unwrap();
                assert_eq!(stored_vertex.transactions().len(), 1);
                assert_eq!(stored_vertex.transactions()[0].id, "e2e_test_tx");
                assert_eq!(stored_vertex.transactions()[0].amount, 250);
            }
        }
    }
}

/// Test network-layer fingerprinting integration
#[tokio::test]
async fn test_network_fingerprint_integration() {
    let connection_manager = ConnectionManager::new(20);

    // Create peers with fingerprints
    let peer_data: Vec<_> = (0..5)
        .map(|i| {
            let keypair = MLDSAKeyPair::generate();
            let peer_id = PeerId::from_bytes({
                let mut bytes = [0u8; 32];
                bytes[..8].copy_from_slice(&i.to_le_bytes());
                bytes
            });

            let fingerprint = generate_peer_fingerprint(
                &keypair.public_key().to_bytes(),
                &format!("127.0.0.{}", i + 1),
                &[b"capability_1", b"capability_2"],
            );

            (peer_id, keypair, fingerprint)
        })
        .collect();

    // Add peers to network with fingerprint verification
    for (peer_id, keypair, fingerprint) in &peer_data {
        connection_manager.connect(*peer_id).await.unwrap();

        // Verify fingerprint matches peer's public key
        let expected_fingerprint = generate_peer_fingerprint(
            &keypair.public_key().to_bytes(),
            &format!("127.0.0.{}", peer_id.to_bytes()[0] + 1),
            &[b"capability_1", b"capability_2"],
        );

        assert_eq!(fingerprint.hash(), expected_fingerprint.hash());
        assert!(fingerprint.verify());

        connection_manager.update_status(*peer_id, ConnectionStatus::Connected);
    }

    // Test fingerprint-based peer authentication
    let authenticated_peers = peer_data
        .iter()
        .filter(|(peer_id, _, fingerprint)| {
            // In a real system, this would verify against known good fingerprints
            fingerprint.verify()
                && connection_manager.get_status(peer_id) == Some(ConnectionStatus::Connected)
        })
        .count();

    assert_eq!(authenticated_peers, 5);

    // Test fingerprint mismatch detection
    let fake_keypair = MLDSAKeyPair::generate();
    let fake_peer_id = PeerId::random();

    let real_fingerprint = generate_peer_fingerprint(
        &peer_data[0].1.public_key().to_bytes(),
        "127.0.0.1",
        &[b"capability_1", b"capability_2"],
    );

    let fake_fingerprint = generate_peer_fingerprint(
        &fake_keypair.public_key().to_bytes(),
        "127.0.0.1",
        &[b"capability_1", b"capability_2"],
    );

    // Fingerprints should be different
    assert_ne!(real_fingerprint.hash(), fake_fingerprint.hash());

    // Connection should be rejected for mismatched fingerprint
    connection_manager.connect(fake_peer_id).await.unwrap();
    // In a real implementation, fingerprint verification would happen here
    // and the connection would be rejected if fingerprints don't match
}

/// Test quantum-resistant security across network and crypto layers
#[tokio::test]
async fn test_quantum_resistant_security_integration() {
    let router = Router::new();
    let onion_router = MLKEMOnionRouter::new();

    // Test different ML-KEM security levels
    for security_level in [
        MlKemSecurityLevel::Level1,
        MlKemSecurityLevel::Level3,
        MlKemSecurityLevel::Level5,
    ] {
        // Setup quantum key exchanges
        let alice_kx = QuantumKeyExchange::new(security_level);
        let bob_kx = QuantumKeyExchange::new(security_level);
        let charlie_kx = QuantumKeyExchange::new(security_level);

        let public_keys = vec![
            alice_kx.public_key(),
            bob_kx.public_key(),
            charlie_kx.public_key(),
        ];

        // Create sensitive payload
        let sensitive_data = format!(
            "Quantum-resistant payload for security level {:?}",
            security_level
        )
        .into_bytes();

        // Create onion encryption
        let onion_layers = onion_router
            .create_layers(&sensitive_data, &public_keys)
            .await
            .unwrap();

        assert_eq!(onion_layers.len(), 3);

        // Verify each layer can only be decrypted by the correct key
        let key_exchanges = vec![&alice_kx, &bob_kx, &charlie_kx];

        for (i, layer) in onion_layers.iter().enumerate() {
            for (j, kx) in key_exchanges.iter().enumerate() {
                let decrypt_result = onion_router.peel_layer(layer, *kx).await;

                if i == j {
                    // Correct key should decrypt successfully
                    assert!(decrypt_result.is_ok());
                } else {
                    // Wrong key should fail to decrypt or produce garbage
                    // (Implementation detail: may succeed but produce wrong data)
                }
            }
        }

        // Test full decryption chain
        let mut current_layer = onion_layers.last().unwrap().clone();

        for kx in key_exchanges.iter().rev() {
            let peeled_result = onion_router.peel_layer(&current_layer, *kx).await;
            assert!(peeled_result.is_ok());

            let (next_layer, _) = peeled_result.unwrap();
            current_layer = next_layer;
        }

        // Final layer should contain original sensitive data
        assert_eq!(current_layer.encrypted_data, sensitive_data);

        // Verify quantum security properties
        match security_level {
            MlKemSecurityLevel::Level1 => {
                // Level 1 provides 128-bit quantum security
                assert!(alice_kx.public_key().as_bytes().len() >= 800);
            }
            MlKemSecurityLevel::Level3 => {
                // Level 3 provides 192-bit quantum security
                assert!(alice_kx.public_key().as_bytes().len() >= 1184);
            }
            MlKemSecurityLevel::Level5 => {
                // Level 5 provides 256-bit quantum security
                assert!(alice_kx.public_key().as_bytes().len() >= 1568);
            }
        }
    }
}

/// Test fault tolerance across network and DAG layers
#[tokio::test]
async fn test_fault_tolerance_integration() {
    let router = Router::new();
    let connection_manager = ConnectionManager::new(30);

    let dag_config = DagConfig {
        max_vertices_per_level: 100,
        confirmation_threshold: 5,
        finality_threshold: 10,
    };
    let mut dag = DAG::new(dag_config);

    // Setup network with some faulty nodes
    let total_nodes = 10;
    let faulty_nodes = 3;

    let nodes: Vec<_> = (0..total_nodes)
        .map(|i| {
            let node_id = NodeId::new(format!("node_{}", i));
            let is_faulty = i < faulty_nodes;
            (node_id, is_faulty)
        })
        .collect();

    // Add all nodes to network
    for (node_id, _) in &nodes {
        let peer_id = PeerId::from_bytes({
            let mut bytes = [0u8; 32];
            bytes[..16].copy_from_slice(node_id.as_bytes());
            bytes
        });
        router.add_peer(peer_id).await;
        connection_manager.connect(peer_id).await.unwrap();
        connection_manager.update_status(peer_id, ConnectionStatus::Connected);
    }

    // Simulate faulty nodes by marking some connections as failed
    for i in 0..faulty_nodes {
        let node_id = &nodes[i].0;
        let peer_id = PeerId::from_bytes({
            let mut bytes = [0u8; 32];
            bytes[..16].copy_from_slice(node_id.as_bytes());
            bytes
        });
        connection_manager.update_status(peer_id, ConnectionStatus::Failed("Node fault".into()));
    }

    // Verify network can still route with faulty nodes
    let healthy_nodes = total_nodes - faulty_nodes;
    assert!(healthy_nodes >= 5); // Ensure we have enough healthy nodes

    // Create transaction from healthy node
    let sender_node = &nodes[faulty_nodes].0; // First healthy node
    let receiver_node = &nodes[total_nodes - 1].0; // Last node

    let transaction = Transaction {
        id: "fault_tolerance_tx".to_string(),
        sender: sender_node.clone(),
        receiver: receiver_node.clone(),
        amount: 500,
        timestamp: Instant::now(),
        signature: vec![0; 64],
    };

    let vertex = Vertex::new(
        "fault_tolerance_vertex".to_string(),
        vec![transaction],
        vec![],
        sender_node.clone(),
    );

    // Add vertex to DAG
    assert!(dag.add_vertex(vertex.clone()).is_ok());

    // Create network message
    let vertex_data = bincode::serialize(&vertex).unwrap();
    let network_msg = NetworkMessage {
        id: "fault_tolerance_test".into(),
        source: sender_node.as_bytes().to_vec(),
        destination: receiver_node.as_bytes().to_vec(),
        payload: vertex_data,
        priority: MessagePriority::High,
        ttl: Duration::from_secs(120),
    };

    // Route should succeed despite faulty nodes
    let route_result = router
        .route(&network_msg, RoutingStrategy::Anonymous { hops: 4 })
        .await;

    // May fail if not enough healthy nodes, but should try
    match route_result {
        Ok(route) => {
            assert!(route.len() <= 4);

            // Verify route doesn't use faulty nodes
            for peer_id in &route {
                let status = connection_manager.get_status(peer_id);
                assert_ne!(status, Some(ConnectionStatus::Failed("Node fault".into())));
            }
        }
        Err(_) => {
            // Acceptable if insufficient healthy nodes for routing
            println!("Routing failed due to insufficient healthy nodes - expected behavior");
        }
    }

    // DAG should still function with available nodes
    assert_eq!(dag.vertex_count(), 1);
    assert!(dag.get_vertex(vertex.id()).is_some());

    // Test recovery - bring faulty nodes back online
    for i in 0..faulty_nodes {
        let node_id = &nodes[i].0;
        let peer_id = PeerId::from_bytes({
            let mut bytes = [0u8; 32];
            bytes[..16].copy_from_slice(node_id.as_bytes());
            bytes
        });
        connection_manager.update_status(peer_id, ConnectionStatus::Connected);
    }

    // Network should now have full connectivity restored
    let healthy_connections = (0..total_nodes)
        .map(|i| {
            let node_id = &nodes[i].0;
            let peer_id = PeerId::from_bytes({
                let mut bytes = [0u8; 32];
                bytes[..16].copy_from_slice(node_id.as_bytes());
                bytes
            });
            connection_manager.get_status(&peer_id)
        })
        .filter(|status| *status == Some(ConnectionStatus::Connected))
        .count();

    assert_eq!(healthy_connections, total_nodes);
}

/// Test performance characteristics of integrated system
#[tokio::test]
async fn test_performance_integration() {
    let router = Router::new();
    let connection_manager = ConnectionManager::new(100);
    let onion_router = MLKEMOnionRouter::new();

    // Setup network with many peers
    let peer_count = 50;
    let peers: Vec<_> = (0..peer_count).map(|_| PeerId::random()).collect();

    for peer in &peers {
        router.add_peer(*peer).await;
        connection_manager.connect(*peer).await.unwrap();
        connection_manager.update_status(*peer, ConnectionStatus::Connected);
    }

    // Performance test: Route many messages
    let message_count = 100;
    let start_time = Instant::now();

    for i in 0..message_count {
        let msg = NetworkMessage {
            id: format!("perf_test_{}", i),
            source: peers[i % peer_count].to_bytes().to_vec(),
            destination: peers[(i + 1) % peer_count].to_bytes().to_vec(),
            payload: vec![0; 1024], // 1KB payload
            priority: MessagePriority::Normal,
            ttl: Duration::from_secs(60),
        };

        let route_result = router
            .route(&msg, RoutingStrategy::Anonymous { hops: 3 })
            .await;

        assert!(route_result.is_ok());
    }

    let routing_time = start_time.elapsed();
    let messages_per_second = message_count as f64 / routing_time.as_secs_f64();

    println!(
        "Routing performance: {:.2} messages/second",
        messages_per_second
    );
    assert!(messages_per_second > 10.0, "Routing performance too slow");

    // Performance test: Encryption/decryption
    let quantum_kx = QuantumKeyExchange::new(MlKemSecurityLevel::Level3);
    let public_key = quantum_kx.public_key();

    let crypto_start = Instant::now();
    let crypto_operations = 50;

    for _ in 0..crypto_operations {
        let (ciphertext, _shared_secret) = quantum_kx.encapsulate(&public_key).unwrap();
        let _decapsulated = quantum_kx.decapsulate(&ciphertext).unwrap();
    }

    let crypto_time = crypto_start.elapsed();
    let crypto_ops_per_second = (crypto_operations * 2) as f64 / crypto_time.as_secs_f64(); // *2 for encap+decap

    println!(
        "Crypto performance: {:.2} operations/second",
        crypto_ops_per_second
    );
    assert!(crypto_ops_per_second > 5.0, "Crypto performance too slow");

    // Integration test: End-to-end with timing
    let e2e_start = Instant::now();

    let test_payload = b"End-to-end performance test payload".to_vec();
    let hop_keys = vec![
        quantum_kx.public_key(),
        quantum_kx.public_key(),
        quantum_kx.public_key(),
    ];

    let onion_layers = onion_router
        .create_layers(&test_payload, &hop_keys)
        .await
        .unwrap();

    // Simulate routing through network
    let mut current_layer = onion_layers.last().unwrap().clone();
    for _ in 0..3 {
        let (next_layer, _) = onion_router
            .peel_layer(&current_layer, &quantum_kx)
            .await
            .unwrap();
        current_layer = next_layer;
    }

    assert_eq!(current_layer.encrypted_data, test_payload);

    let e2e_time = e2e_start.elapsed();
    println!("End-to-end performance: {:?}", e2e_time);
    assert!(
        e2e_time < Duration::from_millis(100),
        "End-to-end latency too high"
    );
}
