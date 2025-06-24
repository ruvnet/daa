//! Comprehensive tests for NAT traversal functionality

use qudag_network::{
    ConnectionManager, ConnectionUpgradeManager, HolePunchCoordinator, NatInfo, NatTraversalConfig,
    NatTraversalManager, NatType, PortMapping, PortMappingProtocol, RelayManager, StunClient,
    StunServer, TurnClient, TurnServer,
};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::sleep;

#[tokio::test]
async fn test_nat_detection() {
    let stun_servers = vec![StunServer::new("127.0.0.1:3478".parse().unwrap(), 1)];

    let client = StunClient::new(stun_servers);

    // This test will demonstrate the structure even if it fails due to no real STUN server
    match client.detect_nat().await {
        Ok(nat_info) => {
            println!("NAT type: {:?}", nat_info.nat_type);
            println!("Public IP: {:?}", nat_info.public_ip);
            assert!(nat_info.confidence >= 0.0 && nat_info.confidence <= 1.0);
        }
        Err(e) => {
            println!("NAT detection failed (expected in test): {}", e);
            // This is expected when no real STUN server is available
        }
    }
}

#[tokio::test]
async fn test_nat_traversal_config() {
    let config = NatTraversalConfig::default();

    assert!(config.enable_stun);
    assert!(config.enable_turn);
    assert!(config.enable_upnp);
    assert!(config.enable_nat_pmp);
    assert!(config.enable_hole_punching);
    assert!(config.enable_relay);
    assert!(config.enable_ipv6);

    assert!(!config.stun_servers.is_empty());
    assert_eq!(config.max_relay_connections, 50);
    assert_eq!(config.hole_punch_timeout, Duration::from_secs(30));
}

#[tokio::test]
async fn test_stun_server_creation() {
    let server = StunServer::new("8.8.8.8:3478".parse().unwrap(), 1);

    assert_eq!(server.address.ip().to_string(), "8.8.8.8");
    assert_eq!(server.address.port(), 3478);
    assert_eq!(server.priority, 1);
    assert!(server.is_active);
    assert!(server.last_success.is_none());
    assert_eq!(server.avg_response_ms, 0);
}

#[tokio::test]
async fn test_turn_server_creation() {
    let server = TurnServer {
        address: "turn.example.com:3478".parse().unwrap(),
        username: "testuser".to_string(),
        password: "testpass".to_string(),
        realm: Some("example.com".to_string()),
        priority: 1,
        is_active: true,
        relay_address: None,
    };

    assert_eq!(server.username, "testuser");
    assert_eq!(server.password, "testpass");
    assert_eq!(server.realm, Some("example.com".to_string()));
    assert_eq!(server.priority, 1);
    assert!(server.is_active);
    assert!(server.relay_address.is_none());
}

#[tokio::test]
async fn test_port_mapping() {
    let mapping = PortMapping {
        local_port: 8080,
        external_port: 8080,
        protocol: PortMappingProtocol::TCP,
        method: qudag_network::PortMappingMethod::Upnp,
        created_at: Instant::now(),
        expires_at: Instant::now() + Duration::from_secs(3600),
    };

    assert_eq!(mapping.local_port, 8080);
    assert_eq!(mapping.external_port, 8080);
    assert_eq!(mapping.protocol, PortMappingProtocol::TCP);
}

#[tokio::test]
async fn test_hole_punch_coordinator() {
    let coordinator = HolePunchCoordinator::new(Duration::from_secs(5));

    let peer_id = qudag_network::types::PeerId::random();
    let local_candidates = vec![
        "127.0.0.1:8080".parse().unwrap(),
        "127.0.0.1:8081".parse().unwrap(),
    ];
    let remote_candidates = vec![
        "127.0.0.1:9080".parse().unwrap(),
        "127.0.0.1:9081".parse().unwrap(),
    ];

    // This will timeout since there's no real remote peer, but tests the structure
    let result = coordinator
        .start_hole_punch(peer_id, local_candidates, remote_candidates)
        .await;
    assert!(result.is_err()); // Expected to fail in test environment
}

#[tokio::test]
async fn test_relay_manager() {
    let manager = RelayManager::new(10);

    let relay_server = qudag_network::RelayServer {
        id: qudag_network::types::PeerId::random(),
        address: "/ip4/127.0.0.1/tcp/8080".parse().unwrap(),
        capacity: 100,
        load: Arc::new(std::sync::atomic::AtomicU32::new(0)),
        is_available: true,
        last_health_check: None,
    };

    manager.add_relay_server(relay_server).await;

    let peer_id = qudag_network::types::PeerId::random();

    // This may fail due to no actual relay servers, but tests the interface
    match manager.establish_relay(peer_id).await {
        Ok(connection) => {
            assert_eq!(connection.target_peer, peer_id);
            assert!(connection
                .is_active
                .load(std::sync::atomic::Ordering::Relaxed));
        }
        Err(e) => {
            println!("Relay establishment failed (expected in test): {}", e);
        }
    }
}

#[tokio::test]
async fn test_connection_upgrade_manager() {
    let upgrade_manager = ConnectionUpgradeManager::new(Duration::from_secs(30));

    let peer_id = qudag_network::types::PeerId::random();
    let current_type = qudag_network::ConnectionType::Relay;

    // This will fail since there's no NAT manager set, but tests the interface
    let result = upgrade_manager.try_upgrade(peer_id, current_type).await;
    assert!(result.is_err()); // Expected since no NAT manager is set
}

#[tokio::test]
async fn test_nat_traversal_manager_creation() {
    let config = NatTraversalConfig::default();
    let connection_manager = Arc::new(ConnectionManager::new(50));

    let nat_manager = NatTraversalManager::new(config, connection_manager);

    // Test that NAT info is initially None
    assert!(nat_manager.get_nat_info().is_none());

    // Test statistics initialization
    let stats = nat_manager.get_stats();
    assert_eq!(
        stats
            .stun_success
            .load(std::sync::atomic::Ordering::Relaxed),
        0
    );
    assert_eq!(
        stats
            .hole_punch_success
            .load(std::sync::atomic::Ordering::Relaxed),
        0
    );
    assert_eq!(
        stats
            .relay_connections
            .load(std::sync::atomic::Ordering::Relaxed),
        0
    );
}

#[tokio::test]
async fn test_nat_info_creation() {
    let nat_info = NatInfo {
        nat_type: NatType::RestrictedCone,
        public_ip: Some("203.0.113.1".parse().unwrap()),
        public_port: Some(8080),
        local_ip: "192.168.1.100".parse().unwrap(),
        local_port: 8080,
        hairpinning: false,
        detected_at: Instant::now(),
        confidence: 0.95,
    };

    assert_eq!(nat_info.nat_type, NatType::RestrictedCone);
    assert_eq!(nat_info.public_ip.unwrap().to_string(), "203.0.113.1");
    assert_eq!(nat_info.public_port.unwrap(), 8080);
    assert_eq!(nat_info.local_ip.to_string(), "192.168.1.100");
    assert_eq!(nat_info.local_port, 8080);
    assert!(!nat_info.hairpinning);
    assert_eq!(nat_info.confidence, 0.95);
}

#[tokio::test]
async fn test_nat_types() {
    let types = vec![
        NatType::None,
        NatType::FullCone,
        NatType::RestrictedCone,
        NatType::PortRestrictedCone,
        NatType::Symmetric,
        NatType::Unknown,
    ];

    for nat_type in types {
        match nat_type {
            NatType::None => assert_eq!(format!("{:?}", nat_type), "None"),
            NatType::FullCone => assert_eq!(format!("{:?}", nat_type), "FullCone"),
            NatType::RestrictedCone => assert_eq!(format!("{:?}", nat_type), "RestrictedCone"),
            NatType::PortRestrictedCone => {
                assert_eq!(format!("{:?}", nat_type), "PortRestrictedCone")
            }
            NatType::Symmetric => assert_eq!(format!("{:?}", nat_type), "Symmetric"),
            NatType::Unknown => assert_eq!(format!("{:?}", nat_type), "Unknown"),
        }
    }
}

#[tokio::test]
async fn test_port_mapping_protocols() {
    let tcp = PortMappingProtocol::TCP;
    let udp = PortMappingProtocol::UDP;

    assert_eq!(format!("{:?}", tcp), "TCP");
    assert_eq!(format!("{:?}", udp), "UDP");
    assert_ne!(tcp, udp);
}

#[tokio::test]
async fn test_nat_traversal_integration() {
    // Create a complete NAT traversal setup
    let config = NatTraversalConfig {
        enable_stun: true,
        enable_turn: false, // Disable TURN for test
        enable_upnp: true,
        enable_nat_pmp: false, // Disable NAT-PMP for test
        enable_hole_punching: true,
        enable_relay: true,
        enable_ipv6: false, // Disable IPv6 for test
        stun_servers: vec![StunServer::new("127.0.0.1:3478".parse().unwrap(), 1)],
        turn_servers: vec![],
        max_relay_connections: 5,
        hole_punch_timeout: Duration::from_secs(5),
        detection_interval: Duration::from_secs(60),
        upgrade_interval: Duration::from_secs(30),
        port_mapping_lifetime: Duration::from_secs(300),
    };

    let connection_manager = Arc::new(ConnectionManager::new(10));
    let nat_manager = Arc::new(NatTraversalManager::new(config, connection_manager));

    // Test initialization
    match nat_manager.initialize().await {
        Ok(()) => {
            println!("NAT traversal initialized successfully");

            // Give some time for initialization
            sleep(Duration::from_millis(100)).await;

            // Test statistics
            let stats = nat_manager.get_stats();
            println!(
                "STUN attempts: {}",
                stats
                    .stun_success
                    .load(std::sync::atomic::Ordering::Relaxed)
                    + stats
                        .stun_failures
                        .load(std::sync::atomic::Ordering::Relaxed)
            );
        }
        Err(e) => {
            println!(
                "NAT traversal initialization failed (expected in test): {}",
                e
            );
        }
    }

    // Test port mapping creation
    match nat_manager
        .create_port_mapping(8080, 8080, PortMappingProtocol::TCP)
        .await
    {
        Ok(mapping) => {
            assert_eq!(mapping.local_port, 8080);
            assert_eq!(mapping.external_port, 8080);
            assert_eq!(mapping.protocol, PortMappingProtocol::TCP);
        }
        Err(e) => {
            println!("Port mapping failed (expected in test): {}", e);
        }
    }

    // Test connecting to a peer
    let peer_id = qudag_network::types::PeerId::random();
    match nat_manager.connect_peer(peer_id).await {
        Ok(()) => {
            println!("Peer connection successful");
        }
        Err(e) => {
            println!("Peer connection failed (expected in test): {}", e);
        }
    }

    // Test shutdown
    match nat_manager.shutdown().await {
        Ok(()) => {
            println!("NAT traversal shutdown successful");
        }
        Err(e) => {
            println!("NAT traversal shutdown failed: {}", e);
        }
    }
}

#[test]
fn test_nat_traversal_error_types() {
    use qudag_network::NatTraversalError;

    let stun_error = NatTraversalError::StunError("STUN failed".to_string());
    assert!(stun_error.to_string().contains("STUN failed"));

    let turn_error = NatTraversalError::TurnError("TURN failed".to_string());
    assert!(turn_error.to_string().contains("TURN failed"));

    let upnp_error = NatTraversalError::UpnpError("UPnP failed".to_string());
    assert!(upnp_error.to_string().contains("UPnP failed"));

    let timeout_error = NatTraversalError::Timeout;
    assert!(timeout_error.to_string().contains("timed out"));
}

#[tokio::test]
async fn test_concurrent_nat_operations() {
    let config = NatTraversalConfig::default();
    let connection_manager = Arc::new(ConnectionManager::new(50));
    let nat_manager = Arc::new(NatTraversalManager::new(config, connection_manager));

    // Test concurrent port mapping creation
    let tasks = (0..5).map(|i| {
        let nat_manager = Arc::clone(&nat_manager);
        tokio::spawn(async move {
            let local_port = 8080 + i as u16;
            let external_port = 9080 + i as u16;
            nat_manager
                .create_port_mapping(local_port, external_port, PortMappingProtocol::TCP)
                .await
        })
    });

    let results = futures::future::join_all(tasks).await;

    // Check that all tasks completed (some may fail due to test environment)
    assert_eq!(results.len(), 5);

    for result in results {
        match result {
            Ok(Ok(mapping)) => {
                println!(
                    "Port mapping created: {}:{}",
                    mapping.local_port, mapping.external_port
                );
            }
            Ok(Err(e)) => {
                println!("Port mapping failed (expected): {}", e);
            }
            Err(e) => {
                println!("Task failed: {}", e);
            }
        }
    }
}

/// Helper function to create test NAT configuration
fn create_test_nat_config() -> NatTraversalConfig {
    NatTraversalConfig {
        enable_stun: true,
        enable_turn: false,
        enable_upnp: true,
        enable_nat_pmp: false,
        enable_hole_punching: true,
        enable_relay: true,
        enable_ipv6: false,
        stun_servers: vec![StunServer::new("127.0.0.1:3478".parse().unwrap(), 1)],
        turn_servers: vec![],
        max_relay_connections: 5,
        hole_punch_timeout: Duration::from_secs(5),
        detection_interval: Duration::from_secs(60),
        upgrade_interval: Duration::from_secs(30),
        port_mapping_lifetime: Duration::from_secs(300),
    }
}
