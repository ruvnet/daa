//! Bridge between DAA Chain and DAA Rules system
//! This module provides integration between the blockchain layer and the rules engine

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use async_trait::async_trait;

use daa_rules::{Rule, RuleEngine, RuleResult, ExecutionContext};
use crate::qudag_stubs::qudag_core::{Block, Transaction, Hash};

use crate::{Result, ChainError};
use crate::transaction::{DaaTransaction, TransactionType};

/// Bridge between chain and rules system
pub struct RulesBridge {
    /// Rules engine instance
    rule_engine: RuleEngine,
    
    /// Chain-specific rule configurations
    chain_rules: HashMap<String, ChainRule>,
}

/// Chain-specific rule definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainRule {
    /// Rule identifier
    pub id: String,
    
    /// Rule name
    pub name: String,
    
    /// When this rule should be triggered
    pub trigger: RuleTrigger,
    
    /// Rule conditions
    pub conditions: Vec<RuleCondition>,
    
    /// Actions to take when rule matches
    pub actions: Vec<RuleAction>,
    
    /// Whether rule is enabled
    pub enabled: bool,
}

/// When a chain rule should be triggered
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleTrigger {
    /// Before transaction validation
    BeforeTransactionValidation,
    
    /// After transaction validation
    AfterTransactionValidation,
    
    /// Before block validation
    BeforeBlockValidation,
    
    /// After block validation
    AfterBlockValidation,
    
    /// On transaction execution
    OnTransactionExecution,
    
    /// On block commit
    OnBlockCommit,
    
    /// On consensus vote
    OnConsensusVote,
    
    /// Custom trigger with parameters
    Custom(String, HashMap<String, String>),
}

/// Rule condition for chain operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleCondition {
    /// Transaction type matches
    TransactionType(String),
    
    /// Transaction amount greater than
    AmountGreaterThan(u64),
    
    /// Transaction amount less than
    AmountLessThan(u64),
    
    /// From specific agent
    FromAgent(String),
    
    /// To specific agent
    ToAgent(String),
    
    /// Block size greater than
    BlockSizeGreaterThan(usize),
    
    /// Custom condition with parameters
    Custom(String, HashMap<String, String>),
}

/// Action to take when rule matches
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleAction {
    /// Allow the operation
    Allow,
    
    /// Deny the operation
    Deny(String),
    
    /// Modify transaction parameters
    ModifyTransaction(HashMap<String, String>),
    
    /// Add fee to transaction
    AddFee(u64),
    
    /// Log event
    LogEvent(String),
    
    /// Send notification
    SendNotification {
        recipient: String,
        message: String,
    },
    
    /// Custom action with parameters
    Custom(String, HashMap<String, String>),
}

/// Context for rule execution in chain operations
#[derive(Debug, Clone)]
pub struct ChainExecutionContext {
    /// Current block being processed
    pub current_block: Option<Block>,
    
    /// Transaction being processed
    pub current_transaction: Option<DaaTransaction>,
    
    /// Chain height
    pub chain_height: u64,
    
    /// Current timestamp
    pub timestamp: u64,
    
    /// Additional context data
    pub metadata: HashMap<String, String>,
}

impl RulesBridge {
    /// Create new rules bridge
    pub fn new(rule_engine: RuleEngine) -> Self {
        Self {
            rule_engine,
            chain_rules: HashMap::new(),
        }
    }

    /// Add a chain rule
    pub fn add_rule(&mut self, rule: ChainRule) {
        self.chain_rules.insert(rule.id.clone(), rule);
    }

    /// Remove a chain rule
    pub fn remove_rule(&mut self, rule_id: &str) {
        self.chain_rules.remove(rule_id);
    }

    /// Validate transaction using rules
    pub async fn validate_transaction(
        &mut self,
        transaction: &DaaTransaction,
        context: &ChainExecutionContext,
    ) -> Result<ValidationResult> {
        let mut results = Vec::new();
        
        // Find applicable rules
        let applicable_rules = self.find_applicable_rules(&RuleTrigger::BeforeTransactionValidation);
        
        for rule in applicable_rules {
            if self.evaluate_conditions(&rule.conditions, transaction, context).await? {
                let result = self.execute_actions(&rule.actions, transaction, context).await?;
                results.push((rule.id.clone(), result));
            }
        }
        
        Ok(ValidationResult::new(results))
    }

    /// Validate block using rules
    pub async fn validate_block(
        &mut self,
        block: &Block,
        context: &ChainExecutionContext,
    ) -> Result<ValidationResult> {
        let mut results = Vec::new();
        
        // Find applicable rules
        let applicable_rules = self.find_applicable_rules(&RuleTrigger::BeforeBlockValidation);
        
        for rule in applicable_rules {
            if self.evaluate_block_conditions(&rule.conditions, block, context).await? {
                let result = self.execute_block_actions(&rule.actions, block, context).await?;
                results.push((rule.id.clone(), result));
            }
        }
        
        Ok(ValidationResult::new(results))
    }

    /// Process transaction execution through rules
    pub async fn process_transaction_execution(
        &mut self,
        transaction: &DaaTransaction,
        context: &ChainExecutionContext,
    ) -> Result<ExecutionResult> {
        let applicable_rules = self.find_applicable_rules(&RuleTrigger::OnTransactionExecution);
        let mut execution_context = self.create_execution_context(context)?;
        
        for rule in applicable_rules {
            if self.evaluate_conditions(&rule.conditions, transaction, context).await? {
                // Execute rule through rules engine
                let rule_def = self.convert_to_rule_engine_rule(&rule)?;
                let result = self.rule_engine.execute_rule(&rule_def, &mut execution_context).await
                    .map_err(|e| ChainError::Storage(format!("Rule execution failed: {}", e)))?;
                
                match result {
                    RuleResult::Allow => continue,
                    RuleResult::Deny(reason) => {
                        return Ok(ExecutionResult::Denied(reason));
                    }
                    RuleResult::Modified(data) => {
                        return Ok(ExecutionResult::Modified(data));
                    }
                }
            }
        }
        
        Ok(ExecutionResult::Allowed)
    }

    /// Find rules applicable to a trigger
    fn find_applicable_rules(&self, trigger: &RuleTrigger) -> Vec<&ChainRule> {
        self.chain_rules
            .values()
            .filter(|rule| rule.enabled && std::mem::discriminant(&rule.trigger) == std::mem::discriminant(trigger))
            .collect()
    }

    /// Evaluate rule conditions for transaction
    async fn evaluate_conditions(
        &self,
        conditions: &[RuleCondition],
        transaction: &DaaTransaction,
        context: &ChainExecutionContext,
    ) -> Result<bool> {
        for condition in conditions {
            match condition {
                RuleCondition::TransactionType(expected_type) => {
                    let actual_type = match &transaction.data.transaction_type {
                        TransactionType::AgentRegistration { .. } => "AgentRegistration",
                        TransactionType::ResourceAllocation { .. } => "ResourceAllocation",
                        TransactionType::TaskAssignment { .. } => "TaskAssignment",
                        TransactionType::RewardDistribution { .. } => "RewardDistribution",
                        TransactionType::Data { .. } => "Data",
                    };
                    
                    if actual_type != expected_type {
                        return Ok(false);
                    }
                }
                
                RuleCondition::AmountGreaterThan(threshold) => {
                    let amount = self.extract_amount_from_transaction(transaction)?;
                    if amount <= *threshold {
                        return Ok(false);
                    }
                }
                
                RuleCondition::AmountLessThan(threshold) => {
                    let amount = self.extract_amount_from_transaction(transaction)?;
                    if amount >= *threshold {
                        return Ok(false);
                    }
                }
                
                RuleCondition::Custom(condition_type, params) => {
                    // Delegate to rules engine for custom conditions
                    let rule_condition = daa_rules::RuleCondition::Custom(condition_type.clone(), params.clone());
                    let mut exec_context = self.create_execution_context(context)?;
                    let result = self.rule_engine.evaluate_condition(&rule_condition, &exec_context).await
                        .map_err(|e| ChainError::Storage(format!("Condition evaluation failed: {}", e)))?;
                    
                    if !result {
                        return Ok(false);
                    }
                }
                
                _ => {
                    // Handle other condition types
                    continue;
                }
            }
        }
        
        Ok(true)
    }

    /// Evaluate rule conditions for block
    async fn evaluate_block_conditions(
        &self,
        conditions: &[RuleCondition],
        block: &Block,
        context: &ChainExecutionContext,
    ) -> Result<bool> {
        for condition in conditions {
            match condition {
                RuleCondition::BlockSizeGreaterThan(threshold) => {
                    let block_size = block.transactions().len();
                    if block_size <= *threshold {
                        return Ok(false);
                    }
                }
                
                _ => {
                    // Handle other condition types or delegate to transaction conditions
                    continue;
                }
            }
        }
        
        Ok(true)
    }

    /// Execute rule actions for transaction
    async fn execute_actions(
        &self,
        actions: &[RuleAction],
        transaction: &DaaTransaction,
        context: &ChainExecutionContext,
    ) -> Result<ActionResult> {
        for action in actions {
            match action {
                RuleAction::Allow => {
                    return Ok(ActionResult::Allow);
                }
                
                RuleAction::Deny(reason) => {
                    return Ok(ActionResult::Deny(reason.clone()));
                }
                
                RuleAction::ModifyTransaction(modifications) => {
                    return Ok(ActionResult::Modify(modifications.clone()));
                }
                
                RuleAction::AddFee(fee) => {
                    let mut modifications = HashMap::new();
                    modifications.insert("additional_fee".to_string(), fee.to_string());
                    return Ok(ActionResult::Modify(modifications));
                }
                
                RuleAction::LogEvent(message) => {
                    tracing::info!("Chain rule log: {}", message);
                }
                
                RuleAction::SendNotification { recipient, message } => {
                    // Would integrate with notification system
                    tracing::info!("Notification to {}: {}", recipient, message);
                }
                
                RuleAction::Custom(action_type, params) => {
                    // Delegate to rules engine
                    let rule_action = daa_rules::RuleAction::Custom(action_type.clone(), params.clone());
                    let mut exec_context = self.create_execution_context(context)?;
                    self.rule_engine.execute_action(&rule_action, &mut exec_context).await
                        .map_err(|e| ChainError::Storage(format!("Action execution failed: {}", e)))?;
                }
            }
        }
        
        Ok(ActionResult::Allow)
    }

    /// Execute rule actions for block
    async fn execute_block_actions(
        &self,
        actions: &[RuleAction],
        block: &Block,
        context: &ChainExecutionContext,
    ) -> Result<ActionResult> {
        // Similar to execute_actions but for block context
        for action in actions {
            match action {
                RuleAction::LogEvent(message) => {
                    tracing::info!("Block rule log: {}", message);
                }
                
                _ => {
                    // Handle other actions
                    continue;
                }
            }
        }
        
        Ok(ActionResult::Allow)
    }

    /// Extract amount from transaction
    fn extract_amount_from_transaction(&self, transaction: &DaaTransaction) -> Result<u64> {
        match &transaction.data.transaction_type {
            TransactionType::ResourceAllocation { amount, .. } => Ok(*amount),
            TransactionType::RewardDistribution { amount, .. } => Ok(*amount),
            _ => Ok(0), // No amount for other transaction types
        }
    }

    /// Create execution context for rules engine
    fn create_execution_context(&self, chain_context: &ChainExecutionContext) -> Result<ExecutionContext> {
        let mut context = ExecutionContext::new();
        
        // Add chain-specific context
        context.set_variable("chain_height".to_string(), chain_context.chain_height.to_string());
        context.set_variable("timestamp".to_string(), chain_context.timestamp.to_string());
        
        // Add metadata
        for (key, value) in &chain_context.metadata {
            context.set_variable(key.clone(), value.clone());
        }
        
        Ok(context)
    }

    /// Convert chain rule to rules engine rule
    fn convert_to_rule_engine_rule(&self, chain_rule: &ChainRule) -> Result<Rule> {
        // Convert ChainRule to daa_rules::Rule
        // This would involve mapping conditions and actions
        Ok(Rule::new(
            chain_rule.id.clone(),
            chain_rule.name.clone(),
            Vec::new(), // Would convert conditions
            Vec::new(), // Would convert actions
        ))
    }
}

/// Result of rule validation
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub rule_results: Vec<(String, ActionResult)>,
}

impl ValidationResult {
    pub fn new(results: Vec<(String, ActionResult)>) -> Self {
        Self {
            rule_results: results,
        }
    }

    pub fn is_allowed(&self) -> bool {
        !self.rule_results.iter().any(|(_, result)| matches!(result, ActionResult::Deny(_)))
    }

    pub fn get_modifications(&self) -> HashMap<String, String> {
        let mut modifications = HashMap::new();
        
        for (_, result) in &self.rule_results {
            if let ActionResult::Modify(mods) = result {
                modifications.extend(mods.clone());
            }
        }
        
        modifications
    }
}

/// Result of rule action execution
#[derive(Debug, Clone)]
pub enum ActionResult {
    Allow,
    Deny(String),
    Modify(HashMap<String, String>),
}

/// Result of transaction execution with rules
#[derive(Debug, Clone)]
pub enum ExecutionResult {
    Allowed,
    Denied(String),
    Modified(HashMap<String, String>),
}

#[cfg(test)]
mod tests {
    use super::*;
    use daa_rules::RuleEngine;

    #[tokio::test]
    async fn test_rules_bridge_creation() {
        let rule_engine = RuleEngine::new();
        let bridge = RulesBridge::new(rule_engine);
        assert_eq!(bridge.chain_rules.len(), 0);
    }

    #[tokio::test]
    async fn test_add_chain_rule() {
        let rule_engine = RuleEngine::new();
        let mut bridge = RulesBridge::new(rule_engine);
        
        let rule = ChainRule {
            id: "test-rule".to_string(),
            name: "Test Rule".to_string(),
            trigger: RuleTrigger::BeforeTransactionValidation,
            conditions: vec![RuleCondition::TransactionType("Data".to_string())],
            actions: vec![RuleAction::Allow],
            enabled: true,
        };
        
        bridge.add_rule(rule);
        assert_eq!(bridge.chain_rules.len(), 1);
    }
}