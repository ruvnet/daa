//! Audit logging for rule engine operations

use crate::error::{Result, RuleError};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::{Arc, RwLock};
use tracing::{debug, info, warn};

/// Audit logger for rule engine operations
#[derive(Debug, Clone)]
pub struct AuditLogger {
    inner: Arc<RwLock<AuditLoggerInner>>,
}

#[derive(Debug)]
struct AuditLoggerInner {
    entries: VecDeque<AuditEntry>,
    max_entries: usize,
    qudag_integration: bool,
}

/// Audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub rule_id: String,
    pub action: AuditAction,
    pub context_hash: String,
    pub result: AuditResult,
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

/// Audit actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditAction {
    RuleAdded,
    RuleRemoved,
    RuleEvaluated,
    RuleViolated,
    EngineStarted,
    EngineStopped,
    ContextUpdated,
    ConsensusStarted,
    ConsensusCompleted,
    ConsensusViolated,
}

/// Audit results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditResult {
    Success,
    Failure { error: String },
    Violation { details: String },
}

/// Audit log for querying entries
#[derive(Debug, Clone)]
pub struct AuditLog {
    logger: AuditLogger,
}

impl AuditLogger {
    /// Create a new audit logger
    pub fn new(max_entries: usize, qudag_integration: bool) -> Self {
        Self {
            inner: Arc::new(RwLock::new(AuditLoggerInner {
                entries: VecDeque::new(),
                max_entries,
                qudag_integration,
            })),
        }
    }
    
    /// Log an audit entry
    pub fn log(&self, entry: AuditEntry) -> Result<()> {
        let mut inner = self.inner.write()
            .map_err(|e| RuleError::AuditFailed(format!("Failed to acquire write lock: {}", e)))?;
        
        // Add entry to local log
        inner.entries.push_back(entry.clone());
        
        // Maintain max entries limit
        if inner.entries.len() > inner.max_entries {
            inner.entries.pop_front();
        }
        
        // Log to tracing
        match &entry.result {
            AuditResult::Success => {
                info!(
                    rule_id = %entry.rule_id,
                    action = ?entry.action,
                    "Rule audit entry logged"
                );
            }
            AuditResult::Failure { error } => {
                warn!(
                    rule_id = %entry.rule_id,
                    action = ?entry.action,
                    error = %error,
                    "Rule failure logged"
                );
            }
            AuditResult::Violation { details } => {
                warn!(
                    rule_id = %entry.rule_id,
                    action = ?entry.action,
                    details = %details,
                    "Rule violation logged"
                );
            }
        }
        
        // If QuDAG integration is enabled, forward to QuDAG audit system
        if inner.qudag_integration {
            self.forward_to_qudag(&entry)?;
        }
        
        Ok(())
    }
    
    /// Get all audit entries
    pub fn get_entries(&self) -> Result<Vec<AuditEntry>> {
        let inner = self.inner.read()
            .map_err(|e| RuleError::AuditFailed(format!("Failed to acquire read lock: {}", e)))?;
        
        Ok(inner.entries.iter().cloned().collect())
    }
    
    /// Get entries for a specific rule
    pub fn get_entries_for_rule(&self, rule_id: &str) -> Result<Vec<AuditEntry>> {
        let inner = self.inner.read()
            .map_err(|e| RuleError::AuditFailed(format!("Failed to acquire read lock: {}", e)))?;
        
        Ok(inner.entries
            .iter()
            .filter(|entry| entry.rule_id == rule_id)
            .cloned()
            .collect())
    }
    
    /// Get entries by action type
    pub fn get_entries_by_action(&self, action: &AuditAction) -> Result<Vec<AuditEntry>> {
        let inner = self.inner.read()
            .map_err(|e| RuleError::AuditFailed(format!("Failed to acquire read lock: {}", e)))?;
        
        Ok(inner.entries
            .iter()
            .filter(|entry| std::mem::discriminant(&entry.action) == std::mem::discriminant(action))
            .cloned()
            .collect())
    }
    
    /// Clear all audit entries
    pub fn clear(&self) -> Result<()> {
        let mut inner = self.inner.write()
            .map_err(|e| RuleError::AuditFailed(format!("Failed to acquire write lock: {}", e)))?;
        
        inner.entries.clear();
        info!("Audit log cleared");
        Ok(())
    }
    
    /// Forward audit entry to QuDAG audit system
    fn forward_to_qudag(&self, entry: &AuditEntry) -> Result<()> {
        // TODO: Implement actual QuDAG audit forwarding
        // This would integrate with QuDAG's audit logging system
        debug!(
            entry_id = %entry.id,
            "Forwarding audit entry to QuDAG audit system"
        );
        
        // For now, just log that we would forward it
        // In a real implementation, this would:
        // 1. Serialize the entry to QuDAG format
        // 2. Submit to QuDAG audit log
        // 3. Handle any errors from QuDAG
        
        Ok(())
    }
}

impl AuditLog {
    /// Create a new audit log
    pub fn new(logger: AuditLogger) -> Self {
        Self { logger }
    }
    
    /// Query entries with filters
    pub fn query(&self, filter: AuditFilter) -> Result<Vec<AuditEntry>> {
        let entries = self.logger.get_entries()?;
        
        let filtered: Vec<AuditEntry> = entries
            .into_iter()
            .filter(|entry| {
                // Filter by rule ID
                if let Some(rule_id) = &filter.rule_id {
                    if &entry.rule_id != rule_id {
                        return false;
                    }
                }
                
                // Filter by action
                if let Some(action) = &filter.action {
                    if std::mem::discriminant(&entry.action) != std::mem::discriminant(action) {
                        return false;
                    }
                }
                
                // Filter by time range
                if let Some(start) = filter.start_time {
                    if entry.timestamp < start {
                        return false;
                    }
                }
                
                if let Some(end) = filter.end_time {
                    if entry.timestamp > end {
                        return false;
                    }
                }
                
                // Filter by result type
                if let Some(result_type) = &filter.result_type {
                    match (result_type, &entry.result) {
                        (AuditResultType::Success, AuditResult::Success) => {},
                        (AuditResultType::Failure, AuditResult::Failure { .. }) => {},
                        (AuditResultType::Violation, AuditResult::Violation { .. }) => {},
                        _ => return false,
                    }
                }
                
                true
            })
            .collect();
        
        Ok(filtered)
    }
}

/// Filter for querying audit entries
#[derive(Debug, Clone, Default)]
pub struct AuditFilter {
    pub rule_id: Option<String>,
    pub action: Option<AuditAction>,
    pub start_time: Option<chrono::DateTime<chrono::Utc>>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    pub result_type: Option<AuditResultType>,
}

/// Audit result types for filtering
#[derive(Debug, Clone)]
pub enum AuditResultType {
    Success,
    Failure,
    Violation,
}

impl AuditEntry {
    /// Create a new audit entry
    pub fn new(
        rule_id: String,
        action: AuditAction,
        context_hash: String,
        result: AuditResult,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            rule_id,
            action,
            context_hash,
            result,
            metadata: std::collections::HashMap::new(),
        }
    }
    
    /// Add metadata to the entry
    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

impl Default for AuditLogger {
    fn default() -> Self {
        Self::new(1000, false) // Default: 1000 entries, no QuDAG integration
    }
}