//! Shadow address implementation for stealth payments.
//!
//! This module implements a stealth address system that allows generating
//! one-time addresses for anonymous communication.

use rand::{thread_rng, Rng, RngCore};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use thiserror::Error;
use tokio::sync::{Mutex, RwLock};
use tokio::time::interval;
use x25519_dalek::{EphemeralSecret, PublicKey};

/// Errors that can occur during shadow address operations.
#[derive(Debug, Error)]
pub enum ShadowAddressError {
    /// Key generation failed
    #[error("Key generation failed")]
    KeyGenerationFailed,

    /// Invalid key format
    #[error("Invalid key format: {0}")]
    InvalidKeyFormat(String),

    /// Address resolution failed
    #[error("Address resolution failed: {0}")]
    ResolutionFailed(String),

    /// Cryptographic operation failed
    #[error("Cryptographic error: {0}")]
    CryptoError(String),
}

/// Shadow address components for stealth address generation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ShadowAddress {
    /// Public view key for address scanning
    pub view_key: Vec<u8>,

    /// Public spend key for payment authorization
    pub spend_key: Vec<u8>,

    /// Optional payment ID for transaction correlation
    pub payment_id: Option<[u8; 32]>,

    /// Address metadata including TTL and privacy features
    pub metadata: ShadowMetadata,

    /// Shadow-specific features
    pub shadow_features: ShadowFeatures,
}

/// Shadow-specific features for enhanced privacy
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ShadowFeatures {
    /// Indicates if this is a temporary address
    pub is_temporary: bool,

    /// Address derivation index for HD wallets
    pub derivation_index: Option<u32>,

    /// Stealth prefix for efficient scanning
    pub stealth_prefix: Option<[u8; 4]>,

    /// Indicates if address mixing is enabled
    pub mixing_enabled: bool,

    /// Address pool identifier
    pub pool_id: Option<String>,
}

/// Metadata for shadow addresses.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ShadowMetadata {
    /// Address version
    pub version: u8,

    /// Network identifier
    pub network: NetworkType,

    /// Optional expiration timestamp (Unix timestamp)
    pub expires_at: Option<u64>,

    /// Creation timestamp (Unix timestamp)
    pub created_at: u64,

    /// Last used timestamp (Unix timestamp)
    pub last_used: Option<u64>,

    /// Additional flags for privacy features
    pub flags: u32,

    /// Time-to-live in seconds
    pub ttl: Option<u64>,

    /// Usage count for rotation policies
    pub usage_count: u32,

    /// Maximum allowed uses (None = unlimited)
    pub max_uses: Option<u32>,
}

/// Network type for shadow addresses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetworkType {
    /// Main network
    Mainnet,
    /// Test network
    Testnet,
    /// Local development network
    Devnet,
}

impl fmt::Display for ShadowAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ShadowAddress({:?})", self.metadata)
    }
}

/// Generator for creating shadow addresses.
pub trait ShadowAddressGenerator {
    /// Generate a new shadow address.
    fn generate_address(&self, network: NetworkType) -> Result<ShadowAddress, ShadowAddressError>;

    /// Generate a temporary shadow address with TTL.
    fn generate_temporary_address(
        &self,
        network: NetworkType,
        ttl: Duration,
    ) -> Result<ShadowAddress, ShadowAddressError>;

    /// Generate a stealth address.
    fn generate_stealth_address(
        &self,
        network: NetworkType,
        recipient_view_key: &[u8],
        recipient_spend_key: &[u8],
    ) -> Result<ShadowAddress, ShadowAddressError>;

    /// Derive a one-time address from a shadow address.
    fn derive_address(&self, base: &ShadowAddress) -> Result<ShadowAddress, ShadowAddressError>;

    /// Derive address from master key using index.
    fn derive_from_master(
        &self,
        master_key: &[u8],
        index: u32,
    ) -> Result<ShadowAddress, ShadowAddressError>;

    /// Validate a shadow address.
    fn validate_address(&self, address: &ShadowAddress) -> Result<bool, ShadowAddressError>;
}

/// Resolver for shadow addresses.
pub trait ShadowAddressResolver {
    /// Resolve a shadow address to its one-time address.
    fn resolve_address(&self, address: &ShadowAddress) -> Result<Vec<u8>, ShadowAddressError>;

    /// Check if a one-time address belongs to a shadow address.
    fn check_address(
        &self,
        shadow: &ShadowAddress,
        onetime: &[u8],
    ) -> Result<bool, ShadowAddressError>;
}

/// Shadow address pool for managing multiple addresses
#[derive(Debug, Clone)]
pub struct ShadowAddressPool {
    /// Pool identifier
    pub id: String,

    /// Pool size limit
    pub max_size: usize,

    /// Active addresses in the pool
    pub addresses: Vec<ShadowAddress>,

    /// Pool creation time
    pub created_at: u64,

    /// Pool expiration time
    pub expires_at: Option<u64>,
}

/// Shadow address manager for comprehensive address lifecycle
pub struct ShadowAddressManager {
    /// Address generator
    generator: Arc<RwLock<DefaultShadowAddressHandler>>,

    /// Active addresses mapped by ID
    active_addresses: Arc<RwLock<HashMap<String, ShadowAddress>>>,

    /// Address pools for rotation
    address_pools: Arc<RwLock<HashMap<String, ShadowAddressPool>>>,

    /// Expired addresses for cleanup
    expired_addresses: Arc<RwLock<Vec<ShadowAddress>>>,

    /// Address rotation policies
    rotation_policies: Arc<RwLock<RotationPolicies>>,

    /// Cleanup task handle
    #[allow(dead_code)]
    cleanup_handle: Option<tokio::task::JoinHandle<()>>,
}

/// Address rotation policies
#[derive(Debug, Clone)]
pub struct RotationPolicies {
    /// Auto-rotate after N uses
    pub rotate_after_uses: Option<u32>,

    /// Auto-rotate after duration
    pub rotate_after_duration: Option<Duration>,

    /// Minimum addresses in pool
    pub min_pool_size: usize,

    /// Maximum addresses in pool
    pub max_pool_size: usize,
}

/// Default implementation of shadow address generation and resolution.
pub struct DefaultShadowAddressHandler {
    /// Network type
    network: NetworkType,

    /// Master seed for deterministic generation
    #[allow(dead_code)]
    master_seed: [u8; 32],

    /// Current derivation counter
    #[allow(dead_code)]
    derivation_counter: Mutex<u32>,
}

impl DefaultShadowAddressHandler {
    /// Create a new shadow address handler.
    pub fn new(network: NetworkType, master_seed: [u8; 32]) -> Self {
        Self {
            network,
            master_seed,
            derivation_counter: Mutex::new(0),
        }
    }

    /// Generate a random 32-byte seed.
    fn generate_seed(&self) -> [u8; 32] {
        let mut seed = [0u8; 32];
        thread_rng().fill_bytes(&mut seed);
        seed
    }

    /// Derive keys from seed using proper cryptographic derivation.
    fn derive_keys(&self, seed: &[u8; 32]) -> Result<(Vec<u8>, Vec<u8>), ShadowAddressError> {
        // Use SHA256 for key derivation
        let mut hasher = Sha256::new();
        hasher.update(b"SHADOW_VIEW_KEY");
        hasher.update(seed);
        let view_key = hasher.finalize().to_vec();

        let mut hasher = Sha256::new();
        hasher.update(b"SHADOW_SPEND_KEY");
        hasher.update(seed);
        let spend_key = hasher.finalize().to_vec();

        Ok((view_key, spend_key))
    }

    /// Generate stealth keys for one-time addresses.
    fn generate_stealth_keys(
        &self,
        recipient_view_key: &[u8],
        _recipient_spend_key: &[u8],
    ) -> Result<(Vec<u8>, Vec<u8>, [u8; 32]), ShadowAddressError> {
        // Generate ephemeral keypair
        let ephemeral_secret = EphemeralSecret::random_from_rng(thread_rng());
        let ephemeral_public = PublicKey::from(&ephemeral_secret);

        // Create shared secret
        let recipient_view_pubkey =
            PublicKey::from(<[u8; 32]>::try_from(recipient_view_key).map_err(|_| {
                ShadowAddressError::InvalidKeyFormat("Invalid view key length".into())
            })?);

        let shared_secret = ephemeral_secret.diffie_hellman(&recipient_view_pubkey);

        // Derive one-time keys
        let mut hasher = Sha256::new();
        hasher.update(shared_secret.as_bytes());
        hasher.update(b"STEALTH_VIEW");
        let stealth_view_key = hasher.finalize().to_vec();

        let mut hasher = Sha256::new();
        hasher.update(shared_secret.as_bytes());
        hasher.update(b"STEALTH_SPEND");
        let stealth_spend_key = hasher.finalize().to_vec();

        Ok((
            stealth_view_key,
            stealth_spend_key,
            ephemeral_public.to_bytes(),
        ))
    }

    /// Get current timestamp.
    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    /// Generate address ID.
    fn generate_address_id() -> String {
        let mut rng = thread_rng();
        let id: u64 = rng.gen();
        format!("shadow_{:016x}", id)
    }
}

impl ShadowAddressGenerator for DefaultShadowAddressHandler {
    fn generate_address(&self, network: NetworkType) -> Result<ShadowAddress, ShadowAddressError> {
        let seed = self.generate_seed();
        let (view_key, spend_key) = self.derive_keys(&seed)?;
        let current_time = Self::current_timestamp();

        Ok(ShadowAddress {
            view_key,
            spend_key,
            payment_id: None,
            metadata: ShadowMetadata {
                version: 1,
                network,
                expires_at: None,
                created_at: current_time,
                last_used: None,
                flags: 0,
                ttl: None,
                usage_count: 0,
                max_uses: None,
            },
            shadow_features: ShadowFeatures {
                is_temporary: false,
                derivation_index: None,
                stealth_prefix: None,
                mixing_enabled: false,
                pool_id: None,
            },
        })
    }

    fn generate_temporary_address(
        &self,
        network: NetworkType,
        ttl: Duration,
    ) -> Result<ShadowAddress, ShadowAddressError> {
        let seed = self.generate_seed();
        let (view_key, spend_key) = self.derive_keys(&seed)?;
        let current_time = Self::current_timestamp();
        let expires_at = current_time + ttl.as_secs();

        Ok(ShadowAddress {
            view_key,
            spend_key,
            payment_id: None,
            metadata: ShadowMetadata {
                version: 1,
                network,
                expires_at: Some(expires_at),
                created_at: current_time,
                last_used: None,
                flags: 0x01, // Temporary flag
                ttl: Some(ttl.as_secs()),
                usage_count: 0,
                max_uses: None,
            },
            shadow_features: ShadowFeatures {
                is_temporary: true,
                derivation_index: None,
                stealth_prefix: None,
                mixing_enabled: false,
                pool_id: None,
            },
        })
    }

    fn generate_stealth_address(
        &self,
        network: NetworkType,
        recipient_view_key: &[u8],
        recipient_spend_key: &[u8],
    ) -> Result<ShadowAddress, ShadowAddressError> {
        let (stealth_view_key, stealth_spend_key, ephemeral_pubkey) =
            self.generate_stealth_keys(recipient_view_key, recipient_spend_key)?;

        let current_time = Self::current_timestamp();

        // Generate stealth prefix for efficient scanning
        let mut hasher = Sha256::new();
        hasher.update(&ephemeral_pubkey);
        let hash = hasher.finalize();
        let stealth_prefix = [hash[0], hash[1], hash[2], hash[3]];

        Ok(ShadowAddress {
            view_key: stealth_view_key,
            spend_key: stealth_spend_key,
            payment_id: Some(ephemeral_pubkey),
            metadata: ShadowMetadata {
                version: 2, // Version 2 for stealth addresses
                network,
                expires_at: None,
                created_at: current_time,
                last_used: None,
                flags: 0x02, // Stealth flag
                ttl: None,
                usage_count: 0,
                max_uses: Some(1), // One-time use
            },
            shadow_features: ShadowFeatures {
                is_temporary: false,
                derivation_index: None,
                stealth_prefix: Some(stealth_prefix),
                mixing_enabled: true,
                pool_id: None,
            },
        })
    }

    fn derive_address(&self, base: &ShadowAddress) -> Result<ShadowAddress, ShadowAddressError> {
        let seed = self.generate_seed();
        let (view_key, spend_key) = self.derive_keys(&seed)?;
        let current_time = Self::current_timestamp();

        Ok(ShadowAddress {
            view_key,
            spend_key,
            payment_id: base.payment_id,
            metadata: ShadowMetadata {
                version: base.metadata.version,
                network: base.metadata.network,
                expires_at: base.metadata.expires_at,
                created_at: current_time,
                last_used: None,
                flags: base.metadata.flags,
                ttl: base.metadata.ttl,
                usage_count: 0,
                max_uses: base.metadata.max_uses,
            },
            shadow_features: base.shadow_features.clone(),
        })
    }

    fn derive_from_master(
        &self,
        master_key: &[u8],
        index: u32,
    ) -> Result<ShadowAddress, ShadowAddressError> {
        // Hierarchical deterministic derivation
        let mut hasher = Sha256::new();
        hasher.update(b"SHADOW_HD_DERIVE");
        hasher.update(master_key);
        hasher.update(&index.to_le_bytes());
        let derived_seed = hasher.finalize();

        let seed_array: [u8; 32] = derived_seed.into();
        let (view_key, spend_key) = self.derive_keys(&seed_array)?;
        let current_time = Self::current_timestamp();

        Ok(ShadowAddress {
            view_key,
            spend_key,
            payment_id: None,
            metadata: ShadowMetadata {
                version: 1,
                network: self.network,
                expires_at: None,
                created_at: current_time,
                last_used: None,
                flags: 0x04, // HD derived flag
                ttl: None,
                usage_count: 0,
                max_uses: None,
            },
            shadow_features: ShadowFeatures {
                is_temporary: false,
                derivation_index: Some(index),
                stealth_prefix: None,
                mixing_enabled: false,
                pool_id: None,
            },
        })
    }

    fn validate_address(&self, address: &ShadowAddress) -> Result<bool, ShadowAddressError> {
        // Check key lengths
        if address.view_key.len() != 32 || address.spend_key.len() != 32 {
            return Ok(false);
        }

        // Check expiration
        if let Some(expires_at) = address.metadata.expires_at {
            if Self::current_timestamp() > expires_at {
                return Ok(false);
            }
        }

        // Check usage limits
        if let Some(max_uses) = address.metadata.max_uses {
            if address.metadata.usage_count >= max_uses {
                return Ok(false);
            }
        }

        // Check network
        if address.metadata.network != self.network {
            return Ok(false);
        }

        Ok(true)
    }
}

impl ShadowAddressResolver for DefaultShadowAddressHandler {
    fn resolve_address(&self, address: &ShadowAddress) -> Result<Vec<u8>, ShadowAddressError> {
        // TODO: Implement proper resolution
        // This is a placeholder implementation
        let mut resolved = Vec::new();
        resolved.extend_from_slice(&address.view_key);
        resolved.extend_from_slice(&address.spend_key);
        if let Some(payment_id) = address.payment_id {
            resolved.extend_from_slice(&payment_id);
        }
        Ok(resolved)
    }

    fn check_address(
        &self,
        shadow: &ShadowAddress,
        onetime: &[u8],
    ) -> Result<bool, ShadowAddressError> {
        let resolved = self.resolve_address(shadow)?;
        Ok(resolved == onetime)
    }
}

impl ShadowAddressManager {
    /// Create a new shadow address manager.
    pub async fn new(network: NetworkType, master_seed: [u8; 32]) -> Self {
        let generator = Arc::new(RwLock::new(DefaultShadowAddressHandler::new(
            network,
            master_seed,
        )));
        let manager = Self {
            generator,
            active_addresses: Arc::new(RwLock::new(HashMap::new())),
            address_pools: Arc::new(RwLock::new(HashMap::new())),
            expired_addresses: Arc::new(RwLock::new(Vec::new())),
            rotation_policies: Arc::new(RwLock::new(RotationPolicies::default())),
            cleanup_handle: None,
        };

        // Start cleanup task
        let cleanup_manager = manager.clone();
        let cleanup_handle = tokio::spawn(async move {
            cleanup_manager.cleanup_task().await;
        });

        Self {
            cleanup_handle: Some(cleanup_handle),
            ..manager
        }
    }

    /// Clone for internal use (implements Clone manually for Arc fields).
    fn clone(&self) -> Self {
        Self {
            generator: self.generator.clone(),
            active_addresses: self.active_addresses.clone(),
            address_pools: self.address_pools.clone(),
            expired_addresses: self.expired_addresses.clone(),
            rotation_policies: self.rotation_policies.clone(),
            cleanup_handle: None,
        }
    }

    /// Create a new temporary address with auto-expiry.
    pub async fn create_temporary_address(
        &self,
        ttl: Duration,
    ) -> Result<ShadowAddress, ShadowAddressError> {
        let generator = self.generator.read().await;
        let address = generator.generate_temporary_address(generator.network, ttl)?;

        // Store in active addresses
        let address_id = DefaultShadowAddressHandler::generate_address_id();
        self.active_addresses
            .write()
            .await
            .insert(address_id, address.clone());

        Ok(address)
    }

    /// Create a stealth address.
    pub async fn create_stealth_address(
        &self,
        recipient_view_key: &[u8],
        recipient_spend_key: &[u8],
    ) -> Result<ShadowAddress, ShadowAddressError> {
        let generator = self.generator.read().await;
        let address = generator.generate_stealth_address(
            generator.network,
            recipient_view_key,
            recipient_spend_key,
        )?;

        // Stealth addresses are not stored (one-time use)
        Ok(address)
    }

    /// Create an address pool for rotation.
    pub async fn create_address_pool(
        &self,
        pool_id: String,
        size: usize,
        ttl: Option<Duration>,
    ) -> Result<(), ShadowAddressError> {
        let generator = self.generator.read().await;
        let mut addresses = Vec::new();

        for _ in 0..size {
            let mut address = if let Some(ttl) = ttl {
                generator.generate_temporary_address(generator.network, ttl)?
            } else {
                generator.generate_address(generator.network)?
            };

            address.shadow_features.pool_id = Some(pool_id.clone());
            addresses.push(address);
        }

        let current_time = DefaultShadowAddressHandler::current_timestamp();
        let pool = ShadowAddressPool {
            id: pool_id.clone(),
            max_size: size,
            addresses,
            created_at: current_time,
            expires_at: ttl.map(|d| current_time + d.as_secs()),
        };

        self.address_pools.write().await.insert(pool_id, pool);
        Ok(())
    }

    /// Get a random address from pool.
    pub async fn get_pool_address(&self, pool_id: &str) -> Option<ShadowAddress> {
        let pools = self.address_pools.read().await;
        if let Some(pool) = pools.get(pool_id) {
            if !pool.addresses.is_empty() {
                let mut rng = thread_rng();
                let index = rng.gen_range(0..pool.addresses.len());
                return Some(pool.addresses[index].clone());
            }
        }
        None
    }

    /// Rotate addresses in a pool.
    pub async fn rotate_pool(&self, pool_id: &str) -> Result<(), ShadowAddressError> {
        let mut pools = self.address_pools.write().await;
        if let Some(pool) = pools.get_mut(pool_id) {
            let generator = self.generator.read().await;
            let size = pool.max_size;
            let ttl = pool.expires_at.map(|exp| {
                let current = DefaultShadowAddressHandler::current_timestamp();
                Duration::from_secs(exp.saturating_sub(current))
            });

            // Move old addresses to expired
            let old_addresses = std::mem::take(&mut pool.addresses);
            self.expired_addresses.write().await.extend(old_addresses);

            // Generate new addresses
            for _ in 0..size {
                let mut address = if let Some(ttl) = ttl {
                    generator.generate_temporary_address(generator.network, ttl)?
                } else {
                    generator.generate_address(generator.network)?
                };

                address.shadow_features.pool_id = Some(pool_id.to_string());
                pool.addresses.push(address);
            }

            Ok(())
        } else {
            Err(ShadowAddressError::ResolutionFailed(
                "Pool not found".into(),
            ))
        }
    }

    /// Mark address as used.
    pub async fn mark_address_used(&self, address: &mut ShadowAddress) {
        address.metadata.usage_count += 1;
        address.metadata.last_used = Some(DefaultShadowAddressHandler::current_timestamp());

        // Check rotation policies
        let policies = self.rotation_policies.read().await;
        if let Some(max_uses) = policies.rotate_after_uses {
            if address.metadata.usage_count >= max_uses {
                if let Some(pool_id) = &address.shadow_features.pool_id {
                    let _ = self.rotate_pool(pool_id).await;
                }
            }
        }
    }

    /// Cleanup expired addresses.
    async fn cleanup_task(&self) {
        let mut cleanup_interval = interval(Duration::from_secs(60)); // Every minute

        loop {
            cleanup_interval.tick().await;

            // Clean expired addresses
            let current_time = DefaultShadowAddressHandler::current_timestamp();

            // Check active addresses
            let mut active = self.active_addresses.write().await;
            let expired: Vec<_> = active
                .iter()
                .filter(|(_, addr)| {
                    if let Some(expires_at) = addr.metadata.expires_at {
                        current_time > expires_at
                    } else {
                        false
                    }
                })
                .map(|(id, _)| id.clone())
                .collect();

            for id in expired {
                if let Some(addr) = active.remove(&id) {
                    self.expired_addresses.write().await.push(addr);
                }
            }

            // Clean expired pools
            let mut pools = self.address_pools.write().await;
            let expired_pools: Vec<_> = pools
                .iter()
                .filter(|(_, pool)| {
                    if let Some(expires_at) = pool.expires_at {
                        current_time > expires_at
                    } else {
                        false
                    }
                })
                .map(|(id, _)| id.clone())
                .collect();

            for id in expired_pools {
                pools.remove(&id);
            }

            // Limit expired address storage
            let mut expired = self.expired_addresses.write().await;
            if expired.len() > 1000 {
                expired.drain(0..500); // Keep last 500
            }
        }
    }
}

impl Default for RotationPolicies {
    fn default() -> Self {
        Self {
            rotate_after_uses: Some(100),
            rotate_after_duration: Some(Duration::from_secs(3600)), // 1 hour
            min_pool_size: 10,
            max_pool_size: 100,
        }
    }
}

/// Shadow address mixer for unlinkability
pub struct ShadowAddressMixer {
    /// Mixing rounds
    rounds: usize,

    /// Mixing delay
    delay: Duration,
}

impl ShadowAddressMixer {
    /// Create a new address mixer.
    pub fn new(rounds: usize, delay: Duration) -> Self {
        Self { rounds, delay }
    }

    /// Mix addresses for unlinkability.
    pub async fn mix_addresses(
        &self,
        addresses: Vec<ShadowAddress>,
    ) -> Result<Vec<ShadowAddress>, ShadowAddressError> {
        let mut mixed = addresses;

        for _round in 0..self.rounds {
            // Shuffle addresses
            let mut rng = thread_rng();
            use rand::seq::SliceRandom;
            mixed.shuffle(&mut rng);

            // Add mixing delay
            tokio::time::sleep(self.delay).await;

            // Apply mixing transformation
            mixed = mixed
                .into_iter()
                .map(|mut addr| {
                    // Update mixing metadata
                    addr.shadow_features.mixing_enabled = true;
                    addr.metadata.flags |= 0x08; // Mixed flag
                    addr
                })
                .collect();
        }

        Ok(mixed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use std::convert::TryInto;

    // Proptest strategy for generating network types
    fn arb_network_type() -> impl Strategy<Value = NetworkType> {
        prop_oneof![
            Just(NetworkType::Mainnet),
            Just(NetworkType::Testnet),
            Just(NetworkType::Devnet)
        ]
    }

    // Proptest strategy for generating shadow metadata
    fn arb_shadow_metadata() -> impl Strategy<Value = ShadowMetadata> {
        (
            arb_network_type(),
            any::<u8>(),
            any::<Option<u64>>(),
            any::<u32>(),
        )
            .prop_map(|(network, version, expires_at, flags)| ShadowMetadata {
                version,
                network,
                expires_at,
                flags,
            })
    }

    // Proptest strategy for generating shadow addresses
    fn arb_shadow_address() -> impl Strategy<Value = ShadowAddress> {
        (
            proptest::collection::vec(any::<u8>(), 32..64),
            proptest::collection::vec(any::<u8>(), 32..64),
            any::<Option<[u8; 32]>>(),
            arb_shadow_metadata(),
        )
            .prop_map(
                |(view_key, spend_key, payment_id, metadata)| ShadowAddress {
                    view_key,
                    spend_key,
                    payment_id,
                    metadata,
                },
            )
    }

    // Test helper to create a sample shadow address
    fn create_test_address() -> ShadowAddress {
        ShadowAddress {
            view_key: vec![1, 2, 3, 4],
            spend_key: vec![5, 6, 7, 8],
            payment_id: None,
            metadata: ShadowMetadata {
                version: 1,
                network: NetworkType::Testnet,
                expires_at: None,
                flags: 0,
            },
        }
    }

    #[test]
    fn test_shadow_address_display() {
        let addr = create_test_address();
        let display = format!("{}", addr);
        assert!(display.contains("ShadowAddress"));
    }

    #[test]
    fn test_shadow_address_serialize() {
        let addr = create_test_address();
        let serialized = serde_json::to_string(&addr).unwrap();
        let deserialized: ShadowAddress = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.view_key, addr.view_key);
        assert_eq!(deserialized.metadata.network, NetworkType::Testnet);
    }

    proptest! {
        #[test]
        fn test_address_generation(network in arb_network_type()) {
            let seed = [0u8; 32];
            let handler = DefaultShadowAddressHandler::new(network, seed);
            let addr = handler.generate_address(network).unwrap();

            prop_assert_eq!(addr.metadata.network, network);
            prop_assert!(!addr.view_key.is_empty());
            prop_assert!(!addr.spend_key.is_empty());
        }

        #[test]
        fn test_address_resolution(addr in arb_shadow_address()) {
            let seed = [0u8; 32];
            let handler = DefaultShadowAddressHandler::new(addr.metadata.network, seed);
            let resolved = handler.resolve_address(&addr).unwrap();

            // Check basic properties of resolved address
            prop_assert!(!resolved.is_empty());
            prop_assert!(resolved.len() >= addr.view_key.len() + addr.spend_key.len());
        }

        #[test]
        fn test_address_derivation(base in arb_shadow_address()) {
            let seed = [0u8; 32];
            let handler = DefaultShadowAddressHandler::new(base.metadata.network, seed);
            let derived = handler.derive_address(&base).unwrap();

            // Derived address should maintain certain properties from base
            prop_assert_eq!(derived.metadata.network, base.metadata.network);
            prop_assert_eq!(derived.metadata.version, base.metadata.version);
            prop_assert_eq!(derived.payment_id, base.payment_id);

            // But should have different keys
            prop_assert_ne!(derived.view_key, base.view_key);
            prop_assert_ne!(derived.spend_key, base.spend_key);
        }

        #[test]
        fn test_address_validation(addr in arb_shadow_address()) {
            let seed = [0u8; 32];
            let handler = DefaultShadowAddressHandler::new(addr.metadata.network, seed);
            let valid = handler.validate_address(&addr).unwrap();

            // Our current validation just checks for non-empty keys
            prop_assert_eq!(valid, !addr.view_key.is_empty() && !addr.spend_key.is_empty());
        }

        #[test]
        fn test_address_check(addr in arb_shadow_address()) {
            let seed = [0u8; 32];
            let handler = DefaultShadowAddressHandler::new(addr.metadata.network, seed);
            let resolved = handler.resolve_address(&addr).unwrap();
            let matches = handler.check_address(&addr, &resolved).unwrap();

            // An address should match its own resolution
            prop_assert!(matches);
        }
    }

    #[test]
    fn test_temporary_address_generation() {
        let seed = [0u8; 32];
        let handler = DefaultShadowAddressHandler::new(NetworkType::Testnet, seed);
        let ttl = Duration::from_secs(300);
        let addr = handler
            .generate_temporary_address(NetworkType::Testnet, ttl)
            .unwrap();

        assert!(addr.shadow_features.is_temporary);
        assert_eq!(addr.metadata.ttl, Some(300));
        assert!(addr.metadata.expires_at.is_some());
        assert_eq!(addr.metadata.flags & 0x01, 0x01); // Temporary flag
    }

    #[test]
    fn test_stealth_address_generation() {
        let seed = [0u8; 32];
        let handler = DefaultShadowAddressHandler::new(NetworkType::Testnet, seed);
        let view_key = [1u8; 32];
        let spend_key = [2u8; 32];

        let addr = handler
            .generate_stealth_address(NetworkType::Testnet, &view_key, &spend_key)
            .unwrap();

        assert_eq!(addr.metadata.version, 2); // Stealth version
        assert_eq!(addr.metadata.flags & 0x02, 0x02); // Stealth flag
        assert!(addr.shadow_features.stealth_prefix.is_some());
        assert!(addr.shadow_features.mixing_enabled);
        assert_eq!(addr.metadata.max_uses, Some(1)); // One-time use
    }

    #[test]
    fn test_hierarchical_derivation() {
        let master_key = [42u8; 32];
        let handler = DefaultShadowAddressHandler::new(NetworkType::Testnet, master_key);

        // Derive multiple addresses
        let addr1 = handler.derive_from_master(&master_key, 0).unwrap();
        let addr2 = handler.derive_from_master(&master_key, 1).unwrap();
        let addr3 = handler.derive_from_master(&master_key, 0).unwrap();

        // Same index should produce same address
        assert_eq!(addr1.view_key, addr3.view_key);
        assert_eq!(addr1.spend_key, addr3.spend_key);

        // Different indices should produce different addresses
        assert_ne!(addr1.view_key, addr2.view_key);
        assert_ne!(addr1.spend_key, addr2.spend_key);

        // Check derivation metadata
        assert_eq!(addr1.shadow_features.derivation_index, Some(0));
        assert_eq!(addr2.shadow_features.derivation_index, Some(1));
        assert_eq!(addr1.metadata.flags & 0x04, 0x04); // HD derived flag
    }

    #[test]
    fn test_address_expiration_validation() {
        let seed = [0u8; 32];
        let handler = DefaultShadowAddressHandler::new(NetworkType::Testnet, seed);

        // Create expired address
        let mut addr = create_test_address();
        addr.metadata.expires_at = Some(1); // Past timestamp

        let is_valid = handler.validate_address(&addr).unwrap();
        assert!(!is_valid);

        // Create future expiry address
        addr.metadata.expires_at = Some(u64::MAX);
        let is_valid = handler.validate_address(&addr).unwrap();
        assert!(is_valid);
    }

    #[test]
    fn test_usage_limit_validation() {
        let seed = [0u8; 32];
        let handler = DefaultShadowAddressHandler::new(NetworkType::Testnet, seed);

        // Create address with usage limit
        let mut addr = create_test_address();
        addr.metadata.max_uses = Some(5);
        addr.metadata.usage_count = 5;

        let is_valid = handler.validate_address(&addr).unwrap();
        assert!(!is_valid); // Reached limit

        addr.metadata.usage_count = 4;
        let is_valid = handler.validate_address(&addr).unwrap();
        assert!(is_valid); // Under limit
    }

    #[tokio::test]
    async fn test_shadow_address_manager() {
        let seed = [0u8; 32];
        let manager = ShadowAddressManager::new(NetworkType::Testnet, seed).await;

        // Create temporary address
        let temp_addr = manager
            .create_temporary_address(Duration::from_secs(60))
            .await
            .unwrap();
        assert!(temp_addr.shadow_features.is_temporary);
        assert!(temp_addr.metadata.expires_at.is_some());

        // Create address pool
        manager
            .create_address_pool("test_pool".to_string(), 3, Some(Duration::from_secs(120)))
            .await
            .unwrap();

        // Get address from pool
        let pool_addr = manager.get_pool_address("test_pool").await;
        assert!(pool_addr.is_some());

        // Rotate pool
        manager.rotate_pool("test_pool").await.unwrap();
    }

    #[tokio::test]
    async fn test_address_mixing() {
        let mixer = ShadowAddressMixer::new(2, Duration::from_millis(10));

        let addresses = vec![
            create_test_address(),
            create_test_address(),
            create_test_address(),
        ];

        let mixed = mixer.mix_addresses(addresses.clone()).await.unwrap();

        assert_eq!(mixed.len(), addresses.len());
        for addr in &mixed {
            assert!(addr.shadow_features.mixing_enabled);
            assert_eq!(addr.metadata.flags & 0x08, 0x08); // Mixed flag
        }
    }

    #[tokio::test]
    async fn test_address_usage_tracking() {
        let seed = [0u8; 32];
        let manager = ShadowAddressManager::new(NetworkType::Testnet, seed).await;

        let mut addr = create_test_address();
        let initial_count = addr.metadata.usage_count;

        manager.mark_address_used(&mut addr).await;

        assert_eq!(addr.metadata.usage_count, initial_count + 1);
        assert!(addr.metadata.last_used.is_some());
    }
}
