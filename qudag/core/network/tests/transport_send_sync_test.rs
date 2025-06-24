//! Test to verify transport types implement Send + Sync

use qudag_network::transport::{SecureTransport, Transport, TransportConfig};
use std::sync::Arc;

#[test]
fn test_secure_transport_is_send_sync() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}
    fn assert_send_sync<T: Send + Sync>() {}

    // Verify SecureTransport is Send + Sync
    assert_send::<SecureTransport>();
    assert_sync::<SecureTransport>();
    assert_send_sync::<SecureTransport>();

    // Verify it can be wrapped in Arc
    let transport = Arc::new(SecureTransport::new());
    assert_send_sync::<Arc<SecureTransport>>();

    // Verify the trait object is Send + Sync
    fn verify_trait_object() -> Box<dyn Transport + Send + Sync> {
        Box::new(SecureTransport::new())
    }

    let _trait_obj = verify_trait_object();
}

#[tokio::test]
async fn test_transport_basic_operations() {
    let mut transport = SecureTransport::new();
    let config = TransportConfig::default();

    // Initialize transport
    transport.init(config).await.unwrap();

    // Verify we can get stats
    let stats = transport.get_stats();
    assert_eq!(stats.active_connections, 0);

    // Verify we can get connections
    let connections = transport.get_connections();
    assert!(connections.is_empty());
}
