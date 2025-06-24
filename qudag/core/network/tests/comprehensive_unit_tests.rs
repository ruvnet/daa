//! Comprehensive unit tests for all network components
//!
//! This module provides exhaustive testing coverage for individual network components,
//! ensuring each module functions correctly in isolation.

use qudag_network::{
    connection::{
        ConnectionInfo, ConnectionManager, HealthStatistics, SecureConfig, TransportKeys,
    },
    dark_resolver::{DarkDomainRecord, DarkResolver, DarkResolverError},
    discovery::{DiscoveryConfig, DiscoveryEvent, DiscoveryMethod, KademliaPeerDiscovery},
    dns::{CloudflareClient, CloudflareConfig, DnsManager, DnsRecord, RecordType},
    message::MessageEnvelope,
    onion::{
        MLKEMOnionRouter, MetadataConfig, MetadataProtector, MixConfig, MixNode, OnionError,
        OnionLayer, TrafficAnalysisConfig,
    },
    peer::{PeerInfo, PeerManager, PeerStatus},
    quantum_crypto::{MlKemSecurityLevel, QuantumKeyExchange},
    router::{HopInfo, Router},
    shadow_address::{
        DefaultShadowAddressHandler, NetworkType, ShadowAddress, ShadowAddressGenerator,
    },
    transport::{AsyncTransport, Transport, TransportConfig},
    types::{
        ConnectionStatus, MessagePriority, NetworkError, NetworkMessage, PeerId, RoutingStrategy,
    },
    NetworkConfig, NetworkManager, NetworkStats, PeerMetadata, ReputationManager,
};
use rand::{thread_rng, Rng};
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

mod connection_tests {
    use super::*;

    #[tokio::test]
    async fn test_connection_manager_lifecycle() {
        let manager = ConnectionManager::new(100);

        // Test basic connection operations
        let peer_id = PeerId::random();
        assert!(manager.connect(peer_id).await.is_ok());
        assert_eq!(manager.connection_count(), 1);

        // Test status updates
        manager.update_status(peer_id, ConnectionStatus::Connected);
        assert_eq!(
            manager.get_status(&peer_id),
            Some(ConnectionStatus::Connected)
        );

        // Test metrics
        manager.update_metrics(1000.0, 50);
        let metrics = manager.get_metrics();
        assert_eq!(metrics.active_connections, 1);
        assert!(metrics.messages_per_second > 0.0);

        // Test disconnection
        manager.disconnect(&peer_id);
        assert_eq!(manager.connection_count(), 0);
        assert_eq!(manager.get_status(&peer_id), None);
    }

    #[tokio::test]
    async fn test_connection_limits() {
        let manager = ConnectionManager::new(3); // Small limit for testing

        let peers: Vec<_> = (0..5).map(|_| PeerId::random()).collect();
        let mut successful_connections = 0;

        for peer in peers {
            if manager.connect(peer).await.is_ok() {
                successful_connections += 1;
            }
        }

        // Should enforce the limit
        assert!(successful_connections <= 3);
        assert!(manager.connection_count() <= 3);
    }

    #[tokio::test]
    async fn test_connection_health_monitoring() {
        let manager = ConnectionManager::new(10);

        // Create some connections
        let peers: Vec<_> = (0..5).map(|_| PeerId::random()).collect();
        for peer in &peers {
            manager.connect(*peer).await.unwrap();
            manager.update_status(*peer, ConnectionStatus::Connected);
        }

        // Simulate unhealthy connections
        manager.update_status(peers[0], ConnectionStatus::Failed("Timeout".into()));
        manager.update_status(peers[1], ConnectionStatus::Failed("Network error".into()));

        let health_stats = manager.get_health_statistics();
        assert_eq!(health_stats.total_connections, 5);
        assert_eq!(health_stats.unhealthy_connections, 2);
        assert_eq!(health_stats.healthy_connections, 3);

        let unhealthy = manager.get_unhealthy_connections();
        assert_eq!(unhealthy.len(), 2);
        assert!(unhealthy.iter().any(|info| info.peer_id == peers[0]));
        assert!(unhealthy.iter().any(|info| info.peer_id == peers[1]));
    }

    #[tokio::test]
    async fn test_secure_config_generation() {
        let config = SecureConfig {
            transport_keys: TransportKeys::generate(),
            timeout: Duration::from_secs(30),
            keepalive: Duration::from_secs(60),
        };

        // Test that keys are properly generated
        assert!(config.transport_keys.public_key.len() > 0);
        assert!(config.transport_keys.private_key.len() > 0);
        assert_ne!(
            config.transport_keys.public_key,
            config.transport_keys.private_key
        );
    }

    #[tokio::test]
    async fn test_connection_auto_recovery() {
        let manager = ConnectionManager::new(10);

        // Create connections and mark some as failed
        let peers: Vec<_> = (0..5).map(|_| PeerId::random()).collect();
        for peer in &peers {
            manager.connect(*peer).await.unwrap();
        }

        // Mark some as failed
        manager.update_status(peers[0], ConnectionStatus::Failed("Network error".into()));
        manager.update_status(peers[1], ConnectionStatus::Failed("Timeout".into()));

        // Test auto-recovery
        let recovered = manager.auto_recover().await.unwrap();
        assert_eq!(recovered, 2);

        // Verify connections are healthy again
        assert_eq!(
            manager.get_status(&peers[0]),
            Some(ConnectionStatus::Connected)
        );
        assert_eq!(
            manager.get_status(&peers[1]),
            Some(ConnectionStatus::Connected)
        );
    }
}

mod peer_tests {
    use super::*;

    #[tokio::test]
    async fn test_peer_manager_operations() {
        let manager = PeerManager::new();

        let peer_id = PeerId::random();
        let peer_info = PeerInfo {
            id: peer_id,
            address: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
            status: PeerStatus::Connected,
            last_seen: Instant::now(),
            reputation: 100,
        };

        // Test adding peer
        assert!(manager.add_peer(peer_info.clone()).await.is_ok());
        assert_eq!(manager.peer_count().await, 1);

        // Test getting peer
        let retrieved = manager.get_peer(&peer_id).await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, peer_id);

        // Test updating peer status
        assert!(manager
            .update_peer_status(peer_id, PeerStatus::Disconnected)
            .await
            .is_ok());
        let updated = manager.get_peer(&peer_id).await.unwrap();
        assert_eq!(updated.status, PeerStatus::Disconnected);

        // Test removing peer
        assert!(manager.remove_peer(&peer_id).await.is_ok());
        assert_eq!(manager.peer_count().await, 0);
        assert!(manager.get_peer(&peer_id).await.is_none());
    }

    #[tokio::test]
    async fn test_peer_reputation_system() {
        let manager = PeerManager::new();

        let peer_id = PeerId::random();
        let mut peer_info = PeerInfo {
            id: peer_id,
            address: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
            status: PeerStatus::Connected,
            last_seen: Instant::now(),
            reputation: 50,
        };

        manager.add_peer(peer_info.clone()).await.unwrap();

        // Test reputation updates
        manager.update_peer_reputation(peer_id, 25).await.unwrap();
        let updated = manager.get_peer(&peer_id).await.unwrap();
        assert_eq!(updated.reputation, 75);

        // Test reputation bounds
        manager.update_peer_reputation(peer_id, 50).await.unwrap();
        let bounded = manager.get_peer(&peer_id).await.unwrap();
        assert!(bounded.reputation <= 100); // Should be clamped
    }

    #[tokio::test]
    async fn test_peer_discovery_integration() {
        let manager = PeerManager::new();

        // Test peer discovery with Kademlia
        let config = DiscoveryConfig {
            method: DiscoveryMethod::Kademlia,
            interval: Duration::from_secs(30),
            max_peers: 100,
            bootstrap_addresses: vec!["127.0.0.1:8080".to_string()],
        };

        let discovery = KademliaPeerDiscovery::new(config);
        // Note: This would need actual network setup for full integration testing

        // Test discovery event handling
        let discovered_peer = PeerId::random();
        let event = DiscoveryEvent::PeerDiscovered {
            peer_id: discovered_peer,
            address: "127.0.0.1:8081".to_string(),
        };

        // In a real implementation, this would be handled by the discovery service
        match event {
            DiscoveryEvent::PeerDiscovered { peer_id, address } => {
                let peer_info = PeerInfo {
                    id: peer_id,
                    address: address.parse().unwrap(),
                    status: PeerStatus::Discovered,
                    last_seen: Instant::now(),
                    reputation: 0,
                };
                assert!(manager.add_peer(peer_info).await.is_ok());
            }
            _ => {}
        }

        assert_eq!(manager.peer_count().await, 1);
    }

    #[tokio::test]
    async fn test_peer_status_transitions() {
        let manager = PeerManager::new();

        let peer_id = PeerId::random();
        let peer_info = PeerInfo {
            id: peer_id,
            address: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
            status: PeerStatus::Discovered,
            last_seen: Instant::now(),
            reputation: 0,
        };

        manager.add_peer(peer_info).await.unwrap();

        // Test valid status transitions
        let transitions = vec![
            PeerStatus::Connecting,
            PeerStatus::Connected,
            PeerStatus::Disconnected,
            PeerStatus::Banned,
        ];

        for status in transitions {
            assert!(manager.update_peer_status(peer_id, status).await.is_ok());
            let peer = manager.get_peer(&peer_id).await.unwrap();
            assert_eq!(peer.status, status);
        }
    }
}

mod router_tests {
    use super::*;

    #[tokio::test]
    async fn test_anonymous_routing() {
        let router = Router::new();

        // Add test peers
        let peers: Vec<_> = (0..10).map(|_| PeerId::random()).collect();
        for peer in &peers {
            router.add_peer(*peer).await;
        }

        let msg = NetworkMessage {
            id: "test_route".into(),
            source: peers[0].to_bytes().to_vec(),
            destination: peers[9].to_bytes().to_vec(),
            payload: vec![0; 100],
            priority: MessagePriority::Normal,
            ttl: Duration::from_secs(60),
        };

        // Test anonymous routing
        let route = router
            .route(&msg, RoutingStrategy::Anonymous { hops: 3 })
            .await
            .unwrap();

        assert_eq!(route.len(), 3);

        // Verify route doesn't contain source or destination
        assert!(!route.contains(&peers[0]));
        assert!(!route.contains(&peers[9]));

        // Verify all peers in route are unique
        let mut unique_check = std::collections::HashSet::new();
        for peer in &route {
            assert!(unique_check.insert(*peer));
        }
    }

    #[tokio::test]
    async fn test_hop_information() {
        let router = Router::new();

        let peers: Vec<_> = (0..5).map(|_| PeerId::random()).collect();
        for peer in &peers {
            router.add_peer(*peer).await;
        }

        let msg = NetworkMessage {
            id: "hop_test".into(),
            source: peers[0].to_bytes().to_vec(),
            destination: peers[4].to_bytes().to_vec(),
            payload: vec![0; 100],
            priority: MessagePriority::Normal,
            ttl: Duration::from_secs(60),
        };

        let route = router
            .route(&msg, RoutingStrategy::Anonymous { hops: 3 })
            .await
            .unwrap();

        // Test hop information for each peer in route
        for (i, peer_id) in route.iter().enumerate() {
            let hop_info = router.get_hop_info(peer_id).await;
            assert!(hop_info.is_ok());

            let info = hop_info.unwrap();
            assert_eq!(info.peer_id, *peer_id);
            assert!(info.can_decrypt_layer(i));

            // Verify isolation - hop should only know adjacent peers
            let known_peers = info.known_peers.len();
            assert!(known_peers <= 2); // Previous and next hop only
        }
    }

    #[tokio::test]
    async fn test_routing_strategies() {
        let router = Router::new();

        let peers: Vec<_> = (0..20).map(|_| PeerId::random()).collect();
        for peer in &peers {
            router.add_peer(*peer).await;
        }

        let msg = NetworkMessage {
            id: "strategy_test".into(),
            source: peers[0].to_bytes().to_vec(),
            destination: peers[19].to_bytes().to_vec(),
            payload: vec![0; 100],
            priority: MessagePriority::Normal,
            ttl: Duration::from_secs(60),
        };

        // Test different routing strategies
        let anonymous_route = router
            .route(&msg, RoutingStrategy::Anonymous { hops: 5 })
            .await
            .unwrap();
        assert_eq!(anonymous_route.len(), 5);

        let direct_route = router.route(&msg, RoutingStrategy::Direct).await.unwrap();
        assert_eq!(direct_route.len(), 1);
        assert_eq!(direct_route[0], peers[19]); // Should route directly to destination
    }

    #[tokio::test]
    async fn test_route_validation() {
        let router = Router::new();

        let peers: Vec<_> = (0..5).map(|_| PeerId::random()).collect();
        for peer in &peers {
            router.add_peer(*peer).await;
        }

        // Test valid route
        let valid_route = vec![peers[1], peers[2], peers[3]];
        assert!(router.validate_route(&valid_route).await.is_ok());

        // Test route with duplicate peers (should fail)
        let duplicate_route = vec![peers[1], peers[2], peers[1]];
        assert!(router.validate_route(&duplicate_route).await.is_err());

        // Test route with unknown peer (should fail)
        let unknown_peer = PeerId::random();
        let unknown_route = vec![peers[1], unknown_peer, peers[3]];
        assert!(router.validate_route(&unknown_route).await.is_err());
    }
}

mod onion_tests {
    use super::*;

    #[tokio::test]
    async fn test_onion_layer_creation() {
        let router = MLKEMOnionRouter::new();

        let payload = b"secret message".to_vec();
        let peer_keys: Vec<_> = (0..3)
            .map(|_| {
                let kx = QuantumKeyExchange::new(MlKemSecurityLevel::Level3);
                kx.public_key()
            })
            .collect();

        // Create onion layers
        let layers = router.create_layers(&payload, &peer_keys).await;
        assert!(layers.is_ok());

        let onion_layers = layers.unwrap();
        assert_eq!(onion_layers.len(), 3);

        // Each layer should be larger than the previous (due to encryption overhead)
        for i in 1..onion_layers.len() {
            assert!(
                onion_layers[i].encrypted_data.len() >= onion_layers[i - 1].encrypted_data.len()
            );
        }
    }

    #[tokio::test]
    async fn test_onion_layer_peeling() {
        let router = MLKEMOnionRouter::new();

        let original_payload = b"test message for onion routing".to_vec();
        let key_exchanges: Vec<_> = (0..3)
            .map(|_| QuantumKeyExchange::new(MlKemSecurityLevel::Level3))
            .collect();

        let peer_keys: Vec<_> = key_exchanges.iter().map(|kx| kx.public_key()).collect();

        // Create onion layers
        let layers = router
            .create_layers(&original_payload, &peer_keys)
            .await
            .unwrap();

        // Test peeling layers in reverse order (as each hop would do)
        let mut current_layer = layers.last().unwrap().clone();

        for (i, kx) in key_exchanges.iter().enumerate().rev() {
            let peeled = router.peel_layer(&current_layer, kx).await;
            assert!(peeled.is_ok());

            let (next_layer, _routing_info) = peeled.unwrap();

            if i == 0 {
                // Last layer should reveal original payload
                assert_eq!(next_layer.encrypted_data, original_payload);
            } else {
                current_layer = next_layer;
            }
        }
    }

    #[tokio::test]
    async fn test_metadata_protection() {
        let config = MetadataConfig {
            padding_size: 1024,
            timing_variance: Duration::from_millis(100),
            dummy_traffic_rate: 0.1,
        };

        let protector = MetadataProtector::new(config);

        let original_data = b"sensitive data".to_vec();
        let protected = protector.protect_metadata(&original_data).await;

        // Protected data should be larger due to padding
        assert!(protected.data.len() >= original_data.len());
        assert!(protected.data.len() >= 1024); // At least padding size

        // Should have timing protection
        assert!(protected.timing_delay > Duration::ZERO);

        // Test unprotecting
        let unprotected = protector.unprotect_metadata(&protected).await.unwrap();
        assert_eq!(unprotected, original_data);
    }

    #[tokio::test]
    async fn test_traffic_analysis_resistance() {
        let config = TrafficAnalysisConfig {
            min_message_size: 512,
            max_message_size: 2048,
            padding_probability: 0.3,
            dummy_message_probability: 0.1,
        };

        let resistance = TrafficAnalysisResistance::new(config);

        // Test message normalization
        let small_message = vec![0u8; 100];
        let normalized = resistance.normalize_message(&small_message).await;
        assert!(normalized.len() >= 512); // Should be padded to minimum size

        let large_message = vec![0u8; 3000];
        let normalized_large = resistance.normalize_message(&large_message).await;
        assert!(normalized_large.len() <= 2048); // Should be chunked to maximum size

        // Test dummy traffic generation
        let dummy = resistance.generate_dummy_traffic().await;
        assert!(dummy.len() >= 512 && dummy.len() <= 2048);
    }

    #[tokio::test]
    async fn test_mix_node_operations() {
        let config = MixConfig {
            batch_size: 10,
            batch_timeout: Duration::from_millis(100),
            mixing_strategy: "random".to_string(),
        };

        let mut mix_node = MixNode::new(config);

        // Add messages to mix
        for i in 0..5 {
            let message = format!("message_{}", i).into_bytes();
            mix_node.add_message(message).await;
        }

        // Process batch
        let mixed_messages = mix_node.process_batch().await.unwrap();
        assert_eq!(mixed_messages.len(), 5);

        // Messages should be reordered (mixed)
        let original_order: Vec<_> = (0..5)
            .map(|i| format!("message_{}", i).into_bytes())
            .collect();
        assert_ne!(mixed_messages, original_order); // Very likely to be different order

        // Get statistics
        let stats = mix_node.get_stats().await;
        assert_eq!(stats.messages_processed, 5);
        assert!(stats.average_batch_size > 0.0);
    }
}

mod quantum_crypto_tests {
    use super::*;

    #[tokio::test]
    async fn test_quantum_key_exchange() {
        // Test different security levels
        for level in [
            MlKemSecurityLevel::Level1,
            MlKemSecurityLevel::Level3,
            MlKemSecurityLevel::Level5,
        ] {
            let alice_kx = QuantumKeyExchange::new(level);
            let bob_kx = QuantumKeyExchange::new(level);

            let alice_public = alice_kx.public_key();
            let bob_public = bob_kx.public_key();

            // Verify key sizes are correct for security level
            match level {
                MlKemSecurityLevel::Level1 => {
                    assert_eq!(alice_public.as_bytes().len(), 800);
                }
                MlKemSecurityLevel::Level3 => {
                    assert_eq!(alice_public.as_bytes().len(), 1184);
                }
                MlKemSecurityLevel::Level5 => {
                    assert_eq!(alice_public.as_bytes().len(), 1568);
                }
            }

            // Test encapsulation/decapsulation
            let (ciphertext, alice_shared_secret) = alice_kx.encapsulate(&bob_public).unwrap();
            let bob_shared_secret = bob_kx.decapsulate(&ciphertext).unwrap();

            // Shared secrets should match
            assert_eq!(alice_shared_secret.as_bytes(), bob_shared_secret.as_bytes());
            assert_eq!(alice_shared_secret.as_bytes().len(), 32); // 256 bits
        }
    }

    #[tokio::test]
    async fn test_quantum_security_properties() {
        let kx = QuantumKeyExchange::new(MlKemSecurityLevel::Level3);

        // Test key freshness - each generation should produce different keys
        let key1 = kx.public_key();
        let key2 = kx.public_key();
        assert_ne!(key1.as_bytes(), key2.as_bytes());

        // Test that invalid ciphertexts are rejected
        let public_key = kx.public_key();
        let (valid_ciphertext, _) = kx.encapsulate(&public_key).unwrap();

        // Corrupt the ciphertext
        let mut corrupted_ciphertext = valid_ciphertext.clone();
        corrupted_ciphertext.as_bytes_mut()[0] ^= 0xFF;

        // Decapsulation should fail or produce different shared secret
        let result = kx.decapsulate(&corrupted_ciphertext);
        if let Ok(corrupted_secret) = result {
            let (_, original_secret) = kx.encapsulate(&public_key).unwrap();
            assert_ne!(corrupted_secret.as_bytes(), original_secret.as_bytes());
        }
    }

    #[tokio::test]
    async fn test_quantum_performance() {
        let kx = QuantumKeyExchange::new(MlKemSecurityLevel::Level3);
        let public_key = kx.public_key();

        let start = Instant::now();
        let iterations = 100;

        for _ in 0..iterations {
            let (ciphertext, _shared_secret) = kx.encapsulate(&public_key).unwrap();
            let _decapsulated = kx.decapsulate(&ciphertext).unwrap();
        }

        let elapsed = start.elapsed();
        let ops_per_sec = iterations as f64 / elapsed.as_secs_f64();

        // Performance should be reasonable (at least 10 ops/sec)
        assert!(
            ops_per_sec > 10.0,
            "Quantum crypto performance too slow: {} ops/sec",
            ops_per_sec
        );
    }
}

mod shadow_address_tests {
    use super::*;

    #[tokio::test]
    async fn test_shadow_address_generation() {
        let mut generator = ShadowAddressGenerator::new();

        // Test IPv4 shadow addresses
        let ipv4_addr = generator.generate_ipv4().await;
        assert!(ipv4_addr.is_private());

        // Test IPv6 shadow addresses
        let ipv6_addr = generator.generate_ipv6().await;
        assert!(ipv6_addr.is_unique_local());

        // Test Tor-style addresses
        let tor_addr = generator.generate_tor_style().await;
        assert_eq!(tor_addr.len(), 56); // Tor v3 address length
        assert!(tor_addr.ends_with(".onion"));
    }

    #[tokio::test]
    async fn test_shadow_address_resolution() {
        let handler = DefaultShadowAddressHandler::new();

        // Create test shadow address
        let shadow_addr = ShadowAddress {
            address: "test.shadow".to_string(),
            network_type: NetworkType::Tor,
            metadata: Default::default(),
        };

        // Test registration and resolution
        assert!(handler.register_address(shadow_addr.clone()).await.is_ok());

        let resolved = handler.resolve_address("test.shadow").await;
        assert!(resolved.is_ok());

        let resolved_addr = resolved.unwrap();
        assert_eq!(resolved_addr.address, "test.shadow");
        assert_eq!(resolved_addr.network_type, NetworkType::Tor);
    }

    #[tokio::test]
    async fn test_shadow_address_privacy() {
        let mut generator = ShadowAddressGenerator::new();

        // Generate multiple addresses and ensure they're different
        let mut addresses = std::collections::HashSet::new();
        for _ in 0..100 {
            let addr = generator.generate_tor_style().await;
            assert!(addresses.insert(addr)); // Should be unique
        }

        // Test that addresses don't reveal patterns
        let addresses: Vec<_> = addresses.into_iter().collect();
        for i in 0..addresses.len() - 1 {
            let addr1 = &addresses[i];
            let addr2 = &addresses[i + 1];

            // Should not have obvious sequential patterns
            let common_prefix_len = addr1
                .chars()
                .zip(addr2.chars())
                .take_while(|(a, b)| a == b)
                .count();

            // Common prefix should be small (just the .onion suffix)
            assert!(common_prefix_len < 10);
        }
    }
}

mod transport_tests {
    use super::*;

    #[tokio::test]
    async fn test_transport_config() {
        let config = TransportConfig {
            max_frame_size: 1024 * 1024,
            timeout: Duration::from_secs(30),
            keepalive: Duration::from_secs(60),
            buffer_size: 64 * 1024,
        };

        // Test that config values are reasonable
        assert!(config.max_frame_size > 0);
        assert!(config.timeout > Duration::ZERO);
        assert!(config.keepalive > Duration::ZERO);
        assert!(config.buffer_size > 0);

        // Test validation
        assert!(config.max_frame_size <= 16 * 1024 * 1024); // Reasonable max
        assert!(config.timeout <= Duration::from_secs(300)); // Reasonable timeout
    }

    #[tokio::test]
    async fn test_async_transport_interface() {
        // This would test the AsyncTransport trait implementation
        // For now, we test the interface design

        let config = TransportConfig::default();

        // In a real implementation, this would create an actual transport
        // For testing, we verify the interface is properly designed
        assert_eq!(config.max_frame_size, 1024 * 1024);
        assert_eq!(config.timeout, Duration::from_secs(30));
    }
}

mod dns_tests {
    use super::*;

    #[tokio::test]
    async fn test_dns_record_creation() {
        let record = DnsRecord {
            name: "test.example.com".to_string(),
            record_type: RecordType::A,
            content: "192.0.2.1".to_string(),
            ttl: 300,
        };

        assert_eq!(record.name, "test.example.com");
        assert_eq!(record.record_type, RecordType::A);
        assert_eq!(record.content, "192.0.2.1");
        assert_eq!(record.ttl, 300);
    }

    #[tokio::test]
    async fn test_cloudflare_config() {
        let config = CloudflareConfig {
            api_token: "test_token".to_string(),
            zone_id: "test_zone".to_string(),
            base_url: "https://api.cloudflare.com/client/v4".to_string(),
        };

        assert!(!config.api_token.is_empty());
        assert!(!config.zone_id.is_empty());
        assert!(config.base_url.starts_with("https://"));
    }
}

mod dark_resolver_tests {
    use super::*;

    #[tokio::test]
    async fn test_dark_domain_record() {
        let record = DarkDomainRecord {
            domain: "dark.test".to_string(),
            target: "192.0.2.100".to_string(),
            port: 8080,
            priority: 10,
            weight: 5,
        };

        assert_eq!(record.domain, "dark.test");
        assert_eq!(record.target, "192.0.2.100");
        assert_eq!(record.port, 8080);
        assert!(record.priority > 0);
        assert!(record.weight > 0);
    }

    #[tokio::test]
    async fn test_dark_resolver_operations() {
        let mut resolver = DarkResolver::new();

        let record = DarkDomainRecord {
            domain: "test.dark".to_string(),
            target: "10.0.0.1".to_string(),
            port: 9999,
            priority: 1,
            weight: 1,
        };

        // Test adding record
        assert!(resolver.add_record(record.clone()).await.is_ok());

        // Test resolving
        let resolved = resolver.resolve("test.dark").await;
        assert!(resolved.is_ok());

        let resolved_record = resolved.unwrap();
        assert_eq!(resolved_record.domain, "test.dark");
        assert_eq!(resolved_record.target, "10.0.0.1");
        assert_eq!(resolved_record.port, 9999);

        // Test removing record
        assert!(resolver.remove_record("test.dark").await.is_ok());

        // Should no longer resolve
        let not_found = resolver.resolve("test.dark").await;
        assert!(matches!(not_found, Err(DarkResolverError::NotFound)));
    }
}

mod network_manager_tests {
    use super::*;

    #[tokio::test]
    async fn test_network_manager_initialization() {
        let mut manager = NetworkManager::new();

        // Test initialization
        assert!(manager.initialize().await.is_ok());
        assert!(manager.local_peer_id().is_some());

        // Test configuration
        let config = NetworkConfig {
            max_connections: 25,
            connection_timeout: Duration::from_secs(15),
            discovery_interval: Duration::from_secs(30),
            bootstrap_peers: vec!["127.0.0.1:8080".to_string()],
            enable_dht: true,
            quantum_resistant: true,
        };

        let configured_manager = NetworkManager::with_config(config.clone());
        assert_eq!(configured_manager.config.max_connections, 25);
        assert_eq!(
            configured_manager.config.connection_timeout,
            Duration::from_secs(15)
        );
    }

    #[tokio::test]
    async fn test_peer_management() {
        let mut manager = NetworkManager::new();
        manager.initialize().await.unwrap();

        // Test connecting to peer
        let peer_id = manager.connect_peer("127.0.0.1:8080").await.unwrap();

        let connected_peers = manager.get_connected_peers().await;
        assert_eq!(connected_peers.len(), 1);
        assert!(connected_peers.contains(&peer_id));

        // Test getting peer metadata
        let metadata = manager.get_peer_metadata(&peer_id).await;
        assert!(metadata.is_some());

        // Test sending message
        let message_data = b"test message".to_vec();
        assert!(manager.send_message(&peer_id, message_data).await.is_ok());

        // Test disconnecting
        assert!(manager.disconnect_peer(&peer_id).await.is_ok());

        let connected_after_disconnect = manager.get_connected_peers().await;
        assert_eq!(connected_after_disconnect.len(), 0);
    }

    #[tokio::test]
    async fn test_reputation_management() {
        let mut manager = NetworkManager::new();
        manager.initialize().await.unwrap();

        let peer_id = manager.connect_peer("127.0.0.1:8080").await.unwrap();

        // Test adding trusted peer
        manager.add_trusted_peer(peer_id).await;

        let stats = manager.get_network_stats().await;
        assert_eq!(stats.trusted_peers, 1);

        // Test blacklisting peer
        manager.blacklist_peer(peer_id).await;

        let updated_stats = manager.get_network_stats().await;
        assert_eq!(updated_stats.blacklisted_peers, 1);
        assert_eq!(updated_stats.connected_peers, 0); // Should be disconnected

        // Test that blacklisted peer cannot reconnect
        let reconnect_result = manager.connect_peer("127.0.0.1:8080").await;
        assert!(reconnect_result.is_err());
    }

    #[tokio::test]
    async fn test_maintenance_operations() {
        let mut manager = NetworkManager::new();
        manager.initialize().await.unwrap();

        // Connect some peers
        let peer1 = manager.connect_peer("127.0.0.1:8080").await.unwrap();
        let peer2 = manager.connect_peer("127.0.0.1:8081").await.unwrap();

        assert_eq!(manager.get_connected_peers().await.len(), 2);

        // Run maintenance - should cleanup inactive peers after timeout simulation
        manager.maintenance().await;

        // For this test, we'd need to simulate time passage or modify timestamps
        // In a real scenario, inactive peers would be disconnected

        // Test shutdown
        assert!(manager.shutdown().await.is_ok());
        assert_eq!(manager.get_connected_peers().await.len(), 0);
    }
}

mod reputation_tests {
    use super::*;

    #[tokio::test]
    async fn test_reputation_manager() {
        let mut rep_manager = ReputationManager::default();

        let peer_id = libp2p::PeerId::random();

        // Test initial reputation
        assert_eq!(rep_manager.get_reputation(&peer_id), 0.0);

        // Test reputation updates
        rep_manager.update_reputation(peer_id, 25.0);
        assert_eq!(rep_manager.get_reputation(&peer_id), 25.0);

        rep_manager.update_reputation(peer_id, -10.0);
        assert_eq!(rep_manager.get_reputation(&peer_id), 15.0);

        // Test reputation bounds
        rep_manager.update_reputation(peer_id, 200.0); // Should be clamped
        assert_eq!(rep_manager.get_reputation(&peer_id), 100.0);

        rep_manager.update_reputation(peer_id, -300.0); // Should be clamped and blacklisted
        assert_eq!(rep_manager.get_reputation(&peer_id), -100.0);
        assert!(rep_manager.is_blacklisted(&peer_id));
    }

    #[tokio::test]
    async fn test_trusted_peers() {
        let mut rep_manager = ReputationManager::default();

        let peer_id = libp2p::PeerId::random();

        // Test adding trusted peer
        rep_manager.add_trusted(peer_id);
        assert!(rep_manager.is_trusted(&peer_id));
        assert_eq!(rep_manager.get_reputation(&peer_id), 75.0);
    }

    #[tokio::test]
    async fn test_blacklist_cleanup() {
        let mut rep_manager = ReputationManager::default();

        let peer_id = libp2p::PeerId::random();

        // Blacklist peer
        rep_manager.update_reputation(peer_id, -100.0);
        assert!(rep_manager.is_blacklisted(&peer_id));

        // Test cleanup (would need time manipulation in real test)
        rep_manager.cleanup_expired();

        // For now, peer should still be blacklisted since time hasn't passed
        assert!(rep_manager.is_blacklisted(&peer_id));
    }
}

#[tokio::test]
async fn test_message_envelope_integrity() {
    let msg = NetworkMessage {
        id: "integrity_test".into(),
        source: vec![1, 2, 3, 4],
        destination: vec![5, 6, 7, 8],
        payload: b"test payload".to_vec(),
        priority: MessagePriority::High,
        ttl: Duration::from_secs(60),
    };

    let envelope = MessageEnvelope::new(msg.clone());

    // Test verification
    assert!(envelope.verify());

    // Test that tampering is detected
    let mut tampered_envelope = envelope.clone();
    tampered_envelope.message.payload = b"tampered payload".to_vec();

    // Should fail verification due to hash mismatch
    assert!(!tampered_envelope.verify());
}

#[tokio::test]
async fn test_comprehensive_integration() {
    // Test integration between multiple components
    let mut network_manager = NetworkManager::new();
    network_manager.initialize().await.unwrap();

    let router = Router::new();
    let connection_manager = ConnectionManager::new(100);

    // Add peers to router
    let peers: Vec<_> = (0..5).map(|_| PeerId::random()).collect();
    for peer in &peers {
        router.add_peer(*peer).await;
        connection_manager.connect(*peer).await.unwrap();
    }

    // Create and route a message
    let msg = NetworkMessage {
        id: "integration_test".into(),
        source: peers[0].to_bytes().to_vec(),
        destination: peers[4].to_bytes().to_vec(),
        payload: b"integration test message".to_vec(),
        priority: MessagePriority::Normal,
        ttl: Duration::from_secs(60),
    };

    let route = router
        .route(&msg, RoutingStrategy::Anonymous { hops: 3 })
        .await
        .unwrap();

    assert_eq!(route.len(), 3);

    // Verify all peers in route are connected
    for peer in &route {
        assert_eq!(
            connection_manager.get_status(peer),
            Some(ConnectionStatus::Connecting)
        );
    }

    // Test network statistics
    let stats = network_manager.get_network_stats().await;
    assert!(stats.connected_peers >= 0);
    assert!(stats.average_reputation >= -100.0 && stats.average_reputation <= 100.0);
}
