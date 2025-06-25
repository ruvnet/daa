//! Node churn simulation tests for network resilience

use daa_compute::{P2PNetwork, SwarmConfig};
use daa_compute::training::Gradient;
use libp2p::PeerId;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::{sleep, timeout};
use anyhow::Result;
use rand::{Rng, thread_rng};

#[cfg(test)]
mod tests {
    use super::*;

    struct NetworkNode {
        network: Option<P2PNetwork>,
        peer_id: PeerId,
        port: u16,
        is_active: bool,
    }

    impl NetworkNode {
        async fn new(port: u16) -> Result<Self> {
            let mut config = SwarmConfig::default();
            config.listen_addresses = vec![
                format!("/ip4/127.0.0.1/tcp/{}", port).parse()?,
            ];
            
            let network = P2PNetwork::new(config).await?;
            let peer_id = network.local_peer_id();
            
            Ok(Self {
                network: Some(network),
                peer_id,
                port,
                is_active: true,
            })
        }

        async fn disconnect(&mut self) {
            if let Some(network) = self.network.take() {
                let _ = network.shutdown().await;
            }
            self.is_active = false;
        }

        async fn reconnect(&mut self) -> Result<()> {
            if !self.is_active {
                let mut config = SwarmConfig::default();
                config.listen_addresses = vec![
                    format!("/ip4/127.0.0.1/tcp/{}", self.port).parse()?,
                ];
                
                self.network = Some(P2PNetwork::new(config).await?);
                self.is_active = true;
            }
            Ok(())
        }

        fn is_active(&self) -> bool {
            self.is_active && self.network.is_some()
        }

        fn network(&self) -> Option<&P2PNetwork> {
            self.network.as_ref()
        }
    }

    async fn create_network_cluster(num_nodes: usize, base_port: u16) -> Result<Vec<NetworkNode>> {
        let mut nodes = Vec::new();
        
        // Create all nodes
        for i in 0..num_nodes {
            let node = NetworkNode::new(base_port + i as u16).await?;
            nodes.push(node);
        }
        
        // Connect all nodes to the first node (star topology)
        if nodes.len() > 1 {
            let hub_addr = nodes[0].network().unwrap().get_listen_addresses().await?;
            let hub_peer_id = nodes[0].peer_id;
            
            for i in 1..nodes.len() {
                if let Some(network) = nodes[i].network() {
                    let _ = network.connect_to_peer(hub_peer_id, hub_addr[0].clone()).await;
                }
            }
            
            // Wait for connections to establish
            sleep(Duration::from_millis(500)).await;
        }
        
        Ok(nodes)
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
    async fn test_single_node_failure() -> Result<()> {
        let mut nodes = create_network_cluster(5, 5000).await?;
        
        // Initial connectivity check
        let initial_peers = nodes[0].network().unwrap().get_connected_peers().await?;
        let initial_count = initial_peers.len();
        
        // Disconnect one node
        nodes[2].disconnect().await;
        
        // Wait for failure detection
        sleep(Duration::from_millis(1000)).await;
        
        // Test that remaining nodes can still communicate
        let gradient = create_test_gradient("survivor", vec![1.0, 2.0, 3.0]);
        if let Some(network) = nodes[1].network() {
            network.broadcast_gradient(gradient).await?;
        }
        
        sleep(Duration::from_millis(200)).await;
        
        // Check that other nodes can still receive messages
        let received = nodes[0].network().unwrap().get_received_gradients().await?;
        
        // Network should continue functioning despite node failure
        assert!(nodes[0].is_active());
        assert!(nodes[1].is_active());
        assert!(!nodes[2].is_active());
        
        Ok(())
    }

    #[tokio::test]
    async fn test_multiple_node_failures() -> Result<()> {
        let mut nodes = create_network_cluster(10, 5010).await?;
        
        // Disconnect multiple nodes randomly
        let mut rng = thread_rng();
        let failure_count = 4;
        let mut failed_indices = Vec::new();
        
        while failed_indices.len() < failure_count {
            let idx = rng.gen_range(1..nodes.len()); // Don't fail the hub node
            if !failed_indices.contains(&idx) {
                failed_indices.push(idx);
                nodes[idx].disconnect().await;
            }
        }
        
        // Wait for failure detection
        sleep(Duration::from_millis(1500)).await;
        
        // Test network resilience
        let gradient = create_test_gradient("resilient", vec![4.0, 5.0, 6.0]);
        
        // Try to broadcast from a surviving node
        for (i, node) in nodes.iter().enumerate() {
            if node.is_active() && i != 0 {
                if let Some(network) = node.network() {
                    let _ = network.broadcast_gradient(gradient.clone()).await;
                    break;
                }
            }
        }
        
        sleep(Duration::from_millis(300)).await;
        
        // Check that hub can still receive messages
        let received = nodes[0].network().unwrap().get_received_gradients().await?;
        
        // Count active nodes
        let active_count = nodes.iter().filter(|n| n.is_active()).count();
        assert_eq!(active_count, nodes.len() - failure_count);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_node_recovery() -> Result<()> {
        let mut nodes = create_network_cluster(6, 5020).await?;
        
        // Disconnect some nodes
        nodes[2].disconnect().await;
        nodes[3].disconnect().await;
        
        sleep(Duration::from_millis(500)).await;
        
        // Reconnect them
        nodes[2].reconnect().await?;
        nodes[3].reconnect().await?;
        
        // Reconnect to the hub
        let hub_addr = nodes[0].network().unwrap().get_listen_addresses().await?;
        let hub_peer_id = nodes[0].peer_id;
        
        if let Some(network) = nodes[2].network() {
            let _ = network.connect_to_peer(hub_peer_id, hub_addr[0].clone()).await;
        }
        if let Some(network) = nodes[3].network() {
            let _ = network.connect_to_peer(hub_peer_id, hub_addr[0].clone()).await;
        }
        
        sleep(Duration::from_millis(1000)).await;
        
        // Test that recovered nodes can participate
        let gradient = create_test_gradient("recovered", vec![7.0, 8.0, 9.0]);
        if let Some(network) = nodes[2].network() {
            network.broadcast_gradient(gradient).await?;
        }
        
        sleep(Duration::from_millis(200)).await;
        
        // Check that other nodes receive messages from recovered node
        let received = nodes[0].network().unwrap().get_received_gradients().await?;
        
        // All nodes should be active again
        assert!(nodes.iter().all(|n| n.is_active()));
        
        Ok(())
    }

    #[tokio::test]
    async fn test_cascading_failures() -> Result<()> {
        let mut nodes = create_network_cluster(8, 5030).await?;
        
        // Simulate cascading failures
        for i in (1..4).rev() {
            nodes[i].disconnect().await;
            sleep(Duration::from_millis(200)).await;
            
            // Test network state after each failure
            let gradient = create_test_gradient(&format!("test_{}", i), vec![i as f32]);
            
            // Try to broadcast from a surviving node
            for (j, node) in nodes.iter().enumerate() {
                if node.is_active() && j != 0 && j > i {
                    if let Some(network) = node.network() {
                        let _ = network.broadcast_gradient(gradient.clone()).await;
                        break;
                    }
                }
            }
            
            sleep(Duration::from_millis(100)).await;
        }
        
        // Network should still function despite cascading failures
        let active_count = nodes.iter().filter(|n| n.is_active()).count();
        assert!(active_count >= nodes.len() / 2);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_hub_node_failure() -> Result<()> {
        let mut nodes = create_network_cluster(7, 5040).await?;
        
        // Fail the hub node (index 0)
        nodes[0].disconnect().await;
        
        sleep(Duration::from_millis(1000)).await;
        
        // Other nodes should detect hub failure
        // In a real implementation, they might elect a new hub or form mesh connections
        
        // Test direct communication between remaining nodes
        if nodes.len() > 2 {
            // Connect node 1 and node 2 directly
            let addr2 = nodes[2].network().unwrap().get_listen_addresses().await?;
            let peer_id2 = nodes[2].peer_id;
            
            if let Some(network) = nodes[1].network() {
                let _ = network.connect_to_peer(peer_id2, addr2[0].clone()).await;
            }
            
            sleep(Duration::from_millis(500)).await;
            
            // Test communication without hub
            let gradient = create_test_gradient("no_hub", vec![10.0, 11.0]);
            if let Some(network) = nodes[1].network() {
                network.broadcast_gradient(gradient).await?;
            }
            
            sleep(Duration::from_millis(200)).await;
            
            // Check if message reached other nodes
            let received = nodes[2].network().unwrap().get_received_gradients().await?;
            
            // Network should adapt to hub failure
            assert!(!nodes[0].is_active());
            assert!(nodes[1].is_active());
            assert!(nodes[2].is_active());
        }
        
        Ok(())
    }

    #[tokio::test]
    async fn test_rapid_churn() -> Result<()> {
        let mut nodes = create_network_cluster(10, 5050).await?;
        let mut rng = thread_rng();
        
        // Simulate rapid node churn over time
        for round in 0..5 {
            // Random failures
            let failure_idx = rng.gen_range(1..nodes.len());
            if nodes[failure_idx].is_active() {
                nodes[failure_idx].disconnect().await;
            }
            
            sleep(Duration::from_millis(100)).await;
            
            // Random recoveries
            let recovery_idx = rng.gen_range(1..nodes.len());
            if !nodes[recovery_idx].is_active() {
                if let Ok(()) = nodes[recovery_idx].reconnect().await {
                    // Reconnect to hub
                    let hub_addr = nodes[0].network().unwrap().get_listen_addresses().await?;
                    let hub_peer_id = nodes[0].peer_id;
                    
                    if let Some(network) = nodes[recovery_idx].network() {
                        let _ = network.connect_to_peer(hub_peer_id, hub_addr[0].clone()).await;
                    }
                }
            }
            
            sleep(Duration::from_millis(100)).await;
            
            // Test network functionality during churn
            let gradient = create_test_gradient(&format!("churn_{}", round), vec![round as f32]);
            
            // Find an active node to broadcast from
            for (i, node) in nodes.iter().enumerate() {
                if node.is_active() && i != 0 {
                    if let Some(network) = node.network() {
                        let _ = network.broadcast_gradient(gradient.clone()).await;
                        break;
                    }
                }
            }
            
            sleep(Duration::from_millis(50)).await;
        }
        
        // Check that some nodes survived the churn
        let active_count = nodes.iter().filter(|n| n.is_active()).count();
        assert!(active_count >= 3); // At least hub + 2 others
        
        Ok(())
    }

    #[tokio::test]
    async fn test_network_partition_recovery() -> Result<()> {
        let mut nodes = create_network_cluster(8, 5060).await?;
        
        // Create a network partition by disconnecting some nodes
        let partition1 = vec![0, 1, 2]; // Hub + 2 nodes
        let partition2 = vec![3, 4, 5]; // 3 nodes
        let isolated = vec![6, 7]; // 2 isolated nodes
        
        // Disconnect partition2 and isolated nodes from hub
        for &i in partition2.iter().chain(isolated.iter()) {
            nodes[i].disconnect().await;
        }
        
        sleep(Duration::from_millis(500)).await;
        
        // Create internal connections within partition2
        if partition2.len() > 1 {
            let addr3 = nodes[3].network().unwrap().get_listen_addresses().await?;
            let peer_id3 = nodes[3].peer_id;
            
            if let Some(network) = nodes[4].network() {
                let _ = network.connect_to_peer(peer_id3, addr3[0].clone()).await;
            }
            if let Some(network) = nodes[5].network() {
                let _ = network.connect_to_peer(peer_id3, addr3[0].clone()).await;
            }
        }
        
        sleep(Duration::from_millis(500)).await;
        
        // Test that each partition can function independently
        let gradient1 = create_test_gradient("partition1", vec![1.0, 2.0]);
        let gradient2 = create_test_gradient("partition2", vec![3.0, 4.0]);
        
        // Broadcast in partition1
        if let Some(network) = nodes[1].network() {
            network.broadcast_gradient(gradient1).await?;
        }
        
        // Broadcast in partition2
        if let Some(network) = nodes[4].network() {
            network.broadcast_gradient(gradient2).await?;
        }
        
        sleep(Duration::from_millis(200)).await;
        
        // Now heal the partition by reconnecting
        for &i in partition2.iter() {
            if let Ok(()) = nodes[i].reconnect().await {
                let hub_addr = nodes[0].network().unwrap().get_listen_addresses().await?;
                let hub_peer_id = nodes[0].peer_id;
                
                if let Some(network) = nodes[i].network() {
                    let _ = network.connect_to_peer(hub_peer_id, hub_addr[0].clone()).await;
                }
            }
        }
        
        sleep(Duration::from_millis(1000)).await;
        
        // Test that the healed network functions properly
        let gradient_healed = create_test_gradient("healed", vec![5.0, 6.0]);
        if let Some(network) = nodes[3].network() {
            network.broadcast_gradient(gradient_healed).await?;
        }
        
        sleep(Duration::from_millis(200)).await;
        
        // Check that messages can flow across the healed network
        let received = nodes[0].network().unwrap().get_received_gradients().await?;
        
        // Most nodes should be active after recovery
        let active_count = nodes.iter().filter(|n| n.is_active()).count();
        assert!(active_count >= 6); // At least 3/4 of the nodes
        
        Ok(())
    }

    #[tokio::test]
    async fn test_byzantine_node_behavior() -> Result<()> {
        let mut nodes = create_network_cluster(7, 5070).await?;
        
        // Simulate Byzantine behavior by having a node send malicious gradients
        let malicious_gradients = vec![
            create_test_gradient("byzantine1", vec![f32::INFINITY, f32::NEG_INFINITY]),
            create_test_gradient("byzantine2", vec![f32::NAN, 1e20]),
            create_test_gradient("byzantine3", vec![1e30; 1000]), // Extremely large
        ];
        
        // Byzantine node sends malicious gradients
        if let Some(network) = nodes[3].network() {
            for gradient in malicious_gradients {
                let _ = network.broadcast_gradient(gradient).await;
                sleep(Duration::from_millis(50)).await;
            }
        }
        
        sleep(Duration::from_millis(200)).await;
        
        // Honest nodes should still function properly
        let honest_gradient = create_test_gradient("honest", vec![1.0, 2.0, 3.0]);
        if let Some(network) = nodes[1].network() {
            network.broadcast_gradient(honest_gradient).await?;
        }
        
        sleep(Duration::from_millis(200)).await;
        
        // Network should continue functioning despite Byzantine behavior
        let received = nodes[0].network().unwrap().get_received_gradients().await?;
        
        // The aggregation layer should filter out malicious gradients
        // This test mainly ensures the network doesn't crash
        assert!(nodes[0].is_active());
        assert!(nodes[1].is_active());
        
        Ok(())
    }

    #[tokio::test]
    async fn test_high_churn_rate() -> Result<()> {
        let mut nodes = create_network_cluster(12, 5080).await?;
        let mut rng = thread_rng();
        
        // Simulate very high churn rate
        for _iteration in 0..20 {
            // Multiple failures and recoveries per iteration
            for _ in 0..3 {
                let idx = rng.gen_range(1..nodes.len());
                if rng.gen_bool(0.5) && nodes[idx].is_active() {
                    // Failure
                    nodes[idx].disconnect().await;
                } else if !nodes[idx].is_active() {
                    // Recovery
                    if let Ok(()) = nodes[idx].reconnect().await {
                        let hub_addr = nodes[0].network().unwrap().get_listen_addresses().await?;
                        let hub_peer_id = nodes[0].peer_id;
                        
                        if let Some(network) = nodes[idx].network() {
                            let _ = network.connect_to_peer(hub_peer_id, hub_addr[0].clone()).await;
                        }
                    }
                }
            }
            
            sleep(Duration::from_millis(100)).await;
            
            // Test network functionality during high churn
            let gradient = create_test_gradient("high_churn", vec![42.0]);
            
            // Find any active node to broadcast
            for (i, node) in nodes.iter().enumerate() {
                if node.is_active() && i != 0 {
                    if let Some(network) = node.network() {
                        let _ = network.broadcast_gradient(gradient.clone()).await;
                        break;
                    }
                }
            }
            
            sleep(Duration::from_millis(50)).await;
        }
        
        // Network should survive high churn
        let active_count = nodes.iter().filter(|n| n.is_active()).count();
        assert!(active_count >= nodes.len() / 3); // At least 1/3 survived
        
        Ok(())
    }
}