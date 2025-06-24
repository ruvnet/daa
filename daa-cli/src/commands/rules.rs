//! Rules command implementation

use anyhow::Result;
use colorful::Colorful;

use crate::{Cli, config::CliConfig};

/// Handle the add-rule command
pub async fn handle_add_rule(
    name: String,
    rule_type: String,
    params: Option<String>,
    description: Option<String>,
    config: &CliConfig,
    cli: &Cli,
) -> Result<()> {
    if cli.verbose {
        println!("Adding rule: {}", name);
        println!("Type: {}", rule_type);
        println!("Params: {:?}", params);
        println!("Description: {:?}", description);
    }

    // Mock rule addition
    let rule_id = uuid::Uuid::new_v4().to_string();

    if cli.json {
        println!("{}", serde_json::json!({
            "status": "added",
            "rule_id": rule_id,
            "name": name,
            "rule_type": rule_type,
            "params": params,
            "description": description
        }));
    } else {
        println!("{}", "✓ Rule added successfully".green());
        println!("Rule ID: {}", rule_id);
        println!("Name: {}", name);
        println!("Type: {}", rule_type);
        if let Some(desc) = description {
            println!("Description: {}", desc);
        }
    }

    Ok(())
}