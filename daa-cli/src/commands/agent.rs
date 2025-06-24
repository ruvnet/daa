//! Agent command implementation

use anyhow::Result;
use colorful::Colorful;

use crate::{Cli, config::CliConfig, AgentAction};

/// Handle the agent command
pub async fn handle_agent(
    action: AgentAction,
    config: &CliConfig,
    cli: &Cli,
) -> Result<()> {
    match action {
        AgentAction::List => {
            let agents = get_agents().await?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&agents)?);
            } else {
                display_agents(&agents);
            }
        }
        AgentAction::Show { agent_id } => {
            let agent = get_agent_details(&agent_id).await?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&agent)?);
            } else {
                display_agent_details(&agent);
            }
        }
        AgentAction::Create { name, agent_type, capabilities } => {
            let agent_id = create_agent(name, agent_type, capabilities).await?;
            if cli.json {
                println!("{}", serde_json::json!({
                    "status": "created",
                    "agent_id": agent_id
                }));
            } else {
                println!("{}", "✓ Agent created successfully".green());
                println!("Agent ID: {}", agent_id);
            }
        }
        AgentAction::Stop { agent_id, force } => {
            stop_agent(&agent_id, force).await?;
            if cli.json {
                println!("{}", serde_json::json!({
                    "status": "stopped",
                    "agent_id": agent_id
                }));
            } else {
                println!("{}", "✓ Agent stopped".yellow());
            }
        }
        AgentAction::Restart { agent_id } => {
            restart_agent(&agent_id).await?;
            if cli.json {
                println!("{}", serde_json::json!({
                    "status": "restarted",
                    "agent_id": agent_id
                }));
            } else {
                println!("{}", "✓ Agent restarted".green());
            }
        }
    }
    
    Ok(())
}

async fn get_agents() -> Result<Vec<AgentInfo>> {
    Ok(vec![
        AgentInfo {
            id: "agent-001".to_string(),
            name: "Market Analyzer".to_string(),
            agent_type: "analyzer".to_string(),
            state: "Active".to_string(),
            capabilities: vec!["DataAnalysis".to_string(), "MarketAnalysis".to_string()],
            tasks_completed: 42,
            uptime: "1h 23m".to_string(),
        },
        AgentInfo {
            id: "agent-002".to_string(),
            name: "Risk Manager".to_string(),
            agent_type: "risk".to_string(),
            state: "Idle".to_string(),
            capabilities: vec!["RiskAssessment".to_string()],
            tasks_completed: 15,
            uptime: "45m".to_string(),
        },
    ])
}

async fn get_agent_details(agent_id: &str) -> Result<AgentDetails> {
    Ok(AgentDetails {
        id: agent_id.to_string(),
        name: "Market Analyzer".to_string(),
        agent_type: "analyzer".to_string(),
        state: "Active".to_string(),
        capabilities: vec!["DataAnalysis".to_string(), "MarketAnalysis".to_string()],
        tasks_completed: 42,
        tasks_failed: 2,
        success_rate: 95.5,
        uptime: "1h 23m".to_string(),
        current_task: Some("Analyzing BTC price trends".to_string()),
        queue_length: 3,
    })
}

async fn create_agent(name: String, agent_type: String, capabilities: Option<String>) -> Result<String> {
    let agent_id = format!("agent-{}", uuid::Uuid::new_v4().to_string()[..8].to_uppercase());
    // Mock agent creation
    Ok(agent_id)
}

async fn stop_agent(agent_id: &str, force: bool) -> Result<()> {
    // Mock agent stopping
    Ok(())
}

async fn restart_agent(agent_id: &str) -> Result<()> {
    // Mock agent restarting
    Ok(())
}

fn display_agents(agents: &[AgentInfo]) {
    println!("{}", "DAA Agents".blue().bold());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    for agent in agents {
        let state_color = match agent.state.as_str() {
            "Active" => agent.state.green(),
            "Idle" => agent.state.yellow(),
            "Busy" => agent.state.blue(),
            "Error" => agent.state.red(),
            _ => agent.state.white(),
        };
        
        println!("{} - {} ({})", agent.id, agent.name, state_color);
        println!("  Type: {} | Tasks: {} | Uptime: {}", 
                 agent.agent_type, agent.tasks_completed, agent.uptime);
        println!("  Capabilities: {}", agent.capabilities.join(", "));
        println!();
    }
}

fn display_agent_details(agent: &AgentDetails) {
    println!("{}", format!("Agent Details: {}", agent.name).blue().bold());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    let state_color = match agent.state.as_str() {
        "Active" => agent.state.green(),
        "Idle" => agent.state.yellow(),
        "Busy" => agent.state.blue(),
        "Error" => agent.state.red(),
        _ => agent.state.white(),
    };
    
    println!("ID:              {}", agent.id);
    println!("Name:            {}", agent.name);
    println!("Type:            {}", agent.agent_type);
    println!("State:           {}", state_color);
    println!("Uptime:          {}", agent.uptime);
    println!("Tasks Completed: {}", agent.tasks_completed);
    println!("Tasks Failed:    {}", agent.tasks_failed);
    println!("Success Rate:    {:.1}%", agent.success_rate);
    println!("Queue Length:    {}", agent.queue_length);
    
    if let Some(ref task) = agent.current_task {
        println!("Current Task:    {}", task);
    }
    
    println!("Capabilities:    {}", agent.capabilities.join(", "));
}

#[derive(serde::Serialize)]
struct AgentInfo {
    id: String,
    name: String,
    agent_type: String,
    state: String,
    capabilities: Vec<String>,
    tasks_completed: u32,
    uptime: String,
}

#[derive(serde::Serialize)]
struct AgentDetails {
    id: String,
    name: String,
    agent_type: String,
    state: String,
    capabilities: Vec<String>,
    tasks_completed: u32,
    tasks_failed: u32,
    success_rate: f64,
    uptime: String,
    current_task: Option<String>,
    queue_length: u32,
}