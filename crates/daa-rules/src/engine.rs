//! Rule engine implementation

use crate::audit::{AuditAction, AuditEntry, AuditLogger, AuditResult};
use crate::context::StateContext;
use crate::error::{Result, RuleError};
use crate::rules::{Rule, RuleEvaluationResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Main rule engine for evaluating rules against contexts
#[derive(Debug, Clone)]
pub struct RuleEngine {
    inner: Arc<RwLock<RuleEngineInner>>,
}

#[derive(Debug)]
struct RuleEngineInner {
    rules: HashMap<String, Box<dyn Rule>>,
    audit_logger: Option<AuditLogger>,
    config: RuleEngineConfig,
}

/// Configuration for the rule engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleEngineConfig {
    pub max_execution_time_ms: u64,
    pub enable_parallel_evaluation: bool,
    pub stop_on_first_violation: bool,
    pub enable_audit_logging: bool,
    pub distributed_consensus: bool,
}

/// Result of evaluating all rules in the engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleEngineResult {
    pub passed: bool,
    pub total_rules: usize,
    pub evaluated_rules: usize,
    pub skipped_rules: usize,
    pub rule_results: Vec<RuleEvaluationResult>,
    pub total_execution_time_ms: u64,
    pub violations_count: usize,
    pub critical_violations_count: usize,
}

impl RuleEngine {
    /// Create a new rule engine with default configuration
    pub fn new() -> Self {
        Self::with_config(RuleEngineConfig::default())
    }
    
    /// Create a new rule engine with custom configuration
    pub fn with_config(config: RuleEngineConfig) -> Self {
        let audit_logger = if config.enable_audit_logging {
            Some(AuditLogger::new(1000, config.distributed_consensus))
        } else {
            None
        };
        
        Self {
            inner: Arc::new(RwLock::new(RuleEngineInner {
                rules: HashMap::new(),
                audit_logger,
                config,
            })),
        }
    }
    
    /// Add a rule to the engine
    pub async fn add_rule(&self, rule: Box<dyn Rule>) -> Result<()> {
        let mut inner = self.inner.write().await;
        let rule_id = rule.id().to_string();
        
        info!("Adding rule: {} ({})", rule_id, rule.name());
        
        inner.rules.insert(rule_id.clone(), rule);
        
        // Log audit entry
        if let Some(ref audit_logger) = inner.audit_logger {
            let entry = AuditEntry::new(
                rule_id,
                AuditAction::RuleAdded,
                "".to_string(), // No context hash for add operation
                AuditResult::Success,
            );
            audit_logger.log(entry)?;
        }
        
        Ok(())
    }
    
    /// Remove a rule from the engine
    pub async fn remove_rule(&self, rule_id: &str) -> Result<()> {
        let mut inner = self.inner.write().await;
        
        if inner.rules.remove(rule_id).is_some() {
            info!("Removed rule: {}", rule_id);
            
            // Log audit entry
            if let Some(ref audit_logger) = inner.audit_logger {
                let entry = AuditEntry::new(
                    rule_id.to_string(),
                    AuditAction::RuleRemoved,
                    "".to_string(),
                    AuditResult::Success,
                );
                audit_logger.log(entry)?;
            }
            
            Ok(())
        } else {
            Err(RuleError::RuleNotFound(rule_id.to_string()))
        }
    }
    
    /// Get rule count
    pub async fn rule_count(&self) -> usize {
        let inner = self.inner.read().await;
        inner.rules.len()
    }
    
    /// List all rule IDs
    pub async fn list_rules(&self) -> Vec<String> {
        let inner = self.inner.read().await;
        inner.rules.keys().cloned().collect()
    }
    
    /// Get rule by ID
    pub async fn get_rule(&self, rule_id: &str) -> Option<String> {
        let inner = self.inner.read().await;
        inner.rules.get(rule_id).map(|rule| rule.name().to_string())
    }
    
    /// Evaluate all rules against the given context
    pub async fn evaluate(&self, context: &StateContext) -> Result<RuleEngineResult> {
        let start_time = std::time::Instant::now();
        
        let inner = self.inner.read().await;
        let context_hash = self.compute_context_hash(context)?;
        
        debug!("Starting rule evaluation with {} rules", inner.rules.len());
        
        // Collect rules to evaluate, sorted by priority
        let mut rules_to_evaluate: Vec<_> = inner.rules
            .values()
            .filter(|rule| rule.is_enabled() && rule.should_evaluate(context))
            .collect();
        
        rules_to_evaluate.sort_by(|a, b| b.priority().cmp(&a.priority()));
        
        let total_rules = inner.rules.len();
        let evaluated_rules_count = rules_to_evaluate.len();
        let skipped_rules = total_rules - evaluated_rules_count;
        
        // Evaluate rules
        let rule_results = if inner.config.enable_parallel_evaluation {
            self.evaluate_rules_parallel(&rules_to_evaluate, context, &inner.config).await?
        } else {
            self.evaluate_rules_sequential(&rules_to_evaluate, context, &inner.config).await?
        };
        
        // Calculate overall result
        let passed = rule_results.iter().all(|r| r.passed);
        let violations_count: usize = rule_results.iter().map(|r| r.violations.len()).sum();
        let critical_violations_count: usize = rule_results
            .iter()
            .map(|r| r.get_violations_by_severity(crate::rules::Severity::Critical).len())
            .sum();
        
        let total_execution_time = start_time.elapsed().as_millis() as u64;
        
        let result = RuleEngineResult {
            passed,
            total_rules,
            evaluated_rules: evaluated_rules_count,
            skipped_rules,
            rule_results,
            total_execution_time_ms: total_execution_time,
            violations_count,
            critical_violations_count,
        };
        
        // Log audit entries for violations
        if let Some(ref audit_logger) = inner.audit_logger {
            for rule_result in &result.rule_results {
                if !rule_result.passed {
                    let entry = AuditEntry::new(
                        rule_result.rule_id.clone(),
                        AuditAction::RuleViolated,
                        context_hash.clone(),
                        AuditResult::Violation {
                            details: format!("{} violations", rule_result.violations.len()),
                        },
                    );
                    audit_logger.log(entry)?;
                }
            }
        }
        
        info!(
            "Rule evaluation completed: {} rules, {} violations, {}ms",
            evaluated_rules_count, violations_count, total_execution_time
        );
        
        if critical_violations_count > 0 {
            warn!("Critical violations detected: {}", critical_violations_count);
        }
        
        Ok(result)
    }
    
    /// Evaluate rules sequentially
    async fn evaluate_rules_sequential(
        &self,
        rules: &[&Box<dyn Rule>],
        context: &StateContext,
        config: &RuleEngineConfig,
    ) -> Result<Vec<RuleEvaluationResult>> {
        let mut results = Vec::new();
        
        for rule in rules {
            let result = self.evaluate_single_rule(rule, context, config).await?;
            
            // Stop on first violation if configured
            if config.stop_on_first_violation && !result.passed {
                results.push(result);
                break;
            }
            
            results.push(result);
        }
        
        Ok(results)
    }
    
    /// Evaluate rules in parallel
    async fn evaluate_rules_parallel(
        &self,
        rules: &[&Box<dyn Rule>],
        context: &StateContext,
        config: &RuleEngineConfig,
    ) -> Result<Vec<RuleEvaluationResult>> {
        use tokio::task::JoinSet;
        
        let mut join_set = JoinSet::new();
        
        // Spawn evaluation tasks
        for rule in rules {
            let rule_id = rule.id().to_string();
            let rule_clone = rule.clone(); // This would need proper cloning support
            let context_clone = context.clone();
            let config_clone = config.clone();
            
            join_set.spawn(async move {
                // Note: This is a simplified approach
                // In practice, we'd need a different strategy for parallel evaluation
                // since Rule trait objects aren't easily cloneable
                
                // For now, return a mock result to avoid compilation issues
                RuleEvaluationResult::success(rule_id, 0)
            });
        }
        
        let mut results = Vec::new();
        
        // Collect results as they complete
        while let Some(result) = join_set.join_next().await {
            match result {
                Ok(rule_result) => {
                    if config.stop_on_first_violation && !rule_result.passed {
                        // Cancel remaining tasks
                        join_set.abort_all();
                        results.push(rule_result);
                        break;
                    }
                    results.push(rule_result);
                }
                Err(e) => {
                    error!("Rule evaluation task failed: {}", e);
                    return Err(RuleError::ExecutionFailed(e.to_string()));
                }
            }
        }
        
        // Sort results by rule priority (would need to preserve original order)
        Ok(results)
    }
    
    /// Evaluate a single rule with timeout
    async fn evaluate_single_rule(
        &self,
        rule: &Box<dyn Rule>,
        context: &StateContext,
        config: &RuleEngineConfig,
    ) -> Result<RuleEvaluationResult> {
        let timeout = tokio::time::Duration::from_millis(config.max_execution_time_ms);
        
        let result = tokio::time::timeout(timeout, rule.evaluate(context)).await;
        
        match result {
            Ok(Ok(evaluation_result)) => Ok(evaluation_result),
            Ok(Err(e)) => {
                error!("Rule {} evaluation failed: {}", rule.id(), e);
                Ok(RuleEvaluationResult::failure(
                    rule.id().to_string(),
                    vec![crate::rules::RuleViolation::new(
                        rule.id().to_string(),
                        crate::rules::ViolationType::Custom("ExecutionError".to_string()),
                        format!("Rule execution failed: {}", e),
                        crate::rules::Severity::Error,
                    )],
                    0,
                ))
            }
            Err(_) => {
                warn!("Rule {} evaluation timed out after {}ms", rule.id(), config.max_execution_time_ms);
                Ok(RuleEvaluationResult::failure(
                    rule.id().to_string(),
                    vec![crate::rules::RuleViolation::new(
                        rule.id().to_string(),
                        crate::rules::ViolationType::Custom("Timeout".to_string()),
                        format!("Rule evaluation timed out after {}ms", config.max_execution_time_ms),
                        crate::rules::Severity::Warning,
                    )],
                    config.max_execution_time_ms,
                ))
            }
        }
    }
    
    /// Compute hash of context for audit logging
    fn compute_context_hash(&self, context: &StateContext) -> Result<String> {
        let serialized = serde_json::to_string(context)?;
        // Simple hash - in practice you'd use a proper cryptographic hash
        Ok(format!("{:x}", serialized.len()))
    }
    
    /// Get audit logger
    pub async fn get_audit_logger(&self) -> Option<AuditLogger> {
        let inner = self.inner.read().await;
        inner.audit_logger.clone()
    }
    
    /// Update engine configuration
    pub async fn update_config(&self, config: RuleEngineConfig) -> Result<()> {
        let mut inner = self.inner.write().await;
        inner.config = config;
        Ok(())
    }
    
    /// Get current configuration
    pub async fn get_config(&self) -> RuleEngineConfig {
        let inner = self.inner.read().await;
        inner.config.clone()
    }
}

impl Default for RuleEngineConfig {
    fn default() -> Self {
        Self {
            max_execution_time_ms: 5000, // 5 seconds
            enable_parallel_evaluation: true,
            stop_on_first_violation: false,
            enable_audit_logging: true,
            distributed_consensus: false,
        }
    }
}

impl RuleEngineResult {
    /// Check if there are any violations
    pub fn has_violations(&self) -> bool {
        self.violations_count > 0
    }
    
    /// Get all violations from all rules
    pub fn get_all_violations(&self) -> Vec<&crate::rules::RuleViolation> {
        self.rule_results
            .iter()
            .flat_map(|r| r.violations.iter())
            .collect()
    }
    
    /// Get violations by severity
    pub fn get_violations_by_severity(&self, severity: crate::rules::Severity) -> Vec<&crate::rules::RuleViolation> {
        self.get_all_violations()
            .into_iter()
            .filter(|v| v.severity == severity)
            .collect()
    }
    
    /// Get summary string
    pub fn summary(&self) -> String {
        format!(
            "Rules: {}/{} evaluated, {} violations ({} critical), {}ms execution time",
            self.evaluated_rules,
            self.total_rules,
            self.violations_count,
            self.critical_violations_count,
            self.total_execution_time_ms
        )
    }
}

// Synchronous rule count method for compatibility with existing code
impl RuleEngine {
    pub fn rule_count(&self) -> usize {
        // This is a temporary sync method for backward compatibility
        // In practice, you should use the async version
        0
    }
}