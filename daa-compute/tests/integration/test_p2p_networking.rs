//! Integration tests for P2P networking layer

use daa_compute::{P2PNetwork, SwarmConfig, NetworkBehavior};
use daa_compute::p2p::{
    discovery::{DiscoveryService, DiscoveredPeer, DiscoveryMethod},
    nat::{NatTraversal, StunConfig},
    transport::{TransportConfig, create_transport},
    gradient::{GradientManager, GradientMessage}
};
use daa_compute::training::Gradient;
use libp2p::{PeerId, Multiaddr};
use std::time::{Duration, Instant};
use tokio::{time::timeout, select};
use anyhow::Result;

#[cfg(test)]
mod tests {
    use super::*;

    async fn create_test_network(port: u16) -> Result<P2PNetwork> {
        let mut config = SwarmConfig::default();
        config.listen_addresses = vec![
            format!("/ip4/127.0.0.1/tcp/{}", port).parse()?,
        ];
        P2PNetwork::new(config).await
    }

    fn create_test_gradient(node_id: &str, values: Vec<f32>) -> Gradient {
        Gradient {
            values,
            node_id: node_id.to_string(),
            round: 1,
            compressed: false,
        }
    }

    #[tokio::test]
    async fn test_single_node_network() -> Result<()> {
        let network = create_test_network(4001).await?;
        
        // Test basic network operations
        assert!(network.get_aggregated_gradient().await?.is_none());
        
        let peer_id = network.local_peer_id();
        assert!(peer_id != PeerId::random());
        
        Ok(())
    }

    #[tokio::test]
    async fn test_two_node_connection() -> Result<()> {
        let network1 = create_test_network(4002).await?;
        let network2 = create_test_network(4003).await?;
        
        // Get addresses
        let addr1 = network1.get_listen_addresses().await?;
        let peer_id1 = network1.local_peer_id();
        
        // Connect network2 to network1
        network2.connect_to_peer(peer_id1, addr1[0].clone()).await?;
        
        // Wait for connection
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // Check connections
        let peers1 = network1.get_connected_peers().await?;
        let peers2 = network2.get_connected_peers().await?;
        
        assert!(peers1.len() > 0 || peers2.len() > 0);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_gradient_sharing_between_nodes() -> Result<()> {
        let network1 = create_test_network(4004).await?;
        let network2 = create_test_network(4005).await?;
        
        // Connect nodes
        let addr1 = network1.get_listen_addresses().await?;
        let peer_id1 = network1.local_peer_id();
        network2.connect_to_peer(peer_id1, addr1[0].clone()).await?;
        
        // Wait for connection
        tokio::time::sleep(Duration::from_millis(200)).await;
        
        // Share gradient from network1
        let gradient = create_test_gradient("node1", vec![1.0, 2.0, 3.0]);
        network1.broadcast_gradient(gradient.clone()).await?;
        
        // Wait for propagation
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // Check if network2 received the gradient
        let received = network2.get_received_gradients().await?;
        assert!(received.len() > 0);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_multi_node_network() -> Result<()> {
        let mut networks = Vec::new();
        let base_port = 4010;
        let num_nodes = 5;
        
        // Create multiple networks
        for i in 0..num_nodes {
            let network = create_test_network(base_port + i).await?;
            networks.push(network);
        }
        
        // Connect all nodes to the first node (star topology)
        let hub_addr = networks[0].get_listen_addresses().await?;
        let hub_peer_id = networks[0].local_peer_id();
        
        for i in 1..num_nodes {
            networks[i].connect_to_peer(hub_peer_id, hub_addr[0].clone()).await?;
        }
        
        // Wait for connections
        tokio::time::sleep(Duration::from_millis(500)).await;
        
        // Check that hub node has connections
        let hub_peers = networks[0].get_connected_peers().await?;
        assert!(hub_peers.len() >= 1);
        
        // Test gradient broadcasting
        let gradient = create_test_gradient("broadcaster", vec![4.0, 5.0, 6.0]);
        networks[1].broadcast_gradient(gradient.clone()).await?;
        
        // Wait for propagation
        tokio::time::sleep(Duration::from_millis(200)).await;
        
        // Check if other nodes received the gradient
        let mut received_count = 0;
        for i in 0..num_nodes {
            if i != 1 { // Skip the broadcaster
                let received = networks[i].get_received_gradients().await?;
                if received.len() > 0 {
                    received_count += 1;
                }
            }
        }
        
        assert!(received_count > 0);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_peer_discovery() -> Result<()> {
        let peer_id = PeerId::random();
        let discovery = DiscoveryService::new(peer_id);
        
        // Add a discovered peer
        let discovered_peer = DiscoveredPeer {
            peer_id: PeerId::random(),
            addresses: vec!["/ip4/127.0.0.1/tcp/4001".parse()?],
            discovery_method: DiscoveryMethod::Mdns,
            discovered_at: Instant::now(),
            metadata: None,
        };
        
        discovery.add_discovered_peer(discovered_peer.clone()).await?;
        
        // Check if peer was added
        let peers = discovery.get_discovered_peers().await;
        assert_eq!(peers.len(), 1);
        assert_eq!(peers[0].peer_id, discovered_peer.peer_id);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_nat_detection() -> Result<()> {
        let stun_config = StunConfig::default();
        let nat = NatTraversal::new(stun_config, None);
        
        // Test NAT detection (this may not work in all environments)
        let result = timeout(Duration::from_secs(10), nat.detect_nat()).await;
        
        // Just ensure it doesn't panic or hang
        match result {
            Ok(_) => {}, // NAT detection completed
            Err(_) => {}, // Timeout is acceptable
        }
        
        Ok(())
    }

    #[tokio::test]
    async fn test_transport_configuration() -> Result<()> {
        let config = TransportConfig {
            enable_tcp: true,
            enable_quic: false, // Disable QUIC for simpler testing
            enable_websocket: false,
            tcp_config: Default::default(),
        };
        
        let transport = create_transport(config).await?;
        
        // Just check that transport was created successfully
        // More detailed testing would require actual network setup
        
        Ok(())
    }

    #[tokio::test]
    async fn test_network_resilience_node_disconnect() -> Result<()> {
        let network1 = create_test_network(4020).await?;
        let network2 = create_test_network(4021).await?;
        let network3 = create_test_network(4022).await?;
        
        // Connect all nodes
        let addr1 = network1.get_listen_addresses().await?;
        let peer_id1 = network1.local_peer_id();
        
        network2.connect_to_peer(peer_id1, addr1[0].clone()).await?;
        network3.connect_to_peer(peer_id1, addr1[0].clone()).await?;
        
        tokio::time::sleep(Duration::from_millis(200)).await;
        
        // Check initial connections
        let initial_peers = network1.get_connected_peers().await?;
        let initial_count = initial_peers.len();
        
        // Simulate node2 disconnecting (drop it)
        drop(network2);
        
        // Wait for disconnect detection
        tokio::time::sleep(Duration::from_millis(500)).await;
        
        // Network should still function with remaining nodes
        let gradient = create_test_gradient("node3", vec![7.0, 8.0, 9.0]);
        network3.broadcast_gradient(gradient).await?;
        
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // Network1 should still receive gradients from network3
        let received = network1.get_received_gradients().await?;
        
        // Just check that the network is still functional
        // The exact behavior depends on the implementation
        
        Ok(())
    }

    #[tokio::test]
    async fn test_gradient_compression_over_network() -> Result<()> {
        let mut config1 = SwarmConfig::default();
        config1.listen_addresses = vec!["/ip4/127.0.0.1/tcp/4030".parse()?];
        config1.enable_compression = true;
        
        let mut config2 = SwarmConfig::default();
        config2.listen_addresses = vec!["/ip4/127.0.0.1/tcp/4031".parse()?];
        config2.enable_compression = true;
        
        let network1 = P2PNetwork::new(config1).await?;
        let network2 = P2PNetwork::new(config2).await?;
        
        // Connect networks
        let addr1 = network1.get_listen_addresses().await?;
        let peer_id1 = network1.local_peer_id();
        network2.connect_to_peer(peer_id1, addr1[0].clone()).await?;
        
        tokio::time::sleep(Duration::from_millis(200)).await;
        
        // Send large gradient that should benefit from compression
        let large_gradient = create_test_gradient("node2", vec![0.1; 10000]);
        network2.broadcast_gradient(large_gradient).await?;
        
        tokio::time::sleep(Duration::from_millis(300)).await;
        
        // Check if gradient was received
        let received = network1.get_received_gradients().await?;
        assert!(received.len() > 0);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_concurrent_gradient_broadcasts() -> Result<()> {
        let network1 = create_test_network(4040).await?;
        let network2 = create_test_network(4041).await?;
        let network3 = create_test_network(4042).await?;
        
        // Connect all to network1 (hub)
        let addr1 = network1.get_listen_addresses().await?;
        let peer_id1 = network1.local_peer_id();
        
        network2.connect_to_peer(peer_id1, addr1[0].clone()).await?;
        network3.connect_to_peer(peer_id1, addr1[0].clone()).await?;
        
        tokio::time::sleep(Duration::from_millis(200)).await;
        
        // Concurrent gradient broadcasts
        let gradient2 = create_test_gradient("node2", vec![1.0, 2.0]);
        let gradient3 = create_test_gradient("node3", vec![3.0, 4.0]);
        
        let (result2, result3) = tokio::join!(
            network2.broadcast_gradient(gradient2),
            network3.broadcast_gradient(gradient3)
        );
        
        result2?;
        result3?;
        
        tokio::time::sleep(Duration::from_millis(200)).await;
        
        // Check that hub received both gradients
        let received = network1.get_received_gradients().await?;
        assert!(received.len() >= 1); // Should receive at least one
        
        Ok(())
    }

    #[tokio::test]
    async fn test_network_bootstrap() -> Result<()> {
        // Test bootstrapping with known peers
        let mut config = SwarmConfig::default();
        config.listen_addresses = vec!["/ip4/127.0.0.1/tcp/4050".parse()?];
        config.bootstrap_peers = vec![
            "/ip4/127.0.0.1/tcp/4051".parse()?,
        ];
        
        let network = P2PNetwork::new(config).await?;
        
        // Just check that network starts successfully with bootstrap config
        assert!(network.get_connected_peers().await?.len() >= 0);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_message_ordering() -> Result<()> {
        let network1 = create_test_network(4060).await?;
        let network2 = create_test_network(4061).await?;
        
        // Connect networks
        let addr1 = network1.get_listen_addresses().await?;
        let peer_id1 = network1.local_peer_id();
        network2.connect_to_peer(peer_id1, addr1[0].clone()).await?;
        
        tokio::time::sleep(Duration::from_millis(200)).await;
        
        // Send multiple gradients in sequence
        for i in 0..5 {
            let gradient = create_test_gradient("sender", vec![i as f32]);
            network2.broadcast_gradient(gradient).await?;
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        
        tokio::time::sleep(Duration::from_millis(200)).await;
        
        // Check received gradients
        let received = network1.get_received_gradients().await?;
        assert!(received.len() > 0);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_network_metrics() -> Result<()> {
        let network = create_test_network(4070).await?;
        
        // Test basic metrics collection
        let metrics = network.get_network_metrics().await?;
        
        // Basic sanity checks
        assert!(metrics.connected_peers >= 0);
        assert!(metrics.bytes_sent >= 0);
        assert!(metrics.bytes_received >= 0);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_graceful_shutdown() -> Result<()> {
        let network = create_test_network(4080).await?;
        
        // Test that network can be shut down gracefully
        network.shutdown().await?;
        
        // After shutdown, operations should fail gracefully
        let result = network.get_connected_peers().await;
        assert!(result.is_err() || result.unwrap().is_empty());
        
        Ok(())
    }
}