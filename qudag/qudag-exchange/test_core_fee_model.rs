use qudag_exchange_core::{FeeModel, AgentStatus, rUv, types::Timestamp};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing Core Fee Model Implementation");
    println!("========================================");
    
    let model = FeeModel::new()?;
    
    // Test 1: New unverified agent
    println!("\n📊 Test 1: New unverified agent (t=0, u=0)");
    let agent1 = AgentStatus::new_unverified(Timestamp::new(0));
    let rate1 = model.calculate_fee_rate(&agent1, Timestamp::new(0))?;
    println!("Fee rate: {:.3}% (expected: ~0.1%)", rate1 * 100.0);
    
    // Test 2: Unverified agent with usage and time
    println!("\n📊 Test 2: Unverified agent (u=5000, t=3 months)");
    let mut agent2 = AgentStatus::new_unverified(Timestamp::new(0));
    agent2.update_usage(5000);
    let three_months = 3 * 30 * 24 * 60 * 60;
    let rate2 = model.calculate_fee_rate(&agent2, Timestamp::new(three_months))?;
    println!("Fee rate: {:.3}% (expected: ~0.32%)", rate2 * 100.0);
    
    // Test 3: High usage unverified agent
    println!("\n📊 Test 3: High usage unverified agent (u=50000, t=6 months)");
    let mut agent3 = AgentStatus::new_unverified(Timestamp::new(0));
    agent3.update_usage(50000);
    let six_months = 6 * 30 * 24 * 60 * 60;
    let rate3 = model.calculate_fee_rate(&agent3, Timestamp::new(six_months))?;
    println!("Fee rate: {:.3}% (expected: approaching 1.0%)", rate3 * 100.0);
    
    // Test 4: New verified agent
    println!("\n📊 Test 4: New verified agent (t=0, u=0)");
    let agent4 = AgentStatus::new_verified(Timestamp::new(0), vec![1, 2, 3]);
    let rate4 = model.calculate_fee_rate(&agent4, Timestamp::new(0))?;
    println!("Fee rate: {:.3}% (expected: ~0.25%)", rate4 * 100.0);
    
    // Test 5: Verified agent with high usage
    println!("\n📊 Test 5: Verified agent (u=20000, t=6 months)");
    let mut agent5 = AgentStatus::new_verified(Timestamp::new(0), vec![1, 2, 3]);
    agent5.update_usage(20000);
    let rate5 = model.calculate_fee_rate(&agent5, Timestamp::new(six_months))?;
    println!("Fee rate: {:.3}% (expected: ~0.28%)", rate5 * 100.0);
    
    // Test fee amount calculation
    println!("\n📊 Test 6: Fee amount calculation");
    let transaction_amount = rUv::new(1000);
    let fee_amount = model.calculate_fee_amount(transaction_amount, &agent1, Timestamp::new(0))?;
    println!("Transaction: {} rUv, Fee: {} rUv", transaction_amount.amount(), fee_amount.amount());
    
    // Test examples from calculator
    println!("\n📊 Fee Calculator Examples:");
    let examples = qudag_exchange_core::FeeCalculator::calculate_examples();
    for (desc, rate) in examples {
        println!("  {}: {:.3}% ({:.6} rate)", desc, rate * 100.0, rate);
    }
    
    println!("\n✅ All core fee model tests completed successfully!");
    Ok(())
}