//! Resource metering for QuDAG Exchange
//!
//! Implements cost calculation and resource usage tracking for operations

#[cfg(not(feature = "std"))]
use alloc::{collections::BTreeMap, string::String, vec::Vec};

#[cfg(feature = "std")]
use std::collections::BTreeMap;

use crate::{account::AccountId, types::rUv, Error, Result};
use serde::{Deserialize, Serialize};

/// Types of operations that can be metered
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum OperationType {
    /// Basic transfer between accounts
    Transfer,
    /// Mint new tokens
    Mint,
    /// Burn tokens
    Burn,
    /// Create new account
    CreateAccount,
    /// Update account metadata
    UpdateAccount,
    /// Store data (per byte)
    StoreData,
    /// Execute computation (per instruction)
    ExecuteComputation,
    /// Verify signature
    VerifySignature,
    /// Query balance
    QueryBalance,
    /// Query transaction status
    QueryTransaction,
}

/// Cost configuration for different operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationCost {
    /// Base cost for the operation
    pub base_cost: rUv,

    /// Cost per unit (e.g., per byte for storage)
    pub per_unit_cost: rUv,

    /// Minimum fee required
    pub min_fee: rUv,

    /// Maximum allowed units (0 = unlimited)
    pub max_units: u64,
}

impl Default for OperationCost {
    fn default() -> Self {
        Self {
            base_cost: rUv::ZERO,
            per_unit_cost: rUv::ZERO,
            min_fee: rUv::ZERO,
            max_units: 0,
        }
    }
}

/// Resource meter for tracking and calculating costs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMeter {
    /// Cost configuration for each operation type
    cost_table: BTreeMap<OperationType, OperationCost>,

    /// Global multiplier for all costs (for dynamic adjustment)
    cost_multiplier: f64,

    /// Usage statistics
    usage_stats: UsageStatistics,
}

/// Usage statistics for monitoring
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UsageStatistics {
    /// Total operations metered
    pub total_operations: u64,

    /// Total rUv consumed
    pub total_ruv_consumed: rUv,

    /// Operations by type
    pub operations_by_type: BTreeMap<OperationType, u64>,

    /// rUv consumed by type
    pub ruv_by_type: BTreeMap<OperationType, rUv>,
}

impl ResourceMeter {
    /// Create a new resource meter with default costs
    pub fn new() -> Self {
        let mut meter = Self {
            cost_table: BTreeMap::new(),
            cost_multiplier: 1.0,
            usage_stats: UsageStatistics::default(),
        };

        // Initialize default costs
        meter.set_default_costs();
        meter
    }

    /// Set default operation costs
    fn set_default_costs(&mut self) {
        // Transfer: 1 rUv base + 0.1 rUv per 1KB of memo
        self.cost_table.insert(
            OperationType::Transfer,
            OperationCost {
                base_cost: rUv::new(1),
                per_unit_cost: rUv::new(0), // Will use 0.1 rUv per KB in practice
                min_fee: rUv::new(1),
                max_units: 10 * 1024, // 10KB max memo
            },
        );

        // Mint: 10 rUv base (privileged operation)
        self.cost_table.insert(
            OperationType::Mint,
            OperationCost {
                base_cost: rUv::new(10),
                per_unit_cost: rUv::ZERO,
                min_fee: rUv::new(10),
                max_units: 0,
            },
        );

        // Burn: 5 rUv base
        self.cost_table.insert(
            OperationType::Burn,
            OperationCost {
                base_cost: rUv::new(5),
                per_unit_cost: rUv::ZERO,
                min_fee: rUv::new(5),
                max_units: 0,
            },
        );

        // Create account: 100 rUv (to prevent spam)
        self.cost_table.insert(
            OperationType::CreateAccount,
            OperationCost {
                base_cost: rUv::new(100),
                per_unit_cost: rUv::ZERO,
                min_fee: rUv::new(100),
                max_units: 0,
            },
        );

        // Update account: 5 rUv base
        self.cost_table.insert(
            OperationType::UpdateAccount,
            OperationCost {
                base_cost: rUv::new(5),
                per_unit_cost: rUv::ZERO,
                min_fee: rUv::new(5),
                max_units: 0,
            },
        );

        // Store data: 1 rUv per KB
        self.cost_table.insert(
            OperationType::StoreData,
            OperationCost {
                base_cost: rUv::new(1),
                per_unit_cost: rUv::new(1), // 1 rUv per KB
                min_fee: rUv::new(1),
                max_units: 1024 * 1024, // 1MB max
            },
        );

        // Execute computation: 1 rUv per 1000 instructions
        self.cost_table.insert(
            OperationType::ExecuteComputation,
            OperationCost {
                base_cost: rUv::new(1),
                per_unit_cost: rUv::new(1), // 1 rUv per 1000 instructions
                min_fee: rUv::new(1),
                max_units: 1_000_000, // 1M instructions max
            },
        );

        // Verify signature: 2 rUv (quantum signatures are expensive)
        self.cost_table.insert(
            OperationType::VerifySignature,
            OperationCost {
                base_cost: rUv::new(2),
                per_unit_cost: rUv::ZERO,
                min_fee: rUv::new(2),
                max_units: 0,
            },
        );

        // Query operations: 0.1 rUv (cheap reads)
        self.cost_table.insert(
            OperationType::QueryBalance,
            OperationCost {
                base_cost: rUv::new(0), // Actually 0.1, but we use integer rUv
                per_unit_cost: rUv::ZERO,
                min_fee: rUv::new(1), // Minimum 1 rUv
                max_units: 0,
            },
        );

        self.cost_table.insert(
            OperationType::QueryTransaction,
            OperationCost {
                base_cost: rUv::new(0),
                per_unit_cost: rUv::ZERO,
                min_fee: rUv::new(1),
                max_units: 0,
            },
        );
    }

    /// Calculate cost for an operation
    pub fn calculate_cost(&self, operation: OperationType, units: u64) -> Result<rUv> {
        let cost_config = self
            .cost_table
            .get(&operation)
            .ok_or_else(|| Error::Other("Unknown operation type".into()))?;

        // Check unit limits
        if cost_config.max_units > 0 && units > cost_config.max_units {
            return Err(Error::resource_limit_exceeded(
                "operation_units",
                cost_config.max_units,
                units,
            ));
        }

        // Calculate base cost
        let unit_cost = cost_config
            .per_unit_cost
            .checked_mul(units)
            .ok_or_else(|| Error::Other("Cost calculation overflow".into()))?;

        let raw_cost = cost_config
            .base_cost
            .checked_add(unit_cost)
            .ok_or_else(|| Error::Other("Cost calculation overflow".into()))?;

        // Apply multiplier (convert to integer math to avoid float precision issues)
        let multiplied = (raw_cost.amount() as f64 * self.cost_multiplier) as u64;
        let final_cost = rUv::new(multiplied);

        // Ensure minimum fee
        Ok(if final_cost < cost_config.min_fee {
            cost_config.min_fee
        } else {
            final_cost
        })
    }

    /// Execute a metered operation
    pub fn execute_metered<F, T>(
        &mut self,
        account: &AccountId,
        operation: OperationType,
        units: u64,
        ledger_check: impl Fn(&AccountId, rUv) -> Result<()>,
        operation_fn: F,
    ) -> Result<T>
    where
        F: FnOnce() -> Result<T>,
    {
        // Calculate cost
        let cost = self.calculate_cost(operation, units)?;

        // Check if account can afford it
        ledger_check(account, cost)?;

        // Execute the operation
        let result = operation_fn()?;

        // Update statistics
        self.record_usage(operation, cost);

        Ok(result)
    }

    /// Record resource usage for statistics
    fn record_usage(&mut self, operation: OperationType, cost: rUv) {
        self.usage_stats.total_operations += 1;
        self.usage_stats.total_ruv_consumed =
            self.usage_stats.total_ruv_consumed.saturating_add(cost);

        *self
            .usage_stats
            .operations_by_type
            .entry(operation)
            .or_insert(0) += 1;

        let current = self
            .usage_stats
            .ruv_by_type
            .entry(operation)
            .or_insert(rUv::ZERO);
        *current = current.saturating_add(cost);
    }

    /// Get current usage statistics
    pub fn usage_stats(&self) -> &UsageStatistics {
        &self.usage_stats
    }

    /// Update cost for a specific operation
    pub fn update_cost(&mut self, operation: OperationType, cost: OperationCost) {
        self.cost_table.insert(operation, cost);
    }

    /// Set global cost multiplier
    pub fn set_multiplier(&mut self, multiplier: f64) -> Result<()> {
        if multiplier <= 0.0 || multiplier > 100.0 {
            return Err(Error::Other("Invalid cost multiplier".into()));
        }
        self.cost_multiplier = multiplier;
        Ok(())
    }

    /// Get cost configuration for an operation
    pub fn get_cost(&self, operation: OperationType) -> Option<&OperationCost> {
        self.cost_table.get(&operation)
    }

    /// Estimate transaction size in units (for transfer operations)
    pub fn estimate_transfer_units(memo: Option<&str>) -> u64 {
        // Base transaction size (~200 bytes) + memo size
        let base_size = 200;
        let memo_size = memo.map(|m| m.len()).unwrap_or(0);
        let total_bytes = base_size + memo_size;

        // Convert to KB (round up)
        ((total_bytes + 1023) / 1024) as u64
    }
}

impl Default for ResourceMeter {
    fn default() -> Self {
        Self::new()
    }
}

/// Resource limits for accounts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum operations per time period
    pub max_operations_per_hour: u64,

    /// Maximum rUv consumption per time period
    pub max_ruv_per_hour: rUv,

    /// Maximum data storage
    pub max_storage_bytes: u64,

    /// Maximum computation units
    pub max_computation_units: u64,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_operations_per_hour: 1000,
            max_ruv_per_hour: rUv::new(10_000),
            max_storage_bytes: 10 * 1024 * 1024, // 10MB
            max_computation_units: 1_000_000,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cost_calculation() {
        let meter = ResourceMeter::new();

        // Test transfer cost
        let cost = meter.calculate_cost(OperationType::Transfer, 0).unwrap();
        assert_eq!(cost, rUv::new(1)); // Base cost

        // Test storage cost (5KB)
        let cost = meter.calculate_cost(OperationType::StoreData, 5).unwrap();
        assert_eq!(cost, rUv::new(6)); // 1 base + 5 * 1 per KB

        // Test computation cost (5000 instructions = 5 units)
        let cost = meter
            .calculate_cost(OperationType::ExecuteComputation, 5)
            .unwrap();
        assert_eq!(cost, rUv::new(6)); // 1 base + 5 * 1
    }

    #[test]
    fn test_cost_limits() {
        let meter = ResourceMeter::new();

        // Exceed storage limit
        let result = meter.calculate_cost(OperationType::StoreData, 2000);
        assert!(result.is_err());

        // Within limits
        let result = meter.calculate_cost(OperationType::StoreData, 1000);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cost_multiplier() {
        let mut meter = ResourceMeter::new();

        // Normal cost
        let normal_cost = meter.calculate_cost(OperationType::Transfer, 0).unwrap();
        assert_eq!(normal_cost, rUv::new(1));

        // Double costs
        meter.set_multiplier(2.0).unwrap();
        let double_cost = meter.calculate_cost(OperationType::Transfer, 0).unwrap();
        assert_eq!(double_cost, rUv::new(2));

        // Half costs (but respect minimum)
        meter.set_multiplier(0.5).unwrap();
        let half_cost = meter.calculate_cost(OperationType::Transfer, 0).unwrap();
        assert_eq!(half_cost, rUv::new(1)); // Min fee is 1
    }

    #[test]
    fn test_usage_statistics() {
        let mut meter = ResourceMeter::new();

        // Simulate some operations
        meter.record_usage(OperationType::Transfer, rUv::new(1));
        meter.record_usage(OperationType::Transfer, rUv::new(1));
        meter.record_usage(OperationType::StoreData, rUv::new(10));

        let stats = meter.usage_stats();
        assert_eq!(stats.total_operations, 3);
        assert_eq!(stats.total_ruv_consumed, rUv::new(12));
        assert_eq!(stats.operations_by_type[&OperationType::Transfer], 2);
        assert_eq!(stats.ruv_by_type[&OperationType::Transfer], rUv::new(2));
    }

    #[test]
    fn test_metered_execution() {
        let mut meter = ResourceMeter::new();
        let account = AccountId::new("alice");

        // Mock ledger check that always succeeds
        let ledger_check = |_: &AccountId, _: rUv| -> Result<()> { Ok(()) };

        // Execute a metered operation
        let result = meter
            .execute_metered(&account, OperationType::Transfer, 0, ledger_check, || {
                Ok(42)
            })
            .unwrap();

        assert_eq!(result, 42);
        assert_eq!(meter.usage_stats().total_operations, 1);
    }

    #[test]
    fn test_transfer_size_estimation() {
        // No memo
        assert_eq!(ResourceMeter::estimate_transfer_units(None), 1); // 200 bytes < 1KB

        // Small memo
        assert_eq!(ResourceMeter::estimate_transfer_units(Some("Hello")), 1);

        // Large memo (900 bytes)
        let large_memo = "x".repeat(900);
        assert_eq!(ResourceMeter::estimate_transfer_units(Some(&large_memo)), 2);
        // 200 + 900 > 1KB
    }
}
