#!/usr/bin/env rust-script
//! Fee Model Testing Script
//! 
//! Tests the mathematical fee calculations for unverified and verified agents
//! according to the specifications:
//! 
//! **Unverified Agents:**
//! - New agent (t=0, u=0) should get F_min = 0.1%
//! - Agent with u=5000 rUv, t=3 months should get ~0.32%
//! - High usage agent (u=50000, t=6 months) should approach F_max = 1.0%
//! 
//! **Verified Agents:**
//! - New verified agent should get F_min_verified = 0.25%
//! - Verified agent with high usage should get lower fees
//! - Agent with u=20000, t=6 months should get ~0.28%

use std::process;

// Simulate the fee calculation logic from fee_model.rs
#[derive(Debug, Clone)]
struct FeeModelParams {
    f_min: f64,              // 0.1%
    f_max: f64,              // 1.0%
    f_min_verified: f64,     // 0.25%
    f_max_verified: f64,     // 0.50%
    time_constant_seconds: u64, // 3 months
    usage_threshold_ruv: u64,   // 10,000 rUv
}

impl Default for FeeModelParams {
    fn default() -> Self {
        Self {
            f_min: 0.001,              // 0.1%
            f_max: 0.010,              // 1.0%
            f_min_verified: 0.0025,    // 0.25%
            f_max_verified: 0.005,     // 0.50%
            time_constant_seconds: 3 * 30 * 24 * 60 * 60, // 3 months
            usage_threshold_ruv: 10_000, // 10,000 rUv
        }
    }
}

#[derive(Debug, Clone)]
struct AgentStatus {
    verified: bool,
    first_transaction_timestamp: u64,
    monthly_usage_ruv: u64,
}

impl AgentStatus {
    fn new_unverified(first_transaction: u64) -> Self {
        Self {
            verified: false,
            first_transaction_timestamp: first_transaction,
            monthly_usage_ruv: 0,
        }
    }
    
    fn new_verified(first_transaction: u64) -> Self {
        Self {
            verified: true,
            first_transaction_timestamp: first_transaction,
            monthly_usage_ruv: 0,
        }
    }
    
    fn update_usage(&mut self, monthly_usage: u64) {
        self.monthly_usage_ruv = monthly_usage;
    }
}

struct FeeModel {
    params: FeeModelParams,
}

impl FeeModel {
    fn new() -> Self {
        Self { params: FeeModelParams::default() }
    }
    
    fn calculate_fee_rate(&self, agent_status: &AgentStatus, current_time: u64) -> f64 {
        // Calculate time since first transaction in seconds
        let time_since_first = if current_time >= agent_status.first_transaction_timestamp {
            current_time - agent_status.first_transaction_timestamp
        } else {
            0
        };
        
        // Calculate time phase-in: α(t) = 1 - e^(-t/T)
        let alpha = self.time_phase_in(time_since_first as f64);
        
        // Calculate usage scaling: β(u) = 1 - e^(-u/U)
        let beta = self.usage_scaling(agent_status.monthly_usage_ruv as f64);
        
        let fee_rate = if agent_status.verified {
            // Verified fee: f_ver(u,t) = F_min_ver + (F_max_ver - F_min_ver) * α(t) * (1 - β(u))
            // Fee decreases with usage (rewards high throughput)
            self.params.f_min_verified 
                + (self.params.f_max_verified - self.params.f_min_verified) 
                * alpha * (1.0 - beta)
        } else {
            // Unverified fee: f_unv(u,t) = F_min + (F_max - F_min) * α(t) * β(u)
            // Fee increases with usage and time
            self.params.f_min 
                + (self.params.f_max - self.params.f_min) 
                * alpha * beta
        };
        
        fee_rate.max(0.0).min(1.0) // Clamp to [0, 1]
    }
    
    fn time_phase_in(&self, time_seconds: f64) -> f64 {
        let t_normalized = time_seconds / (self.params.time_constant_seconds as f64);
        1.0 - (-t_normalized).exp()
    }
    
    fn usage_scaling(&self, usage_ruv: f64) -> f64 {
        let u_normalized = usage_ruv / (self.params.usage_threshold_ruv as f64);
        1.0 - (-u_normalized).exp()
    }
}

fn main() {
    println!("🧪 QuDAG Exchange Fee Model Testing");
    println!("=====================================");
    
    let model = FeeModel::new();
    let mut test_results = Vec::new();
    let mut all_passed = true;
    
    // Test Cases for Unverified Agents
    println!("\n📊 Testing Unverified Agents:");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    // Test 1: New agent (t=0, u=0) should get F_min = 0.1%
    {
        let mut agent = AgentStatus::new_unverified(0);
        agent.update_usage(0);
        let fee_rate = model.calculate_fee_rate(&agent, 0);
        let fee_percentage = fee_rate * 100.0;
        
        println!("Test 1: New agent (t=0, u=0)");
        println!("  Expected: ~0.1%");
        println!("  Actual:   {:.3}%", fee_percentage);
        
        let passed = (fee_percentage - 0.1).abs() < 0.01; // Within 0.01%
        println!("  Result:   {}", if passed { "✅ PASS" } else { "❌ FAIL" });
        test_results.push(("Unverified new agent", passed));
        if !passed { all_passed = false; }
    }
    
    // Test 2: Agent with u=5000 rUv, t=3 months should get ~0.32%
    {
        let mut agent = AgentStatus::new_unverified(0);
        agent.update_usage(5000);
        let three_months = 3 * 30 * 24 * 60 * 60; // 3 months in seconds
        let fee_rate = model.calculate_fee_rate(&agent, three_months);
        let fee_percentage = fee_rate * 100.0;
        
        println!("\nTest 2: Agent with u=5000 rUv, t=3 months");
        println!("  Expected: ~0.32%");
        println!("  Actual:   {:.3}%", fee_percentage);
        
        let passed = (fee_percentage - 0.32).abs() < 0.1; // Within 0.1%
        println!("  Result:   {}", if passed { "✅ PASS" } else { "❌ FAIL" });
        test_results.push(("Unverified medium usage", passed));
        if !passed { all_passed = false; }
    }
    
    // Test 3: High usage agent (u=50000, t=6 months) should approach F_max = 1.0%
    {
        let mut agent = AgentStatus::new_unverified(0);
        agent.update_usage(50000);
        let six_months = 6 * 30 * 24 * 60 * 60; // 6 months in seconds
        let fee_rate = model.calculate_fee_rate(&agent, six_months);
        let fee_percentage = fee_rate * 100.0;
        
        println!("\nTest 3: High usage agent (u=50000, t=6 months)");
        println!("  Expected: ~1.0% (approaching F_max)");
        println!("  Actual:   {:.3}%", fee_percentage);
        
        let passed = fee_percentage > 0.8 && fee_percentage <= 1.0; // Should be high, approaching 1.0%
        println!("  Result:   {}", if passed { "✅ PASS" } else { "❌ FAIL" });
        test_results.push(("Unverified high usage", passed));
        if !passed { all_passed = false; }
    }
    
    // Test Cases for Verified Agents
    println!("\n📊 Testing Verified Agents:");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    // Test 4: New verified agent should get F_min_verified = 0.25%
    {
        let mut agent = AgentStatus::new_verified(0);
        agent.update_usage(0);
        let fee_rate = model.calculate_fee_rate(&agent, 0);
        let fee_percentage = fee_rate * 100.0;
        
        println!("Test 4: New verified agent");
        println!("  Expected: ~0.25%");
        println!("  Actual:   {:.3}%", fee_percentage);
        
        let passed = (fee_percentage - 0.25).abs() < 0.01; // Within 0.01%
        println!("  Result:   {}", if passed { "✅ PASS" } else { "❌ FAIL" });
        test_results.push(("Verified new agent", passed));
        if !passed { all_passed = false; }
    }
    
    // Test 5: Agent with u=20000, t=6 months should get ~0.28%
    {
        let mut agent = AgentStatus::new_verified(0);
        agent.update_usage(20000);
        let six_months = 6 * 30 * 24 * 60 * 60; // 6 months in seconds
        let fee_rate = model.calculate_fee_rate(&agent, six_months);
        let fee_percentage = fee_rate * 100.0;
        
        println!("\nTest 5: Verified agent with u=20000, t=6 months");
        println!("  Expected: ~0.28%");
        println!("  Actual:   {:.3}%", fee_percentage);
        
        let passed = (fee_percentage - 0.28).abs() < 0.1; // Within 0.1%
        println!("  Result:   {}", if passed { "✅ PASS" } else { "❌ FAIL" });
        test_results.push(("Verified high usage", passed));
        if !passed { all_passed = false; }
    }
    
    // Test 6: Verified agent with high usage should get lower fees than unverified
    {
        let mut unverified = AgentStatus::new_unverified(0);
        unverified.update_usage(20000);
        let mut verified = AgentStatus::new_verified(0);
        verified.update_usage(20000);
        
        let six_months = 6 * 30 * 24 * 60 * 60;
        let unverified_rate = model.calculate_fee_rate(&unverified, six_months);
        let verified_rate = model.calculate_fee_rate(&verified, six_months);
        
        println!("\nTest 6: Verified vs Unverified (same usage)");
        println!("  Unverified: {:.3}%", unverified_rate * 100.0);
        println!("  Verified:   {:.3}%", verified_rate * 100.0);
        
        let passed = verified_rate < unverified_rate;
        println!("  Result:     {}", if passed { "✅ PASS (Verified < Unverified)" } else { "❌ FAIL" });
        test_results.push(("Verified advantage", passed));
        if !passed { all_passed = false; }
    }
    
    // Edge Cases
    println!("\n📊 Testing Edge Cases:");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━");
    
    // Test 7: Zero amounts
    {
        let agent = AgentStatus::new_unverified(0);
        let fee_rate = model.calculate_fee_rate(&agent, 0);
        let passed = fee_rate >= 0.0 && fee_rate <= 1.0;
        
        println!("Test 7: Zero amounts");
        println!("  Fee Rate: {:.6}", fee_rate);
        println!("  Result:   {}", if passed { "✅ PASS (Valid range)" } else { "❌ FAIL" });
        test_results.push(("Zero amounts", passed));
        if !passed { all_passed = false; }
    }
    
    // Test 8: Time boundaries
    {
        let agent = AgentStatus::new_unverified(1000);
        let fee_rate = model.calculate_fee_rate(&agent, 500); // Current time before first transaction
        let passed = fee_rate >= 0.0 && fee_rate <= 1.0;
        
        println!("\nTest 8: Time before first transaction");
        println!("  Fee Rate: {:.6}", fee_rate);
        println!("  Result:   {}", if passed { "✅ PASS (Valid range)" } else { "❌ FAIL" });
        test_results.push(("Time boundaries", passed));
        if !passed { all_passed = false; }
    }
    
    // Summary
    println!("\n📋 Test Summary:");
    println!("═════════════════");
    
    let passed_count = test_results.iter().filter(|(_, passed)| *passed).count();
    let total_count = test_results.len();
    
    for (test_name, passed) in &test_results {
        println!("{} {}", if *passed { "✅" } else { "❌" }, test_name);
    }
    
    println!("\nResults: {}/{} tests passed", passed_count, total_count);
    
    if all_passed {
        println!("🎉 All fee model tests PASSED!");
        process::exit(0);
    } else {
        println!("💥 Some fee model tests FAILED!");
        process::exit(1);
    }
}