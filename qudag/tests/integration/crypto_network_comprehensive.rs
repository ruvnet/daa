//! Comprehensive crypto + network integration tests
//! Tests the integration between cryptographic operations and network layer

use qudag_network::{
    NetworkManager, P2PMessage, RoutingTable, Connection,
    types::{PeerId, MessageId, NetworkConfig},
    routing::OnionRouter,
    discovery::PeerDiscovery,
    metrics::NetworkMetrics,
};
use qudag_crypto::{
    ml_kem::{MlKem768, KeyPair as KemKeyPair},
    ml_dsa::{MlDsaKeyPair, MlDsaPublicKey},
    hqc::{Hqc256, SecurityParameter},
    fingerprint::Fingerprint,
    encryption::AsymmetricEncryption,
    signature::DigitalSignature,
};
use qudag_protocol::{Coordinator, ProtocolConfig, Message};
use std::collections::{HashMap, HashSet};
use std::net::{SocketAddr, IpAddr, Ipv4Addr};
use std::time::{Duration, Instant};
use tokio::time::{sleep, timeout};
use tracing::{info, warn, error, debug};

/// Test secure peer discovery with cryptographic authentication
#[tokio::test]
async fn test_secure_peer_discovery() {
    let node_count = 5;
    let mut networks = Vec::new();
    let mut keypairs = Vec::new();
    
    // Create ML-DSA keypairs for each node
    for i in 0..node_count {
        let keypair = MlDsaKeyPair::generate();
        keypairs.push(keypair);
    }
    
    // Create network managers with crypto authentication
    for i in 0..node_count {
        let config = NetworkConfig {
            listen_port: 12000 + i as u16,
            max_peers: 10,
            connection_timeout: Duration::from_secs(5),
            discovery_interval: Duration::from_secs(1),
            bootstrap_nodes: if i == 0 { 
                vec![] 
            } else { 
                vec![SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 12000)]
            },
        };
        
        let mut network = NetworkManager::new(config).await.unwrap();
        network.set_identity_keypair(keypairs[i].clone()).await.unwrap();
        networks.push(network);
    }
    
    // Start all networks
    for network in networks.iter_mut() {
        network.start().await.unwrap();
    }
    
    // Allow time for secure discovery
    sleep(Duration::from_secs(3)).await;
    
    // Verify authenticated connections
    for (i, network) in networks.iter().enumerate() {
        let authenticated_peers = network.get_authenticated_peers().await;
        assert!(
            authenticated_peers.len() >= node_count - 2,
            "Node {} should discover and authenticate at least {} peers, found {}",
            i, node_count - 2, authenticated_peers.len()
        );
        
        // Verify each peer has valid authentication
        for peer_id in &authenticated_peers {
            let peer_pubkey = network.get_peer_public_key(*peer_id).await.unwrap();
            assert!(peer_pubkey.is_some(), "Authenticated peer should have public key");
        }
    }
    
    // Test message authentication during discovery
    let test_message = b"Discovery authentication test";
    let signed_message = keypairs[0].sign(test_message);
    
    networks[0].broadcast_authenticated_discovery(test_message.to_vec(), signed_message).await.unwrap();
    
    sleep(Duration::from_millis(500)).await;
    
    // Verify signature verification during discovery
    for i in 1..node_count {
        let discovery_messages = networks[i].get_discovery_messages().await;
        let authenticated_msg = discovery_messages.iter()
            .find(|msg| msg.payload() == test_message)
            .expect("Should find authenticated discovery message");
        
        assert!(authenticated_msg.is_authenticated(), "Discovery message should be authenticated");
    }
    
    // Stop all networks
    for network in networks.iter_mut() {
        network.stop().await.unwrap();
    }
}

/// Test encrypted message routing with ML-KEM
#[tokio::test]
async fn test_encrypted_message_routing() {
    let node_count = 4;
    let mut networks = Vec::new();
    let mut kem_keypairs = Vec::new();
    
    // Generate ML-KEM keypairs for each node
    for _ in 0..node_count {
        let (pk, sk) = MlKem768::keygen().unwrap();
        kem_keypairs.push((pk, sk));
    }
    
    // Create and start networks
    for i in 0..node_count {
        let config = NetworkConfig {
            listen_port: 12100 + i as u16,
            max_peers: 10,
            connection_timeout: Duration::from_secs(5),
            discovery_interval: Duration::from_secs(1),
            bootstrap_nodes: if i == 0 { 
                vec![] 
            } else { 
                vec![SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 12100)]
            },
        };
        
        let mut network = NetworkManager::new(config).await.unwrap();
        network.set_kem_keypair(kem_keypairs[i].0.clone(), kem_keypairs[i].1.clone()).await.unwrap();
        networks.push(network);
    }
    
    for network in networks.iter_mut() {
        network.start().await.unwrap();
    }
    
    sleep(Duration::from_secs(2)).await;
    
    // Exchange public keys
    for i in 0..node_count {
        for j in 0..node_count {
            if i != j {
                networks[i].register_peer_kem_key(
                    networks[j].local_peer_id(),
                    kem_keypairs[j].0.clone()
                ).await.unwrap();
            }
        }
    }
    
    // Test direct encrypted communication
    let secret_message = b"Top secret encrypted message";
    let sender_id = networks[0].local_peer_id();
    let receiver_id = networks[3].local_peer_id();
    
    // Node 0 sends encrypted message to Node 3
    networks[0].send_encrypted_message(
        receiver_id,
        secret_message.to_vec()
    ).await.unwrap();
    
    sleep(Duration::from_millis(300)).await;
    
    // Verify Node 3 received and decrypted the message
    let received_messages = networks[3].get_decrypted_messages().await;
    assert!(
        received_messages.iter().any(|(sender, msg)| {
            *sender == sender_id && msg == secret_message
        }),
        "Encrypted message not properly received and decrypted"
    );
    
    // Verify intermediate nodes cannot decrypt
    for i in 1..3 {
        let plaintext_messages = networks[i].get_plaintext_messages().await;
        assert!(
            !plaintext_messages.iter().any(|msg| msg == secret_message),
            "Intermediate node {} should not have plaintext access", i
        );
    }
    
    // Test encrypted broadcasting with multiple recipients
    let broadcast_message = b"Encrypted broadcast to all nodes";
    let encrypted_broadcasts = networks[0].create_encrypted_broadcast(
        broadcast_message.to_vec(),
        networks.iter().skip(1).map(|n| n.local_peer_id()).collect()
    ).await.unwrap();
    
    networks[0].send_encrypted_broadcast(encrypted_broadcasts).await.unwrap();
    
    sleep(Duration::from_millis(500)).await;
    
    // Verify all target nodes received the broadcast
    for i in 1..node_count {
        let broadcast_messages = networks[i].get_decrypted_broadcasts().await;
        assert!(
            broadcast_messages.iter().any(|msg| msg == broadcast_message),
            "Node {} didn't receive encrypted broadcast", i
        );
    }
    
    // Stop networks
    for network in networks.iter_mut() {
        network.stop().await.unwrap();
    }
}

/// Test key exchange protocol implementation
#[tokio::test]
async fn test_key_exchange_protocol() {
    let mut networks = Vec::new();
    
    // Create two nodes for key exchange
    for i in 0..2 {
        let config = NetworkConfig {
            listen_port: 12200 + i as u16,
            max_peers: 5,
            connection_timeout: Duration::from_secs(5),
            discovery_interval: Duration::from_secs(1),
            bootstrap_nodes: if i == 0 { 
                vec![] 
            } else { 
                vec![SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 12200)]
            },
        };
        
        let network = NetworkManager::new(config).await.unwrap();
        networks.push(network);
    }
    
    for network in networks.iter_mut() {
        network.start().await.unwrap();
    }
    
    sleep(Duration::from_millis(500)).await;
    
    // Initiate key exchange from Node 0 to Node 1
    let peer_id = networks[1].local_peer_id();
    let key_exchange_result = networks[0].initiate_key_exchange(peer_id).await.unwrap();
    
    // Allow key exchange protocol to complete
    sleep(Duration::from_millis(500)).await;
    
    // Verify shared secret established
    let shared_secret_0 = networks[0].get_shared_secret(peer_id).await.unwrap();
    let shared_secret_1 = networks[1].get_shared_secret(networks[0].local_peer_id()).await.unwrap();
    
    assert!(shared_secret_0.is_some(), "Node 0 should have shared secret");
    assert!(shared_secret_1.is_some(), "Node 1 should have shared secret");
    assert_eq!(
        shared_secret_0.unwrap(),
        shared_secret_1.unwrap(),
        "Shared secrets should match"
    );
    
    // Test secure communication using established keys
    let secure_message = b"Message using established shared secret";
    networks[0].send_secure_message(
        peer_id,
        secure_message.to_vec()
    ).await.unwrap();
    
    sleep(Duration::from_millis(200)).await;
    
    let received_secure = networks[1].get_secure_messages().await;
    assert!(
        received_secure.iter().any(|msg| msg == secure_message),
        "Secure message not received"
    );
    
    // Test key rotation
    networks[0].rotate_shared_key(peer_id).await.unwrap();
    
    sleep(Duration::from_millis(300)).await;
    
    // Verify new keys work
    let post_rotation_message = b"Message after key rotation";
    networks[0].send_secure_message(
        peer_id,
        post_rotation_message.to_vec()
    ).await.unwrap();
    
    sleep(Duration::from_millis(200)).await;
    
    let post_rotation_received = networks[1].get_secure_messages().await;
    assert!(
        post_rotation_received.iter().any(|msg| msg == post_rotation_message),
        "Message after key rotation not received"
    );
    
    for network in networks.iter_mut() {
        network.stop().await.unwrap();
    }
}

/// Test HQC encryption integration with network layer
#[tokio::test]
async fn test_hqc_network_integration() {
    let mut networks = Vec::new();
    let mut hqc_keypairs = Vec::new();
    
    // Generate HQC keypairs
    for _ in 0..3 {
        let (pk, sk) = Hqc256::keygen().unwrap();
        hqc_keypairs.push((pk, sk));
    }
    
    // Create networks with HQC encryption
    for i in 0..3 {
        let config = NetworkConfig {
            listen_port: 12300 + i as u16,
            max_peers: 5,
            connection_timeout: Duration::from_secs(5),
            discovery_interval: Duration::from_secs(1),
            bootstrap_nodes: if i == 0 { 
                vec![] 
            } else { 
                vec![SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 12300)]
            },
        };
        
        let mut network = NetworkManager::new(config).await.unwrap();
        network.set_hqc_keypair(hqc_keypairs[i].0.clone(), hqc_keypairs[i].1.clone()).await.unwrap();
        networks.push(network);
    }
    
    for network in networks.iter_mut() {
        network.start().await.unwrap();
    }
    
    sleep(Duration::from_secs(1)).await;
    
    // Register HQC public keys
    for i in 0..3 {
        for j in 0..3 {
            if i != j {
                networks[i].register_peer_hqc_key(
                    networks[j].local_peer_id(),
                    hqc_keypairs[j].0.clone()
                ).await.unwrap();
            }
        }
    }
    
    // Test HQC encrypted message transmission
    let hqc_message = b"HQC encrypted network message";
    let target_peer = networks[2].local_peer_id();
    
    networks[0].send_hqc_encrypted_message(
        target_peer,
        hqc_message.to_vec()
    ).await.unwrap();
    
    sleep(Duration::from_millis(400)).await;
    
    let hqc_received = networks[2].get_hqc_decrypted_messages().await;
    assert!(
        hqc_received.iter().any(|msg| msg == hqc_message),
        "HQC encrypted message not received"
    );
    
    // Test HQC performance under network load
    let message_count = 50;
    let start_time = Instant::now();
    
    for i in 0..message_count {
        let msg = format!("HQC load test message {}", i).into_bytes();
        networks[0].send_hqc_encrypted_message(target_peer, msg).await.unwrap();
    }
    
    let encryption_time = start_time.elapsed();
    
    sleep(Duration::from_millis(2000)).await;
    
    let final_received = networks[2].get_hqc_decrypted_messages().await;
    assert!(
        final_received.len() >= message_count,
        "Should receive all HQC encrypted messages under load"
    );
    
    info!(
        "HQC network performance: {} messages in {:?} ({:.2} msg/sec)",
        message_count,
        encryption_time,
        message_count as f64 / encryption_time.as_secs_f64()
    );
    
    for network in networks.iter_mut() {
        network.stop().await.unwrap();
    }
}

/// Test fingerprint-based routing with cryptographic validation
#[tokio::test]
async fn test_fingerprint_routing_validation() {
    let node_count = 6;
    let mut networks = Vec::new();
    let mut fingerprints = Vec::new();
    let mut identity_keys = Vec::new();
    
    // Generate fingerprints and identity keys
    for i in 0..node_count {
        let identity_key = MlDsaKeyPair::generate();
        let node_id = format!("node_{}", i);
        let fingerprint = Fingerprint::from_identity_key(&identity_key.public_key(), &node_id);
        
        fingerprints.push(fingerprint);
        identity_keys.push(identity_key);
    }
    
    // Create networks
    for i in 0..node_count {
        let config = NetworkConfig {
            listen_port: 12400 + i as u16,
            max_peers: 10,
            connection_timeout: Duration::from_secs(5),
            discovery_interval: Duration::from_secs(1),
            bootstrap_nodes: if i == 0 { 
                vec![] 
            } else { 
                vec![SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 12400)]
            },
        };
        
        let mut network = NetworkManager::new(config).await.unwrap();
        network.set_fingerprint(fingerprints[i].clone()).await.unwrap();
        network.set_identity_keypair(identity_keys[i].clone()).await.unwrap();
        networks.push(network);
    }
    
    for network in networks.iter_mut() {
        network.start().await.unwrap();
    }
    
    sleep(Duration::from_secs(2)).await;
    
    // Register all fingerprints and public keys
    for i in 0..node_count {
        for j in 0..node_count {
            if i != j {
                networks[i].register_peer_fingerprint(
                    networks[j].local_peer_id(),
                    fingerprints[j].clone()
                ).await.unwrap();
                networks[i].register_peer_identity_key(
                    networks[j].local_peer_id(),
                    identity_keys[j].public_key()
                ).await.unwrap();
            }
        }
    }
    
    // Test fingerprint-based routing with validation
    let source_fp = fingerprints[0].clone();
    let dest_fp = fingerprints[5].clone();
    let anonymous_message = b"Validated anonymous routing message";
    
    // Sign the message for validation
    let signature = identity_keys[0].sign(anonymous_message);
    
    networks[0].send_validated_anonymous_message(
        dest_fp,
        anonymous_message.to_vec(),
        signature
    ).await.unwrap();
    
    sleep(Duration::from_millis(800)).await;
    
    // Verify destination received and validated the message
    let validated_messages = networks[5].get_validated_anonymous_messages().await;
    assert!(
        validated_messages.iter().any(|(fp, msg, valid)| {
            *fp == source_fp && msg == anonymous_message && *valid
        }),
        "Validated anonymous message not received"
    );
    
    // Test routing path validation
    let routing_path = networks[0].get_last_routing_path().await.unwrap();
    assert!(routing_path.len() >= 2, "Routing path should have multiple hops");
    
    // Verify each hop in the path has valid fingerprints
    for hop_fp in &routing_path {
        assert!(
            fingerprints.contains(hop_fp),
            "Routing path contains invalid fingerprint"
        );
    }
    
    // Test route discovery with fingerprint constraints
    let constrained_route = networks[0].discover_route_to_fingerprint(
        dest_fp,
        |fp| fp.anonymity_level() >= 3 // Only use high-anonymity nodes
    ).await.unwrap();
    
    assert!(!constrained_route.is_empty(), "Should find constrained route");
    for hop in &constrained_route {
        assert!(
            hop.anonymity_level() >= 3,
            "Route constraint not satisfied"
        );
    }
    
    for network in networks.iter_mut() {
        network.stop().await.unwrap();
    }
}

/// Test multi-layer encryption for enhanced security
#[tokio::test]
async fn test_multi_layer_encryption() {
    let mut networks = Vec::new();
    
    // Create 4 nodes for multi-layer encryption test
    for i in 0..4 {
        let config = NetworkConfig {
            listen_port: 12500 + i as u16,
            max_peers: 10,
            connection_timeout: Duration::from_secs(5),
            discovery_interval: Duration::from_secs(1),
            bootstrap_nodes: if i == 0 { 
                vec![] 
            } else { 
                vec![SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 12500)]
            },
        };
        
        let mut network = NetworkManager::new(config).await.unwrap();
        
        // Set up multiple encryption layers (ML-KEM + HQC)
        let (kem_pk, kem_sk) = MlKem768::keygen().unwrap();
        let (hqc_pk, hqc_sk) = Hqc256::keygen().unwrap();
        
        network.set_kem_keypair(kem_pk, kem_sk).await.unwrap();
        network.set_hqc_keypair(hqc_pk, hqc_sk).await.unwrap();
        
        networks.push(network);
    }
    
    for network in networks.iter_mut() {
        network.start().await.unwrap();
    }
    
    sleep(Duration::from_secs(1)).await;
    
    // Exchange all public keys
    for i in 0..4 {
        for j in 0..4 {
            if i != j {
                let kem_pk = networks[j].get_kem_public_key().await.unwrap();
                let hqc_pk = networks[j].get_hqc_public_key().await.unwrap();
                
                networks[i].register_peer_kem_key(
                    networks[j].local_peer_id(),
                    kem_pk
                ).await.unwrap();
                networks[i].register_peer_hqc_key(
                    networks[j].local_peer_id(),
                    hqc_pk
                ).await.unwrap();
            }
        }
    }
    
    // Test double encryption (ML-KEM + HQC)
    let sensitive_data = b"Ultra-sensitive multi-layer encrypted data";
    let target_peer = networks[3].local_peer_id();
    
    networks[0].send_double_encrypted_message(
        target_peer,
        sensitive_data.to_vec()
    ).await.unwrap();
    
    sleep(Duration::from_millis(600)).await;
    
    let double_decrypted = networks[3].get_double_decrypted_messages().await;
    assert!(
        double_decrypted.iter().any(|msg| msg == sensitive_data),
        "Double encrypted message not properly decrypted"
    );
    
    // Test onion routing with multi-layer encryption
    let onion_path = vec![
        networks[1].local_peer_id(),
        networks[2].local_peer_id(),
        networks[3].local_peer_id(),
    ];
    
    let onion_message = b"Multi-layer onion routed message";
    networks[0].send_multi_layer_onion_message(
        onion_message.to_vec(),
        onion_path
    ).await.unwrap();
    
    sleep(Duration::from_millis(1000)).await;
    
    // Verify destination received the message
    let onion_received = networks[3].get_onion_decrypted_messages().await;
    assert!(
        onion_received.iter().any(|msg| msg == onion_message),
        "Multi-layer onion message not received"
    );
    
    // Verify intermediate nodes cannot decrypt any layer
    for i in 1..3 {
        let partial_decrypted = networks[i].get_partial_decrypted_messages().await;
        assert!(
            !partial_decrypted.iter().any(|msg| msg == onion_message),
            "Intermediate node {} decrypted too many layers", i
        );
    }
    
    for network in networks.iter_mut() {
        network.stop().await.unwrap();
    }
}

/// Test cryptographic protocol resilience under network stress
#[tokio::test]
async fn test_crypto_network_stress() {
    let node_count = 8;
    let mut networks = Vec::new();
    let mut all_keypairs = Vec::new();
    
    // Set up networks with full crypto stack
    for i in 0..node_count {
        let config = NetworkConfig {
            listen_port: 12600 + i as u16,
            max_peers: 15,
            connection_timeout: Duration::from_secs(10),
            discovery_interval: Duration::from_millis(500),
            bootstrap_nodes: if i == 0 { 
                vec![] 
            } else { 
                vec![SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 12600)]
            },
        };
        
        let mut network = NetworkManager::new(config).await.unwrap();
        
        // Generate all key types
        let identity_key = MlDsaKeyPair::generate();
        let (kem_pk, kem_sk) = MlKem768::keygen().unwrap();
        let (hqc_pk, hqc_sk) = Hqc256::keygen().unwrap();
        let fingerprint = Fingerprint::from_identity_key(&identity_key.public_key(), &format!("stress_node_{}", i));
        
        network.set_identity_keypair(identity_key.clone()).await.unwrap();
        network.set_kem_keypair(kem_pk.clone(), kem_sk).await.unwrap();
        network.set_hqc_keypair(hqc_pk.clone(), hqc_sk).await.unwrap();
        network.set_fingerprint(fingerprint.clone()).await.unwrap();
        
        all_keypairs.push((identity_key, kem_pk, hqc_pk, fingerprint));
        networks.push(network);
    }
    
    for network in networks.iter_mut() {
        network.start().await.unwrap();
    }
    
    sleep(Duration::from_secs(3)).await;
    
    // Full key exchange between all nodes
    for i in 0..node_count {
        for j in 0..node_count {
            if i != j {
                let peer_id = networks[j].local_peer_id();
                let (identity, kem_pk, hqc_pk, fingerprint) = &all_keypairs[j];
                
                networks[i].register_peer_identity_key(peer_id, identity.public_key()).await.unwrap();
                networks[i].register_peer_kem_key(peer_id, kem_pk.clone()).await.unwrap();
                networks[i].register_peer_hqc_key(peer_id, hqc_pk.clone()).await.unwrap();
                networks[i].register_peer_fingerprint(peer_id, fingerprint.clone()).await.unwrap();
            }
        }
    }
    
    // Stress test: High-frequency encrypted message exchange
    let stress_duration = Duration::from_secs(10);
    let start_time = Instant::now();
    let mut message_count = 0;
    
    while start_time.elapsed() < stress_duration {
        // Each node sends encrypted messages to random targets
        for i in 0..node_count {
            let target_idx = (i + 1) % node_count;
            let target_peer = networks[target_idx].local_peer_id();
            
            let message = format!("Stress test message {} from node {}", message_count, i).into_bytes();
            
            // Randomly choose encryption type
            match message_count % 3 {
                0 => {
                    networks[i].send_encrypted_message(target_peer, message).await.unwrap();
                }
                1 => {
                    networks[i].send_hqc_encrypted_message(target_peer, message).await.unwrap();
                }
                2 => {
                    networks[i].send_double_encrypted_message(target_peer, message).await.unwrap();
                }
                _ => unreachable!(),
            }
            
            message_count += 1;
        }
        
        sleep(Duration::from_millis(10)).await;
    }
    
    // Allow message propagation
    sleep(Duration::from_secs(2)).await;
    
    info!("Stress test completed: {} messages sent in {:?}", message_count, stress_duration);
    
    // Verify message delivery under stress
    let mut total_received = 0;
    for (i, network) in networks.iter().enumerate() {
        let received_encrypted = network.get_decrypted_messages().await.len();
        let received_hqc = network.get_hqc_decrypted_messages().await.len();
        let received_double = network.get_double_decrypted_messages().await.len();
        
        let node_total = received_encrypted + received_hqc + received_double;
        total_received += node_total;
        
        info!("Node {} received {} messages (encrypted: {}, HQC: {}, double: {})",
              i, node_total, received_encrypted, received_hqc, received_double);
    }
    
    let delivery_rate = total_received as f64 / message_count as f64;
    assert!(
        delivery_rate > 0.95,
        "Message delivery rate under stress should exceed 95%, got {:.2}%",
        delivery_rate * 100.0
    );
    
    // Test network recovery after stress
    let recovery_message = b"Network recovery test message";
    networks[0].broadcast_message(recovery_message.to_vec()).await.unwrap();
    
    sleep(Duration::from_millis(500)).await;
    
    for i in 1..node_count {
        let broadcast_messages = networks[i].get_broadcast_messages().await;
        assert!(
            broadcast_messages.iter().any(|msg| msg == recovery_message),
            "Node {} didn't receive recovery broadcast", i
        );
    }
    
    for network in networks.iter_mut() {
        network.stop().await.unwrap();
    }
}