//! Resource management and allocation

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, warn};

use crate::error::{EconomyError, Result};

/// Resource type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ResourceType {
    Computational,
    Storage,
    Network,
    Token(String),
    Custom(String),
}

impl std::fmt::Display for ResourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResourceType::Computational => write!(f, "Computational"),
            ResourceType::Storage => write!(f, "Storage"),
            ResourceType::Network => write!(f, "Network"),
            ResourceType::Token(symbol) => write!(f, "Token({})", symbol),
            ResourceType::Custom(name) => write!(f, "Custom({})", name),
        }
    }
}

/// Resource with quantity and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    pub resource_type: ResourceType,
    pub quantity: Decimal,
    pub unit: String,
    pub cost_per_unit: Decimal,
    pub reserved: Decimal,
    pub last_updated: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Resource {
    pub fn new(resource_type: ResourceType, quantity: Decimal, unit: String, cost_per_unit: Decimal) -> Self {
        Self {
            resource_type,
            quantity,
            unit,
            cost_per_unit,
            reserved: Decimal::ZERO,
            last_updated: Utc::now(),
            metadata: HashMap::new(),
        }
    }

    pub fn available_quantity(&self) -> Decimal {
        self.quantity - self.reserved
    }

    pub fn total_value(&self) -> Decimal {
        self.quantity * self.cost_per_unit
    }

    pub fn reserve(&mut self, amount: Decimal) -> Result<()> {
        if self.available_quantity() < amount {
            return Err(EconomyError::ResourceNotAvailable(format!(
                "Insufficient {} available: requested {}, available {}",
                self.resource_type, amount, self.available_quantity()
            )));
        }

        self.reserved += amount;
        self.last_updated = Utc::now();
        debug!("Reserved {} {} of {}", amount, self.unit, self.resource_type);
        Ok(())
    }

    pub fn release(&mut self, amount: Decimal) -> Result<()> {
        if self.reserved < amount {
            return Err(EconomyError::ResourceAllocationError(format!(
                "Cannot release {} {}, only {} reserved",
                amount, self.unit, self.reserved
            )));
        }

        self.reserved -= amount;
        self.last_updated = Utc::now();
        debug!("Released {} {} of {}", amount, self.unit, self.resource_type);
        Ok(())
    }

    pub fn consume(&mut self, amount: Decimal) -> Result<()> {
        if self.available_quantity() < amount {
            return Err(EconomyError::ResourceNotAvailable(format!(
                "Insufficient {} to consume: requested {}, available {}",
                self.resource_type, amount, self.available_quantity()
            )));
        }

        self.quantity -= amount;
        self.last_updated = Utc::now();
        debug!("Consumed {} {} of {}", amount, self.unit, self.resource_type);
        Ok(())
    }

    pub fn add_quantity(&mut self, amount: Decimal) {
        self.quantity += amount;
        self.last_updated = Utc::now();
        debug!("Added {} {} of {}", amount, self.unit, self.resource_type);
    }

    pub fn set_metadata(&mut self, key: String, value: serde_json::Value) {
        self.metadata.insert(key, value);
        self.last_updated = Utc::now();
    }

    pub fn get_metadata(&self, key: &str) -> Option<&serde_json::Value> {
        self.metadata.get(key)
    }
}

/// Resource allocation record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAllocation {
    pub id: String,
    pub resource_type: ResourceType,
    pub allocated_amount: Decimal,
    pub requesting_entity: String,
    pub purpose: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub cost: Decimal,
    pub status: AllocationStatus,
}

/// Status of resource allocation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AllocationStatus {
    Active,
    Completed,
    Cancelled,
    Expired,
}

impl ResourceAllocation {
    pub fn new(
        id: String,
        resource_type: ResourceType,
        allocated_amount: Decimal,
        requesting_entity: String,
        purpose: String,
        cost: Decimal,
    ) -> Self {
        Self {
            id,
            resource_type,
            allocated_amount,
            requesting_entity: requesting_entity.clone(),
            purpose,
            start_time: Utc::now(),
            end_time: None,
            cost,
            status: AllocationStatus::Active,
        }
    }

    pub fn complete(&mut self) {
        self.status = AllocationStatus::Completed;
        self.end_time = Some(Utc::now());
    }

    pub fn cancel(&mut self) {
        self.status = AllocationStatus::Cancelled;
        self.end_time = Some(Utc::now());
    }

    pub fn is_active(&self) -> bool {
        self.status == AllocationStatus::Active
    }

    pub fn duration(&self) -> Option<chrono::Duration> {
        self.end_time.map(|end| end - self.start_time)
    }
}

/// Resource manager for handling multiple resources
pub struct ResourceManager {
    resources: HashMap<ResourceType, Resource>,
    allocations: HashMap<String, ResourceAllocation>,
    next_allocation_id: u64,
}

impl ResourceManager {
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
            allocations: HashMap::new(),
            next_allocation_id: 1,
        }
    }

    /// Add a new resource
    pub fn add_resource(&mut self, resource: Resource) {
        info!("Adding resource: {} with quantity {}", 
              resource.resource_type, resource.quantity);
        self.resources.insert(resource.resource_type.clone(), resource);
    }

    /// Get resource by type
    pub fn get_resource(&self, resource_type: &ResourceType) -> Option<&Resource> {
        self.resources.get(resource_type)
    }

    /// Get mutable resource by type
    pub fn get_resource_mut(&mut self, resource_type: &ResourceType) -> Option<&mut Resource> {
        self.resources.get_mut(resource_type)
    }

    /// List all available resources
    pub fn available_resources(&self) -> Vec<&Resource> {
        self.resources.values().collect()
    }

    /// Allocate resources
    pub fn allocate_resource(
        &mut self,
        resource_type: ResourceType,
        amount: Decimal,
        requesting_entity: String,
        purpose: String,
    ) -> Result<String> {
        let resource = self.resources.get_mut(&resource_type)
            .ok_or_else(|| EconomyError::ResourceNotAvailable(format!("Resource type {} not found", resource_type)))?;

        resource.reserve(amount)?;

        let allocation_id = format!("alloc_{}", self.next_allocation_id);
        self.next_allocation_id += 1;

        let cost = amount * resource.cost_per_unit;
        let allocation = ResourceAllocation::new(
            allocation_id.clone(),
            resource_type,
            amount,
            requesting_entity.clone(),
            purpose,
            cost,
        );

        let requesting_entity_clone = allocation.requesting_entity.clone();
        self.allocations.insert(allocation_id.clone(), allocation);
        info!("Allocated {} {} to {}", amount, resource.unit, requesting_entity_clone);
        
        Ok(allocation_id)
    }

    /// Release allocated resources
    pub fn release_allocation(&mut self, allocation_id: &str) -> Result<()> {
        let allocation = self.allocations.get_mut(allocation_id)
            .ok_or_else(|| EconomyError::ResourceAllocationError(format!("Allocation {} not found", allocation_id)))?;

        if !allocation.is_active() {
            return Err(EconomyError::ResourceAllocationError(format!("Allocation {} is not active", allocation_id)));
        }

        let resource = self.resources.get_mut(&allocation.resource_type)
            .ok_or_else(|| EconomyError::ResourceAllocationError("Resource not found".to_string()))?;

        resource.release(allocation.allocated_amount)?;
        allocation.complete();

        info!("Released allocation: {}", allocation_id);
        Ok(())
    }

    /// Cancel allocation
    pub fn cancel_allocation(&mut self, allocation_id: &str) -> Result<()> {
        let allocation = self.allocations.get_mut(allocation_id)
            .ok_or_else(|| EconomyError::ResourceAllocationError(format!("Allocation {} not found", allocation_id)))?;

        if !allocation.is_active() {
            return Err(EconomyError::ResourceAllocationError(format!("Allocation {} is not active", allocation_id)));
        }

        let resource = self.resources.get_mut(&allocation.resource_type)
            .ok_or_else(|| EconomyError::ResourceAllocationError("Resource not found".to_string()))?;

        resource.release(allocation.allocated_amount)?;
        allocation.cancel();

        warn!("Cancelled allocation: {}", allocation_id);
        Ok(())
    }

    /// Get active allocations
    pub fn get_active_allocations(&self) -> Vec<&ResourceAllocation> {
        self.allocations
            .values()
            .filter(|alloc| alloc.is_active())
            .collect()
    }

    /// Get allocations by entity
    pub fn get_allocations_by_entity(&self, entity: &str) -> Vec<&ResourceAllocation> {
        self.allocations
            .values()
            .filter(|alloc| alloc.requesting_entity == entity)
            .collect()
    }

    /// Calculate total resource value
    pub fn total_resource_value(&self) -> Decimal {
        self.resources
            .values()
            .map(|resource| resource.total_value())
            .sum()
    }

    /// Calculate total allocation cost
    pub fn total_allocation_cost(&self) -> Decimal {
        self.allocations
            .values()
            .filter(|alloc| alloc.is_active())
            .map(|alloc| alloc.cost)
            .sum()
    }

    /// Get resource utilization rate
    pub fn get_utilization_rate(&self, resource_type: &ResourceType) -> Result<Decimal> {
        let resource = self.get_resource(resource_type)
            .ok_or_else(|| EconomyError::ResourceNotAvailable(format!("Resource {} not found", resource_type)))?;

        if resource.quantity.is_zero() {
            return Ok(Decimal::ZERO);
        }

        let utilization_rate = resource.reserved / resource.quantity;
        Ok(utilization_rate * Decimal::from(100)) // Return as percentage
    }

    /// Cleanup expired allocations
    pub fn cleanup_expired_allocations(&mut self, max_age_hours: i64) -> Result<usize> {
        let cutoff_time = Utc::now() - chrono::Duration::hours(max_age_hours);
        let mut cleaned_count = 0;

        let expired_ids: Vec<String> = self.allocations
            .iter()
            .filter_map(|(id, alloc)| {
                if alloc.status == AllocationStatus::Completed || alloc.status == AllocationStatus::Cancelled {
                    if let Some(end_time) = alloc.end_time {
                        if end_time < cutoff_time {
                            return Some(id.clone());
                        }
                    }
                }
                None
            })
            .collect();

        for id in expired_ids {
            self.allocations.remove(&id);
            cleaned_count += 1;
        }

        if cleaned_count > 0 {
            info!("Cleaned up {} expired allocations", cleaned_count);
        }

        Ok(cleaned_count)
    }
}

impl Default for ResourceManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_resource_creation() {
        let resource = Resource::new(
            ResourceType::Computational,
            dec!(100.0),
            "cores".to_string(),
            dec!(0.1),
        );

        assert_eq!(resource.quantity, dec!(100.0));
        assert_eq!(resource.available_quantity(), dec!(100.0));
        assert_eq!(resource.total_value(), dec!(10.0));
    }

    #[test]
    fn test_resource_reservation() {
        let mut resource = Resource::new(
            ResourceType::Storage,
            dec!(1000.0),
            "GB".to_string(),
            dec!(0.01),
        );

        resource.reserve(dec!(100.0)).unwrap();
        assert_eq!(resource.reserved, dec!(100.0));
        assert_eq!(resource.available_quantity(), dec!(900.0));
    }

    #[test]
    fn test_resource_manager() {
        let mut manager = ResourceManager::new();
        let resource = Resource::new(
            ResourceType::Computational,
            dec!(50.0),
            "cores".to_string(),
            dec!(1.0),
        );

        manager.add_resource(resource);
        assert_eq!(manager.available_resources().len(), 1);

        let allocation_id = manager.allocate_resource(
            ResourceType::Computational,
            dec!(10.0),
            "test_entity".to_string(),
            "testing".to_string(),
        ).unwrap();

        assert!(!allocation_id.is_empty());
        assert_eq!(manager.get_active_allocations().len(), 1);

        manager.release_allocation(&allocation_id).unwrap();
        assert_eq!(manager.get_active_allocations().len(), 0);
    }

    #[test]
    fn test_resource_utilization() {
        let mut manager = ResourceManager::new();
        let resource = Resource::new(
            ResourceType::Network,
            dec!(100.0),
            "Mbps".to_string(),
            dec!(0.5),
        );

        manager.add_resource(resource);
        
        manager.allocate_resource(
            ResourceType::Network,
            dec!(25.0),
            "test".to_string(),
            "test".to_string(),
        ).unwrap();

        let utilization = manager.get_utilization_rate(&ResourceType::Network).unwrap();
        assert_eq!(utilization, dec!(25.0));
    }

    #[test]
    fn test_insufficient_resources() {
        let mut manager = ResourceManager::new();
        let resource = Resource::new(
            ResourceType::Computational,
            dec!(10.0),
            "cores".to_string(),
            dec!(1.0),
        );

        manager.add_resource(resource);

        let result = manager.allocate_resource(
            ResourceType::Computational,
            dec!(20.0),
            "test".to_string(),
            "test".to_string(),
        );

        assert!(result.is_err());
    }
}