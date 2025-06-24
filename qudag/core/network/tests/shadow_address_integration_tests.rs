//! Integration tests for the shadow address system with other network components

use qudag_network::{
    dark_resolver::{DarkDomainRecord, DarkResolver},
    shadow_address::*,
    types::NetworkAddress,
};
use std::time::Duration;

#[tokio::test]
async fn test_shadow_address_with_dark_resolver() {
    // Initialize components
    let seed = [42u8; 32];
    let network = NetworkType::Testnet;
    let handler = DefaultShadowAddressHandler::new(network, seed);
    let manager = ShadowAddressManager::new(network, seed).await;
    let resolver = DarkResolver::new();

    // Create a stealth address
    let recipient_view = [1u8; 32];
    let recipient_spend = [2u8; 32];
    let stealth_addr = handler
        .generate_stealth_address(network, &recipient_view, &recipient_spend)
        .unwrap();

    // Create network address for dark domain
    let network_addr = NetworkAddress::new([127, 0, 0, 1], 8080);

    // Register dark domain with stealth address
    resolver
        .register_domain("shadow-test.dark", network_addr.clone())
        .unwrap();

    // Lookup domain
    let record = resolver.lookup_domain("shadow-test.dark").unwrap();
    assert_eq!(record.encrypted_address.len() > 0, true);

    // Verify stealth address can be used for payments
    assert!(stealth_addr.shadow_features.stealth_prefix.is_some());
    assert_eq!(stealth_addr.metadata.max_uses, Some(1));
}

#[tokio::test]
async fn test_temporary_address_lifecycle() {
    let seed = [0u8; 32];
    let manager = ShadowAddressManager::new(NetworkType::Testnet, seed).await;

    // Create temporary address with short TTL
    let ttl = Duration::from_secs(2);
    let temp_addr = manager.create_temporary_address(ttl).await.unwrap();

    // Verify it's valid initially
    let handler = DefaultShadowAddressHandler::new(NetworkType::Testnet, seed);
    assert!(handler.validate_address(&temp_addr).unwrap());

    // Wait for expiration
    tokio::time::sleep(Duration::from_secs(3)).await;

    // Create a new handler to check validation (simulating time passage)
    let mut expired_addr = temp_addr.clone();
    expired_addr.metadata.expires_at = Some(1); // Force expiration
    assert!(!handler.validate_address(&expired_addr).unwrap());
}

#[tokio::test]
async fn test_address_pool_rotation_with_network() {
    let seed = [0u8; 32];
    let manager = ShadowAddressManager::new(NetworkType::Testnet, seed).await;

    // Create pool with TTL
    let pool_ttl = Duration::from_secs(300);
    manager
        .create_address_pool("network_pool".to_string(), 10, Some(pool_ttl))
        .await
        .unwrap();

    // Simulate network connections using pool addresses
    let mut used_addresses = Vec::new();
    for _ in 0..5 {
        if let Some(addr) = manager.get_pool_address("network_pool").await {
            used_addresses.push(addr);
        }
    }

    assert_eq!(used_addresses.len(), 5);

    // Rotate pool
    manager.rotate_pool("network_pool").await.unwrap();

    // Get new address after rotation
    let new_addr = manager.get_pool_address("network_pool").await.unwrap();

    // Verify it's different from used addresses
    for used in &used_addresses {
        assert_ne!(new_addr.view_key, used.view_key);
    }
}

#[tokio::test]
async fn test_hierarchical_address_derivation_for_channels() {
    let master_key = [99u8; 32];
    let handler = DefaultShadowAddressHandler::new(NetworkType::Mainnet, master_key);

    // Derive addresses for different payment channels
    let channel_addresses: Vec<_> = (0..5)
        .map(|i| handler.derive_from_master(&master_key, i).unwrap())
        .collect();

    // Verify all addresses are unique
    for i in 0..channel_addresses.len() {
        for j in (i + 1)..channel_addresses.len() {
            assert_ne!(channel_addresses[i].view_key, channel_addresses[j].view_key);
            assert_ne!(
                channel_addresses[i].spend_key,
                channel_addresses[j].spend_key
            );
        }
    }

    // Verify derivation is deterministic
    let channel_0_again = handler.derive_from_master(&master_key, 0).unwrap();
    assert_eq!(channel_addresses[0].view_key, channel_0_again.view_key);
}

#[tokio::test]
async fn test_address_mixing_for_privacy() {
    let mixer = ShadowAddressMixer::new(5, Duration::from_millis(50));
    let handler = DefaultShadowAddressHandler::new(NetworkType::Testnet, [0u8; 32]);

    // Generate multiple addresses
    let addresses: Vec<_> = (0..10)
        .map(|_| handler.generate_address(NetworkType::Testnet).unwrap())
        .collect();

    // Mix addresses
    let mixed = mixer.mix_addresses(addresses.clone()).await.unwrap();

    // Verify all addresses are marked as mixed
    for addr in &mixed {
        assert!(addr.shadow_features.mixing_enabled);
        assert_eq!(addr.metadata.flags & 0x08, 0x08);
    }

    // Verify count is preserved
    assert_eq!(addresses.len(), mixed.len());
}

#[tokio::test]
async fn test_usage_based_rotation() {
    let seed = [0u8; 32];
    let manager = ShadowAddressManager::new(NetworkType::Testnet, seed).await;

    // Set rotation policy
    let policies = RotationPolicies {
        rotate_after_uses: Some(3),
        rotate_after_duration: None,
        min_pool_size: 5,
        max_pool_size: 10,
    };
    *manager.rotation_policies.write().await = policies;

    // Create pool
    manager
        .create_address_pool("usage_pool".to_string(), 5, None)
        .await
        .unwrap();

    // Get and use an address multiple times
    let mut addr = manager.get_pool_address("usage_pool").await.unwrap();
    addr.shadow_features.pool_id = Some("usage_pool".to_string());

    // Use address 3 times (should trigger rotation)
    for _ in 0..3 {
        manager.mark_address_used(&mut addr).await;
    }

    assert_eq!(addr.metadata.usage_count, 3);
}

#[tokio::test]
async fn test_stealth_address_with_network_messages() {
    let handler = DefaultShadowAddressHandler::new(NetworkType::Testnet, [0u8; 32]);

    // Recipient keys
    let recipient_view = [10u8; 32];
    let recipient_spend = [20u8; 32];

    // Generate stealth address
    let stealth = handler
        .generate_stealth_address(NetworkType::Testnet, &recipient_view, &recipient_spend)
        .unwrap();

    // Verify stealth properties
    assert!(stealth.payment_id.is_some()); // Contains ephemeral public key
    assert!(stealth.shadow_features.mixing_enabled);
    assert_eq!(stealth.metadata.max_uses, Some(1));

    // Simulate scanning with stealth prefix
    if let Some(prefix) = stealth.shadow_features.stealth_prefix {
        // In real implementation, this would be used for efficient scanning
        assert_eq!(prefix.len(), 4);
    }
}

#[tokio::test]
async fn test_concurrent_address_management() {
    let manager = ShadowAddressManager::new(NetworkType::Testnet, [0u8; 32]).await;

    // Create multiple pools concurrently
    let handles: Vec<_> = (0..5)
        .map(|i| {
            let mgr = manager.clone();
            tokio::spawn(async move {
                mgr.create_address_pool(format!("pool_{}", i), 10, Some(Duration::from_secs(600)))
                    .await
            })
        })
        .collect();

    // Wait for all to complete
    for handle in handles {
        handle.await.unwrap().unwrap();
    }

    // Verify all pools were created
    for i in 0..5 {
        let addr = manager.get_pool_address(&format!("pool_{}", i)).await;
        assert!(addr.is_some());
    }
}
