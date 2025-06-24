//! Stub modules for QuDAG exchange types

#[derive(Debug, Clone)]
pub struct Exchange;

#[derive(Debug, Clone)]
pub struct ExchangeConfig;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Order {
    pub id: String,
    pub order_type: OrderType,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum OrderType {
    Buy,
    Sell,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Trade {
    pub id: String,
}

pub mod qudag_exchange {
    pub use super::{Exchange, ExchangeConfig, Order, OrderType, Trade};
}
