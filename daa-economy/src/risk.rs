//! Risk assessment and management

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};

use crate::error::{EconomyError, Result};

/// Risk level enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Hash)]
pub enum RiskLevel {
    VeryLow,
    Low,
    Medium,
    High,
    VeryHigh,
    Critical,
}

impl RiskLevel {
    /// Convert risk level to numeric score (0.0 to 1.0)
    pub fn to_score(&self) -> Decimal {
        match self {
            RiskLevel::VeryLow => rust_decimal_macros::dec!(0.1),
            RiskLevel::Low => rust_decimal_macros::dec!(0.2),
            RiskLevel::Medium => rust_decimal_macros::dec!(0.4),
            RiskLevel::High => rust_decimal_macros::dec!(0.7),
            RiskLevel::VeryHigh => rust_decimal_macros::dec!(0.9),
            RiskLevel::Critical => rust_decimal_macros::dec!(1.0),
        }
    }

    /// Create risk level from numeric score
    pub fn from_score(score: Decimal) -> Self {
        if score <= rust_decimal_macros::dec!(0.15) {
            RiskLevel::VeryLow
        } else if score <= rust_decimal_macros::dec!(0.3) {
            RiskLevel::Low
        } else if score <= rust_decimal_macros::dec!(0.5) {
            RiskLevel::Medium
        } else if score <= rust_decimal_macros::dec!(0.8) {
            RiskLevel::High
        } else if score <= rust_decimal_macros::dec!(0.95) {
            RiskLevel::VeryHigh
        } else {
            RiskLevel::Critical
        }
    }
}

impl std::fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RiskLevel::VeryLow => write!(f, "Very Low"),
            RiskLevel::Low => write!(f, "Low"),
            RiskLevel::Medium => write!(f, "Medium"),
            RiskLevel::High => write!(f, "High"),
            RiskLevel::VeryHigh => write!(f, "Very High"),
            RiskLevel::Critical => write!(f, "Critical"),
        }
    }
}

/// Individual risk factor
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RiskFactor {
    pub name: String,
    pub description: String,
    pub weight: Decimal,
    pub score: Decimal,
    pub category: RiskCategory,
    pub last_updated: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl RiskFactor {
    pub fn new(name: String, description: String, weight: Decimal, score: Decimal, category: RiskCategory) -> Self {
        Self {
            name,
            description,
            weight,
            score,
            category,
            last_updated: Utc::now(),
            metadata: HashMap::new(),
        }
    }

    /// Calculate weighted contribution to overall risk
    pub fn weighted_contribution(&self) -> Decimal {
        self.weight * self.score
    }

    /// Update the risk score
    pub fn update_score(&mut self, new_score: Decimal) {
        self.score = new_score.min(rust_decimal_macros::dec!(1.0)).max(rust_decimal_macros::dec!(0.0));
        self.last_updated = Utc::now();
    }

    /// Set metadata
    pub fn set_metadata(&mut self, key: String, value: serde_json::Value) {
        self.metadata.insert(key, value);
        self.last_updated = Utc::now();
    }
}

/// Risk categories
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum RiskCategory {
    Market,
    Liquidity,
    Credit,
    Operational,
    Technical,
    Regulatory,
    Counterparty,
    SystemRisk,
}

impl std::fmt::Display for RiskCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RiskCategory::Market => write!(f, "Market"),
            RiskCategory::Liquidity => write!(f, "Liquidity"),
            RiskCategory::Credit => write!(f, "Credit"),
            RiskCategory::Operational => write!(f, "Operational"),
            RiskCategory::Technical => write!(f, "Technical"),
            RiskCategory::Regulatory => write!(f, "Regulatory"),
            RiskCategory::Counterparty => write!(f, "Counterparty"),
            RiskCategory::SystemRisk => write!(f, "System Risk"),
        }
    }
}

/// Complete risk assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub id: String,
    pub entity: String,
    pub risk_factors: HashMap<String, RiskFactor>,
    pub overall_score: Decimal,
    pub risk_level: RiskLevel,
    pub assessment_time: DateTime<Utc>,
    pub recommendations: Vec<String>,
    pub action_required: bool,
}

impl RiskAssessment {
    pub fn new(id: String, entity: String) -> Self {
        Self {
            id,
            entity,
            risk_factors: HashMap::new(),
            overall_score: Decimal::ZERO,
            risk_level: RiskLevel::VeryLow,
            assessment_time: Utc::now(),
            recommendations: Vec::new(),
            action_required: false,
        }
    }

    /// Add a risk factor
    pub fn add_risk_factor(&mut self, factor: RiskFactor) {
        self.risk_factors.insert(factor.name.clone(), factor);
        self.recalculate_risk();
    }

    /// Remove a risk factor
    pub fn remove_risk_factor(&mut self, name: &str) {
        self.risk_factors.remove(name);
        self.recalculate_risk();
    }

    /// Update a risk factor score
    pub fn update_risk_factor(&mut self, name: &str, new_score: Decimal) -> Result<()> {
        if let Some(factor) = self.risk_factors.get_mut(name) {
            factor.update_score(new_score);
            self.recalculate_risk();
            Ok(())
        } else {
            Err(EconomyError::RiskAssessmentError(format!("Risk factor '{}' not found", name)))
        }
    }

    /// Recalculate overall risk score
    fn recalculate_risk(&mut self) {
        let total_weighted_score: Decimal = self.risk_factors.values()
            .map(|factor| factor.weighted_contribution())
            .sum();

        let total_weight: Decimal = self.risk_factors.values()
            .map(|factor| factor.weight)
            .sum();

        self.overall_score = if total_weight > Decimal::ZERO {
            total_weighted_score / total_weight
        } else {
            Decimal::ZERO
        };

        self.risk_level = RiskLevel::from_score(self.overall_score);
        self.assessment_time = Utc::now();
        
        // Determine if action is required
        self.action_required = matches!(self.risk_level, RiskLevel::High | RiskLevel::VeryHigh | RiskLevel::Critical);

        debug!("Recalculated risk for {}: {} ({})", self.entity, self.overall_score, self.risk_level);
    }

    /// Get risk factors by category
    pub fn get_factors_by_category(&self, category: RiskCategory) -> Vec<&RiskFactor> {
        self.risk_factors.values()
            .filter(|factor| factor.category == category)
            .collect()
    }

    /// Get highest risk factors
    pub fn get_top_risk_factors(&self, limit: usize) -> Vec<&RiskFactor> {
        let mut factors: Vec<&RiskFactor> = self.risk_factors.values().collect();
        factors.sort_by(|a, b| b.score.cmp(&a.score));
        factors.into_iter().take(limit).collect()
    }

    /// Add recommendation
    pub fn add_recommendation(&mut self, recommendation: String) {
        self.recommendations.push(recommendation);
    }

    /// Generate summary
    pub fn summary(&self) -> String {
        format!(
            "Risk Assessment for {}: {} ({:.2}) - {} factors, Action Required: {}",
            self.entity,
            self.risk_level,
            self.overall_score,
            self.risk_factors.len(),
            self.action_required
        )
    }
}

/// Risk assessment engine
pub struct RiskAssessmentEngine {
    assessments: HashMap<String, RiskAssessment>,
    default_factors: HashMap<RiskCategory, Vec<RiskFactor>>,
    assessment_interval_hours: u64,
}

impl RiskAssessmentEngine {
    pub fn new() -> Self {
        let mut engine = Self {
            assessments: HashMap::new(),
            default_factors: HashMap::new(),
            assessment_interval_hours: 24, // Daily assessments by default
        };
        
        engine.initialize_default_factors();
        engine
    }

    /// Initialize default risk factors
    fn initialize_default_factors(&mut self) {
        // Market risk factors
        let market_factors = vec![
            RiskFactor::new(
                "price_volatility".to_string(),
                "Price volatility over last 30 days".to_string(),
                rust_decimal_macros::dec!(0.3),
                rust_decimal_macros::dec!(0.0),
                RiskCategory::Market,
            ),
            RiskFactor::new(
                "correlation_risk".to_string(),
                "Correlation with major market indices".to_string(),
                rust_decimal_macros::dec!(0.2),
                rust_decimal_macros::dec!(0.0),
                RiskCategory::Market,
            ),
        ];

        // Liquidity risk factors
        let liquidity_factors = vec![
            RiskFactor::new(
                "bid_ask_spread".to_string(),
                "Average bid-ask spread".to_string(),
                rust_decimal_macros::dec!(0.4),
                rust_decimal_macros::dec!(0.0),
                RiskCategory::Liquidity,
            ),
            RiskFactor::new(
                "trading_volume".to_string(),
                "24h trading volume".to_string(),
                rust_decimal_macros::dec!(0.3),
                rust_decimal_macros::dec!(0.0),
                RiskCategory::Liquidity,
            ),
        ];

        // Technical risk factors
        let technical_factors = vec![
            RiskFactor::new(
                "system_uptime".to_string(),
                "System availability and uptime".to_string(),
                rust_decimal_macros::dec!(0.5),
                rust_decimal_macros::dec!(0.0),
                RiskCategory::Technical,
            ),
            RiskFactor::new(
                "security_incidents".to_string(),
                "Recent security incidents".to_string(),
                rust_decimal_macros::dec!(0.8),
                rust_decimal_macros::dec!(0.0),
                RiskCategory::Technical,
            ),
        ];

        self.default_factors.insert(RiskCategory::Market, market_factors);
        self.default_factors.insert(RiskCategory::Liquidity, liquidity_factors);
        self.default_factors.insert(RiskCategory::Technical, technical_factors);
    }

    /// Create new risk assessment
    pub fn create_assessment(&mut self, entity: String) -> Result<String> {
        let assessment_id = format!("risk_{}_{}", entity, Utc::now().timestamp());
        let mut assessment = RiskAssessment::new(assessment_id.clone(), entity.clone());

        // Add default risk factors
        for factors in self.default_factors.values() {
            for factor in factors {
                assessment.add_risk_factor(factor.clone());
            }
        }

        self.assessments.insert(assessment_id.clone(), assessment);
        info!("Created risk assessment: {}", assessment_id);
        
        Ok(assessment_id)
    }

    /// Get risk assessment
    pub fn get_assessment(&self, assessment_id: &str) -> Option<&RiskAssessment> {
        self.assessments.get(assessment_id)
    }

    /// Get mutable risk assessment
    pub fn get_assessment_mut(&mut self, assessment_id: &str) -> Option<&mut RiskAssessment> {
        self.assessments.get_mut(assessment_id)
    }

    /// Update risk factor across all assessments
    pub fn update_global_risk_factor(&mut self, factor_name: &str, score_updater: impl Fn(&str) -> Decimal) -> Result<usize> {
        let mut updated_count = 0;

        for assessment in self.assessments.values_mut() {
            if assessment.risk_factors.contains_key(factor_name) {
                let new_score = score_updater(&assessment.entity);
                assessment.update_risk_factor(factor_name, new_score)?;
                updated_count += 1;
            }
        }

        if updated_count > 0 {
            info!("Updated risk factor '{}' across {} assessments", factor_name, updated_count);
        }

        Ok(updated_count)
    }

    /// Get high-risk entities
    pub fn get_high_risk_entities(&self) -> Vec<&RiskAssessment> {
        self.assessments.values()
            .filter(|assessment| assessment.risk_level >= RiskLevel::High)
            .collect()
    }

    /// Get entities requiring action
    pub fn get_entities_requiring_action(&self) -> Vec<&RiskAssessment> {
        self.assessments.values()
            .filter(|assessment| assessment.action_required)
            .collect()
    }

    /// Calculate portfolio risk
    pub fn calculate_portfolio_risk(&self, entity_weights: &HashMap<String, Decimal>) -> Result<RiskAssessment> {
        let mut portfolio_assessment = RiskAssessment::new(
            format!("portfolio_{}", Utc::now().timestamp()),
            "Portfolio".to_string(),
        );

        let mut category_scores: HashMap<RiskCategory, (Decimal, Decimal)> = HashMap::new();

        for (entity, weight) in entity_weights {
            // Find assessment for this entity
            if let Some(assessment) = self.assessments.values().find(|a| &a.entity == entity) {
                for factor in assessment.risk_factors.values() {
                    let (total_score, total_weight) = category_scores
                        .entry(factor.category.clone())
                        .or_insert((Decimal::ZERO, Decimal::ZERO));
                    
                    *total_score += factor.score * weight;
                    *total_weight += weight;
                }
            }
        }

        // Create portfolio risk factors
        for (category, (total_score, total_weight)) in category_scores {
            if total_weight > Decimal::ZERO {
                let avg_score = total_score / total_weight;
                let portfolio_factor = RiskFactor::new(
                    format!("portfolio_{}", category).to_lowercase(),
                    format!("Portfolio {} risk", category),
                    rust_decimal_macros::dec!(1.0), // Equal weight for portfolio factors
                    avg_score,
                    category,
                );
                portfolio_assessment.add_risk_factor(portfolio_factor);
            }
        }

        Ok(portfolio_assessment)
    }

    /// Cleanup old assessments
    pub fn cleanup_old_assessments(&mut self, max_age_days: i64) -> Result<usize> {
        let cutoff_time = Utc::now() - chrono::Duration::days(max_age_days);
        let mut removed_count = 0;

        let old_ids: Vec<String> = self.assessments.iter()
            .filter_map(|(id, assessment)| {
                if assessment.assessment_time < cutoff_time {
                    Some(id.clone())
                } else {
                    None
                }
            })
            .collect();

        for id in old_ids {
            self.assessments.remove(&id);
            removed_count += 1;
        }

        if removed_count > 0 {
            info!("Cleaned up {} old risk assessments", removed_count);
        }

        Ok(removed_count)
    }

    /// Get assessment statistics
    pub fn get_statistics(&self) -> RiskStatistics {
        let total_assessments = self.assessments.len();
        let mut level_counts = HashMap::new();
        let mut action_required_count = 0;

        for assessment in self.assessments.values() {
            *level_counts.entry(assessment.risk_level.clone()).or_insert(0) += 1;
            if assessment.action_required {
                action_required_count += 1;
            }
        }

        RiskStatistics {
            total_assessments,
            level_distribution: level_counts,
            action_required_count,
        }
    }
}

impl Default for RiskAssessmentEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Risk statistics summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskStatistics {
    pub total_assessments: usize,
    pub level_distribution: HashMap<RiskLevel, usize>,
    pub action_required_count: usize,
}

impl RiskStatistics {
    pub fn summary(&self) -> String {
        format!(
            "Risk Statistics: {} total assessments, {} require action",
            self.total_assessments, self.action_required_count
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_risk_level_conversion() {
        assert_eq!(RiskLevel::from_score(dec!(0.1)), RiskLevel::VeryLow);
        assert_eq!(RiskLevel::from_score(dec!(0.5)), RiskLevel::Medium);
        assert_eq!(RiskLevel::from_score(dec!(0.9)), RiskLevel::VeryHigh);
        assert_eq!(RiskLevel::from_score(dec!(1.0)), RiskLevel::Critical);
    }

    #[test]
    fn test_risk_factor() {
        let mut factor = RiskFactor::new(
            "test_factor".to_string(),
            "Test risk factor".to_string(),
            dec!(0.5),
            dec!(0.3),
            RiskCategory::Market,
        );

        assert_eq!(factor.weighted_contribution(), dec!(0.15));

        factor.update_score(dec!(0.8));
        assert_eq!(factor.score, dec!(0.8));
    }

    #[test]
    fn test_risk_assessment() {
        let mut assessment = RiskAssessment::new(
            "test_1".to_string(),
            "test_entity".to_string(),
        );

        let factor = RiskFactor::new(
            "market_risk".to_string(),
            "Market volatility".to_string(),
            dec!(1.0),
            dec!(0.6),
            RiskCategory::Market,
        );

        assessment.add_risk_factor(factor);
        assert_eq!(assessment.overall_score, dec!(0.6));
        assert_eq!(assessment.risk_level, RiskLevel::High);
        assert!(assessment.action_required);
    }

    #[test]
    fn test_risk_assessment_engine() {
        let mut engine = RiskAssessmentEngine::new();
        let assessment_id = engine.create_assessment("test_entity".to_string()).unwrap();
        
        let assessment = engine.get_assessment(&assessment_id).unwrap();
        assert_eq!(assessment.entity, "test_entity");
        assert!(!assessment.risk_factors.is_empty());
    }

    #[test]
    fn test_portfolio_risk_calculation() {
        let mut engine = RiskAssessmentEngine::new();
        
        // Create assessments for multiple entities
        let id1 = engine.create_assessment("entity1".to_string()).unwrap();
        let id2 = engine.create_assessment("entity2".to_string()).unwrap();

        // Update some risk scores
        engine.get_assessment_mut(&id1).unwrap()
            .update_risk_factor("price_volatility", dec!(0.8)).unwrap();
        engine.get_assessment_mut(&id2).unwrap()
            .update_risk_factor("price_volatility", dec!(0.4)).unwrap();

        // Calculate portfolio risk
        let mut weights = HashMap::new();
        weights.insert("entity1".to_string(), dec!(0.6));
        weights.insert("entity2".to_string(), dec!(0.4));

        let portfolio_risk = engine.calculate_portfolio_risk(&weights).unwrap();
        assert!(!portfolio_risk.risk_factors.is_empty());
    }
}