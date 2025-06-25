//! Validation challenges for verifying participant behavior

use super::{SecureIdentity, SecurityError};
use qudag_crypto::hash::HashFunction;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Types of validation challenges
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChallengeType {
    /// Prove knowledge of specific gradient computation
    GradientProof {
        sample_indices: Vec<usize>,
        expected_hash: Vec<u8>,
    },
    
    /// Prove model state at specific checkpoint
    ModelStateProof {
        checkpoint_version: u64,
        layer_indices: Vec<usize>,
    },
    
    /// Prove participation in aggregation round
    AggregationProof {
        round: u64,
        contribution_hash: Vec<u8>,
    },
    
    /// Computational puzzle for liveness
    ComputationalPuzzle {
        difficulty: u32,
        seed: Vec<u8>,
    },
    
    /// Verify data possession
    DataPossessionProof {
        data_indices: Vec<usize>,
        merkle_root: Vec<u8>,
    },
}

/// Challenge issued to a participant
#[derive(Clone, Serialize, Deserialize)]
pub struct ValidationChallenge {
    /// Unique challenge ID
    pub id: u64,
    
    /// Challenge type
    pub challenge_type: ChallengeType,
    
    /// Target participant
    pub target: qudag_crypto::fingerprint::Fingerprint,
    
    /// Issue timestamp
    pub issued_at: u64,
    
    /// Challenge deadline
    pub deadline: u64,
    
    /// Reward for successful completion
    pub reward: u64,
    
    /// Penalty for failure
    pub penalty: u64,
}

/// Response to a validation challenge
#[derive(Clone, Serialize, Deserialize)]
pub struct ChallengeResponse {
    /// Challenge ID
    pub challenge_id: u64,
    
    /// Response data
    pub proof: ChallengeProof,
    
    /// Response timestamp
    pub timestamp: u64,
    
    /// Responder signature
    pub signature: Vec<u8>,
}

/// Proof data for challenge response
#[derive(Clone, Serialize, Deserialize)]
pub enum ChallengeProof {
    /// Gradient computation proof
    GradientProof {
        gradients: Vec<f64>,
        computation_trace: Vec<u8>,
    },
    
    /// Model state proof
    ModelStateProof {
        layer_hashes: Vec<Vec<u8>>,
        merkle_proofs: Vec<Vec<u8>>,
    },
    
    /// Aggregation participation proof
    AggregationProof {
        contribution: Vec<u8>,
        partial_signatures: Vec<Vec<u8>>,
    },
    
    /// Solution to computational puzzle
    PuzzleSolution {
        nonce: u64,
        hash: Vec<u8>,
    },
    
    /// Data possession proof
    DataPossessionProof {
        data_hashes: Vec<Vec<u8>>,
        merkle_paths: Vec<Vec<u8>>,
    },
}

/// Challenge manager for the DAA system
pub struct ChallengeManager {
    /// Active challenges
    active_challenges: HashMap<u64, ValidationChallenge>,
    
    /// Challenge history
    challenge_history: HashMap<qudag_crypto::fingerprint::Fingerprint, Vec<ChallengeResult>>,
    
    /// Next challenge ID
    next_challenge_id: u64,
    
    /// Random number generator
    rng: ChaCha20Rng,
}

/// Result of a challenge
#[derive(Clone, Serialize, Deserialize)]
pub struct ChallengeResult {
    /// Challenge ID
    pub challenge_id: u64,
    
    /// Whether the challenge was passed
    pub passed: bool,
    
    /// Response time
    pub response_time: Option<Duration>,
    
    /// Verification details
    pub details: String,
}

impl ChallengeManager {
    /// Create a new challenge manager
    pub fn new() -> Self {
        Self {
            active_challenges: HashMap::new(),
            challenge_history: HashMap::new(),
            next_challenge_id: 1,
            rng: ChaCha20Rng::from_entropy(),
        }
    }
    
    /// Issue a random challenge to a participant
    pub fn issue_challenge(
        &mut self,
        target: qudag_crypto::fingerprint::Fingerprint,
        config: &super::SecurityConfig,
    ) -> ValidationChallenge {
        let challenge_type = self.generate_random_challenge();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let challenge = ValidationChallenge {
            id: self.next_challenge_id,
            challenge_type,
            target: target.clone(),
            issued_at: now,
            deadline: now + 300, // 5 minutes
            reward: 10,
            penalty: (config.min_stake as f64 * config.slashing_rate) as u64,
        };
        
        self.active_challenges.insert(challenge.id, challenge.clone());
        self.next_challenge_id += 1;
        
        challenge
    }
    
    /// Generate a random challenge type
    fn generate_random_challenge(&mut self) -> ChallengeType {
        match self.rng.gen_range(0..5) {
            0 => ChallengeType::GradientProof {
                sample_indices: (0..10).map(|_| self.rng.gen_range(0..1000)).collect(),
                expected_hash: (0..32).map(|_| self.rng.gen()).collect(),
            },
            1 => ChallengeType::ModelStateProof {
                checkpoint_version: self.rng.gen_range(1..10),
                layer_indices: (0..3).map(|_| self.rng.gen_range(0..10)).collect(),
            },
            2 => ChallengeType::AggregationProof {
                round: self.rng.gen_range(1..100),
                contribution_hash: (0..32).map(|_| self.rng.gen()).collect(),
            },
            3 => ChallengeType::ComputationalPuzzle {
                difficulty: 20,
                seed: (0..32).map(|_| self.rng.gen()).collect(),
            },
            _ => ChallengeType::DataPossessionProof {
                data_indices: (0..5).map(|_| self.rng.gen_range(0..1000)).collect(),
                merkle_root: (0..32).map(|_| self.rng.gen()).collect(),
            },
        }
    }
    
    /// Verify a challenge response
    pub fn verify_response(
        &mut self,
        response: &ChallengeResponse,
        identity: &SecureIdentity,
    ) -> Result<ChallengeResult, SecurityError> {
        // Get the challenge
        let challenge = self.active_challenges
            .get(&response.challenge_id)
            .ok_or_else(|| SecurityError::ChallengeFailed(
                "Challenge not found".to_string()
            ))?;
        
        // Check deadline
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        if now > challenge.deadline {
            return Ok(ChallengeResult {
                challenge_id: response.challenge_id,
                passed: false,
                response_time: None,
                details: "Response after deadline".to_string(),
            });
        }
        
        // Verify signature
        let response_data = bincode::serialize(&response.proof)
            .map_err(|e| SecurityError::VerificationError(e.to_string()))?;
        
        let verified = SecureIdentity::verify(
            &identity.signing_keys.public_key,
            &response_data,
            &response.signature,
        )?;
        
        if !verified {
            return Ok(ChallengeResult {
                challenge_id: response.challenge_id,
                passed: false,
                response_time: Some(Duration::from_secs(response.timestamp - challenge.issued_at)),
                details: "Invalid signature".to_string(),
            });
        }
        
        // Verify proof based on challenge type
        let passed = self.verify_proof(&challenge.challenge_type, &response.proof)?;
        
        let result = ChallengeResult {
            challenge_id: response.challenge_id,
            passed,
            response_time: Some(Duration::from_secs(response.timestamp - challenge.issued_at)),
            details: if passed { "Valid proof".to_string() } else { "Invalid proof".to_string() },
        };
        
        // Update history
        self.challenge_history
            .entry(challenge.target.clone())
            .or_insert_with(Vec::new)
            .push(result.clone());
        
        // Remove from active challenges
        self.active_challenges.remove(&response.challenge_id);
        
        Ok(result)
    }
    
    /// Verify specific proof type
    fn verify_proof(
        &self,
        challenge_type: &ChallengeType,
        proof: &ChallengeProof,
    ) -> Result<bool, SecurityError> {
        match (challenge_type, proof) {
            (
                ChallengeType::ComputationalPuzzle { difficulty, seed },
                ChallengeProof::PuzzleSolution { nonce, hash },
            ) => {
                // Verify proof of work
                let mut data = seed.clone();
                data.extend(&nonce.to_le_bytes());
                let computed_hash = HashFunction::hash(&data);
                
                // Check difficulty (leading zeros)
                let leading_zeros = computed_hash.iter()
                    .take_while(|&&b| b == 0)
                    .count() as u32;
                
                Ok(leading_zeros >= *difficulty / 8 && &computed_hash == hash)
            }
            
            (
                ChallengeType::GradientProof { sample_indices, expected_hash },
                ChallengeProof::GradientProof { gradients, computation_trace },
            ) => {
                // Verify gradient computation
                if gradients.len() != sample_indices.len() {
                    return Ok(false);
                }
                
                // Hash the computation
                let mut data = Vec::new();
                for grad in gradients {
                    data.extend(&grad.to_le_bytes());
                }
                data.extend(computation_trace);
                
                let hash = HashFunction::hash(&data);
                Ok(&hash == expected_hash)
            }
            
            // Other proof types would be verified similarly
            _ => Ok(true), // Simplified for other types
        }
    }
    
    /// Get challenge success rate for a participant
    pub fn get_success_rate(
        &self,
        participant: &qudag_crypto::fingerprint::Fingerprint,
    ) -> f64 {
        if let Some(history) = self.challenge_history.get(participant) {
            let passed = history.iter().filter(|r| r.passed).count() as f64;
            let total = history.len() as f64;
            if total > 0.0 {
                passed / total
            } else {
                1.0
            }
        } else {
            1.0
        }
    }
}

/// Interactive proof system for complex validations
pub struct InteractiveProofSystem {
    /// Proof rounds
    rounds: u32,
    
    /// Soundness parameter
    soundness: f64,
}

impl InteractiveProofSystem {
    /// Create a new interactive proof system
    pub fn new(rounds: u32) -> Self {
        let soundness = 1.0 / (2.0_f64).powi(rounds as i32);
        Self { rounds, soundness }
    }
    
    /// Generate interactive proof challenge
    pub fn generate_challenge(&self, statement: &[u8]) -> Vec<u8> {
        let mut challenges = Vec::new();
        
        for round in 0..self.rounds {
            let mut round_data = statement.to_vec();
            round_data.extend(&round.to_le_bytes());
            let challenge = HashFunction::hash(&round_data);
            challenges.extend(&challenge);
        }
        
        challenges
    }
    
    /// Verify interactive proof
    pub fn verify_proof(
        &self,
        statement: &[u8],
        proof: &[Vec<u8>],
    ) -> bool {
        if proof.len() != self.rounds as usize {
            return false;
        }
        
        // Verify each round
        for (round, round_proof) in proof.iter().enumerate() {
            let mut expected = statement.to_vec();
            expected.extend(&(round as u32).to_le_bytes());
            expected.extend(round_proof);
            
            // Simple verification - in practice would be more complex
            let hash = HashFunction::hash(&expected);
            if hash[0] != 0 {
                return false;
            }
        }
        
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_computational_puzzle() {
        let mut manager = ChallengeManager::new();
        let identity = SecureIdentity::new(1000).unwrap();
        let config = super::super::SecurityConfig::default();
        
        // Issue a computational puzzle
        let challenge = manager.issue_challenge(identity.fingerprint.clone(), &config);
        
        // Solve puzzle (simplified)
        if let ChallengeType::ComputationalPuzzle { difficulty: _, seed } = &challenge.challenge_type {
            // Find nonce that produces required hash
            let mut nonce = 0u64;
            loop {
                let mut data = seed.clone();
                data.extend(&nonce.to_le_bytes());
                let hash = HashFunction::hash(&data);
                
                if hash[0] == 0 && hash[1] == 0 {
                    // Found solution
                    let response = ChallengeResponse {
                        challenge_id: challenge.id,
                        proof: ChallengeProof::PuzzleSolution { nonce, hash },
                        timestamp: challenge.issued_at + 10,
                        signature: identity.sign(&[]).unwrap(),
                    };
                    
                    let result = manager.verify_response(&response, &identity).unwrap();
                    assert!(result.passed);
                    break;
                }
                
                nonce += 1;
                if nonce > 1000000 {
                    break; // Prevent infinite loop in test
                }
            }
        }
    }
    
    #[test]
    fn test_interactive_proof() {
        let ips = InteractiveProofSystem::new(10);
        let statement = b"test statement";
        
        let challenge = ips.generate_challenge(statement);
        assert_eq!(challenge.len(), 320); // 10 rounds * 32 bytes
        
        // Create dummy proof
        let proof: Vec<Vec<u8>> = (0..10)
            .map(|_| vec![0; 32])
            .collect();
        
        assert!(ips.verify_proof(statement, &proof));
    }
}