use qudag_exchange_core::{rUv, types::Timestamp, AgentStatus, FeeCalculator, FeeModel};

#[test]
fn test_fee_model_mathematical_requirements() {
    let model = FeeModel::new().expect("Should create fee model");

    // Test Case 1: New agent (t=0, u=0) should get F_min = 0.1%
    let agent1 = AgentStatus::new_unverified(Timestamp::new(0));
    let rate1 = model
        .calculate_fee_rate(&agent1, Timestamp::new(0))
        .expect("Should calculate rate");
    println!("Test 1 - New unverified agent: {:.3}%", rate1 * 100.0);
    assert!(
        (rate1 * 100.0 - 0.1).abs() < 0.01,
        "New agent should get ~0.1% fee"
    );

    // Test Case 2: Agent with u=5000 rUv, t=3 months should get ~0.32%
    let mut agent2 = AgentStatus::new_unverified(Timestamp::new(0));
    agent2.update_usage(5000);
    let three_months = 3 * 30 * 24 * 60 * 60; // 3 months in seconds
    let rate2 = model
        .calculate_fee_rate(&agent2, Timestamp::new(three_months))
        .expect("Should calculate rate");
    println!("Test 2 - Medium usage unverified: {:.3}%", rate2 * 100.0);
    assert!(
        (rate2 * 100.0 - 0.32).abs() < 0.1,
        "Medium usage agent should get ~0.32% fee"
    );

    // Test Case 3: High usage agent (u=50000, t=6 months) should approach F_max = 1.0%
    let mut agent3 = AgentStatus::new_unverified(Timestamp::new(0));
    agent3.update_usage(50000);
    let six_months = 6 * 30 * 24 * 60 * 60; // 6 months in seconds
    let rate3 = model
        .calculate_fee_rate(&agent3, Timestamp::new(six_months))
        .expect("Should calculate rate");
    println!("Test 3 - High usage unverified: {:.3}%", rate3 * 100.0);
    assert!(
        rate3 * 100.0 > 0.8 && rate3 * 100.0 <= 1.0,
        "High usage agent should approach 1.0%"
    );
}

#[test]
fn test_verified_agent_fees() {
    let model = FeeModel::new().expect("Should create fee model");

    // Test Case 4: New verified agent should get F_min_verified = 0.25%
    let agent4 = AgentStatus::new_verified(Timestamp::new(0), vec![1, 2, 3]);
    let rate4 = model
        .calculate_fee_rate(&agent4, Timestamp::new(0))
        .expect("Should calculate rate");
    println!("Test 4 - New verified agent: {:.3}%", rate4 * 100.0);
    assert!(
        (rate4 * 100.0 - 0.25).abs() < 0.01,
        "New verified agent should get ~0.25% fee"
    );

    // Test Case 5: Agent with u=20000, t=6 months should get ~0.28%
    let mut agent5 = AgentStatus::new_verified(Timestamp::new(0), vec![1, 2, 3]);
    agent5.update_usage(20000);
    let six_months = 6 * 30 * 24 * 60 * 60;
    let rate5 = model
        .calculate_fee_rate(&agent5, Timestamp::new(six_months))
        .expect("Should calculate rate");
    println!("Test 5 - High usage verified: {:.3}%", rate5 * 100.0);
    assert!(
        (rate5 * 100.0 - 0.28).abs() < 0.1,
        "High usage verified agent should get ~0.28% fee"
    );
}

#[test]
fn test_verified_advantage() {
    let model = FeeModel::new().expect("Should create fee model");

    // Verified agent with high usage should get lower fees than unverified
    let mut unverified = AgentStatus::new_unverified(Timestamp::new(0));
    unverified.update_usage(20000);
    let mut verified = AgentStatus::new_verified(Timestamp::new(0), vec![1, 2, 3]);
    verified.update_usage(20000);

    let six_months = 6 * 30 * 24 * 60 * 60;
    let unverified_rate = model
        .calculate_fee_rate(&unverified, Timestamp::new(six_months))
        .expect("Should calculate rate");
    let verified_rate = model
        .calculate_fee_rate(&verified, Timestamp::new(six_months))
        .expect("Should calculate rate");

    println!(
        "Verified advantage test - Unverified: {:.3}%, Verified: {:.3}%",
        unverified_rate * 100.0,
        verified_rate * 100.0
    );
    assert!(
        verified_rate < unverified_rate,
        "Verified agent should pay lower fees than unverified with same usage"
    );
}

#[test]
fn test_fee_amount_calculation() {
    let model = FeeModel::new().expect("Should create fee model");
    let agent = AgentStatus::new_unverified(Timestamp::new(0));

    // Test fee amount for 1000 rUv transaction
    let transaction_amount = rUv::new(1000);
    let fee_amount = model
        .calculate_fee_amount(transaction_amount, &agent, Timestamp::new(0))
        .expect("Should calculate fee amount");

    println!(
        "Fee amount test - Transaction: {} rUv, Fee: {} rUv",
        transaction_amount.amount(),
        fee_amount.amount()
    );

    // Should be 1000 * 0.001 = 1 rUv (minimum fee rate)
    assert_eq!(
        fee_amount.amount(),
        1,
        "Fee amount should be 1 rUv for 1000 rUv transaction at minimum rate"
    );
}

#[test]
fn test_edge_cases() {
    let model = FeeModel::new().expect("Should create fee model");

    // Test with time before first transaction (edge case)
    let agent = AgentStatus::new_unverified(Timestamp::new(1000));
    let rate = model
        .calculate_fee_rate(&agent, Timestamp::new(500))
        .expect("Should calculate rate");
    assert!(
        rate >= 0.0 && rate <= 1.0,
        "Fee rate should be in valid range even with edge case timing"
    );

    // Test zero usage
    let agent_zero = AgentStatus::new_unverified(Timestamp::new(0));
    let rate_zero = model
        .calculate_fee_rate(&agent_zero, Timestamp::new(0))
        .expect("Should calculate rate");
    assert!(
        (rate_zero - 0.001).abs() < 1e-10,
        "Zero usage should give minimum fee"
    );
}

#[test]
fn test_fee_calculator_examples() {
    let examples = FeeCalculator::calculate_examples();
    assert_eq!(examples.len(), 3, "Should return 3 examples");

    // First example should be minimum fee
    assert!(
        (examples[0].1 - 0.001).abs() < 1e-10,
        "First example should be minimum fee"
    );

    // Print examples for verification
    for (desc, rate) in examples {
        println!("Example: {}: {:.4}% ({:.6} rate)", desc, rate * 100.0, rate);
    }
}
