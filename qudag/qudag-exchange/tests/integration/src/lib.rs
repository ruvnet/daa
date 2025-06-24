//! Integration tests for QuDAG Exchange

#[cfg(test)]
mod tests {
    use qudag_exchange_core::{
        Ledger, RuvAmount, Transaction, TransactionType,
        ResourceMetrics, ResourceType,
    };
    use serial_test::serial;

    #[tokio::test]
    #[serial]
    async fn test_basic_transfer_flow() {
        // Create ledger
        let ledger = Ledger::new();

        // Create wallets
        let alice = ledger.get_or_create_wallet("alice".to_string(), false);
        let bob = ledger.get_or_create_wallet("bob".to_string(), false);

        // Give Alice some initial balance via minting
        ledger.start_resource_contribution("alice".to_string());
        
        let metric = ResourceMetrics {
            resource_type: ResourceType::Cpu,
            amount: 1000.0,
            duration: 3600,
            quality_score: 1.0,
            timestamp: 0,
        };
        
        ledger.record_resource_metric("alice", metric).unwrap();
        let mint_tx = ledger.finalize_resource_contribution("alice").unwrap().unwrap();
        ledger.process_transaction(&mint_tx).unwrap();

        // Check Alice's balance
        let alice_balance = ledger.get_balance("alice").unwrap();
        assert_eq!(alice_balance.as_ruv(), 100); // 1000 * 1 * 1.0 * 0.1 = 100 rUv

        // Create transfer transaction
        let tx = Transaction::new(
            TransactionType::Transfer {
                from: "alice".to_string(),
                to: "bob".to_string(),
                amount: RuvAmount::from_ruv(50),
            },
            RuvAmount::from_ruv(1),
        );

        // Submit and process transaction
        let tx_id = ledger.submit_transaction(tx).unwrap();
        ledger.process_transaction(&tx_id).unwrap();

        // Verify balances
        let alice_balance = ledger.get_balance("alice").unwrap();
        let bob_balance = ledger.get_balance("bob").unwrap();
        
        assert_eq!(alice_balance.as_ruv(), 49); // 100 - 50 - 1 (fee)
        assert_eq!(bob_balance.as_ruv(), 50);
    }

    #[tokio::test]
    #[serial]
    async fn test_insufficient_balance() {
        let ledger = Ledger::new();
        
        // Create wallets
        ledger.get_or_create_wallet("alice".to_string(), false);
        ledger.get_or_create_wallet("bob".to_string(), false);

        // Try to transfer without balance
        let tx = Transaction::new(
            TransactionType::Transfer {
                from: "alice".to_string(),
                to: "bob".to_string(),
                amount: RuvAmount::from_ruv(100),
            },
            RuvAmount::from_ruv(1),
        );

        // Should fail
        assert!(ledger.submit_transaction(tx).is_err());
    }

    #[tokio::test]
    #[serial]
    async fn test_resource_contribution_flow() {
        let ledger = Ledger::new();
        
        // Start contribution
        ledger.start_resource_contribution("agent1".to_string());
        
        // Submit multiple metrics
        let metrics = vec![
            ResourceMetrics {
                resource_type: ResourceType::Cpu,
                amount: 500.0,
                duration: 3600,
                quality_score: 0.9,
                timestamp: 0,
            },
            ResourceMetrics {
                resource_type: ResourceType::Gpu,
                amount: 10.0,
                duration: 1800,
                quality_score: 1.0,
                timestamp: 0,
            },
            ResourceMetrics {
                resource_type: ResourceType::Memory,
                amount: 32.0,
                duration: 7200,
                quality_score: 0.95,
                timestamp: 0,
            },
        ];

        for metric in metrics {
            ledger.record_resource_metric("agent1", metric).unwrap();
        }

        // Finalize and mint
        let mint_tx = ledger.finalize_resource_contribution("agent1").unwrap().unwrap();
        ledger.process_transaction(&mint_tx).unwrap();

        // Check balance
        let balance = ledger.get_balance("agent1").unwrap();
        assert!(balance.as_ruv() > 0);
        
        // Check total supply increased
        let total_supply = ledger.total_supply();
        assert_eq!(total_supply.as_ruv(), balance.as_ruv());
    }

    #[tokio::test]
    #[serial]
    async fn test_burn_transaction() {
        let ledger = Ledger::new();
        
        // Create wallet with balance
        ledger.get_or_create_wallet("alice".to_string(), false);
        
        // Mint some rUv
        ledger.start_resource_contribution("alice".to_string());
        let metric = ResourceMetrics {
            resource_type: ResourceType::Cpu,
            amount: 1000.0,
            duration: 3600,
            quality_score: 1.0,
            timestamp: 0,
        };
        ledger.record_resource_metric("alice", metric).unwrap();
        let mint_tx = ledger.finalize_resource_contribution("alice").unwrap().unwrap();
        ledger.process_transaction(&mint_tx).unwrap();

        let initial_balance = ledger.get_balance("alice").unwrap();
        let initial_supply = ledger.total_supply();

        // Burn some rUv
        let burn_tx = Transaction::new(
            TransactionType::Burn {
                from: "alice".to_string(),
                amount: RuvAmount::from_ruv(30),
            },
            RuvAmount::from_ruv(1),
        );

        let tx_id = ledger.submit_transaction(burn_tx).unwrap();
        ledger.process_transaction(&tx_id).unwrap();

        // Verify balance and supply decreased
        let final_balance = ledger.get_balance("alice").unwrap();
        let final_supply = ledger.total_supply();
        
        assert_eq!(final_balance.as_ruv(), initial_balance.as_ruv() - 30 - 1);
        assert_eq!(final_supply.as_ruv(), initial_supply.as_ruv() - 30);
    }

    #[tokio::test]
    #[serial]
    async fn test_fee_distribution() {
        let ledger = Ledger::new();
        
        // Create validator wallets
        let validators = vec!["validator1", "validator2", "validator3"];
        for v in &validators {
            ledger.get_or_create_wallet(v.to_string(), false);
        }

        // Create fee distribution transaction
        let fee_tx = Transaction::new(
            TransactionType::FeeDistribution {
                amount: RuvAmount::from_ruv(100),
                recipients: vec![
                    ("validator1".to_string(), 50),
                    ("validator2".to_string(), 30),
                    ("validator3".to_string(), 20),
                ],
            },
            RuvAmount::from_ruv(0),
        );

        // Process (this would normally come from collected fees)
        let tx_id = ledger.submit_transaction(fee_tx).unwrap();
        ledger.process_transaction(&tx_id).unwrap();

        // Verify distributions
        assert_eq!(ledger.get_balance("validator1").unwrap().as_ruv(), 50);
        assert_eq!(ledger.get_balance("validator2").unwrap().as_ruv(), 30);
        assert_eq!(ledger.get_balance("validator3").unwrap().as_ruv(), 20);
    }

    #[tokio::test]
    #[serial]
    async fn test_concurrent_transactions() {
        use tokio::task::JoinSet;
        
        let ledger = std::sync::Arc::new(Ledger::new());
        
        // Create source wallet with balance
        ledger.get_or_create_wallet("source".to_string(), false);
        
        // Mint initial balance
        ledger.start_resource_contribution("source".to_string());
        let metric = ResourceMetrics {
            resource_type: ResourceType::Gpu,
            amount: 100.0,
            duration: 3600,
            quality_score: 1.0,
            timestamp: 0,
        };
        ledger.record_resource_metric("source", metric).unwrap();
        let mint_tx = ledger.finalize_resource_contribution("source").unwrap().unwrap();
        ledger.process_transaction(&mint_tx).unwrap();

        // Create multiple recipient wallets
        for i in 0..10 {
            ledger.get_or_create_wallet(format!("recipient{}", i), false);
        }

        // Submit concurrent transfers
        let mut tasks = JoinSet::new();
        
        for i in 0..10 {
            let ledger_clone = ledger.clone();
            tasks.spawn(async move {
                let tx = Transaction::new(
                    TransactionType::Transfer {
                        from: "source".to_string(),
                        to: format!("recipient{}", i),
                        amount: RuvAmount::from_ruv(5),
                    },
                    RuvAmount::from_ruv(1),
                );
                
                ledger_clone.submit_transaction(tx)
            });
        }

        // Collect results
        let mut successful = 0;
        while let Some(result) = tasks.join_next().await {
            if result.unwrap().is_ok() {
                successful += 1;
            }
        }

        // Some should succeed based on available balance
        assert!(successful > 0);
        assert!(successful <= 10); // Can't send more than we have
    }
}

#[cfg(test)]
mod cli_tests {
    use assert_cmd::Command;
    use predicates::prelude::*;

    #[test]
    fn test_cli_help() {
        let mut cmd = Command::cargo_bin("qudag-exchange").unwrap();
        cmd.arg("--help")
            .assert()
            .success()
            .stdout(predicate::str::contains("QuDAG Exchange"));
    }

    #[test]
    fn test_cli_init() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        
        let mut cmd = Command::cargo_bin("qudag-exchange").unwrap();
        cmd.arg("--config")
            .arg(config_path.to_str().unwrap())
            .arg("init")
            .assert()
            .success();
    }
}