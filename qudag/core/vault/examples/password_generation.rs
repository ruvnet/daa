//! Password generation example for qudag-vault

use qudag_vault_core::{
    utils::{generate_password, CharacterSet},
    VaultResult,
};

fn main() -> VaultResult<()> {
    println!("QuDAG Vault - Password Generation Examples\n");

    // Generate different types of passwords
    let simple_password = generate_password(12, CharacterSet::Lowercase);
    println!("Simple (lowercase only): {}", simple_password);

    let alphanumeric = generate_password(16, CharacterSet::Alphanumeric);
    println!("Alphanumeric: {}", alphanumeric);

    let complex = generate_password(20, CharacterSet::All);
    println!("Complex (with symbols): {}", complex);

    // Generate multiple passwords for comparison
    println!("\nBatch generation (5 passwords, 16 chars each):");
    for i in 1..=5 {
        let password = generate_password(16, CharacterSet::All);
        println!("  {}. {}", i, password);
    }

    // Generate passwords of different lengths
    println!("\nDifferent lengths:");
    for length in [8, 12, 16, 24, 32] {
        let password = generate_password(length, CharacterSet::All);
        println!("  {} chars: {}", length, password);
    }

    Ok(())
}
