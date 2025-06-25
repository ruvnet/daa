//! Comprehensive security module for DAA with post-quantum cryptography,
//! secure aggregation, differential privacy, and economic incentives.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod aggregation;
pub mod challenges;
pub mod differential_privacy;
pub mod integrity;
pub mod staking;

use qudag_crypto::{
    ml_kem::MlKem768,
    ml_dsa::{MlDsa, MlDsaKeyPair, MlDsaPublicKey},
    kem::{KeyPair as KemKeyPair, SharedSecret},
    signature::DigitalSignature,
    fingerprint::Fingerprint,
    hash::HashFunction,
};

#[derive(Debug, Error)]
pub enum SecurityError {
    #[error("Cryptographic error: {0}")]
    CryptoError(String),
    
    #[error("Verification failed: {0}")]
    VerificationError(String),
    
    #[error("Stake insufficient: required {required}, got {actual}")]
    InsufficientStake { required: u64, actual: u64 },
    
    #[error("Challenge failed: {0}")]
    ChallengeFailed(String),
    
    #[error("Privacy budget exceeded")]
    PrivacyBudgetExceeded,
    
    #[error("Aggregation error: {0}")]
    AggregationError(String),
}

/// Security configuration for the DAA system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Minimum stake required for participation
    pub min_stake: u64,
    
    /// Slashing percentage for malicious behavior
    pub slashing_rate: f64,
    
    /// Differential privacy epsilon
    pub privacy_epsilon: f64,
    
    /// Differential privacy delta
    pub privacy_delta: f64,
    
    /// Challenge frequency (in rounds)
    pub challenge_frequency: u32,
    
    /// Maximum participants in secure aggregation
    pub max_aggregation_participants: usize,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            min_stake: 1000,
            slashing_rate: 0.1,
            privacy_epsilon: 1.0,
            privacy_delta: 1e-6,
            challenge_frequency: 10,
            max_aggregation_participants: 100,
        }
    }
}

/// Post-quantum secure identity for DAA participants
#[derive(Clone)]
pub struct SecureIdentity {
    /// ML-DSA key pair for signatures
    pub signing_keys: Arc<MlDsaKeyPair>,
    
    /// ML-KEM key pair for encryption
    pub kem_keys: Arc<KemKeyPair>,
    
    /// Public fingerprint
    pub fingerprint: Fingerprint,
    
    /// Stake amount
    pub stake: Arc<Mutex<u64>>,
}

impl SecureIdentity {
    /// Create a new secure identity with post-quantum keys
    pub fn new(initial_stake: u64) -> Result<Self, SecurityError> {
        // Generate ML-DSA signing keys
        let signing_keys = MlDsa::generate_keypair()
            .map_err(|e| SecurityError::CryptoError(e.to_string()))?;
        
        // Generate ML-KEM encryption keys
        let kem_keys = MlKem768::generate_keypair()
            .map_err(|e| SecurityError::CryptoError(e.to_string()))?;
        
        // Create fingerprint from public keys
        let mut fingerprint_data = Vec::new();
        fingerprint_data.extend(signing_keys.public_key.as_bytes());
        fingerprint_data.extend(kem_keys.public_key.as_bytes());
        
        let fingerprint = Fingerprint::new(&fingerprint_data)
            .map_err(|e| SecurityError::CryptoError(e.to_string()))?;
        
        Ok(Self {
            signing_keys: Arc::new(signing_keys),
            kem_keys: Arc::new(kem_keys),
            fingerprint,
            stake: Arc::new(Mutex::new(initial_stake)),
        })
    }
    
    /// Sign data using ML-DSA
    pub fn sign(&self, data: &[u8]) -> Result<Vec<u8>, SecurityError> {
        MlDsa::sign(&self.signing_keys, data)
            .map_err(|e| SecurityError::CryptoError(e.to_string()))
    }
    
    /// Verify a signature
    pub fn verify(
        public_key: &MlDsaPublicKey,
        data: &[u8],
        signature: &[u8],
    ) -> Result<bool, SecurityError> {
        MlDsa::verify(public_key, data, signature)
            .map_err(|e| SecurityError::CryptoError(e.to_string()))
    }
    
    /// Get current stake amount
    pub fn get_stake(&self) -> u64 {
        *self.stake.lock().unwrap()
    }
    
    /// Update stake (for rewards/slashing)
    pub fn update_stake(&self, delta: i64) -> Result<u64, SecurityError> {
        let mut stake = self.stake.lock().unwrap();
        if delta < 0 && stake.saturating_sub((-delta) as u64) == 0 {
            return Err(SecurityError::InsufficientStake {
                required: (-delta) as u64,
                actual: *stake,
            });
        }
        
        if delta > 0 {
            *stake = stake.saturating_add(delta as u64);
        } else {
            *stake = stake.saturating_sub((-delta) as u64);
        }
        
        Ok(*stake)
    }
}

/// Security manager for the DAA system
pub struct SecurityManager {
    config: SecurityConfig,
    identities: Arc<Mutex<HashMap<Fingerprint, SecureIdentity>>>,
}

impl SecurityManager {
    /// Create a new security manager
    pub fn new(config: SecurityConfig) -> Self {
        Self {
            config,
            identities: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// Register a new participant
    pub fn register_participant(
        &self,
        identity: SecureIdentity,
    ) -> Result<(), SecurityError> {
        if identity.get_stake() < self.config.min_stake {
            return Err(SecurityError::InsufficientStake {
                required: self.config.min_stake,
                actual: identity.get_stake(),
            });
        }
        
        let mut identities = self.identities.lock().unwrap();
        identities.insert(identity.fingerprint.clone(), identity);
        Ok(())
    }
    
    /// Slash a participant for malicious behavior
    pub fn slash_participant(
        &self,
        fingerprint: &Fingerprint,
    ) -> Result<u64, SecurityError> {
        let identities = self.identities.lock().unwrap();
        
        if let Some(identity) = identities.get(fingerprint) {
            let slash_amount = (identity.get_stake() as f64 * self.config.slashing_rate) as u64;
            identity.update_stake(-(slash_amount as i64))?;
            Ok(slash_amount)
        } else {
            Err(SecurityError::VerificationError(
                "Participant not found".to_string(),
            ))
        }
    }
    
    /// Reward a participant for good behavior
    pub fn reward_participant(
        &self,
        fingerprint: &Fingerprint,
        amount: u64,
    ) -> Result<u64, SecurityError> {
        let identities = self.identities.lock().unwrap();
        
        if let Some(identity) = identities.get(fingerprint) {
            identity.update_stake(amount as i64)
        } else {
            Err(SecurityError::VerificationError(
                "Participant not found".to_string(),
            ))
        }
    }
    
    /// Get all registered participants
    pub fn get_participants(&self) -> Vec<Fingerprint> {
        let identities = self.identities.lock().unwrap();
        identities.keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_secure_identity_creation() {
        let identity = SecureIdentity::new(1000).unwrap();
        assert_eq!(identity.get_stake(), 1000);
    }
    
    #[test]
    fn test_post_quantum_signing() {
        let identity = SecureIdentity::new(1000).unwrap();
        let data = b"Test message for DAA";
        
        let signature = identity.sign(data).unwrap();
        let verified = SecureIdentity::verify(
            &identity.signing_keys.public_key,
            data,
            &signature,
        )
        .unwrap();
        
        assert!(verified);
    }
    
    #[test]
    fn test_staking_and_slashing() {
        let config = SecurityConfig::default();
        let manager = SecurityManager::new(config);
        
        let identity = SecureIdentity::new(1000).unwrap();
        let fingerprint = identity.fingerprint.clone();
        
        manager.register_participant(identity).unwrap();
        
        // Test slashing
        let slashed = manager.slash_participant(&fingerprint).unwrap();
        assert_eq!(slashed, 100); // 10% of 1000
        
        // Test rewarding
        let new_stake = manager.reward_participant(&fingerprint, 200).unwrap();
        assert_eq!(new_stake, 1100); // 900 + 200
    }
}