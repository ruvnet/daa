# Phase 2: CLI Integration Implementation Details

## Overview
This document details the integration of the vault functionality into the existing QuDAG CLI framework, maintaining consistency with current command patterns and user experience.

## CLI Architecture Integration

### 1. Command Structure Updates

#### 1.1 Main CLI Extension (`tools/cli/src/main.rs`)
```rust
// Add to existing Commands enum
#[derive(Subcommand)]
enum Commands {
    // ... existing commands (Start, Stop, Peer, Network, Address) ...
    
    /// Quantum-resistant password vault management
    Vault {
        #[command(subcommand)]
        command: VaultCommands,
    },
}

// Add VaultCommands enum
#[derive(Subcommand)]
enum VaultCommands {
    /// Initialize a new password vault
    Init {
        /// Vault file path (default: ~/.qudag/vault.qdag)
        #[arg(short, long)]
        path: Option<PathBuf>,
        
        /// Skip password confirmation prompt
        #[arg(long)]
        no_confirm: bool,
        
        /// Enable post-quantum key exchange
        #[arg(long)]
        enable_kem: bool,
        
        /// Enable digital signatures
        #[arg(long)]
        enable_signing: bool,
    },
    
    /// Add a new secret to the vault
    Add {
        /// Secret label (e.g., "email/gmail", "server/prod-db")
        label: String,
        
        /// Username or identifier
        #[arg(short, long)]
        username: Option<String>,
        
        /// Password (prompted if not provided)
        #[arg(short, long)]
        password: Option<String>,
        
        /// Generate a random password
        #[arg(short, long)]
        generate: bool,
        
        /// Generated password length
        #[arg(long, default_value = "20")]
        length: usize,
        
        /// URL associated with the secret
        #[arg(long)]
        url: Option<String>,
        
        /// Category for organization
        #[arg(short, long)]
        category: Option<String>,
        
        /// Tags (comma-separated)
        #[arg(long)]
        tags: Option<String>,
        
        /// Additional notes
        #[arg(long)]
        notes: Option<String>,
        
        /// Custom fields in key=value format
        #[arg(long = "field")]
        fields: Vec<String>,
    },
    
    /// Retrieve a secret from the vault
    Get {
        /// Secret label or ID
        identifier: String,
        
        /// Copy password to clipboard
        #[arg(short, long)]
        copy: bool,
        
        /// Clear clipboard after N seconds
        #[arg(long, default_value = "30")]
        clear_after: u64,
        
        /// Show only specific field
        #[arg(long)]
        field: Option<String>,
        
        /// Output format
        #[arg(short, long, value_enum, default_value = "text")]
        format: OutputFormat,
    },
    
    /// List secrets in the vault
    List {
        /// Filter by category
        #[arg(short, long)]
        category: Option<String>,
        
        /// Search pattern (supports wildcards)
        #[arg(short, long)]
        search: Option<String>,
        
        /// Filter by tags
        #[arg(long)]
        tags: Option<String>,
        
        /// Sort by field
        #[arg(long, value_enum, default_value = "label")]
        sort: SortField,
        
        /// Reverse sort order
        #[arg(long)]
        reverse: bool,
        
        /// Show detailed information
        #[arg(long)]
        detailed: bool,
        
        /// Output format
        #[arg(short, long, value_enum, default_value = "table")]
        format: OutputFormat,
    },
    
    /// Update an existing secret
    Update {
        /// Secret identifier
        identifier: String,
        
        /// New username
        #[arg(long)]
        username: Option<String>,
        
        /// New password
        #[arg(long)]
        password: Option<String>,
        
        /// Generate new password
        #[arg(long)]
        generate: bool,
        
        /// New URL
        #[arg(long)]
        url: Option<String>,
        
        /// New category
        #[arg(long)]
        category: Option<String>,
        
        /// Add tags
        #[arg(long)]
        add_tags: Option<String>,
        
        /// Remove tags
        #[arg(long)]
        remove_tags: Option<String>,
        
        /// Update custom fields
        #[arg(long = "field")]
        fields: Vec<String>,
        
        /// Create new version (preserves history)
        #[arg(long)]
        version: bool,
    },
    
    /// Delete a secret from the vault
    Delete {
        /// Secret identifier
        identifier: String,
        
        /// Skip confirmation prompt
        #[arg(long)]
        force: bool,
        
        /// Permanently delete (no recovery)
        #[arg(long)]
        permanent: bool,
    },
    
    /// Export vault contents
    Export {
        /// Output file path
        output: PathBuf,
        
        /// Export format
        #[arg(long, value_enum, default_value = "encrypted")]
        format: ExportFormat,
        
        /// Include deleted items
        #[arg(long)]
        include_deleted: bool,
        
        /// Filter by category
        #[arg(long)]
        category: Option<String>,
        
        /// Encryption password for plain exports
        #[arg(long)]
        export_password: Option<String>,
    },
    
    /// Import secrets into vault
    Import {
        /// Input file path
        input: PathBuf,
        
        /// Import format
        #[arg(long, value_enum)]
        format: Option<ImportFormat>,
        
        /// Conflict resolution strategy
        #[arg(long, value_enum, default_value = "skip")]
        on_conflict: ConflictStrategy,
        
        /// Dry run (preview changes)
        #[arg(long)]
        dry_run: bool,
        
        /// Import password for encrypted files
        #[arg(long)]
        import_password: Option<String>,
    },
    
    /// Generate secure passwords
    Generate {
        /// Password length
        #[arg(default_value = "20")]
        length: usize,
        
        /// Number of passwords
        #[arg(short, long, default_value = "1")]
        count: usize,
        
        /// Include uppercase letters
        #[arg(long, default_value = "true")]
        uppercase: bool,
        
        /// Include lowercase letters
        #[arg(long, default_value = "true")]
        lowercase: bool,
        
        /// Include numbers
        #[arg(long, default_value = "true")]
        numbers: bool,
        
        /// Include symbols
        #[arg(long)]
        symbols: bool,
        
        /// Custom character set
        #[arg(long)]
        charset: Option<String>,
        
        /// Exclude ambiguous characters
        #[arg(long)]
        no_ambiguous: bool,
        
        /// Copy to clipboard
        #[arg(short, long)]
        copy: bool,
        
        /// Output format
        #[arg(short, long, value_enum, default_value = "text")]
        format: OutputFormat,
    },
    
    /// Manage vault configuration
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
    
    /// Show vault statistics
    Stats {
        /// Show category breakdown
        #[arg(long)]
        by_category: bool,
        
        /// Show age analysis
        #[arg(long)]
        by_age: bool,
        
        /// Show strength analysis
        #[arg(long)]
        strength: bool,
        
        /// Output format
        #[arg(short, long, value_enum, default_value = "table")]
        format: OutputFormat,
    },
}

#[derive(Subcommand)]
enum ConfigCommands {
    /// Show current configuration
    Show,
    
    /// Set configuration value
    Set {
        /// Configuration key
        key: String,
        
        /// Configuration value
        value: String,
    },
    
    /// Get configuration value
    Get {
        /// Configuration key
        key: String,
    },
    
    /// Reset to defaults
    Reset {
        /// Skip confirmation
        #[arg(long)]
        force: bool,
    },
}
```

### 2. Command Handler Implementation

#### 2.1 Vault Command Router (`tools/cli/src/vault/mod.rs`)
```rust
use qudag_vault_core::{Vault, SecretEntry, Filter, PasswordGenerator};
use rpassword::prompt_password;
use clipboard::{ClipboardContext, ClipboardProvider};
use std::time::Duration;
use tokio::time::sleep;

pub struct VaultCommandHandler {
    config: CliConfig,
    session: Option<VaultSession>,
}

struct VaultSession {
    vault: Vault,
    path: PathBuf,
    last_activity: std::time::Instant,
    timeout: Duration,
}

impl VaultCommandHandler {
    pub fn new(config: CliConfig) -> Self {
        Self {
            config,
            session: None,
        }
    }
    
    pub async fn handle(&mut self, command: VaultCommands) -> Result<(), CliError> {
        match command {
            VaultCommands::Init { path, no_confirm, enable_kem, enable_signing } => {
                self.handle_init(path, no_confirm, enable_kem, enable_signing).await
            }
            VaultCommands::Add { label, username, password, generate, length, url, category, tags, notes, fields } => {
                self.handle_add(AddParams {
                    label,
                    username,
                    password,
                    generate,
                    length,
                    url,
                    category,
                    tags,
                    notes,
                    fields,
                }).await
            }
            VaultCommands::Get { identifier, copy, clear_after, field, format } => {
                self.handle_get(identifier, copy, clear_after, field, format).await
            }
            VaultCommands::List { category, search, tags, sort, reverse, detailed, format } => {
                self.handle_list(ListParams {
                    category,
                    search,
                    tags,
                    sort,
                    reverse,
                    detailed,
                    format,
                }).await
            }
            // ... other command handlers ...
        }
    }
    
    async fn handle_init(
        &mut self,
        path: Option<PathBuf>,
        no_confirm: bool,
        enable_kem: bool,
        enable_signing: bool,
    ) -> Result<(), CliError> {
        let vault_path = path.unwrap_or_else(|| {
            self.config.data_dir.join("vault.qdag")
        });
        
        if vault_path.exists() {
            return Err(CliError::VaultExists(vault_path));
        }
        
        // Create parent directory if needed
        if let Some(parent) = vault_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        println!("Initializing new QuDAG vault at: {}", vault_path.display());
        println!();
        
        // Get master password
        let password = prompt_password("Enter master password: ")?;
        
        if !no_confirm {
            let confirm = prompt_password("Confirm master password: ")?;
            if password != confirm {
                return Err(CliError::PasswordMismatch);
            }
        }
        
        // Check password strength
        let strength = check_password_strength(&password);
        if strength < PasswordStrength::Good {
            println!("⚠️  Warning: Password strength is {}. Consider using a stronger password.", strength);
            if !no_confirm {
                let proceed = prompt_confirmation("Continue with this password?")?;
                if !proceed {
                    return Ok(());
                }
            }
        }
        
        // Create vault with options
        let mut builder = VaultBuilder::new();
        
        if enable_kem {
            println!("✓ Enabling post-quantum key exchange (Kyber)");
            builder = builder.with_kem();
        }
        
        if enable_signing {
            println!("✓ Enabling digital signatures (Dilithium)");
            builder = builder.with_signing();
        }
        
        let vault = builder.create(&vault_path, &password)?;
        
        println!();
        println!("✅ Vault initialized successfully!");
        println!();
        println!("Features enabled:");
        println!("  • AES-256-GCM encryption");
        println!("  • Argon2id key derivation");
        println!("  • BLAKE3 hashing");
        if enable_kem {
            println!("  • Kyber post-quantum KEM");
        }
        if enable_signing {
            println!("  • Dilithium signatures");
        }
        println!();
        println!("Next steps:");
        println!("  1. Add your first secret: qudag vault add \"email/gmail\"");
        println!("  2. List secrets: qudag vault list");
        println!("  3. Get help: qudag vault --help");
        
        Ok(())
    }
    
    async fn handle_add(&mut self, params: AddParams) -> Result<(), CliError> {
        let vault = self.open_vault().await?;
        
        // Prepare secret entry
        let mut entry = SecretEntry::new(params.label.clone());
        
        // Username
        entry.username = if let Some(username) = params.username {
            username
        } else {
            prompt_line("Username: ")?
        };
        
        // Password
        entry.password = if params.generate {
            let generator = PasswordGenerator::new()
                .length(params.length)
                .symbols(true)
                .no_ambiguous(true);
            
            let password = generator.generate();
            println!("Generated password: {}", password);
            
            if prompt_confirmation("Copy to clipboard?")? {
                self.copy_to_clipboard(&password)?;
                println!("✓ Password copied to clipboard");
            }
            
            SecureString::from(password)
        } else if let Some(password) = params.password {
            SecureString::from(password)
        } else {
            let password = prompt_password("Password (leave blank to generate): ")?;
            if password.is_empty() {
                let generator = PasswordGenerator::new().length(params.length);
                let generated = generator.generate();
                println!("Generated password: {}", generated);
                SecureString::from(generated)
            } else {
                SecureString::from(password)
            }
        };
        
        // Optional fields
        entry.url = params.url;
        entry.category = params.category;
        entry.notes = params.notes.map(SecureString::from);
        
        // Tags
        if let Some(tags) = params.tags {
            entry.tags = tags.split(',').map(|t| t.trim().to_string()).collect();
        }
        
        // Custom fields
        for field in params.fields {
            if let Some((key, value)) = field.split_once('=') {
                entry.custom_fields.insert(
                    key.trim().to_string(),
                    SecureString::from(value.trim())
                );
            }
        }
        
        // Add to vault
        let node_id = vault.add_secret(entry)?;
        vault.save()?;
        
        println!("✅ Secret '{}' added successfully", params.label);
        println!("   ID: {}", node_id);
        
        Ok(())
    }
    
    async fn handle_get(
        &mut self,
        identifier: String,
        copy: bool,
        clear_after: u64,
        field: Option<String>,
        format: OutputFormat,
    ) -> Result<(), CliError> {
        let vault = self.open_vault().await?;
        
        // Find secret
        let (node_id, entry) = self.find_secret(&vault, &identifier)?;
        
        if copy {
            // Copy password to clipboard
            self.copy_to_clipboard(entry.password.as_ref())?;
            println!("✓ Password copied to clipboard");
            
            // Schedule clipboard clear
            if clear_after > 0 {
                let ctx = ClipboardContext::new().unwrap();
                tokio::spawn(async move {
                    sleep(Duration::from_secs(clear_after)).await;
                    let mut ctx = ctx;
                    let _ = ctx.set_contents(String::new());
                });
                println!("  (will clear in {} seconds)", clear_after);
            }
        } else if let Some(field_name) = field {
            // Show specific field
            match field_name.as_str() {
                "username" => println!("{}", entry.username),
                "password" => println!("{}", entry.password.as_ref()),
                "url" => println!("{}", entry.url.as_deref().unwrap_or("")),
                "notes" => println!("{}", entry.notes.as_ref().map(|n| n.as_ref()).unwrap_or("")),
                _ => {
                    if let Some(value) = entry.custom_fields.get(&field_name) {
                        println!("{}", value.as_ref());
                    } else {
                        return Err(CliError::FieldNotFound(field_name));
                    }
                }
            }
        } else {
            // Show full entry
            match format {
                OutputFormat::Text => self.print_secret_text(&entry),
                OutputFormat::Json => self.print_secret_json(&entry)?,
                OutputFormat::Table => self.print_secret_table(&entry),
            }
        }
        
        Ok(())
    }
    
    async fn open_vault(&mut self) -> Result<&mut Vault, CliError> {
        // Check if we have an active session
        if let Some(session) = &mut self.session {
            if session.last_activity.elapsed() < session.timeout {
                session.last_activity = std::time::Instant::now();
                return Ok(&mut session.vault);
            }
        }
        
        // Open new session
        let vault_path = self.config.data_dir.join("vault.qdag");
        if !vault_path.exists() {
            return Err(CliError::VaultNotFound(vault_path));
        }
        
        let password = prompt_password("Enter vault password: ")?;
        let vault = Vault::open(&vault_path, &password)?;
        
        self.session = Some(VaultSession {
            vault,
            path: vault_path,
            last_activity: std::time::Instant::now(),
            timeout: Duration::from_secs(300), // 5 minutes
        });
        
        Ok(&mut self.session.as_mut().unwrap().vault)
    }
}
```

### 3. Output Formatting

#### 3.1 Table Output (`tools/cli/src/vault/output.rs`)
```rust
use comfy_table::{Table, Cell, Color, Attribute, ContentArrangement};

pub struct VaultOutputFormatter;

impl VaultOutputFormatter {
    pub fn format_secret_list(
        secrets: &[SecretMetadata],
        detailed: bool,
    ) -> String {
        let mut table = Table::new();
        
        if detailed {
            table.set_header(vec![
                Cell::new("Label").add_attribute(Attribute::Bold),
                Cell::new("Username").add_attribute(Attribute::Bold),
                Cell::new("Category").add_attribute(Attribute::Bold),
                Cell::new("Modified").add_attribute(Attribute::Bold),
                Cell::new("Strength").add_attribute(Attribute::Bold),
                Cell::new("Tags").add_attribute(Attribute::Bold),
            ]);
            
            for secret in secrets {
                table.add_row(vec![
                    Cell::new(&secret.label),
                    Cell::new(&secret.username),
                    Cell::new(secret.category.as_deref().unwrap_or("-")),
                    Cell::new(&format_timestamp(secret.modified_at)),
                    Cell::new(&secret.password_strength.to_string())
                        .fg(strength_color(&secret.password_strength)),
                    Cell::new(&secret.tags.join(", ")),
                ]);
            }
        } else {
            table.set_header(vec![
                Cell::new("Label").add_attribute(Attribute::Bold),
                Cell::new("Username").add_attribute(Attribute::Bold),
                Cell::new("Modified").add_attribute(Attribute::Bold),
            ]);
            
            for secret in secrets {
                table.add_row(vec![
                    Cell::new(&secret.label),
                    Cell::new(&secret.username),
                    Cell::new(&format_timestamp(secret.modified_at)),
                ]);
            }
        }
        
        table.to_string()
    }
    
    pub fn format_stats(stats: &VaultStats) -> String {
        let mut sections = Vec::new();
        
        // Overview section
        let mut overview = Table::new();
        overview.set_header(vec!["Metric", "Value"]);
        overview.add_row(vec!["Total Secrets", &stats.total_secrets.to_string()]);
        overview.add_row(vec!["Categories", &stats.categories.to_string()]);
        overview.add_row(vec!["Average Age", &format_duration(stats.average_age)]);
        overview.add_row(vec!["Last Modified", &format_timestamp(stats.last_modified)]);
        
        sections.push(("Overview", overview.to_string()));
        
        // Password strength breakdown
        if let Some(strength_stats) = &stats.strength_breakdown {
            let mut strength = Table::new();
            strength.set_header(vec!["Strength", "Count", "Percentage"]);
            
            for (level, count) in strength_stats {
                let percentage = (*count as f64 / stats.total_secrets as f64) * 100.0;
                strength.add_row(vec![
                    Cell::new(&level.to_string()).fg(strength_color(level)),
                    Cell::new(&count.to_string()),
                    Cell::new(&format!("{:.1}%", percentage)),
                ]);
            }
            
            sections.push(("Password Strength", strength.to_string()));
        }
        
        // Category breakdown
        if let Some(category_stats) = &stats.category_breakdown {
            let mut categories = Table::new();
            categories.set_header(vec!["Category", "Count", "Percentage"]);
            
            for (cat, count) in category_stats {
                let percentage = (*count as f64 / stats.total_secrets as f64) * 100.0;
                categories.add_row(vec![
                    cat,
                    &count.to_string(),
                    &format!("{:.1}%", percentage),
                ]);
            }
            
            sections.push(("Categories", categories.to_string()));
        }
        
        sections.into_iter()
            .map(|(title, content)| format!("## {}\n\n{}", title, content))
            .collect::<Vec<_>>()
            .join("\n\n")
    }
}

fn strength_color(strength: &PasswordStrength) -> Color {
    match strength {
        PasswordStrength::VeryWeak => Color::Red,
        PasswordStrength::Weak => Color::Magenta,
        PasswordStrength::Fair => Color::Yellow,
        PasswordStrength::Good => Color::Cyan,
        PasswordStrength::Strong => Color::Green,
    }
}

fn format_timestamp(ts: u64) -> String {
    use chrono::{DateTime, Utc};
    let dt = DateTime::<Utc>::from_timestamp(ts as i64, 0).unwrap();
    dt.format("%Y-%m-%d %H:%M").to_string()
}

fn format_duration(seconds: u64) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    
    if days > 0 {
        format!("{} days", days)
    } else if hours > 0 {
        format!("{} hours", hours)
    } else {
        format!("{} minutes", seconds / 60)
    }
}
```

### 4. Security Features

#### 4.1 Session Management (`tools/cli/src/vault/session.rs`)
```rust
use std::sync::Arc;
use tokio::sync::RwLock;
use zeroize::Zeroize;

pub struct VaultSessionManager {
    sessions: Arc<RwLock<HashMap<PathBuf, VaultSession>>>,
    config: SessionConfig,
}

#[derive(Clone)]
pub struct SessionConfig {
    pub timeout: Duration,
    pub max_sessions: usize,
    pub auto_lock: bool,
    pub require_password_on_resume: bool,
}

impl VaultSessionManager {
    pub fn new(config: SessionConfig) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }
    
    pub async fn get_or_create(
        &self,
        path: &Path,
        password_provider: impl FnOnce() -> Result<String, CliError>,
    ) -> Result<Arc<RwLock<Vault>>, CliError> {
        let mut sessions = self.sessions.write().await;
        
        // Check existing session
        if let Some(session) = sessions.get_mut(path) {
            if session.is_valid() {
                session.touch();
                return Ok(Arc::clone(&session.vault));
            } else {
                // Session expired, remove it
                sessions.remove(path);
            }
        }
        
        // Create new session
        let password = password_provider()?;
        let vault = Vault::open(path, &password)?;
        
        let session = VaultSession {
            vault: Arc::new(RwLock::new(vault)),
            created_at: std::time::Instant::now(),
            last_activity: std::time::Instant::now(),
            timeout: self.config.timeout,
        };
        
        let vault_ref = Arc::clone(&session.vault);
        sessions.insert(path.to_owned(), session);
        
        // Cleanup old sessions if needed
        if sessions.len() > self.config.max_sessions {
            self.cleanup_old_sessions(&mut sessions);
        }
        
        Ok(vault_ref)
    }
    
    fn cleanup_old_sessions(&self, sessions: &mut HashMap<PathBuf, VaultSession>) {
        let mut expired: Vec<PathBuf> = sessions
            .iter()
            .filter(|(_, session)| !session.is_valid())
            .map(|(path, _)| path.clone())
            .collect();
        
        if expired.len() < sessions.len() - self.config.max_sessions {
            // Remove oldest sessions
            let mut entries: Vec<_> = sessions
                .iter()
                .map(|(path, session)| (path.clone(), session.last_activity))
                .collect();
            
            entries.sort_by_key(|(_, time)| *time);
            
            let to_remove = entries.len() - self.config.max_sessions;
            expired.extend(entries.into_iter().take(to_remove).map(|(path, _)| path));
        }
        
        for path in expired {
            sessions.remove(&path);
        }
    }
}
```

#### 4.2 Clipboard Integration (`tools/cli/src/vault/clipboard.rs`)
```rust
use clipboard::{ClipboardContext, ClipboardProvider};
use tokio::time::{sleep, Duration};
use zeroize::Zeroize;

pub struct SecureClipboard {
    context: ClipboardContext,
}

impl SecureClipboard {
    pub fn new() -> Result<Self, CliError> {
        let context = ClipboardContext::new()
            .map_err(|e| CliError::Clipboard(e.to_string()))?;
        Ok(Self { context })
    }
    
    pub fn copy_with_timeout(
        &mut self,
        content: &str,
        timeout: Duration,
    ) -> Result<(), CliError> {
        // Store current clipboard content
        let previous = self.context.get_contents()
            .unwrap_or_default();
        
        // Set new content
        self.context.set_contents(content.to_owned())
            .map_err(|e| CliError::Clipboard(e.to_string()))?;
        
        // Schedule cleanup
        let mut cleanup_content = content.to_owned();
        tokio::spawn(async move {
            sleep(timeout).await;
            
            // Clear the content from memory first
            cleanup_content.zeroize();
            
            // Try to restore previous content or clear
            if let Ok(mut ctx) = ClipboardContext::new() {
                let _ = ctx.set_contents(previous);
            }
        });
        
        Ok(())
    }
}
```

### 5. Import/Export Functionality

#### 5.1 Export Formats (`tools/cli/src/vault/export.rs`)
```rust
use serde_json;
use csv::Writer;

#[derive(Debug, Clone, ValueEnum)]
pub enum ExportFormat {
    /// QuDAG encrypted format (default)
    Encrypted,
    /// JSON format (optionally encrypted)
    Json,
    /// CSV format (optionally encrypted)
    Csv,
    /// KeePass XML format
    KeepassXml,
    /// 1Password format
    OnePassword,
    /// Bitwarden format
    Bitwarden,
}

pub struct VaultExporter;

impl VaultExporter {
    pub async fn export(
        vault: &Vault,
        format: ExportFormat,
        output: &Path,
        options: ExportOptions,
    ) -> Result<ExportStats, CliError> {
        match format {
            ExportFormat::Encrypted => self.export_encrypted(vault, output, options).await,
            ExportFormat::Json => self.export_json(vault, output, options).await,
            ExportFormat::Csv => self.export_csv(vault, output, options).await,
            ExportFormat::KeepassXml => self.export_keepass(vault, output, options).await,
            ExportFormat::OnePassword => self.export_1password(vault, output, options).await,
            ExportFormat::Bitwarden => self.export_bitwarden(vault, output, options).await,
        }
    }
    
    async fn export_json(
        &self,
        vault: &Vault,
        output: &Path,
        options: ExportOptions,
    ) -> Result<ExportStats, CliError> {
        let filter = Filter::from_options(&options);
        let secrets = vault.list_secrets(Some(&filter))?;
        
        let mut entries = Vec::new();
        for metadata in secrets {
            let entry = vault.get_secret(&metadata.id)?;
            entries.push(self.entry_to_json(&entry));
        }
        
        let json_data = serde_json::json!({
            "version": "1.0",
            "exported_at": current_timestamp(),
            "entries": entries,
        });
        
        let json_string = serde_json::to_string_pretty(&json_data)?;
        
        // Optionally encrypt
        let output_data = if let Some(password) = options.export_password {
            self.encrypt_export(&json_string.as_bytes(), &password)?
        } else {
            json_string.into_bytes()
        };
        
        std::fs::write(output, output_data)?;
        
        Ok(ExportStats {
            total_entries: entries.len(),
            format: ExportFormat::Json,
            encrypted: options.export_password.is_some(),
            file_size: std::fs::metadata(output)?.len(),
        })
    }
}
```

#### 5.2 Import Formats (`tools/cli/src/vault/import.rs`)
```rust
#[derive(Debug, Clone, ValueEnum)]
pub enum ImportFormat {
    /// QuDAG vault format
    Qudag,
    /// JSON format
    Json,
    /// CSV format
    Csv,
    /// KeePass XML
    KeepassXml,
    /// 1Password export
    OnePassword,
    /// Bitwarden export
    Bitwarden,
    /// LastPass export
    Lastpass,
}

pub struct VaultImporter;

impl VaultImporter {
    pub async fn import(
        &self,
        vault: &mut Vault,
        input: &Path,
        format: Option<ImportFormat>,
        options: ImportOptions,
    ) -> Result<ImportStats, CliError> {
        // Auto-detect format if not specified
        let format = format.unwrap_or_else(|| self.detect_format(input));
        
        let entries = match format {
            ImportFormat::Qudag => self.import_qudag(input, options).await?,
            ImportFormat::Json => self.import_json(input, options).await?,
            ImportFormat::Csv => self.import_csv(input, options).await?,
            ImportFormat::KeepassXml => self.import_keepass(input, options).await?,
            ImportFormat::OnePassword => self.import_1password(input, options).await?,
            ImportFormat::Bitwarden => self.import_bitwarden(input, options).await?,
            ImportFormat::Lastpass => self.import_lastpass(input, options).await?,
        };
        
        // Import entries with conflict resolution
        let mut stats = ImportStats::default();
        
        for entry in entries {
            match self.import_entry(vault, entry, &options).await {
                Ok(ImportResult::Added) => stats.added += 1,
                Ok(ImportResult::Updated) => stats.updated += 1,
                Ok(ImportResult::Skipped) => stats.skipped += 1,
                Err(e) => {
                    stats.errors += 1;
                    if options.stop_on_error {
                        return Err(e);
                    }
                }
            }
        }
        
        // Save vault
        if !options.dry_run {
            vault.save()?;
        }
        
        Ok(stats)
    }
    
    fn detect_format(&self, path: &Path) -> ImportFormat {
        // Check file extension
        if let Some(ext) = path.extension() {
            match ext.to_str() {
                Some("qdag") => return ImportFormat::Qudag,
                Some("json") => return ImportFormat::Json,
                Some("csv") => return ImportFormat::Csv,
                Some("xml") => return ImportFormat::KeepassXml,
                Some("1pif") => return ImportFormat::OnePassword,
                _ => {}
            }
        }
        
        // Try to detect by content
        if let Ok(content) = std::fs::read_to_string(path) {
            if content.trim_start().starts_with('{') {
                return ImportFormat::Json;
            } else if content.contains("<?xml") {
                return ImportFormat::KeepassXml;
            }
        }
        
        // Default to CSV
        ImportFormat::Csv
    }
}
```

### 6. Interactive Features

#### 6.1 Interactive Mode (`tools/cli/src/vault/interactive.rs`)
```rust
use rustyline::Editor;
use rustyline::error::ReadlineError;

pub struct InteractiveVault {
    vault: Vault,
    editor: Editor<()>,
    handler: VaultCommandHandler,
}

impl InteractiveVault {
    pub async fn run(&mut self) -> Result<(), CliError> {
        println!("QuDAG Vault Interactive Mode");
        println!("Type 'help' for commands, 'exit' to quit");
        println!();
        
        loop {
            let readline = self.editor.readline("vault> ");
            
            match readline {
                Ok(line) => {
                    let line = line.trim();
                    
                    if line.is_empty() {
                        continue;
                    }
                    
                    self.editor.add_history_entry(line);
                    
                    match self.parse_command(line) {
                        Ok(Some(command)) => {
                            if let Err(e) = self.handler.handle(command).await {
                                eprintln!("Error: {}", e);
                            }
                        }
                        Ok(None) => break, // Exit command
                        Err(e) => eprintln!("Error: {}", e),
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    println!("Use 'exit' to quit");
                }
                Err(ReadlineError::Eof) => break,
                Err(err) => {
                    eprintln!("Error: {:?}", err);
                    break;
                }
            }
        }
        
        Ok(())
    }
    
    fn parse_command(&self, input: &str) -> Result<Option<VaultCommands>, CliError> {
        let parts: Vec<&str> = input.split_whitespace().collect();
        
        if parts.is_empty() {
            return Ok(None);
        }
        
        match parts[0] {
            "help" | "?" => {
                self.print_help();
                Ok(None)
            }
            "exit" | "quit" => Ok(None),
            "add" => self.parse_add_command(&parts[1..]),
            "get" => self.parse_get_command(&parts[1..]),
            "list" | "ls" => self.parse_list_command(&parts[1..]),
            "update" => self.parse_update_command(&parts[1..]),
            "delete" | "rm" => self.parse_delete_command(&parts[1..]),
            "generate" | "gen" => self.parse_generate_command(&parts[1..]),
            _ => Err(CliError::UnknownCommand(parts[0].to_string())),
        }
    }
}
```

## Testing Strategy

### Integration Tests
```rust
#[cfg(test)]
mod cli_tests {
    use super::*;
    use assert_cmd::Command;
    use predicates::prelude::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_vault_init_command() {
        let temp = tempdir().unwrap();
        let vault_path = temp.path().join("test.vault");
        
        Command::cargo_bin("qudag")
            .unwrap()
            .args(&["vault", "init", "--path", vault_path.to_str().unwrap()])
            .write_stdin("test_password\\ntest_password\\n")
            .assert()
            .success()
            .stdout(predicate::str::contains("Vault initialized successfully"));
        
        assert!(vault_path.exists());
    }
    
    #[test]
    fn test_vault_add_get_flow() {
        let temp = tempdir().unwrap();
        std::env::set_var("QUDAG_DATA_DIR", temp.path());
        
        // Initialize vault
        Command::cargo_bin("qudag")
            .unwrap()
            .args(&["vault", "init", "--no-confirm"])
            .write_stdin("test_password\\n")
            .assert()
            .success();
        
        // Add secret
        Command::cargo_bin("qudag")
            .unwrap()
            .args(&["vault", "add", "test/secret", "-u", "testuser", "-p", "testpass"])
            .write_stdin("test_password\\n")
            .assert()
            .success();
        
        // Get secret
        Command::cargo_bin("qudag")
            .unwrap()
            .args(&["vault", "get", "test/secret"])
            .write_stdin("test_password\\n")
            .assert()
            .success()
            .stdout(predicate::str::contains("testuser"));
    }
}
```

## Performance Considerations

1. **Session Caching**: Keep vault open for multiple operations
2. **Lazy Loading**: Load only required secrets, not entire vault
3. **Background Saves**: Async save operations for better responsiveness
4. **Indexed Search**: Build search indices for large vaults
5. **Parallel Operations**: Use tokio for concurrent operations

## Security Best Practices

1. **Password Input**: Always use secure password prompts
2. **Memory Cleanup**: Zeroize sensitive data after use
3. **Session Timeout**: Auto-lock vault after inactivity
4. **Clipboard Security**: Auto-clear clipboard after timeout
5. **Audit Logging**: Log all vault operations (optional)
6. **Environment Safety**: Warn about password in environment variables

## Next Steps

1. Implement core command handlers
2. Add comprehensive error handling
3. Create interactive mode
4. Add shell completions
5. Write extensive documentation
6. Performance optimization for large vaults