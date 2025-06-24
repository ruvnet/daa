//! Agent management for DAA AI

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::{Result, AIError, AgentConfig};

/// Agent types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentType {
    Researcher,
    Coder,
    Analyst,
    Coordinator,
    Specialist(String),
}

impl std::fmt::Display for AgentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentType::Researcher => write!(f, "Researcher"),
            AgentType::Coder => write!(f, "Coder"),
            AgentType::Analyst => write!(f, "Analyst"),
            AgentType::Coordinator => write!(f, "Coordinator"),
            AgentType::Specialist(s) => write!(f, "Specialist({})", s),
        }
    }
}

/// AI Agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: String,
    pub agent_type: AgentType,
    pub capabilities: Vec<String>,
    pub config: HashMap<String, String>,
}

/// Agent manager
pub struct AgentManager {
    config: AgentConfig,
    agents: HashMap<String, Agent>,
}

impl AgentManager {
    pub fn new(config: AgentConfig) -> Self {
        Self {
            config,
            agents: HashMap::new(),
        }
    }

    pub async fn spawn_agent(
        &mut self,
        agent_type: AgentType,
        capabilities: Vec<String>,
        custom_config: Option<HashMap<String, String>>,
    ) -> Result<Agent> {
        let agent = Agent {
            id: Uuid::new_v4().to_string(),
            agent_type,
            capabilities,
            config: custom_config.unwrap_or_default(),
        };

        self.agents.insert(agent.id.clone(), agent.clone());
        Ok(agent)
    }

    pub async fn get_agent(&self, agent_id: &str) -> Result<Agent> {
        self.agents
            .get(agent_id)
            .cloned()
            .ok_or_else(|| AIError::AgentNotFound(agent_id.to_string()))
    }

    pub async fn get_agent_count(&self) -> u64 {
        self.agents.len() as u64
    }
}