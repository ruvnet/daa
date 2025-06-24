//! Streaming JSON parser for MCP protocol

use crate::error::{AIError, Result};
use futures::{Stream, StreamExt};
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::task::{Context, Poll};
use tracing::{debug, error, trace};

/// Streaming JSON parser for handling MCP messages
#[derive(Debug)]
pub struct StreamingJsonParser {
    buffer: String,
    in_object: bool,
    brace_count: i32,
}

/// A parsed JSON message from the stream
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonMessage {
    pub content: serde_json::Value,
    pub raw: String,
}

/// Stream wrapper for parsing JSON messages
pub struct JsonMessageStream<S> {
    inner: S,
    parser: StreamingJsonParser,
}

impl StreamingJsonParser {
    /// Create a new streaming JSON parser
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            in_object: false,
            brace_count: 0,
        }
    }
    
    /// Process incoming data and extract complete JSON messages
    pub fn process_data(&mut self, data: &str) -> Result<Vec<JsonMessage>> {
        trace!("Processing {} bytes of data", data.len());
        
        let mut messages = Vec::new();
        self.buffer.push_str(data);
        
        // Process characters to find complete JSON objects
        let mut start_pos = 0;
        let chars: Vec<char> = self.buffer.chars().collect();
        
        for (i, &ch) in chars.iter().enumerate() {
            match ch {
                '{' => {
                    if !self.in_object {
                        self.in_object = true;
                        start_pos = i;
                        self.brace_count = 1;
                    } else {
                        self.brace_count += 1;
                    }
                }
                '}' => {
                    if self.in_object {
                        self.brace_count -= 1;
                        if self.brace_count == 0 {
                            // Found complete JSON object
                            let json_str: String = chars[start_pos..=i].iter().collect();
                            
                            match self.parse_json_message(&json_str) {
                                Ok(message) => {
                                    messages.push(message);
                                    debug!("Parsed complete JSON message");
                                }
                                Err(e) => {
                                    error!("Failed to parse JSON message: {}", e);
                                    // Continue processing even if one message fails
                                }
                            }
                            
                            self.in_object = false;
                            
                            // Remove processed content from buffer
                            self.buffer = chars[(i + 1)..].iter().collect();
                            start_pos = 0;
                            
                            // Break and restart processing with updated buffer
                            return Ok(messages);
                        }
                    }
                }
                '"' => {
                    // Handle string escaping (simplified)
                    // In a full implementation, we'd need proper string parsing
                    // to handle escaped quotes within strings
                }
                _ => {
                    // Other characters don't affect JSON structure counting
                }
            }
        }
        
        Ok(messages)
    }
    
    /// Parse a complete JSON string into a JsonMessage
    fn parse_json_message(&self, json_str: &str) -> Result<JsonMessage> {
        let trimmed = json_str.trim();
        
        if trimmed.is_empty() {
            return Err(AIError::JsonParsingError("Empty JSON string".to_string()));
        }
        
        let parsed: serde_json::Value = serde_json::from_str(trimmed)
            .map_err(|e| AIError::JsonParsingError(format!("Failed to parse JSON: {}", e)))?;
        
        Ok(JsonMessage {
            content: parsed,
            raw: trimmed.to_string(),
        })
    }
    
    /// Clear the internal buffer
    pub fn clear_buffer(&mut self) {
        self.buffer.clear();
        self.in_object = false;
        self.brace_count = 0;
    }
    
    /// Get current buffer size
    pub fn buffer_size(&self) -> usize {
        self.buffer.len()
    }
    
    /// Check if parser is currently inside a JSON object
    pub fn is_parsing(&self) -> bool {
        self.in_object
    }
}

impl<S> JsonMessageStream<S>
where
    S: Stream<Item = Result<bytes::Bytes>> + Unpin,
{
    /// Create a new JSON message stream wrapper
    pub fn new(stream: S) -> Self {
        Self {
            inner: stream,
            parser: StreamingJsonParser::new(),
        }
    }
    
    /// Get the next JSON message from the stream
    pub async fn next_message(&mut self) -> Option<Result<JsonMessage>> {
        loop {
            match self.inner.next().await {
                Some(Ok(bytes)) => {
                    let data = String::from_utf8_lossy(&bytes);
                    
                    match self.parser.process_data(&data) {
                        Ok(messages) => {
                            if !messages.is_empty() {
                                // Return the first message, buffer the rest
                                // In a full implementation, we'd queue multiple messages
                                return Some(Ok(messages.into_iter().next().unwrap()));
                            }
                            // Continue reading if no complete messages yet
                        }
                        Err(e) => {
                            return Some(Err(e));
                        }
                    }
                }
                Some(Err(e)) => {
                    return Some(Err(e));
                }
                None => {
                    // Stream ended
                    return None;
                }
            }
        }
    }
}

impl<S> Stream for JsonMessageStream<S>
where
    S: Stream<Item = Result<bytes::Bytes>> + Unpin,
{
    type Item = Result<JsonMessage>;
    
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            match Pin::new(&mut self.inner).poll_next(cx) {
                Poll::Ready(Some(Ok(bytes))) => {
                    let data = String::from_utf8_lossy(&bytes);
                    
                    match self.parser.process_data(&data) {
                        Ok(messages) => {
                            if !messages.is_empty() {
                                // Return the first message
                                return Poll::Ready(Some(Ok(messages.into_iter().next().unwrap())));
                            }
                            // Continue polling if no complete messages yet
                        }
                        Err(e) => {
                            return Poll::Ready(Some(Err(e)));
                        }
                    }
                }
                Poll::Ready(Some(Err(e))) => {
                    return Poll::Ready(Some(Err(e)));
                }
                Poll::Ready(None) => {
                    return Poll::Ready(None);
                }
                Poll::Pending => {
                    return Poll::Pending;
                }
            }
        }
    }
}

impl Default for StreamingJsonParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_single_json_object() {
        let mut parser = StreamingJsonParser::new();
        let data = r#"{"jsonrpc": "2.0", "id": 1, "method": "test"}"#;
        
        let messages = parser.process_data(data).unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].content["jsonrpc"], "2.0");
    }
    
    #[test]
    fn test_multiple_json_objects() {
        let mut parser = StreamingJsonParser::new();
        let data = r#"{"id": 1}{"id": 2}"#;
        
        let messages = parser.process_data(data).unwrap();
        assert_eq!(messages.len(), 1); // Only processes first complete object
        assert_eq!(messages[0].content["id"], 1);
    }
    
    #[test]
    fn test_partial_json_object() {
        let mut parser = StreamingJsonParser::new();
        let data1 = r#"{"jsonrpc": "2.0""#;
        let data2 = r#", "id": 1}"#;
        
        let messages1 = parser.process_data(data1).unwrap();
        assert_eq!(messages1.len(), 0); // Incomplete
        
        let messages2 = parser.process_data(data2).unwrap();
        assert_eq!(messages2.len(), 1); // Now complete
        assert_eq!(messages2[0].content["id"], 1);
    }
    
    #[test]
    fn test_nested_json_objects() {
        let mut parser = StreamingJsonParser::new();
        let data = r#"{"outer": {"inner": {"value": 42}}}"#;
        
        let messages = parser.process_data(data).unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].content["outer"]["inner"]["value"], 42);
    }
}