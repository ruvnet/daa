//! Integration tests for P2P communication layer

use daa_compute::{P2PNetwork, SwarmConfig};
use libp2p::{PeerId, Multiaddr};
use tokio;
use anyhow::Result;

#[tokio::test]
async fn test_network_creation() -> Result<()> {
    let config = SwarmConfig::default();
    let network = P2PNetwork::new(config).await?;
    assert!(network.get_aggregated_gradient().await?.is_none());
    Ok(())
}

#[tokio::test]
async fn test_gradient_compression() -> Result<()> {
    use daa_compute::p2p::compression::{CompressionMethod, GradientCompressor};
    
    let gradient = vec![0.1_f32; 1000];
    let compressor = GradientCompressor::new(CompressionMethod::Zstd { level: 3 });
    
    let compressed = compressor.compress_sparse(&gradient)?;
    assert!(compressed.len() < gradient.len() * 4); // Should be compressed
    
    Ok(())
}

#[tokio::test]
async fn test_all_reduce_algorithms() -> Result<()> {
    use daa_compute::p2p::gradient::{GradientManager, AllReduceAlgorithm};
    use std::collections::HashMap;
    
    let peer_id = PeerId::random();
    let manager = GradientManager::new(peer_id, 3);
    
    // Test data
    let mut gradients = HashMap::new();
    gradients.insert(PeerId::random(), vec![1.0, 2.0, 3.0]);
    gradients.insert(PeerId::random(), vec![4.0, 5.0, 6.0]);
    gradients.insert(peer_id, vec![7.0, 8.0, 9.0]);
    
    // Test ring all-reduce
    let result = manager.ring_allreduce(&gradients).await?;
    assert_eq!(result.len(), 3);
    assert!((result[0] - 4.0).abs() < 0.001); // (1+4+7)/3 = 4
    
    Ok(())
}

#[tokio::test]
async fn test_nat_detection() -> Result<()> {
    use daa_compute::p2p::nat::{NatTraversal, StunConfig};
    
    let stun_config = StunConfig::default();
    let nat = NatTraversal::new(stun_config, None);
    
    // This would require actual STUN servers to work
    // Just test that it doesn't panic
    let _ = nat.detect_nat().await;
    
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::test]
async fn test_peer_discovery() -> Result<()> {
    use daa_compute::p2p::discovery::{DiscoveryService, DiscoveredPeer, DiscoveryMethod};
    use std::time::Instant;
    
    let peer_id = PeerId::random();
    let discovery = DiscoveryService::new(peer_id);
    
    let peer = DiscoveredPeer {
        peer_id: PeerId::random(),
        addresses: vec!["/ip4/127.0.0.1/tcp/4001".parse()?],
        discovery_method: DiscoveryMethod::Mdns,
        discovered_at: Instant::now(),
        metadata: None,
    };
    
    discovery.add_discovered_peer(peer).await?;
    
    let peers = discovery.get_discovered_peers().await;
    assert_eq!(peers.len(), 1);
    
    Ok(())
}