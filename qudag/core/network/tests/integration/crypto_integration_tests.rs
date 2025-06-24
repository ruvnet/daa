//! Integration tests between network and crypto modules

use qudag_network::{
    onion::{MLKEMOnionRouter, OnionRouter},
    quantum_crypto::{MlKemSecurityLevel, QuantumKeyExchange},
    transport::{SecureTransport, TransportConfig},
    types::{NetworkMessage, PeerId},
    NetworkManager, NetworkConfig,
};
use qudag_crypto::{
    ml_kem::{MLKEMKeyPair, MLKEMParams},
    ml_dsa::{MLDSAKeyPair, MLDSAParams},
    signature::Signature,
    hash::Hash,
};
use std::collections::HashMap;
use std::time::Duration;

#[tokio::test]
async fn test_quantum_secure_onion_routing() {
    // Setup quantum-secure onion router
    let router = MLKEMOnionRouter::new(MLKEMParams::ML_KEM_768);
    
    // Generate quantum-resistant keys for nodes
    let node_count = 5;
    let mut node_keys = HashMap::new();
    let mut node_ids = vec![];
    
    for i in 0..node_count {
        let peer_id = PeerId::from_bytes([i as u8; 32]);
        let keypair = MLKEMKeyPair::generate(MLKEMParams::ML_KEM_768);
        node_keys.insert(peer_id, keypair);
        node_ids.push(peer_id);
    }
    
    // Create sensitive message
    let secret_message = b"Quantum-resistant secret data".to_vec();
    
    // Extract public keys
    let public_keys: HashMap<_, _> = node_keys.iter()
        .map(|(id, kp)| (*id, kp.public_key.clone()))
        .collect();
    
    // Wrap message with quantum-secure encryption
    let wrapped = router.wrap_quantum_message(
        secret_message.clone(),
        node_ids.clone(),
        &public_keys
    ).await;
    
    assert!(wrapped.is_ok());
    let onion = wrapped.unwrap();
    
    // Verify each layer uses quantum-resistant encryption
    assert_eq!(onion.layers.len(), node_count);
    for layer in &onion.layers {
        // Each layer should have ML-KEM encrypted data
        assert!(!layer.encryption_key().is_empty());
        assert!(!layer.payload().is_empty());
    }
}

#[tokio::test]
async fn test_signed_network_messages() {
    // Generate ML-DSA signing keys
    let sender_keys = MLDSAKeyPair::generate(MLDSAParams::ML_DSA_65);
    let sender_id = PeerId::random();
    
    // Create and sign message
    let message_data = b"Authenticated network message";
    let message_hash = Hash::sha256(message_data);
    let signature = sender_keys.sign(&message_hash.as_bytes());
    
    // Create signed network message
    let network_msg = NetworkMessage::SignedData {
        data: message_data.to_vec(),
        signature: signature.to_bytes(),
        signer_id: sender_id,
        algorithm: "ML-DSA-65".to_string(),
    };
    
    // Verify signature on receiving end
    match network_msg {
        NetworkMessage::SignedData { data, signature, signer_id, algorithm } => {
            assert_eq!(signer_id, sender_id);
            assert_eq!(algorithm, "ML-DSA-65");
            
            // Reconstruct and verify
            let received_hash = Hash::sha256(&data);
            let sig = Signature::from_bytes(&signature).unwrap();
            assert!(sender_keys.public_key.verify(&received_hash.as_bytes(), &sig));
        }
        _ => panic!("Expected signed message"),
    }
}

#[tokio::test]
async fn test_encrypted_transport_layer() {
    // Setup secure transport with quantum crypto
    let config = TransportConfig {
        enable_encryption: true,
        enable_authentication: true,
        quantum_resistant: true,
        timeout: Duration::from_secs(30),
    };
    
    let transport = SecureTransport::new(config);
    
    // Generate ML-KEM keys for transport encryption
    let alice_transport_keys = MLKEMKeyPair::generate(MLKEMParams::ML_KEM_1024);
    let bob_transport_keys = MLKEMKeyPair::generate(MLKEMParams::ML_KEM_1024);
    
    // Simulate key exchange
    let alice_id = PeerId::random();
    let bob_id = PeerId::random();
    
    // Alice encapsulates for Bob
    let (ciphertext, shared_secret_alice) = alice_transport_keys.public_key
        .encapsulate(&bob_transport_keys.public_key);
    
    // Bob decapsulates
    let shared_secret_bob = bob_transport_keys.decapsulate(&ciphertext).unwrap();
    
    // Verify shared secrets match
    assert_eq!(shared_secret_alice.as_bytes(), shared_secret_bob.as_bytes());
    
    // Use shared secret for transport encryption
    let encrypted_data = transport.encrypt_with_key(
        b"Secure transport data",
        shared_secret_alice.as_bytes()
    ).await;
    
    assert!(encrypted_data.is_ok());
}

#[tokio::test]
async fn test_network_manager_with_crypto() {
    let config = NetworkConfig {
        quantum_resistant: true,
        max_connections: 10,
        ..Default::default()
    };
    
    let mut manager = NetworkManager::with_config(config);
    manager.initialize().await.unwrap();
    
    // Generate crypto materials for peer
    let peer_keys = MLDSAKeyPair::generate(MLDSAParams::ML_DSA_87);
    let peer_id = manager.connect_peer("127.0.0.1:8080").await.unwrap();
    
    // Create authenticated message
    let data = b"Authenticated peer message";
    let signature = peer_keys.sign(data);
    
    let auth_message = NetworkMessage::SignedData {
        data: data.to_vec(),
        signature: signature.to_bytes(),
        signer_id: PeerId::random(),
        algorithm: "ML-DSA-87".to_string(),
    };
    
    // Send authenticated message
    let serialized = bincode::serialize(&auth_message).unwrap();
    assert!(manager.send_message(&peer_id, serialized).await.is_ok());
}

#[tokio::test]
async fn test_hash_chain_verification() {
    use qudag_crypto::hash::HashChain;
    
    // Create hash chain for message integrity
    let mut chain = HashChain::new();
    
    // Simulate network message sequence
    let messages = vec![
        NetworkMessage::Data(b"Message 1".to_vec()),
        NetworkMessage::Data(b"Message 2".to_vec()),
        NetworkMessage::Data(b"Message 3".to_vec()),
    ];
    
    let mut message_hashes = vec![];
    
    for msg in &messages {
        let serialized = bincode::serialize(msg).unwrap();
        let hash = chain.add(&serialized);
        message_hashes.push(hash);
    }
    
    // Verify chain integrity
    assert!(chain.verify());
    
    // Verify individual message membership
    for (i, hash) in message_hashes.iter().enumerate() {
        assert!(chain.contains(hash));
        assert_eq!(chain.get_index(hash), Some(i));
    }
}

#[tokio::test]
async fn test_fingerprint_based_peer_identity() {
    use qudag_crypto::fingerprint::Fingerprint;
    
    // Generate peer identity fingerprints
    let peer_keys = MLDSAKeyPair::generate(MLDSAParams::ML_DSA_65);
    let public_key_bytes = bincode::serialize(&peer_keys.public_key).unwrap();
    
    let fingerprint = Fingerprint::from_data(&public_key_bytes);
    let peer_id = PeerId::from_bytes(fingerprint.as_bytes()[..32].try_into().unwrap());
    
    // Create identity-bound message
    let message = NetworkMessage::IdentityBound {
        data: b"Peer-specific data".to_vec(),
        fingerprint: fingerprint.to_string(),
        peer_id,
    };
    
    // Verify identity binding
    match message {
        NetworkMessage::IdentityBound { data: _, fingerprint: fp, peer_id: pid } => {
            let reconstructed_fp = Fingerprint::from_string(&fp).unwrap();
            let expected_peer_id = PeerId::from_bytes(
                reconstructed_fp.as_bytes()[..32].try_into().unwrap()
            );
            assert_eq!(pid, expected_peer_id);
        }
        _ => panic!("Expected identity-bound message"),
    }
}

#[tokio::test]
async fn test_multi_signature_consensus() {
    // Multiple nodes sign the same message for consensus
    let node_count = 5;
    let mut node_keys = vec![];
    let mut signatures = vec![];
    
    for _ in 0..node_count {
        let keys = MLDSAKeyPair::generate(MLDSAParams::ML_DSA_65);
        node_keys.push(keys);
    }
    
    // Message to be signed by all nodes
    let consensus_data = b"Network consensus block #12345";
    let consensus_hash = Hash::sha256(consensus_data);
    
    // Each node signs
    for keys in &node_keys {
        let sig = keys.sign(&consensus_hash.as_bytes());
        signatures.push(sig);
    }
    
    // Create multi-signed network message
    let multi_sig_msg = NetworkMessage::MultiSigned {
        data: consensus_data.to_vec(),
        signatures: signatures.iter().map(|s| s.to_bytes()).collect(),
        signers: (0..node_count).map(|i| PeerId::from_bytes([i as u8; 32])).collect(),
        threshold: 3, // Require at least 3 signatures
    };
    
    // Verify multi-signature
    match multi_sig_msg {
        NetworkMessage::MultiSigned { data, signatures: sigs, signers: _, threshold } => {
            let data_hash = Hash::sha256(&data);
            let mut valid_signatures = 0;
            
            for (i, sig_bytes) in sigs.iter().enumerate() {
                if let Ok(sig) = Signature::from_bytes(sig_bytes) {
                    if node_keys[i].public_key.verify(&data_hash.as_bytes(), &sig) {
                        valid_signatures += 1;
                    }
                }
            }
            
            assert!(valid_signatures >= threshold);
        }
        _ => panic!("Expected multi-signed message"),
    }
}

#[tokio::test]
async fn test_encrypted_routing_metadata() {
    use qudag_crypto::encryption::{encrypt, decrypt};
    
    // Encrypt routing metadata to hide traffic patterns
    let routing_info = HashMap::from([
        ("next_hop", "peer123"),
        ("final_dest", "peer456"),
        ("route_id", "route789"),
    ]);
    
    let metadata_bytes = bincode::serialize(&routing_info).unwrap();
    let encryption_key = vec![0u8; 32]; // In practice, use proper key derivation
    
    // Encrypt metadata
    let encrypted_metadata = encrypt(&metadata_bytes, &encryption_key).unwrap();
    
    // Create message with encrypted routing
    let routed_msg = NetworkMessage::EncryptedRoute {
        payload: b"Actual message data".to_vec(),
        encrypted_metadata,
        algorithm: "ChaCha20Poly1305".to_string(),
    };
    
    // On receiving end, decrypt routing info
    match routed_msg {
        NetworkMessage::EncryptedRoute { payload: _, encrypted_metadata: enc_meta, algorithm: _ } => {
            let decrypted = decrypt(&enc_meta, &encryption_key).unwrap();
            let routing: HashMap<&str, &str> = bincode::deserialize(&decrypted).unwrap();
            
            assert_eq!(routing.get("next_hop"), Some(&"peer123"));
            assert_eq!(routing.get("final_dest"), Some(&"peer456"));
        }
        _ => panic!("Expected encrypted route message"),
    }
}