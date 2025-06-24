//! Resource metering and cost management

use crate::{AccountId, Balance, ExchangeError, OperationCosts, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use parking_lot::RwLock;

/// Resource cost for an operation
pub type ResourceCost = u64;

/// Resource meter for tracking and charging resource usage
pub struct ResourceMeter {
    costs: OperationCosts,
    usage: Arc<RwLock<ResourceUsage>>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub total_compute_ms: u64,
    pub total_storage_kb: u64,
    pub total_operations: u64,
}

impl ResourceMeter {
    pub fn new(costs: OperationCosts) -> Self {
        Self {
            costs,
            usage: Arc::new(RwLock::new(ResourceUsage::default())),
        }
    }
    
    /// Calculate cost for an operation
    pub fn calculate_cost(&self, operation: &Operation) -> ResourceCost {
        match operation {
            Operation::CreateAccount => self.costs.create_account,
            Operation::Transfer => self.costs.transfer,
            Operation::StoreData(kb) => self.costs.store_data_per_kb * kb,
            Operation::Compute(ms) => self.costs.compute_per_ms * ms,
        }
    }
    
    /// Execute an operation with resource metering
    pub fn metered_execute<F, T>(
        &self,
        _account: &AccountId,
        operation: Operation,
        f: F,
    ) -> Result<T>
    where
        F: FnOnce() -> Result<T>,
    {
        let _cost = self.calculate_cost(&operation);
        
        // TODO: Check and deduct balance
        // For now, just execute
        
        // Update usage statistics
        let mut usage = self.usage.write();
        match &operation {
            Operation::Compute(ms) => usage.total_compute_ms += ms,
            Operation::StoreData(kb) => usage.total_storage_kb += kb,
            _ => usage.total_operations += 1,
        }
        
        f()
    }
    
    /// Get current resource usage
    pub fn get_usage(&self) -> ResourceUsage {
        self.usage.read().clone()
    }
}

/// Types of operations that consume resources
#[derive(Debug, Clone)]
pub enum Operation {
    CreateAccount,
    Transfer,
    StoreData(u64), // KB
    Compute(u64),   // milliseconds
}

/// Resource status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceStatus {
    pub usage: ResourceUsage,
    pub costs: OperationCosts,
}