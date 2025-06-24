//! Storage interface for rules

use std::collections::HashMap;
use crate::{Rule, Result, RulesError};

/// Storage interface for rules
#[async_trait::async_trait]
pub trait RuleStorage: Send + Sync {
    /// Store a rule
    async fn store_rule(&mut self, rule: Rule) -> Result<()>;
    
    /// Retrieve a rule by ID
    async fn get_rule(&self, rule_id: &str) -> Result<Option<Rule>>;
    
    /// Get all rules
    async fn get_all_rules(&self) -> Result<Vec<Rule>>;
    
    /// Delete a rule
    async fn delete_rule(&mut self, rule_id: &str) -> Result<()>;
    
    /// Update a rule
    async fn update_rule(&mut self, rule: Rule) -> Result<()>;
}

/// In-memory storage implementation
pub struct InMemoryStorage {
    rules: HashMap<String, Rule>,
}

impl InMemoryStorage {
    /// Create a new in-memory storage
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
        }
    }
}

impl Default for InMemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl RuleStorage for InMemoryStorage {
    async fn store_rule(&mut self, rule: Rule) -> Result<()> {
        self.rules.insert(rule.id.clone(), rule);
        Ok(())
    }
    
    async fn get_rule(&self, rule_id: &str) -> Result<Option<Rule>> {
        Ok(self.rules.get(rule_id).cloned())
    }
    
    async fn get_all_rules(&self) -> Result<Vec<Rule>> {
        Ok(self.rules.values().cloned().collect())
    }
    
    async fn delete_rule(&mut self, rule_id: &str) -> Result<()> {
        self.rules.remove(rule_id);
        Ok(())
    }
    
    async fn update_rule(&mut self, rule: Rule) -> Result<()> {
        self.rules.insert(rule.id.clone(), rule);
        Ok(())
    }
}