use qudag_crypto::{Blake3Hasher, DilithiumKeyPair, KyberKeyPair};
use qudag_dag::{Dag, Node};
use qudag_vault_core::*;
use std::sync::Arc;
use tempfile::TempDir;
use tokio::sync::RwLock;

#[cfg(test)]
mod crypto_integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_vault_with_qudag_crypto() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("test_vault.qdag");

        // Create vault using QuDAG crypto primitives
        let mut vault = Vault::create_with_crypto_backend(
            vault_path.to_str().unwrap(),
            "TestPassword123!",
            CryptoBackend::QuDAG,
        )
        .unwrap();

        // Add secret - should use BLAKE3 for hashing
        vault
            .add_secret("test/secret", "user", Some("password"))
            .unwrap();

        // Verify BLAKE3 is used for fingerprinting
        let fingerprint = vault.get_secret_fingerprint("test/secret").unwrap();
        assert_eq!(fingerprint.len(), 32, "BLAKE3 hash should be 32 bytes");

        // Verify we can compute the same fingerprint
        let hasher = Blake3Hasher::new();
        let computed = hasher.hash(b"test/secret").unwrap();
        assert_eq!(fingerprint, computed.as_bytes());
    }

    #[tokio::test]
    async fn test_vault_kyber_integration() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("test_vault.qdag");

        // Create two vaults for Alice and Bob
        let alice_vault = Vault::create(vault_path.to_str().unwrap(), "AlicePassword").unwrap();
        let bob_vault_path = temp_dir.path().join("bob_vault.qdag");
        let bob_vault = Vault::create(bob_vault_path.to_str().unwrap(), "BobPassword").unwrap();

        // Generate Kyber keypairs
        let alice_keypair = KyberKeyPair::generate().unwrap();
        let bob_keypair = KyberKeyPair::generate().unwrap();

        // Alice shares her vault key with Bob
        let encapsulated = alice_vault
            .encapsulate_vault_key(&bob_keypair.public())
            .unwrap();

        // Bob decapsulates the vault key
        let shared_key = bob_vault
            .decapsulate_vault_key(&encapsulated, &bob_keypair)
            .unwrap();

        // Verify Bob can now access Alice's vault with the shared key
        let alice_vault_copy =
            Vault::open_with_key(vault_path.to_str().unwrap(), &shared_key).unwrap();

        assert_eq!(
            alice_vault.vault_id(),
            alice_vault_copy.vault_id(),
            "Vault IDs should match after key sharing"
        );
    }

    #[tokio::test]
    async fn test_vault_dilithium_signatures() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("test_vault.qdag");

        let mut vault = Vault::create(vault_path.to_str().unwrap(), "TestPassword").unwrap();
        vault
            .add_secret("api/key", "service", Some("secret_key_123"))
            .unwrap();

        // Generate Dilithium keypair for signing
        let signing_key = DilithiumKeyPair::generate().unwrap();

        // Export vault with signature
        let signed_export = vault
            .export_with_signature(
                temp_dir.path().join("export.qdag").to_str().unwrap(),
                &signing_key,
            )
            .unwrap();

        // Verify signature is valid
        assert!(
            signed_export.verify(&signing_key.public()).unwrap(),
            "Dilithium signature should be valid"
        );

        // Import with signature verification
        let imported =
            Vault::import_with_verification(&signed_export, &signing_key.public(), "TestPassword")
                .unwrap();

        // Verify imported vault has same content
        let secret = imported.get_secret("api/key").unwrap();
        assert_eq!(secret.password, "secret_key_123");
    }
}

#[cfg(test)]
mod dag_integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_vault_dag_structure() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("test_vault.qdag");

        let mut vault = Vault::create(vault_path.to_str().unwrap(), "TestPassword").unwrap();

        // Build DAG structure for secrets
        vault.create_dag_node("root", NodeType::Category).unwrap();
        vault
            .create_dag_node("root/personal", NodeType::Category)
            .unwrap();
        vault
            .create_dag_node("root/work", NodeType::Category)
            .unwrap();

        // Add secrets as DAG nodes
        vault
            .add_secret_as_dag_node(
                "email/gmail",
                "user@gmail.com",
                Some("pass"),
                vec!["root/personal"],
            )
            .unwrap();

        vault
            .add_secret_as_dag_node(
                "email/work",
                "user@company.com",
                Some("pass"),
                vec!["root/work", "root/personal"], // Multi-parent
            )
            .unwrap();

        // Verify DAG structure
        let dag = vault.get_dag_view().unwrap();

        // Check root has two children
        let root_children = dag.get_children("root").unwrap();
        assert_eq!(root_children.len(), 2);

        // Check work email has two parents
        let work_parents = dag.get_parents("email/work").unwrap();
        assert_eq!(work_parents.len(), 2);
        assert!(work_parents.contains(&"root/work".to_string()));
        assert!(work_parents.contains(&"root/personal".to_string()));
    }

    #[tokio::test]
    async fn test_vault_dag_traversal() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("test_vault.qdag");

        let mut vault = Vault::create(vault_path.to_str().unwrap(), "TestPassword").unwrap();

        // Create complex DAG
        vault.create_dag_node("apps", NodeType::Category).unwrap();
        vault
            .create_dag_node("apps/dev", NodeType::Category)
            .unwrap();
        vault
            .create_dag_node("apps/prod", NodeType::Category)
            .unwrap();
        vault.create_dag_node("shared", NodeType::Category).unwrap();

        // Add interconnected secrets
        vault
            .add_secret_as_dag_node("db/dev", "dev_user", Some("pass"), vec!["apps/dev"])
            .unwrap();
        vault
            .add_secret_as_dag_node("db/prod", "prod_user", Some("pass"), vec!["apps/prod"])
            .unwrap();
        vault
            .add_secret_as_dag_node(
                "api/key",
                "api",
                Some("key"),
                vec!["apps/dev", "apps/prod", "shared"],
            )
            .unwrap();

        // Traverse from apps node
        let all_app_secrets = vault.traverse_from_node("apps").unwrap();
        assert!(all_app_secrets.len() >= 5); // apps, dev, prod, db/dev, db/prod, api/key

        // Traverse from shared
        let shared_secrets = vault.traverse_from_node("shared").unwrap();
        assert!(shared_secrets.contains(&"api/key".to_string()));
    }

    #[tokio::test]
    async fn test_vault_dag_consensus() {
        let temp_dir = TempDir::new().unwrap();

        // Simulate multiple vault replicas with DAG consensus
        let vault_paths: Vec<_> = (0..3)
            .map(|i| temp_dir.path().join(format!("vault_{}.qdag", i)))
            .collect();

        let vaults: Vec<_> = vault_paths
            .iter()
            .map(|p| {
                Arc::new(RwLock::new(
                    Vault::create_with_consensus(p.to_str().unwrap(), "TestPassword").unwrap(),
                ))
            })
            .collect();

        // Add secret to first vault
        {
            let mut vault0 = vaults[0].write().await;
            vault0
                .add_secret("distributed/secret", "user", Some("pass"))
                .unwrap();
        }

        // Sync via DAG consensus
        for i in 1..3 {
            let vault0 = vaults[0].read().await;
            let mut vault_i = vaults[i].write().await;

            let updates = vault0
                .get_dag_updates_since(vault_i.last_sync_point())
                .unwrap();
            vault_i.apply_dag_updates(updates).unwrap();
        }

        // Verify all vaults have the secret
        for vault in &vaults {
            let v = vault.read().await;
            let secret = v.get_secret("distributed/secret").unwrap();
            assert_eq!(secret.username, "user");
            assert_eq!(secret.password, "pass");
        }
    }
}

#[cfg(test)]
mod network_integration_tests {
    use super::*;
    use qudag_network::{NetworkConfig, PeerManager};

    #[tokio::test]
    async fn test_vault_p2p_sync() {
        let temp_dir = TempDir::new().unwrap();

        // Create network configs for two peers
        let config1 = NetworkConfig {
            listen_addr: "127.0.0.1:9001".parse().unwrap(),
            ..Default::default()
        };

        let config2 = NetworkConfig {
            listen_addr: "127.0.0.1:9002".parse().unwrap(),
            bootstrap_peers: vec!["127.0.0.1:9001".parse().unwrap()],
            ..Default::default()
        };

        // Create peer managers
        let peer1 = PeerManager::new(config1).await.unwrap();
        let peer2 = PeerManager::new(config2).await.unwrap();

        // Create vaults with P2P sync
        let vault1_path = temp_dir.path().join("vault1.qdag");
        let vault1 =
            Vault::create_with_p2p(vault1_path.to_str().unwrap(), "Password1", peer1.clone())
                .unwrap();

        let vault2_path = temp_dir.path().join("vault2.qdag");
        let vault2 =
            Vault::create_with_p2p(vault2_path.to_str().unwrap(), "Password2", peer2.clone())
                .unwrap();

        // Add secret to vault1
        vault1
            .add_secret("sync/test", "user", Some("password"))
            .unwrap();

        // Enable sync between vaults
        vault1.enable_p2p_sync(&vault2.vault_id()).unwrap();
        vault2.enable_p2p_sync(&vault1.vault_id()).unwrap();

        // Wait for sync
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // Verify vault2 received the update
        let synced_secret = vault2.get_secret("sync/test");
        assert!(synced_secret.is_ok(), "Secret should be synced to vault2");

        let secret = synced_secret.unwrap();
        assert_eq!(secret.username, "user");
        assert_eq!(secret.password, "password");
    }

    #[tokio::test]
    async fn test_vault_onion_routing() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("test_vault.qdag");

        // Create vault with anonymous networking
        let vault = Vault::create_with_anonymity(
            vault_path.to_str().unwrap(),
            "TestPassword",
            AnonymityLevel::High,
        )
        .unwrap();

        // Share vault key anonymously
        let recipient_pubkey = KyberKeyPair::generate().unwrap().public().clone();

        let anonymous_share = vault
            .share_anonymously(
                &recipient_pubkey,
                OnionRoutingConfig {
                    hops: 3,
                    exit_node: None,
                },
            )
            .await
            .unwrap();

        // Verify the share went through onion routing
        assert!(
            anonymous_share.routing_proof.len() >= 3,
            "Should have at least 3 hops in routing proof"
        );

        // Verify each hop is properly encrypted
        for hop in &anonymous_share.routing_proof {
            assert!(
                hop.encrypted_layer.len() > 100,
                "Each onion layer should be encrypted"
            );
        }
    }
}

#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[tokio::test]
    async fn test_vault_concurrent_operations() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("test_vault.qdag");

        let vault = Arc::new(RwLock::new(
            Vault::create(vault_path.to_str().unwrap(), "TestPassword").unwrap(),
        ));

        // Spawn multiple concurrent tasks
        let mut handles = vec![];

        for i in 0..10 {
            let vault_clone = vault.clone();
            let handle = tokio::spawn(async move {
                let mut vault = vault_clone.write().await;
                for j in 0..100 {
                    vault
                        .add_secret(
                            &format!("concurrent/secret_{}_{}", i, j),
                            &format!("user_{}", i),
                            Some(&format!("pass_{}", j)),
                        )
                        .unwrap();
                }
            });
            handles.push(handle);
        }

        // Wait for all tasks
        let start = Instant::now();
        for handle in handles {
            handle.await.unwrap();
        }
        let duration = start.elapsed();

        // Verify all secrets were added
        let vault = vault.read().await;
        assert_eq!(vault.secret_count(), 1000, "Should have 1000 secrets");

        // Performance check
        println!("Added 1000 secrets concurrently in {:?}", duration);
        assert!(
            duration < Duration::from_secs(5),
            "Concurrent operations too slow: {:?}",
            duration
        );
    }

    #[tokio::test]
    async fn test_vault_dag_scalability() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("test_vault.qdag");

        let mut vault = Vault::create(vault_path.to_str().unwrap(), "TestPassword").unwrap();

        // Create large DAG structure
        let start = Instant::now();

        // Create categories
        for i in 0..100 {
            vault
                .create_dag_node(&format!("category_{}", i), NodeType::Category)
                .unwrap();
        }

        // Add secrets with multiple parents
        for i in 0..1000 {
            let parents: Vec<String> = (0..5)
                .map(|j| format!("category_{}", (i + j) % 100))
                .collect();

            vault
                .add_secret_as_dag_node(
                    &format!("secret_{}", i),
                    &format!("user_{}", i),
                    Some("password"),
                    parents,
                )
                .unwrap();
        }

        let build_time = start.elapsed();
        println!("Built DAG with 1100 nodes in {:?}", build_time);

        // Test traversal performance
        let traverse_start = Instant::now();
        let all_nodes = vault.traverse_from_node("category_0").unwrap();
        let traverse_time = traverse_start.elapsed();

        println!("Traversed {} nodes in {:?}", all_nodes.len(), traverse_time);
        assert!(
            traverse_time < Duration::from_millis(100),
            "DAG traversal too slow: {:?}",
            traverse_time
        );
    }
}
