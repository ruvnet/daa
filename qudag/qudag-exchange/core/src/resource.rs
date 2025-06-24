//! Resource metering and contribution tracking

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::Result;
use crate::ruv::RuvAmount;

/// Types of resources that can be contributed to the network
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceType {
    /// CPU cycles (measured in GFLOPS)
    Cpu,
    /// GPU compute (measured in TFLOPS)
    Gpu,
    /// Memory (measured in GB-hours)
    Memory,
    /// Storage (measured in TB-hours)
    Storage,
    /// Network bandwidth (measured in GB transferred)
    Bandwidth,
    /// Custom resource type
    Custom(String),
}

/// Resource metrics for a specific contribution
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResourceMetrics {
    /// Type of resource
    pub resource_type: ResourceType,
    
    /// Amount of resource contributed
    pub amount: f64,
    
    /// Duration of contribution in seconds
    pub duration: u64,
    
    /// Quality score (0.0 to 1.0)
    pub quality_score: f64,
    
    /// Timestamp of measurement
    pub timestamp: u64,
}

impl ResourceMetrics {
    /// Calculate the rUv value for this resource contribution
    pub fn calculate_ruv_value(&self) -> RuvAmount {
        // Base rate per resource type (rUv per unit)
        let base_rate = match &self.resource_type {
            ResourceType::Cpu => 0.1,      // 0.1 rUv per GFLOP
            ResourceType::Gpu => 1.0,      // 1.0 rUv per TFLOP
            ResourceType::Memory => 0.01,   // 0.01 rUv per GB-hour
            ResourceType::Storage => 0.001, // 0.001 rUv per TB-hour
            ResourceType::Bandwidth => 0.05, // 0.05 rUv per GB
            ResourceType::Custom(_) => 0.01, // Default rate for custom
        };

        // Calculate value: amount * duration * quality * base_rate
        let duration_hours = self.duration as f64 / 3600.0;
        let raw_value = self.amount * duration_hours * self.quality_score * base_rate;
        
        // Convert to rUv (capped at reasonable maximum per contribution)
        let ruv_value = raw_value.min(1000.0) as u64;
        RuvAmount::from_ruv(ruv_value.max(1))
    }
}

/// Tracks resource contributions from agents
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ResourceContribution {
    /// Agent identifier
    pub agent_id: String,
    
    /// List of resource metrics
    pub metrics: Vec<ResourceMetrics>,
    
    /// Total rUv earned from this contribution
    pub total_ruv: RuvAmount,
    
    /// Verification status
    pub verified: bool,
    
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl ResourceContribution {
    /// Create a new resource contribution
    pub fn new(agent_id: String) -> Self {
        Self {
            agent_id,
            metrics: Vec::new(),
            total_ruv: RuvAmount::default(),
            verified: false,
            metadata: HashMap::new(),
        }
    }

    /// Add a resource metric to the contribution
    pub fn add_metric(&mut self, metric: ResourceMetrics) -> Result<()> {
        let ruv_value = metric.calculate_ruv_value();
        self.total_ruv = self.total_ruv.checked_add(&ruv_value)?;
        self.metrics.push(metric);
        Ok(())
    }

    /// Mark the contribution as verified
    pub fn verify(&mut self) {
        self.verified = true;
    }

    /// Get total contribution value in rUv
    pub fn total_value(&self) -> &RuvAmount {
        &self.total_ruv
    }
}

/// Resource metering service
#[derive(Clone, Debug)]
pub struct ResourceMeter {
    /// Active contributions being tracked
    contributions: HashMap<String, ResourceContribution>,
}

impl ResourceMeter {
    /// Create a new resource meter
    pub fn new() -> Self {
        Self {
            contributions: HashMap::new(),
        }
    }

    /// Start tracking a contribution
    pub fn start_contribution(&mut self, agent_id: String) -> &mut ResourceContribution {
        self.contributions
            .entry(agent_id.clone())
            .or_insert_with(|| ResourceContribution::new(agent_id))
    }

    /// Record a resource metric
    pub fn record_metric(&mut self, agent_id: &str, metric: ResourceMetrics) -> Result<()> {
        if let Some(contribution) = self.contributions.get_mut(agent_id) {
            contribution.add_metric(metric)?;
        }
        Ok(())
    }

    /// Finalize and verify a contribution
    pub fn finalize_contribution(&mut self, agent_id: &str) -> Option<ResourceContribution> {
        self.contributions.remove(agent_id).map(|mut contrib| {
            contrib.verify();
            contrib
        })
    }

    /// Get active contributions
    pub fn active_contributions(&self) -> &HashMap<String, ResourceContribution> {
        &self.contributions
    }
}

impl Default for ResourceMeter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_metrics_ruv_calculation() {
        let metric = ResourceMetrics {
            resource_type: ResourceType::Cpu,
            amount: 100.0, // 100 GFLOPS
            duration: 3600, // 1 hour
            quality_score: 0.9,
            timestamp: 0,
        };

        let ruv = metric.calculate_ruv_value();
        assert_eq!(ruv.as_ruv(), 9); // 100 * 1 * 0.9 * 0.1 = 9 rUv
    }

    #[test]
    fn test_resource_contribution() {
        let mut contribution = ResourceContribution::new("agent1".to_string());
        
        let metric = ResourceMetrics {
            resource_type: ResourceType::Gpu,
            amount: 10.0, // 10 TFLOPS
            duration: 7200, // 2 hours
            quality_score: 1.0,
            timestamp: 0,
        };

        contribution.add_metric(metric).unwrap();
        assert_eq!(contribution.total_value().as_ruv(), 20); // 10 * 2 * 1.0 * 1.0 = 20 rUv
    }
}