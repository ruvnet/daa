//! MCP configuration management

use crate::CliError;
use qudag_mcp::McpConfig;
use std::path::PathBuf;
use tracing::info;

/// Show current MCP configuration
pub async fn show_mcp_config() -> Result<(), CliError> {
    info!("Showing MCP configuration");

    println!("MCP Server Configuration");
    println!("========================");

    // Try to load existing config
    let config_path = get_default_config_path()?;

    if config_path.exists() {
        println!("Config file: {:?}", config_path);

        match McpConfig::from_file(&config_path) {
            Ok(config) => {
                println!();
                display_config(&config);
            }
            Err(e) => {
                println!("Error loading config: {}", e);
                println!();
                println!("Using default configuration:");
                display_config(&McpConfig::default());
            }
        }
    } else {
        println!("Config file: Not found");
        println!();
        println!("Default configuration:");
        display_config(&McpConfig::default());
        println!();
        println!("To create a config file, run: qudag mcp config init");
    }

    Ok(())
}

/// Initialize MCP configuration file
pub async fn init_mcp_config(output: Option<PathBuf>, force: bool) -> Result<(), CliError> {
    info!("Initializing MCP configuration");

    let config_path = output.unwrap_or_else(|| get_default_config_path().unwrap());

    // Check if file already exists
    if config_path.exists() && !force {
        return Err(CliError::Config(format!(
            "Config file already exists at {:?}. Use --force to overwrite.",
            config_path
        )));
    }

    // Create parent directory if it doesn't exist
    if let Some(parent) = config_path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| CliError::Config(format!("Failed to create config directory: {}", e)))?;
    }

    // Create default config
    let config = create_default_config();

    // Save config to file
    config
        .save_to_file(&config_path)
        .map_err(|e| CliError::Config(format!("Failed to save config file: {}", e)))?;

    println!("✓ MCP configuration initialized at {:?}", config_path);
    println!();
    display_config(&config);
    println!();
    println!("You can now:");
    println!("  - Edit the config file manually");
    println!("  - Start the MCP server: qudag mcp start");
    println!(
        "  - Validate the config: qudag mcp config validate {:?}",
        config_path
    );

    Ok(())
}

/// Validate MCP configuration file
pub async fn validate_mcp_config(config_path: PathBuf) -> Result<(), CliError> {
    info!("Validating MCP configuration at {:?}", config_path);

    println!("Validating MCP Configuration");
    println!("=============================");
    println!("Config file: {:?}", config_path);
    println!();

    if !config_path.exists() {
        return Err(CliError::Config(format!(
            "Config file not found: {:?}",
            config_path
        )));
    }

    // Try to load and validate config
    match McpConfig::from_file(&config_path) {
        Ok(config) => {
            println!("✓ Config file syntax is valid");

            // Validate configuration
            match config.validate() {
                Ok(()) => {
                    println!("✓ Configuration validation passed");
                    println!();

                    // Show validation summary
                    println!("Configuration Summary:");
                    println!("  Server Host: {}", config.server.host);
                    println!("  Server Port: {}", config.server.port);
                    println!("  Max Connections: {}", config.server.max_connections);
                    println!(
                        "  Request Timeout: {}s",
                        config.server.request_timeout.as_secs()
                    );
                    println!("  TLS Enabled: {}", config.server.tls_enabled);
                    println!("  Auth Vault: {:?}", config.auth.vault_path);
                    println!("  MFA Enabled: {}", config.auth.mfa_enabled);
                    println!("  RBAC Enabled: {}", config.auth.rbac_enabled);
                    println!("  Rate Limiting: {}", config.rate_limit.enabled);
                    println!("  Audit Logging: {}", config.audit.enabled);

                    // Validate network connectivity if possible
                    println!();
                    println!("Network Validation:");

                    let bind_addr = format!("{}:{}", config.server.host, config.server.port);
                    if let Ok(addr) = bind_addr.parse::<std::net::SocketAddr>() {
                        if let Err(e) = validate_bind_address(&addr).await {
                            println!("⚠ Bind address warning: {}", e);
                        } else {
                            println!("✓ Bind address is available");
                        }
                    } else {
                        println!("⚠ Invalid bind address format: {}", bind_addr);
                    }

                    if config.server.tls_enabled {
                        match validate_tls_config(&config.server).await {
                            Ok(()) => println!("✓ TLS configuration is valid"),
                            Err(e) => println!("⚠ TLS configuration warning: {}", e),
                        }
                    }

                    println!();
                    println!("✓ Configuration validation completed successfully");
                }
                Err(e) => {
                    println!("✗ Configuration validation failed: {}", e);
                    return Err(CliError::Config(format!("Invalid configuration: {}", e)));
                }
            }
        }
        Err(e) => {
            println!("✗ Config file syntax error: {}", e);
            println!();
            println!("Common issues:");
            println!("  - Check TOML syntax (brackets, quotes, etc.)");
            println!("  - Verify all required fields are present");
            println!("  - Check data types match expected values");
            return Err(CliError::Config(format!("Invalid config file: {}", e)));
        }
    }

    Ok(())
}

// Helper functions

fn display_config(config: &McpConfig) {
    println!("Server Settings:");
    println!("  Host: {}", config.server.host);
    println!("  Port: {}", config.server.port);
    println!("  Max Connections: {}", config.server.max_connections);
    println!(
        "  Request Timeout: {}s",
        config.server.request_timeout.as_secs()
    );
    println!(
        "  Keep Alive Timeout: {}s",
        config.server.keep_alive_timeout.as_secs()
    );
    println!();

    println!("TLS Settings:");
    println!("  TLS Enabled: {}", config.server.tls_enabled);
    if let Some(cert_path) = &config.server.tls_cert_path {
        println!("  TLS Cert: {:?}", cert_path);
    }
    if let Some(key_path) = &config.server.tls_key_path {
        println!("  TLS Key: {:?}", key_path);
    }
    println!();

    println!("Authentication:");
    println!("  Vault Path: {:?}", config.auth.vault_path);
    println!(
        "  JWT Expiration: {}s",
        config.auth.jwt_expiration.as_secs()
    );
    println!("  MFA Enabled: {}", config.auth.mfa_enabled);
    println!("  RBAC Enabled: {}", config.auth.rbac_enabled);
    println!(
        "  Session Timeout: {}s",
        config.auth.session_timeout.as_secs()
    );
    println!("  Max Login Attempts: {}", config.auth.max_login_attempts);
    println!();

    println!("Security:");
    println!(
        "  Encryption Key Size: {} bytes",
        config.security.encryption_key_size
    );
    println!("  Request Signing: {}", config.security.request_signing);
    println!(
        "  Response Encryption: {}",
        config.security.response_encryption
    );
    println!(
        "  Min Password Strength: {}",
        config.security.min_password_strength
    );
    println!("  Secure Headers: {}", config.security.secure_headers);
    println!("  CORS Origins: {:?}", config.security.cors_origins);
    println!();

    println!("Rate Limiting:");
    println!("  Enabled: {}", config.rate_limit.enabled);
    if config.rate_limit.enabled {
        println!("  Max Requests: {}", config.rate_limit.max_requests);
        println!(
            "  Window Duration: {}s",
            config.rate_limit.window_duration.as_secs()
        );
        println!("  Burst Size: {}", config.rate_limit.burst_size);
    }
    println!();

    println!("Audit Logging:");
    println!("  Enabled: {}", config.audit.enabled);
    if config.audit.enabled {
        println!("  Log File: {:?}", config.audit.log_file);
        println!(
            "  Rotation Size: {} MB",
            config.audit.rotation_size / 1024 / 1024
        );
        println!(
            "  Retention: {} days",
            config.audit.retention_duration.as_secs() / 86400
        );
    }
    println!();

    println!("Storage:");
    println!("  Base Directory: {:?}", config.storage.base_dir);
    println!("  Encrypt at Rest: {}", config.storage.encrypt_at_rest);
    println!("  Backup Enabled: {}", config.storage.backup.enabled);
    if config.storage.backup.enabled {
        println!("  Backup Directory: {:?}", config.storage.backup.directory);
        println!(
            "  Backup Interval: {}h",
            config.storage.backup.interval.as_secs() / 3600
        );
        println!("  Retain Count: {}", config.storage.backup.retain_count);
    }
}

fn create_default_config() -> McpConfig {
    McpConfig::default()
}

fn get_default_config_path() -> Result<PathBuf, CliError> {
    let home = std::env::var("HOME")
        .map_err(|_| CliError::Config("Unable to determine home directory".to_string()))?;
    Ok(PathBuf::from(home).join(".qudag").join("mcp-config.toml"))
}

async fn validate_bind_address(bind_addr: &std::net::SocketAddr) -> Result<(), String> {
    use tokio::net::TcpListener;

    // Try to bind to the address to check if it's available
    match TcpListener::bind(bind_addr).await {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Cannot bind to {}: {}", bind_addr, e)),
    }
}

async fn validate_tls_config(
    server_config: &qudag_mcp::config::ServerConfig,
) -> Result<(), String> {
    if let (Some(cert_path), Some(key_path)) =
        (&server_config.tls_cert_path, &server_config.tls_key_path)
    {
        if !cert_path.exists() {
            return Err(format!("TLS certificate file not found: {:?}", cert_path));
        }

        if !key_path.exists() {
            return Err(format!("TLS key file not found: {:?}", key_path));
        }

        // Additional TLS validation could be added here
        // (e.g., checking certificate validity, key format, etc.)

        Ok(())
    } else {
        Err("TLS enabled but certificate or key path not specified".to_string())
    }
}
