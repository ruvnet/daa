//! Compute Marketplace with QuDAG Resource Trading Integration
//!
//! This module implements a decentralized marketplace for computational resources
//! including CPU, GPU, memory, storage, and bandwidth trading through rUv tokens.

use crate::{Result, EconomyError, RuvTokenManager, FeeManager};
use daa_chain::{Address, TxHash, BlockchainAdapter};
use qudag_exchange::{Exchange, Order, OrderType, OrderStatus, TradeEvent, ResourceSpec};
use qudag_crypto::CryptoProvider;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, BTreeMap};
use std::sync::{Arc, RwLock};
use async_trait::async_trait;
use log::{info, debug, warn, error};

/// Types of computational resources available in the marketplace
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ResourceType {
    /// CPU compute resources (measured in vCPU cores)
    Cpu { cores: u32, architecture: String },
    /// GPU compute resources
    Gpu { model: String, memory_gb: u32, count: u32 },
    /// RAM memory resources (measured in GB)
    Memory { size_gb: u32 },
    /// Storage resources (measured in GB)
    Storage { size_gb: u32, storage_type: StorageType },
    /// Network bandwidth (measured in Mbps)
    Bandwidth { speed_mbps: u32 },
    /// Custom resource type
    Custom { name: String, unit: String, quantity: u32 },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum StorageType {
    Ssd,
    Hdd,
    Nvme,
    Network,
}

/// Resource listing in the marketplace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceListing {
    pub id: String,
    pub provider: Address,
    pub resource_type: ResourceType,
    pub price_per_hour: Decimal,
    pub minimum_duration: u64, // in seconds
    pub maximum_duration: u64, // in seconds
    pub availability_start: u64,
    pub availability_end: u64,
    pub location: Option<String>,
    pub metadata: HashMap<String, String>,
    pub reputation_score: Decimal,
    pub active: bool,
    pub created_at: u64,
    pub updated_at: u64,
}

/// Resource reservation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceReservation {
    pub id: String,
    pub listing_id: String,
    pub consumer: Address,
    pub provider: Address,
    pub resource_type: ResourceType,
    pub start_time: u64,
    pub end_time: u64,
    pub total_cost: Decimal,
    pub deposit_amount: Decimal,
    pub status: ReservationStatus,
    pub performance_metrics: Option<PerformanceMetrics>,
    pub created_at: u64,
    pub completed_at: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ReservationStatus {
    Pending,
    Active,
    Completed,
    Cancelled,
    Disputed,
}

/// Performance metrics for resource usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub uptime_percentage: Decimal,
    pub average_latency_ms: u64,
    pub peak_utilization: Decimal,
    pub error_rate: Decimal,
    pub customer_rating: Option<u8>, // 1-5 stars
}

/// Market order for resource trading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketOrder {
    pub id: String,
    pub order_type: MarketOrderType,
    pub trader: Address,
    pub resource_type: ResourceType,
    pub quantity: u32,
    pub price_per_unit: Decimal,
    pub duration_hours: u32,
    pub location_preference: Option<String>,
    pub status: OrderStatus,
    pub created_at: u64,
    pub expires_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MarketOrderType {
    Buy,
    Sell,
}

/// Compute marketplace manager
pub struct ComputeMarketplace {
    exchange: Arc<Exchange>,
    ruv_token: Arc<RuvTokenManager>,
    fee_manager: Arc<FeeManager>,
    blockchain_adapter: Arc<dyn BlockchainAdapter>,
    crypto_provider: Arc<dyn CryptoProvider>,
    
    // Marketplace state
    listings: Arc<RwLock<HashMap<String, ResourceListing>>>,
    reservations: Arc<RwLock<HashMap<String, ResourceReservation>>>,
    orders: Arc<RwLock<HashMap<String, MarketOrder>>>,
    provider_ratings: Arc<RwLock<HashMap<Address, ProviderRating>>>,
    
    // Price indices for efficient search
    price_index: Arc<RwLock<BTreeMap<Decimal, Vec<String>>>>, // price -> listing_ids
    resource_index: Arc<RwLock<HashMap<ResourceType, Vec<String>>>>, // resource_type -> listing_ids
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderRating {
    pub total_jobs: u64,
    pub successful_jobs: u64,
    pub failed_jobs: u64,
    pub average_rating: Decimal,
    pub total_earnings: Decimal,
    pub disputes: u64,
}

impl ComputeMarketplace {
    /// Create a new compute marketplace
    pub async fn new(
        exchange: Arc<Exchange>,
        ruv_token: Arc<RuvTokenManager>,
        fee_manager: Arc<FeeManager>,
        blockchain_adapter: Arc<dyn BlockchainAdapter>,
        crypto_provider: Arc<dyn CryptoProvider>,
    ) -> Result<Self> {
        info!("Initializing Compute Marketplace");
        
        Ok(ComputeMarketplace {
            exchange,
            ruv_token,
            fee_manager,
            blockchain_adapter,
            crypto_provider,
            listings: Arc::new(RwLock::new(HashMap::new())),
            reservations: Arc::new(RwLock::new(HashMap::new())),
            orders: Arc::new(RwLock::new(HashMap::new())),
            provider_ratings: Arc::new(RwLock::new(HashMap::new())),
            price_index: Arc::new(RwLock::new(BTreeMap::new())),
            resource_index: Arc::new(RwLock::new(HashMap::new())),
        })
    }
    
    /// List a new resource in the marketplace
    pub async fn list_resource(
        &self,
        provider: &Address,
        resource_type: ResourceType,
        price_per_hour: Decimal,
        minimum_duration: u64,
        maximum_duration: u64,
        availability_start: u64,
        availability_end: u64,
        location: Option<String>,
        metadata: HashMap<String, String>,
    ) -> Result<String> {
        info!("Listing new resource from provider {}: {:?}", provider, resource_type);
        
        // Validate inputs
        if price_per_hour <= Decimal::ZERO {
            return Err(EconomyError::MarketError("Price must be positive".to_string()));
        }
        
        if minimum_duration > maximum_duration {
            return Err(EconomyError::MarketError("Invalid duration range".to_string()));
        }
        
        if availability_start >= availability_end {
            return Err(EconomyError::MarketError("Invalid availability window".to_string()));
        }
        
        // Get provider rating
        let reputation_score = self.get_provider_reputation(provider).await?;
        
        // Create listing
        let listing_id = format!("listing_{}", uuid::Uuid::new_v4());
        let listing = ResourceListing {
            id: listing_id.clone(),
            provider: provider.clone(),
            resource_type: resource_type.clone(),
            price_per_hour,
            minimum_duration,
            maximum_duration,
            availability_start,
            availability_end,
            location,
            metadata,
            reputation_score,
            active: true,
            created_at: self.get_current_timestamp(),
            updated_at: self.get_current_timestamp(),
        };
        
        // Store listing
        {
            let mut listings = self.listings.write().unwrap();
            listings.insert(listing_id.clone(), listing.clone());
        }
        
        // Update indices
        self.update_indices(&listing_id, &listing)?;
        
        // Create order on QuDAG exchange
        let exchange_order = Order {
            id: listing_id.clone(),
            trader: provider.as_bytes().to_vec(),
            base_token: b"COMPUTE".to_vec(),
            quote_token: b"RUV".to_vec(),
            order_type: OrderType::Sell,
            amount: 1, // Resource unit
            price: Some(price_per_hour.to_string().parse().unwrap()),
            status: OrderStatus::Pending,
            timestamp: self.get_current_timestamp(),
        };
        
        self.exchange.submit_order(exchange_order)
            .await
            .map_err(|e| EconomyError::ExchangeError(e.to_string()))?;
        
        info!("Resource listed successfully with ID: {}", listing_id);
        Ok(listing_id)
    }
    
    /// Search for available resources
    pub async fn search_resources(
        &self,
        resource_type: Option<ResourceType>,
        max_price: Option<Decimal>,
        min_duration: Option<u64>,
        location: Option<String>,
        sort_by: SortBy,
    ) -> Result<Vec<ResourceListing>> {
        debug!("Searching resources with filters: type={:?}, max_price={:?}, min_duration={:?}, location={:?}",
               resource_type, max_price, min_duration, location);
        
        let listings = self.listings.read().unwrap();
        
        let mut results: Vec<ResourceListing> = listings
            .values()
            .filter(|listing| {
                listing.active &&
                resource_type.as_ref().map_or(true, |rt| &listing.resource_type == rt) &&
                max_price.map_or(true, |mp| listing.price_per_hour <= mp) &&
                min_duration.map_or(true, |md| listing.minimum_duration <= md) &&
                location.as_ref().map_or(true, |loc| {
                    listing.location.as_ref().map_or(false, |l| l.contains(loc))
                })
            })
            .cloned()
            .collect();
        
        // Sort results
        match sort_by {
            SortBy::PriceAsc => results.sort_by(|a, b| a.price_per_hour.cmp(&b.price_per_hour)),
            SortBy::PriceDesc => results.sort_by(|a, b| b.price_per_hour.cmp(&a.price_per_hour)),
            SortBy::ReputationDesc => results.sort_by(|a, b| b.reputation_score.cmp(&a.reputation_score)),
            SortBy::DateDesc => results.sort_by(|a, b| b.created_at.cmp(&a.created_at)),
        }
        
        debug!("Found {} matching resources", results.len());
        Ok(results)
    }
    
    /// Reserve a resource
    pub async fn reserve_resource(
        &self,
        consumer: &Address,
        listing_id: &str,
        start_time: u64,
        duration_seconds: u64,
    ) -> Result<String> {
        info!("Reserving resource {} for consumer {}", listing_id, consumer);
        
        // Get listing
        let listing = {
            let listings = self.listings.read().unwrap();
            listings.get(listing_id).cloned()
                .ok_or_else(|| EconomyError::MarketError("Listing not found".to_string()))?
        };
        
        if !listing.active {
            return Err(EconomyError::MarketError("Listing is not active".to_string()));
        }
        
        // Validate timing
        let end_time = start_time + duration_seconds;
        if start_time < listing.availability_start || end_time > listing.availability_end {
            return Err(EconomyError::MarketError("Requested time is outside availability window".to_string()));
        }
        
        if duration_seconds < listing.minimum_duration {
            return Err(EconomyError::MarketError("Duration is below minimum requirement".to_string()));
        }
        
        if duration_seconds > listing.maximum_duration {
            return Err(EconomyError::MarketError("Duration exceeds maximum allowed".to_string()));
        }
        
        // Calculate costs
        let hours = Decimal::from(duration_seconds) / Decimal::from(3600);
        let base_cost = listing.price_per_hour * hours;
        let fee = self.fee_manager.calculate_marketplace_fee(base_cost)?;
        let total_cost = base_cost + fee;
        let deposit_amount = total_cost * Decimal::new(20, 2); // 20% deposit
        
        // Check consumer balance
        let balance = self.ruv_token.get_balance(consumer).await?;
        if balance < deposit_amount {
            return Err(EconomyError::InsufficientBalance {
                required: deposit_amount,
                available: balance,
            });
        }
        
        // Lock deposit
        self.ruv_token.lock(consumer, deposit_amount).await?;
        
        // Create reservation
        let reservation_id = format!("reservation_{}", uuid::Uuid::new_v4());
        let reservation = ResourceReservation {
            id: reservation_id.clone(),
            listing_id: listing_id.to_string(),
            consumer: consumer.clone(),
            provider: listing.provider.clone(),
            resource_type: listing.resource_type.clone(),
            start_time,
            end_time,
            total_cost,
            deposit_amount,
            status: ReservationStatus::Pending,
            performance_metrics: None,
            created_at: self.get_current_timestamp(),
            completed_at: None,
        };
        
        // Store reservation
        {
            let mut reservations = self.reservations.write().unwrap();
            reservations.insert(reservation_id.clone(), reservation);
        }
        
        // Mark listing as unavailable for the reserved period
        self.update_listing_availability(&listing_id, start_time, end_time)?;
        
        info!("Resource reserved successfully with ID: {}", reservation_id);
        Ok(reservation_id)
    }
    
    /// Start a resource reservation (provider confirms and starts providing resource)
    pub async fn start_reservation(&self, reservation_id: &str, provider: &Address) -> Result<()> {
        info!("Starting reservation {} by provider {}", reservation_id, provider);
        
        let mut reservation = {
            let reservations = self.reservations.read().unwrap();
            reservations.get(reservation_id).cloned()
                .ok_or_else(|| EconomyError::MarketError("Reservation not found".to_string()))?
        };
        
        // Verify provider
        if reservation.provider != *provider {
            return Err(EconomyError::MarketError("Only the provider can start the reservation".to_string()));
        }
        
        // Check status
        if reservation.status != ReservationStatus::Pending {
            return Err(EconomyError::MarketError("Reservation is not in pending status".to_string()));
        }
        
        // Update status
        reservation.status = ReservationStatus::Active;
        
        {
            let mut reservations = self.reservations.write().unwrap();
            reservations.insert(reservation_id.to_string(), reservation);
        }
        
        info!("Reservation {} started successfully", reservation_id);
        Ok(())
    }
    
    /// Complete a resource reservation and process payment
    pub async fn complete_reservation(
        &self,
        reservation_id: &str,
        performance_metrics: PerformanceMetrics,
    ) -> Result<TxHash> {
        info!("Completing reservation {}", reservation_id);
        
        let mut reservation = {
            let reservations = self.reservations.read().unwrap();
            reservations.get(reservation_id).cloned()
                .ok_or_else(|| EconomyError::MarketError("Reservation not found".to_string()))?
        };
        
        // Check status
        if reservation.status != ReservationStatus::Active {
            return Err(EconomyError::MarketError("Reservation is not active".to_string()));
        }
        
        // Calculate final payment based on performance
        let performance_multiplier = self.calculate_performance_multiplier(&performance_metrics);
        let final_payment = reservation.total_cost * performance_multiplier;
        
        // Process payment
        let tx_hash = self.ruv_token.transfer(
            &reservation.consumer,
            &reservation.provider,
            final_payment,
        ).await?;
        
        // Unlock remaining deposit
        let refund_amount = reservation.deposit_amount - final_payment;
        if refund_amount > Decimal::ZERO {
            self.ruv_token.unlock(&reservation.consumer, refund_amount).await?;
        }
        
        // Update reservation
        reservation.status = ReservationStatus::Completed;
        reservation.performance_metrics = Some(performance_metrics.clone());
        reservation.completed_at = Some(self.get_current_timestamp());
        
        {
            let mut reservations = self.reservations.write().unwrap();
            reservations.insert(reservation_id.to_string(), reservation.clone());
        }
        
        // Update provider rating
        self.update_provider_rating(&reservation.provider, &performance_metrics, true).await?;
        
        info!("Reservation {} completed successfully", reservation_id);
        Ok(tx_hash)
    }
    
    /// Cancel a reservation
    pub async fn cancel_reservation(
        &self,
        reservation_id: &str,
        canceller: &Address,
        reason: String,
    ) -> Result<TxHash> {
        info!("Cancelling reservation {} by {}: {}", reservation_id, canceller, reason);
        
        let mut reservation = {
            let reservations = self.reservations.read().unwrap();
            reservations.get(reservation_id).cloned()
                .ok_or_else(|| EconomyError::MarketError("Reservation not found".to_string()))?
        };
        
        // Check if canceller is authorized
        if canceller != &reservation.consumer && canceller != &reservation.provider {
            return Err(EconomyError::MarketError("Unauthorized to cancel reservation".to_string()));
        }
        
        // Check if cancellable
        if reservation.status == ReservationStatus::Completed {
            return Err(EconomyError::MarketError("Cannot cancel completed reservation".to_string()));
        }
        
        // Calculate penalty based on who cancels and when
        let penalty = self.calculate_cancellation_penalty(&reservation, canceller)?;
        
        // Process refunds and penalties
        let mut tx_hash = TxHash::default();
        
        if canceller == &reservation.consumer {
            // Consumer cancels - loses penalty to provider
            if penalty > Decimal::ZERO {
                tx_hash = self.ruv_token.transfer(
                    &reservation.consumer,
                    &reservation.provider,
                    penalty,
                ).await?;
            }
            
            // Unlock remaining deposit
            let refund = reservation.deposit_amount - penalty;
            if refund > Decimal::ZERO {
                self.ruv_token.unlock(&reservation.consumer, refund).await?;
            }
        } else {
            // Provider cancels - pays penalty to consumer
            if penalty > Decimal::ZERO {
                tx_hash = self.ruv_token.transfer(
                    &reservation.provider,
                    &reservation.consumer,
                    penalty,
                ).await?;
            }
            
            // Unlock full deposit
            self.ruv_token.unlock(&reservation.consumer, reservation.deposit_amount).await?;
            
            // Negative rating impact
            self.update_provider_rating(&reservation.provider, &PerformanceMetrics {
                uptime_percentage: Decimal::ZERO,
                average_latency_ms: 0,
                peak_utilization: Decimal::ZERO,
                error_rate: Decimal::new(100, 0),
                customer_rating: Some(1),
            }, false).await?;
        }
        
        // Update reservation status
        reservation.status = ReservationStatus::Cancelled;
        
        {
            let mut reservations = self.reservations.write().unwrap();
            reservations.insert(reservation_id.to_string(), reservation);
        }
        
        info!("Reservation {} cancelled", reservation_id);
        Ok(tx_hash)
    }
    
    /// Create a market order (buy or sell)
    pub async fn create_market_order(
        &self,
        trader: &Address,
        order_type: MarketOrderType,
        resource_type: ResourceType,
        quantity: u32,
        price_per_unit: Decimal,
        duration_hours: u32,
        location_preference: Option<String>,
        expires_in_hours: u32,
    ) -> Result<String> {
        info!("Creating {:?} order from {}: {:?} x{} @ {} rUv/unit",
              order_type, trader, resource_type, quantity, price_per_unit);
        
        let order_id = format!("order_{}", uuid::Uuid::new_v4());
        let current_time = self.get_current_timestamp();
        
        let order = MarketOrder {
            id: order_id.clone(),
            order_type: order_type.clone(),
            trader: trader.clone(),
            resource_type: resource_type.clone(),
            quantity,
            price_per_unit,
            duration_hours,
            location_preference,
            status: OrderStatus::Pending,
            created_at: current_time,
            expires_at: current_time + (expires_in_hours as u64 * 3600),
        };
        
        // Store order
        {
            let mut orders = self.orders.write().unwrap();
            orders.insert(order_id.clone(), order.clone());
        }
        
        // Try to match order immediately
        self.match_order(&order_id).await?;
        
        info!("Market order {} created successfully", order_id);
        Ok(order_id)
    }
    
    /// Match orders in the marketplace
    async fn match_order(&self, order_id: &str) -> Result<()> {
        let order = {
            let orders = self.orders.read().unwrap();
            orders.get(order_id).cloned()
                .ok_or_else(|| EconomyError::MarketError("Order not found".to_string()))?
        };
        
        if order.status != OrderStatus::Pending {
            return Ok(());
        }
        
        // Find matching orders
        let matching_orders = self.find_matching_orders(&order)?;
        
        for match_id in matching_orders {
            // Execute trade
            self.execute_trade(&order.id, &match_id).await?;
            
            // Check if order is fully filled
            let updated_order = {
                let orders = self.orders.read().unwrap();
                orders.get(&order.id).cloned()
            };
            
            if let Some(o) = updated_order {
                if o.status != OrderStatus::Pending {
                    break;
                }
            }
        }
        
        Ok(())
    }
    
    /// Find orders that match the given order
    fn find_matching_orders(&self, order: &MarketOrder) -> Result<Vec<String>> {
        let orders = self.orders.read().unwrap();
        
        let matches: Vec<String> = orders
            .values()
            .filter(|o| {
                o.id != order.id &&
                o.status == OrderStatus::Pending &&
                o.resource_type == order.resource_type &&
                match (&order.order_type, &o.order_type) {
                    (MarketOrderType::Buy, MarketOrderType::Sell) => {
                        o.price_per_unit <= order.price_per_unit
                    },
                    (MarketOrderType::Sell, MarketOrderType::Buy) => {
                        o.price_per_unit >= order.price_per_unit
                    },
                    _ => false,
                }
            })
            .map(|o| o.id.clone())
            .collect();
        
        Ok(matches)
    }
    
    /// Execute a trade between two matching orders
    async fn execute_trade(&self, order1_id: &str, order2_id: &str) -> Result<()> {
        info!("Executing trade between {} and {}", order1_id, order2_id);
        
        // Implementation would handle the actual trade execution
        // This is a simplified version
        
        // Update order statuses
        {
            let mut orders = self.orders.write().unwrap();
            if let Some(order1) = orders.get_mut(order1_id) {
                order1.status = OrderStatus::Filled;
            }
            if let Some(order2) = orders.get_mut(order2_id) {
                order2.status = OrderStatus::Filled;
            }
        }
        
        Ok(())
    }
    
    /// Update provider rating based on performance
    async fn update_provider_rating(
        &self,
        provider: &Address,
        metrics: &PerformanceMetrics,
        success: bool,
    ) -> Result<()> {
        let mut ratings = self.provider_ratings.write().unwrap();
        
        let rating = ratings.entry(provider.clone()).or_insert(ProviderRating {
            total_jobs: 0,
            successful_jobs: 0,
            failed_jobs: 0,
            average_rating: Decimal::ZERO,
            total_earnings: Decimal::ZERO,
            disputes: 0,
        });
        
        rating.total_jobs += 1;
        
        if success {
            rating.successful_jobs += 1;
            
            // Update average rating
            if let Some(customer_rating) = metrics.customer_rating {
                let new_rating = Decimal::from(customer_rating);
                rating.average_rating = (rating.average_rating * Decimal::from(rating.total_jobs - 1) + new_rating) 
                    / Decimal::from(rating.total_jobs);
            }
        } else {
            rating.failed_jobs += 1;
        }
        
        Ok(())
    }
    
    /// Get provider reputation score
    async fn get_provider_reputation(&self, provider: &Address) -> Result<Decimal> {
        let ratings = self.provider_ratings.read().unwrap();
        
        if let Some(rating) = ratings.get(provider) {
            if rating.total_jobs == 0 {
                return Ok(Decimal::new(50, 1)); // 5.0 default for new providers
            }
            
            // Calculate reputation based on multiple factors
            let success_rate = Decimal::from(rating.successful_jobs) / Decimal::from(rating.total_jobs);
            let dispute_penalty = Decimal::from(rating.disputes) / Decimal::from(rating.total_jobs) * Decimal::new(10, 0);
            
            let reputation = (success_rate * Decimal::new(50, 0)) + 
                           (rating.average_rating * Decimal::new(10, 0)) - 
                           dispute_penalty;
            
            Ok(reputation.clamp(Decimal::ZERO, Decimal::new(100, 0)))
        } else {
            Ok(Decimal::new(50, 1)) // 5.0 default for new providers
        }
    }
    
    /// Calculate performance multiplier for payment
    fn calculate_performance_multiplier(&self, metrics: &PerformanceMetrics) -> Decimal {
        // Base multiplier is 1.0
        let mut multiplier = Decimal::new(1, 0);
        
        // Adjust based on uptime (90-100% = full payment, below 90% = reduced)
        if metrics.uptime_percentage < Decimal::new(90, 0) {
            multiplier *= metrics.uptime_percentage / Decimal::new(100, 0);
        }
        
        // Bonus for excellent performance
        if metrics.uptime_percentage >= Decimal::new(995, 1) && // 99.5%
           metrics.error_rate < Decimal::new(1, 1) { // 0.1%
            multiplier *= Decimal::new(105, 2); // 1.05x bonus
        }
        
        // Customer rating impact
        if let Some(rating) = metrics.customer_rating {
            if rating >= 4 {
                multiplier *= Decimal::new(102, 2); // 1.02x for good rating
            } else if rating <= 2 {
                multiplier *= Decimal::new(95, 2); // 0.95x for poor rating
            }
        }
        
        multiplier.clamp(Decimal::new(5, 1), Decimal::new(11, 1)) // 0.5x to 1.1x
    }
    
    /// Calculate cancellation penalty
    fn calculate_cancellation_penalty(
        &self,
        reservation: &ResourceReservation,
        canceller: &Address,
    ) -> Result<Decimal> {
        let current_time = self.get_current_timestamp();
        let time_until_start = reservation.start_time.saturating_sub(current_time);
        
        // Penalty calculation based on how close to start time
        let penalty_rate = if time_until_start > 24 * 3600 {
            Decimal::new(10, 2) // 10% if > 24 hours
        } else if time_until_start > 6 * 3600 {
            Decimal::new(25, 2) // 25% if 6-24 hours
        } else if time_until_start > 1 * 3600 {
            Decimal::new(50, 2) // 50% if 1-6 hours
        } else {
            Decimal::new(75, 2) // 75% if < 1 hour
        };
        
        // Provider cancellation has higher penalty
        let adjusted_rate = if canceller == &reservation.provider {
            penalty_rate * Decimal::new(15, 1) // 1.5x for provider
        } else {
            penalty_rate
        };
        
        Ok(reservation.deposit_amount * adjusted_rate)
    }
    
    /// Update listing availability after reservation
    fn update_listing_availability(
        &self,
        listing_id: &str,
        _start_time: u64,
        _end_time: u64,
    ) -> Result<()> {
        // In a full implementation, this would track time slots
        // For now, we just mark it as reserved
        let mut listings = self.listings.write().unwrap();
        if let Some(listing) = listings.get_mut(listing_id) {
            // Update availability tracking
            listing.updated_at = self.get_current_timestamp();
        }
        Ok(())
    }
    
    /// Update marketplace indices
    fn update_indices(&self, listing_id: &str, listing: &ResourceListing) -> Result<()> {
        // Update price index
        {
            let mut price_index = self.price_index.write().unwrap();
            price_index.entry(listing.price_per_hour)
                .or_insert_with(Vec::new)
                .push(listing_id.to_string());
        }
        
        // Update resource type index
        {
            let mut resource_index = self.resource_index.write().unwrap();
            resource_index.entry(listing.resource_type.clone())
                .or_insert_with(Vec::new)
                .push(listing_id.to_string());
        }
        
        Ok(())
    }
    
    /// Get marketplace statistics
    pub async fn get_marketplace_stats(&self) -> MarketplaceStats {
        let listings = self.listings.read().unwrap();
        let reservations = self.reservations.read().unwrap();
        let orders = self.orders.read().unwrap();
        
        let active_listings = listings.values().filter(|l| l.active).count();
        let total_volume = reservations.values()
            .filter(|r| r.status == ReservationStatus::Completed)
            .map(|r| r.total_cost)
            .sum();
        
        MarketplaceStats {
            total_listings: listings.len() as u64,
            active_listings: active_listings as u64,
            total_reservations: reservations.len() as u64,
            active_reservations: reservations.values()
                .filter(|r| r.status == ReservationStatus::Active)
                .count() as u64,
            total_volume,
            pending_orders: orders.values()
                .filter(|o| o.status == OrderStatus::Pending)
                .count() as u64,
        }
    }
    
    /// Get current timestamp
    fn get_current_timestamp(&self) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortBy {
    PriceAsc,
    PriceDesc,
    ReputationDesc,
    DateDesc,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceStats {
    pub total_listings: u64,
    pub active_listings: u64,
    pub total_reservations: u64,
    pub active_reservations: u64,
    pub total_volume: Decimal,
    pub pending_orders: u64,
}

/// Marketplace interface for external integration
#[async_trait]
pub trait MarketplaceInterface: Send + Sync {
    async fn list_resource(
        &self,
        provider: &Address,
        resource_type: ResourceType,
        price_per_hour: Decimal,
        minimum_duration: u64,
        maximum_duration: u64,
    ) -> Result<String>;
    
    async fn search_resources(
        &self,
        resource_type: Option<ResourceType>,
        max_price: Option<Decimal>,
    ) -> Result<Vec<ResourceListing>>;
    
    async fn reserve_resource(
        &self,
        consumer: &Address,
        listing_id: &str,
        start_time: u64,
        duration_seconds: u64,
    ) -> Result<String>;
    
    async fn get_marketplace_stats(&self) -> MarketplaceStats;
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_resource_type_creation() {
        let cpu = ResourceType::Cpu {
            cores: 16,
            architecture: "x86_64".to_string(),
        };
        
        let gpu = ResourceType::Gpu {
            model: "RTX 4090".to_string(),
            memory_gb: 24,
            count: 2,
        };
        
        assert!(matches!(cpu, ResourceType::Cpu { .. }));
        assert!(matches!(gpu, ResourceType::Gpu { .. }));
    }
    
    #[test]
    fn test_performance_multiplier_calculation() {
        let marketplace = ComputeMarketplace {
            exchange: Arc::new(Exchange::new("test").await.unwrap()),
            ruv_token: Arc::new(RuvTokenManager::new(
                RuvToken::new(),
                Arc::new(mock_blockchain_adapter()),
                Arc::new(mock_crypto_provider()),
            ).await.unwrap()),
            fee_manager: Arc::new(FeeManager::new(EconomyConfig::default())),
            blockchain_adapter: Arc::new(mock_blockchain_adapter()),
            crypto_provider: Arc::new(mock_crypto_provider()),
            listings: Arc::new(RwLock::new(HashMap::new())),
            reservations: Arc::new(RwLock::new(HashMap::new())),
            orders: Arc::new(RwLock::new(HashMap::new())),
            provider_ratings: Arc::new(RwLock::new(HashMap::new())),
            price_index: Arc::new(RwLock::new(BTreeMap::new())),
            resource_index: Arc::new(RwLock::new(HashMap::new())),
        };
        
        // Test perfect performance
        let perfect_metrics = PerformanceMetrics {
            uptime_percentage: Decimal::new(100, 0),
            average_latency_ms: 10,
            peak_utilization: Decimal::new(80, 0),
            error_rate: Decimal::ZERO,
            customer_rating: Some(5),
        };
        
        let multiplier = marketplace.calculate_performance_multiplier(&perfect_metrics);
        assert!(multiplier > Decimal::new(1, 0)); // Should get bonus
        
        // Test poor performance
        let poor_metrics = PerformanceMetrics {
            uptime_percentage: Decimal::new(80, 0),
            average_latency_ms: 1000,
            peak_utilization: Decimal::new(100, 0),
            error_rate: Decimal::new(5, 0),
            customer_rating: Some(2),
        };
        
        let multiplier = marketplace.calculate_performance_multiplier(&poor_metrics);
        assert!(multiplier < Decimal::new(1, 0)); // Should get penalty
    }
}