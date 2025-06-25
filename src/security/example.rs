//! Example usage of the comprehensive DAA security system

use super::*;
use aggregation::{SecureAggregator, SecureMultiPartyComputation};
use challenges::{ChallengeManager, ChallengeResponse, ChallengeProof};
use differential_privacy::{DifferentialPrivacy, LocalDifferentialPrivacy};
use integrity::{ModelIntegrityVerifier, ModelMetadata, LayerHash};
use staking::{StakingPool, StakingParameters, SlashingReason, ParticipationUpdate};

/// Example of secure federated learning round
pub async fn secure_federated_learning_round() -> Result<(), SecurityError> {
    println!("=== DAA Secure Federated Learning Example ===");
    
    // 1. Initialize security components
    let security_config = SecurityConfig {
        min_stake: 1000,
        slashing_rate: 0.1,
        privacy_epsilon: 1.0,
        privacy_delta: 1e-6,
        challenge_frequency: 10,
        max_aggregation_participants: 100,
    };
    
    let security_manager = SecurityManager::new(security_config.clone());
    let staking_pool = StakingPool::new(StakingParameters::default());
    let mut challenge_manager = ChallengeManager::new();
    let mut model_verifier = ModelIntegrityVerifier::new();
    
    // 2. Register participants with post-quantum identities
    println!("\n1. Creating secure identities with post-quantum cryptography...");
    let mut participants = Vec::new();
    
    for i in 0..5 {
        let identity = SecureIdentity::new(2000)?;
        println!("  - Participant {} fingerprint: {:?}", i, identity.fingerprint);
        
        // Register with security manager
        security_manager.register_participant(identity.clone())?;
        
        // Stake tokens
        staking_pool.stake(&identity.fingerprint, 2000, 1)?;
        
        // Add as trusted signer for model verification
        model_verifier.add_trusted_signer(
            identity.fingerprint.clone(),
            identity.signing_keys.public_key.clone(),
        );
        
        participants.push(identity);
    }
    
    // 3. Secure aggregation setup
    println!("\n2. Setting up secure aggregation with threshold 3/5...");
    let mut aggregator = SecureAggregator::new(participants.clone(), 3)?;
    
    // 4. Differential privacy setup
    println!("\n3. Initializing differential privacy (ε=1.0, δ=1e-6)...");
    let mut dp = DifferentialPrivacy::new(
        security_config.privacy_epsilon,
        security_config.privacy_delta,
        10.0, // Total privacy budget
    )?;
    
    // 5. Training round with secure aggregation
    println!("\n4. Performing secure model training round...");
    
    // Each participant computes gradients
    let mut all_shares = HashMap::new();
    
    for (i, participant) in participants.iter().enumerate() {
        // Simulate gradient computation
        let gradients = vec![0.1 * i as f64, 0.2 * i as f64, 0.3 * i as f64, 0.4 * i as f64];
        
        // Apply differential privacy
        let private_gradients = dp.privatize_gradients(&gradients, 1000)?;
        println!("  - Participant {} computed private gradients", i);
        
        // Create masked shares for secure aggregation
        let shares = aggregator.create_masked_gradients(participant, &private_gradients)?;
        all_shares.insert(i, shares);
        
        // Update participation metrics
        staking_pool.update_participation(
            &participant.fingerprint,
            ParticipationUpdate::RoundParticipated,
        )?;
    }
    
    // 6. Aggregate gradients securely
    println!("\n5. Aggregating gradients with secure multi-party computation...");
    let aggregated_result = aggregator.aggregate_shares(all_shares)?;
    println!("  - Aggregated {} gradients", aggregated_result.num_contributors);
    println!("  - Result: {:?}", aggregated_result.gradients);
    
    // 7. Create model checkpoint with integrity verification
    println!("\n6. Creating model checkpoint with post-quantum signatures...");
    
    let model_data = b"Model parameters after aggregation";
    let metadata = ModelMetadata {
        round: 1,
        num_samples: 5000,
        loss: 0.25,
        accuracy: 0.92,
        privacy_budget_used: dp.used_budget,
        contributors: participants.iter().map(|p| p.fingerprint.clone()).collect(),
    };
    
    let layer_hashes = vec![
        LayerHash {
            index: 0,
            name: "embedding".to_string(),
            param_count: 10000,
            hash: vec![1; 32],
        },
        LayerHash {
            index: 1,
            name: "transformer".to_string(),
            param_count: 50000,
            hash: vec![2; 32],
        },
        LayerHash {
            index: 2,
            name: "output".to_string(),
            param_count: 1000,
            hash: vec![3; 32],
        },
    ];
    
    let checkpoint = model_verifier.create_checkpoint(
        &participants[0],
        model_data,
        metadata,
        layer_hashes,
    )?;
    
    println!("  - Checkpoint version: {}", checkpoint.version);
    println!("  - Merkle root: {:?}", checkpoint.merkle_root);
    
    // 8. Issue validation challenges
    println!("\n7. Issuing validation challenges...");
    
    for participant in participants.iter().take(2) {
        let challenge = challenge_manager.issue_challenge(
            participant.fingerprint.clone(),
            &security_config,
        );
        
        println!("  - Challenge {} issued to participant", challenge.id);
        
        // Simulate challenge response
        if let challenges::ChallengeType::ComputationalPuzzle { difficulty: _, seed } = &challenge.challenge_type {
            // Create a simple response
            let response = ChallengeResponse {
                challenge_id: challenge.id,
                proof: ChallengeProof::PuzzleSolution {
                    nonce: 12345,
                    hash: vec![0; 32],
                },
                timestamp: challenge.issued_at + 10,
                signature: participant.sign(b"response")?,
            };
            
            let result = challenge_manager.verify_response(&response, participant)?;
            
            if result.passed {
                staking_pool.update_participation(
                    &participant.fingerprint,
                    ParticipationUpdate::ChallengePass,
                )?;
                println!("    ✓ Challenge passed");
            } else {
                staking_pool.update_participation(
                    &participant.fingerprint,
                    ParticipationUpdate::ChallengeFail,
                )?;
                println!("    ✗ Challenge failed");
            }
        }
    }
    
    // 9. Distribute rewards
    println!("\n8. Distributing rewards based on participation...");
    let reward_distribution = staking_pool.distribute_rewards(1000, 2)?;
    
    for (participant, reward) in reward_distribution.rewards {
        println!("  - Participant rewarded: {} tokens", reward);
    }
    
    // 10. Demonstrate slashing for malicious behavior
    println!("\n9. Demonstrating slashing mechanism...");
    
    // Simulate malicious behavior detection
    let malicious_participant = &participants[4];
    let slashed_amount = staking_pool.slash(
        &malicious_participant.fingerprint,
        SlashingReason::InvalidComputation,
        vec![0; 32], // Evidence hash
        2,
    )?;
    
    println!("  - Slashed {} tokens for invalid computation", slashed_amount);
    
    // Also slash via security manager
    let additional_slash = security_manager.slash_participant(&malicious_participant.fingerprint)?;
    println!("  - Additional slash via security manager: {} tokens", additional_slash);
    
    // 11. Summary
    println!("\n=== Security Summary ===");
    println!("- Total staked: {} tokens", staking_pool.get_total_staked());
    println!("- Privacy budget used: {:.2}/{:.2}", dp.used_budget, dp.total_budget);
    println!("- Remaining privacy budget: {:.2}", dp.remaining_budget());
    println!("- Model checkpoint verified: {}", model_verifier.verify_checkpoint(&checkpoint)?);
    
    Ok(())
}

/// Example of local differential privacy for individual updates
pub fn local_differential_privacy_example() {
    println!("\n=== Local Differential Privacy Example ===");
    
    let ldp = LocalDifferentialPrivacy::new(2.0);
    
    // Binary data randomization
    let sensitive_bit = true;
    let randomized = ldp.randomize_binary(sensitive_bit);
    println!("Original: {}, Randomized: {}", sensitive_bit, randomized);
    
    // Continuous data randomization
    let sensitive_value = 42.0;
    let randomized_value = ldp.randomize_continuous(sensitive_value, 10.0);
    println!("Original: {:.2}, Randomized: {:.2}", sensitive_value, randomized_value);
}

/// Example integration with MCP server
pub fn mcp_integration_example() {
    println!("\n=== MCP Integration Example ===");
    println!("The security module can be exposed via MCP tools:");
    println!("- create_secure_identity: Generate post-quantum identity");
    println!("- verify_model_checkpoint: Verify model integrity");
    println!("- issue_validation_challenge: Issue challenge to participant");
    println!("- apply_differential_privacy: Add privacy to gradients");
    println!("- manage_stakes: Handle staking operations");
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_secure_federated_learning() {
        secure_federated_learning_round().await.unwrap();
    }
    
    #[test]
    fn test_local_dp() {
        local_differential_privacy_example();
    }
}