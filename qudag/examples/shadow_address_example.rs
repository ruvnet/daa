//! Example demonstrating shadow address functionality with temporary addresses,
//! stealth addresses, and address pool management.

use qudag_network::shadow_address::*;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Shadow Address System Example ===\n");

    // Initialize the shadow address handler
    let master_seed = [42u8; 32]; // In production, use a secure random seed
    let network = NetworkType::Testnet;
    let handler = DefaultShadowAddressHandler::new(network, master_seed);

    // 1. Generate a regular shadow address
    println!("1. Generating regular shadow address:");
    let regular_address = handler.generate_address(network)?;
    println!("   View Key: {:?}", &regular_address.view_key[..8]);
    println!("   Spend Key: {:?}", &regular_address.spend_key[..8]);
    println!("   Created at: {}", regular_address.metadata.created_at);
    println!("   Expires: {:?}", regular_address.metadata.expires_at);
    println!();

    // 2. Generate a temporary address with TTL
    println!("2. Generating temporary address (30 second TTL):");
    let temp_address = handler.generate_temporary_address(network, Duration::from_secs(30))?;
    println!("   View Key: {:?}", &temp_address.view_key[..8]);
    println!("   Is Temporary: {}", temp_address.shadow_features.is_temporary);
    println!("   TTL: {} seconds", temp_address.metadata.ttl.unwrap_or(0));
    println!("   Expires at: {}", temp_address.metadata.expires_at.unwrap());
    println!();

    // 3. Generate a stealth address
    println!("3. Generating stealth address:");
    let recipient_view_key = [1u8; 32];
    let recipient_spend_key = [2u8; 32];
    let stealth_address = handler.generate_stealth_address(
        network,
        &recipient_view_key,
        &recipient_spend_key,
    )?;
    println!("   View Key: {:?}", &stealth_address.view_key[..8]);
    println!("   Spend Key: {:?}", &stealth_address.spend_key[..8]);
    println!("   Stealth Prefix: {:?}", stealth_address.shadow_features.stealth_prefix);
    println!("   Max Uses: {:?}", stealth_address.metadata.max_uses);
    println!("   Mixing Enabled: {}", stealth_address.shadow_features.mixing_enabled);
    println!();

    // 4. Derive address from master key
    println!("4. Deriving addresses from master key:");
    for i in 0..3 {
        let derived = handler.derive_from_master(&master_seed, i)?;
        println!("   Index {}: {:?}", i, &derived.view_key[..8]);
    }
    println!();

    // 5. Using the Shadow Address Manager
    println!("5. Shadow Address Manager:");
    let manager = ShadowAddressManager::new(network, master_seed).await;

    // Create temporary addresses
    let temp1 = manager.create_temporary_address(Duration::from_secs(60)).await?;
    let temp2 = manager.create_temporary_address(Duration::from_secs(120)).await?;
    println!("   Created {} temporary addresses", 2);

    // Create an address pool
    println!("\n6. Creating address pool:");
    manager.create_address_pool("main_pool".to_string(), 5, Some(Duration::from_secs(300))).await?;
    println!("   Created pool 'main_pool' with 5 addresses");

    // Get random address from pool
    if let Some(pool_addr) = manager.get_pool_address("main_pool").await {
        println!("   Random address from pool: {:?}", &pool_addr.view_key[..8]);
    }

    // 7. Address mixing for unlinkability
    println!("\n7. Address mixing:");
    let mixer = ShadowAddressMixer::new(3, Duration::from_millis(100));
    let addresses_to_mix = vec![regular_address.clone(), temp1.clone(), temp2.clone()];
    let mixed_addresses = mixer.mix_addresses(addresses_to_mix).await?;
    println!("   Mixed {} addresses with {} rounds", mixed_addresses.len(), 3);
    for (i, addr) in mixed_addresses.iter().enumerate() {
        println!("   Address {}: mixing_enabled = {}", i, addr.shadow_features.mixing_enabled);
    }

    // 8. Address validation
    println!("\n8. Address validation:");
    let is_valid = handler.validate_address(&regular_address)?;
    println!("   Regular address valid: {}", is_valid);

    // Simulate expired address
    let mut expired_addr = temp_address.clone();
    expired_addr.metadata.expires_at = Some(1); // Set to past timestamp
    let is_expired_valid = handler.validate_address(&expired_addr)?;
    println!("   Expired address valid: {}", is_expired_valid);

    // 9. Mark address as used and check rotation
    println!("\n9. Address usage tracking:");
    let mut tracked_addr = regular_address.clone();
    println!("   Initial usage count: {}", tracked_addr.metadata.usage_count);
    
    manager.mark_address_used(&mut tracked_addr).await;
    println!("   After one use: {}", tracked_addr.metadata.usage_count);
    println!("   Last used: {:?}", tracked_addr.metadata.last_used);

    // 10. Pool rotation
    println!("\n10. Pool rotation:");
    manager.rotate_pool("main_pool").await?;
    println!("   Pool 'main_pool' rotated with fresh addresses");

    // Wait a bit to demonstrate cleanup
    println!("\n11. Automatic cleanup (simulating...)");
    sleep(Duration::from_secs(2)).await;
    println!("   Cleanup task running in background, removing expired addresses");

    println!("\n=== Example Complete ===");
    Ok(())
}