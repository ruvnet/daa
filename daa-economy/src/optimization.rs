//! Economic optimization algorithms and strategies

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};

use crate::error::{EconomyError, Result};
use crate::market::{MarketData, PriceTrend};
use crate::resources::{Resource, ResourceType};
use crate::risk::{RiskAssessment, RiskLevel};

/// Optimization objective
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OptimizationObjective {
    MaximizeReturn,
    MinimizeRisk,
    MaximizeSharpeRatio,
    MinimizeCost,
    MaximizeEfficiency,
    BalancedGrowth,
}

impl std::fmt::Display for OptimizationObjective {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OptimizationObjective::MaximizeReturn => write!(f, "Maximize Return"),
            OptimizationObjective::MinimizeRisk => write!(f, "Minimize Risk"),
            OptimizationObjective::MaximizeSharpeRatio => write!(f, "Maximize Sharpe Ratio"),
            OptimizationObjective::MinimizeCost => write!(f, "Minimize Cost"),
            OptimizationObjective::MaximizeEfficiency => write!(f, "Maximize Efficiency"),
            OptimizationObjective::BalancedGrowth => write!(f, "Balanced Growth"),
        }
    }
}

/// Optimization strategy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationStrategy {
    pub name: String,
    pub objective: OptimizationObjective,
    pub constraints: Vec<OptimizationConstraint>,
    pub parameters: HashMap<String, Decimal>,
    pub risk_tolerance: RiskLevel,
    pub time_horizon_days: u32,
}

impl OptimizationStrategy {
    pub fn new(name: String, objective: OptimizationObjective) -> Self {
        Self {
            name,
            objective,
            constraints: Vec::new(),
            parameters: HashMap::new(),
            risk_tolerance: RiskLevel::Medium,
            time_horizon_days: 30,
        }
    }

    pub fn with_risk_tolerance(mut self, risk_tolerance: RiskLevel) -> Self {
        self.risk_tolerance = risk_tolerance;
        self
    }

    pub fn with_time_horizon(mut self, days: u32) -> Self {
        self.time_horizon_days = days;
        self
    }

    pub fn add_constraint(&mut self, constraint: OptimizationConstraint) {
        self.constraints.push(constraint);
    }

    pub fn set_parameter(&mut self, key: String, value: Decimal) {
        self.parameters.insert(key, value);
    }

    pub fn get_parameter(&self, key: &str) -> Option<Decimal> {
        self.parameters.get(key).copied()
    }
}

/// Optimization constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationConstraint {
    pub name: String,
    pub constraint_type: ConstraintType,
    pub target_value: Decimal,
    pub tolerance: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConstraintType {
    MaxRisk,
    MinReturn,
    MaxCost,
    MinLiquidity,
    MaxAllocation(String), // Max allocation to specific asset/resource
    MinDiversification,
}

impl OptimizationConstraint {
    pub fn max_risk(max_risk_score: Decimal) -> Self {
        Self {
            name: "Maximum Risk".to_string(),
            constraint_type: ConstraintType::MaxRisk,
            target_value: max_risk_score,
            tolerance: rust_decimal_macros::dec!(0.05),
        }
    }

    pub fn min_return(min_return_rate: Decimal) -> Self {
        Self {
            name: "Minimum Return".to_string(),
            constraint_type: ConstraintType::MinReturn,
            target_value: min_return_rate,
            tolerance: rust_decimal_macros::dec!(0.01),
        }
    }

    pub fn max_allocation(asset: String, max_percentage: Decimal) -> Self {
        Self {
            name: format!("Max {} Allocation", asset),
            constraint_type: ConstraintType::MaxAllocation(asset),
            target_value: max_percentage,
            tolerance: rust_decimal_macros::dec!(0.02),
        }
    }
}

/// Optimization result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationResult {
    pub strategy_name: String,
    pub objective: OptimizationObjective,
    pub allocations: HashMap<String, Decimal>,
    pub expected_return: Decimal,
    pub expected_risk: Decimal,
    pub expected_cost: Decimal,
    pub sharpe_ratio: Option<Decimal>,
    pub constraints_satisfied: bool,
    pub optimization_score: Decimal,
    pub recommendations: Vec<String>,
    pub timestamp: DateTime<Utc>,
}

impl OptimizationResult {
    pub fn new(strategy_name: String, objective: OptimizationObjective) -> Self {
        Self {
            strategy_name,
            objective,
            allocations: HashMap::new(),
            expected_return: Decimal::ZERO,
            expected_risk: Decimal::ZERO,
            expected_cost: Decimal::ZERO,
            sharpe_ratio: None,
            constraints_satisfied: false,
            optimization_score: Decimal::ZERO,
            recommendations: Vec::new(),
            timestamp: Utc::now(),
        }
    }

    pub fn add_allocation(&mut self, asset: String, allocation: Decimal) {
        self.allocations.insert(asset, allocation);
    }

    pub fn add_recommendation(&mut self, recommendation: String) {
        self.recommendations.push(recommendation);
    }

    pub fn calculate_sharpe_ratio(&mut self, risk_free_rate: Decimal) {
        if self.expected_risk > Decimal::ZERO {
            self.sharpe_ratio = Some((self.expected_return - risk_free_rate) / self.expected_risk);
        }
    }

    pub fn summary(&self) -> String {
        format!(
            "Optimization: {} | Return: {:.2}% | Risk: {:.2} | Cost: {} | Constraints: {}",
            self.strategy_name,
            self.expected_return * rust_decimal_macros::dec!(100),
            self.expected_risk,
            self.expected_cost,
            if self.constraints_satisfied { "✓" } else { "✗" }
        )
    }
}

/// Portfolio allocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioAllocation {
    pub asset: String,
    pub current_allocation: Decimal,
    pub target_allocation: Decimal,
    pub rebalance_amount: Decimal,
    pub rebalance_direction: RebalanceDirection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RebalanceDirection {
    Buy,
    Sell,
    Hold,
}

/// Economic optimizer
pub struct EconomicOptimizer {
    market_data: HashMap<String, MarketData>,
    risk_assessments: HashMap<String, RiskAssessment>,
    resources: HashMap<ResourceType, Resource>,
    optimization_history: Vec<OptimizationResult>,
    risk_free_rate: Decimal,
}

impl EconomicOptimizer {
    pub fn new() -> Self {
        Self {
            market_data: HashMap::new(),
            risk_assessments: HashMap::new(),
            resources: HashMap::new(),
            optimization_history: Vec::new(),
            risk_free_rate: rust_decimal_macros::dec!(0.02), // 2% default risk-free rate
        }
    }

    pub fn with_risk_free_rate(mut self, rate: Decimal) -> Self {
        self.risk_free_rate = rate;
        self
    }

    /// Add market data for optimization
    pub fn add_market_data(&mut self, symbol: String, data: MarketData) {
        self.market_data.insert(symbol, data);
    }

    /// Add risk assessment
    pub fn add_risk_assessment(&mut self, entity: String, assessment: RiskAssessment) {
        self.risk_assessments.insert(entity, assessment);
    }

    /// Add resource for optimization
    pub fn add_resource(&mut self, resource: Resource) {
        self.resources.insert(resource.resource_type.clone(), resource);
    }

    /// Optimize portfolio allocation
    pub fn optimize_portfolio(&mut self, strategy: &OptimizationStrategy, assets: &[String]) -> Result<OptimizationResult> {
        let mut result = OptimizationResult::new(strategy.name.clone(), strategy.objective.clone());

        match strategy.objective {
            OptimizationObjective::MaximizeReturn => {
                self.optimize_for_maximum_return(&mut result, assets, strategy)?;
            }
            OptimizationObjective::MinimizeRisk => {
                self.optimize_for_minimum_risk(&mut result, assets, strategy)?;
            }
            OptimizationObjective::MaximizeSharpeRatio => {
                self.optimize_for_sharpe_ratio(&mut result, assets, strategy)?;
            }
            OptimizationObjective::BalancedGrowth => {
                self.optimize_for_balanced_growth(&mut result, assets, strategy)?;
            }
            _ => {
                return Err(EconomyError::OptimizationError(
                    format!("Optimization objective {:?} not yet implemented", strategy.objective)
                ));
            }
        }

        // Check constraints
        result.constraints_satisfied = self.check_constraints(&result, strategy)?;

        // Calculate optimization score
        result.optimization_score = self.calculate_optimization_score(&result, strategy);

        // Generate recommendations
        self.generate_recommendations(&mut result, strategy);

        // Store in history
        self.optimization_history.push(result.clone());
        info!("Completed optimization: {}", result.summary());

        Ok(result)
    }

    /// Optimize for maximum return
    fn optimize_for_maximum_return(
        &self,
        result: &mut OptimizationResult,
        assets: &[String],
        _strategy: &OptimizationStrategy,
    ) -> Result<()> {
        let mut asset_returns: Vec<(String, Decimal)> = Vec::new();

        // Calculate expected returns for each asset
        for asset in assets {
            if let Some(market_data) = self.market_data.get(asset) {
                let expected_return = self.calculate_expected_return(market_data)?;
                asset_returns.push((asset.clone(), expected_return));
            }
        }

        // Sort by expected return (descending)
        asset_returns.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Allocate based on return ranking (simplified approach)
        let total_assets = asset_returns.len();
        for (i, (asset, expected_return)) in asset_returns.iter().enumerate() {
            let weight = rust_decimal_macros::dec!(1.0) / Decimal::from(total_assets);
            result.add_allocation(asset.clone(), weight);
            result.expected_return += expected_return * weight;
        }

        Ok(())
    }

    /// Optimize for minimum risk
    fn optimize_for_minimum_risk(
        &self,
        result: &mut OptimizationResult,
        assets: &[String],
        _strategy: &OptimizationStrategy,
    ) -> Result<()> {
        let mut asset_risks: Vec<(String, Decimal)> = Vec::new();

        // Calculate risk for each asset
        for asset in assets {
            let risk = if let Some(assessment) = self.risk_assessments.get(asset) {
                assessment.overall_score
            } else if let Some(market_data) = self.market_data.get(asset) {
                market_data.calculate_volatility().unwrap_or(rust_decimal_macros::dec!(0.5))
            } else {
                rust_decimal_macros::dec!(0.5) // Default risk
            };
            asset_risks.push((asset.clone(), risk));
        }

        // Sort by risk (ascending)
        asset_risks.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        // Allocate more to lower-risk assets
        let total_weight = rust_decimal_macros::dec!(1.0);
        let num_assets = Decimal::from(asset_risks.len());
        
        for (asset, risk) in asset_risks {
            let weight = total_weight / num_assets; // Equal weight for simplicity
            result.add_allocation(asset, weight);
            result.expected_risk += risk * weight;
        }

        Ok(())
    }

    /// Optimize for Sharpe ratio
    fn optimize_for_sharpe_ratio(
        &self,
        result: &mut OptimizationResult,
        assets: &[String],
        strategy: &OptimizationStrategy,
    ) -> Result<()> {
        let mut asset_sharpe_ratios: Vec<(String, Decimal)> = Vec::new();

        // Calculate Sharpe ratio for each asset
        for asset in assets {
            if let Some(market_data) = self.market_data.get(asset) {
                let expected_return = self.calculate_expected_return(market_data)?;
                let risk = market_data.calculate_volatility().unwrap_or(rust_decimal_macros::dec!(0.1));
                
                let sharpe_ratio = if risk > Decimal::ZERO {
                    (expected_return - self.risk_free_rate) / risk
                } else {
                    Decimal::ZERO
                };
                
                asset_sharpe_ratios.push((asset.clone(), sharpe_ratio));
            }
        }

        // Sort by Sharpe ratio (descending)
        asset_sharpe_ratios.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Allocate based on Sharpe ratio ranking
        let total_weight = rust_decimal_macros::dec!(1.0);
        let num_assets = Decimal::from(asset_sharpe_ratios.len());
        
        for (asset, _sharpe) in asset_sharpe_ratios {
            let weight = total_weight / num_assets; // Equal weight for simplicity
            result.add_allocation(asset, weight);
        }

        // Calculate portfolio metrics
        self.calculate_portfolio_metrics(result, strategy)?;

        Ok(())
    }

    /// Optimize for balanced growth
    fn optimize_for_balanced_growth(
        &self,
        result: &mut OptimizationResult,
        assets: &[String],
        _strategy: &OptimizationStrategy,
    ) -> Result<()> {
        // Simple balanced approach: equal weights with slight bias towards growth assets
        let num_assets = assets.len();
        let base_weight = rust_decimal_macros::dec!(1.0) / Decimal::from(num_assets);

        for asset in assets {
            // Check if asset is in growth trend
            let growth_bias = if let Some(market_data) = self.market_data.get(asset) {
                match market_data.get_price_trend(30)? {
                    PriceTrend::Bullish => rust_decimal_macros::dec!(0.1),
                    PriceTrend::Bearish => rust_decimal_macros::dec!(-0.05),
                    PriceTrend::Neutral => Decimal::ZERO,
                }
            } else {
                Decimal::ZERO
            };

            let weight = base_weight + growth_bias;
            result.add_allocation(asset.clone(), weight);
        }

        // Normalize weights to ensure they sum to 1.0
        let total_weight: Decimal = result.allocations.values().sum();
        if total_weight > Decimal::ZERO {
            for allocation in result.allocations.values_mut() {
                *allocation /= total_weight;
            }
        }

        Ok(())
    }

    /// Calculate expected return for an asset
    fn calculate_expected_return(&self, market_data: &MarketData) -> Result<Decimal> {
        // For now, use a simple calculation based on price change
        if market_data.price_change_24h != Decimal::ZERO {
            // Annualize the 24h change (rough approximation)
            Ok(market_data.price_change_24h * rust_decimal_macros::dec!(365))
        } else {
            Ok(rust_decimal_macros::dec!(0.05)) // Default 5% return
        }
    }

    /// Calculate portfolio metrics
    fn calculate_portfolio_metrics(&self, result: &mut OptimizationResult, _strategy: &OptimizationStrategy) -> Result<()> {
        let mut portfolio_return = Decimal::ZERO;
        let mut portfolio_risk = Decimal::ZERO;

        for (asset, weight) in &result.allocations {
            if let Some(market_data) = self.market_data.get(asset) {
                let asset_return = self.calculate_expected_return(market_data)?;
                let asset_risk = market_data.calculate_volatility().unwrap_or(rust_decimal_macros::dec!(0.1));
                
                portfolio_return += asset_return * weight;
                portfolio_risk += asset_risk * weight; // Simplified risk calculation
            }
        }

        result.expected_return = portfolio_return;
        result.expected_risk = portfolio_risk;
        result.calculate_sharpe_ratio(self.risk_free_rate);

        Ok(())
    }

    /// Check if constraints are satisfied
    fn check_constraints(&self, result: &OptimizationResult, strategy: &OptimizationStrategy) -> Result<bool> {
        for constraint in &strategy.constraints {
            match &constraint.constraint_type {
                ConstraintType::MaxRisk => {
                    if result.expected_risk > constraint.target_value + constraint.tolerance {
                        return Ok(false);
                    }
                }
                ConstraintType::MinReturn => {
                    if result.expected_return < constraint.target_value - constraint.tolerance {
                        return Ok(false);
                    }
                }
                ConstraintType::MaxAllocation(asset) => {
                    if let Some(allocation) = result.allocations.get(asset) {
                        if *allocation > constraint.target_value + constraint.tolerance {
                            return Ok(false);
                        }
                    }
                }
                _ => {
                    // Other constraint types not implemented yet
                    debug!("Constraint type {:?} not implemented", constraint.constraint_type);
                }
            }
        }
        Ok(true)
    }

    /// Calculate optimization score
    fn calculate_optimization_score(&self, result: &OptimizationResult, strategy: &OptimizationStrategy) -> Decimal {
        let mut score = rust_decimal_macros::dec!(0.0);

        // Base score from objective achievement
        match strategy.objective {
            OptimizationObjective::MaximizeReturn => {
                score += result.expected_return * rust_decimal_macros::dec!(100.0);
            }
            OptimizationObjective::MinimizeRisk => {
                score += (rust_decimal_macros::dec!(1.0) - result.expected_risk) * rust_decimal_macros::dec!(100.0);
            }
            OptimizationObjective::MaximizeSharpeRatio => {
                if let Some(sharpe) = result.sharpe_ratio {
                    score += sharpe * rust_decimal_macros::dec!(50.0);
                }
            }
            _ => {
                score += rust_decimal_macros::dec!(50.0); // Default score
            }
        }

        // Penalty for constraint violations
        if !result.constraints_satisfied {
            score *= rust_decimal_macros::dec!(0.8);
        }

        score.max(Decimal::ZERO).min(rust_decimal_macros::dec!(100.0))
    }

    /// Generate optimization recommendations
    fn generate_recommendations(&self, result: &mut OptimizationResult, strategy: &OptimizationStrategy) {
        if !result.constraints_satisfied {
            result.add_recommendation("Consider adjusting constraints or strategy parameters".to_string());
        }

        if result.expected_risk > strategy.risk_tolerance.to_score() {
            result.add_recommendation("Portfolio risk exceeds risk tolerance - consider reducing high-risk allocations".to_string());
        }

        if let Some(sharpe_ratio) = result.sharpe_ratio {
            if sharpe_ratio < rust_decimal_macros::dec!(0.5) {
                result.add_recommendation("Low Sharpe ratio - consider alternative assets or strategy".to_string());
            }
        }

        // Check for concentration risk
        let max_allocation = result.allocations.values().max().copied().unwrap_or(Decimal::ZERO);
        if max_allocation > rust_decimal_macros::dec!(0.5) {
            result.add_recommendation("High concentration risk - consider diversifying allocations".to_string());
        }
    }

    /// Get optimization history
    pub fn get_optimization_history(&self) -> &[OptimizationResult] {
        &self.optimization_history
    }

    /// Get best optimization result by score
    pub fn get_best_optimization(&self) -> Option<&OptimizationResult> {
        self.optimization_history.iter()
            .max_by(|a, b| a.optimization_score.partial_cmp(&b.optimization_score).unwrap_or(std::cmp::Ordering::Equal))
    }
}

impl Default for EconomicOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::market::{MarketData, PricePoint};
    use rust_decimal_macros::dec;

    #[test]
    fn test_optimization_strategy() {
        let mut strategy = OptimizationStrategy::new(
            "Test Strategy".to_string(),
            OptimizationObjective::MaximizeReturn,
        );

        strategy.add_constraint(OptimizationConstraint::max_risk(dec!(0.3)));
        strategy.set_parameter("min_allocation".to_string(), dec!(0.05));

        assert_eq!(strategy.constraints.len(), 1);
        assert_eq!(strategy.get_parameter("min_allocation"), Some(dec!(0.05)));
    }

    #[test]
    fn test_optimization_result() {
        let mut result = OptimizationResult::new(
            "Test".to_string(),
            OptimizationObjective::MaximizeReturn,
        );

        result.add_allocation("BTC".to_string(), dec!(0.6));
        result.add_allocation("ETH".to_string(), dec!(0.4));
        result.expected_return = dec!(0.15);
        result.expected_risk = dec!(0.2);
        result.calculate_sharpe_ratio(dec!(0.02));

        assert_eq!(result.allocations.len(), 2);
        assert!(result.sharpe_ratio.is_some());
    }

    #[test]
    fn test_economic_optimizer() {
        let mut optimizer = EconomicOptimizer::new();
        
        // Add sample market data
        let mut btc_data = MarketData::new("BTC".to_string());
        btc_data.add_price_point(PricePoint::new(dec!(50000.0), dec!(100.0)));
        btc_data.add_price_point(PricePoint::new(dec!(52000.0), dec!(110.0)));
        
        optimizer.add_market_data("BTC".to_string(), btc_data);

        let strategy = OptimizationStrategy::new(
            "Simple Strategy".to_string(),
            OptimizationObjective::MaximizeReturn,
        );

        let assets = vec!["BTC".to_string()];
        let result = optimizer.optimize_portfolio(&strategy, &assets).unwrap();

        assert!(!result.allocations.is_empty());
        assert!(result.expected_return >= Decimal::ZERO);
    }

    #[test]
    fn test_constraint_checking() {
        let optimizer = EconomicOptimizer::new();
        let mut strategy = OptimizationStrategy::new(
            "Constrained Strategy".to_string(),
            OptimizationObjective::MaximizeReturn,
        );

        strategy.add_constraint(OptimizationConstraint::max_risk(dec!(0.1)));

        let mut result = OptimizationResult::new(
            "Test".to_string(),
            OptimizationObjective::MaximizeReturn,
        );
        result.expected_risk = dec!(0.2); // Exceeds constraint

        let satisfied = optimizer.check_constraints(&result, &strategy).unwrap();
        assert!(!satisfied);
    }
}