//! Integration tests for business plan features
//!
//! Tests the complete workflow of enabling business plan features,
//! registering contributors, and processing payouts.

use qudag_exchange_core::{
    rUv, types::Timestamp, AccountId, BusinessPlanConfig, ContributorInfo, ContributorRole,
    ExchangeConfig, ExchangeConfigBuilder, FeeRouter, PayoutConfig,
};

#[test]
fn test_business_plan_enable_disable() {
    let mut config = ExchangeConfig::new().unwrap();

    // Initially disabled
    assert!(!config.has_business_plan());

    // Enable business plan
    let bp_config = BusinessPlanConfig {
        enabled: true,
        enable_auto_distribution: true,
        enable_role_earnings: true,
        ..Default::default()
    };

    config.enable_business_plan(bp_config).unwrap();
    assert!(config.has_business_plan());
    assert!(config.fee_router().is_some());

    // Disable business plan
    config.disable_business_plan();
    assert!(!config.has_business_plan());
    assert!(config.fee_router().is_none());
}

#[test]
fn test_business_plan_config_builder() {
    let config = ExchangeConfigBuilder::new()
        .with_basic_business_plan()
        .build()
        .unwrap();

    assert!(config.has_business_plan());

    let summary = config.get_summary(Timestamp::now());
    let bp_summary = summary.business_plan_summary.unwrap();

    assert!(bp_summary.enabled);
    assert!(bp_summary.auto_distribution_enabled);
    assert!(bp_summary.role_earnings_enabled);
    assert_eq!(bp_summary.min_payout_threshold, 10); // Default threshold
}

#[test]
fn test_fee_router_contributor_registration() {
    let payout_config = PayoutConfig::default();
    let mut fee_router = FeeRouter::new(payout_config);

    // Register an agent provider
    let contributor_info = ContributorInfo {
        vault_id: AccountId::new("vault_123"),
        role: ContributorRole::AgentProvider {
            agent_id: "agent_456".to_string(),
            resource_consumed: 100,
        },
        custom_percentage: Some(0.90), // 90% instead of default
        registered_at: Timestamp::new(1000),
        total_earnings: rUv::new(0),
        last_payout: None,
    };

    fee_router
        .register_contributor("agent_456".to_string(), contributor_info)
        .unwrap();

    // Verify contributor is registered
    let contributor = fee_router.get_contributor("agent_456").unwrap();
    assert_eq!(contributor.vault_id.as_str(), "vault_123");
    assert_eq!(contributor.custom_percentage, Some(0.90));
    assert_eq!(contributor.total_earnings.amount(), 0);
}

#[test]
fn test_fee_distribution_single_agent() {
    let mut payout_config = PayoutConfig::default();
    payout_config.enabled = true;
    let mut fee_router = FeeRouter::new(payout_config);

    // Register agent
    let contributor_info = ContributorInfo {
        vault_id: AccountId::new("vault_agent"),
        role: ContributorRole::AgentProvider {
            agent_id: "agent_123".to_string(),
            resource_consumed: 100,
        },
        custom_percentage: None, // Use default 95%
        registered_at: Timestamp::new(1000),
        total_earnings: rUv::new(0),
        last_payout: None,
    };

    fee_router
        .register_contributor("agent_123".to_string(), contributor_info)
        .unwrap();

    // Distribute fees for single-agent job
    let roles = vec![ContributorRole::AgentProvider {
        agent_id: "agent_123".to_string(),
        resource_consumed: 100,
    }];

    let payout_tx = fee_router
        .distribute_fees(
            "tx_test_001".to_string(),
            rUv::new(1000), // 1000 rUv total fee
            roles,
            Timestamp::new(2000),
        )
        .unwrap();

    // Verify payout structure
    assert_eq!(payout_tx.payouts.len(), 1);
    let agent_payout = &payout_tx.payouts[0];

    assert_eq!(agent_payout.amount.amount(), 950); // 95% of 1000
    assert_eq!(agent_payout.percentage, 0.95);
    assert_eq!(agent_payout.vault_id.as_str(), "vault_agent");

    // Verify contributor earnings updated
    let contributor = fee_router.get_contributor("agent_123").unwrap();
    assert_eq!(contributor.total_earnings.amount(), 950);
    assert_eq!(contributor.last_payout, Some(Timestamp::new(2000)));
}

#[test]
fn test_fee_distribution_plugin_enhanced() {
    let mut payout_config = PayoutConfig::default();
    payout_config.enabled = true;
    let mut fee_router = FeeRouter::new(payout_config);

    // Register agent and plugin creator
    fee_router
        .register_contributor(
            "agent_123".to_string(),
            ContributorInfo {
                vault_id: AccountId::new("vault_agent"),
                role: ContributorRole::AgentProvider {
                    agent_id: "agent_123".to_string(),
                    resource_consumed: 100,
                },
                custom_percentage: None,
                registered_at: Timestamp::new(1000),
                total_earnings: rUv::new(0),
                last_payout: None,
            },
        )
        .unwrap();

    fee_router
        .register_contributor(
            "plugin_456".to_string(),
            ContributorInfo {
                vault_id: AccountId::new("vault_plugin"),
                role: ContributorRole::PluginCreator {
                    module_id: "plugin_456".to_string(),
                    usage_count: 5,
                },
                custom_percentage: None,
                registered_at: Timestamp::new(1000),
                total_earnings: rUv::new(0),
                last_payout: None,
            },
        )
        .unwrap();

    // Distribute fees for plugin-enhanced job
    let roles = vec![
        ContributorRole::AgentProvider {
            agent_id: "agent_123".to_string(),
            resource_consumed: 100,
        },
        ContributorRole::PluginCreator {
            module_id: "plugin_456".to_string(),
            usage_count: 5,
        },
    ];

    let payout_tx = fee_router
        .distribute_fees(
            "tx_test_002".to_string(),
            rUv::new(1000),
            roles,
            Timestamp::new(2000),
        )
        .unwrap();

    // Verify payouts
    assert_eq!(payout_tx.payouts.len(), 2);

    // Find agent and plugin payouts
    let agent_payout = payout_tx
        .payouts
        .iter()
        .find(|p| p.vault_id.as_str() == "vault_agent")
        .unwrap();
    let plugin_payout = payout_tx
        .payouts
        .iter()
        .find(|p| p.vault_id.as_str() == "vault_plugin")
        .unwrap();

    assert_eq!(agent_payout.amount.amount(), 850); // 85% of 1000
    assert_eq!(plugin_payout.amount.amount(), 100); // 10% of 1000
}

#[test]
fn test_custom_percentage_override() {
    let mut payout_config = PayoutConfig::default();
    payout_config.enabled = true;
    let mut fee_router = FeeRouter::new(payout_config);

    // Register agent with custom percentage
    fee_router
        .register_contributor(
            "agent_custom".to_string(),
            ContributorInfo {
                vault_id: AccountId::new("vault_custom"),
                role: ContributorRole::AgentProvider {
                    agent_id: "agent_custom".to_string(),
                    resource_consumed: 100,
                },
                custom_percentage: Some(0.80), // 80% instead of default 95%
                registered_at: Timestamp::new(1000),
                total_earnings: rUv::new(0),
                last_payout: None,
            },
        )
        .unwrap();

    // Distribute fees
    let roles = vec![ContributorRole::AgentProvider {
        agent_id: "agent_custom".to_string(),
        resource_consumed: 100,
    }];

    let payout_tx = fee_router
        .distribute_fees(
            "tx_custom_001".to_string(),
            rUv::new(1000),
            roles,
            Timestamp::new(2000),
        )
        .unwrap();

    // Verify custom percentage is used
    let payout = &payout_tx.payouts[0];
    assert_eq!(payout.amount.amount(), 800); // 80% of 1000
    assert_eq!(payout.percentage, 0.80);
}

#[test]
fn test_payout_config_validation() {
    let mut config = PayoutConfig::default();

    // Valid configuration should pass
    config.validate().unwrap();

    // Invalid max contributor percentage
    config.max_contributor_percentage = 1.5;
    assert!(config.validate().is_err());

    // Reset and test invalid system fee
    config.max_contributor_percentage = 0.85;
    config.system_fee_percentage = 0.2; // Too high
    assert!(config.validate().is_err());

    // Reset and test zero threshold
    config.system_fee_percentage = 0.0001;
    config.min_payout_threshold = rUv::new(0);
    assert!(config.validate().is_err());
}

#[test]
fn test_payout_history_tracking() {
    let mut payout_config = PayoutConfig::default();
    payout_config.enabled = true;
    let mut fee_router = FeeRouter::new(payout_config);

    // Register contributor
    fee_router
        .register_contributor(
            "agent_history".to_string(),
            ContributorInfo {
                vault_id: AccountId::new("vault_history"),
                role: ContributorRole::AgentProvider {
                    agent_id: "agent_history".to_string(),
                    resource_consumed: 100,
                },
                custom_percentage: None,
                registered_at: Timestamp::new(1000),
                total_earnings: rUv::new(0),
                last_payout: None,
            },
        )
        .unwrap();

    // Process multiple transactions
    for i in 1..=3 {
        let roles = vec![ContributorRole::AgentProvider {
            agent_id: "agent_history".to_string(),
            resource_consumed: 100,
        }];

        fee_router
            .distribute_fees(
                format!("tx_history_{:03}", i),
                rUv::new(1000),
                roles,
                Timestamp::new(2000 + i * 1000),
            )
            .unwrap();
    }

    // Check payout history
    let history = fee_router.get_payout_history(Some(2));
    assert_eq!(history.len(), 2); // Limited to 2 most recent

    let all_history = fee_router.get_payout_history(None);
    assert_eq!(all_history.len(), 3); // All transactions

    // Verify contributor total earnings
    let contributor = fee_router.get_contributor("agent_history").unwrap();
    assert_eq!(contributor.total_earnings.amount(), 2850); // 3 * 950
}

#[test]
fn test_business_plan_summary() {
    let bp_config = BusinessPlanConfig {
        enabled: true,
        enable_auto_distribution: true,
        enable_vault_management: false,
        enable_role_earnings: true,
        enable_bounty_rewards: false,
        payout_config: PayoutConfig {
            enabled: true,
            min_payout_threshold: rUv::new(50),
            system_fee_percentage: 0.002,
            ..Default::default()
        },
        ..Default::default()
    };

    let config = ExchangeConfigBuilder::new()
        .with_business_plan(bp_config)
        .build()
        .unwrap();

    let summary = config.get_summary(Timestamp::now());
    let bp_summary = summary.business_plan_summary.unwrap();

    assert!(bp_summary.enabled);
    assert!(bp_summary.auto_distribution_enabled);
    assert!(!bp_summary.vault_management_enabled);
    assert!(bp_summary.role_earnings_enabled);
    assert!(!bp_summary.bounty_rewards_enabled);
    assert_eq!(bp_summary.min_payout_threshold, 50);
    assert_eq!(bp_summary.system_fee_percentage, 0.002);
}

#[test]
fn test_node_operator_payouts() {
    let mut payout_config = PayoutConfig::default();
    payout_config.enabled = true;
    let mut fee_router = FeeRouter::new(payout_config);

    // Register node operator
    fee_router
        .register_contributor(
            "node_001".to_string(),
            ContributorInfo {
                vault_id: AccountId::new("vault_node"),
                role: ContributorRole::NodeOperator {
                    node_id: "node_001".to_string(),
                    consensus_rounds: 100,
                    uptime_percentage: 0.99,
                },
                custom_percentage: None,
                registered_at: Timestamp::new(1000),
                total_earnings: rUv::new(0),
                last_payout: None,
            },
        )
        .unwrap();

    // Process node operation payout
    let roles = vec![ContributorRole::NodeOperator {
        node_id: "node_001".to_string(),
        consensus_rounds: 100,
        uptime_percentage: 0.99,
    }];

    let payout_tx = fee_router
        .distribute_fees(
            "tx_node_001".to_string(),
            rUv::new(1000),
            roles,
            Timestamp::new(2000),
        )
        .unwrap();

    // Verify node operation payout uses correct template
    assert_eq!(payout_tx.payouts.len(), 1);
    let node_payout = &payout_tx.payouts[0];

    // Should use node operation template (80% for node operator)
    assert_eq!(node_payout.amount.amount(), 800);
    assert_eq!(node_payout.percentage, 0.80);
}
