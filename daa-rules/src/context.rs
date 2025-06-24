//! Execution context for rules

use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Execution context containing variables and state for rule evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    /// Variables available to rules
    variables: HashMap<String, String>,
    
    /// Execution timestamp
    timestamp: DateTime<Utc>,
    
    /// Context metadata
    metadata: HashMap<String, String>,
}

impl ExecutionContext {
    /// Create a new execution context
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        }
    }

    /// Set a variable value
    pub fn set_variable(&mut self, key: String, value: String) {
        self.variables.insert(key, value);
    }

    /// Get a variable value
    pub fn get_variable(&self, key: &str) -> Option<&String> {
        self.variables.get(key)
    }

    /// Get all variables
    pub fn get_variables(&self) -> &HashMap<String, String> {
        &self.variables
    }

    /// Get execution timestamp
    pub fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    /// Set metadata
    pub fn set_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    /// Get metadata
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
}

impl Default for ExecutionContext {
    fn default() -> Self {
        Self::new()
    }
}