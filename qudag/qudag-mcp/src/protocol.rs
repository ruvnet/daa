//! MCP Protocol message handling and JSON-RPC implementation

use crate::error::{Error, Result};
use crate::types::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// JSON-RPC 2.0 request message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPRequest {
    pub jsonrpc: String,
    pub id: RequestId,
    pub method: String,
    pub params: Option<serde_json::Value>,
}

impl MCPRequest {
    pub fn new(id: impl Into<RequestId>, method: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: id.into(),
            method: method.into(),
            params: None,
        }
    }

    pub fn with_params(mut self, params: serde_json::Value) -> Self {
        self.params = Some(params);
        self
    }

    /// Create an initialize request
    pub fn initialize(
        client_info: ClientInfo,
        capabilities: HashMap<String, serde_json::Value>,
    ) -> Self {
        Self::new(RequestId::generate(), "initialize").with_params(serde_json::json!({
            "protocolVersion": crate::MCP_PROTOCOL_VERSION,
            "capabilities": capabilities,
            "clientInfo": client_info
        }))
    }

    /// Create a tools/list request
    pub fn list_tools() -> Self {
        Self::new(RequestId::generate(), "tools/list")
    }

    /// Create a tools/call request
    pub fn call_tool(name: impl Into<String>, arguments: serde_json::Value) -> Self {
        Self::new(RequestId::generate(), "tools/call").with_params(serde_json::json!({
            "name": name.into(),
            "arguments": arguments
        }))
    }

    /// Create a resources/list request
    pub fn list_resources() -> Self {
        Self::new(RequestId::generate(), "resources/list")
    }

    /// Create a resources/read request
    pub fn read_resource(uri: impl Into<String>) -> Self {
        Self::new(RequestId::generate(), "resources/read").with_params(serde_json::json!({
            "uri": uri.into()
        }))
    }

    /// Create a prompts/list request
    pub fn list_prompts() -> Self {
        Self::new(RequestId::generate(), "prompts/list")
    }

    /// Create a prompts/get request
    pub fn get_prompt(name: impl Into<String>, arguments: Option<HashMap<String, String>>) -> Self {
        let mut params = serde_json::json!({
            "name": name.into()
        });
        if let Some(args) = arguments {
            params["arguments"] = serde_json::to_value(args).unwrap();
        }
        Self::new(RequestId::generate(), "prompts/get").with_params(params)
    }
}

/// JSON-RPC 2.0 response message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPResponse {
    pub jsonrpc: String,
    pub id: RequestId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<serde_json::Value>,
}

impl MCPResponse {
    pub fn success(id: RequestId, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn error(id: RequestId, error: Error) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(error.to_json_rpc_error()),
        }
    }

    /// Create an initialize response
    pub fn initialize_success(
        id: RequestId,
        server_info: ServerInfo,
        capabilities: ServerCapabilities,
    ) -> Self {
        Self::success(
            id,
            serde_json::json!({
                "protocolVersion": crate::MCP_PROTOCOL_VERSION,
                "capabilities": capabilities,
                "serverInfo": server_info
            }),
        )
    }

    /// Create a tools/list response
    pub fn tools_list_success(id: RequestId, tools: Vec<Tool>) -> Self {
        Self::success(
            id,
            serde_json::json!({
                "tools": tools
            }),
        )
    }

    /// Create a tools/call response
    pub fn tool_call_success(id: RequestId, result: ToolResult) -> Self {
        Self::success(id, serde_json::to_value(result).unwrap())
    }

    /// Create a resources/list response
    pub fn resources_list_success(id: RequestId, resources: Vec<Resource>) -> Self {
        Self::success(
            id,
            serde_json::json!({
                "resources": resources
            }),
        )
    }

    /// Create a resources/read response
    pub fn resource_read_success(id: RequestId, contents: Vec<ResourceContent>) -> Self {
        Self::success(
            id,
            serde_json::json!({
                "contents": contents
            }),
        )
    }

    /// Create a prompts/list response
    pub fn prompts_list_success(id: RequestId, prompts: Vec<Prompt>) -> Self {
        Self::success(
            id,
            serde_json::json!({
                "prompts": prompts
            }),
        )
    }

    /// Create a prompts/get response
    pub fn prompt_get_success(
        id: RequestId,
        description: Option<String>,
        messages: Vec<PromptMessage>,
    ) -> Self {
        let mut result = serde_json::json!({
            "messages": messages
        });
        if let Some(desc) = description {
            result["description"] = serde_json::Value::String(desc);
        }
        Self::success(id, result)
    }
}

/// JSON-RPC 2.0 notification message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPNotification {
    pub jsonrpc: String,
    pub method: String,
    pub params: Option<serde_json::Value>,
}

impl MCPNotification {
    pub fn new(method: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method: method.into(),
            params: None,
        }
    }

    pub fn with_params(mut self, params: serde_json::Value) -> Self {
        self.params = Some(params);
        self
    }

    /// Create an initialized notification
    pub fn initialized() -> Self {
        Self::new("notifications/initialized")
    }

    /// Create a progress notification
    pub fn progress(operation: impl Into<String>, progress: f64, message: Option<String>) -> Self {
        let mut params = serde_json::json!({
            "operation": operation.into(),
            "progress": progress
        });
        if let Some(msg) = message {
            params["message"] = serde_json::Value::String(msg);
        }
        Self::new("notifications/progress").with_params(params)
    }

    /// Create a tools/list_changed notification
    pub fn tools_list_changed() -> Self {
        Self::new("notifications/tools/list_changed")
    }

    /// Create a resources/list_changed notification
    pub fn resources_list_changed() -> Self {
        Self::new("notifications/resources/list_changed")
    }

    /// Create a resources/updated notification
    pub fn resource_updated(uri: impl Into<String>) -> Self {
        Self::new("notifications/resources/updated").with_params(serde_json::json!({
            "uri": uri.into()
        }))
    }

    /// Create a prompts/list_changed notification
    pub fn prompts_list_changed() -> Self {
        Self::new("notifications/prompts/list_changed")
    }

    /// Create a logging message notification
    pub fn log_message(
        level: impl Into<String>,
        data: serde_json::Value,
        logger: Option<String>,
    ) -> Self {
        let mut params = serde_json::json!({
            "level": level.into(),
            "data": data
        });
        if let Some(log) = logger {
            params["logger"] = serde_json::Value::String(log);
        }
        Self::new("notifications/message").with_params(params)
    }
}

/// Request ID type that can be string, number, or null
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RequestId {
    String(String),
    Number(i64),
    Null,
}

impl RequestId {
    pub fn generate() -> Self {
        Self::String(Uuid::new_v4().to_string())
    }

    pub fn from_number(n: i64) -> Self {
        Self::Number(n)
    }

    pub fn from_string(s: impl Into<String>) -> Self {
        Self::String(s.into())
    }
}

impl From<String> for RequestId {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

impl From<&str> for RequestId {
    fn from(s: &str) -> Self {
        Self::String(s.to_string())
    }
}

impl From<i64> for RequestId {
    fn from(n: i64) -> Self {
        Self::Number(n)
    }
}

impl From<i32> for RequestId {
    fn from(n: i32) -> Self {
        Self::Number(n as i64)
    }
}

/// Unified message type for MCP communication
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MCPMessage {
    Request(MCPRequest),
    Response(MCPResponse),
    Notification(MCPNotification),
}

impl MCPMessage {
    /// Parse a JSON string into an MCP message
    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json).map_err(|e| Error::parse_error(e.to_string()))
    }

    /// Serialize the message to JSON
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(|e| Error::serialization_error(e.to_string()))
    }

    /// Get the method name if this is a request or notification
    pub fn method(&self) -> Option<&str> {
        match self {
            MCPMessage::Request(req) => Some(&req.method),
            MCPMessage::Notification(notif) => Some(&notif.method),
            MCPMessage::Response(_) => None,
        }
    }

    /// Get the request ID if this is a request or response
    pub fn id(&self) -> Option<&RequestId> {
        match self {
            MCPMessage::Request(req) => Some(&req.id),
            MCPMessage::Response(resp) => Some(&resp.id),
            MCPMessage::Notification(_) => None,
        }
    }

    /// Check if this is a request message
    pub fn is_request(&self) -> bool {
        matches!(self, MCPMessage::Request(_))
    }

    /// Check if this is a response message
    pub fn is_response(&self) -> bool {
        matches!(self, MCPMessage::Response(_))
    }

    /// Check if this is a notification message
    pub fn is_notification(&self) -> bool {
        matches!(self, MCPMessage::Notification(_))
    }
}

/// Initialize parameters for the initialize request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeParams {
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    pub capabilities: HashMap<String, serde_json::Value>,
    #[serde(rename = "clientInfo")]
    pub client_info: ClientInfo,
}

/// Initialize result for the initialize response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeResult {
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    pub capabilities: ServerCapabilities,
    #[serde(rename = "serverInfo")]
    pub server_info: ServerInfo,
}

/// Tools/call parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallToolParams {
    pub name: String,
    pub arguments: Option<serde_json::Value>,
}

/// Resources/read parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadResourceParams {
    pub uri: String,
}

/// Resources/subscribe parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscribeResourceParams {
    pub uri: String,
}

/// Prompts/get parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetPromptParams {
    pub name: String,
    pub arguments: Option<HashMap<String, String>>,
}

/// Logging message parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingMessageParams {
    pub level: String,
    pub data: serde_json::Value,
    pub logger: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_creation() {
        let req = MCPRequest::new("test-id", "test/method")
            .with_params(serde_json::json!({"key": "value"}));

        assert_eq!(req.jsonrpc, "2.0");
        assert_eq!(req.method, "test/method");
        assert!(req.params.is_some());
    }

    #[test]
    fn test_response_creation() {
        let resp = MCPResponse::success(
            RequestId::from_string("test-id"),
            serde_json::json!({"result": "success"}),
        );

        assert_eq!(resp.jsonrpc, "2.0");
        assert!(resp.result.is_some());
        assert!(resp.error.is_none());
    }

    #[test]
    fn test_error_response() {
        let error = Error::tool_not_found("nonexistent_tool");
        let resp = MCPResponse::error(RequestId::from_string("test-id"), error);

        assert!(resp.result.is_none());
        assert!(resp.error.is_some());
    }

    #[test]
    fn test_message_serialization() {
        let req = MCPRequest::initialize(ClientInfo::new("test-client", "1.0.0"), HashMap::new());
        let msg = MCPMessage::Request(req);

        let json = msg.to_json().unwrap();
        let parsed = MCPMessage::from_json(&json).unwrap();

        assert!(parsed.is_request());
        assert_eq!(parsed.method(), Some("initialize"));
    }

    #[test]
    fn test_request_id_types() {
        let string_id = RequestId::from_string("test");
        let number_id = RequestId::from_number(42);

        assert!(matches!(string_id, RequestId::String(_)));
        assert!(matches!(number_id, RequestId::Number(_)));
    }
}
