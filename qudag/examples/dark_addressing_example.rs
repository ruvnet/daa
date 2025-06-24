//! Example demonstrating the Dark Addressing system with quantum-resistant cryptography

use anyhow::Result;
use qudag_network::dark_resolver::{DarkResolver, DarkAddress, AddressBookEntry, DarkDomainRecord, DhtClient, DarkResolverError};
use qudag_network::types::{NetworkAddress, PeerId};
use rand::thread_rng;
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use tracing::{info, warn};
use tracing_subscriber;

/// Mock DHT client for demonstration
struct MockDhtClient {
    storage: Arc<RwLock<HashMap<Vec<u8>, Vec<u8>>>>,
}

impl MockDhtClient {
    fn new() -> Self {
        Self {
            storage: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl DhtClient for MockDhtClient {
    fn put(&self, key: &[u8], value: &[u8]) -> Result<(), DarkResolverError> {
        let mut storage = self.storage.write()
            .map_err(|_| DarkResolverError::StorageError)?;
        storage.insert(key.to_vec(), value.to_vec());
        info!("DHT: Stored {} bytes at key", value.len());
        Ok(())
    }
    
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, DarkResolverError> {
        let storage = self.storage.read()
            .map_err(|_| DarkResolverError::StorageError)?;
        let result = storage.get(key).cloned();
        info!("DHT: Retrieved {:?} bytes from key", result.as_ref().map(|v| v.len()));
        Ok(result)
    }
    
    fn remove(&self, key: &[u8]) -> Result<(), DarkResolverError> {
        let mut storage = self.storage.write()
            .map_err(|_| DarkResolverError::StorageError)?;
        storage.remove(key);
        info!("DHT: Removed key");
        Ok(())
    }
}

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    
    info!("=== QuDAG Dark Addressing Example ===");
    
    // Create resolver with mock DHT
    let dht_client = Arc::new(MockDhtClient::new());
    let resolver = DarkResolver::with_dht(dht_client);
    let mut rng = thread_rng();
    
    // Example 1: Register a .dark domain with custom name
    info!("\n1. Registering custom .dark domain...");
    let owner_id = PeerId::random();
    let addresses = vec![
        NetworkAddress::new([192, 168, 1, 100], 8080),
        NetworkAddress::new([10, 0, 0, 5], 9090),
    ];
    
    let dark_addr = resolver.register_domain(
        Some("alice-node"),
        addresses.clone(),
        Some("Alice's Primary Node".to_string()),
        3600, // 1 hour TTL
        owner_id.clone(),
        &mut rng,
    )?;
    
    info!("Registered domain: {}", dark_addr.domain);
    info!("Dark address: {}", dark_addr.address);
    
    // Example 2: Look up the domain
    info!("\n2. Looking up the registered domain...");
    let record = resolver.lookup_domain(&dark_addr.domain)?;
    info!("Found domain record:");
    info!("  - Alias: {:?}", record.alias);
    info!("  - Addresses: {:?}", record.addresses);
    info!("  - TTL: {} seconds", record.ttl);
    info!("  - Owner: {:?}", record.owner_id);
    
    // Example 3: Resolve addresses
    info!("\n3. Resolving network addresses...");
    let resolved_addresses = resolver.resolve_addresses(&dark_addr.domain)?;
    for (i, addr) in resolved_addresses.iter().enumerate() {
        info!("  Address {}: {}:{}", i + 1, addr.ip(), addr.port());
    }
    
    // Example 4: Register without custom name (auto-generated)
    info!("\n4. Registering with auto-generated name...");
    let bob_id = PeerId::random();
    let bob_addr = resolver.register_domain(
        None, // Auto-generate name
        vec![NetworkAddress::new([172, 16, 0, 10], 7777)],
        Some("Bob's Node".to_string()),
        7200, // 2 hour TTL
        bob_id,
        &mut rng,
    )?;
    
    info!("Auto-generated domain: {}", bob_addr.domain);
    info!("Dark address: {}", bob_addr.address);
    
    // Example 5: Address book functionality
    info!("\n5. Using address book...");
    resolver.add_to_address_book(
        "Alice".to_string(),
        dark_addr.clone(),
        Some("Primary contact".to_string()),
    )?;
    
    resolver.add_to_address_book(
        "Bob".to_string(),
        bob_addr.clone(),
        Some("Secondary node".to_string()),
    )?;
    
    // Look up by name
    let alice_entry = resolver.lookup_address_book("Alice")?;
    info!("Found Alice in address book:");
    info!("  - Domain: {}", alice_entry.dark_address.domain);
    info!("  - Address: {}", alice_entry.dark_address.address);
    info!("  - Notes: {:?}", alice_entry.notes);
    
    // List all entries
    let all_entries = resolver.list_address_book()?;
    info!("\nAll address book entries:");
    for entry in all_entries {
        info!("  - {}: {}", entry.name, entry.dark_address.domain);
    }
    
    // Example 6: Try to register duplicate custom name (should fail)
    info!("\n6. Testing duplicate domain protection...");
    let result = resolver.register_domain(
        Some("alice-node"),
        vec![],
        None,
        3600,
        PeerId::random(),
        &mut rng,
    );
    
    match result {
        Err(DarkResolverError::DomainExists) => {
            info!("✓ Correctly prevented duplicate domain registration");
        }
        _ => {
            warn!("✗ Duplicate domain check failed!");
        }
    }
    
    // Example 7: Demonstrate quantum-resistant signatures
    info!("\n7. Verifying quantum-resistant signatures...");
    let record = resolver.lookup_domain(&dark_addr.domain)?;
    match record.verify_signature() {
        Ok(()) => info!("✓ ML-DSA signature verified successfully"),
        Err(e) => warn!("✗ Signature verification failed: {}", e),
    }
    
    // Example 8: Domain expiration handling
    info!("\n8. Testing domain expiration...");
    
    // Register a domain with very short TTL for testing
    let short_ttl_addr = resolver.register_domain(
        Some("temp-node"),
        vec![NetworkAddress::new([127, 0, 0, 1], 8888)],
        None,
        1, // 1 second TTL (for demo only)
        PeerId::random(),
        &mut rng,
    )?;
    
    info!("Registered short-TTL domain: {}", short_ttl_addr.domain);
    
    // Wait for expiration
    std::thread::sleep(std::time::Duration::from_secs(2));
    
    // Try to look up expired domain
    match resolver.lookup_domain(&short_ttl_addr.domain) {
        Err(DarkResolverError::DomainExpired) => {
            info!("✓ Domain correctly expired after TTL");
        }
        _ => {
            warn!("✗ Domain expiration check failed!");
        }
    }
    
    // Clean up expired domains
    let cleaned = resolver.cleanup_expired()?;
    info!("Cleaned up {} expired domains", cleaned);
    
    // Example 9: Generate dark address from public key
    info!("\n9. Dark address generation from public key...");
    let public_key = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
    let generated_addr = DarkResolver::generate_dark_address(&public_key, None)?;
    info!("Generated dark address: {}", generated_addr.address);
    info!("Generated domain: {}", generated_addr.domain);
    
    // Summary
    info!("\n=== Summary ===");
    info!("Successfully demonstrated:");
    info!("✓ Quantum-resistant .dark domain registration");
    info!("✓ ML-DSA signature generation and verification");
    info!("✓ DNS-like domain resolution");
    info!("✓ DHT-based distributed storage");
    info!("✓ Address book functionality");
    info!("✓ TTL and expiration handling");
    info!("✓ Human-readable domain mapping");
    info!("✓ Duplicate domain protection");
    
    Ok(())
}