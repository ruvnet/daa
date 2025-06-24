//! Resource status command implementation

use crate::{config::CliConfig, output::OutputFormat};
use anyhow::Result;
use colored::Colorize;
use tabled::{Table, Tabled};

#[derive(Tabled)]
struct ResourceCost {
    #[tabled(rename = "Operation")]
    operation: String,
    #[tabled(rename = "Cost (rUv)")]
    cost: String,
    #[tabled(rename = "Description")]
    description: String,
}

pub async fn execute(
    detailed: bool,
    _config: &CliConfig,
    output: OutputFormat,
) -> Result<()> {
    println!("{}", "Resource Usage Status".cyan().bold());
    println!();
    
    // TODO: Get actual resource status from exchange
    // Mock data for now
    let total_compute = 1523;
    let total_storage = 847;
    let total_operations = 342;
    
    match output {
        OutputFormat::Text => {
            println!("{}", "Current Usage:".green());
            println!("  Compute Time: {} ms", total_compute.to_string().cyan());
            println!("  Storage Used: {} KB", total_storage.to_string().cyan());
            println!("  Operations:   {}", total_operations.to_string().cyan());
            println!();
            
            if detailed {
                println!("{}", "Resource Costs:".green());
                
                let costs = vec![
                    ResourceCost {
                        operation: "Create Account".to_string(),
                        cost: "10".to_string(),
                        description: "One-time account creation".to_string(),
                    },
                    ResourceCost {
                        operation: "Transfer".to_string(),
                        cost: "1".to_string(),
                        description: "Per transaction fee".to_string(),
                    },
                    ResourceCost {
                        operation: "Store Data".to_string(),
                        cost: "5/KB".to_string(),
                        description: "Per kilobyte stored".to_string(),
                    },
                    ResourceCost {
                        operation: "Compute".to_string(),
                        cost: "2/ms".to_string(),
                        description: "Per millisecond of computation".to_string(),
                    },
                ];
                
                let table = Table::new(costs).to_string();
                println!("{}", table);
                println!();
            }
            
            println!("{}", "rUv = Resource Utilization Voucher".dimmed());
            println!("{}", "Earn rUv by contributing resources to the network".dimmed());
        }
        OutputFormat::Json => {
            let result = serde_json::json!({
                "resource_usage": {
                    "compute_ms": total_compute,
                    "storage_kb": total_storage,
                    "operations": total_operations,
                },
                "costs": {
                    "create_account": 10,
                    "transfer": 1,
                    "store_data_per_kb": 5,
                    "compute_per_ms": 2,
                },
                "currency": {
                    "symbol": "rUv",
                    "name": "Resource Utilization Voucher",
                }
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }
    
    Ok(())
}