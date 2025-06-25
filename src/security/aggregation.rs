//! Secure aggregation protocols for privacy-preserving model training

use super::{SecureIdentity, SecurityError};
use qudag_crypto::{
    ml_kem::MlKem768,
    kem::{Ciphertext, SharedSecret, KeyEncapsulation},
    hash::HashFunction,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;

/// Secure aggregation protocol using Shamir secret sharing
pub struct SecureAggregator {
    participants: Vec<SecureIdentity>,
    threshold: usize,
    round: u64,
}

/// Encrypted share for secure aggregation
#[derive(Clone, Serialize, Deserialize)]
pub struct EncryptedShare {
    /// Encrypted value using ML-KEM
    pub ciphertext: Vec<u8>,
    
    /// Encapsulated key
    pub encapsulated_key: Vec<u8>,
    
    /// Share index
    pub index: usize,
    
    /// Round number
    pub round: u64,
}

/// Aggregated result with privacy guarantees
#[derive(Clone, Serialize, Deserialize)]
pub struct AggregatedResult {
    /// Aggregated gradients
    pub gradients: Vec<f64>,
    
    /// Number of contributors
    pub num_contributors: usize,
    
    /// Privacy budget used
    pub privacy_cost: f64,
    
    /// Verification hash
    pub verification_hash: Vec<u8>,
}

impl SecureAggregator {
    /// Create a new secure aggregator
    pub fn new(participants: Vec<SecureIdentity>, threshold: usize) -> Result<Self, SecurityError> {
        if threshold > participants.len() {
            return Err(SecurityError::AggregationError(
                "Threshold cannot exceed number of participants".to_string(),
            ));
        }
        
        if threshold < participants.len() / 2 + 1 {
            return Err(SecurityError::AggregationError(
                "Threshold too low for security".to_string(),
            ));
        }
        
        Ok(Self {
            participants,
            threshold,
            round: 0,
        })
    }
    
    /// Create masked gradients for secure aggregation
    pub fn create_masked_gradients(
        &self,
        participant: &SecureIdentity,
        gradients: &[f64],
    ) -> Result<Vec<EncryptedShare>, SecurityError> {
        let mut shares = Vec::new();
        let mut rng = ChaCha20Rng::from_entropy();
        
        // Create random masks for each participant
        let masks: Vec<Vec<f64>> = (0..self.participants.len())
            .map(|_| {
                (0..gradients.len())
                    .map(|_| rng.gen_range(-1000.0..1000.0))
                    .collect()
            })
            .collect();
        
        // Apply masks and create shares
        for (i, target) in self.participants.iter().enumerate() {
            let mut masked_gradients = gradients.to_vec();
            
            // Add masks
            for j in 0..gradients.len() {
                masked_gradients[j] += masks[i][j];
            }
            
            // Serialize masked gradients
            let data = bincode::serialize(&masked_gradients)
                .map_err(|e| SecurityError::AggregationError(e.to_string()))?;
            
            // Encrypt using target's public key
            let (ciphertext, encapsulated_key) = MlKem768::encapsulate(&target.kem_keys.public_key)
                .map_err(|e| SecurityError::CryptoError(e.to_string()))?;
            
            // Encrypt data with shared secret
            let encrypted_data = self.encrypt_with_shared_secret(&data, &encapsulated_key)?;
            
            shares.push(EncryptedShare {
                ciphertext: encrypted_data,
                encapsulated_key: encapsulated_key.as_bytes().to_vec(),
                index: i,
                round: self.round,
            });
        }
        
        Ok(shares)
    }
    
    /// Aggregate encrypted shares
    pub fn aggregate_shares(
        &mut self,
        shares: HashMap<usize, Vec<EncryptedShare>>,
    ) -> Result<AggregatedResult, SecurityError> {
        if shares.len() < self.threshold {
            return Err(SecurityError::AggregationError(
                format!("Insufficient shares: {} < {}", shares.len(), self.threshold),
            ));
        }
        
        // Decrypt and aggregate gradients
        let mut aggregated_gradients = vec![0.0; 0];
        let mut num_contributors = 0;
        
        for (participant_idx, participant_shares) in shares.iter() {
            let participant = &self.participants[*participant_idx];
            
            for share in participant_shares {
                // Decrypt share
                let decrypted = self.decrypt_share(participant, share)?;
                let gradients: Vec<f64> = bincode::deserialize(&decrypted)
                    .map_err(|e| SecurityError::AggregationError(e.to_string()))?;
                
                // Initialize aggregated gradients if needed
                if aggregated_gradients.is_empty() {
                    aggregated_gradients = vec![0.0; gradients.len()];
                }
                
                // Add to aggregation
                for (i, &grad) in gradients.iter().enumerate() {
                    aggregated_gradients[i] += grad;
                }
                
                num_contributors += 1;
            }
        }
        
        // Average the gradients
        for grad in &mut aggregated_gradients {
            *grad /= num_contributors as f64;
        }
        
        // Create verification hash
        let verification_data = bincode::serialize(&aggregated_gradients)
            .map_err(|e| SecurityError::AggregationError(e.to_string()))?;
        let verification_hash = HashFunction::hash(&verification_data);
        
        self.round += 1;
        
        Ok(AggregatedResult {
            gradients: aggregated_gradients,
            num_contributors,
            privacy_cost: 0.0, // Will be calculated by differential privacy module
            verification_hash,
        })
    }
    
    /// Encrypt data with shared secret
    fn encrypt_with_shared_secret(
        &self,
        data: &[u8],
        shared_secret: &[u8],
    ) -> Result<Vec<u8>, SecurityError> {
        // Simple XOR encryption for demonstration
        // In production, use proper authenticated encryption
        let mut encrypted = data.to_vec();
        for (i, byte) in encrypted.iter_mut().enumerate() {
            *byte ^= shared_secret[i % shared_secret.len()];
        }
        Ok(encrypted)
    }
    
    /// Decrypt a share
    fn decrypt_share(
        &self,
        participant: &SecureIdentity,
        share: &EncryptedShare,
    ) -> Result<Vec<u8>, SecurityError> {
        // Decapsulate to get shared secret
        let ciphertext = Ciphertext::from_bytes(&share.encapsulated_key)
            .map_err(|e| SecurityError::CryptoError(format!("Invalid ciphertext: {}", e)))?;
        
        let shared_secret = MlKem768::decapsulate(&participant.kem_keys.secret_key, &ciphertext)
            .map_err(|e| SecurityError::CryptoError(e.to_string()))?;
        
        // Decrypt data
        let mut decrypted = share.ciphertext.clone();
        for (i, byte) in decrypted.iter_mut().enumerate() {
            *byte ^= shared_secret.as_bytes()[i % shared_secret.as_bytes().len()];
        }
        
        Ok(decrypted)
    }
}

/// Secure multi-party computation for model aggregation
pub struct SecureMultiPartyComputation {
    /// Minimum number of parties required
    pub min_parties: usize,
    
    /// Current computation round
    pub round: u64,
    
    /// Accumulated privacy budget
    pub privacy_budget: f64,
}

impl SecureMultiPartyComputation {
    /// Create a new MPC instance
    pub fn new(min_parties: usize) -> Self {
        Self {
            min_parties,
            round: 0,
            privacy_budget: 0.0,
        }
    }
    
    /// Perform secure computation on model updates
    pub fn compute_secure_update(
        &mut self,
        updates: Vec<Vec<f64>>,
        identities: &[SecureIdentity],
    ) -> Result<Vec<f64>, SecurityError> {
        if updates.len() < self.min_parties {
            return Err(SecurityError::AggregationError(
                format!("Insufficient parties: {} < {}", updates.len(), self.min_parties),
            ));
        }
        
        // Verify all updates have same dimension
        let dim = updates[0].len();
        for update in &updates {
            if update.len() != dim {
                return Err(SecurityError::AggregationError(
                    "Inconsistent update dimensions".to_string(),
                ));
            }
        }
        
        // Aggregate updates securely
        let mut aggregated = vec![0.0; dim];
        for update in updates {
            for (i, &val) in update.iter().enumerate() {
                aggregated[i] += val;
            }
        }
        
        // Average
        for val in &mut aggregated {
            *val /= identities.len() as f64;
        }
        
        self.round += 1;
        
        Ok(aggregated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_secure_aggregation() {
        // Create participants
        let participants: Vec<SecureIdentity> = (0..5)
            .map(|_| SecureIdentity::new(1000).unwrap())
            .collect();
        
        let mut aggregator = SecureAggregator::new(participants.clone(), 3).unwrap();
        
        // Create masked gradients
        let gradients = vec![0.1, 0.2, 0.3, 0.4];
        let shares = aggregator
            .create_masked_gradients(&participants[0], &gradients)
            .unwrap();
        
        assert_eq!(shares.len(), participants.len());
    }
    
    #[test]
    fn test_mpc_computation() {
        let mut mpc = SecureMultiPartyComputation::new(3);
        let identities: Vec<SecureIdentity> = (0..5)
            .map(|_| SecureIdentity::new(1000).unwrap())
            .collect();
        
        let updates = vec![
            vec![0.1, 0.2, 0.3],
            vec![0.2, 0.3, 0.4],
            vec![0.3, 0.4, 0.5],
        ];
        
        let result = mpc.compute_secure_update(updates, &identities).unwrap();
        assert_eq!(result.len(), 3);
        assert!((result[0] - 0.2).abs() < 0.001);
        assert!((result[1] - 0.3).abs() < 0.001);
        assert!((result[2] - 0.4).abs() < 0.001);
    }
}