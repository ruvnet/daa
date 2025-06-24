//! Initialize command implementation

use anyhow::Result;
use colorful::Colorful;
use std::path::PathBuf;

use crate::Cli;

/// Handle the init command
pub async fn handle_init(
    directory: Option<PathBuf>,
    template: String,
    force: bool,
    cli: &Cli,
) -> Result<()> {
    let target_dir = directory.unwrap_or_else(|| std::env::current_dir().unwrap());
    
    if cli.verbose {
        println!("Initializing DAA configuration in: {}", target_dir.display());
        println!("Template: {}", template);
        println!("Force: {}", force);
    }

    // Create configuration directory
    let config_dir = target_dir.join(".daa");
    if config_dir.exists() && !force {
        anyhow::bail!("DAA configuration already exists. Use --force to overwrite.");
    }

    std::fs::create_dir_all(&config_dir)?;

    // Create default orchestrator configuration
    let orchestrator_config = daa_orchestrator::OrchestratorConfig::default();
    let orchestrator_config_path = config_dir.join("orchestrator.toml");
    orchestrator_config.to_file(&orchestrator_config_path)?;

    // Create CLI configuration
    let cli_config = crate::config::CliConfig::default();
    let cli_config_path = config_dir.join("config.toml");
    cli_config.to_file(&cli_config_path)?;

    // Create .gitignore
    let gitignore_content = r#"# DAA runtime files
*.pid
*.log
data/
tmp/
"#;
    std::fs::write(config_dir.join(".gitignore"), gitignore_content)?;

    if cli.json {
        println!("{}", serde_json::json!({
            "status": "success",
            "directory": target_dir,
            "template": template,
            "files_created": [
                orchestrator_config_path.display().to_string(),
                cli_config_path.display().to_string(),
                config_dir.join(".gitignore").display().to_string()
            ]
        }));
    } else {
        println!("{}", "✓ DAA configuration initialized successfully".green());
        println!("  Directory: {}", target_dir.display());
        println!("  Template: {}", template);
        println!("  Configuration files created:");
        println!("    - {}", orchestrator_config_path.display());
        println!("    - {}", cli_config_path.display());
        println!("    - {}", config_dir.join(".gitignore").display());
        println!();
        println!("Next steps:");
        println!("  1. Edit the configuration files as needed");
        println!("  2. Run 'daa start' to start the orchestrator");
    }

    Ok(())
}