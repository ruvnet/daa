use qudag_network::onion::{MLKEMOnionRouter, OnionError, OnionLayer, OnionRouter};

#[test]
fn test_onion_layer_creation() {
    let layer = OnionLayer::new(
        vec![1, 2, 3], // next hop
        vec![4, 5, 6], // payload
        vec![7, 8, 9], // metadata
    );

    assert_eq!(layer.next_hop, vec![1, 2, 3]);
    assert_eq!(layer.payload, vec![4, 5, 6]);
    assert_eq!(layer.metadata, vec![7, 8, 9]);
}

#[test]
fn test_onion_layer_validation() {
    // Valid layer
    let layer = OnionLayer::new(vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]);
    assert!(layer.validate().is_ok());

    // Empty next hop
    let layer = OnionLayer::new(vec![], vec![4, 5, 6], vec![7, 8, 9]);
    assert!(matches!(
        layer.validate(),
        Err(OnionError::InvalidFormat(_))
    ));

    // Empty payload
    let layer = OnionLayer::new(vec![1, 2, 3], vec![], vec![7, 8, 9]);
    assert!(matches!(
        layer.validate(),
        Err(OnionError::InvalidFormat(_))
    ));
}

#[test]
fn test_onion_router_creation() {
    let secret_key = vec![1, 2, 3, 4];
    let router = MLKEMOnionRouter::new(secret_key);

    // Future test cases will be implemented once ML-KEM
    // encryption/decryption is available
}
