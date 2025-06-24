use std::process::Command;
use tempfile::TempDir;
use std::fs;

#[cfg(test)]
mod vault_cli_tests {
    use super::*;

    fn run_qudag_command(args: &[&str]) -> (String, String, bool) {
        let output = Command::new("./qudag")
            .args(args)
            .output()
            .expect("Failed to execute command");

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let success = output.status.success();

        (stdout, stderr, success)
    }

    #[test]
    fn test_vault_init_command() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("test_vault.qdag");

        // Test vault init
        let (stdout, stderr, success) = run_qudag_command(&[
            "vault",
            "init",
            vault_path.to_str().unwrap(),
            "--password",
            "TestPassword123!"
        ]);

        assert!(success, "Vault init should succeed: stderr={}", stderr);
        assert!(stdout.contains("Vault initialized"), "Should show success message");
        assert!(vault_path.exists(), "Vault file should be created");
    }

    #[test]
    fn test_vault_init_interactive() {
        // Test interactive password prompt (would need PTY in real test)
        // For now, test that it fails without password
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("test_vault.qdag");

        let (_, stderr, success) = run_qudag_command(&[
            "vault",
            "init",
            vault_path.to_str().unwrap()
        ]);

        assert!(!success, "Should fail without password");
        assert!(stderr.contains("password") || stderr.contains("required"),
                "Should mention password requirement");
    }

    #[test]
    fn test_vault_add_secret() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("test_vault.qdag");

        // Initialize vault first
        run_qudag_command(&[
            "vault",
            "init",
            vault_path.to_str().unwrap(),
            "--password",
            "TestPassword123!"
        ]);

        // Add secret
        let (stdout, _, success) = run_qudag_command(&[
            "vault",
            "add",
            "email/gmail",
            "--vault",
            vault_path.to_str().unwrap(),
            "--password",
            "TestPassword123!",
            "--username",
            "user@gmail.com",
            "--secret-password",
            "GmailPassword123"
        ]);

        assert!(success, "Adding secret should succeed");
        assert!(stdout.contains("Secret") && stdout.contains("added"),
                "Should confirm secret was added");
    }

    #[test]
    fn test_vault_add_with_generated_password() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("test_vault.qdag");

        // Initialize vault
        run_qudag_command(&[
            "vault",
            "init",
            vault_path.to_str().unwrap(),
            "--password",
            "TestPassword123!"
        ]);

        // Add secret with generated password
        let (stdout, _, success) = run_qudag_command(&[
            "vault",
            "add",
            "server/prod",
            "--vault",
            vault_path.to_str().unwrap(),
            "--password",
            "TestPassword123!",
            "--username",
            "admin",
            "--generate"
        ]);

        assert!(success, "Adding secret with generated password should succeed");
        assert!(stdout.contains("Generated password"),
                "Should show generated password");
    }

    #[test]
    fn test_vault_get_secret() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("test_vault.qdag");

        // Initialize and add secret
        run_qudag_command(&[
            "vault",
            "init",
            vault_path.to_str().unwrap(),
            "--password",
            "TestPassword123!"
        ]);

        run_qudag_command(&[
            "vault",
            "add",
            "test/secret",
            "--vault",
            vault_path.to_str().unwrap(),
            "--password",
            "TestPassword123!",
            "--username",
            "testuser",
            "--secret-password",
            "TestSecretPass"
        ]);

        // Get secret
        let (stdout, _, success) = run_qudag_command(&[
            "vault",
            "get",
            "test/secret",
            "--vault",
            vault_path.to_str().unwrap(),
            "--password",
            "TestPassword123!"
        ]);

        assert!(success, "Getting secret should succeed");
        assert!(stdout.contains("testuser"), "Should show username");
        assert!(stdout.contains("TestSecretPass"), "Should show password");
    }

    #[test]
    fn test_vault_get_json_output() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("test_vault.qdag");

        // Setup vault with secret
        run_qudag_command(&[
            "vault",
            "init",
            vault_path.to_str().unwrap(),
            "--password",
            "TestPassword123!"
        ]);

        run_qudag_command(&[
            "vault",
            "add",
            "api/key",
            "--vault",
            vault_path.to_str().unwrap(),
            "--password",
            "TestPassword123!",
            "--username",
            "api_service",
            "--secret-password",
            "api_key_12345"
        ]);

        // Get secret as JSON
        let (stdout, _, success) = run_qudag_command(&[
            "vault",
            "get",
            "api/key",
            "--vault",
            vault_path.to_str().unwrap(),
            "--password",
            "TestPassword123!",
            "--format",
            "json"
        ]);

        assert!(success, "Getting secret as JSON should succeed");
        
        // Parse JSON output
        let json: serde_json::Value = serde_json::from_str(&stdout)
            .expect("Output should be valid JSON");
        
        assert_eq!(json["label"], "api/key");
        assert_eq!(json["username"], "api_service");
        assert_eq!(json["password"], "api_key_12345");
    }

    #[test]
    fn test_vault_list_secrets() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("test_vault.qdag");

        // Setup vault with multiple secrets
        run_qudag_command(&[
            "vault",
            "init",
            vault_path.to_str().unwrap(),
            "--password",
            "TestPassword123!"
        ]);

        let secrets = vec![
            ("email/personal", "personal@example.com"),
            ("email/work", "work@company.com"),
            ("social/twitter", "@handle"),
            ("social/linkedin", "profile"),
        ];

        for (label, username) in &secrets {
            run_qudag_command(&[
                "vault",
                "add",
                label,
                "--vault",
                vault_path.to_str().unwrap(),
                "--password",
                "TestPassword123!",
                "--username",
                username,
                "--generate"
            ]);
        }

        // List all secrets
        let (stdout, _, success) = run_qudag_command(&[
            "vault",
            "list",
            "--vault",
            vault_path.to_str().unwrap(),
            "--password",
            "TestPassword123!"
        ]);

        assert!(success, "Listing secrets should succeed");
        for (label, _) in &secrets {
            assert!(stdout.contains(label), "Should list secret: {}", label);
        }

        // List by category
        let (stdout, _, success) = run_qudag_command(&[
            "vault",
            "list",
            "--vault",
            vault_path.to_str().unwrap(),
            "--password",
            "TestPassword123!",
            "--category",
            "email"
        ]);

        assert!(success, "Listing by category should succeed");
        assert!(stdout.contains("email/personal"));
        assert!(stdout.contains("email/work"));
        assert!(!stdout.contains("social/twitter"));
    }

    #[test]
    fn test_vault_export_import() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("test_vault.qdag");
        let export_path = temp_dir.path().join("export.qdag");
        let import_vault_path = temp_dir.path().join("import_vault.qdag");

        // Create vault with secrets
        run_qudag_command(&[
            "vault",
            "init",
            vault_path.to_str().unwrap(),
            "--password",
            "TestPassword123!"
        ]);

        run_qudag_command(&[
            "vault",
            "add",
            "export/test",
            "--vault",
            vault_path.to_str().unwrap(),
            "--password",
            "TestPassword123!",
            "--username",
            "exportuser",
            "--secret-password",
            "ExportPass123"
        ]);

        // Export vault
        let (stdout, _, success) = run_qudag_command(&[
            "vault",
            "export",
            export_path.to_str().unwrap(),
            "--vault",
            vault_path.to_str().unwrap(),
            "--password",
            "TestPassword123!"
        ]);

        assert!(success, "Export should succeed");
        assert!(export_path.exists(), "Export file should be created");
        assert!(stdout.contains("exported"), "Should confirm export");

        // Create new vault and import
        run_qudag_command(&[
            "vault",
            "init",
            import_vault_path.to_str().unwrap(),
            "--password",
            "ImportPassword"
        ]);

        let (stdout, _, success) = run_qudag_command(&[
            "vault",
            "import",
            export_path.to_str().unwrap(),
            "--vault",
            import_vault_path.to_str().unwrap(),
            "--password",
            "ImportPassword"
        ]);

        assert!(success, "Import should succeed");
        assert!(stdout.contains("imported"), "Should confirm import");

        // Verify imported secret
        let (stdout, _, success) = run_qudag_command(&[
            "vault",
            "get",
            "export/test",
            "--vault",
            import_vault_path.to_str().unwrap(),
            "--password",
            "ImportPassword"
        ]);

        assert!(success, "Getting imported secret should succeed");
        assert!(stdout.contains("exportuser"));
        assert!(stdout.contains("ExportPass123"));
    }

    #[test]
    fn test_vault_generate_password_command() {
        let (stdout, _, success) = run_qudag_command(&[
            "vault",
            "genpw",
            "--length",
            "24",
            "--no-symbols"
        ]);

        assert!(success, "Password generation should succeed");
        
        let password = stdout.trim();
        assert_eq!(password.len(), 24, "Generated password should be 24 chars");
        assert!(password.chars().all(|c| c.is_alphanumeric()),
                "Password should only contain alphanumeric chars");
    }

    #[test]
    fn test_vault_error_handling() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("test_vault.qdag");

        // Try to open non-existent vault
        let (_, stderr, success) = run_qudag_command(&[
            "vault",
            "get",
            "test",
            "--vault",
            vault_path.to_str().unwrap(),
            "--password",
            "TestPassword"
        ]);

        assert!(!success, "Should fail for non-existent vault");
        assert!(stderr.contains("not found") || stderr.contains("does not exist"),
                "Should show appropriate error");

        // Create vault and try wrong password
        run_qudag_command(&[
            "vault",
            "init",
            vault_path.to_str().unwrap(),
            "--password",
            "CorrectPassword"
        ]);

        let (_, stderr, success) = run_qudag_command(&[
            "vault",
            "list",
            "--vault",
            vault_path.to_str().unwrap(),
            "--password",
            "WrongPassword"
        ]);

        assert!(!success, "Should fail with wrong password");
        assert!(stderr.contains("authentication") || stderr.contains("incorrect"),
                "Should show authentication error");
    }

    #[test]
    fn test_vault_help_command() {
        let (stdout, _, success) = run_qudag_command(&["vault", "--help"]);

        assert!(success, "Help command should succeed");
        assert!(stdout.contains("vault"), "Should show vault in help");
        assert!(stdout.contains("init"), "Should show init command");
        assert!(stdout.contains("add"), "Should show add command");
        assert!(stdout.contains("get"), "Should show get command");
        assert!(stdout.contains("list"), "Should show list command");
        assert!(stdout.contains("export"), "Should show export command");
        assert!(stdout.contains("import"), "Should show import command");
    }
}