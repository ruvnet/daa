//! Audit logging for rule engine operations

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use tracing::{debug, info};

use crate::engine::RuleEvaluationResult;
use crate::error::{Result, RuleError};
use crate::rules::RuleViolation;

/// Audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: DateTime<Utc>,
    pub event_type: AuditEventType,
    pub message: String,
    pub metadata: serde_json::Value,
}

/// Types of audit events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditEventType {
    RuleRegistration,
    RuleRemoval,
    RuleViolation,
    EvaluationStarted,
    EvaluationCompleted,
    SystemError,
}

/// Trait for audit logging implementations
pub trait AuditLogger: Send + Sync {
    /// Log a rule registration event
    fn log_rule_registration(&self, rule_id: &str) -> Result<()>;
    
    /// Log a rule removal event
    fn log_rule_removal(&self, rule_id: &str) -> Result<()>;
    
    /// Log a rule violation
    fn log_rule_violation(&self, violation: &RuleViolation) -> Result<()>;
    
    /// Log evaluation summary
    fn log_evaluation_summary(&self, result: &RuleEvaluationResult) -> Result<()>;
    
    /// Log a system error
    fn log_system_error(&self, error: &str) -> Result<()>;
    
    /// Get recent audit entries
    fn get_recent_entries(&self, limit: usize) -> Result<Vec<AuditEntry>>;
    
    /// Get entries by event type
    fn get_entries_by_type(&self, event_type: AuditEventType, limit: usize) -> Result<Vec<AuditEntry>>;
}

/// In-memory audit logger implementation
pub struct MemoryAuditLogger {
    entries: Arc<Mutex<VecDeque<AuditEntry>>>,
    max_entries: usize,
}

impl MemoryAuditLogger {
    /// Create a new memory audit logger
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: Arc::new(Mutex::new(VecDeque::new())),
            max_entries,
        }
    }

    /// Create with default capacity
    pub fn default() -> Self {
        Self::new(10000)
    }

    fn add_entry(&self, event_type: AuditEventType, message: String, metadata: serde_json::Value) -> Result<()> {
        let entry = AuditEntry {
            timestamp: Utc::now(),
            event_type,
            message,
            metadata,
        };

        let mut entries = self.entries.lock()
            .map_err(|_| RuleError::Internal("Failed to acquire audit log lock".to_string()))?;

        // Remove old entries if we're at capacity
        while entries.len() >= self.max_entries {
            entries.pop_front();
        }

        entries.push_back(entry);
        debug!("Added audit entry: {:?}", entries.back().unwrap().event_type);
        
        Ok(())
    }

    /// Get the number of entries
    pub fn entry_count(&self) -> Result<usize> {
        let entries = self.entries.lock()
            .map_err(|_| RuleError::Internal("Failed to acquire audit log lock".to_string()))?;
        Ok(entries.len())
    }

    /// Clear all entries
    pub fn clear(&self) -> Result<()> {
        let mut entries = self.entries.lock()
            .map_err(|_| RuleError::Internal("Failed to acquire audit log lock".to_string()))?;
        entries.clear();
        info!("Cleared audit log");
        Ok(())
    }

    /// Export entries to JSON
    pub fn export_to_json(&self) -> Result<String> {
        let entries = self.entries.lock()
            .map_err(|_| RuleError::Internal("Failed to acquire audit log lock".to_string()))?;
        
        let entries_vec: Vec<AuditEntry> = entries.iter().cloned().collect();
        serde_json::to_string_pretty(&entries_vec)
            .map_err(|e| RuleError::SerializationError(e.to_string()))
    }
}

impl AuditLogger for MemoryAuditLogger {
    fn log_rule_registration(&self, rule_id: &str) -> Result<()> {
        self.add_entry(
            AuditEventType::RuleRegistration,
            format!("Rule registered: {}", rule_id),
            serde_json::json!({ "rule_id": rule_id }),
        )
    }

    fn log_rule_removal(&self, rule_id: &str) -> Result<()> {
        self.add_entry(
            AuditEventType::RuleRemoval,
            format!("Rule removed: {}", rule_id),
            serde_json::json!({ "rule_id": rule_id }),
        )
    }

    fn log_rule_violation(&self, violation: &RuleViolation) -> Result<()> {
        self.add_entry(
            AuditEventType::RuleViolation,
            format!("Rule violation: {}", violation.message),
            serde_json::json!({
                "rule_id": violation.rule_id,
                "severity": violation.severity,
                "context": violation.context
            }),
        )
    }

    fn log_evaluation_summary(&self, result: &RuleEvaluationResult) -> Result<()> {
        self.add_entry(
            AuditEventType::EvaluationCompleted,
            result.summary(),
            serde_json::json!({
                "rules_evaluated": result.rules_evaluated,
                "rules_passed": result.rules_passed,
                "rules_failed": result.rules_failed,
                "violations_count": result.violations.len(),
                "execution_time_ms": result.execution_time_ms,
                "has_critical_violations": result.has_critical_violations()
            }),
        )
    }

    fn log_system_error(&self, error: &str) -> Result<()> {
        self.add_entry(
            AuditEventType::SystemError,
            format!("System error: {}", error),
            serde_json::json!({ "error": error }),
        )
    }

    fn get_recent_entries(&self, limit: usize) -> Result<Vec<AuditEntry>> {
        let entries = self.entries.lock()
            .map_err(|_| RuleError::Internal("Failed to acquire audit log lock".to_string()))?;
        
        let recent: Vec<AuditEntry> = entries
            .iter()
            .rev() // Most recent first
            .take(limit)
            .cloned()
            .collect();
        
        Ok(recent)
    }

    fn get_entries_by_type(&self, event_type: AuditEventType, limit: usize) -> Result<Vec<AuditEntry>> {
        let entries = self.entries.lock()
            .map_err(|_| RuleError::Internal("Failed to acquire audit log lock".to_string()))?;
        
        let filtered: Vec<AuditEntry> = entries
            .iter()
            .rev()
            .filter(|entry| std::mem::discriminant(&entry.event_type) == std::mem::discriminant(&event_type))
            .take(limit)
            .cloned()
            .collect();
        
        Ok(filtered)
    }
}

/// File-based audit logger implementation
pub struct FileAuditLogger {
    file_path: String,
    memory_logger: MemoryAuditLogger,
}

impl FileAuditLogger {
    /// Create a new file audit logger
    pub fn new(file_path: String) -> Self {
        Self {
            file_path,
            memory_logger: MemoryAuditLogger::new(1000), // Keep recent entries in memory
        }
    }

    fn write_to_file(&self, entry: &AuditEntry) -> Result<()> {
        use std::fs::OpenOptions;
        use std::io::Write;
        
        let json_line = format!("{}\n", serde_json::to_string(entry)?);
        
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.file_path)
            .map_err(|e| RuleError::Internal(format!("Failed to open audit file: {}", e)))?;
        
        file.write_all(json_line.as_bytes())
            .map_err(|e| RuleError::Internal(format!("Failed to write to audit file: {}", e)))?;
        
        file.sync_all()
            .map_err(|e| RuleError::Internal(format!("Failed to sync audit file: {}", e)))?;
        
        Ok(())
    }
}

impl AuditLogger for FileAuditLogger {
    fn log_rule_registration(&self, rule_id: &str) -> Result<()> {
        let entry = AuditEntry {
            timestamp: Utc::now(),
            event_type: AuditEventType::RuleRegistration,
            message: format!("Rule registered: {}", rule_id),
            metadata: serde_json::json!({ "rule_id": rule_id }),
        };
        
        self.write_to_file(&entry)?;
        self.memory_logger.log_rule_registration(rule_id)?;
        Ok(())
    }

    fn log_rule_removal(&self, rule_id: &str) -> Result<()> {
        let entry = AuditEntry {
            timestamp: Utc::now(),
            event_type: AuditEventType::RuleRemoval,
            message: format!("Rule removed: {}", rule_id),
            metadata: serde_json::json!({ "rule_id": rule_id }),
        };
        
        self.write_to_file(&entry)?;
        self.memory_logger.log_rule_removal(rule_id)?;
        Ok(())
    }

    fn log_rule_violation(&self, violation: &RuleViolation) -> Result<()> {
        let entry = AuditEntry {
            timestamp: Utc::now(),
            event_type: AuditEventType::RuleViolation,
            message: format!("Rule violation: {}", violation.message),
            metadata: serde_json::json!({
                "rule_id": violation.rule_id,
                "severity": violation.severity,
                "context": violation.context
            }),
        };
        
        self.write_to_file(&entry)?;
        self.memory_logger.log_rule_violation(violation)?;
        Ok(())
    }

    fn log_evaluation_summary(&self, result: &RuleEvaluationResult) -> Result<()> {
        let entry = AuditEntry {
            timestamp: Utc::now(),
            event_type: AuditEventType::EvaluationCompleted,
            message: result.summary(),
            metadata: serde_json::json!({
                "rules_evaluated": result.rules_evaluated,
                "rules_passed": result.rules_passed,
                "rules_failed": result.rules_failed,
                "violations_count": result.violations.len(),
                "execution_time_ms": result.execution_time_ms,
                "has_critical_violations": result.has_critical_violations()
            }),
        };
        
        self.write_to_file(&entry)?;
        self.memory_logger.log_evaluation_summary(result)?;
        Ok(())
    }

    fn log_system_error(&self, error: &str) -> Result<()> {
        let entry = AuditEntry {
            timestamp: Utc::now(),
            event_type: AuditEventType::SystemError,
            message: format!("System error: {}", error),
            metadata: serde_json::json!({ "error": error }),
        };
        
        self.write_to_file(&entry)?;
        self.memory_logger.log_system_error(error)?;
        Ok(())
    }

    fn get_recent_entries(&self, limit: usize) -> Result<Vec<AuditEntry>> {
        // Return from memory cache for recent entries
        self.memory_logger.get_recent_entries(limit)
    }

    fn get_entries_by_type(&self, event_type: AuditEventType, limit: usize) -> Result<Vec<AuditEntry>> {
        // Return from memory cache for recent entries
        self.memory_logger.get_entries_by_type(event_type, limit)
    }
}

/// Audit log aggregator
pub struct AuditLog {
    logger: Arc<dyn AuditLogger>,
}

impl AuditLog {
    /// Create new audit log with given logger
    pub fn new(logger: Arc<dyn AuditLogger>) -> Self {
        Self { logger }
    }

    /// Create with memory logger
    pub fn with_memory_logger(max_entries: usize) -> Self {
        Self::new(Arc::new(MemoryAuditLogger::new(max_entries)))
    }

    /// Create with file logger
    pub fn with_file_logger(file_path: String) -> Self {
        Self::new(Arc::new(FileAuditLogger::new(file_path)))
    }

    /// Get the underlying logger
    pub fn logger(&self) -> Arc<dyn AuditLogger> {
        self.logger.clone()
    }

    /// Get violation statistics
    pub fn get_violation_stats(&self) -> Result<ViolationStats> {
        let violations = self.logger.get_entries_by_type(AuditEventType::RuleViolation, 1000)?;
        
        let mut stats = ViolationStats::default();
        stats.total_violations = violations.len();
        
        for entry in violations {
            if let Some(severity) = entry.metadata.get("severity") {
                if let Some(severity_str) = severity.as_str() {
                    match severity_str {
                        "Info" => stats.info_violations += 1,
                        "Warning" => stats.warning_violations += 1,
                        "Error" => stats.error_violations += 1,
                        "Critical" => stats.critical_violations += 1,
                        _ => {}
                    }
                }
            }
        }
        
        Ok(stats)
    }
}

/// Statistics about rule violations
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ViolationStats {
    pub total_violations: usize,
    pub info_violations: usize,
    pub warning_violations: usize,
    pub error_violations: usize,
    pub critical_violations: usize,
}

impl ViolationStats {
    pub fn summary(&self) -> String {
        format!(
            "Total: {}, Critical: {}, Error: {}, Warning: {}, Info: {}",
            self.total_violations,
            self.critical_violations,
            self.error_violations,
            self.warning_violations,
            self.info_violations
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::ViolationSeverity;

    #[test]
    fn test_memory_audit_logger() {
        let logger = MemoryAuditLogger::new(10);
        
        logger.log_rule_registration("test_rule").unwrap();
        assert_eq!(logger.entry_count().unwrap(), 1);
        
        let entries = logger.get_recent_entries(5).unwrap();
        assert_eq!(entries.len(), 1);
        assert!(matches!(entries[0].event_type, AuditEventType::RuleRegistration));
    }

    #[test]
    fn test_memory_logger_capacity() {
        let logger = MemoryAuditLogger::new(2);
        
        logger.log_rule_registration("rule1").unwrap();
        logger.log_rule_registration("rule2").unwrap();
        logger.log_rule_registration("rule3").unwrap();
        
        // Should only keep 2 entries (most recent)
        assert_eq!(logger.entry_count().unwrap(), 2);
    }

    #[test]
    fn test_violation_logging() {
        let logger = MemoryAuditLogger::new(10);
        
        let violation = RuleViolation {
            rule_id: "test_rule".to_string(),
            message: "Test violation".to_string(),
            severity: ViolationSeverity::Error,
            timestamp: Utc::now(),
            context: serde_json::json!({}),
        };
        
        logger.log_rule_violation(&violation).unwrap();
        
        let entries = logger.get_entries_by_type(AuditEventType::RuleViolation, 5).unwrap();
        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn test_audit_log_stats() {
        let audit_log = AuditLog::with_memory_logger(100);
        
        let violation = RuleViolation {
            rule_id: "test_rule".to_string(),
            message: "Test violation".to_string(),
            severity: ViolationSeverity::Critical,
            timestamp: Utc::now(),
            context: serde_json::json!({}),
        };
        
        audit_log.logger.log_rule_violation(&violation).unwrap();
        
        let stats = audit_log.get_violation_stats().unwrap();
        assert_eq!(stats.total_violations, 1);
        assert_eq!(stats.critical_violations, 1);
    }
}