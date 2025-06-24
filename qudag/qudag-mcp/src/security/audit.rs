//! Audit logging for QuDAG MCP security events.

use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, RwLock};
use tokio::time::{interval, Duration};
use tracing::{debug, info, warn, error};

use crate::error::{McpError, McpResult};
use crate::security::{SecurityEvent, SecurityEventType, SecuritySeverity, AuditConfig};

/// Audit logging trait
#[async_trait::async_trait]
pub trait AuditLogger: Send + Sync {
    /// Log a security event
    async fn log_event(&self, event: &SecurityEvent) -> McpResult<()>;
    
    /// Query events by criteria
    async fn query_events(&self, criteria: AuditQuery) -> McpResult<Vec<SecurityEvent>>;
    
    /// Get audit statistics
    async fn get_statistics(&self) -> McpResult<AuditStatistics>;
    
    /// Cleanup old audit logs
    async fn cleanup_old_logs(&self) -> McpResult<u64>;
    
    /// Export audit logs
    async fn export_logs(&self, format: ExportFormat, output_path: &Path) -> McpResult<()>;
}

/// File-based audit logger implementation
pub struct FileAuditLogger {
    /// Current log file
    log_file: Arc<RwLock<std::fs::File>>,
    
    /// Log file path
    log_path: PathBuf,
    
    /// Configuration
    config: AuditConfig,
    
    /// Event buffer for batch writing
    buffer: Arc<RwLock<Vec<AuditLogEntry>>>,
    
    /// Buffer flush sender
    flush_sender: mpsc::Sender<()>,
    
    /// Event statistics
    stats: Arc<RwLock<AuditStatistics>>,
}

/// Structured audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    /// Entry ID
    pub id: String,
    
    /// Timestamp (ISO 8601)
    pub timestamp: String,
    
    /// Event level
    pub level: AuditLevel,
    
    /// Event type
    pub event_type: SecurityEventType,
    
    /// Event severity
    pub severity: SecuritySeverity,
    
    /// Event source
    pub source: String,
    
    /// Event message
    pub message: String,
    
    /// User ID (if applicable)
    pub user_id: Option<String>,
    
    /// Session ID (if applicable)
    pub session_id: Option<String>,
    
    /// Client IP address (if applicable)
    pub client_ip: Option<String>,
    
    /// Request ID for correlation
    pub request_id: Option<String>,
    
    /// Event data
    pub data: serde_json::Value,
    
    /// Event tags
    pub tags: Vec<String>,
    
    /// Event fingerprint for deduplication
    pub fingerprint: Option<String>,
}

/// Audit log levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuditLevel {
    /// Debug level events
    Debug,
    
    /// Informational events
    Info,
    
    /// Warning events
    Warn,
    
    /// Error events
    Error,
    
    /// Critical events
    Critical,
}

/// Audit query criteria
#[derive(Debug, Clone)]
pub struct AuditQuery {
    /// Time range filter
    pub time_range: Option<TimeRange>,
    
    /// Event types to include
    pub event_types: Option<Vec<SecurityEventType>>,
    
    /// Severity levels to include
    pub severities: Option<Vec<SecuritySeverity>>,
    
    /// User ID filter
    pub user_id: Option<String>,
    
    /// Client IP filter
    pub client_ip: Option<String>,
    
    /// Request ID filter
    pub request_id: Option<String>,
    
    /// Tags filter (any of these tags)
    pub tags: Option<Vec<String>>,
    
    /// Text search in messages
    pub text_search: Option<String>,
    
    /// Maximum number of results
    pub limit: Option<usize>,
    
    /// Result offset
    pub offset: Option<usize>,
    
    /// Sort order
    pub sort_order: SortOrder,
}

/// Time range for queries
#[derive(Debug, Clone)]
pub struct TimeRange {
    /// Start time (inclusive)
    pub start: SystemTime,
    
    /// End time (exclusive)
    pub end: SystemTime,
}

/// Sort order for query results
#[derive(Debug, Clone)]
pub enum SortOrder {
    /// Newest first
    Newest,
    
    /// Oldest first
    Oldest,
    
    /// By severity (highest first)
    SeverityDesc,
    
    /// By severity (lowest first)
    SeverityAsc,
}

/// Export format options
#[derive(Debug, Clone)]
pub enum ExportFormat {
    /// JSON format
    Json,
    
    /// CSV format
    Csv,
    
    /// Plain text format
    Text,
    
    /// Structured syslog format
    Syslog,
}

/// Audit statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditStatistics {
    /// Total events logged
    pub total_events: u64,
    
    /// Events by type
    pub events_by_type: HashMap<String, u64>,
    
    /// Events by severity
    pub events_by_severity: HashMap<String, u64>,
    
    /// Events by level
    pub events_by_level: HashMap<String, u64>,
    
    /// Unique users
    pub unique_users: u64,
    
    /// Unique client IPs
    pub unique_client_ips: u64,
    
    /// Events in last 24 hours
    pub events_last_24h: u64,
    
    /// Critical events in last 24 hours
    pub critical_events_last_24h: u64,
    
    /// Average events per hour
    pub avg_events_per_hour: f64,
    
    /// Log file size in bytes
    pub log_file_size: u64,
    
    /// Last cleanup time
    pub last_cleanup: Option<SystemTime>,
}

impl FileAuditLogger {
    /// Create new file audit logger
    pub async fn new(config: AuditConfig) -> McpResult<Self> {
        let log_path = PathBuf::from(&config.log_file);
        
        // Ensure directory exists
        if let Some(parent) = log_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        // Open or create log file
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)?;
        
        let log_file = Arc::new(RwLock::new(file));
        
        // Create buffer and flush channel
        let buffer = Arc::new(RwLock::new(Vec::new()));
        let (flush_sender, mut flush_receiver) = mpsc::channel::<()>(100);
        let stats = Arc::new(RwLock::new(AuditStatistics::new()));
        
        // Start background tasks
        let logger = Self {
            log_file: log_file.clone(),
            log_path: log_path.clone(),
            config: config.clone(),
            buffer: buffer.clone(),
            flush_sender,
            stats: stats.clone(),
        };
        
        // Background flush task
        let flush_buffer = buffer.clone();
        let flush_file = log_file.clone();
        let flush_config = config.clone();
        tokio::spawn(async move {
            let mut flush_interval = interval(Duration::from_secs(5));
            
            loop {
                tokio::select! {
                    _ = flush_interval.tick() => {
                        Self::flush_buffer_internal(&flush_buffer, &flush_file, &flush_config).await.ok();
                    }
                    _ = flush_receiver.recv() => {
                        Self::flush_buffer_internal(&flush_buffer, &flush_file, &flush_config).await.ok();
                    }
                }
            }
        });
        
        // Background cleanup task
        let cleanup_logger = logger.clone();
        tokio::spawn(async move {
            let mut cleanup_interval = interval(Duration::from_secs(3600)); // Hourly cleanup
            
            loop {
                cleanup_interval.tick().await;
                if let Err(e) = cleanup_logger.cleanup_old_logs().await {
                    warn!("Audit log cleanup failed: {}", e);
                }
            }
        });
        
        info!("File audit logger initialized: {}", log_path.display());
        Ok(logger)
    }
    
    /// Convert security event to audit log entry
    fn event_to_log_entry(&self, event: &SecurityEvent) -> AuditLogEntry {
        let level = match event.severity {
            SecuritySeverity::Info => AuditLevel::Info,
            SecuritySeverity::Low => AuditLevel::Info,
            SecuritySeverity::Medium => AuditLevel::Warn,
            SecuritySeverity::High => AuditLevel::Error,
            SecuritySeverity::Critical => AuditLevel::Critical,
        };
        
        let timestamp = event.timestamp
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        let timestamp_str = chrono::DateTime::from_timestamp(timestamp.as_secs() as i64, 0)
            .unwrap_or_default()
            .to_rfc3339();
        
        // Generate fingerprint for deduplication
        let fingerprint = if !event.tags.contains(&"no-dedup".to_string()) {
            Some(self.generate_fingerprint(event))
        } else {
            None
        };
        
        AuditLogEntry {
            id: event.id.clone(),
            timestamp: timestamp_str,
            level,
            event_type: event.event_type.clone(),
            severity: event.severity.clone(),
            source: event.source.clone(),
            message: event.description.clone(),
            user_id: event.user_id.clone(),
            session_id: None, // Would be extracted from request_id if needed
            client_ip: event.client_ip.clone(),
            request_id: event.request_id.clone(),
            data: event.data.clone(),
            tags: event.tags.clone(),
            fingerprint,
        }
    }
    
    /// Generate event fingerprint for deduplication
    fn generate_fingerprint(&self, event: &SecurityEvent) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        event.event_type.hash(&mut hasher);
        event.source.hash(&mut hasher);
        event.description.hash(&mut hasher);
        event.user_id.hash(&mut hasher);
        
        format!("{:x}", hasher.finish())
    }
    
    /// Format log entry for file output
    fn format_log_entry(&self, entry: &AuditLogEntry) -> String {
        if self.config.structured {
            // JSON structured format
            serde_json::to_string(entry).unwrap_or_else(|_| {
                format!("{} [{}] {}: {}", entry.timestamp, entry.level, entry.source, entry.message)
            })
        } else {
            // Plain text format
            format!(
                "{} [{}] {} {} {}: {}{}",
                entry.timestamp,
                entry.level,
                entry.event_type,
                entry.severity,
                entry.source,
                entry.message,
                if !entry.tags.is_empty() {
                    format!(" [tags: {}]", entry.tags.join(", "))
                } else {
                    String::new()
                }
            )
        }
    }
    
    /// Flush buffer to file
    async fn flush_buffer_internal(
        buffer: &Arc<RwLock<Vec<AuditLogEntry>>>,
        log_file: &Arc<RwLock<std::fs::File>>,
        config: &AuditConfig,
    ) -> McpResult<()> {
        let mut buffer_guard = buffer.write().await;
        if buffer_guard.is_empty() {
            return Ok(());
        }
        
        let entries = buffer_guard.drain(..).collect::<Vec<_>>();
        drop(buffer_guard);
        
        let mut file_guard = log_file.write().await;
        for entry in entries {
            let formatted = if config.structured {
                serde_json::to_string(&entry).unwrap_or_else(|_| {
                    format!("{} [{}] {}: {}", entry.timestamp, entry.level, entry.source, entry.message)
                })
            } else {
                format!(
                    "{} [{}] {} {} {}: {}{}",
                    entry.timestamp,
                    entry.level,
                    entry.event_type,
                    entry.severity,
                    entry.source,
                    entry.message,
                    if !entry.tags.is_empty() {
                        format!(" [tags: {}]", entry.tags.join(", "))
                    } else {
                        String::new()
                    }
                )
            };
            
            writeln!(file_guard, "{}", formatted)?;
        }
        
        file_guard.flush()?;
        Ok(())
    }
    
    /// Update statistics
    async fn update_statistics(&self, entry: &AuditLogEntry) {
        let mut stats = self.stats.write().await;
        
        stats.total_events += 1;
        
        // Update by type
        let type_key = format!("{:?}", entry.event_type);
        *stats.events_by_type.entry(type_key).or_insert(0) += 1;
        
        // Update by severity
        let severity_key = format!("{:?}", entry.severity);
        *stats.events_by_severity.entry(severity_key).or_insert(0) += 1;
        
        // Update by level
        let level_key = format!("{:?}", entry.level);
        *stats.events_by_level.entry(level_key).or_insert(0) += 1;
        
        // Check if critical event in last 24 hours
        if matches!(entry.severity, SecuritySeverity::Critical) {
            let now = SystemTime::now();
            if let Ok(entry_time) = chrono::DateTime::parse_from_rfc3339(&entry.timestamp) {
                let entry_system_time = SystemTime::UNIX_EPOCH + 
                    Duration::from_secs(entry_time.timestamp() as u64);
                if now.duration_since(entry_system_time).unwrap_or_default() <= Duration::from_secs(24 * 3600) {
                    stats.critical_events_last_24h += 1;
                }
            }
        }
        
        // Update file size
        if let Ok(metadata) = std::fs::metadata(&self.log_path) {
            stats.log_file_size = metadata.len();
        }
    }
    
    /// Check if log rotation is needed
    async fn check_rotation(&self) -> McpResult<bool> {
        let metadata = std::fs::metadata(&self.log_path)?;
        let size_mb = metadata.len() / (1024 * 1024);
        
        Ok(size_mb >= self.config.rotation_size_mb)
    }
    
    /// Rotate log file
    async fn rotate_log(&self) -> McpResult<()> {
        // Close current file
        drop(self.log_file.write().await);
        
        // Rename current log file with timestamp
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let rotated_path = self.log_path.with_extension(format!("log.{}", timestamp));
        std::fs::rename(&self.log_path, &rotated_path)?;
        
        // Create new log file
        let new_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)?;
        
        *self.log_file.write().await = new_file;
        
        // Clean up old rotated files
        self.cleanup_rotated_files().await?;
        
        info!("Audit log rotated: {} -> {}", self.log_path.display(), rotated_path.display());
        Ok(())
    }
    
    /// Clean up old rotated log files
    async fn cleanup_rotated_files(&self) -> McpResult<()> {
        let log_dir = self.log_path.parent()
            .ok_or_else(|| McpError::audit("Cannot determine log directory"))?;
        
        let log_name = self.log_path.file_stem()
            .ok_or_else(|| McpError::audit("Cannot determine log file name"))?
            .to_string_lossy();
        
        let mut rotated_files = Vec::new();
        
        for entry in std::fs::read_dir(log_dir)? {
            let entry = entry?;
            let file_name = entry.file_name().to_string_lossy().to_string();
            
            if file_name.starts_with(&format!("{}.log.", log_name)) {
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(modified) = metadata.modified() {
                        rotated_files.push((entry.path(), modified));
                    }
                }
            }
        }
        
        // Sort by modification time (newest first)
        rotated_files.sort_by(|a, b| b.1.cmp(&a.1));
        
        // Remove files beyond rotation count
        for (path, _) in rotated_files.into_iter().skip(self.config.rotation_count as usize) {
            if let Err(e) = std::fs::remove_file(&path) {
                warn!("Failed to remove old log file {}: {}", path.display(), e);
            } else {
                debug!("Removed old log file: {}", path.display());
            }
        }
        
        Ok(())
    }
}

impl Clone for FileAuditLogger {
    fn clone(&self) -> Self {
        Self {
            log_file: self.log_file.clone(),
            log_path: self.log_path.clone(),
            config: self.config.clone(),
            buffer: self.buffer.clone(),
            flush_sender: self.flush_sender.clone(),
            stats: self.stats.clone(),
        }
    }
}

#[async_trait::async_trait]
impl AuditLogger for FileAuditLogger {
    async fn log_event(&self, event: &SecurityEvent) -> McpResult<()> {
        let entry = self.event_to_log_entry(event);
        
        // Add to buffer
        let mut buffer = self.buffer.write().await;
        buffer.push(entry.clone());
        
        // Update statistics
        self.update_statistics(&entry).await;
        
        // Trigger flush if buffer is full
        if buffer.len() >= 100 {
            drop(buffer);
            self.flush_sender.send(()).await.ok();
        }
        
        // Check if log rotation is needed
        if self.check_rotation().await? {
            self.rotate_log().await?;
        }
        
        Ok(())
    }
    
    async fn query_events(&self, criteria: AuditQuery) -> McpResult<Vec<SecurityEvent>> {
        // For file-based logging, we'll implement a simple grep-like search
        // In production, you might want to use a proper database or indexing system
        
        let content = tokio::fs::read_to_string(&self.log_path).await?;
        let mut events = Vec::new();
        
        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }
            
            // Try to parse as JSON if structured logging is enabled
            if self.config.structured {
                if let Ok(entry) = serde_json::from_str::<AuditLogEntry>(line) {
                    if self.matches_criteria(&entry, &criteria) {
                        // Convert back to SecurityEvent (simplified)
                        let event = SecurityEvent {
                            id: entry.id,
                            event_type: entry.event_type,
                            severity: entry.severity,
                            timestamp: chrono::DateTime::parse_from_rfc3339(&entry.timestamp)
                                .map(|dt| SystemTime::UNIX_EPOCH + Duration::from_secs(dt.timestamp() as u64))
                                .unwrap_or(SystemTime::now()),
                            source: entry.source,
                            description: entry.message,
                            user_id: entry.user_id,
                            client_ip: entry.client_ip,
                            request_id: entry.request_id,
                            data: entry.data,
                            tags: entry.tags,
                        };
                        events.push(event);
                    }
                }
            }
        }
        
        // Apply sorting
        match criteria.sort_order {
            SortOrder::Newest => events.sort_by(|a, b| b.timestamp.cmp(&a.timestamp)),
            SortOrder::Oldest => events.sort_by(|a, b| a.timestamp.cmp(&b.timestamp)),
            SortOrder::SeverityDesc => events.sort_by(|a, b| b.severity.cmp(&a.severity)),
            SortOrder::SeverityAsc => events.sort_by(|a, b| a.severity.cmp(&b.severity)),
        }
        
        // Apply limit and offset
        let start = criteria.offset.unwrap_or(0);
        let end = criteria.limit
            .map(|limit| start + limit)
            .unwrap_or(events.len());
        
        Ok(events.into_iter().skip(start).take(end - start).collect())
    }
    
    async fn get_statistics(&self) -> McpResult<AuditStatistics> {
        Ok(self.stats.read().await.clone())
    }
    
    async fn cleanup_old_logs(&self) -> McpResult<u64> {
        let retention_duration = Duration::from_secs(self.config.retention_days as u64 * 24 * 3600);
        let cutoff_time = SystemTime::now() - retention_duration;
        
        // For file-based logging, we'll clean up rotated files
        self.cleanup_rotated_files().await?;
        
        // Update cleanup time
        let mut stats = self.stats.write().await;
        stats.last_cleanup = Some(SystemTime::now());
        
        Ok(0) // Return number of cleaned up entries
    }
    
    async fn export_logs(&self, format: ExportFormat, output_path: &Path) -> McpResult<()> {
        let content = tokio::fs::read_to_string(&self.log_path).await?;
        let mut output = Vec::new();
        
        match format {
            ExportFormat::Json => {
                // Export as JSON array
                let mut entries = Vec::new();
                for line in content.lines() {
                    if self.config.structured {
                        if let Ok(entry) = serde_json::from_str::<AuditLogEntry>(line) {
                            entries.push(entry);
                        }
                    }
                }
                output = serde_json::to_vec_pretty(&entries)?;
            }
            ExportFormat::Csv => {
                // Export as CSV
                output.extend_from_slice(b"timestamp,level,event_type,severity,source,message,user_id,client_ip\n");
                for line in content.lines() {
                    if self.config.structured {
                        if let Ok(entry) = serde_json::from_str::<AuditLogEntry>(line) {
                            let csv_line = format!(
                                "{},{:?},{:?},{:?},{},{},{},{}\n",
                                entry.timestamp,
                                entry.level,
                                entry.event_type,
                                entry.severity,
                                entry.source,
                                entry.message.replace(',', ";"),
                                entry.user_id.unwrap_or_default(),
                                entry.client_ip.unwrap_or_default()
                            );
                            output.extend_from_slice(csv_line.as_bytes());
                        }
                    }
                }
            }
            ExportFormat::Text => {
                // Export as plain text
                output = content.into_bytes();
            }
            ExportFormat::Syslog => {
                // Export in syslog format
                for line in content.lines() {
                    if self.config.structured {
                        if let Ok(entry) = serde_json::from_str::<AuditLogEntry>(line) {
                            let syslog_line = format!(
                                "<134>{} {} qudag-mcp[{}]: {}\n",
                                entry.timestamp,
                                "localhost", // Would be actual hostname
                                std::process::id(),
                                entry.message
                            );
                            output.extend_from_slice(syslog_line.as_bytes());
                        }
                    }
                }
            }
        }
        
        tokio::fs::write(output_path, output).await?;
        info!("Audit logs exported to: {}", output_path.display());
        Ok(())
    }
}

impl FileAuditLogger {
    /// Check if log entry matches query criteria
    fn matches_criteria(&self, entry: &AuditLogEntry, criteria: &AuditQuery) -> bool {
        // Time range check
        if let Some(time_range) = &criteria.time_range {
            if let Ok(entry_time) = chrono::DateTime::parse_from_rfc3339(&entry.timestamp) {
                let entry_system_time = SystemTime::UNIX_EPOCH + 
                    Duration::from_secs(entry_time.timestamp() as u64);
                if entry_system_time < time_range.start || entry_system_time >= time_range.end {
                    return false;
                }
            }
        }
        
        // Event type check
        if let Some(event_types) = &criteria.event_types {
            if !event_types.contains(&entry.event_type) {
                return false;
            }
        }
        
        // Severity check
        if let Some(severities) = &criteria.severities {
            if !severities.contains(&entry.severity) {
                return false;
            }
        }
        
        // User ID check
        if let Some(user_id) = &criteria.user_id {
            if entry.user_id.as_ref() != Some(user_id) {
                return false;
            }
        }
        
        // Client IP check
        if let Some(client_ip) = &criteria.client_ip {
            if entry.client_ip.as_ref() != Some(client_ip) {
                return false;
            }
        }
        
        // Request ID check
        if let Some(request_id) = &criteria.request_id {
            if entry.request_id.as_ref() != Some(request_id) {
                return false;
            }
        }
        
        // Tags check
        if let Some(tags) = &criteria.tags {
            if !tags.iter().any(|tag| entry.tags.contains(tag)) {
                return false;
            }
        }
        
        // Text search check
        if let Some(search_text) = &criteria.text_search {
            if !entry.message.to_lowercase().contains(&search_text.to_lowercase()) {
                return false;
            }
        }
        
        true
    }
}

impl AuditStatistics {
    /// Create new empty statistics
    fn new() -> Self {
        Self {
            total_events: 0,
            events_by_type: HashMap::new(),
            events_by_severity: HashMap::new(),
            events_by_level: HashMap::new(),
            unique_users: 0,
            unique_client_ips: 0,
            events_last_24h: 0,
            critical_events_last_24h: 0,
            avg_events_per_hour: 0.0,
            log_file_size: 0,
            last_cleanup: None,
        }
    }
}

impl std::fmt::Display for AuditLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditLevel::Debug => write!(f, "DEBUG"),
            AuditLevel::Info => write!(f, "INFO"),
            AuditLevel::Warn => write!(f, "WARN"),
            AuditLevel::Error => write!(f, "ERROR"),
            AuditLevel::Critical => write!(f, "CRITICAL"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use tokio;
    
    async fn create_test_logger() -> FileAuditLogger {
        let temp_dir = tempdir().unwrap();
        let log_file = temp_dir.path().join("test_audit.log");
        
        let config = AuditConfig {
            structured: true,
            retention_days: 30,
            include_bodies: false,
            include_sensitive: false,
            log_file: log_file.to_string_lossy().to_string(),
            rotation_size_mb: 10,
            rotation_count: 5,
        };
        
        FileAuditLogger::new(config).await.unwrap()
    }
    
    #[tokio::test]
    async fn test_audit_logging() {
        let logger = create_test_logger().await;
        
        let event = SecurityEvent {
            id: "test-event-1".to_string(),
            event_type: SecurityEventType::Authentication,
            severity: SecuritySeverity::Info,
            timestamp: SystemTime::now(),
            source: "test".to_string(),
            description: "Test authentication event".to_string(),
            user_id: Some("user123".to_string()),
            client_ip: Some("192.168.1.1".to_string()),
            request_id: Some("req-123".to_string()),
            data: serde_json::json!({"test": "data"}),
            tags: vec!["authentication".to_string(), "test".to_string()],
        };
        
        logger.log_event(&event).await.unwrap();
        
        // Wait for flush
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // Check statistics
        let stats = logger.get_statistics().await.unwrap();
        assert_eq!(stats.total_events, 1);
        assert!(stats.events_by_type.contains_key("Authentication"));
    }
    
    #[tokio::test]
    async fn test_audit_query() {
        let logger = create_test_logger().await;
        
        // Log multiple events
        for i in 0..5 {
            let event = SecurityEvent {
                id: format!("test-event-{}", i),
                event_type: if i % 2 == 0 { 
                    SecurityEventType::Authentication 
                } else { 
                    SecurityEventType::Authorization 
                },
                severity: SecuritySeverity::Info,
                timestamp: SystemTime::now(),
                source: "test".to_string(),
                description: format!("Test event {}", i),
                user_id: Some(format!("user{}", i)),
                client_ip: Some("192.168.1.1".to_string()),
                request_id: Some(format!("req-{}", i)),
                data: serde_json::json!({"index": i}),
                tags: vec!["test".to_string()],
            };
            
            logger.log_event(&event).await.unwrap();
        }
        
        // Wait for flush
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // Query events
        let query = AuditQuery {
            time_range: None,
            event_types: Some(vec![SecurityEventType::Authentication]),
            severities: None,
            user_id: None,
            client_ip: None,
            request_id: None,
            tags: None,
            text_search: None,
            limit: None,
            offset: None,
            sort_order: SortOrder::Newest,
        };
        
        let results = logger.query_events(query).await.unwrap();
        assert_eq!(results.len(), 3); // Should find 3 authentication events
    }
    
    #[tokio::test]
    async fn test_log_export() {
        let logger = create_test_logger().await;
        
        let event = SecurityEvent {
            id: "export-test".to_string(),
            event_type: SecurityEventType::System,
            severity: SecuritySeverity::Info,
            timestamp: SystemTime::now(),
            source: "test".to_string(),
            description: "Export test event".to_string(),
            user_id: None,
            client_ip: None,
            request_id: None,
            data: serde_json::json!({}),
            tags: vec!["export".to_string()],
        };
        
        logger.log_event(&event).await.unwrap();
        
        // Wait for flush
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // Export logs
        let temp_dir = tempdir().unwrap();
        let export_path = temp_dir.path().join("exported.json");
        
        logger.export_logs(ExportFormat::Json, &export_path).await.unwrap();
        
        // Check exported file exists
        assert!(export_path.exists());
        
        // Check content
        let content = tokio::fs::read_to_string(&export_path).await.unwrap();
        assert!(content.contains("Export test event"));
    }
}