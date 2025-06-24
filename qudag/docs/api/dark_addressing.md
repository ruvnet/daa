# Dark Addressing System

The QuDAG Dark Addressing system provides quantum-resistant, privacy-preserving network addresses using the `.dark` domain namespace. This system integrates ML-DSA (Module-Lattice Digital Signature Algorithm) for signatures and ML-KEM (Module-Lattice Key Encapsulation Mechanism) for encryption.

## Overview

Dark addresses are cryptographically generated identifiers that:
- Are derived from ML-DSA public keys (quantum-resistant)
- Support human-readable `.dark` domain names
- Include DHT-based distributed resolution
- Provide TTL-based expiration
- Support multiple network addresses per domain
- Include an address book for contact management

## Key Features

### Quantum-Resistant Cryptography
- **ML-DSA-65**: Digital signatures with 128-bit post-quantum security
- **ML-KEM-768**: Key encapsulation for secure communications
- **BLAKE3**: Fast cryptographic hashing
- **Base58**: Human-readable address encoding

### Domain Name System
- Custom subdomain support (e.g., `alice-node.dark`)
- Auto-generated domains from public keys
- Validation rules:
  - 3-63 character subdomains
  - Alphanumeric + hyphens only
  - No leading/trailing hyphens
  - No consecutive hyphens

### Distributed Storage
- DHT integration for decentralized resolution
- Local caching with automatic synchronization
- Signature verification on retrieval
- Automatic expiration handling

## API Reference

### Core Types

```rust
/// A dark address with its domain name
pub struct DarkAddress {
    /// Base58-encoded address hash
    pub address: String,
    /// Full domain name (e.g., "mynode.dark")
    pub domain: String,
}

/// A dark domain record
pub struct DarkDomainRecord {
    /// ML-DSA public key for verification
    pub signing_public_key: Vec<u8>,
    /// ML-KEM public key for encryption
    pub encryption_public_key: Vec<u8>,
    /// Network addresses
    pub addresses: Vec<NetworkAddress>,
    /// Human-readable alias
    pub alias: Option<String>,
    /// Time-to-live in seconds
    pub ttl: u32,
    /// Registration timestamp
    pub registered_at: u64,
    /// Expiration timestamp
    pub expires_at: u64,
    /// Owner's peer ID
    pub owner_id: PeerId,
    /// ML-DSA signature
    pub signature: Vec<u8>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}
```

### DarkResolver API

#### Creating a Resolver

```rust
// Basic resolver with local storage only
let resolver = DarkResolver::new();

// Resolver with DHT integration
let dht_client = Arc::new(MyDhtClient::new());
let resolver = DarkResolver::with_dht(dht_client);
```

#### Registering Domains

```rust
// Register with custom name
let dark_addr = resolver.register_domain(
    Some("alice-node"),           // Custom subdomain
    vec![NetworkAddress::new([192, 168, 1, 100], 8080)],
    Some("Alice's Node".to_string()), // Alias
    3600,                         // TTL in seconds
    owner_id,                     // PeerId
    &mut rng,                     // RNG for key generation
)?;

// Register with auto-generated name
let dark_addr = resolver.register_domain(
    None,  // Auto-generate from public key
    addresses,
    alias,
    ttl,
    owner_id,
    &mut rng,
)?;
```

#### Domain Resolution

```rust
// Look up full domain record
let record = resolver.lookup_domain("alice-node.dark")?;

// Resolve to network addresses only
let addresses = resolver.resolve_addresses("alice-node.dark")?;
```

#### Address Book Management

```rust
// Add to address book
resolver.add_to_address_book(
    "Alice".to_string(),
    dark_address,
    Some("Primary contact".to_string()),
)?;

// Look up by name
let entry = resolver.lookup_address_book("Alice")?;

// List all entries
let entries = resolver.list_address_book()?;
```

#### Domain Updates

```rust
// Update domain (requires ownership verification)
resolver.update_domain("alice-node.dark", new_record)?;

// Clean up expired domains
let removed_count = resolver.cleanup_expired()?;
```

### DHT Integration

Implement the `DhtClient` trait for distributed storage:

```rust
pub trait DhtClient: Send + Sync {
    fn put(&self, key: &[u8], value: &[u8]) -> Result<(), DarkResolverError>;
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, DarkResolverError>;
    fn remove(&self, key: &[u8]) -> Result<(), DarkResolverError>;
}
```

## Security Considerations

### Signature Verification
- All domain records are signed with ML-DSA
- Signatures are verified on retrieval
- Tampering is detected automatically

### Owner Verification
- Updates require matching signing public keys
- Only the original registrant can modify records
- Ownership transfer requires re-registration

### Expiration Handling
- Domains expire after their TTL
- Expired domains cannot be resolved
- Automatic cleanup prevents stale records

## Example Usage

```rust
use qudag_network::dark_resolver::{DarkResolver, DarkAddress};
use qudag_network::types::{NetworkAddress, PeerId};
use rand::thread_rng;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let resolver = DarkResolver::new();
    let mut rng = thread_rng();
    
    // Register a domain
    let dark_addr = resolver.register_domain(
        Some("mynode"),
        vec![NetworkAddress::new([127, 0, 0, 1], 8080)],
        Some("My Node".to_string()),
        3600,
        PeerId::random(),
        &mut rng,
    )?;
    
    println!("Registered: {}", dark_addr.domain);
    println!("Address: {}", dark_addr.address);
    
    // Resolve the domain
    let addresses = resolver.resolve_addresses(&dark_addr.domain)?;
    for addr in addresses {
        println!("Network address: {}:{}", addr.ip(), addr.port());
    }
    
    Ok(())
}
```

## Performance Characteristics

### Key Generation
- ML-DSA keypair: ~1-5ms
- ML-KEM keypair: ~1-3ms
- Dark address derivation: <1ms

### Domain Operations
- Registration: O(1) local, O(log n) DHT
- Resolution: O(1) cached, O(log n) DHT
- Signature verification: ~1-2ms

### Memory Usage
- Domain record: ~4KB
- Address book entry: ~200 bytes
- Cached entries: Configurable limit

## Best Practices

1. **TTL Selection**
   - Use longer TTLs (hours/days) for stable nodes
   - Use shorter TTLs (minutes) for dynamic nodes
   - Balance between freshness and DHT load

2. **Address Management**
   - Store multiple addresses for redundancy
   - Include both IPv4 and IPv6 when available
   - Update addresses before TTL expiration

3. **Security**
   - Keep signing keys secure
   - Verify signatures on critical operations
   - Monitor for unexpected domain changes

4. **Performance**
   - Enable local caching for frequently accessed domains
   - Batch DHT operations when possible
   - Implement exponential backoff for retries