//! Configuration management for QuDAG Exchange
//!
//! Provides unified configuration management with support for
//! immutable deployment and dynamic fee model configuration.

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};

use crate::{
    fee_model::{AgentStatus, FeeModel, FeeModelParams},
    immutable::{ImmutableDeployment, ImmutableStatus, LockableConfig},
    payout::{FeeRouter, PayoutConfig},
    types::{rUv, Timestamp},
    Error, Result,
};
use serde::{Deserialize, Serialize};

/// Main configuration for QuDAG Exchange
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeConfig {
    /// Immutable deployment manager
    pub immutable_deployment: ImmutableDeployment,

    /// Dynamic fee model calculator
    #[serde(skip)]
    pub fee_model: Option<FeeModel>,

    /// Network configuration
    pub network: NetworkConfig,

    /// Security configuration
    pub security: SecurityConfig,

    /// Business plan features configuration (optional)
    pub business_plan: Option<BusinessPlanConfig>,

    /// Fee router for automatic payouts (not serialized, recreated from config)
    #[serde(skip)]
    pub fee_router: Option<FeeRouter>,
}

/// Network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Chain ID for the network
    pub chain_id: u64,

    /// Network name
    pub network_name: String,

    /// Bootstrap peers for networking
    pub bootstrap_peers: Vec<String>,

    /// Listen address for P2P networking
    pub listen_address: String,

    /// Enable dark addressing features
    pub enable_dark_addressing: bool,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            chain_id: 1,
            network_name: "qudag-exchange".to_string(),
            bootstrap_peers: Vec::new(),
            listen_address: "0.0.0.0:8080".to_string(),
            enable_dark_addressing: true,
        }
    }
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Require quantum-resistant signatures for all transactions
    pub require_quantum_signatures: bool,

    /// Minimum signature algorithm strength (e.g., "ML-DSA-87")
    pub min_signature_strength: String,

    /// Enable transaction replay protection
    pub enable_replay_protection: bool,

    /// Transaction expiry time in seconds
    pub default_tx_expiry_seconds: u64,

    /// Enable rate limiting
    pub enable_rate_limiting: bool,

    /// Maximum transactions per account per minute
    pub max_tx_per_minute: u32,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            require_quantum_signatures: true,
            min_signature_strength: "ML-DSA-87".to_string(),
            enable_replay_protection: true,
            default_tx_expiry_seconds: 300, // 5 minutes
            enable_rate_limiting: true,
            max_tx_per_minute: 10,
        }
    }
}

impl ExchangeConfig {
    /// Create a new exchange configuration with defaults
    pub fn new() -> Result<Self> {
        let mut config = Self {
            immutable_deployment: ImmutableDeployment::new(),
            fee_model: None,
            network: NetworkConfig::default(),
            security: SecurityConfig::default(),
            business_plan: None, // Disabled by default
            fee_router: None,
        };

        // Initialize fee model from immutable deployment params
        config.initialize_fee_model()?;
        Ok(config)
    }

    /// Create configuration from a lockable config
    pub fn from_lockable_config(lockable: LockableConfig) -> Result<Self> {
        let mut config = Self {
            immutable_deployment: ImmutableDeployment::with_config(lockable.clone())?,
            fee_model: None,
            network: NetworkConfig {
                chain_id: lockable.chain_id,
                ..NetworkConfig::default()
            },
            security: SecurityConfig::default(),
            business_plan: None, // Disabled by default
            fee_router: None,
        };

        config.initialize_fee_model()?;
        Ok(config)
    }

    /// Initialize the fee model from immutable deployment parameters
    fn initialize_fee_model(&mut self) -> Result<()> {
        let fee_params = self.immutable_deployment.system_config.fee_params.clone();
        self.fee_model = Some(FeeModel::with_params(fee_params)?);
        Ok(())
    }

    /// Update fee model parameters (respects immutable restrictions)
    pub fn update_fee_params(
        &mut self,
        params: FeeModelParams,
        current_time: Timestamp,
    ) -> Result<()> {
        // Update immutable deployment first (this enforces restrictions)
        self.immutable_deployment
            .update_fee_params(params.clone(), current_time)?;

        // Update fee model
        if let Some(ref mut fee_model) = self.fee_model {
            fee_model.update_params(params)?;
        } else {
            self.fee_model = Some(FeeModel::with_params(params)?);
        }

        Ok(())
    }

    /// Calculate fee for a transaction
    pub fn calculate_transaction_fee(
        &self,
        transaction_amount: rUv,
        agent_status: &AgentStatus,
        current_time: Timestamp,
    ) -> Result<rUv> {
        let fee_model = self
            .fee_model
            .as_ref()
            .ok_or_else(|| Error::Other("Fee model not initialized".into()))?;

        fee_model.calculate_fee_amount(transaction_amount, agent_status, current_time)
    }

    /// Get fee rate for an agent
    pub fn get_fee_rate(&self, agent_status: &AgentStatus, current_time: Timestamp) -> Result<f64> {
        let fee_model = self
            .fee_model
            .as_ref()
            .ok_or_else(|| Error::Other("Fee model not initialized".into()))?;

        fee_model.calculate_fee_rate(agent_status, current_time)
    }

    /// Enable immutable deployment mode
    pub fn enable_immutable_mode(&mut self) -> Result<()> {
        self.immutable_deployment.enable_immutable_mode()
    }

    /// Lock the system configuration (immutable deployment)
    #[cfg(feature = "std")]
    pub fn lock_system(
        &mut self,
        keypair: &qudag_crypto::MlDsaKeyPair,
        current_time: Timestamp,
    ) -> Result<()> {
        self.immutable_deployment.lock_system(keypair, current_time)
    }

    /// Check if configuration can be modified
    pub fn can_modify_config(&self, current_time: Timestamp) -> bool {
        self.immutable_deployment.can_modify_config(current_time)
    }

    /// Get immutable deployment status
    pub fn get_immutable_status(&self, current_time: Timestamp) -> ImmutableStatus {
        self.immutable_deployment.get_status(current_time)
    }

    /// Update network configuration (respects immutable restrictions)
    pub fn update_network_config(
        &mut self,
        network: NetworkConfig,
        current_time: Timestamp,
    ) -> Result<()> {
        if !self.can_modify_config(current_time) {
            return Err(Error::Other(
                "Cannot modify network configuration: system is immutably locked".into(),
            ));
        }

        // Update chain ID in lockable config too
        self.immutable_deployment.system_config.chain_id = network.chain_id;
        self.network = network;
        Ok(())
    }

    /// Update security configuration (respects immutable restrictions)
    pub fn update_security_config(
        &mut self,
        security: SecurityConfig,
        current_time: Timestamp,
    ) -> Result<()> {
        if !self.can_modify_config(current_time) {
            return Err(Error::Other(
                "Cannot modify security configuration: system is immutably locked".into(),
            ));
        }

        self.security = security;
        Ok(())
    }

    /// Validate the entire configuration
    pub fn validate(&self) -> Result<()> {
        // Validate immutable deployment config
        self.immutable_deployment.system_config.validate()?;

        // Validate network config
        if self.network.chain_id == 0 {
            return Err(Error::Other("chain_id must be greater than 0".into()));
        }

        if self.network.network_name.is_empty() {
            return Err(Error::Other("network_name cannot be empty".into()));
        }

        // Validate security config
        if self.security.default_tx_expiry_seconds == 0 {
            return Err(Error::Other(
                "default_tx_expiry_seconds must be greater than 0".into(),
            ));
        }

        if self.security.max_tx_per_minute == 0 {
            return Err(Error::Other(
                "max_tx_per_minute must be greater than 0".into(),
            ));
        }

        Ok(())
    }

    /// Get configuration summary for display
    pub fn get_summary(&self, current_time: Timestamp) -> ConfigSummary {
        let immutable_status = self.get_immutable_status(current_time);
        let fee_params = &self.immutable_deployment.system_config.fee_params;

        let business_plan_summary = self.business_plan.as_ref().map(|bp| {
            let total_contributors = self
                .fee_router
                .as_ref()
                .map(|router| router.get_payout_history(None).len() as u32)
                .unwrap_or(0);

            BusinessPlanSummary {
                enabled: bp.enabled,
                auto_distribution_enabled: bp.enable_auto_distribution,
                vault_management_enabled: bp.enable_vault_management,
                role_earnings_enabled: bp.enable_role_earnings,
                bounty_rewards_enabled: bp.enable_bounty_rewards,
                total_contributors,
                min_payout_threshold: bp.payout_config.min_payout_threshold.amount(),
                system_fee_percentage: bp.payout_config.system_fee_percentage,
            }
        });

        ConfigSummary {
            network_name: self.network.network_name.clone(),
            chain_id: self.network.chain_id,
            immutable_status,
            fee_model_summary: FeeModelSummary {
                f_min: fee_params.f_min,
                f_max: fee_params.f_max,
                f_min_verified: fee_params.f_min_verified,
                f_max_verified: fee_params.f_max_verified,
                time_constant_days: fee_params.time_constant_seconds / (24 * 60 * 60),
                usage_threshold: fee_params.usage_threshold_ruv,
            },
            security_enabled: self.security.require_quantum_signatures,
            dark_addressing_enabled: self.network.enable_dark_addressing,
            business_plan_summary,
        }
    }

    /// Emergency governance override (unlock immutable system)
    #[cfg(feature = "std")]
    pub fn governance_override(
        &mut self,
        governance_keypair: &qudag_crypto::MlDsaKeyPair,
        current_time: Timestamp,
    ) -> Result<()> {
        self.immutable_deployment
            .governance_override(governance_keypair, current_time)
    }

    /// Enable business plan features
    pub fn enable_business_plan(&mut self, config: BusinessPlanConfig) -> Result<()> {
        // Validate business plan configuration
        config.payout_config.validate()?;

        // Initialize fee router if auto-distribution is enabled
        if config.enable_auto_distribution {
            self.fee_router = Some(FeeRouter::new(config.payout_config.clone()));
        }

        self.business_plan = Some(config);
        Ok(())
    }

    /// Disable business plan features
    pub fn disable_business_plan(&mut self) {
        self.business_plan = None;
        self.fee_router = None;
    }

    /// Check if business plan features are enabled
    pub fn has_business_plan(&self) -> bool {
        self.business_plan.as_ref().map_or(false, |bp| bp.enabled)
    }

    /// Get fee router if available
    pub fn fee_router(&self) -> Option<&FeeRouter> {
        self.fee_router.as_ref()
    }

    /// Get mutable fee router if available
    pub fn fee_router_mut(&mut self) -> Option<&mut FeeRouter> {
        self.fee_router.as_mut()
    }

    /// Update business plan configuration
    pub fn update_business_plan(
        &mut self,
        config: BusinessPlanConfig,
        current_time: Timestamp,
    ) -> Result<()> {
        if !self.can_modify_config(current_time) {
            return Err(Error::Other(
                "Cannot modify business plan configuration: system is immutably locked".into(),
            ));
        }

        // Validate new configuration
        config.payout_config.validate()?;

        // Update fee router if needed
        if config.enable_auto_distribution {
            if let Some(ref mut fee_router) = self.fee_router {
                fee_router.update_config(config.payout_config.clone())?;
            } else {
                self.fee_router = Some(FeeRouter::new(config.payout_config.clone()));
            }
        } else {
            self.fee_router = None;
        }

        self.business_plan = Some(config);
        Ok(())
    }

    /// Save configuration to bytes for persistence
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        bincode::serialize(self).map_err(|e| Error::SerializationError(e.to_string()))
    }

    /// Load configuration from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let mut config: Self =
            bincode::deserialize(bytes).map_err(|e| Error::SerializationError(e.to_string()))?;

        // Re-initialize fee model since it's not serialized
        config.initialize_fee_model()?;

        // Re-initialize fee router if business plan is enabled
        if let Some(ref business_plan) = config.business_plan {
            if business_plan.enabled && business_plan.enable_auto_distribution {
                config.fee_router = Some(FeeRouter::new(business_plan.payout_config.clone()));
            }
        }

        config.validate()?;
        Ok(config)
    }
}

impl Default for ExchangeConfig {
    fn default() -> Self {
        Self::new().expect("Default configuration should be valid")
    }
}

/// Business plan features configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessPlanConfig {
    /// Enable business plan features
    pub enabled: bool,

    /// Payout system configuration
    pub payout_config: PayoutConfig,

    /// Enable contributor vault management
    pub enable_vault_management: bool,

    /// Enable automatic fee distribution
    pub enable_auto_distribution: bool,

    /// Enable role-based earnings tracking
    pub enable_role_earnings: bool,

    /// Enable bounty agent rewards
    pub enable_bounty_rewards: bool,

    /// Governance settings
    pub governance: GovernanceConfig,
}

impl Default for BusinessPlanConfig {
    fn default() -> Self {
        Self {
            enabled: false, // Opt-in by default
            payout_config: PayoutConfig::default(),
            enable_vault_management: false,
            enable_auto_distribution: false,
            enable_role_earnings: false,
            enable_bounty_rewards: false,
            governance: GovernanceConfig::default(),
        }
    }
}

/// Governance configuration for business plan features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceConfig {
    /// Allow contributors to override default payout percentages
    pub allow_custom_percentages: bool,

    /// Require governance approval for large payouts
    pub require_approval_threshold: Option<rUv>,

    /// Enable democratic voting on payout changes
    pub enable_voting: bool,

    /// Minimum voting period in seconds
    pub min_voting_period_seconds: u64,
}

impl Default for GovernanceConfig {
    fn default() -> Self {
        Self {
            allow_custom_percentages: true,
            require_approval_threshold: Some(rUv::new(10000)), // 10k rUv
            enable_voting: false,
            min_voting_period_seconds: 7 * 24 * 60 * 60, // 7 days
        }
    }
}

/// Summary of configuration for display purposes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSummary {
    pub network_name: String,
    pub chain_id: u64,
    pub immutable_status: ImmutableStatus,
    pub fee_model_summary: FeeModelSummary,
    pub security_enabled: bool,
    pub dark_addressing_enabled: bool,
    pub business_plan_summary: Option<BusinessPlanSummary>,
}

/// Business plan features summary for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessPlanSummary {
    pub enabled: bool,
    pub auto_distribution_enabled: bool,
    pub vault_management_enabled: bool,
    pub role_earnings_enabled: bool,
    pub bounty_rewards_enabled: bool,
    pub total_contributors: u32,
    pub min_payout_threshold: u64,
    pub system_fee_percentage: f64,
}

/// Fee model summary for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeModelSummary {
    pub f_min: f64,
    pub f_max: f64,
    pub f_min_verified: f64,
    pub f_max_verified: f64,
    pub time_constant_days: u64,
    pub usage_threshold: u64,
}

/// Configuration builder for easier setup
pub struct ExchangeConfigBuilder {
    network: NetworkConfig,
    security: SecurityConfig,
    fee_params: FeeModelParams,
    enable_immutable: bool,
    business_plan: Option<BusinessPlanConfig>,
}

impl ExchangeConfigBuilder {
    /// Create a new configuration builder
    pub fn new() -> Self {
        Self {
            network: NetworkConfig::default(),
            security: SecurityConfig::default(),
            fee_params: FeeModelParams::default(),
            enable_immutable: false,
            business_plan: None,
        }
    }

    /// Set network configuration
    pub fn with_network(mut self, network: NetworkConfig) -> Self {
        self.network = network;
        self
    }

    /// Set security configuration
    pub fn with_security(mut self, security: SecurityConfig) -> Self {
        self.security = security;
        self
    }

    /// Set fee model parameters
    pub fn with_fee_params(mut self, fee_params: FeeModelParams) -> Self {
        self.fee_params = fee_params;
        self
    }

    /// Enable immutable deployment mode
    pub fn with_immutable_mode(mut self) -> Self {
        self.enable_immutable = true;
        self
    }

    /// Set chain ID
    pub fn with_chain_id(mut self, chain_id: u64) -> Self {
        self.network.chain_id = chain_id;
        self
    }

    /// Set network name
    pub fn with_network_name(mut self, name: impl Into<String>) -> Self {
        self.network.network_name = name.into();
        self
    }

    /// Enable business plan features
    pub fn with_business_plan(mut self, business_plan: BusinessPlanConfig) -> Self {
        self.business_plan = Some(business_plan);
        self
    }

    /// Enable basic business plan features with default configuration
    pub fn with_basic_business_plan(mut self) -> Self {
        let mut bp_config = BusinessPlanConfig::default();
        bp_config.enabled = true;
        bp_config.enable_auto_distribution = true;
        bp_config.enable_role_earnings = true;
        self.business_plan = Some(bp_config);
        self
    }

    /// Build the configuration
    pub fn build(self) -> Result<ExchangeConfig> {
        let lockable_config = LockableConfig {
            fee_params: self.fee_params,
            chain_id: self.network.chain_id,
            ..LockableConfig::default()
        };

        let mut config = ExchangeConfig::from_lockable_config(lockable_config)?;
        config.network = self.network;
        config.security = self.security;

        if self.enable_immutable {
            config.enable_immutable_mode()?;
        }

        // Enable business plan if configured
        if let Some(business_plan) = self.business_plan {
            config.enable_business_plan(business_plan)?;
        }

        config.validate()?;
        Ok(config)
    }
}

impl Default for ExchangeConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_creation() {
        let config = ExchangeConfig::new().unwrap();
        assert!(config.fee_model.is_some());
        assert_eq!(config.network.chain_id, 1);
        assert_eq!(config.network.network_name, "qudag-exchange");
        config.validate().unwrap();
    }

    #[test]
    fn test_config_builder() {
        let config = ExchangeConfigBuilder::new()
            .with_chain_id(42)
            .with_network_name("test-network")
            .with_immutable_mode()
            .build()
            .unwrap();

        assert_eq!(config.network.chain_id, 42);
        assert_eq!(config.network.network_name, "test-network");
        assert!(config.immutable_deployment.config.enabled);
    }

    #[test]
    fn test_fee_calculation() {
        let config = ExchangeConfig::new().unwrap();
        let agent = AgentStatus::new_unverified(Timestamp::new(0));
        let current_time = Timestamp::new(1000);

        let fee = config
            .calculate_transaction_fee(rUv::new(1000), &agent, current_time)
            .unwrap();

        // Should be minimum fee for new unverified agent
        assert_eq!(fee.amount(), 1); // 1000 * 0.001 = 1

        let rate = config.get_fee_rate(&agent, current_time).unwrap();
        assert!((rate - 0.001).abs() < 1e-10);
    }

    #[test]
    fn test_config_modification_restrictions() {
        let mut config = ExchangeConfig::new().unwrap();
        config.enable_immutable_mode().unwrap();

        let current_time = Timestamp::new(1000);

        // Should be able to modify before locking
        assert!(config.can_modify_config(current_time));

        let new_params = FeeModelParams::default();
        config.update_fee_params(new_params, current_time).unwrap();

        // Simulate locked state
        config.immutable_deployment.config.locked_at = Some(current_time);
        config.immutable_deployment.config.lock_signature =
            Some(crate::immutable::ImmutableSignature {
                algorithm: "ML-DSA-87".to_string(),
                public_key: vec![1, 2, 3],
                signature: vec![4, 5, 6],
                config_hash: crate::types::Hash::from_bytes([0u8; 32]),
            });

        // Should not be able to modify after grace period
        let post_grace = Timestamp::new(current_time.value() + 25 * 60 * 60);
        assert!(!config.can_modify_config(post_grace));

        let result = config.update_fee_params(FeeModelParams::default(), post_grace);
        assert!(result.is_err());
    }

    #[test]
    fn test_config_summary() {
        let config = ExchangeConfig::new().unwrap();
        let current_time = Timestamp::new(1000);

        let summary = config.get_summary(current_time);
        assert_eq!(summary.network_name, "qudag-exchange");
        assert_eq!(summary.chain_id, 1);
        assert!(!summary.immutable_status.enabled);
        assert_eq!(summary.fee_model_summary.f_min, 0.001);
    }

    #[test]
    fn test_config_serialization() {
        let config = ExchangeConfig::new().unwrap();

        let bytes = config.to_bytes().unwrap();
        let restored = ExchangeConfig::from_bytes(&bytes).unwrap();

        // Fee model should be restored
        assert!(restored.fee_model.is_some());
        assert_eq!(config.network.chain_id, restored.network.chain_id);
        assert_eq!(config.network.network_name, restored.network.network_name);
    }

    #[test]
    fn test_network_config_validation() {
        let mut config = ExchangeConfig::new().unwrap();

        // Valid config should pass
        config.validate().unwrap();

        // Invalid chain ID should fail
        config.network.chain_id = 0;
        assert!(config.validate().is_err());

        // Empty network name should fail
        config.network.chain_id = 1;
        config.network.network_name = String::new();
        assert!(config.validate().is_err());
    }
}
