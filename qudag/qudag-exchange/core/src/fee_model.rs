//! Dynamic Tiered Fee Model for QuDAG Exchange
//!
//! Implements a continuous, parameterized fee model that phases agents from
//! introductory rates (0.1%) to tiered rates based on usage and verification status.
//!
//! - Unverified agents: 0.1% → 1.0% (increasing with time and usage)
//! - Verified agents: 0.25% → 0.50% → 0.25% (rewards high throughput)

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};

use crate::{
    types::{rUv, Timestamp},
    Error, Result,
};
use serde::{Deserialize, Serialize};

/// Fee model parameters based on the mathematical specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeModelParams {
    /// F_min: Minimum fee (introductory) - 0.001 (0.1%)
    pub f_min: f64,
    /// F_max: Maximum fee for unverified - 0.010 (1.0%)
    pub f_max: f64,
    /// F_min_verified: Minimum fee for verified - 0.0025 (0.25%)
    pub f_min_verified: f64,
    /// F_max_verified: Maximum fee for verified - 0.005 (0.50%)
    pub f_max_verified: f64,
    /// T: Phase-in time constant (3 months in seconds)
    pub time_constant_seconds: u64,
    /// U: Usage scale threshold (10,000 rUv per month)
    pub usage_threshold_ruv: u64,
}

impl Default for FeeModelParams {
    fn default() -> Self {
        Self {
            f_min: 0.001,                                 // 0.1%
            f_max: 0.010,                                 // 1.0%
            f_min_verified: 0.0025,                       // 0.25%
            f_max_verified: 0.005,                        // 0.50%
            time_constant_seconds: 3 * 30 * 24 * 60 * 60, // 3 months
            usage_threshold_ruv: 10_000,                  // 10,000 rUv
        }
    }
}

impl FeeModelParams {
    /// Validate that parameters are within reasonable bounds
    pub fn validate(&self) -> Result<()> {
        if self.f_min <= 0.0 || self.f_min >= 1.0 {
            return Err(Error::Other("f_min must be between 0 and 1".into()));
        }
        if self.f_max <= self.f_min || self.f_max >= 1.0 {
            return Err(Error::Other(
                "f_max must be greater than f_min and less than 1".into(),
            ));
        }
        if self.f_min_verified <= 0.0 || self.f_min_verified >= 1.0 {
            return Err(Error::Other(
                "f_min_verified must be between 0 and 1".into(),
            ));
        }
        if self.f_max_verified <= self.f_min_verified || self.f_max_verified >= 1.0 {
            return Err(Error::Other(
                "f_max_verified must be greater than f_min_verified and less than 1".into(),
            ));
        }
        if self.time_constant_seconds == 0 {
            return Err(Error::Other(
                "time_constant_seconds must be greater than 0".into(),
            ));
        }
        if self.usage_threshold_ruv == 0 {
            return Err(Error::Other(
                "usage_threshold_ruv must be greater than 0".into(),
            ));
        }
        Ok(())
    }
}

/// Agent verification status and usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStatus {
    /// Whether the agent is verified (KYC/presence proof)
    pub verified: bool,
    /// Timestamp of first transaction (for calculating time phase-in)
    pub first_transaction_timestamp: Timestamp,
    /// Monthly usage rate in rUv
    pub monthly_usage_ruv: u64,
    /// Optional verification proof data
    pub verification_proof: Option<Vec<u8>>,
}

impl AgentStatus {
    /// Create a new unverified agent status
    pub fn new_unverified(first_transaction: Timestamp) -> Self {
        Self {
            verified: false,
            first_transaction_timestamp: first_transaction,
            monthly_usage_ruv: 0,
            verification_proof: None,
        }
    }

    /// Create a new verified agent status
    pub fn new_verified(first_transaction: Timestamp, verification_proof: Vec<u8>) -> Self {
        Self {
            verified: true,
            first_transaction_timestamp: first_transaction,
            monthly_usage_ruv: 0,
            verification_proof: Some(verification_proof),
        }
    }

    /// Update monthly usage statistics
    pub fn update_usage(&mut self, monthly_usage: u64) {
        self.monthly_usage_ruv = monthly_usage;
    }

    /// Verify the agent (add verification proof)
    pub fn verify(&mut self, proof: Vec<u8>) {
        self.verified = true;
        self.verification_proof = Some(proof);
    }

    /// Revoke verification status
    pub fn revoke_verification(&mut self) {
        self.verified = false;
        self.verification_proof = None;
    }
}

/// Dynamic fee model calculator
#[derive(Debug, Clone)]
pub struct FeeModel {
    /// Model parameters
    params: FeeModelParams,
}

impl FeeModel {
    /// Create a new fee model with default parameters
    pub fn new() -> Result<Self> {
        Self::with_params(FeeModelParams::default())
    }

    /// Create a fee model with custom parameters
    pub fn with_params(params: FeeModelParams) -> Result<Self> {
        params.validate()?;
        Ok(Self { params })
    }

    /// Calculate fee rate for a transaction
    ///
    /// # Arguments
    /// * `agent_status` - Agent's verification status and usage statistics
    /// * `current_time` - Current timestamp for time phase-in calculation
    ///
    /// # Returns
    /// Fee rate as a fraction (e.g., 0.001 = 0.1%)
    pub fn calculate_fee_rate(
        &self,
        agent_status: &AgentStatus,
        current_time: Timestamp,
    ) -> Result<f64> {
        // Calculate time since first transaction in seconds
        let time_since_first =
            if current_time.value() >= agent_status.first_transaction_timestamp.value() {
                current_time.value() - agent_status.first_transaction_timestamp.value()
            } else {
                0 // Handle edge case where current_time is before first transaction
            };

        // Calculate time phase-in: α(t) = 1 - e^(-t/T)
        let alpha = self.time_phase_in(time_since_first as f64);

        // Calculate usage scaling: β(u) = 1 - e^(-u/U)
        let beta = self.usage_scaling(agent_status.monthly_usage_ruv as f64);

        let fee_rate = if agent_status.verified {
            // Verified fee: f_ver(u,t) = F_min_ver + (F_max_ver - F_min_ver) * α(t) * (1 - β(u))
            // Fee decreases with usage (rewards high throughput)
            self.params.f_min_verified
                + (self.params.f_max_verified - self.params.f_min_verified) * alpha * (1.0 - beta)
        } else {
            // Unverified fee: f_unv(u,t) = F_min + (F_max - F_min) * α(t) * β(u)
            // Fee increases with usage and time
            self.params.f_min + (self.params.f_max - self.params.f_min) * alpha * beta
        };

        Ok(fee_rate.max(0.0).min(1.0)) // Clamp to [0, 1]
    }

    /// Calculate actual fee amount for a transaction
    pub fn calculate_fee_amount(
        &self,
        transaction_amount: rUv,
        agent_status: &AgentStatus,
        current_time: Timestamp,
    ) -> Result<rUv> {
        let fee_rate = self.calculate_fee_rate(agent_status, current_time)?;
        let fee_amount = (transaction_amount.amount() as f64 * fee_rate) as u64;
        Ok(rUv::new(fee_amount))
    }

    /// Time phase-in function: α(t) = 1 - e^(-t/T)
    fn time_phase_in(&self, time_seconds: f64) -> f64 {
        let t_normalized = time_seconds / (self.params.time_constant_seconds as f64);
        1.0 - (-t_normalized).exp()
    }

    /// Usage scaling function: β(u) = 1 - e^(-u/U)
    fn usage_scaling(&self, usage_ruv: f64) -> f64 {
        let u_normalized = usage_ruv / (self.params.usage_threshold_ruv as f64);
        1.0 - (-u_normalized).exp()
    }

    /// Get model parameters
    pub fn params(&self) -> &FeeModelParams {
        &self.params
    }

    /// Update model parameters (only allowed if system is not immutable)
    pub fn update_params(&mut self, params: FeeModelParams) -> Result<()> {
        params.validate()?;
        self.params = params;
        Ok(())
    }
}

impl Default for FeeModel {
    fn default() -> Self {
        Self::new().expect("Default parameters should be valid")
    }
}

/// Fee calculation utilities
pub struct FeeCalculator;

impl FeeCalculator {
    /// Calculate fee examples for documentation and testing
    pub fn calculate_examples() -> Vec<(String, f64)> {
        let model = FeeModel::new().expect("Default model should work");
        let mut examples = Vec::new();

        // Example 1: Unverified, new user, no usage
        let status1 = AgentStatus::new_unverified(Timestamp::new(0));
        let rate1 = model
            .calculate_fee_rate(&status1, Timestamp::new(0))
            .unwrap();
        examples.push(("Unverified, t=0, u=0".to_string(), rate1));

        // Example 2: Unverified, 3 months, 5000 rUv/month
        let mut status2 = AgentStatus::new_unverified(Timestamp::new(0));
        status2.update_usage(5000);
        let three_months = 3 * 30 * 24 * 60 * 60; // 3 months in seconds
        let rate2 = model
            .calculate_fee_rate(&status2, Timestamp::new(three_months))
            .unwrap();
        examples.push(("Unverified, t=3mo, u=5000".to_string(), rate2));

        // Example 3: Verified, 6 months, 20000 rUv/month
        let mut status3 = AgentStatus::new_verified(Timestamp::new(0), vec![1, 2, 3]);
        status3.update_usage(20000);
        let six_months = 6 * 30 * 24 * 60 * 60; // 6 months in seconds
        let rate3 = model
            .calculate_fee_rate(&status3, Timestamp::new(six_months))
            .unwrap();
        examples.push(("Verified, t=6mo, u=20000".to_string(), rate3));

        examples
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fee_model_params_validation() {
        let mut params = FeeModelParams::default();
        assert!(params.validate().is_ok());

        // Test invalid f_min
        params.f_min = -0.1;
        assert!(params.validate().is_err());

        params.f_min = 1.1;
        assert!(params.validate().is_err());

        // Reset and test f_max
        params = FeeModelParams::default();
        params.f_max = 0.0005; // Less than f_min
        assert!(params.validate().is_err());
    }

    #[test]
    fn test_agent_status_creation() {
        let timestamp = Timestamp::new(1000);

        // Unverified agent
        let unverified = AgentStatus::new_unverified(timestamp);
        assert!(!unverified.verified);
        assert_eq!(unverified.first_transaction_timestamp, timestamp);
        assert_eq!(unverified.monthly_usage_ruv, 0);
        assert!(unverified.verification_proof.is_none());

        // Verified agent
        let proof = vec![1, 2, 3, 4];
        let verified = AgentStatus::new_verified(timestamp, proof.clone());
        assert!(verified.verified);
        assert_eq!(verified.verification_proof.as_ref().unwrap(), &proof);
    }

    #[test]
    fn test_agent_verification() {
        let mut agent = AgentStatus::new_unverified(Timestamp::new(0));
        assert!(!agent.verified);

        // Verify agent
        let proof = vec![5, 6, 7, 8];
        agent.verify(proof.clone());
        assert!(agent.verified);
        assert_eq!(agent.verification_proof.as_ref().unwrap(), &proof);

        // Revoke verification
        agent.revoke_verification();
        assert!(!agent.verified);
        assert!(agent.verification_proof.is_none());
    }

    #[test]
    fn test_fee_calculation_edge_cases() {
        let model = FeeModel::new().unwrap();

        // Test with new unverified agent (should be minimum fee)
        let agent = AgentStatus::new_unverified(Timestamp::new(1000));
        let rate = model
            .calculate_fee_rate(&agent, Timestamp::new(1000))
            .unwrap();
        assert!((rate - 0.001).abs() < 1e-10); // Should be f_min

        // Test with time before first transaction (edge case)
        let rate = model
            .calculate_fee_rate(&agent, Timestamp::new(500))
            .unwrap();
        assert!((rate - 0.001).abs() < 1e-10); // Should still be f_min
    }

    #[test]
    fn test_fee_amount_calculation() {
        let model = FeeModel::new().unwrap();
        let agent = AgentStatus::new_unverified(Timestamp::new(0));

        // Test fee amount for 1000 rUv transaction
        let transaction_amount = rUv::new(1000);
        let fee_amount = model
            .calculate_fee_amount(transaction_amount, &agent, Timestamp::new(0))
            .unwrap();

        // Should be 1000 * 0.001 = 1 rUv (minimum fee rate)
        assert_eq!(fee_amount.amount(), 1);
    }

    #[test]
    fn test_smoothing_functions() {
        let model = FeeModel::new().unwrap();

        // Test time phase-in function
        let alpha_0 = model.time_phase_in(0.0);
        assert!((alpha_0 - 0.0).abs() < 1e-10);

        let alpha_inf = model.time_phase_in(f64::INFINITY);
        assert!((alpha_inf - 1.0).abs() < 1e-10);

        // Test usage scaling function
        let beta_0 = model.usage_scaling(0.0);
        assert!((beta_0 - 0.0).abs() < 1e-10);

        let beta_inf = model.usage_scaling(f64::INFINITY);
        assert!((beta_inf - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_verified_vs_unverified_fees() {
        let model = FeeModel::new().unwrap();
        let timestamp = Timestamp::new(0);
        let later_time = Timestamp::new(3 * 30 * 24 * 60 * 60); // 3 months later

        // Create unverified and verified agents with same usage
        let mut unverified = AgentStatus::new_unverified(timestamp);
        unverified.update_usage(5000);

        let mut verified = AgentStatus::new_verified(timestamp, vec![1, 2, 3]);
        verified.update_usage(5000);

        let unverified_rate = model.calculate_fee_rate(&unverified, later_time).unwrap();
        let verified_rate = model.calculate_fee_rate(&verified, later_time).unwrap();

        // At moderate usage, verified agents should generally pay lower fees
        // (though this depends on exact parameters and usage levels)
        println!(
            "Unverified rate: {}, Verified rate: {}",
            unverified_rate, verified_rate
        );

        // Test high usage scenario where verified agents benefit more
        unverified.update_usage(50000); // Very high usage
        verified.update_usage(50000);

        let unverified_high = model.calculate_fee_rate(&unverified, later_time).unwrap();
        let verified_high = model.calculate_fee_rate(&verified, later_time).unwrap();

        // At high usage, verified should have lower fees (reward for high throughput)
        assert!(verified_high < unverified_high);
    }

    #[test]
    fn test_fee_examples() {
        let examples = FeeCalculator::calculate_examples();
        assert_eq!(examples.len(), 3);

        // First example should be minimum fee
        assert!((examples[0].1 - 0.001).abs() < 1e-10);

        // Print examples for verification
        for (desc, rate) in examples {
            println!("{}: {:.4}% ({:.6} rate)", desc, rate * 100.0, rate);
        }
    }
}
