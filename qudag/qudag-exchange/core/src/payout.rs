//! Payout System for QuDAG Exchange Business Plan
//!
//! Implements vault-based automatic revenue distribution to contributors
//! based on usage, with support for agent providers, plugin creators,
//! node operators, and bounty agents.

#[cfg(not(feature = "std"))]
use alloc::{collections::BTreeMap, string::String, vec::Vec};

#[cfg(feature = "std")]
use std::{collections::BTreeMap, string::String, vec::Vec};

use crate::{
    types::{rUv, Hash, Timestamp},
    AccountId, Error, Result,
};
use serde::{Deserialize, Serialize};

/// Contributor role types for payout distribution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContributorRole {
    /// Agent providers earn per compute/storage/bandwidth consumed
    AgentProvider {
        /// Agent ID that generated the revenue
        agent_id: String,
        /// Metered resource consumption
        resource_consumed: u64,
    },
    /// Plugin/Module creators earn micro-payouts per usage
    PluginCreator {
        /// Module ID that was used
        module_id: String,
        /// Number of times module was executed
        usage_count: u64,
    },
    /// Node operators earn via routing/consensus participation
    NodeOperator {
        /// Node ID that participated
        node_id: String,
        /// Consensus rounds participated in
        consensus_rounds: u64,
        /// Uptime percentage (0.0 to 1.0)
        uptime_percentage: f64,
    },
    /// Bounty agents claim rewards for task completion
    BountyAgent {
        /// Bounty/task ID completed
        bounty_id: String,
        /// Completion timestamp
        completed_at: Timestamp,
    },
}

/// Payout configuration for different contributor types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayoutConfig {
    /// Enable automatic payout distribution
    pub enabled: bool,

    /// Default split templates
    pub default_splits: PayoutSplitTemplates,

    /// Minimum payout threshold (payouts below this are accumulated)
    pub min_payout_threshold: rUv,

    /// Maximum payout percentage any single contributor can receive
    pub max_contributor_percentage: f64,

    /// Fee retained for system operations (Genesis allocation)
    pub system_fee_percentage: f64,

    /// Audit trail retention period in seconds
    pub audit_retention_seconds: u64,

    /// Enable zero-knowledge proofs for distribution fairness
    pub enable_zk_proofs: bool,
}

impl Default for PayoutConfig {
    fn default() -> Self {
        Self {
            enabled: false, // Opt-in by default
            default_splits: PayoutSplitTemplates::default(),
            min_payout_threshold: rUv::new(10), // 10 rUv minimum
            max_contributor_percentage: 0.85,   // Max 85% to any single contributor
            system_fee_percentage: 0.0001,      // 0.01% for Genesis allocation
            audit_retention_seconds: 365 * 24 * 60 * 60, // 1 year
            enable_zk_proofs: false,
        }
    }
}

/// Default payout split templates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayoutSplitTemplates {
    /// Single-agent jobs (agent, infrastructure)
    pub single_agent: PayoutSplit,

    /// Plugin-enhanced jobs (agent, plugin, infrastructure)
    pub plugin_enhanced: PayoutSplit,

    /// Node operation rewards (node operator, network, system)
    pub node_operation: PayoutSplit,

    /// Bounty completion rewards (agent, bounty poster, system)
    pub bounty_completion: PayoutSplit,
}

impl Default for PayoutSplitTemplates {
    fn default() -> Self {
        Self {
            single_agent: PayoutSplit {
                percentages: vec![
                    (ContributorType::Agent, 0.95),
                    (ContributorType::Infrastructure, 0.05),
                ],
            },
            plugin_enhanced: PayoutSplit {
                percentages: vec![
                    (ContributorType::Agent, 0.85),
                    (ContributorType::Plugin, 0.10),
                    (ContributorType::Infrastructure, 0.05),
                ],
            },
            node_operation: PayoutSplit {
                percentages: vec![
                    (ContributorType::Node, 0.80),
                    (ContributorType::Network, 0.15),
                    (ContributorType::System, 0.05),
                ],
            },
            bounty_completion: PayoutSplit {
                percentages: vec![
                    (ContributorType::Agent, 0.90),
                    (ContributorType::BountyPoster, 0.05),
                    (ContributorType::System, 0.05),
                ],
            },
        }
    }
}

/// Payout split definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayoutSplit {
    /// Percentage allocation for each contributor type
    pub percentages: Vec<(ContributorType, f64)>,
}

/// Generic contributor types for payout splits
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ContributorType {
    Agent,
    Plugin,
    Node,
    Infrastructure,
    Network,
    System,
    BountyPoster,
}

/// Payout entry for tracking individual distributions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayoutEntry {
    /// Unique payout ID
    pub payout_id: String,

    /// Transaction that triggered this payout
    pub triggering_tx_id: String,

    /// Contributor receiving the payout
    pub contributor: ContributorRole,

    /// Contributor's vault/account ID
    pub vault_id: AccountId,

    /// Amount paid out
    pub amount: rUv,

    /// Percentage of total fee this represents
    pub percentage: f64,

    /// Timestamp of payout
    pub paid_at: Timestamp,

    /// Split template used for calculation
    pub split_template: String,

    /// Optional proof of fair distribution (ZK-proof)
    pub fairness_proof: Option<Vec<u8>>,
}

/// Payout transaction mapping multiple recipients
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayoutTransaction {
    /// Unique transaction ID
    pub tx_id: String,

    /// Original transaction amount
    pub total_amount: rUv,

    /// Total fees collected
    pub total_fees: rUv,

    /// Individual payout entries
    pub payouts: Vec<PayoutEntry>,

    /// Timestamp of distribution
    pub distributed_at: Timestamp,

    /// Hash of the transaction for auditability
    pub transaction_hash: Hash,
}

/// Fee router for automatic payout distribution
#[derive(Debug, Clone)]
pub struct FeeRouter {
    /// Current payout configuration
    config: PayoutConfig,

    /// Accumulated pending payouts below threshold
    pending_payouts: BTreeMap<AccountId, rUv>,

    /// Payout history for auditability
    payout_history: Vec<PayoutTransaction>,

    /// Contributor registry
    contributors: BTreeMap<String, ContributorInfo>,
}

/// Information about a registered contributor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContributorInfo {
    /// Contributor's vault ID for payouts
    pub vault_id: AccountId,

    /// Contributor role and metadata
    pub role: ContributorRole,

    /// Custom payout percentage (if different from template)
    pub custom_percentage: Option<f64>,

    /// Registration timestamp
    pub registered_at: Timestamp,

    /// Total earnings to date
    pub total_earnings: rUv,

    /// Last payout timestamp
    pub last_payout: Option<Timestamp>,
}

impl FeeRouter {
    /// Create a new fee router with configuration
    pub fn new(config: PayoutConfig) -> Self {
        Self {
            config,
            pending_payouts: BTreeMap::new(),
            payout_history: Vec::new(),
            contributors: BTreeMap::new(),
        }
    }

    /// Register a new contributor for payouts
    pub fn register_contributor(
        &mut self,
        contributor_id: String,
        info: ContributorInfo,
    ) -> Result<()> {
        if !self.config.enabled {
            return Err(Error::Other("Payout system is disabled".into()));
        }

        // Validate custom percentage if provided
        if let Some(pct) = info.custom_percentage {
            if pct < 0.0 || pct > self.config.max_contributor_percentage {
                return Err(Error::Other(
                    format!(
                        "Custom percentage {} exceeds maximum allowed {}",
                        pct, self.config.max_contributor_percentage
                    )
                    .into(),
                ));
            }
        }

        self.contributors.insert(contributor_id, info);
        Ok(())
    }

    /// Distribute fees from a transaction to contributors
    pub fn distribute_fees(
        &mut self,
        tx_id: String,
        total_fee: rUv,
        contributor_roles: Vec<ContributorRole>,
        current_time: Timestamp,
    ) -> Result<PayoutTransaction> {
        if !self.config.enabled {
            return Err(Error::Other("Payout system is disabled".into()));
        }

        // Determine appropriate split template
        let split_template = self.determine_split_template(&contributor_roles)?;

        // Calculate individual payouts
        let mut payouts = Vec::new();
        let mut total_distributed = rUv::new(0);

        for role in contributor_roles {
            let (contributor_id, payout_amount, percentage) =
                self.calculate_payout(&role, total_fee, &split_template)?;

            if let Some(contributor_info) = self.contributors.get(&contributor_id) {
                let payout_entry = PayoutEntry {
                    payout_id: format!("{}-{}", tx_id, contributor_id),
                    triggering_tx_id: tx_id.clone(),
                    contributor: role,
                    vault_id: contributor_info.vault_id.clone(),
                    amount: payout_amount,
                    percentage,
                    paid_at: current_time,
                    split_template: split_template.clone(),
                    fairness_proof: None, // TODO: Implement ZK-proofs
                };

                payouts.push(payout_entry);
                total_distributed = total_distributed
                    .add(payout_amount)
                    .map_err(|e| Error::Other(e.into()))?;

                // Update contributor earnings
                self.update_contributor_earnings(&contributor_id, payout_amount, current_time)?;
            }
        }

        // Add system fee (Genesis allocation)
        let system_fee = total_fee
            .multiply(self.config.system_fee_percentage)
            .map_err(|e| Error::Other(e.into()))?;
        total_distributed = total_distributed
            .add(system_fee)
            .map_err(|e| Error::Other(e.into()))?;

        // Create payout transaction record
        let payout_tx = PayoutTransaction {
            tx_id: tx_id.clone(),
            total_amount: total_fee,
            total_fees: total_fee,
            payouts,
            distributed_at: current_time,
            transaction_hash: self.calculate_transaction_hash(&tx_id, total_fee, current_time)?,
        };

        // Store in history
        self.payout_history.push(payout_tx.clone());
        self.cleanup_old_history(current_time);

        Ok(payout_tx)
    }

    /// Process pending payouts that have reached threshold
    pub fn process_pending_payouts(
        &mut self,
        current_time: Timestamp,
    ) -> Result<Vec<PayoutTransaction>> {
        let mut processed = Vec::new();
        let mut to_remove = Vec::new();

        for (account_id, amount) in &self.pending_payouts {
            if amount.amount() >= self.config.min_payout_threshold.amount() {
                // Create payout transaction for accumulated amount
                let tx_id = format!("pending-{}-{}", account_id, current_time.value());

                // TODO: Create proper payout transaction
                to_remove.push(account_id.clone());
            }
        }

        // Remove processed pending payouts
        for account_id in to_remove {
            self.pending_payouts.remove(&account_id);
        }

        Ok(processed)
    }

    /// Get payout history for auditability
    pub fn get_payout_history(&self, limit: Option<usize>) -> Vec<&PayoutTransaction> {
        match limit {
            Some(n) => self.payout_history.iter().rev().take(n).collect(),
            None => self.payout_history.iter().collect(),
        }
    }

    /// Get contributor information
    pub fn get_contributor(&self, contributor_id: &str) -> Option<&ContributorInfo> {
        self.contributors.get(contributor_id)
    }

    /// Update payout configuration
    pub fn update_config(&mut self, config: PayoutConfig) -> Result<()> {
        // Validate new configuration
        config.validate()?;
        self.config = config;
        Ok(())
    }

    /// Private helper methods

    fn determine_split_template(&self, roles: &[ContributorRole]) -> Result<String> {
        let has_plugin = roles
            .iter()
            .any(|r| matches!(r, ContributorRole::PluginCreator { .. }));
        let has_node = roles
            .iter()
            .any(|r| matches!(r, ContributorRole::NodeOperator { .. }));
        let has_bounty = roles
            .iter()
            .any(|r| matches!(r, ContributorRole::BountyAgent { .. }));

        Ok(match (has_plugin, has_node, has_bounty) {
            (true, false, false) => "plugin_enhanced".to_string(),
            (false, true, false) => "node_operation".to_string(),
            (false, false, true) => "bounty_completion".to_string(),
            _ => "single_agent".to_string(),
        })
    }

    fn calculate_payout(
        &self,
        role: &ContributorRole,
        total_fee: rUv,
        split_template: &str,
    ) -> Result<(String, rUv, f64)> {
        let contributor_id = self.get_contributor_id_from_role(role);

        // Get base percentage from template
        let base_percentage = self.get_base_percentage_for_role(role, split_template)?;

        // Check for custom percentage override
        let final_percentage = if let Some(contributor) = self.contributors.get(&contributor_id) {
            contributor.custom_percentage.unwrap_or(base_percentage)
        } else {
            base_percentage
        };

        let payout_amount = total_fee
            .multiply(final_percentage)
            .map_err(|e| Error::Other(e.into()))?;

        Ok((contributor_id, payout_amount, final_percentage))
    }

    fn get_contributor_id_from_role(&self, role: &ContributorRole) -> String {
        match role {
            ContributorRole::AgentProvider { agent_id, .. } => agent_id.clone(),
            ContributorRole::PluginCreator { module_id, .. } => module_id.clone(),
            ContributorRole::NodeOperator { node_id, .. } => node_id.clone(),
            ContributorRole::BountyAgent { bounty_id, .. } => bounty_id.clone(),
        }
    }

    fn get_base_percentage_for_role(
        &self,
        role: &ContributorRole,
        split_template: &str,
    ) -> Result<f64> {
        let template = match split_template {
            "single_agent" => &self.config.default_splits.single_agent,
            "plugin_enhanced" => &self.config.default_splits.plugin_enhanced,
            "node_operation" => &self.config.default_splits.node_operation,
            "bounty_completion" => &self.config.default_splits.bounty_completion,
            _ => {
                return Err(Error::Other(
                    format!("Unknown split template: {}", split_template).into(),
                ))
            }
        };

        let contributor_type = match role {
            ContributorRole::AgentProvider { .. } => ContributorType::Agent,
            ContributorRole::PluginCreator { .. } => ContributorType::Plugin,
            ContributorRole::NodeOperator { .. } => ContributorType::Node,
            ContributorRole::BountyAgent { .. } => ContributorType::Agent,
        };

        template
            .percentages
            .iter()
            .find(|(ct, _)| ct == &contributor_type)
            .map(|(_, pct)| *pct)
            .ok_or_else(|| {
                Error::Other(
                    format!(
                        "No percentage found for contributor type: {:?}",
                        contributor_type
                    )
                    .into(),
                )
            })
    }

    fn update_contributor_earnings(
        &mut self,
        contributor_id: &str,
        amount: rUv,
        current_time: Timestamp,
    ) -> Result<()> {
        if let Some(contributor) = self.contributors.get_mut(contributor_id) {
            contributor.total_earnings = contributor
                .total_earnings
                .add(amount)
                .map_err(|e| Error::Other(e.into()))?;
            contributor.last_payout = Some(current_time);
        }
        Ok(())
    }

    fn calculate_transaction_hash(
        &self,
        tx_id: &str,
        total_fee: rUv,
        timestamp: Timestamp,
    ) -> Result<Hash> {
        use blake3::Hasher;

        let mut hasher = Hasher::new();
        hasher.update(tx_id.as_bytes());
        hasher.update(&total_fee.amount().to_le_bytes());
        hasher.update(&timestamp.value().to_le_bytes());

        let hash_bytes = hasher.finalize();
        Ok(Hash::from_bytes(*hash_bytes.as_bytes()))
    }

    fn cleanup_old_history(&mut self, current_time: Timestamp) {
        let cutoff_time = current_time
            .value()
            .saturating_sub(self.config.audit_retention_seconds);
        self.payout_history
            .retain(|tx| tx.distributed_at.value() >= cutoff_time);
    }
}

impl PayoutConfig {
    /// Validate payout configuration
    pub fn validate(&self) -> Result<()> {
        if self.max_contributor_percentage <= 0.0 || self.max_contributor_percentage > 1.0 {
            return Err(Error::Other(
                "max_contributor_percentage must be between 0.0 and 1.0".into(),
            ));
        }

        if self.system_fee_percentage < 0.0 || self.system_fee_percentage > 0.1 {
            return Err(Error::Other(
                "system_fee_percentage must be between 0.0 and 0.1 (10% max)".into(),
            ));
        }

        if self.min_payout_threshold.amount() == 0 {
            return Err(Error::Other(
                "min_payout_threshold must be greater than 0".into(),
            ));
        }

        // Validate all split templates sum to <= 1.0
        for (name, split) in [
            ("single_agent", &self.default_splits.single_agent),
            ("plugin_enhanced", &self.default_splits.plugin_enhanced),
            ("node_operation", &self.default_splits.node_operation),
            ("bounty_completion", &self.default_splits.bounty_completion),
        ] {
            let total: f64 = split.percentages.iter().map(|(_, pct)| pct).sum();
            if total > 1.0 {
                return Err(Error::Other(
                    format!(
                        "Split template '{}' percentages sum to {}, must be <= 1.0",
                        name, total
                    )
                    .into(),
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_payout_config_validation() {
        let mut config = PayoutConfig::default();
        config.validate().unwrap();

        // Test invalid max contributor percentage
        config.max_contributor_percentage = 1.5;
        assert!(config.validate().is_err());

        config.max_contributor_percentage = 0.85;
        config.system_fee_percentage = 0.2; // Too high
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_fee_router_creation() {
        let config = PayoutConfig::default();
        let router = FeeRouter::new(config);
        assert!(!router.config.enabled); // Disabled by default
    }

    #[test]
    fn test_contributor_registration() {
        let mut config = PayoutConfig::default();
        config.enabled = true; // Enable for testing
        let mut router = FeeRouter::new(config);

        let contributor_info = ContributorInfo {
            vault_id: AccountId::new("test-vault".to_string()),
            role: ContributorRole::AgentProvider {
                agent_id: "agent-1".to_string(),
                resource_consumed: 100,
            },
            custom_percentage: None,
            registered_at: Timestamp::new(1000),
            total_earnings: rUv::new(0),
            last_payout: None,
        };

        router
            .register_contributor("agent-1".to_string(), contributor_info)
            .unwrap();

        let registered = router.get_contributor("agent-1").unwrap();
        assert_eq!(registered.total_earnings.amount(), 0);
    }

    #[test]
    fn test_payout_distribution() {
        let mut config = PayoutConfig::default();
        config.enabled = true;
        let mut router = FeeRouter::new(config);

        // Register a contributor
        let contributor_info = ContributorInfo {
            vault_id: AccountId::new("test-vault".to_string()),
            role: ContributorRole::AgentProvider {
                agent_id: "agent-1".to_string(),
                resource_consumed: 100,
            },
            custom_percentage: None,
            registered_at: Timestamp::new(1000),
            total_earnings: rUv::new(0),
            last_payout: None,
        };

        router
            .register_contributor("agent-1".to_string(), contributor_info)
            .unwrap();

        // Distribute fees
        let roles = vec![ContributorRole::AgentProvider {
            agent_id: "agent-1".to_string(),
            resource_consumed: 100,
        }];

        let payout_tx = router
            .distribute_fees(
                "tx-1".to_string(),
                rUv::new(1000),
                roles,
                Timestamp::new(2000),
            )
            .unwrap();

        assert_eq!(payout_tx.payouts.len(), 1);
        assert_eq!(payout_tx.payouts[0].amount.amount(), 950); // 95% of 1000

        // Check contributor earnings were updated
        let contributor = router.get_contributor("agent-1").unwrap();
        assert_eq!(contributor.total_earnings.amount(), 950);
    }
}
