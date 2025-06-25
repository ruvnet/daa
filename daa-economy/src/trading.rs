//! Trading engine and order management

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use num_traits::ToPrimitive;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};

use crate::error::{EconomyError, Result};

/// Order types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OrderType {
    Market,
    Limit,
    StopLoss,
    TakeProfit,
    TrailingStop,
}

/// Order side (buy or sell)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OrderSide {
    Buy,
    Sell,
}

/// Order status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OrderStatus {
    Pending,
    PartiallyFilled,
    Filled,
    Cancelled,
    Rejected,
    Expired,
}

/// Trading order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeOrder {
    pub id: String,
    pub symbol: String,
    pub order_type: OrderType,
    pub side: OrderSide,
    pub quantity: Decimal,
    pub price: Option<Decimal>, // None for market orders
    pub stop_price: Option<Decimal>, // For stop orders
    pub filled_quantity: Decimal,
    pub average_fill_price: Decimal,
    pub status: OrderStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub client_order_id: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl TradeOrder {
    pub fn new(
        id: String,
        symbol: String,
        order_type: OrderType,
        side: OrderSide,
        quantity: Decimal,
    ) -> Self {
        Self {
            id,
            symbol,
            order_type,
            side,
            quantity,
            price: None,
            stop_price: None,
            filled_quantity: Decimal::ZERO,
            average_fill_price: Decimal::ZERO,
            status: OrderStatus::Pending,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            expires_at: None,
            client_order_id: None,
            metadata: HashMap::new(),
        }
    }

    pub fn with_price(mut self, price: Decimal) -> Self {
        self.price = Some(price);
        self
    }

    pub fn with_stop_price(mut self, stop_price: Decimal) -> Self {
        self.stop_price = Some(stop_price);
        self
    }

    pub fn with_expiry(mut self, expires_at: DateTime<Utc>) -> Self {
        self.expires_at = Some(expires_at);
        self
    }

    pub fn with_client_id(mut self, client_id: String) -> Self {
        self.client_order_id = Some(client_id);
        self
    }

    /// Get remaining quantity to fill
    pub fn remaining_quantity(&self) -> Decimal {
        self.quantity - self.filled_quantity
    }

    /// Check if order is fully filled
    pub fn is_filled(&self) -> bool {
        self.filled_quantity >= self.quantity
    }

    /// Check if order is active (can be filled)
    pub fn is_active(&self) -> bool {
        matches!(self.status, OrderStatus::Pending | OrderStatus::PartiallyFilled)
    }

    /// Check if order is expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }

    /// Partially fill the order
    pub fn fill(&mut self, fill_quantity: Decimal, fill_price: Decimal) -> Result<()> {
        if !self.is_active() {
            return Err(EconomyError::TradingError(
                format!("Cannot fill order {} with status {:?}", self.id, self.status)
            ));
        }

        if fill_quantity > self.remaining_quantity() {
            return Err(EconomyError::TradingError(
                format!("Fill quantity {} exceeds remaining quantity {}", 
                        fill_quantity, self.remaining_quantity())
            ));
        }

        // Update filled quantity and average price
        let total_value = self.average_fill_price * self.filled_quantity + fill_price * fill_quantity;
        self.filled_quantity += fill_quantity;
        self.average_fill_price = total_value / self.filled_quantity;
        
        // Update status
        if self.is_filled() {
            self.status = OrderStatus::Filled;
        } else {
            self.status = OrderStatus::PartiallyFilled;
        }

        self.updated_at = Utc::now();
        debug!("Filled order {}: {} @ {}", self.id, fill_quantity, fill_price);
        
        Ok(())
    }

    /// Cancel the order
    pub fn cancel(&mut self) -> Result<()> {
        if !self.is_active() {
            return Err(EconomyError::TradingError(
                format!("Cannot cancel order {} with status {:?}", self.id, self.status)
            ));
        }

        self.status = OrderStatus::Cancelled;
        self.updated_at = Utc::now();
        debug!("Cancelled order {}", self.id);
        
        Ok(())
    }

    /// Reject the order
    pub fn reject(&mut self, reason: String) -> Result<()> {
        self.status = OrderStatus::Rejected;
        self.updated_at = Utc::now();
        self.metadata.insert("rejection_reason".to_string(), serde_json::Value::String(reason));
        debug!("Rejected order {}", self.id);
        
        Ok(())
    }
}

/// Trade execution record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeExecution {
    pub id: String,
    pub order_id: String,
    pub symbol: String,
    pub side: OrderSide,
    pub quantity: Decimal,
    pub price: Decimal,
    pub fee: Decimal,
    pub fee_currency: String,
    pub execution_time: DateTime<Utc>,
    pub counterparty: Option<String>,
    pub execution_venue: String,
}

impl TradeExecution {
    pub fn new(
        id: String,
        order_id: String,
        symbol: String,
        side: OrderSide,
        quantity: Decimal,
        price: Decimal,
    ) -> Self {
        Self {
            id,
            order_id,
            symbol,
            side,
            quantity,
            price,
            fee: Decimal::ZERO,
            fee_currency: "USD".to_string(),
            execution_time: Utc::now(),
            counterparty: None,
            execution_venue: "LOCAL".to_string(),
        }
    }

    pub fn with_fee(mut self, fee: Decimal, currency: String) -> Self {
        self.fee = fee;
        self.fee_currency = currency;
        self
    }

    pub fn with_venue(mut self, venue: String) -> Self {
        self.execution_venue = venue;
        self
    }

    /// Calculate total value of the trade
    pub fn total_value(&self) -> Decimal {
        self.quantity * self.price
    }

    /// Calculate net value after fees
    pub fn net_value(&self) -> Decimal {
        match self.side {
            OrderSide::Buy => self.total_value() + self.fee,
            OrderSide::Sell => self.total_value() - self.fee,
        }
    }
}

/// Account balance information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountBalance {
    pub asset: String,
    pub free_balance: Decimal,
    pub locked_balance: Decimal,
    pub total_balance: Decimal,
    pub last_updated: DateTime<Utc>,
}

impl AccountBalance {
    pub fn new(asset: String, total_balance: Decimal) -> Self {
        Self {
            asset,
            free_balance: total_balance,
            locked_balance: Decimal::ZERO,
            total_balance,
            last_updated: Utc::now(),
        }
    }

    /// Lock funds for an order
    pub fn lock_funds(&mut self, amount: Decimal) -> Result<()> {
        if self.free_balance < amount {
            return Err(EconomyError::InsufficientFunds {
                required: amount.to_u128().unwrap_or(0),
                available: self.free_balance.to_u128().unwrap_or(0),
            });
        }

        self.free_balance -= amount;
        self.locked_balance += amount;
        self.last_updated = Utc::now();
        
        Ok(())
    }

    /// Unlock funds
    pub fn unlock_funds(&mut self, amount: Decimal) -> Result<()> {
        if self.locked_balance < amount {
            return Err(EconomyError::TradingError(
                format!("Cannot unlock {} {}, only {} locked", 
                        amount, self.asset, self.locked_balance)
            ));
        }

        self.locked_balance -= amount;
        self.free_balance += amount;
        self.last_updated = Utc::now();
        
        Ok(())
    }

    /// Update total balance (and adjust free balance accordingly)
    pub fn update_balance(&mut self, new_total: Decimal) {
        let balance_change = new_total - self.total_balance;
        self.total_balance = new_total;
        self.free_balance += balance_change;
        self.last_updated = Utc::now();
    }
}

/// Trading engine
pub struct TradingEngine {
    orders: HashMap<String, TradeOrder>,
    executions: Vec<TradeExecution>,
    balances: HashMap<String, AccountBalance>,
    market_prices: HashMap<String, Decimal>,
    next_order_id: u64,
    next_execution_id: u64,
    fee_rate: Decimal,
    max_orders_per_symbol: usize,
}

impl TradingEngine {
    pub fn new() -> Self {
        Self {
            orders: HashMap::new(),
            executions: Vec::new(),
            balances: HashMap::new(),
            market_prices: HashMap::new(),
            next_order_id: 1,
            next_execution_id: 1,
            fee_rate: rust_decimal_macros::dec!(0.001), // 0.1% default fee
            max_orders_per_symbol: 100,
        }
    }

    pub fn with_fee_rate(mut self, fee_rate: Decimal) -> Self {
        self.fee_rate = fee_rate;
        self
    }

    /// Set account balance
    pub fn set_balance(&mut self, asset: String, balance: Decimal) {
        self.balances.insert(asset.clone(), AccountBalance::new(asset, balance));
    }

    /// Get account balance
    pub fn get_account_balance(&self) -> Result<&HashMap<String, AccountBalance>> {
        Ok(&self.balances)
    }

    /// Get balance for specific asset
    pub fn get_balance(&self, asset: &str) -> Decimal {
        self.balances.get(asset)
            .map(|b| b.total_balance)
            .unwrap_or(Decimal::ZERO)
    }

    /// Update market price
    pub fn update_market_price(&mut self, symbol: String, price: Decimal) {
        self.market_prices.insert(symbol, price);
    }

    /// Place a trade order
    pub fn place_order(&mut self, mut order: TradeOrder) -> Result<String> {
        // Generate order ID if not provided
        if order.id.is_empty() {
            order.id = format!("order_{}", self.next_order_id);
            self.next_order_id += 1;
        }

        // Validate order
        self.validate_order(&order)?;

        // Check if we can place the order (symbol limits, etc.)
        let symbol_orders = self.orders.values()
            .filter(|o| o.symbol == order.symbol && o.is_active())
            .count();
        
        if symbol_orders >= self.max_orders_per_symbol {
            return Err(EconomyError::TradingError(
                format!("Maximum orders per symbol ({}) exceeded for {}", 
                        self.max_orders_per_symbol, order.symbol)
            ));
        }

        // Lock funds if needed
        if let Err(e) = self.lock_order_funds(&order) {
            order.reject(e.to_string())?;
            return Err(e);
        }

        // Try to fill the order immediately if it's a market order
        if order.order_type == OrderType::Market {
            self.try_fill_market_order(&mut order)?;
        }

        let order_id = order.id.clone();
        self.orders.insert(order_id.clone(), order);
        
        info!("Placed order: {}", order_id);
        Ok(order_id)
    }

    /// Cancel an order
    pub fn cancel_order(&mut self, order_id: &str) -> Result<()> {
        let order = self.orders.get_mut(order_id)
            .ok_or_else(|| EconomyError::TradingError(format!("Order {} not found", order_id)))?;

        if !order.is_active() {
            return Err(EconomyError::TradingError(
                format!("Order {} cannot be cancelled (status: {:?})", order_id, order.status)
            ));
        }

        // Cancel the order
        order.cancel()?;
        
        // Unlock funds
        // Funds unlocking handled elsewhere
        
        info!("Cancelled order: {}", order_id);
        Ok(())
    }

    /// Get order by ID
    pub fn get_order(&self, order_id: &str) -> Option<&TradeOrder> {
        self.orders.get(order_id)
    }

    /// Get all orders for a symbol
    pub fn get_orders_by_symbol(&self, symbol: &str) -> Vec<&TradeOrder> {
        self.orders.values()
            .filter(|order| order.symbol == symbol)
            .collect()
    }

    /// Get active orders
    pub fn get_active_orders(&self) -> Vec<&TradeOrder> {
        self.orders.values()
            .filter(|order| order.is_active())
            .collect()
    }

    /// Get trade executions
    pub fn get_executions(&self) -> &[TradeExecution] {
        &self.executions
    }

    /// Process market data and trigger order fills
    pub fn process_market_update(&mut self, symbol: String, price: Decimal) -> Result<Vec<String>> {
        self.update_market_price(symbol.clone(), price);
        
        let mut filled_orders = Vec::new();
        let order_ids: Vec<String> = self.orders.keys().cloned().collect();
        
        for order_id in order_ids {
            if let Some(order) = self.orders.get(&order_id) {
                if order.symbol == symbol && order.is_active() {
                    let should_fill = match order.order_type {
                        OrderType::Limit => {
                            match order.side {
                                OrderSide::Buy => order.price.map_or(false, |p| price <= p),
                                OrderSide::Sell => order.price.map_or(false, |p| price >= p),
                            }
                        }
                        OrderType::StopLoss => {
                            order.stop_price.map_or(false, |p| 
                                match order.side {
                                    OrderSide::Buy => price >= p,
                                    OrderSide::Sell => price <= p,
                                }
                            )
                        }
                        _ => false,
                    };

                    if should_fill {
                        if let Ok(()) = self.fill_order(&order_id, order.remaining_quantity(), price) {
                            filled_orders.push(order_id);
                        }
                    }
                }
            }
        }

        // Check for expired orders
        self.process_expired_orders()?;

        Ok(filled_orders)
    }

    /// Validate order before placement
    fn validate_order(&self, order: &TradeOrder) -> Result<()> {
        if order.quantity <= Decimal::ZERO {
            return Err(EconomyError::TradingError("Order quantity must be positive".to_string()));
        }

        if order.symbol.is_empty() {
            return Err(EconomyError::TradingError("Order symbol cannot be empty".to_string()));
        }

        // Validate price for limit orders
        if order.order_type == OrderType::Limit && order.price.is_none() {
            return Err(EconomyError::TradingError("Limit orders must have a price".to_string()));
        }

        // Validate stop price for stop orders
        if matches!(order.order_type, OrderType::StopLoss | OrderType::TakeProfit) && order.stop_price.is_none() {
            return Err(EconomyError::TradingError("Stop orders must have a stop price".to_string()));
        }

        Ok(())
    }

    /// Lock funds for an order
    fn lock_order_funds(&mut self, order: &TradeOrder) -> Result<()> {
        let (asset, amount) = match order.side {
            OrderSide::Buy => {
                // For buy orders, lock the quote currency
                let price = order.price.or_else(|| self.market_prices.get(&order.symbol).copied())
                    .unwrap_or(rust_decimal_macros::dec!(1.0));
                ("USD".to_string(), order.quantity * price) // Simplified: assume USD quote
            }
            OrderSide::Sell => {
                // For sell orders, lock the base asset
                (order.symbol.clone(), order.quantity)
            }
        };

        if let Some(balance) = self.balances.get_mut(&asset) {
            balance.lock_funds(amount)?;
        } else {
            return Err(EconomyError::InsufficientFunds {
                required: amount.to_u128().unwrap_or(0),
                available: 0,
            });
        }

        Ok(())
    }

    /// Unlock funds for an order
    fn unlock_order_funds(&mut self, order: &TradeOrder) -> Result<()> {
        let (asset, amount) = match order.side {
            OrderSide::Buy => {
                let price = order.price.or_else(|| self.market_prices.get(&order.symbol).copied())
                    .unwrap_or(rust_decimal_macros::dec!(1.0));
                ("USD".to_string(), order.remaining_quantity() * price)
            }
            OrderSide::Sell => {
                (order.symbol.clone(), order.remaining_quantity())
            }
        };

        if let Some(balance) = self.balances.get_mut(&asset) {
            balance.unlock_funds(amount)?;
        }

        Ok(())
    }

    /// Try to fill market order immediately
    fn try_fill_market_order(&mut self, order: &mut TradeOrder) -> Result<()> {
        if let Some(&market_price) = self.market_prices.get(&order.symbol) {
            let fill_quantity = order.remaining_quantity();
            let execution_id = format!("exec_{}", self.next_execution_id);
            self.next_execution_id += 1;

            // Create execution record
            let execution = TradeExecution::new(
                execution_id,
                order.id.clone(),
                order.symbol.clone(),
                order.side.clone(),
                fill_quantity,
                market_price,
            ).with_fee(fill_quantity * market_price * self.fee_rate, "USD".to_string());

            // Fill the order
            order.fill(fill_quantity, market_price)?;

            // Update balances
            self.update_balances_after_execution(&execution)?;

            // Store execution
            self.executions.push(execution);

            info!("Filled market order {} at price {}", order.id, market_price);
        }

        Ok(())
    }

    /// Fill an order
    fn fill_order(&mut self, order_id: &str, quantity: Decimal, price: Decimal) -> Result<()> {
        let execution_id = format!("exec_{}", self.next_execution_id);
        self.next_execution_id += 1;

        // Get order
        let order = self.orders.get_mut(order_id)
            .ok_or_else(|| EconomyError::TradingError(format!("Order {} not found", order_id)))?;

        // Create execution
        let execution = TradeExecution::new(
            execution_id,
            order_id.to_string(),
            order.symbol.clone(),
            order.side.clone(),
            quantity,
            price,
        ).with_fee(quantity * price * self.fee_rate, "USD".to_string());

        // Fill the order
        order.fill(quantity, price)?;

        // Update balances
        self.update_balances_after_execution(&execution)?;

        // Store execution
        self.executions.push(execution);

        info!("Filled order {} with {} @ {}", order_id, quantity, price);
        Ok(())
    }

    /// Update balances after trade execution
    fn update_balances_after_execution(&mut self, execution: &TradeExecution) -> Result<()> {
        match execution.side {
            OrderSide::Buy => {
                // Decrease quote currency (USD), increase base currency
                if let Some(quote_balance) = self.balances.get_mut("USD") {
                    quote_balance.unlock_funds(execution.net_value())?;
                    quote_balance.update_balance(quote_balance.total_balance - execution.net_value());
                }

                let base_balance = self.balances.entry(execution.symbol.clone())
                    .or_insert_with(|| AccountBalance::new(execution.symbol.clone(), Decimal::ZERO));
                base_balance.update_balance(base_balance.total_balance + execution.quantity);
            }
            OrderSide::Sell => {
                // Decrease base currency, increase quote currency (USD)
                if let Some(base_balance) = self.balances.get_mut(&execution.symbol) {
                    base_balance.unlock_funds(execution.quantity)?;
                    base_balance.update_balance(base_balance.total_balance - execution.quantity);
                }

                let quote_balance = self.balances.entry("USD".to_string())
                    .or_insert_with(|| AccountBalance::new("USD".to_string(), Decimal::ZERO));
                quote_balance.update_balance(quote_balance.total_balance + execution.net_value());
            }
        }

        Ok(())
    }

    /// Process expired orders
    fn process_expired_orders(&mut self) -> Result<()> {
        let expired_order_ids: Vec<String> = self.orders.iter()
            .filter_map(|(id, order)| {
                if order.is_expired() && order.is_active() {
                    Some(id.clone())
                } else {
                    None
                }
            })
            .collect();

        for order_id in expired_order_ids {
            if let Some(order) = self.orders.get_mut(&order_id) {
                // Funds handling for expired orders
                order.status = OrderStatus::Expired;
                order.updated_at = Utc::now();
                info!("Expired order: {}", order_id);
            }
        }

        Ok(())
    }

    /// Get trading statistics
    pub fn get_trading_stats(&self) -> TradingStats {
        let total_orders = self.orders.len();
        let filled_orders = self.orders.values().filter(|o| o.status == OrderStatus::Filled).count();
        let active_orders = self.orders.values().filter(|o| o.is_active()).count();
        let total_volume = self.executions.iter().map(|e| e.total_value()).sum();
        let total_fees = self.executions.iter().map(|e| e.fee).sum();

        TradingStats {
            total_orders,
            filled_orders,
            active_orders,
            total_executions: self.executions.len(),
            total_volume,
            total_fees,
        }
    }

    fn unlock_order_funds_by_id(&mut self, order_id: &str) -> Result<()> {
        if let Some(order) = self.orders.get(order_id) {
            if order.filled_quantity > Decimal::ZERO {
                return Ok(());
            }
            // Funds already unlocked
        }
        Ok(())
    }
}

impl Default for TradingEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Trading statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingStats {
    pub total_orders: usize,
    pub filled_orders: usize,
    pub active_orders: usize,
    pub total_executions: usize,
    pub total_volume: Decimal,
    pub total_fees: Decimal,
}

impl TradingStats {
    pub fn fill_rate(&self) -> f64 {
        if self.total_orders > 0 {
            self.filled_orders as f64 / self.total_orders as f64
        } else {
            0.0
        }
    }

    pub fn summary(&self) -> String {
        format!(
            "Trading Stats: {} orders ({} filled, {:.1}% fill rate), {} executions, Volume: {}, Fees: {}",
            self.total_orders,
            self.filled_orders,
            self.fill_rate() * 100.0,
            self.total_executions,
            self.total_volume,
            self.total_fees
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_trade_order() {
        let mut order = TradeOrder::new(
            "test_1".to_string(),
            "BTC".to_string(),
            OrderType::Limit,
            OrderSide::Buy,
            dec!(1.0),
        ).with_price(dec!(50000.0));

        assert_eq!(order.remaining_quantity(), dec!(1.0));
        assert!(order.is_active());

        order.fill(dec!(0.5), dec!(50000.0)).unwrap();
        assert_eq!(order.filled_quantity, dec!(0.5));
        assert_eq!(order.status, OrderStatus::PartiallyFilled);

        order.fill(dec!(0.5), dec!(51000.0)).unwrap();
        assert!(order.is_filled());
        assert_eq!(order.status, OrderStatus::Filled);
        assert_eq!(order.average_fill_price, dec!(50500.0));
    }

    #[test]
    fn test_trade_execution() {
        let execution = TradeExecution::new(
            "exec_1".to_string(),
            "order_1".to_string(),
            "BTC".to_string(),
            OrderSide::Buy,
            dec!(1.0),
            dec!(50000.0),
        ).with_fee(dec!(50.0), "USD".to_string());

        assert_eq!(execution.total_value(), dec!(50000.0));
        assert_eq!(execution.net_value(), dec!(50050.0)); // Buy side adds fees
    }

    #[test]
    fn test_account_balance() {
        let mut balance = AccountBalance::new("BTC".to_string(), dec!(10.0));
        
        balance.lock_funds(dec!(3.0)).unwrap();
        assert_eq!(balance.free_balance, dec!(7.0));
        assert_eq!(balance.locked_balance, dec!(3.0));

        balance.unlock_funds(dec!(1.0)).unwrap();
        assert_eq!(balance.free_balance, dec!(8.0));
        assert_eq!(balance.locked_balance, dec!(2.0));
    }

    #[test]
    fn test_trading_engine() {
        let mut engine = TradingEngine::new();
        engine.set_balance("USD".to_string(), dec!(100000.0));
        engine.set_balance("BTC".to_string(), dec!(2.0));
        engine.update_market_price("BTC".to_string(), dec!(50000.0));

        // Place a market buy order
        let order = TradeOrder::new(
            "".to_string(), // Engine will generate ID
            "BTC".to_string(),
            OrderType::Market,
            OrderSide::Buy,
            dec!(0.1),
        );

        let order_id = engine.place_order(order).unwrap();
        assert!(!order_id.is_empty());

        // Check that order was filled
        let placed_order = engine.get_order(&order_id).unwrap();
        assert_eq!(placed_order.status, OrderStatus::Filled);

        // Check balances updated
        let btc_balance = engine.get_balance("BTC");
        assert!(btc_balance > dec!(2.0)); // Should have more BTC now
    }

    #[test]
    fn test_order_cancellation() {
        let mut engine = TradingEngine::new();
        engine.set_balance("USD".to_string(), dec!(100000.0));

        // Place a limit order that won't fill immediately
        let order = TradeOrder::new(
            "test_cancel".to_string(),
            "BTC".to_string(),
            OrderType::Limit,
            OrderSide::Buy,
            dec!(1.0),
        ).with_price(dec!(40000.0)); // Below market price

        engine.place_order(order).unwrap();
        assert!(engine.get_order("test_cancel").unwrap().is_active());

        // Cancel the order
        engine.cancel_order("test_cancel").unwrap();
        assert_eq!(engine.get_order("test_cancel").unwrap().status, OrderStatus::Cancelled);
    }
}
