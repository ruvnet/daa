//! Integration tests for P2P networking layer
//!
//! These run real networks over loopback TCP. Every network listens on an
//! OS-assigned port, and waits are deadline-bounded polls (see
//! `crate::common::net`), so the tests do not depend on fixed ports or timing.

use crate::common::net::{self, loopback_network};
use anyhow::Result;
use daa_compute::p2p::{
    discovery::{DiscoveredPeer, DiscoveryMethod, DiscoveryService},
    nat::{NatTraversal, StunConfig},
    transport::{create_transport, TransportConfig},
};
use daa_compute::{P2PNetwork, SwarmConfig};
use libp2p::PeerId;
use std::time::{Duration, Instant};
use tokio::time::timeout;

#[cfg(test)]
mod tests {
    use super::*;

    /// Gradients travel int8-quantised, so compare with a tolerance.
    fn assert_close(actual: &[f32], expected: &[f32]) {
        crate::common::assert_approx_eq(actual, expected, 0.05);
    }

    #[tokio::test]
    async fn test_single_node_network() -> Result<()> {
        let network = loopback_network().await?;

        // Test basic network operations
        assert!(network.get_aggregated_gradient().await?.is_none());
        assert!(network.connected_peers().is_empty());

        let peer_id = network.local_peer_id();
        assert!(peer_id != PeerId::random());

        Ok(())
    }

    #[tokio::test]
    async fn test_two_node_connection() -> Result<()> {
        let mut network1 = loopback_network().await?;
        let mut network2 = loopback_network().await?;

        net::connect_star(&mut network1, &mut [&mut network2]).await?;

        assert!(network2
            .connected_peers()
            .contains(&network1.local_peer_id()));
        assert!(network1
            .connected_peers()
            .contains(&network2.local_peer_id()));

        Ok(())
    }

    #[tokio::test]
    async fn test_gradient_sharing_between_nodes() -> Result<()> {
        let mut network1 = loopback_network().await?;
        let mut network2 = loopback_network().await?;
        net::connect_star(&mut network1, &mut [&mut network2]).await?;

        let mut networks = [&mut network1, &mut network2];
        net::broadcast(&mut networks, 0, vec![1.0, 2.0, 3.0]).await?;

        let received = net::await_gradients(&mut networks, 1, 1).await?;
        assert_close(&received[0], &[1.0, 2.0, 3.0]);

        Ok(())
    }

    #[tokio::test]
    async fn test_multi_node_network() -> Result<()> {
        let num_nodes = 5;
        let mut networks = Vec::new();
        for _ in 0..num_nodes {
            networks.push(loopback_network().await?);
        }

        // Connect all nodes to the first node (star topology)
        let (hub, spokes) = networks.split_first_mut().expect("at least one node");
        let mut spoke_refs: Vec<&mut P2PNetwork> = spokes.iter_mut().collect();
        net::connect_star(hub, &mut spoke_refs).await?;
        assert_eq!(hub.connected_peers().len(), num_nodes - 1);

        // Broadcast from a spoke; the hub must receive it
        let mut all: Vec<&mut P2PNetwork> = networks.iter_mut().collect();
        net::broadcast(&mut all, 1, vec![4.0, 5.0, 6.0]).await?;
        let received = net::await_gradients(&mut all, 0, 1).await?;
        assert_close(&received[0], &[4.0, 5.0, 6.0]);

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

        discovery
            .add_discovered_peer(discovered_peer.clone())
            .await?;

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

        // NAT detection may not work in all environments; it must just not
        // panic or hang (a timeout is acceptable).
        let _ = timeout(Duration::from_secs(10), nat.detect_nat()).await;

        Ok(())
    }

    #[tokio::test]
    async fn test_transport_configuration() -> Result<()> {
        let keypair = libp2p::identity::Keypair::generate_ed25519();
        let config = TransportConfig {
            enable_tcp: true,
            enable_websocket: false, // TCP only for simpler testing
            enable_webrtc: false,
            enable_relay: false,
        };

        // Just check that transport was created successfully; exercising it
        // needs a swarm, which the network tests above cover.
        let _transport = create_transport(&keypair, config)?;

        Ok(())
    }

    #[tokio::test]
    async fn test_network_resilience_node_disconnect() -> Result<()> {
        let mut network1 = loopback_network().await?;
        let mut network2 = loopback_network().await?;
        let mut network3 = loopback_network().await?;
        net::connect_star(&mut network1, &mut [&mut network2, &mut network3]).await?;
        assert_eq!(network1.connected_peers().len(), 2);

        // Simulate node2 disconnecting (drop it)
        let peer_id2 = network2.local_peer_id();
        drop(network2);

        // Wait for disconnect detection
        let start = Instant::now();
        while network1.connected_peers().contains(&peer_id2) {
            assert!(start.elapsed() < net::DEADLINE, "disconnect not detected");
            net::pump(
                &mut [&mut network1, &mut network3],
                Duration::from_millis(50),
            )
            .await?;
        }

        // Network should still function with remaining nodes
        let mut networks = [&mut network1, &mut network3];
        net::broadcast(&mut networks, 1, vec![7.0, 8.0, 9.0]).await?;
        let received = net::await_gradients(&mut networks, 0, 1).await?;
        assert_close(&received[0], &[7.0, 8.0, 9.0]);

        Ok(())
    }

    #[tokio::test]
    async fn test_gradient_compression_over_network() -> Result<()> {
        let config = || -> Result<SwarmConfig> {
            Ok(SwarmConfig {
                listen_addresses: vec!["/ip4/127.0.0.1/tcp/0".parse()?],
                compression_level: 9, // maximum zstd compression
                ..SwarmConfig::default()
            })
        };
        let mut network1 = P2PNetwork::new(config()?).await?;
        let mut network2 = P2PNetwork::new(config()?).await?;
        net::connect_star(&mut network1, &mut [&mut network2]).await?;

        // Send large gradient that should benefit from compression
        let mut networks = [&mut network1, &mut network2];
        net::broadcast(&mut networks, 1, vec![0.1; 10_000]).await?;

        let received = net::await_gradients(&mut networks, 0, 1).await?;
        assert_close(&received[0], &[0.1; 10_000]);

        Ok(())
    }

    #[tokio::test]
    async fn test_concurrent_gradient_broadcasts() -> Result<()> {
        let mut network1 = loopback_network().await?;
        let mut network2 = loopback_network().await?;
        let mut network3 = loopback_network().await?;
        net::connect_star(&mut network1, &mut [&mut network2, &mut network3]).await?;

        // Concurrent gradient broadcasts, retried until both spokes see a
        // subscriber for the gradient topic.
        let start = Instant::now();
        loop {
            let (result2, result3) = tokio::join!(
                network2.broadcast_gradient(vec![1.0, 2.0]),
                network3.broadcast_gradient(vec![3.0, 4.0])
            );
            match (result2, result3) {
                (Ok(()), Ok(())) => break,
                (Err(e), _) | (_, Err(e)) if start.elapsed() > net::DEADLINE => return Err(e),
                _ => {
                    net::pump(
                        &mut [&mut network1, &mut network2, &mut network3],
                        Duration::from_millis(100),
                    )
                    .await?
                }
            }
        }

        // Check that hub received both gradients
        let mut networks = [&mut network1, &mut network2, &mut network3];
        let mut received = net::await_gradients(&mut networks, 0, 2).await?;
        received.sort_by(|a, b| a[0].total_cmp(&b[0]));
        assert_close(&received[0], &[1.0, 2.0]);
        assert_close(&received[1], &[3.0, 4.0]);

        Ok(())
    }

    #[tokio::test]
    async fn test_network_bootstrap() -> Result<()> {
        // Test bootstrapping with a known (unreachable) peer
        let config = SwarmConfig {
            listen_addresses: vec!["/ip4/127.0.0.1/tcp/0".parse()?],
            bootstrap_nodes: vec![(PeerId::random(), "/ip4/127.0.0.1/tcp/1".parse()?)],
            ..SwarmConfig::default()
        };

        let mut network = P2PNetwork::new(config).await?;

        // Bootstrapping only starts dials, so it succeeds even though the
        // bootstrap peer is not reachable.
        network.bootstrap().await?;
        assert!(network.connected_peers().is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_message_ordering() -> Result<()> {
        let mut network1 = loopback_network().await?;
        let mut network2 = loopback_network().await?;
        net::connect_star(&mut network1, &mut [&mut network2]).await?;

        // Send multiple gradients in sequence
        let mut networks = [&mut network1, &mut network2];
        for i in 0..5 {
            net::broadcast(&mut networks, 1, vec![i as f32]).await?;
            net::pump(&mut networks, Duration::from_millis(50)).await?;
        }

        // The receiver keeps one gradient per peer and round, so in-order
        // delivery leaves it holding the last one sent.
        let start = Instant::now();
        loop {
            let received = networks[0].received_gradients().await;
            if let [(_, values)] = received.as_slice() {
                if (values[0] - 4.0).abs() < 0.05 {
                    break;
                }
            }
            assert!(
                start.elapsed() < net::DEADLINE,
                "last gradient not received"
            );
            net::pump(&mut networks, Duration::from_millis(50)).await?;
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_graceful_shutdown() -> Result<()> {
        let mut network1 = loopback_network().await?;
        let mut network2 = loopback_network().await?;
        net::connect_star(&mut network1, &mut [&mut network2]).await?;

        // Shutting a network down is dropping it; its peers must notice and
        // clean up the connection.
        drop(network2);

        let start = Instant::now();
        while !network1.connected_peers().is_empty() {
            assert!(start.elapsed() < net::DEADLINE, "peer still connected");
            net::pump(&mut [&mut network1], Duration::from_millis(50)).await?;
        }

        Ok(())
    }
}
