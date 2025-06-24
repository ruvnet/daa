//! Event management

use serde::{Deserialize, Serialize};
use crate::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event {
    WorkflowCompleted {
        execution_id: String,
        result: crate::workflow::WorkflowResult,
    },
}

pub struct EventManager;

impl EventManager {
    pub fn new() -> Self { Self }
    
    pub async fn initialize(&mut self) -> Result<()> { Ok(()) }
    
    pub async fn publish_event(&self, _event: Event) -> Result<()> { Ok(()) }
    
    pub async fn get_event_count(&self) -> u64 { 0 }
}

impl Default for EventManager {
    fn default() -> Self { Self::new() }
}