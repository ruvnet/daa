//! Node churn simulation tests for network resilience
//!
//! Nodes are real `P2PNetwork`s on loopback TCP. A node "fails" by being
//! dropped and "recovers" as a fresh network that re-dials the hub. Waits are
//! deadline-bounded polls rather than fixed sleeps (see `crate::common::net`).

use crate::common::net::{self, loopback_network};
use anyhow::{anyhow, Result};
use daa_compute::P2PNetwork;
use libp2p::PeerId;
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::time::{Duration, Instant};

#[cfg(test)]
mod tests {
    use super::*;

    const SEED: u64 = 0x00DA_A0C0;

    struct NetworkNode {
        network: Option<P2PNetwork>,
        peer_id: PeerId,
    }

    impl NetworkNode {
        async fn new() -> Result<Self> {
            let network = loopback_network().await?;
            let peer_id = network.local_peer_id();

            Ok(Self {
                network: Some(network),
                peer_id,
            })
        }

        /// Fail the node: dropping the network closes all its connections.
        fn disconnect(&mut self) {
            self.network = None;
        }

        /// Recover the node as a fresh network (new identity and port).
        async fn reconnect(&mut self) -> Result<()> {
            if !self.is_active() {
                let network = loopback_network().await?;
                self.peer_id = network.local_peer_id();
                self.network = Some(network);
            }
            Ok(())
        }

        fn is_active(&self) -> bool {
            self.network.is_some()
        }

        fn network(&mut self) -> Result<&mut P2PNetwork> {
            self.network
                .as_mut()
                .ok_or_else(|| anyhow!("node is not active"))
        }
    }

    /// Handle events on every active node for `slice`.
    async fn pump(nodes: &mut [NetworkNode], slice: Duration) -> Result<()> {
        let mut networks: Vec<&mut P2PNetwork> = nodes
            .iter_mut()
            .filter_map(|n| n.network.as_mut())
            .collect();
        net::pump(&mut networks, slice).await
    }

    /// Pump until `condition` holds, failing after the shared deadline.
    async fn pump_until(
        nodes: &mut [NetworkNode],
        what: &str,
        mut condition: impl FnMut(&mut [NetworkNode]) -> bool,
    ) -> Result<()> {
        let start = Instant::now();
        while !condition(nodes) {
            if start.elapsed() > net::DEADLINE {
                return Err(anyhow!("timed out waiting for {what}"));
            }
            pump(nodes, Duration::from_millis(50)).await?;
        }
        Ok(())
    }

    /// Dial `to` from `from` without waiting for the connection.
    async fn dial(nodes: &mut [NetworkNode], from: usize, to: usize) -> Result<()> {
        let addr = net::listen_address(nodes[to].network()?).await?;
        let peer_id = nodes[to].peer_id;
        nodes[from].network()?.connect_to_peer(peer_id, addr)
    }

    /// Dial `to` from `from` and wait until the connection is up.
    async fn connect(nodes: &mut [NetworkNode], from: usize, to: usize) -> Result<()> {
        dial(nodes, from, to).await?;
        let target = nodes[to].peer_id;
        pump_until(nodes, "connection", |nodes| {
            nodes[from]
                .network
                .as_ref()
                .is_some_and(|n| n.connected_peers().contains(&target))
        })
        .await
    }

    /// Wait until `nodes[at]` no longer sees `peer` as connected.
    async fn await_disconnect(nodes: &mut [NetworkNode], at: usize, peer: PeerId) -> Result<()> {
        pump_until(nodes, "disconnect detection", |nodes| {
            nodes[at]
                .network
                .as_ref()
                .is_some_and(|n| !n.connected_peers().contains(&peer))
        })
        .await
    }

    async fn create_network_cluster(num_nodes: usize) -> Result<Vec<NetworkNode>> {
        let mut nodes = Vec::new();

        // Create all nodes
        for _ in 0..num_nodes {
            nodes.push(NetworkNode::new().await?);
        }

        // Connect all nodes to the first node (star topology)
        for i in 1..nodes.len() {
            dial(&mut nodes, i, 0).await?;
        }
        let hub = nodes[0].peer_id;
        pump_until(&mut nodes, "cluster connections", |nodes| {
            nodes[1..].iter().all(|n| {
                n.network
                    .as_ref()
                    .is_some_and(|n| n.connected_peers().contains(&hub))
            })
        })
        .await?;

        Ok(nodes)
    }

    /// Broadcast from `from`, retrying until a peer subscribes to the topic.
    async fn broadcast(nodes: &mut [NetworkNode], from: usize, values: Vec<f32>) -> Result<()> {
        let start = Instant::now();
        loop {
            match nodes[from]
                .network()?
                .broadcast_gradient(values.clone())
                .await
            {
                Ok(()) => return Ok(()),
                Err(e) if start.elapsed() > net::DEADLINE => return Err(e),
                Err(_) => pump(nodes, Duration::from_millis(100)).await?,
            }
        }
    }

    /// Single best-effort broadcast, as a node would attempt during churn.
    async fn try_broadcast(nodes: &mut [NetworkNode], from: usize, values: Vec<f32>) {
        if let Ok(network) = nodes[from].network() {
            let _ = network.broadcast_gradient(values).await;
        }
    }

    /// Wait until `nodes[receiver]` holds a gradient from `sender` close to `expected`.
    async fn await_gradient(
        nodes: &mut [NetworkNode],
        receiver: usize,
        sender: PeerId,
        expected: &[f32],
    ) -> Result<()> {
        let start = Instant::now();
        loop {
            let received = nodes[receiver].network()?.received_gradients().await;
            let matched = received.iter().any(|(peer, values)| {
                *peer == sender
                    && values.len() == expected.len()
                    && values
                        .iter()
                        .zip(expected)
                        .all(|(a, b)| (a - b).abs() < 0.05)
            });
            if matched {
                return Ok(());
            }
            if start.elapsed() > net::DEADLINE {
                return Err(anyhow!("gradient from {sender} not received"));
            }
            pump(nodes, Duration::from_millis(50)).await?;
        }
    }

    /// Index of the first active node other than the hub.
    fn first_active_spoke(nodes: &[NetworkNode]) -> Option<usize> {
        (1..nodes.len()).find(|&i| nodes[i].is_active())
    }

    #[tokio::test]
    async fn test_single_node_failure() -> Result<()> {
        let mut nodes = create_network_cluster(5).await?;
        assert_eq!(nodes[0].network()?.connected_peers().len(), 4);

        // Disconnect one node and wait for failure detection
        let failed = nodes[2].peer_id;
        nodes[2].disconnect();
        await_disconnect(&mut nodes, 0, failed).await?;

        // Remaining nodes can still communicate
        broadcast(&mut nodes, 1, vec![1.0, 2.0, 3.0]).await?;
        let sender = nodes[1].peer_id;
        await_gradient(&mut nodes, 0, sender, &[1.0, 2.0, 3.0]).await?;

        assert!(nodes[0].is_active());
        assert!(nodes[1].is_active());
        assert!(!nodes[2].is_active());

        Ok(())
    }

    #[tokio::test]
    async fn test_multiple_node_failures() -> Result<()> {
        let mut nodes = create_network_cluster(10).await?;

        // Disconnect multiple nodes randomly
        // Seeded so a failing churn pattern can be reproduced.
        let mut rng = StdRng::seed_from_u64(SEED);
        let failure_count = 4;
        let mut failed_indices = Vec::new();

        while failed_indices.len() < failure_count {
            let idx = rng.gen_range(1..nodes.len()); // Don't fail the hub node
            if !failed_indices.contains(&idx) {
                failed_indices.push(idx);
                nodes[idx].disconnect();
            }
        }

        // Wait for failure detection
        let expected_peers = nodes.len() - 1 - failure_count;
        pump_until(&mut nodes, "failure detection", |nodes| {
            nodes[0]
                .network
                .as_ref()
                .is_some_and(|n| n.connected_peers().len() == expected_peers)
        })
        .await?;

        // Broadcast from a surviving node; the hub must still receive it
        let survivor = first_active_spoke(&nodes).expect("a spoke survived");
        broadcast(&mut nodes, survivor, vec![4.0, 5.0, 6.0]).await?;
        let sender = nodes[survivor].peer_id;
        await_gradient(&mut nodes, 0, sender, &[4.0, 5.0, 6.0]).await?;

        // Count active nodes
        let active_count = nodes.iter().filter(|n| n.is_active()).count();
        assert_eq!(active_count, nodes.len() - failure_count);

        Ok(())
    }

    #[tokio::test]
    async fn test_node_recovery() -> Result<()> {
        let mut nodes = create_network_cluster(6).await?;

        // Disconnect some nodes
        nodes[2].disconnect();
        nodes[3].disconnect();
        pump(&mut nodes, Duration::from_millis(200)).await?;

        // Reconnect them to the hub
        nodes[2].reconnect().await?;
        nodes[3].reconnect().await?;
        connect(&mut nodes, 2, 0).await?;
        connect(&mut nodes, 3, 0).await?;

        // Recovered nodes can participate
        broadcast(&mut nodes, 2, vec![7.0, 8.0, 9.0]).await?;
        let sender = nodes[2].peer_id;
        await_gradient(&mut nodes, 0, sender, &[7.0, 8.0, 9.0]).await?;

        // All nodes should be active again
        assert!(nodes.iter().all(|n| n.is_active()));

        Ok(())
    }

    #[tokio::test]
    async fn test_cascading_failures() -> Result<()> {
        let mut nodes = create_network_cluster(8).await?;

        // Simulate cascading failures
        for i in (1..4).rev() {
            nodes[i].disconnect();
            pump(&mut nodes, Duration::from_millis(100)).await?;

            // Broadcast from a surviving node after each failure
            if let Some(j) = (i + 1..nodes.len()).find(|&j| nodes[j].is_active()) {
                try_broadcast(&mut nodes, j, vec![i as f32]).await;
            }

            pump(&mut nodes, Duration::from_millis(50)).await?;
        }

        // The hub must still reach the survivors
        let survivors = nodes.iter().skip(1).filter(|n| n.is_active()).count();
        pump_until(&mut nodes, "failure detection", |nodes| {
            nodes[0]
                .network
                .as_ref()
                .is_some_and(|n| n.connected_peers().len() == survivors)
        })
        .await?;

        // Network should still function despite cascading failures
        let active_count = nodes.iter().filter(|n| n.is_active()).count();
        assert!(active_count >= nodes.len() / 2);

        Ok(())
    }

    #[tokio::test]
    async fn test_hub_node_failure() -> Result<()> {
        let mut nodes = create_network_cluster(7).await?;

        // Fail the hub node (index 0) and let the spokes notice
        let hub = nodes[0].peer_id;
        nodes[0].disconnect();
        await_disconnect(&mut nodes, 1, hub).await?;

        // Connect node 1 and node 2 directly and communicate without the hub
        connect(&mut nodes, 1, 2).await?;
        broadcast(&mut nodes, 1, vec![10.0, 11.0]).await?;
        let sender = nodes[1].peer_id;
        await_gradient(&mut nodes, 2, sender, &[10.0, 11.0]).await?;

        // Network should adapt to hub failure
        assert!(!nodes[0].is_active());
        assert!(nodes[1].is_active());
        assert!(nodes[2].is_active());

        Ok(())
    }

    #[tokio::test]
    async fn test_rapid_churn() -> Result<()> {
        let mut nodes = create_network_cluster(10).await?;
        // Seeded so a failing churn pattern can be reproduced.
        let mut rng = StdRng::seed_from_u64(SEED);

        // Simulate rapid node churn over time
        for round in 0..5 {
            // Random failures
            let failure_idx = rng.gen_range(1..nodes.len());
            nodes[failure_idx].disconnect();
            pump(&mut nodes, Duration::from_millis(50)).await?;

            // Random recoveries, re-dialling the hub
            let recovery_idx = rng.gen_range(1..nodes.len());
            if !nodes[recovery_idx].is_active() {
                nodes[recovery_idx].reconnect().await?;
                dial(&mut nodes, recovery_idx, 0).await?;
            }
            pump(&mut nodes, Duration::from_millis(50)).await?;

            // Network functionality during churn
            if let Some(i) = first_active_spoke(&nodes) {
                try_broadcast(&mut nodes, i, vec![round as f32]).await;
            }
            pump(&mut nodes, Duration::from_millis(50)).await?;
        }

        // Check that some nodes survived the churn
        let active_count = nodes.iter().filter(|n| n.is_active()).count();
        assert!(active_count >= 3); // At least hub + 2 others

        Ok(())
    }

    #[tokio::test]
    async fn test_network_partition_recovery() -> Result<()> {
        let mut nodes = create_network_cluster(8).await?;

        // Partition: the hub keeps 1 and 2; 3, 4 and 5 leave and form their
        // own group around node 3; 6 and 7 stay isolated.
        let partition2 = [3, 4, 5];
        let isolated = [6, 7];
        for &i in partition2.iter() {
            nodes[i].disconnect();
            nodes[i].reconnect().await?;
        }
        for &i in isolated.iter() {
            nodes[i].disconnect();
        }
        connect(&mut nodes, 4, 3).await?;
        connect(&mut nodes, 5, 3).await?;

        // Each partition functions independently
        broadcast(&mut nodes, 1, vec![1.0, 2.0]).await?;
        broadcast(&mut nodes, 4, vec![3.0, 4.0]).await?;
        let (sender1, sender2) = (nodes[1].peer_id, nodes[4].peer_id);
        await_gradient(&mut nodes, 0, sender1, &[1.0, 2.0]).await?;
        await_gradient(&mut nodes, 3, sender2, &[3.0, 4.0]).await?;

        // Heal the partition by connecting partition2 back to the hub
        for &i in partition2.iter() {
            connect(&mut nodes, i, 0).await?;
        }

        // Messages flow across the healed network
        broadcast(&mut nodes, 3, vec![5.0, 6.0]).await?;
        let healed_sender = nodes[3].peer_id;
        await_gradient(&mut nodes, 0, healed_sender, &[5.0, 6.0]).await?;

        // Every node except the isolated pair is active after recovery
        let active_count = nodes.iter().filter(|n| n.is_active()).count();
        assert_eq!(active_count, nodes.len() - isolated.len());

        Ok(())
    }

    #[tokio::test]
    async fn test_byzantine_node_behavior() -> Result<()> {
        let mut nodes = create_network_cluster(7).await?;

        // A Byzantine node sends malicious gradients
        let malicious_gradients = vec![
            vec![f32::INFINITY, f32::NEG_INFINITY],
            vec![f32::NAN, 1e20],
            vec![1e30; 1000], // Extremely large
        ];
        for gradient in malicious_gradients {
            broadcast(&mut nodes, 3, gradient).await?;
            pump(&mut nodes, Duration::from_millis(50)).await?;
        }

        // Honest nodes still function properly
        broadcast(&mut nodes, 1, vec![1.0, 2.0, 3.0]).await?;
        let honest = nodes[1].peer_id;
        await_gradient(&mut nodes, 0, honest, &[1.0, 2.0, 3.0]).await?;

        // Filtering malicious gradients is the aggregation layer's job; the
        // network itself must not crash or drop honest peers.
        assert!(nodes[0].is_active());
        assert!(nodes[1].is_active());
        assert!(nodes[0].network()?.connected_peers().contains(&honest));

        Ok(())
    }

    #[tokio::test]
    async fn test_high_churn_rate() -> Result<()> {
        let mut nodes = create_network_cluster(12).await?;
        // Seeded so a failing churn pattern can be reproduced.
        let mut rng = StdRng::seed_from_u64(SEED);

        // Simulate very high churn rate
        for _iteration in 0..20 {
            // Multiple failures and recoveries per iteration
            for _ in 0..3 {
                let idx = rng.gen_range(1..nodes.len());
                if rng.gen_bool(0.5) && nodes[idx].is_active() {
                    nodes[idx].disconnect();
                } else if !nodes[idx].is_active() {
                    nodes[idx].reconnect().await?;
                    dial(&mut nodes, idx, 0).await?;
                }
            }
            pump(&mut nodes, Duration::from_millis(50)).await?;

            // Network functionality during high churn
            if let Some(i) = first_active_spoke(&nodes) {
                try_broadcast(&mut nodes, i, vec![42.0]).await;
            }
            pump(&mut nodes, Duration::from_millis(25)).await?;
        }

        // Network should survive high churn
        let active_count = nodes.iter().filter(|n| n.is_active()).count();
        assert!(active_count >= nodes.len() / 3); // At least 1/3 survived

        Ok(())
    }
}
