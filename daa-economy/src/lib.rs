//! # DAA Economy
//! 
//! Economic modeling and decision-making framework for Decentralized Autonomous Applications.
//! This crate provides tools for market analysis, resource allocation, risk assessment,
//! and economic optimization within the DAA ecosystem.

pub mod market;
pub mod resources;
pub mod risk;
pub mod optimization;
pub mod trading;
pub mod error;

pub use error::{EconomyError, Result};
pub use market::{MarketAnalyzer, MarketData, PricePoint};
pub use resources::{ResourceManager, Resource, ResourceAllocation};
pub use risk::{RiskAssessment, RiskFactor, RiskLevel};
pub use optimization::{EconomicOptimizer, OptimizationStrategy, OptimizationResult};
pub use trading::{TradingEngine, TradeOrder, TradeExecution};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_economic_components_integration() {
        // Test that all major components can be instantiated
        let market_analyzer = MarketAnalyzer::new();
        let resource_manager = ResourceManager::new();
        let trading_engine = TradingEngine::new();
        
        assert!(market_analyzer.get_market_data().is_ok());
        assert_eq!(resource_manager.available_resources().len(), 0);
        assert!(trading_engine.get_account_balance().is_ok());
    }

    #[test]
    fn test_integration_workflow() {
        // Test integration between components
        let mut trading_engine = TradingEngine::new();
        let mut market_analyzer = MarketAnalyzer::new();
        
        // Set up initial conditions
        trading_engine.set_balance("USD".to_string(), rust_decimal_macros::dec!(10000.0));
        trading_engine.update_market_price("BTC".to_string(), rust_decimal_macros::dec!(50000.0));
        
        // Create market data
        let mut btc_data = market::MarketData::new("BTC".to_string());
        btc_data.add_price_point(market::PricePoint::new(
            rust_decimal_macros::dec!(50000.0), 
            rust_decimal_macros::dec!(100.0)
        ));
        market_analyzer.add_market_data("BTC".to_string(), btc_data);
        
        // Test resource allocation
        let mut resource_manager = ResourceManager::new();
        let resource = resources::Resource::new(
            resources::ResourceType::Token("BTC".to_string()),
            rust_decimal_macros::dec!(100.0),
            "BTC".to_string(),
            rust_decimal_macros::dec!(50000.0),
        );
        resource_manager.add_resource(resource);
        
        assert_eq!(resource_manager.available_resources().len(), 1);
        assert!(market_analyzer.get_market_data().is_ok());
        assert!(trading_engine.get_account_balance().is_ok());
    }
}