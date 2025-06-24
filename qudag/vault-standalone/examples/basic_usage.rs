//! Basic usage example for qudag-vault

use qudag_vault_core::{
    vault::Vault,
    error::VaultResult,
};
use tempfile;

fn main() -> VaultResult<()> {
    // Create a temporary directory and file path for demonstration
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let vault_path = temp_dir.path().join("test_vault.qdag");
    
    // Create a new vault
    let mut vault = Vault::create(vault_path, "master_password")?;
    
    println!("Created new vault!");
    
    // Add secrets to vault
    vault.add_secret("github", "myusername", Some("ghp_secrettoken123"))?;
    vault.add_secret("email", "user@example.com", None)?; // Auto-generates password
    vault.add_secret("work/gitlab", "work_account", None)?;
    
    println!("Added 3 secrets to vault");
    
    // Retrieve a secret
    let github_secret = vault.get_secret("github")?;
    println!("\nGitHub credentials:");
    println!("  Username: {}", github_secret.username);
    println!("  Password: {}", github_secret.password.as_str());
    
    // List all secrets
    println!("\nAll secrets in vault:");
    let all_secrets = vault.list_secrets(None)?;
    for label in &all_secrets {
        println!("  - {}", label);
    }
    
    println!("\nVault operations completed successfully!");
    
    Ok(())
}