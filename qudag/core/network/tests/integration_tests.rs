use qudag_network::prelude::*;
use std::time::Duration;
use tokio;

#[tokio::test]
async fn test_node_connectivity() {
    let node1_config = NetworkConfig {
        listen_addr: "/ip4/127.0.0.1/tcp/0".into(),
        bootstrap_peers: vec![],
        max_connections: 50,
    };

    let node2_config = NetworkConfig {
        listen_addr: "/ip4/127.0.0.1/tcp/0".into(),
        bootstrap_peers: vec![],
        max_connections: 50,
    };

    let (mut node1, mut rx1) = Node::new(node1_config).await.unwrap();
    let (mut node2, mut rx2) = Node::new(node2_config).await.unwrap();

    // Connect nodes
    let addr1 = node1.swarm.local_peer_id();
    let addr2 = node2.swarm.local_peer_id();
    node1
        .swarm
        .behaviour_mut()
        .connection_manager
        .connect(addr2)
        .await
        .unwrap();

    // Wait for connection events
    let timeout = Duration::from_secs(5);
    tokio::time::timeout(timeout, async {
        while let Some(event) = rx1.recv().await {
            match event {
                NetworkEvent::PeerConnected(peer) if peer == addr2 => break,
                _ => continue,
            }
        }
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn test_message_routing() {
    let node1_config = NetworkConfig {
        listen_addr: "/ip4/127.0.0.1/tcp/0".into(),
        bootstrap_peers: vec![],
        max_connections: 50,
    };

    let (mut node1, mut rx1) = Node::new(node1_config).await.unwrap();

    // Test message queue
    let msg = NetworkMessage {
        id: "test".into(),
        source: vec![1],
        destination: vec![2],
        payload: vec![0; 100],
        priority: MessagePriority::Normal,
        ttl: Duration::from_secs(60),
    };

    node1
        .swarm
        .behaviour_mut()
        .message_queue
        .enqueue(msg.clone())
        .await;

    // Verify routing
    let timeout = Duration::from_secs(5);
    tokio::time::timeout(timeout, async {
        while let Some(event) = rx1.recv().await {
            match event {
                NetworkEvent::MessageReceived { data, .. } if data == msg.payload => break,
                _ => continue,
            }
        }
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn test_anonymous_routing() {
    let node_config = NetworkConfig {
        listen_addr: "/ip4/127.0.0.1/tcp/0".into(),
        bootstrap_peers: vec![],
        max_connections: 50,
    };

    let (mut node, mut rx) = Node::new(node_config).await.unwrap();

    // Add test peers
    let peer_ids: Vec<_> = (0..5).map(|_| PeerId::random()).collect();
    for peer_id in &peer_ids {
        node.swarm.behaviour_mut().router.add_peer(*peer_id).await;
    }

    // Test anonymous message routing
    let msg = NetworkMessage {
        id: "anonymous".into(),
        source: vec![1],
        destination: vec![2],
        payload: vec![0; 100],
        priority: MessagePriority::High,
        ttl: Duration::from_secs(60),
    };

    let route = node
        .swarm
        .behaviour_mut()
        .router
        .route(&msg, RoutingStrategy::Anonymous { hops: 3 })
        .await
        .unwrap();

    assert_eq!(route.len(), 3);
    assert!(route.iter().all(|peer| peer_ids.contains(peer)));
}

#[tokio::test]
async fn test_kademlia_discovery() {
    let node1_config = NetworkConfig {
        listen_addr: "/ip4/127.0.0.1/tcp/0".into(),
        bootstrap_peers: vec![],
        max_connections: 50,
    };

    let node2_config = NetworkConfig {
        listen_addr: "/ip4/127.0.0.1/tcp/0".into(),
        bootstrap_peers: vec![],
        max_connections: 50,
    };

    let (mut node1, _) = Node::new(node1_config).await.unwrap();
    let (mut node2, _) = Node::new(node2_config).await.unwrap();

    // Bootstrap node2 using node1
    let node1_addr = node1.swarm.local_peer_id();
    node2
        .swarm
        .behaviour_mut()
        .kad
        .add_address(&node1_addr, "/ip4/127.0.0.1/tcp/0".parse().unwrap());
    node2.swarm.behaviour_mut().kad.bootstrap().unwrap();

    // Wait for bootstrap completion
    let timeout = Duration::from_secs(5);
    tokio::time::timeout(timeout, async {
        loop {
            match node2.swarm.behaviour_mut().kad.query_stats() {
                QueryStats { completed: c, .. } if c > 0 => break,
                _ => tokio::time::sleep(Duration::from_millis(100)).await,
            }
        }
    })
    .await
    .unwrap();
}
