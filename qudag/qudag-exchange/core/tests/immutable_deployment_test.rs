use qudag_exchange_core::{
    types::Timestamp, FeeModelParams, ImmutableDeployment, LockableConfig, Result,
};

#[test]
fn test_immutable_deployment_flow() -> Result<()> {
    println!("🔒 Testing Immutable Deployment Flow");

    // Initialize exchange with mutable configuration
    let mut deployment = ImmutableDeployment::new();
    assert!(
        !deployment.config.enabled,
        "Should start with immutable mode disabled"
    );
    assert!(!deployment.config.is_locked(), "Should start unlocked");

    // Test modifying fee parameters (should work when mutable)
    println!("✅ Step 1: Modifying parameters in mutable mode");
    deployment.system_config.fee_params.f_min = 0.002;
    deployment.system_config.fee_params.f_max = 0.012;

    // Enable immutable mode (but don't lock yet)
    println!("✅ Step 2: Enabling immutable mode");
    deployment.enable_immutable_mode()?;
    assert!(
        deployment.config.enabled,
        "Immutable mode should be enabled"
    );
    assert!(!deployment.config.is_locked(), "Should not be locked yet");

    // Test that we can still modify during grace period
    println!("✅ Step 3: Testing modifications before lock");
    deployment.system_config.fee_params.f_min = 0.0015;

    // Test grace period functionality
    println!("✅ Step 4: Testing grace period");
    let current_time = Timestamp::new(1000);
    deployment.config.locked_at = Some(current_time);
    deployment.config.grace_period_seconds = 3600; // 1 hour

    let during_grace = Timestamp::new(2000); // 1000 seconds later
    let after_grace = Timestamp::new(5000); // 4000 seconds later (after 1 hour grace)

    assert!(
        deployment.config.is_in_grace_period(during_grace),
        "Should be in grace period"
    );
    assert!(
        !deployment.config.is_in_grace_period(after_grace),
        "Should be past grace period"
    );

    // Test that modifications are blocked after grace period
    println!("✅ Step 5: Testing post-grace period restrictions");
    deployment.config.lock_signature = Some(create_mock_signature());
    assert!(deployment.config.is_locked(), "Should be locked");
    assert!(
        !deployment.config.is_enforced(during_grace),
        "Should not be enforced during grace"
    );
    assert!(
        deployment.config.is_enforced(after_grace),
        "Should be enforced after grace"
    );

    Ok(())
}

#[test]
fn test_configuration_validation() -> Result<()> {
    println!("🔍 Testing Configuration Validation");

    // Test valid configuration
    let mut config = LockableConfig::default();
    assert!(config.validate().is_ok(), "Default config should be valid");

    // Test invalid fee parameters
    config.fee_params.f_min = -0.1; // Invalid
    assert!(config.validate().is_err(), "Should reject negative f_min");

    // Reset and test invalid max supply
    config = LockableConfig::default();
    config.max_total_supply = 0; // Invalid
    assert!(
        config.validate().is_err(),
        "Should reject zero max_total_supply"
    );

    // Reset and test invalid chain ID
    config = LockableConfig::default();
    config.chain_id = 0; // Invalid
    assert!(config.validate().is_err(), "Should reject zero chain_id");

    println!("✅ All validation tests passed");
    Ok(())
}

#[test]
fn test_configuration_hashing() -> Result<()> {
    println!("🔢 Testing Configuration Hashing");

    let config1 = LockableConfig::default();
    let mut config2 = LockableConfig::default();

    // Same configs should have same hash
    let hash1 = config1.hash()?;
    let hash2 = config2.hash()?;
    assert_eq!(hash1, hash2, "Identical configs should have same hash");

    // Different configs should have different hashes
    config2.fee_params.f_min = 0.002;
    let hash3 = config2.hash()?;
    assert_ne!(
        hash1, hash3,
        "Different configs should have different hashes"
    );

    println!("✅ Configuration hashing works correctly");
    Ok(())
}

#[test]
fn test_immutable_config_state_transitions() -> Result<()> {
    println!("🔄 Testing Immutable Config State Transitions");

    let mut config = qudag_exchange_core::ImmutableConfig::new();

    // Test enable/disable
    assert!(!config.enabled, "Should start disabled");
    config.enable();
    assert!(config.enabled, "Should be enabled after enable()");

    config.disable()?;
    assert!(!config.enabled, "Should be disabled after disable()");

    // Test that disable fails when locked
    config.enable();
    config.locked_at = Some(Timestamp::new(1000));
    config.lock_signature = Some(create_mock_signature());

    assert!(config.is_locked(), "Should be locked");
    assert!(
        config.disable().is_err(),
        "Should not allow disable when locked"
    );

    println!("✅ State transitions work correctly");
    Ok(())
}

#[test]
fn test_grace_period_edge_cases() -> Result<()> {
    println!("⏰ Testing Grace Period Edge Cases");

    let mut config = qudag_exchange_core::ImmutableConfig::new();

    // Test grace period when not locked
    let current_time = Timestamp::new(1000);
    assert!(
        !config.is_in_grace_period(current_time),
        "Should not be in grace period when not locked"
    );

    // Test zero grace period
    config.locked_at = Some(Timestamp::new(1000));
    config.grace_period_seconds = 0;
    assert!(
        !config.is_in_grace_period(current_time),
        "Zero grace period should not be in grace period"
    );

    // Test exactly at grace period boundary
    config.grace_period_seconds = 100;
    let boundary_time = Timestamp::new(1100); // Exactly at boundary
    let just_after = Timestamp::new(1101); // Just after boundary

    assert!(
        !config.is_in_grace_period(boundary_time),
        "Should not be in grace at exact boundary"
    );
    assert!(
        !config.is_in_grace_period(just_after),
        "Should not be in grace after boundary"
    );

    println!("✅ Grace period edge cases handled correctly");
    Ok(())
}

#[test]
fn test_governance_key_management() -> Result<()> {
    println!("🔑 Testing Governance Key Management");

    let mut config = qudag_exchange_core::ImmutableConfig::new();

    // Test setting governance key when not locked
    let governance_key = vec![1, 2, 3, 4, 5];
    config.set_governance_key(governance_key.clone())?;
    assert_eq!(config.governance_key.as_ref().unwrap(), &governance_key);

    // Test that governance key cannot be changed when locked
    config.enable(); // Need to enable immutable mode first
    config.locked_at = Some(Timestamp::new(1000));
    config.lock_signature = Some(create_mock_signature());

    let new_key = vec![6, 7, 8, 9, 10];
    assert!(
        config.set_governance_key(new_key).is_err(),
        "Should not allow governance key change when locked"
    );

    println!("✅ Governance key management works correctly");
    Ok(())
}

// Helper function to create a mock signature for testing
fn create_mock_signature() -> qudag_exchange_core::ImmutableSignature {
    qudag_exchange_core::ImmutableSignature {
        algorithm: "ML-DSA-87".to_string(),
        public_key: vec![1, 2, 3, 4],
        signature: vec![5, 6, 7, 8],
        config_hash: qudag_exchange_core::types::Hash::from_bytes([0u8; 32]),
    }
}
