//! AI agent implementation for DAA with QuDAG MCP integration

use crate::error::{AIError, Result};
use crate::mcp::{McpAIClient, McpClientConfig, McpMessage, McpToolRequest};
use crate::tools::{DAAToolSet, ToolResult};
use async_trait::async_trait;
use daa_rules::RuleEngine;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// AI agent for DAA operations
#[derive(Debug, Clone)]
pub struct AIAgent {
    inner: Arc<RwLock<AIAgentInner>>,
}

#[derive(Debug)]
struct AIAgentInner {
    config: AIAgentConfig,
    mcp_client: Option<McpAIClient>,
    tool_set: DAAToolSet,
    session_id: String,
    conversation_history: Vec<ConversationEntry>,
}

/// Configuration for AI agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIAgentConfig {
    pub name: String,
    pub description: String,
    pub mcp_config: McpClientConfig,
    pub max_conversation_history: usize,
    pub enable_tool_execution: bool,
    pub enable_rule_validation: bool,
    pub capabilities: AgentCapabilities,
}

/// Agent capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCapabilities {
    pub can_execute_transactions: bool,
    pub can_query_balances: bool,
    pub can_evaluate_rules: bool,
    pub can_access_dag: bool,
    pub can_modify_rules: bool,
    pub supports_streaming: bool,
}

/// Query request to the AI agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryRequest {
    pub query: String,
    pub context: Option<HashMap<String, serde_json::Value>>,
    pub tools_allowed: Option<Vec<String>>,
    pub require_rule_validation: bool,
}

/// Response from the AI agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResponse {
    pub response: String,
    pub tool_calls: Vec<ToolCall>,
    pub rule_evaluations: Vec<RuleEvaluationSummary>,
    pub confidence: f64,
    pub reasoning: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// AI response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIResponse {
    pub success: bool,
    pub data: Option<QueryResponse>,
    pub error: Option<String>,
    pub execution_time_ms: u64,
}

/// Tool call made by the agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub tool_name: String,
    pub arguments: serde_json::Value,
    pub result: Option<ToolResult>,
    pub error: Option<String>,
}

/// Summary of rule evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleEvaluationSummary {
    pub rule_id: String,
    pub passed: bool,
    pub violations_count: usize,
    pub critical_violations: usize,
}

/// Conversation entry
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ConversationEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub role: String, // "user" or "assistant"
    pub content: String,
    pub tool_calls: Vec<ToolCall>,
}

impl AIAgent {
    /// Create a new AI agent
    pub async fn new(config: AIAgentConfig) -> Result<Self> {
        let session_id = uuid::Uuid::new_v4().to_string();
        
        info!("Creating AI agent: {} ({})", config.name, session_id);
        
        let agent = Self {
            inner: Arc::new(RwLock::new(AIAgentInner {
                config,
                mcp_client: None,
                tool_set: DAAToolSet::new(),
                session_id,
                conversation_history: Vec::new(),
            })),
        };
        
        Ok(agent)
    }
    
    /// Create AI agent with rule engine
    pub async fn with_rule_engine(config: AIAgentConfig, rule_engine: Arc<RuleEngine>) -> Result<Self> {
        let session_id = uuid::Uuid::new_v4().to_string();
        
        info!("Creating AI agent with rule engine: {} ({})", config.name, session_id);
        
        let tool_set = DAAToolSet::with_rule_engine(rule_engine);
        
        let agent = Self {
            inner: Arc::new(RwLock::new(AIAgentInner {
                config,
                mcp_client: None,
                tool_set,
                session_id,
                conversation_history: Vec::new(),
            })),
        };
        
        Ok(agent)
    }
    
    /// Initialize the agent (connect to MCP server)
    pub async fn initialize(&self) -> Result<()> {
        let mut inner = self.inner.write().await;
        
        info!("Initializing AI agent: {}", inner.config.name);
        
        // Create and connect MCP client
        let mcp_client = McpAIClient::new(inner.config.mcp_config.clone()).await?;
        mcp_client.connect().await?;
        
        // Register tools with MCP client
        for tool_def in inner.tool_set.list_tools() {
            mcp_client.register_tool(tool_def).await?;
        }
        
        inner.mcp_client = Some(mcp_client);
        
        info!("AI agent initialized successfully");
        Ok(())
    }
    
    /// Process a query from the user
    pub async fn query(&self, request: QueryRequest) -> Result<AIResponse> {
        let start_time = std::time::Instant::now();
        
        debug!("Processing query: {}", request.query);
        
        let mut inner = self.inner.write().await;
        
        // Check if agent is initialized
        let mcp_client = inner.mcp_client.as_ref()
            .ok_or(AIError::AgentNotInitialized)?;
        
        // Add user message to conversation history
        inner.conversation_history.push(ConversationEntry {
            timestamp: chrono::Utc::now(),
            role: "user".to_string(),
            content: request.query.clone(),
            tool_calls: Vec::new(),
        });
        
        // Prepare context for AI
        let mut context = request.context.unwrap_or_default();
        context.insert("agent_capabilities".to_string(), serde_json::to_value(&inner.config.capabilities)?);
        context.insert("available_tools".to_string(), serde_json::to_value(inner.tool_set.list_tools())?);
        
        // Create MCP message for the AI
        let mcp_message = McpMessage::request(
            serde_json::json!(1),
            "completion/create".to_string(),
            Some(serde_json::json!({
                "messages": [
                    {
                        "role": "system",
                        "content": self.build_system_prompt(&inner.config).await?
                    },
                    {
                        "role": "user", 
                        "content": request.query
                    }
                ],
                "tools": inner.tool_set.list_tools(),
                "context": context
            }))
        );
        
        drop(inner); // Release lock for async operations
        
        // Send request to AI via MCP
        let ai_response = mcp_client.send_request(mcp_message).await?;
        
        // Process AI response
        let response = self.process_ai_response(ai_response, &request).await?;
        
        // Update conversation history
        let mut inner = self.inner.write().await;
        inner.conversation_history.push(ConversationEntry {
            timestamp: chrono::Utc::now(),
            role: "assistant".to_string(),
            content: response.response.clone(),
            tool_calls: response.tool_calls.clone(),
        });
        
        // Maintain conversation history limit
        if inner.conversation_history.len() > inner.config.max_conversation_history {
            inner.conversation_history.drain(0..inner.conversation_history.len() - inner.config.max_conversation_history);
        }
        
        let execution_time = start_time.elapsed().as_millis() as u64;
        
        Ok(AIResponse {
            success: true,
            data: Some(response),
            error: None,
            execution_time_ms: execution_time,
        })
    }
    
    /// Build system prompt for the AI
    async fn build_system_prompt(&self, config: &AIAgentConfig) -> Result<String> {
        let prompt = format!(
            r#"You are {}, an AI agent for DAA (Decentralized Autonomous Application) with QuDAG integration.

Description: {}

Your capabilities:
- Transaction execution: {}
- Balance queries: {}
- Rule evaluation: {}
- DAG access: {}
- Rule modification: {}
- Streaming support: {}

Available tools:
- evaluate_rules: Evaluate DAA rules against contexts
- query_balance: Query account balances (native or rUv)
- execute_transaction: Execute blockchain transactions
- dag_operation: Perform DAG-related operations
- query_ruv_balance: Query QuDAG rUv balances

Guidelines:
1. Always prioritize safety and rule compliance
2. Use tools when appropriate to provide accurate information
3. Validate transactions against DAA rules before execution
4. Provide clear explanations of your reasoning
5. If uncertain, ask for clarification rather than guessing
6. Respect the user's tool restrictions if specified

Remember: You are operating within a QuDAG environment where rUv (recoverable Utility value) balances are critical for system operation."#,
            config.name,
            config.description,
            config.capabilities.can_execute_transactions,
            config.capabilities.can_query_balances,
            config.capabilities.can_evaluate_rules,
            config.capabilities.can_access_dag,
            config.capabilities.can_modify_rules,
            config.capabilities.supports_streaming,
        );
        
        Ok(prompt)
    }
    
    /// Process AI response and execute tools if needed
    async fn process_ai_response(&self, ai_response: McpMessage, request: &QueryRequest) -> Result<QueryResponse> {
        let mut tool_calls = Vec::new();
        let mut rule_evaluations = Vec::new();
        
        // Extract response content
        let response_content = ai_response.result
            .and_then(|r| r.get("content"))
            .and_then(|c| c.as_str())
            .unwrap_or("No response from AI")
            .to_string();
        
        // Extract tool calls if present
        if let Some(tools) = ai_response.result.and_then(|r| r.get("tool_calls")) {
            if let Ok(tool_requests): std::result::Result<Vec<McpToolRequest>, _> = serde_json::from_value(tools.clone()) {
                for tool_request in tool_requests {
                    // Check if tool is allowed
                    if let Some(allowed_tools) = &request.tools_allowed {
                        if !allowed_tools.contains(&tool_request.name) {
                            warn!("Tool {} not allowed for this request", tool_request.name);
                            continue;
                        }
                    }
                    
                    // Execute tool
                    let tool_result = self.execute_tool(tool_request.clone()).await;
                    
                    let tool_call = ToolCall {
                        tool_name: tool_request.name.clone(),
                        arguments: tool_request.arguments,
                        result: tool_result.as_ref().ok().cloned(),
                        error: tool_result.as_ref().err().map(|e| e.to_string()),
                    };
                    
                    tool_calls.push(tool_call);
                    
                    // If this was a rule evaluation, extract summary
                    if tool_request.name == "evaluate_rules" {
                        if let Ok(result) = tool_result {
                            if let Some(data) = result.data {
                                // Extract rule evaluation summary
                                // This would parse the actual rule evaluation results
                                rule_evaluations.push(RuleEvaluationSummary {
                                    rule_id: "example_rule".to_string(),
                                    passed: true,
                                    violations_count: 0,
                                    critical_violations: 0,
                                });
                            }
                        }
                    }
                }
            }
        }
        
        Ok(QueryResponse {
            response: response_content,
            tool_calls,
            rule_evaluations,
            confidence: 0.85, // Mock confidence score
            reasoning: Some("Processed request using available tools and knowledge".to_string()),
            metadata: HashMap::new(),
        })
    }
    
    /// Execute a tool
    async fn execute_tool(&self, request: McpToolRequest) -> Result<ToolResult> {
        let inner = self.inner.read().await;
        
        if !inner.config.enable_tool_execution {
            return Err(AIError::ToolExecutionError("Tool execution disabled".to_string()));
        }
        
        inner.tool_set.execute_tool(request).await
    }
    
    /// Get conversation history
    pub async fn get_conversation_history(&self) -> Vec<ConversationEntry> {
        let inner = self.inner.read().await;
        inner.conversation_history.clone()
    }
    
    /// Clear conversation history
    pub async fn clear_conversation_history(&self) -> Result<()> {
        let mut inner = self.inner.write().await;
        inner.conversation_history.clear();
        info!("Conversation history cleared");
        Ok(())
    }
    
    /// Get agent capabilities
    pub async fn get_capabilities(&self) -> AgentCapabilities {
        let inner = self.inner.read().await;
        inner.config.capabilities.clone()
    }
    
    /// Update agent configuration
    pub async fn update_config(&self, config: AIAgentConfig) -> Result<()> {
        let mut inner = self.inner.write().await;
        inner.config = config;
        info!("Agent configuration updated");
        Ok(())
    }
    
    /// Check if agent is connected
    pub async fn is_connected(&self) -> bool {
        let inner = self.inner.read().await;
        if let Some(mcp_client) = &inner.mcp_client {
            mcp_client.is_connected().await
        } else {
            false
        }
    }
    
    /// Disconnect agent
    pub async fn disconnect(&self) -> Result<()> {
        let mut inner = self.inner.write().await;
        
        if let Some(mcp_client) = &inner.mcp_client {
            mcp_client.disconnect().await?;
        }
        
        inner.mcp_client = None;
        info!("Agent disconnected");
        Ok(())
    }
}

impl Default for AIAgentConfig {
    fn default() -> Self {
        Self {
            name: "DAA AI Agent".to_string(),
            description: "AI agent for DAA operations with QuDAG integration".to_string(),
            mcp_config: McpClientConfig::default(),
            max_conversation_history: 100,
            enable_tool_execution: true,
            enable_rule_validation: true,
            capabilities: AgentCapabilities::default(),
        }
    }
}

impl Default for AgentCapabilities {
    fn default() -> Self {
        Self {
            can_execute_transactions: true,
            can_query_balances: true,
            can_evaluate_rules: true,
            can_access_dag: true,
            can_modify_rules: false, // Disabled by default for safety
            supports_streaming: true,
        }
    }
}

/// Trait for AI agent operations
#[async_trait]
pub trait AIAgentTrait: Send + Sync {
    /// Process a query
    async fn query(&self, request: QueryRequest) -> Result<AIResponse>;
    
    /// Initialize the agent
    async fn initialize(&self) -> Result<()>;
    
    /// Check if agent is ready
    async fn is_ready(&self) -> bool;
    
    /// Get agent capabilities
    async fn capabilities(&self) -> AgentCapabilities;
}

#[async_trait]
impl AIAgentTrait for AIAgent {
    async fn query(&self, request: QueryRequest) -> Result<AIResponse> {
        self.query(request).await
    }
    
    async fn initialize(&self) -> Result<()> {
        self.initialize().await
    }
    
    async fn is_ready(&self) -> bool {
        self.is_connected().await
    }
    
    async fn capabilities(&self) -> AgentCapabilities {
        self.get_capabilities().await
    }
}