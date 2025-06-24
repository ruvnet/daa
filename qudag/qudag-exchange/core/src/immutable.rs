//! Immutable Deployment System for QuDAG Exchange
//!
//! Provides optional immutable deployment mode where exchange configuration
//! can be locked using quantum-resistant signatures, preventing further
//! modifications and enabling governance-free operation.

#[cfg(not(feature = "std"))]
use alloc::{format, string::String, vec::Vec};

use crate::{
    fee_model::FeeModelParams,
    types::{Hash, Timestamp},
    Error, Result,
};
use serde::{Deserialize, Serialize};

/// Quantum-resistant signature for immutable deployment
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImmutableSignature {
    /// Algorithm used (e.g., "ML-DSA-87", "ML-DSA-65")
    pub algorithm: String,

    /// Public key of the signer
    pub public_key: Vec<u8>,

    /// The signature bytes
    pub signature: Vec<u8>,

    /// Hash of the signed configuration
    pub config_hash: Hash,
}

/// Immutable deployment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImmutableConfig {
    /// Whether immutable mode is enabled
    pub enabled: bool,

    /// Timestamp when system was locked (None if not locked)
    pub locked_at: Option<Timestamp>,

    /// Quantum-resistant signature that locks the configuration
    pub lock_signature: Option<ImmutableSignature>,

    /// Optional governance override key (for emergency situations)
    pub governance_key: Option<Vec<u8>>,

    /// Hash of the configuration that was locked
    pub locked_config_hash: Option<Hash>,

    /// Grace period in seconds before lock takes effect
    pub grace_period_seconds: u64,
}

impl Default for ImmutableConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            locked_at: None,
            lock_signature: None,
            governance_key: None,
            locked_config_hash: None,
            grace_period_seconds: 24 * 60 * 60, // 24 hours default grace period
        }
    }
}

impl ImmutableConfig {
    /// Create a new immutable config with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable immutable mode (but don't lock yet)
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Disable immutable mode (only allowed if not locked)
    pub fn disable(&mut self) -> Result<()> {
        if self.is_locked() {
            return Err(Error::Other(
                "Cannot disable immutable mode: system is locked".into(),
            ));
        }
        self.enabled = false;
        Ok(())
    }

    /// Check if the system is currently locked
    pub fn is_locked(&self) -> bool {
        self.enabled && self.locked_at.is_some() && self.lock_signature.is_some()
    }

    /// Check if the system is in grace period (locked but not yet enforced)
    pub fn is_in_grace_period(&self, current_time: Timestamp) -> bool {
        if let Some(locked_at) = self.locked_at {
            let grace_end = locked_at.value() + self.grace_period_seconds;
            current_time.value() < grace_end
        } else {
            false
        }
    }

    /// Check if immutable mode is actively enforced
    pub fn is_enforced(&self, current_time: Timestamp) -> bool {
        self.is_locked() && !self.is_in_grace_period(current_time)
    }

    /// Set grace period (only allowed if not locked)
    pub fn set_grace_period(&mut self, seconds: u64) -> Result<()> {
        if self.is_locked() {
            return Err(Error::Other(
                "Cannot change grace period: system is locked".into(),
            ));
        }
        self.grace_period_seconds = seconds;
        Ok(())
    }

    /// Set governance override key (only allowed if not locked)
    pub fn set_governance_key(&mut self, key: Vec<u8>) -> Result<()> {
        if self.is_locked() {
            return Err(Error::Other(
                "Cannot set governance key: system is locked".into(),
            ));
        }
        self.governance_key = Some(key);
        Ok(())
    }
}

/// System configuration that can be made immutable
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockableConfig {
    /// Fee model parameters
    pub fee_params: FeeModelParams,

    /// Maximum total supply allowed
    pub max_total_supply: u64,

    /// Minimum balance required for accounts
    pub min_account_balance: u64,

    /// Network chain ID
    pub chain_id: u64,

    /// Protocol version
    pub protocol_version: u32,

    /// Whether negative balances are allowed (for special accounts)
    pub allow_negative_balances: bool,
}

impl Default for LockableConfig {
    fn default() -> Self {
        Self {
            fee_params: FeeModelParams::default(),
            max_total_supply: u64::MAX,
            min_account_balance: 0,
            chain_id: 1,
            protocol_version: 1,
            allow_negative_balances: false,
        }
    }
}

impl LockableConfig {
    /// Calculate hash of the configuration for signing
    pub fn hash(&self) -> Result<Hash> {
        let bytes =
            bincode::serialize(self).map_err(|e| Error::SerializationError(e.to_string()))?;
        let hash = blake3::hash(&bytes);
        Ok(Hash::from_bytes(*hash.as_bytes()))
    }

    /// Validate that configuration is valid
    pub fn validate(&self) -> Result<()> {
        self.fee_params.validate()?;

        if self.max_total_supply == 0 {
            return Err(Error::Other(
                "max_total_supply must be greater than 0".into(),
            ));
        }

        if self.chain_id == 0 {
            return Err(Error::Other("chain_id must be greater than 0".into()));
        }

        if self.protocol_version == 0 {
            return Err(Error::Other(
                "protocol_version must be greater than 0".into(),
            ));
        }

        Ok(())
    }
}

/// Immutable deployment manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImmutableDeployment {
    /// Immutable configuration
    pub config: ImmutableConfig,

    /// System configuration that can be locked
    pub system_config: LockableConfig,
}

impl ImmutableDeployment {
    /// Create a new immutable deployment manager
    pub fn new() -> Self {
        Self {
            config: ImmutableConfig::new(),
            system_config: LockableConfig::default(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(system_config: LockableConfig) -> Result<Self> {
        system_config.validate()?;
        Ok(Self {
            config: ImmutableConfig::new(),
            system_config,
        })
    }

    /// Enable immutable mode
    pub fn enable_immutable_mode(&mut self) -> Result<()> {
        self.system_config.validate()?;
        self.config.enable();
        Ok(())
    }

    /// Lock the system with quantum-resistant signature
    #[cfg(feature = "std")]
    pub fn lock_system(
        &mut self,
        keypair: &qudag_crypto::MlDsaKeyPair,
        current_time: Timestamp,
    ) -> Result<()> {
        if !self.config.enabled {
            return Err(Error::Other("Immutable mode not enabled".into()));
        }

        if self.config.is_locked() {
            return Err(Error::Other("System is already locked".into()));
        }

        // Validate configuration before locking
        self.system_config.validate()?;

        // Calculate configuration hash
        let config_hash = self.system_config.hash()?;

        // Prepare message to sign (config hash + timestamp)
        let mut message = Vec::new();
        message.extend_from_slice(config_hash.as_bytes());
        message.extend_from_slice(&current_time.value().to_le_bytes());

        // Sign the message
        let signature = keypair
            .sign(&message, &mut rand::thread_rng())
            .map_err(|e| Error::Other(format!("Signing failed: {:?}", e)))?;

        let public_key = keypair
            .to_public_key()
            .map_err(|e| Error::Other(format!("Public key extraction failed: {:?}", e)))?;

        // Create immutable signature
        let immutable_sig = ImmutableSignature {
            algorithm: "ML-DSA-87".to_string(),
            public_key: public_key.as_bytes().to_vec(),
            signature,
            config_hash,
        };

        // Lock the system
        self.config.locked_at = Some(current_time);
        self.config.lock_signature = Some(immutable_sig);
        self.config.locked_config_hash = Some(config_hash);

        Ok(())
    }

    /// Verify the lock signature
    #[cfg(feature = "std")]
    pub fn verify_lock_signature(&self, current_time: Timestamp) -> Result<bool> {
        let sig_data = self
            .config
            .lock_signature
            .as_ref()
            .ok_or_else(|| Error::Other("No lock signature present".into()))?;

        let locked_at = self
            .config
            .locked_at
            .ok_or_else(|| Error::Other("No lock timestamp present".into()))?;

        // Recreate the signed message
        let mut message = Vec::new();
        message.extend_from_slice(sig_data.config_hash.as_bytes());
        message.extend_from_slice(&locked_at.value().to_le_bytes());

        // Create public key from bytes
        let public_key = qudag_crypto::MlDsaPublicKey::from_bytes(&sig_data.public_key)
            .map_err(|e| Error::Other(format!("Invalid public key: {:?}", e)))?;

        // Verify the signature
        match public_key.verify(&message, &sig_data.signature) {
            Ok(()) => {
                // Also verify that the config hash matches current config
                let current_hash = self.system_config.hash()?;
                Ok(current_hash == sig_data.config_hash)
            }
            Err(_) => Ok(false),
        }
    }

    /// Check if configuration changes are allowed
    pub fn can_modify_config(&self, current_time: Timestamp) -> bool {
        !self.config.is_enforced(current_time)
    }

    /// Update fee parameters (only allowed if not locked)
    pub fn update_fee_params(
        &mut self,
        params: FeeModelParams,
        current_time: Timestamp,
    ) -> Result<()> {
        if !self.can_modify_config(current_time) {
            return Err(Error::Other(
                "Cannot modify configuration: system is immutably locked".into(),
            ));
        }

        params.validate()?;
        self.system_config.fee_params = params;
        Ok(())
    }

    /// Update system configuration (only allowed if not locked)
    pub fn update_system_config(
        &mut self,
        config: LockableConfig,
        current_time: Timestamp,
    ) -> Result<()> {
        if !self.can_modify_config(current_time) {
            return Err(Error::Other(
                "Cannot modify configuration: system is immutably locked".into(),
            ));
        }

        config.validate()?;
        self.system_config = config;
        Ok(())
    }

    /// Get system status for display
    pub fn get_status(&self, current_time: Timestamp) -> ImmutableStatus {
        ImmutableStatus {
            enabled: self.config.enabled,
            locked: self.config.is_locked(),
            enforced: self.config.is_enforced(current_time),
            in_grace_period: self.config.is_in_grace_period(current_time),
            locked_at: self.config.locked_at,
            grace_period_seconds: self.config.grace_period_seconds,
            config_hash: self.config.locked_config_hash,
        }
    }

    /// Emergency governance override (only with governance key)
    #[cfg(feature = "std")]
    pub fn governance_override(
        &mut self,
        governance_keypair: &qudag_crypto::MlDsaKeyPair,
        current_time: Timestamp,
    ) -> Result<()> {
        let governance_key = self
            .config
            .governance_key
            .as_ref()
            .ok_or_else(|| Error::Other("No governance key set".into()))?;

        // Verify governance key matches
        let public_key = governance_keypair
            .to_public_key()
            .map_err(|e| Error::Other(format!("Governance key extraction failed: {:?}", e)))?;

        if public_key.as_bytes() != governance_key {
            return Err(Error::Other("Invalid governance key".into()));
        }

        // Unlock the system (emergency only)
        self.config.locked_at = None;
        self.config.lock_signature = None;
        self.config.locked_config_hash = None;

        Ok(())
    }
}

impl Default for ImmutableDeployment {
    fn default() -> Self {
        Self::new()
    }
}

/// Status information for immutable deployment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImmutableStatus {
    /// Whether immutable mode is enabled
    pub enabled: bool,
    /// Whether system is locked
    pub locked: bool,
    /// Whether immutable restrictions are enforced
    pub enforced: bool,
    /// Whether system is in grace period
    pub in_grace_period: bool,
    /// When system was locked
    pub locked_at: Option<Timestamp>,
    /// Grace period duration
    pub grace_period_seconds: u64,
    /// Hash of locked configuration
    pub config_hash: Option<Hash>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_immutable_config_lifecycle() {
        let mut config = ImmutableConfig::new();

        // Initially not enabled or locked
        assert!(!config.enabled);
        assert!(!config.is_locked());

        // Enable immutable mode
        config.enable();
        assert!(config.enabled);
        assert!(!config.is_locked()); // Not locked yet

        // Can disable if not locked
        config.disable().unwrap();
        assert!(!config.enabled);
    }

    #[test]
    fn test_grace_period() {
        let mut config = ImmutableConfig::new();
        config.enable();

        let lock_time = Timestamp::new(1000);
        config.locked_at = Some(lock_time);
        config.lock_signature = Some(ImmutableSignature {
            algorithm: "ML-DSA-87".to_string(),
            public_key: vec![1, 2, 3],
            signature: vec![4, 5, 6],
            config_hash: Hash::from_bytes([0u8; 32]),
        });

        // During grace period
        let grace_time = Timestamp::new(1000 + 12 * 60 * 60); // 12 hours later
        assert!(config.is_locked());
        assert!(config.is_in_grace_period(grace_time));
        assert!(!config.is_enforced(grace_time));

        // After grace period
        let post_grace = Timestamp::new(1000 + 25 * 60 * 60); // 25 hours later
        assert!(config.is_locked());
        assert!(!config.is_in_grace_period(post_grace));
        assert!(config.is_enforced(post_grace));
    }

    #[test]
    fn test_lockable_config_validation() {
        let mut config = LockableConfig::default();
        assert!(config.validate().is_ok());

        // Test invalid fee params
        config.fee_params.f_min = -0.1;
        assert!(config.validate().is_err());

        // Reset and test other fields
        config = LockableConfig::default();
        config.max_total_supply = 0;
        assert!(config.validate().is_err());

        config = LockableConfig::default();
        config.chain_id = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_lockable_config_hash() {
        let config1 = LockableConfig::default();
        let config2 = LockableConfig::default();

        // Same configs should have same hash
        assert_eq!(config1.hash().unwrap(), config2.hash().unwrap());

        // Different configs should have different hashes
        let mut config3 = LockableConfig::default();
        config3.chain_id = 42;
        assert_ne!(config1.hash().unwrap(), config3.hash().unwrap());
    }

    #[test]
    fn test_immutable_deployment_lifecycle() {
        let mut deployment = ImmutableDeployment::new();
        let current_time = Timestamp::new(1000);

        // Initially can modify
        assert!(deployment.can_modify_config(current_time));

        // Enable immutable mode
        deployment.enable_immutable_mode().unwrap();
        assert!(deployment.can_modify_config(current_time)); // Still can modify until locked

        // Update fee params should work before locking
        let mut new_params = FeeModelParams::default();
        new_params.f_min = 0.002;
        deployment
            .update_fee_params(new_params, current_time)
            .unwrap();
        assert_eq!(deployment.system_config.fee_params.f_min, 0.002);
    }

    #[test]
    fn test_config_modification_restrictions() {
        let mut deployment = ImmutableDeployment::new();
        deployment.enable_immutable_mode().unwrap();

        let current_time = Timestamp::new(1000);

        // Simulate locked state (without actual signature)
        deployment.config.locked_at = Some(current_time);
        deployment.config.lock_signature = Some(ImmutableSignature {
            algorithm: "ML-DSA-87".to_string(),
            public_key: vec![1, 2, 3],
            signature: vec![4, 5, 6],
            config_hash: Hash::from_bytes([0u8; 32]),
        });

        // Should not be able to modify after grace period
        let post_grace = Timestamp::new(current_time.value() + 25 * 60 * 60);
        assert!(!deployment.can_modify_config(post_grace));

        let new_params = FeeModelParams::default();
        let result = deployment.update_fee_params(new_params, post_grace);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("immutably locked"));
    }

    #[test]
    fn test_status_reporting() {
        let mut deployment = ImmutableDeployment::new();
        let current_time = Timestamp::new(1000);

        // Initial status
        let status = deployment.get_status(current_time);
        assert!(!status.enabled);
        assert!(!status.locked);
        assert!(!status.enforced);

        // After enabling
        deployment.enable_immutable_mode().unwrap();
        let status = deployment.get_status(current_time);
        assert!(status.enabled);
        assert!(!status.locked);
        assert!(!status.enforced);
    }
}
