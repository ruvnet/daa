use qudag_network::router::{QuDagRouter, RouteError, Router, RouterConfig};
use std::collections::HashSet;

#[test]
fn test_router_config() {
    let config = RouterConfig::default();
    assert_eq!(config.min_hops, 3);
    assert_eq!(config.max_hops, 10);
    assert_eq!(config.max_attempts, 50);
    assert!(config.required_props.is_empty());
}

#[test]
fn test_path_selection() {
    let config = RouterConfig::default();
    let mut router = QuDagRouter::new(config);

    // Add some test peers
    let peers: Vec<Vec<u8>> = (0..10).map(|i| vec![i as u8]).collect();
    router.update_network(peers);

    let destination = vec![20u8]; // Not in peers list
    let path = router.select_path(destination.clone(), &config);

    assert!(path.is_ok());
    let path = path.unwrap();

    // Verify path properties
    assert!(path.len() >= config.min_hops);
    assert!(path.len() <= config.max_hops);
    assert_eq!(path.last().unwrap(), &destination);

    // Check for duplicates
    let mut seen = HashSet::new();
    for peer in &path {
        assert!(seen.insert(peer));
    }
}

#[test]
fn test_path_validation() {
    let config = RouterConfig {
        min_hops: 3,
        max_hops: 5,
        max_attempts: 50,
        required_props: HashSet::new(),
    };
    let router = QuDagRouter::new(config.clone());

    // Valid path
    let path: Vec<Vec<u8>> = vec![vec![1], vec![2], vec![3]];
    assert!(router.validate_path(&path).is_ok());

    // Path too short
    let path: Vec<Vec<u8>> = vec![vec![1], vec![2]];
    assert!(matches!(
        router.validate_path(&path),
        Err(RouteError::ValidationError(_))
    ));

    // Path too long
    let path: Vec<Vec<u8>> = vec![vec![1], vec![2], vec![3], vec![4], vec![5], vec![6]];
    assert!(matches!(
        router.validate_path(&path),
        Err(RouteError::ValidationError(_))
    ));

    // Duplicate peers
    let path: Vec<Vec<u8>> = vec![vec![1], vec![2], vec![1]];
    assert!(matches!(
        router.validate_path(&path),
        Err(RouteError::ValidationError(_))
    ));
}

#[test]
fn test_network_updates() {
    let config = RouterConfig::default();
    let mut router = QuDagRouter::new(config);

    let peers: Vec<Vec<u8>> = vec![vec![1], vec![2], vec![3]];
    router.update_network(peers.clone());

    // After update, router should be able to create paths using the new peers
    let destination = vec![4u8];
    let path = router.select_path(destination, &config);
    assert!(path.is_ok());
}
