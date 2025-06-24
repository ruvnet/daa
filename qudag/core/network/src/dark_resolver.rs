use blake3::Hasher;
use bs58;
use rand_core::{CryptoRng, RngCore};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

// Import crypto primitives from the crypto module
use qudag_crypto::ml_dsa::{MlDsaError, MlDsaKeyPair, MlDsaPublicKey};
use qudag_crypto::ml_kem::MlKem768;

use crate::types::NetworkAddress;
use crate::types::PeerId;

/// Errors that can occur during dark domain operations
#[derive(Error, Debug)]
pub enum DarkResolverError {
    #[error("Domain name already registered")]
    DomainExists,
    #[error("Domain not found")]
    DomainNotFound,
    #[error("Invalid domain name format")]
    InvalidDomain,
    #[error("Cryptographic operation failed: {0}")]
    CryptoError(String),
    #[error("Domain record access error")]
    StorageError,
    #[error("Domain has expired")]
    DomainExpired,
    #[error("Invalid signature")]
    InvalidSignature,
    #[error("Address generation failed: {0}")]
    AddressGenerationError(String),
    #[error("DHT operation failed: {0}")]
    DhtError(String),
    #[error("ML-DSA error: {0}")]
    MlDsaError(#[from] MlDsaError),
}

/// A resolved dark domain record with quantum-resistant signatures
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DarkDomainRecord {
    /// ML-DSA public key for signature verification
    pub signing_public_key: Vec<u8>,
    /// ML-KEM public key for encryption
    pub encryption_public_key: Vec<u8>,
    /// Network addresses (can have multiple)
    pub addresses: Vec<NetworkAddress>,
    /// Human-readable alias
    pub alias: Option<String>,
    /// Time-to-live in seconds
    pub ttl: u32,
    /// Registration timestamp
    pub registered_at: u64,
    /// Expiration timestamp
    pub expires_at: u64,
    /// Owner's PeerId
    pub owner_id: PeerId,
    /// Record signature using ML-DSA
    pub signature: Vec<u8>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Dark address derived from ML-DSA public key
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DarkAddress {
    /// The .dark address (base58 encoded hash)
    pub address: String,
    /// Full domain name (e.g., "mynode.dark")
    pub domain: String,
}

/// Address book entry for human-readable names
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AddressBookEntry {
    /// Human-readable name
    pub name: String,
    /// Associated dark address
    pub dark_address: DarkAddress,
    /// Optional notes
    pub notes: Option<String>,
    /// Added timestamp
    pub added_at: u64,
}

impl DarkDomainRecord {
    /// Create a new domain record
    pub fn new(
        signing_keypair: &MlDsaKeyPair,
        encryption_public_key: Vec<u8>,
        addresses: Vec<NetworkAddress>,
        alias: Option<String>,
        ttl: u32,
        owner_id: PeerId,
    ) -> Result<Self, DarkResolverError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut record = Self {
            signing_public_key: signing_keypair.public_key().to_vec(),
            encryption_public_key,
            addresses,
            alias,
            ttl,
            registered_at: now,
            expires_at: now + ttl as u64,
            owner_id,
            signature: vec![],
            metadata: HashMap::new(),
        };

        // Sign the record
        record.sign(signing_keypair)?;
        Ok(record)
    }

    /// Sign the record with ML-DSA
    fn sign(&mut self, keypair: &MlDsaKeyPair) -> Result<(), DarkResolverError> {
        let mut rng = rand::thread_rng();
        let message = self.to_signable_bytes()?;
        self.signature = keypair
            .sign(&message, &mut rng)
            .map_err(|e| DarkResolverError::MlDsaError(e))?;
        Ok(())
    }

    /// Verify the record's signature
    pub fn verify_signature(&self) -> Result<(), DarkResolverError> {
        let public_key = MlDsaPublicKey::from_bytes(&self.signing_public_key)
            .map_err(|e| DarkResolverError::MlDsaError(e))?;
        let message = self.to_signable_bytes()?;
        public_key
            .verify(&message, &self.signature)
            .map_err(|e| DarkResolverError::MlDsaError(e))?;
        Ok(())
    }

    /// Convert record to bytes for signing (excludes signature field)
    fn to_signable_bytes(&self) -> Result<Vec<u8>, DarkResolverError> {
        let mut hasher = Hasher::new();
        hasher.update(&self.signing_public_key);
        hasher.update(&self.encryption_public_key);
        for addr in &self.addresses {
            hasher.update(
                &bincode::serialize(addr)
                    .map_err(|e| DarkResolverError::CryptoError(e.to_string()))?,
            );
        }
        if let Some(alias) = &self.alias {
            hasher.update(alias.as_bytes());
        }
        hasher.update(&self.ttl.to_le_bytes());
        hasher.update(&self.registered_at.to_le_bytes());
        hasher.update(&self.expires_at.to_le_bytes());
        hasher.update(
            &bincode::serialize(&self.owner_id)
                .map_err(|e| DarkResolverError::CryptoError(e.to_string()))?,
        );
        Ok(hasher.finalize().as_bytes().to_vec())
    }

    /// Check if the record has expired
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now > self.expires_at
    }
}

/// Dark domain resolver that manages .dark domain registrations and lookups
pub struct DarkResolver {
    /// Thread-safe storage for domain records
    domains: Arc<RwLock<HashMap<String, DarkDomainRecord>>>,
    /// Address book for human-readable names
    address_book: Arc<RwLock<HashMap<String, AddressBookEntry>>>,
    /// Reverse lookup: dark address -> domain name
    reverse_lookup: Arc<RwLock<HashMap<String, String>>>,
    /// DHT client for distributed storage (placeholder)
    dht_client: Option<Arc<dyn DhtClient>>,
}

/// Trait for DHT client operations
pub trait DhtClient: Send + Sync {
    /// Store a value in the DHT
    fn put(&self, key: &[u8], value: &[u8]) -> Result<(), DarkResolverError>;
    /// Retrieve a value from the DHT
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, DarkResolverError>;
    /// Remove a value from the DHT
    fn remove(&self, key: &[u8]) -> Result<(), DarkResolverError>;
}

impl Default for DarkResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl DarkResolver {
    /// Creates a new dark domain resolver
    pub fn new() -> Self {
        Self {
            domains: Arc::new(RwLock::new(HashMap::new())),
            address_book: Arc::new(RwLock::new(HashMap::new())),
            reverse_lookup: Arc::new(RwLock::new(HashMap::new())),
            dht_client: None,
        }
    }

    /// Create a resolver with DHT client
    pub fn with_dht(dht_client: Arc<dyn DhtClient>) -> Self {
        Self {
            domains: Arc::new(RwLock::new(HashMap::new())),
            address_book: Arc::new(RwLock::new(HashMap::new())),
            reverse_lookup: Arc::new(RwLock::new(HashMap::new())),
            dht_client: Some(dht_client),
        }
    }

    /// Generate a .dark address from ML-DSA public key
    pub fn generate_dark_address(
        public_key: &[u8],
        custom_name: Option<&str>,
    ) -> Result<DarkAddress, DarkResolverError> {
        // Hash the public key with BLAKE3
        let mut hasher = Hasher::new();
        hasher.update(b"dark_address_v1");
        hasher.update(public_key);
        let hash = hasher.finalize();

        // Take first 20 bytes and encode as base58
        let address_bytes = &hash.as_bytes()[..20];
        let address = bs58::encode(address_bytes).into_string();

        // Generate domain name
        let domain = if let Some(name) = custom_name {
            if !Self::is_valid_custom_name(name) {
                return Err(DarkResolverError::InvalidDomain);
            }
            format!("{}.dark", name)
        } else {
            // Use first 8 chars of address as subdomain
            format!("{}.dark", &address[..8].to_lowercase())
        };

        Ok(DarkAddress { address, domain })
    }

    /// Validate custom name for .dark domain
    fn is_valid_custom_name(name: &str) -> bool {
        // Must be 3-63 chars, alphanumeric + hyphens, not start/end with hyphen
        name.len() >= 3
            && name.len() <= 63
            && name.chars().all(|c| c.is_alphanumeric() || c == '-')
            && !name.starts_with('-')
            && !name.ends_with('-')
    }

    /// Register a new .dark domain with quantum-resistant signatures
    pub fn register_domain<R: CryptoRng + RngCore>(
        &self,
        custom_name: Option<&str>,
        addresses: Vec<NetworkAddress>,
        alias: Option<String>,
        ttl: u32,
        owner_id: PeerId,
        rng: &mut R,
    ) -> Result<DarkAddress, DarkResolverError> {
        // Generate ML-DSA keypair for signing
        let signing_keypair =
            MlDsaKeyPair::generate(rng).map_err(|e| DarkResolverError::MlDsaError(e))?;

        // Generate ML-KEM keypair for encryption
        let (kem_public, _kem_secret) =
            MlKem768::keygen().map_err(|e| DarkResolverError::CryptoError(e.to_string()))?;

        // Generate dark address from signing public key
        let dark_address = Self::generate_dark_address(signing_keypair.public_key(), custom_name)?;

        // Validate domain doesn't exist
        if !Self::is_valid_dark_domain(&dark_address.domain) {
            return Err(DarkResolverError::InvalidDomain);
        }

        // Create domain record
        let record = DarkDomainRecord::new(
            &signing_keypair,
            kem_public.as_bytes().to_vec(),
            addresses,
            alias,
            ttl,
            owner_id,
        )?;

        // Store locally
        {
            let mut domains = self
                .domains
                .write()
                .map_err(|_| DarkResolverError::StorageError)?;

            if domains.contains_key(&dark_address.domain) {
                return Err(DarkResolverError::DomainExists);
            }

            domains.insert(dark_address.domain.clone(), record.clone());
        }

        // Update reverse lookup
        {
            let mut reverse = self
                .reverse_lookup
                .write()
                .map_err(|_| DarkResolverError::StorageError)?;
            reverse.insert(dark_address.address.clone(), dark_address.domain.clone());
        }

        // Store in DHT if available
        if let Some(dht) = &self.dht_client {
            let key = Self::domain_to_dht_key(&dark_address.domain);
            let value = bincode::serialize(&record)
                .map_err(|e| DarkResolverError::DhtError(e.to_string()))?;
            dht.put(&key, &value)?;
        }

        Ok(dark_address)
    }

    /// Convert domain to DHT key
    fn domain_to_dht_key(domain: &str) -> Vec<u8> {
        let mut hasher = Hasher::new();
        hasher.update(b"dark_domain:");
        hasher.update(domain.as_bytes());
        hasher.finalize().as_bytes().to_vec()
    }

    /// Look up a .dark domain and return its record
    pub fn lookup_domain(&self, domain: &str) -> Result<DarkDomainRecord, DarkResolverError> {
        // Validate domain name
        if !Self::is_valid_dark_domain(domain) {
            return Err(DarkResolverError::InvalidDomain);
        }

        // Try local storage first
        {
            let domains = self
                .domains
                .read()
                .map_err(|_| DarkResolverError::StorageError)?;

            if let Some(record) = domains.get(domain) {
                // Check expiration
                if record.is_expired() {
                    return Err(DarkResolverError::DomainExpired);
                }
                // Verify signature
                record.verify_signature()?;
                return Ok(record.clone());
            }
        }

        // Try DHT if not found locally
        if let Some(dht) = &self.dht_client {
            let key = Self::domain_to_dht_key(domain);
            if let Some(value) = dht.get(&key)? {
                let record: DarkDomainRecord = bincode::deserialize(&value)
                    .map_err(|e| DarkResolverError::DhtError(e.to_string()))?;

                // Verify and cache
                if record.is_expired() {
                    return Err(DarkResolverError::DomainExpired);
                }
                record.verify_signature()?;

                // Cache locally
                let mut domains = self
                    .domains
                    .write()
                    .map_err(|_| DarkResolverError::StorageError)?;
                domains.insert(domain.to_string(), record.clone());

                return Ok(record);
            }
        }

        Err(DarkResolverError::DomainNotFound)
    }

    /// Resolve a .dark domain to network addresses
    pub fn resolve_addresses(
        &self,
        domain: &str,
    ) -> Result<Vec<NetworkAddress>, DarkResolverError> {
        let record = self.lookup_domain(domain)?;
        Ok(record.addresses)
    }

    /// Add entry to address book
    pub fn add_to_address_book(
        &self,
        name: String,
        dark_address: DarkAddress,
        notes: Option<String>,
    ) -> Result<(), DarkResolverError> {
        let entry = AddressBookEntry {
            name: name.clone(),
            dark_address,
            notes,
            added_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        let mut book = self
            .address_book
            .write()
            .map_err(|_| DarkResolverError::StorageError)?;
        book.insert(name, entry);
        Ok(())
    }

    /// Look up address book entry by name
    pub fn lookup_address_book(&self, name: &str) -> Result<AddressBookEntry, DarkResolverError> {
        let book = self
            .address_book
            .read()
            .map_err(|_| DarkResolverError::StorageError)?;
        book.get(name)
            .cloned()
            .ok_or(DarkResolverError::DomainNotFound)
    }

    /// List all address book entries
    pub fn list_address_book(&self) -> Result<Vec<AddressBookEntry>, DarkResolverError> {
        let book = self
            .address_book
            .read()
            .map_err(|_| DarkResolverError::StorageError)?;
        Ok(book.values().cloned().collect())
    }

    /// Update domain record (requires signature from owner)
    pub fn update_domain(
        &self,
        domain: &str,
        record: DarkDomainRecord,
    ) -> Result<(), DarkResolverError> {
        // Verify the new record's signature
        record.verify_signature()?;

        // Get existing record to verify ownership
        let existing = self.lookup_domain(domain)?;

        // Verify same owner (by comparing signing public keys)
        if existing.signing_public_key != record.signing_public_key {
            return Err(DarkResolverError::InvalidSignature);
        }

        // Update local storage
        {
            let mut domains = self
                .domains
                .write()
                .map_err(|_| DarkResolverError::StorageError)?;
            domains.insert(domain.to_string(), record.clone());
        }

        // Update DHT
        if let Some(dht) = &self.dht_client {
            let key = Self::domain_to_dht_key(domain);
            let value = bincode::serialize(&record)
                .map_err(|e| DarkResolverError::DhtError(e.to_string()))?;
            dht.put(&key, &value)?;
        }

        Ok(())
    }

    /// Remove expired domains
    pub fn cleanup_expired(&self) -> Result<usize, DarkResolverError> {
        let mut count = 0;
        let mut to_remove = Vec::new();

        // Find expired domains
        {
            let domains = self
                .domains
                .read()
                .map_err(|_| DarkResolverError::StorageError)?;
            for (domain, record) in domains.iter() {
                if record.is_expired() {
                    to_remove.push(domain.clone());
                }
            }
        }

        // Remove expired domains
        {
            let mut domains = self
                .domains
                .write()
                .map_err(|_| DarkResolverError::StorageError)?;
            let mut reverse = self
                .reverse_lookup
                .write()
                .map_err(|_| DarkResolverError::StorageError)?;

            for domain in to_remove {
                if let Some(record) = domains.remove(&domain) {
                    count += 1;
                    // Remove from reverse lookup
                    let addr = Self::generate_dark_address(&record.signing_public_key, None)?;
                    reverse.remove(&addr.address);

                    // Remove from DHT
                    if let Some(dht) = &self.dht_client {
                        let key = Self::domain_to_dht_key(&domain);
                        let _ = dht.remove(&key);
                    }
                }
            }
        }

        Ok(count)
    }

    /// Validates a .dark domain name format
    fn is_valid_dark_domain(domain: &str) -> bool {
        // Must end with .dark
        if !domain.ends_with(".dark") {
            return false;
        }

        // Extract subdomain
        let subdomain = &domain[..domain.len() - 5];

        // Validation rules:
        // - Subdomain length between 3 and 63 chars
        // - Alphanumeric + hyphens
        // - Cannot start or end with hyphen
        // - No consecutive hyphens
        subdomain.len() >= 3
            && subdomain.len() <= 63
            && !subdomain.starts_with('-')
            && !subdomain.ends_with('-')
            && !subdomain.contains("--")
            && subdomain.chars().all(|c| c.is_alphanumeric() || c == '-')
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::thread_rng;

    // Mock DHT client for testing
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
            let mut storage = self
                .storage
                .write()
                .map_err(|_| DarkResolverError::StorageError)?;
            storage.insert(key.to_vec(), value.to_vec());
            Ok(())
        }

        fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, DarkResolverError> {
            let storage = self
                .storage
                .read()
                .map_err(|_| DarkResolverError::StorageError)?;
            Ok(storage.get(key).cloned())
        }

        fn remove(&self, key: &[u8]) -> Result<(), DarkResolverError> {
            let mut storage = self
                .storage
                .write()
                .map_err(|_| DarkResolverError::StorageError)?;
            storage.remove(key);
            Ok(())
        }
    }

    #[test]
    fn test_valid_dark_domains() {
        assert!(DarkResolver::is_valid_dark_domain("test.dark"));
        assert!(DarkResolver::is_valid_dark_domain("my-domain.dark"));
        assert!(DarkResolver::is_valid_dark_domain("node123.dark"));
        assert!(DarkResolver::is_valid_dark_domain("a2b.dark"));

        // Invalid cases
        assert!(!DarkResolver::is_valid_dark_domain("invalid"));
        assert!(!DarkResolver::is_valid_dark_domain(".dark"));
        assert!(!DarkResolver::is_valid_dark_domain("test.darknet"));
        assert!(!DarkResolver::is_valid_dark_domain("-test.dark"));
        assert!(!DarkResolver::is_valid_dark_domain("test-.dark"));
        assert!(!DarkResolver::is_valid_dark_domain("test--domain.dark"));
        assert!(!DarkResolver::is_valid_dark_domain("ab.dark")); // too short
        assert!(!DarkResolver::is_valid_dark_domain(&format!(
            "{}.dark",
            "a".repeat(64)
        ))); // too long
    }

    #[test]
    fn test_dark_address_generation() {
        let public_key = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

        // Generate without custom name
        let addr1 = DarkResolver::generate_dark_address(&public_key, None).unwrap();
        assert!(addr1.address.len() > 0);
        assert!(addr1.domain.ends_with(".dark"));

        // Generate with custom name
        let addr2 = DarkResolver::generate_dark_address(&public_key, Some("mynode")).unwrap();
        assert_eq!(addr2.domain, "mynode.dark");

        // Same public key should generate same address
        let addr3 = DarkResolver::generate_dark_address(&public_key, None).unwrap();
        assert_eq!(addr1.address, addr3.address);
    }

    #[test]
    fn test_domain_registration_and_resolution() {
        let mut rng = thread_rng();
        let resolver = DarkResolver::with_dht(Arc::new(MockDhtClient::new()));
        let owner_id = PeerId::random();
        let addresses = vec![
            NetworkAddress::new([1, 2, 3, 4], 8080),
            NetworkAddress::new([5, 6, 7, 8], 9090),
        ];

        // Register domain
        let dark_addr = resolver
            .register_domain(
                Some("testnode"),
                addresses.clone(),
                Some("Test Node".to_string()),
                3600, // 1 hour TTL
                owner_id.clone(),
                &mut rng,
            )
            .unwrap();

        assert_eq!(dark_addr.domain, "testnode.dark");

        // Lookup domain
        let record = resolver.lookup_domain(&dark_addr.domain).unwrap();
        assert_eq!(record.addresses, addresses);
        assert_eq!(record.alias, Some("Test Node".to_string()));
        assert_eq!(record.owner_id, owner_id);
        assert_eq!(record.ttl, 3600);

        // Resolve addresses
        let resolved = resolver.resolve_addresses(&dark_addr.domain).unwrap();
        assert_eq!(resolved, addresses);

        // Try to register same custom name (should fail)
        let result = resolver.register_domain(
            Some("testnode"),
            vec![],
            None,
            3600,
            PeerId::random(),
            &mut rng,
        );
        assert!(matches!(result, Err(DarkResolverError::DomainExists)));
    }

    #[test]
    fn test_address_book() {
        let resolver = DarkResolver::new();
        let dark_addr = DarkAddress {
            address: "3HGvnkH2VwR3cD8r7shs7V".to_string(),
            domain: "mynode.dark".to_string(),
        };

        // Add to address book
        resolver
            .add_to_address_book(
                "Alice's Node".to_string(),
                dark_addr.clone(),
                Some("Primary node".to_string()),
            )
            .unwrap();

        // Lookup by name
        let entry = resolver.lookup_address_book("Alice's Node").unwrap();
        assert_eq!(entry.dark_address, dark_addr);
        assert_eq!(entry.notes, Some("Primary node".to_string()));

        // List all entries
        let entries = resolver.list_address_book().unwrap();
        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn test_domain_expiration() {
        let mut rng = thread_rng();
        let resolver = DarkResolver::new();
        let owner_id = PeerId::random();

        // Create an already expired record
        let signing_keypair = MlDsaKeyPair::generate(&mut rng).unwrap();
        let (kem_public, _) = MlKem768::keygen().unwrap();

        let mut record = DarkDomainRecord {
            signing_public_key: signing_keypair.public_key().to_vec(),
            encryption_public_key: kem_public.as_bytes().to_vec(),
            addresses: vec![NetworkAddress::new([1, 2, 3, 4], 8080)],
            alias: None,
            ttl: 60,
            registered_at: 1000,
            expires_at: 1060, // Already expired
            owner_id,
            signature: vec![],
            metadata: HashMap::new(),
        };

        // Sign the record
        record.sign(&signing_keypair).unwrap();

        // Manually insert expired record
        {
            let mut domains = resolver.domains.write().unwrap();
            domains.insert("expired.dark".to_string(), record);
        }

        // Try to lookup - should fail with DomainExpired
        let result = resolver.lookup_domain("expired.dark");
        assert!(matches!(result, Err(DarkResolverError::DomainExpired)));

        // Cleanup should remove it
        let removed = resolver.cleanup_expired().unwrap();
        assert_eq!(removed, 1);

        // Should now be not found
        let result = resolver.lookup_domain("expired.dark");
        assert!(matches!(result, Err(DarkResolverError::DomainNotFound)));
    }

    #[test]
    fn test_signature_verification() {
        let mut rng = thread_rng();
        let signing_keypair = MlDsaKeyPair::generate(&mut rng).unwrap();
        let (kem_public, _) = MlKem768::keygen().unwrap();
        let owner_id = PeerId::random();

        // Create and sign a record
        let record = DarkDomainRecord::new(
            &signing_keypair,
            kem_public.as_bytes().to_vec(),
            vec![NetworkAddress::new([1, 2, 3, 4], 8080)],
            None,
            3600,
            owner_id,
        )
        .unwrap();

        // Verify signature should succeed
        assert!(record.verify_signature().is_ok());

        // Tamper with the record
        let mut tampered = record.clone();
        tampered.ttl = 7200; // Change TTL

        // Verification should fail
        assert!(tampered.verify_signature().is_err());
    }
}
