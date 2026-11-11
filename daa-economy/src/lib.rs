//! # DAA Economy
//!
//! Economic modeling and decision-making framework for Decentralized Autonomous Applications.
//! This crate provides tools for market analysis, resource allocation, risk assessment,
//! and economic optimization within the DAA ecosystem.

pub mod accounts;
pub mod error;
pub mod market;
pub mod optimization;
pub mod resources;
pub mod risk;
pub mod trading;

pub use error::{EconomyError, Result};
pub use market::{MarketAnalyzer, MarketData, PricePoint};
pub use optimization::{EconomicOptimizer, OptimizationResult, OptimizationStrategy};
pub use resources::{Resource, ResourceAllocation, ResourceManager};
pub use risk::{RiskAssessment, RiskFactor, RiskLevel};
pub use trading::{TradeExecution, TradeOrder, TradingEngine};

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn market_maker_config() -> market::MarketMakerConfig {
        market::MarketMakerConfig {
            spread: dec!(0.01),
            depth: 10,
        }
    }

    #[tokio::test]
    async fn test_economic_components_integration() {
        // Test that all major components can be instantiated
        let market_manager = market::MarketManager::new(market_maker_config())
            .await
            .unwrap();
        let resource_manager = ResourceManager::new();
        let trading_engine = TradingEngine::new();

        assert!(market_manager.get_market_data("BTC", "USD").await.is_ok());
        assert_eq!(resource_manager.available_resources().len(), 0);
        assert!(trading_engine.get_account_balance().is_ok());
    }

    #[tokio::test]
    async fn test_integration_workflow() {
        // Test integration between components
        let mut trading_engine = TradingEngine::new();
        let mut market_manager = market::MarketManager::new(market_maker_config())
            .await
            .unwrap();
        market_manager.initialize().await.unwrap();

        // Set up initial conditions
        trading_engine.set_balance("USD".to_string(), dec!(10000.0));
        trading_engine.update_market_price("BTC".to_string(), dec!(50000.0));

        // Create market data and feed it to the optimizer
        let btc_data = MarketData {
            base_token: "BTC".to_string(),
            quote_token: "USD".to_string(),
            last_price: dec!(50000.0),
            bid_price: dec!(49990.0),
            ask_price: dec!(50010.0),
            volume_24h: dec!(100.0),
            price_change_24h: dec!(0.02),
        };
        let mut optimizer = EconomicOptimizer::new();
        optimizer.add_market_data("BTC".to_string(), btc_data);

        // Test resource allocation
        let mut resource_manager = ResourceManager::new();
        let resource = resources::Resource::new(
            resources::ResourceType::Token("BTC".to_string()),
            dec!(100.0),
            "BTC".to_string(),
            dec!(50000.0),
        );
        resource_manager.add_resource(resource);

        assert_eq!(resource_manager.available_resources().len(), 1);
        let quote = market_manager.get_market_data("BTC", "USD").await.unwrap();
        assert_eq!(quote.base_token, "BTC");
        assert!(trading_engine.get_account_balance().is_ok());
    }
}
