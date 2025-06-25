use prime_dht::{DhtNode, DhtConfig, RoutingTable};
use libp2p::PeerId;
use std::time::Duration;

#[tokio::test]
async fn test_dht_node_creation() {
    let config = DhtConfig::default();
    let node = DhtNode::new(config).await.unwrap();
    
    assert!(node.local_peer_id().is_some());
    assert_eq!(node.connected_peers().await, 0);
}

#[tokio::test]
async fn test_routing_table_operations() {
    let mut routing_table = RoutingTable::new();
    let peer_id = PeerId::random();
    
    routing_table.add_peer(peer_id.clone());
    assert!(routing_table.contains(&peer_id));
    
    let nearest = routing_table.find_nearest(&peer_id, 5);
    assert!(!nearest.is_empty());
}

#[tokio::test]
async fn test_dht_bootstrap() {
    let config = DhtConfig::default();
    let mut node = DhtNode::new(config).await.unwrap();
    
    let bootstrap_result = node.bootstrap(vec![]).await;
    assert!(bootstrap_result.is_ok());
}

#[tokio::test]
async fn test_dht_store_and_retrieve() {
    let config = DhtConfig::default();
    let mut node = DhtNode::new(config).await.unwrap();
    
    let key = b"test-key";
    let value = b"test-value";
    
    node.store(key, value).await.unwrap();
    let retrieved = node.get(key).await.unwrap();
    
    assert_eq!(retrieved.as_deref(), Some(&value[..]));
}

#[tokio::test]
async fn test_dht_replication() {
    let config = DhtConfig {
        replication_factor: 3,
        ..Default::default()
    };
    
    let node = DhtNode::new(config).await.unwrap();
    assert_eq!(node.replication_factor(), 3);
}