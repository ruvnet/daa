use qudag_network::shadow_address::{
    ShadowAddress, ShadowAddressGenerator, ShadowAddressResolver,
    DefaultShadowAddressHandler, NetworkType, ShadowMetadata
};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

#[tokio::test]
async fn test_shadow_address_generation() {
    let seed = [42u8; 32];
    let handler = DefaultShadowAddressHandler::new(NetworkType::Mainnet, seed);
    
    // Generate a shadow address
    let address = handler.generate_address(NetworkType::Mainnet).unwrap();
    
    // Verify address properties
    assert!(!address.view_key.is_empty());
    assert!(!address.spend_key.is_empty());
    assert_eq!(address.metadata.network, NetworkType::Mainnet);
    assert_eq!(address.metadata.version, 1);
    assert!(address.payment_id.is_none());
}

#[tokio::test]
async fn test_shadow_address_derivation() {
    let seed = [123u8; 32];
    let handler = DefaultShadowAddressHandler::new(NetworkType::Testnet, seed);
    
    // Generate base address
    let base_address = handler.generate_address(NetworkType::Testnet).unwrap();
    
    // Derive multiple addresses from base
    let mut derived_addresses = vec![];
    for _ in 0..5 {
        let derived = handler.derive_address(&base_address).unwrap();
        
        // Verify derived address maintains base properties
        assert_eq!(derived.metadata.network, base_address.metadata.network);
        assert_eq!(derived.metadata.version, base_address.metadata.version);
        assert_eq!(derived.payment_id, base_address.payment_id);
        
        // But has different keys
        assert_ne!(derived.view_key, base_address.view_key);
        assert_ne!(derived.spend_key, base_address.spend_key);
        
        derived_addresses.push(derived);
    }
    
    // Verify all derived addresses are unique
    for i in 0..derived_addresses.len() {
        for j in i+1..derived_addresses.len() {
            assert_ne!(derived_addresses[i].view_key, derived_addresses[j].view_key);
            assert_ne!(derived_addresses[i].spend_key, derived_addresses[j].spend_key);
        }
    }
}

#[tokio::test]
async fn test_shadow_address_resolution() {
    let seed = [200u8; 32];
    let handler = DefaultShadowAddressHandler::new(NetworkType::Devnet, seed);
    
    // Generate address
    let address = handler.generate_address(NetworkType::Devnet).unwrap();
    
    // Resolve to one-time address
    let resolved = handler.resolve_address(&address).unwrap();
    
    // Verify resolution properties
    assert!(!resolved.is_empty());
    assert!(resolved.len() >= address.view_key.len() + address.spend_key.len());
    
    // Verify resolution is deterministic
    let resolved2 = handler.resolve_address(&address).unwrap();
    assert_eq!(resolved, resolved2);
}

#[tokio::test]
async fn test_shadow_address_validation() {
    let seed = [99u8; 32];
    let handler = DefaultShadowAddressHandler::new(NetworkType::Mainnet, seed);
    
    // Test valid address
    let valid_address = handler.generate_address(NetworkType::Mainnet).unwrap();
    assert!(handler.validate_address(&valid_address).unwrap());
    
    // Test invalid address (empty keys)
    let invalid_address = ShadowAddress {
        view_key: vec![],
        spend_key: vec![],
        payment_id: None,
        metadata: ShadowMetadata {
            version: 1,
            network: NetworkType::Mainnet,
            expires_at: None,
            flags: 0,
        },
    };
    assert!(!handler.validate_address(&invalid_address).unwrap());
}

#[tokio::test]
async fn test_shadow_address_with_payment_id() {
    let seed = [77u8; 32];
    let handler = DefaultShadowAddressHandler::new(NetworkType::Testnet, seed);
    
    // Generate base address
    let mut address = handler.generate_address(NetworkType::Testnet).unwrap();
    
    // Add payment ID
    let payment_id = [0xABu8; 32];
    address.payment_id = Some(payment_id);
    
    // Derive from address with payment ID
    let derived = handler.derive_address(&address).unwrap();
    assert_eq!(derived.payment_id, Some(payment_id));
    
    // Resolve addresses with payment ID
    let resolved = handler.resolve_address(&address).unwrap();
    assert!(resolved.len() > address.view_key.len() + address.spend_key.len() + 32);
}

#[tokio::test]
async fn test_shadow_address_expiration() {
    let seed = [88u8; 32];
    let handler = DefaultShadowAddressHandler::new(NetworkType::Devnet, seed);
    
    // Generate address with expiration
    let mut address = handler.generate_address(NetworkType::Devnet).unwrap();
    let expiry_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() + 3600; // Expires in 1 hour
    
    address.metadata.expires_at = Some(expiry_time);
    
    // Verify expiration is preserved in derivation
    let derived = handler.derive_address(&address).unwrap();
    assert_eq!(derived.metadata.expires_at, Some(expiry_time));
}

#[tokio::test]
async fn test_shadow_address_check() {
    let seed = [55u8; 32];
    let handler = DefaultShadowAddressHandler::new(NetworkType::Mainnet, seed);
    
    // Generate address and resolve it
    let address = handler.generate_address(NetworkType::Mainnet).unwrap();
    let resolved = handler.resolve_address(&address).unwrap();
    
    // Check if resolved address belongs to shadow address
    assert!(handler.check_address(&address, &resolved).unwrap());
    
    // Check with wrong data
    let wrong_data = vec![0u8; resolved.len()];
    assert!(!handler.check_address(&address, &wrong_data).unwrap());
}

#[tokio::test]
async fn test_concurrent_shadow_address_operations() {
    let seed = [111u8; 32];
    let handler = Arc::new(DefaultShadowAddressHandler::new(NetworkType::Testnet, seed));
    let addresses = Arc::new(RwLock::new(HashMap::new()));
    
    let mut handles = vec![];
    
    // Spawn tasks for concurrent operations
    for i in 0..10 {
        let handler_clone = handler.clone();
        let addresses_clone = addresses.clone();
        
        let handle = tokio::spawn(async move {
            // Generate address
            let address = handler_clone.generate_address(NetworkType::Testnet).unwrap();
            
            // Store in shared map
            addresses_clone.write().await.insert(i, address.clone());
            
            // Perform operations
            let _derived = handler_clone.derive_address(&address).unwrap();
            let _resolved = handler_clone.resolve_address(&address).unwrap();
            let _valid = handler_clone.validate_address(&address).unwrap();
        });
        
        handles.push(handle);
    }
    
    // Wait for all operations
    for handle in handles {
        handle.await.unwrap();
    }
    
    // Verify all addresses were generated
    assert_eq!(addresses.read().await.len(), 10);
}

#[tokio::test]
async fn test_shadow_address_serialization() {
    let seed = [222u8; 32];
    let handler = DefaultShadowAddressHandler::new(NetworkType::Devnet, seed);
    
    // Generate address with all features
    let mut address = handler.generate_address(NetworkType::Devnet).unwrap();
    address.payment_id = Some([0xFFu8; 32]);
    address.metadata.expires_at = Some(1234567890);
    address.metadata.flags = 0b1010;
    
    // Serialize
    let serialized = serde_json::to_string(&address).unwrap();
    
    // Deserialize
    let deserialized: ShadowAddress = serde_json::from_str(&serialized).unwrap();
    
    // Verify all fields match
    assert_eq!(deserialized.view_key, address.view_key);
    assert_eq!(deserialized.spend_key, address.spend_key);
    assert_eq!(deserialized.payment_id, address.payment_id);
    assert_eq!(deserialized.metadata.network, address.metadata.network);
    assert_eq!(deserialized.metadata.version, address.metadata.version);
    assert_eq!(deserialized.metadata.expires_at, address.metadata.expires_at);
    assert_eq!(deserialized.metadata.flags, address.metadata.flags);
}

#[tokio::test]
async fn test_shadow_address_network_isolation() {
    let seed = [133u8; 32];
    
    // Create handlers for different networks
    let mainnet_handler = DefaultShadowAddressHandler::new(NetworkType::Mainnet, seed);
    let testnet_handler = DefaultShadowAddressHandler::new(NetworkType::Testnet, seed);
    let devnet_handler = DefaultShadowAddressHandler::new(NetworkType::Devnet, seed);
    
    // Generate addresses on each network
    let mainnet_addr = mainnet_handler.generate_address(NetworkType::Mainnet).unwrap();
    let testnet_addr = testnet_handler.generate_address(NetworkType::Testnet).unwrap();
    let devnet_addr = devnet_handler.generate_address(NetworkType::Devnet).unwrap();
    
    // Verify network types
    assert_eq!(mainnet_addr.metadata.network, NetworkType::Mainnet);
    assert_eq!(testnet_addr.metadata.network, NetworkType::Testnet);
    assert_eq!(devnet_addr.metadata.network, NetworkType::Devnet);
    
    // Verify addresses are different even with same seed
    assert_ne!(mainnet_addr.view_key, testnet_addr.view_key);
    assert_ne!(testnet_addr.view_key, devnet_addr.view_key);
}

#[tokio::test]
async fn test_shadow_address_flag_operations() {
    let seed = [144u8; 32];
    let handler = DefaultShadowAddressHandler::new(NetworkType::Mainnet, seed);
    
    // Generate address with custom flags
    let mut address = handler.generate_address(NetworkType::Mainnet).unwrap();
    
    // Set various flags
    const FLAG_ENCRYPTED: u32 = 1 << 0;
    const FLAG_COMPRESSED: u32 = 1 << 1;
    const FLAG_EPHEMERAL: u32 = 1 << 2;
    
    address.metadata.flags = FLAG_ENCRYPTED | FLAG_COMPRESSED;
    
    // Verify flags are preserved
    let derived = handler.derive_address(&address).unwrap();
    assert_eq!(derived.metadata.flags, FLAG_ENCRYPTED | FLAG_COMPRESSED);
    
    // Test flag checking
    assert!(address.metadata.flags & FLAG_ENCRYPTED != 0);
    assert!(address.metadata.flags & FLAG_COMPRESSED != 0);
    assert!(address.metadata.flags & FLAG_EPHEMERAL == 0);
}