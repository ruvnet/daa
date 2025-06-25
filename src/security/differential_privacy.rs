//! Differential privacy mechanisms for gradient protection

use super::SecurityError;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;
use rand_distr::{Distribution, Normal};
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

/// Differential privacy mechanism for gradients
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DifferentialPrivacy {
    /// Privacy parameter epsilon
    pub epsilon: f64,
    
    /// Privacy parameter delta
    pub delta: f64,
    
    /// Total privacy budget
    pub total_budget: f64,
    
    /// Used privacy budget
    pub used_budget: f64,
    
    /// Noise scale factor
    pub noise_scale: f64,
    
    /// Clipping threshold for gradients
    pub clipping_threshold: f64,
}

impl DifferentialPrivacy {
    /// Create a new differential privacy mechanism
    pub fn new(epsilon: f64, delta: f64, total_budget: f64) -> Result<Self, SecurityError> {
        if epsilon <= 0.0 || delta <= 0.0 || delta >= 1.0 {
            return Err(SecurityError::VerificationError(
                "Invalid privacy parameters".to_string(),
            ));
        }
        
        // Calculate noise scale based on epsilon and delta
        let sensitivity = 1.0; // L2 sensitivity after clipping
        let noise_scale = (2.0 * sensitivity.powi(2) * (1.25 / delta).ln()) / epsilon.powi(2);
        
        Ok(Self {
            epsilon,
            delta,
            total_budget,
            used_budget: 0.0,
            noise_scale: noise_scale.sqrt(),
            clipping_threshold: 1.0,
        })
    }
    
    /// Apply differential privacy to gradients
    pub fn privatize_gradients(
        &mut self,
        gradients: &[f64],
        num_samples: usize,
    ) -> Result<Vec<f64>, SecurityError> {
        // Check privacy budget
        let privacy_cost = self.calculate_privacy_cost(num_samples);
        if self.used_budget + privacy_cost > self.total_budget {
            return Err(SecurityError::PrivacyBudgetExceeded);
        }
        
        // Clip gradients
        let clipped_gradients = self.clip_gradients(gradients);
        
        // Add Gaussian noise
        let noisy_gradients = self.add_gaussian_noise(&clipped_gradients)?;
        
        // Update used budget
        self.used_budget += privacy_cost;
        
        Ok(noisy_gradients)
    }
    
    /// Clip gradients to bound sensitivity
    fn clip_gradients(&self, gradients: &[f64]) -> Vec<f64> {
        let norm: f64 = gradients.iter().map(|x| x * x).sum::<f64>().sqrt();
        
        if norm <= self.clipping_threshold {
            gradients.to_vec()
        } else {
            let scale = self.clipping_threshold / norm;
            gradients.iter().map(|x| x * scale).collect()
        }
    }
    
    /// Add calibrated Gaussian noise
    fn add_gaussian_noise(&self, gradients: &[f64]) -> Result<Vec<f64>, SecurityError> {
        let mut rng = ChaCha20Rng::from_entropy();
        let normal = Normal::new(0.0, self.noise_scale)
            .map_err(|e| SecurityError::VerificationError(e.to_string()))?;
        
        let noisy_gradients: Vec<f64> = gradients
            .iter()
            .map(|&grad| grad + normal.sample(&mut rng))
            .collect();
        
        Ok(noisy_gradients)
    }
    
    /// Calculate privacy cost for this operation
    fn calculate_privacy_cost(&self, num_samples: usize) -> f64 {
        // Using advanced composition theorem
        let sampling_rate = 1.0 / num_samples as f64;
        sampling_rate * self.epsilon
    }
    
    /// Get remaining privacy budget
    pub fn remaining_budget(&self) -> f64 {
        self.total_budget - self.used_budget
    }
}

/// Moments accountant for tighter privacy analysis
pub struct MomentsAccountant {
    /// Maximum moment order to track
    max_order: u32,
    
    /// Accumulated privacy loss moments
    moments: Vec<f64>,
    
    /// Number of compositions
    num_compositions: u32,
}

impl MomentsAccountant {
    /// Create a new moments accountant
    pub fn new(max_order: u32) -> Self {
        Self {
            max_order,
            moments: vec![0.0; max_order as usize],
            num_compositions: 0,
        }
    }
    
    /// Add privacy loss from a mechanism
    pub fn add_privacy_loss(
        &mut self,
        noise_multiplier: f64,
        sampling_rate: f64,
    ) {
        for order in 1..=self.max_order {
            let moment = self.compute_log_moment(order, noise_multiplier, sampling_rate);
            self.moments[(order - 1) as usize] += moment;
        }
        self.num_compositions += 1;
    }
    
    /// Compute log moment of privacy loss
    fn compute_log_moment(
        &self,
        order: u32,
        noise_multiplier: f64,
        sampling_rate: f64,
    ) -> f64 {
        if order == 1 {
            return 0.0;
        }
        
        // Simplified bound for Gaussian mechanism
        let order_f = order as f64;
        sampling_rate * order_f * (order_f - 1.0) / (2.0 * noise_multiplier.powi(2))
    }
    
    /// Convert to (epsilon, delta) privacy guarantee
    pub fn get_privacy_guarantee(&self, target_delta: f64) -> (f64, f64) {
        let mut min_epsilon = f64::INFINITY;
        
        for order in 1..=self.max_order {
            let log_moment = self.moments[(order - 1) as usize];
            let order_f = order as f64;
            
            // Compute epsilon for this order
            let epsilon = (log_moment - target_delta.ln()) / order_f;
            
            if epsilon < min_epsilon {
                min_epsilon = epsilon;
            }
        }
        
        (min_epsilon, target_delta)
    }
}

/// Local differential privacy for individual updates
pub struct LocalDifferentialPrivacy {
    /// Local privacy parameter
    pub epsilon_local: f64,
    
    /// Randomized response probability
    pub flip_probability: f64,
}

impl LocalDifferentialPrivacy {
    /// Create a new local DP mechanism
    pub fn new(epsilon_local: f64) -> Self {
        let flip_probability = 1.0 / (1.0 + epsilon_local.exp());
        
        Self {
            epsilon_local,
            flip_probability,
        }
    }
    
    /// Apply local DP to binary data
    pub fn randomize_binary(&self, value: bool) -> bool {
        let mut rng = ChaCha20Rng::from_entropy();
        
        if rng.gen::<f64>() < self.flip_probability {
            !value
        } else {
            value
        }
    }
    
    /// Apply local DP to continuous data using Laplace mechanism
    pub fn randomize_continuous(&self, value: f64, sensitivity: f64) -> f64 {
        let mut rng = ChaCha20Rng::from_entropy();
        let scale = sensitivity / self.epsilon_local;
        
        // Generate Laplace noise
        let uniform: f64 = rng.gen_range(-0.5..0.5);
        let laplace_noise = -scale * uniform.signum() * (1.0 - 2.0 * uniform.abs()).ln();
        
        value + laplace_noise
    }
}

/// Privacy amplification through shuffling
pub struct ShuffleAmplification {
    /// Number of clients
    pub num_clients: usize,
    
    /// Local privacy parameter
    pub epsilon_local: f64,
}

impl ShuffleAmplification {
    /// Calculate amplified privacy guarantee
    pub fn amplified_privacy(&self) -> (f64, f64) {
        // Simplified bound for shuffle model
        let epsilon_central = self.epsilon_local / (self.num_clients as f64).sqrt();
        let delta = 1.0 / (self.num_clients as f64).powi(2);
        
        (epsilon_central, delta)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_gradient_clipping() {
        let dp = DifferentialPrivacy::new(1.0, 1e-5, 10.0).unwrap();
        
        let gradients = vec![0.5, 0.5, 0.5, 0.5];
        let clipped = dp.clip_gradients(&gradients);
        
        let norm: f64 = clipped.iter().map(|x| x * x).sum::<f64>().sqrt();
        assert!(norm <= dp.clipping_threshold + 1e-6);
    }
    
    #[test]
    fn test_differential_privacy() {
        let mut dp = DifferentialPrivacy::new(1.0, 1e-5, 10.0).unwrap();
        
        let gradients = vec![0.1, 0.2, 0.3];
        let private_gradients = dp.privatize_gradients(&gradients, 100).unwrap();
        
        assert_eq!(private_gradients.len(), gradients.len());
        assert!(dp.used_budget > 0.0);
    }
    
    #[test]
    fn test_local_dp() {
        let ldp = LocalDifferentialPrivacy::new(1.0);
        
        // Test binary randomization
        let mut true_count = 0;
        for _ in 0..1000 {
            if ldp.randomize_binary(true) {
                true_count += 1;
            }
        }
        
        // Should be biased towards true but with some false
        assert!(true_count > 500);
        assert!(true_count < 1000);
    }
    
    #[test]
    fn test_moments_accountant() {
        let mut accountant = MomentsAccountant::new(32);
        
        // Add some privacy losses
        accountant.add_privacy_loss(1.0, 0.01);
        accountant.add_privacy_loss(1.0, 0.01);
        
        let (epsilon, delta) = accountant.get_privacy_guarantee(1e-5);
        assert!(epsilon > 0.0);
        assert_eq!(delta, 1e-5);
    }
}