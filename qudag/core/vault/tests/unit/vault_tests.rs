use qudag_vault_core::*;
use tempfile::TempDir;

#[cfg(test)]
mod vault_lifecycle_tests {
    use super::*;

    #[test]
    fn test_vault_creation() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("test_vault.qdag");
        let master_password = "TestPassword123!@#";

        let result = Vault::create(vault_path.to_str().unwrap(), master_password);

        assert!(result.is_ok(), "Vault creation should succeed");
        let vault = result.unwrap();

        // Verify vault properties
        assert!(vault_path.exists(), "Vault file should be created");
        assert!(vault.is_locked() == false, "New vault should be unlocked");
        assert_eq!(vault.secret_count(), 0, "New vault should have no secrets");
    }

    #[test]
    fn test_vault_open_with_correct_password() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("test_vault.qdag");
        let master_password = "CorrectHorseBatteryStaple";

        // Create vault
        let _vault = Vault::create(vault_path.to_str().unwrap(), master_password).unwrap();
        drop(_vault); // Close vault

        // Open vault
        let result = Vault::open(vault_path.to_str().unwrap(), master_password);
        assert!(
            result.is_ok(),
            "Opening vault with correct password should succeed"
        );
    }

    #[test]
    fn test_vault_open_with_incorrect_password() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("test_vault.qdag");
        let master_password = "CorrectPassword";
        let wrong_password = "WrongPassword";

        // Create vault
        let _vault = Vault::create(vault_path.to_str().unwrap(), master_password).unwrap();
        drop(_vault);

        // Try to open with wrong password
        let result = Vault::open(vault_path.to_str().unwrap(), wrong_password);
        assert!(
            result.is_err(),
            "Opening vault with incorrect password should fail"
        );

        match result.unwrap_err() {
            VaultError::AuthenticationFailed => (),
            other => panic!("Expected AuthenticationFailed, got {:?}", other),
        }
    }

    #[test]
    fn test_vault_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("test_vault.qdag");
        let master_password = "PersistenceTest123";

        // Create vault and add secrets
        {
            let mut vault = Vault::create(vault_path.to_str().unwrap(), master_password).unwrap();
            vault
                .add_secret("test/email", "user@example.com", Some("password123"))
                .unwrap();
            vault.add_secret("test/github", "testuser", None).unwrap(); // Auto-generate password
        }

        // Reopen vault and verify secrets
        {
            let vault = Vault::open(vault_path.to_str().unwrap(), master_password).unwrap();
            assert_eq!(
                vault.secret_count(),
                2,
                "Vault should have 2 secrets after reopening"
            );

            let email_secret = vault.get_secret("test/email").unwrap();
            assert_eq!(email_secret.username, "user@example.com");
            assert_eq!(email_secret.password, "password123");

            let github_secret = vault.get_secret("test/github").unwrap();
            assert_eq!(github_secret.username, "testuser");
            assert!(
                !github_secret.password.is_empty(),
                "Auto-generated password should not be empty"
            );
        }
    }
}

#[cfg(test)]
mod secret_management_tests {
    use super::*;

    #[test]
    fn test_add_and_retrieve_secret() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("test_vault.qdag");
        let mut vault = Vault::create(vault_path.to_str().unwrap(), "TestPassword").unwrap();

        // Add secret
        let result = vault.add_secret("email/personal", "alice@example.com", Some("SecurePass123"));
        assert!(result.is_ok(), "Adding secret should succeed");

        // Retrieve secret
        let secret = vault.get_secret("email/personal").unwrap();
        assert_eq!(secret.label, "email/personal");
        assert_eq!(secret.username, "alice@example.com");
        assert_eq!(secret.password, "SecurePass123");
    }

    #[test]
    fn test_add_secret_with_generated_password() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("test_vault.qdag");
        let mut vault = Vault::create(vault_path.to_str().unwrap(), "TestPassword").unwrap();

        // Add secret without password (should generate)
        let result = vault.add_secret("server/production", "admin", None);
        assert!(
            result.is_ok(),
            "Adding secret with generated password should succeed"
        );

        // Retrieve and verify generated password
        let secret = vault.get_secret("server/production").unwrap();
        assert_eq!(secret.username, "admin");
        assert!(
            secret.password.len() >= 16,
            "Generated password should be at least 16 chars"
        );
        assert!(
            has_uppercase(&secret.password),
            "Generated password should have uppercase"
        );
        assert!(
            has_lowercase(&secret.password),
            "Generated password should have lowercase"
        );
        assert!(
            has_digit(&secret.password),
            "Generated password should have digits"
        );
        assert!(
            has_special(&secret.password),
            "Generated password should have special chars"
        );
    }

    #[test]
    fn test_update_existing_secret() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("test_vault.qdag");
        let mut vault = Vault::create(vault_path.to_str().unwrap(), "TestPassword").unwrap();

        // Add initial secret
        vault
            .add_secret("api/service", "api_user", Some("OldPassword"))
            .unwrap();

        // Update secret
        let result = vault.update_secret("api/service", "api_user", Some("NewPassword"));
        assert!(result.is_ok(), "Updating secret should succeed");

        // Verify update
        let secret = vault.get_secret("api/service").unwrap();
        assert_eq!(secret.password, "NewPassword");
    }

    #[test]
    fn test_delete_secret() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("test_vault.qdag");
        let mut vault = Vault::create(vault_path.to_str().unwrap(), "TestPassword").unwrap();

        // Add and then delete secret
        vault
            .add_secret("temp/secret", "temp_user", Some("TempPass"))
            .unwrap();
        assert_eq!(vault.secret_count(), 1);

        let result = vault.delete_secret("temp/secret");
        assert!(result.is_ok(), "Deleting secret should succeed");
        assert_eq!(vault.secret_count(), 0);

        // Try to retrieve deleted secret
        let result = vault.get_secret("temp/secret");
        assert!(result.is_err(), "Getting deleted secret should fail");
    }

    #[test]
    fn test_list_secrets() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("test_vault.qdag");
        let mut vault = Vault::create(vault_path.to_str().unwrap(), "TestPassword").unwrap();

        // Add multiple secrets
        vault
            .add_secret("email/work", "work@company.com", Some("WorkPass"))
            .unwrap();
        vault
            .add_secret("email/personal", "me@personal.com", Some("PersonalPass"))
            .unwrap();
        vault
            .add_secret("social/twitter", "@handle", Some("TwitterPass"))
            .unwrap();
        vault
            .add_secret("social/linkedin", "profile", Some("LinkedInPass"))
            .unwrap();

        // List all secrets
        let all_secrets = vault.list_secrets(None).unwrap();
        assert_eq!(all_secrets.len(), 4);
        assert!(all_secrets.contains(&"email/work".to_string()));
        assert!(all_secrets.contains(&"email/personal".to_string()));
        assert!(all_secrets.contains(&"social/twitter".to_string()));
        assert!(all_secrets.contains(&"social/linkedin".to_string()));

        // List by category
        let email_secrets = vault.list_secrets(Some("email")).unwrap();
        assert_eq!(email_secrets.len(), 2);
        assert!(email_secrets.contains(&"email/work".to_string()));
        assert!(email_secrets.contains(&"email/personal".to_string()));

        let social_secrets = vault.list_secrets(Some("social")).unwrap();
        assert_eq!(social_secrets.len(), 2);
        assert!(social_secrets.contains(&"social/twitter".to_string()));
        assert!(social_secrets.contains(&"social/linkedin".to_string()));
    }
}

#[cfg(test)]
mod dag_structure_tests {
    use super::*;

    #[test]
    fn test_dag_node_relationships() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("test_vault.qdag");
        let mut vault = Vault::create(vault_path.to_str().unwrap(), "TestPassword").unwrap();

        // Create category nodes
        vault.create_category("email").unwrap();
        vault.create_category("work").unwrap();
        vault.create_category("work/projects").unwrap();

        // Add secrets with DAG relationships
        vault
            .add_secret_to_category("email/gmail", "user@gmail.com", Some("pass"), "email")
            .unwrap();
        vault
            .add_secret_to_category("work/gitlab", "dev@company.com", Some("pass"), "work")
            .unwrap();
        vault
            .add_secret_to_category(
                "work/jira",
                "dev@company.com",
                Some("pass"),
                "work/projects",
            )
            .unwrap();

        // Verify DAG structure
        let email_children = vault.get_category_children("email").unwrap();
        assert_eq!(email_children.len(), 1);
        assert!(email_children.contains(&"email/gmail".to_string()));

        let work_children = vault.get_category_children("work").unwrap();
        assert_eq!(work_children.len(), 2); // gitlab and projects folder

        let project_children = vault.get_category_children("work/projects").unwrap();
        assert_eq!(project_children.len(), 1);
        assert!(project_children.contains(&"work/jira".to_string()));
    }

    #[test]
    fn test_dag_traversal() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("test_vault.qdag");
        let mut vault = Vault::create(vault_path.to_str().unwrap(), "TestPassword").unwrap();

        // Build a DAG structure
        vault.create_category("root").unwrap();
        vault.create_category("root/a").unwrap();
        vault.create_category("root/b").unwrap();
        vault
            .add_secret_to_category("secret1", "user1", Some("pass1"), "root/a")
            .unwrap();
        vault
            .add_secret_to_category("secret2", "user2", Some("pass2"), "root/b")
            .unwrap();

        // Traverse from root
        let all_descendants = vault.traverse_dag_from("root").unwrap();
        assert!(all_descendants.len() >= 4); // root, a, b, secret1, secret2
    }

    #[test]
    fn test_dag_version_history() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("test_vault.qdag");
        let mut vault = Vault::create(vault_path.to_str().unwrap(), "TestPassword").unwrap();

        // Add secret and create versions
        vault
            .add_secret("app/database", "dbuser", Some("version1"))
            .unwrap();
        vault
            .update_secret_with_version("app/database", "dbuser", Some("version2"))
            .unwrap();
        vault
            .update_secret_with_version("app/database", "dbuser", Some("version3"))
            .unwrap();

        // Get version history
        let history = vault.get_secret_history("app/database").unwrap();
        assert_eq!(history.len(), 3);

        // Verify we can access specific versions
        let v1 = vault.get_secret_version("app/database", 0).unwrap();
        assert_eq!(v1.password, "version1");

        let v3 = vault.get_secret_version("app/database", 2).unwrap();
        assert_eq!(v3.password, "version3");
    }
}

// Helper functions for password validation
fn has_uppercase(s: &str) -> bool {
    s.chars().any(|c| c.is_uppercase())
}

fn has_lowercase(s: &str) -> bool {
    s.chars().any(|c| c.is_lowercase())
}

fn has_digit(s: &str) -> bool {
    s.chars().any(|c| c.is_numeric())
}

fn has_special(s: &str) -> bool {
    s.chars().any(|c| !c.is_alphanumeric())
}
