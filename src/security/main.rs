//! Main entry point for DAA security demonstration

use daa_security::{example::secure_federated_learning_round, example::local_differential_privacy_example};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("DAA Security System - Post-Quantum Secure Federated Learning");
    println!("=".repeat(60));
    
    // Run the comprehensive security example
    secure_federated_learning_round().await?;
    
    // Demonstrate local differential privacy
    local_differential_privacy_example();
    
    println!("\n✅ Security demonstration completed successfully!");
    
    Ok(())
}

// Export the security module
mod lib {
    pub mod security {
        pub mod aggregation;
        pub mod challenges;
        pub mod differential_privacy;
        pub mod integrity;
        pub mod staking;
        pub mod example;
        
        mod tests {
            use super::*;
            use crate::security::*;
            
            #[test]
            fn test_security_integration() {
                // Test that all components work together
                let config = SecurityConfig::default();
                let manager = SecurityManager::new(config);
                assert_eq!(manager.get_participants().len(), 0);
            }
        }
    }
}