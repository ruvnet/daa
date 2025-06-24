use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use tracing::{info, warn, error, debug};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogContext {
    pub timestamp: DateTime<Utc>,
    pub node_id: Option<String>,
    pub component: Option<String>,
    pub operation: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl LogContext {
    pub fn new() -> Self {
        Self {
            timestamp: Utc::now(),
            node_id: None,
            component: None,
            operation: None,
            metadata: HashMap::new(),
        }
    }
    
    pub fn with_node(mut self, node_id: String) -> Self {
        self.node_id = Some(node_id);
        self
    }
    
    pub fn with_component(mut self, component: String) -> Self {
        self.component = Some(component);
        self
    }
    
    pub fn with_operation(mut self, operation: String) -> Self {
        self.operation = Some(operation);
        self
    }
    
    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

pub struct StructuredLogger;

impl StructuredLogger {
    pub fn new() -> Self {
        Self
    }
    
    pub fn info(&self, message: &str, context: LogContext) {
        info!(
            message = message,
            context = ?context,
            "{}",
            self.format_log(message, &context)
        );
    }
    
    pub fn warn(&self, message: &str, context: LogContext) {
        warn!(
            message = message,
            context = ?context,
            "{}",
            self.format_log(message, &context)
        );
    }
    
    pub fn error(&self, message: &str, context: LogContext) {
        error!(
            message = message,
            context = ?context,
            "{}",
            self.format_log(message, &context)
        );
    }
    
    pub fn debug(&self, message: &str, context: LogContext) {
        debug!(
            message = message,
            context = ?context,
            "{}",
            self.format_log(message, &context)
        );
    }
    
    // Monitoring-specific log methods
    
    pub fn log_optimization(&self, optimization_type: &str, before_value: f64, after_value: f64, context: LogContext) {
        let improvement = ((before_value - after_value) / before_value * 100.0).abs();
        
        let mut ctx = context
            .with_metadata("optimization_type".to_string(), optimization_type.into())
            .with_metadata("before_value".to_string(), before_value.into())
            .with_metadata("after_value".to_string(), after_value.into())
            .with_metadata("improvement_percent".to_string(), improvement.into());
        
        self.info(
            &format!("Optimization applied: {} improved by {:.2}%", optimization_type, improvement),
            ctx
        );
    }
    
    pub fn log_performance_anomaly(&self, metric_name: &str, expected: f64, actual: f64, context: LogContext) {
        let deviation = ((actual - expected) / expected * 100.0).abs();
        
        let ctx = context
            .with_metadata("metric_name".to_string(), metric_name.into())
            .with_metadata("expected_value".to_string(), expected.into())
            .with_metadata("actual_value".to_string(), actual.into())
            .with_metadata("deviation_percent".to_string(), deviation.into());
        
        if deviation > 50.0 {
            self.error(
                &format!("Critical performance anomaly: {} deviates by {:.2}%", metric_name, deviation),
                ctx
            );
        } else if deviation > 20.0 {
            self.warn(
                &format!("Performance anomaly: {} deviates by {:.2}%", metric_name, deviation),
                ctx
            );
        }
    }
    
    pub fn log_cache_performance(&self, cache_type: &str, hit_rate: f64, memory_mb: f64, context: LogContext) {
        let ctx = context
            .with_metadata("cache_type".to_string(), cache_type.into())
            .with_metadata("hit_rate".to_string(), hit_rate.into())
            .with_metadata("memory_mb".to_string(), memory_mb.into());
        
        self.info(
            &format!("Cache performance: {} hit_rate={:.2}% memory={:.2}MB", cache_type, hit_rate * 100.0, memory_mb),
            ctx
        );
    }
    
    pub fn log_swarm_coordination(&self, swarm_id: &str, active_agents: usize, queue_depth: usize, avg_latency_ms: f64, context: LogContext) {
        let ctx = context
            .with_metadata("swarm_id".to_string(), swarm_id.into())
            .with_metadata("active_agents".to_string(), active_agents.into())
            .with_metadata("queue_depth".to_string(), queue_depth.into())
            .with_metadata("avg_latency_ms".to_string(), avg_latency_ms.into());
        
        self.info(
            &format!("Swarm coordination: {} agents={} queue={} latency={:.2}ms", 
                swarm_id, active_agents, queue_depth, avg_latency_ms),
            ctx
        );
    }
    
    fn format_log(&self, message: &str, context: &LogContext) -> String {
        let node = context.node_id.as_deref().unwrap_or("unknown");
        let component = context.component.as_deref().unwrap_or("system");
        let operation = context.operation.as_deref().unwrap_or("general");
        
        format!("[{}][{}][{}] {}", node, component, operation, message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_log_context_builder() {
        let context = LogContext::new()
            .with_node("node1".to_string())
            .with_component("cache".to_string())
            .with_operation("get".to_string())
            .with_metadata("key".to_string(), "test_key".into());
        
        assert_eq!(context.node_id.unwrap(), "node1");
        assert_eq!(context.component.unwrap(), "cache");
        assert_eq!(context.operation.unwrap(), "get");
        assert_eq!(context.metadata.get("key").unwrap(), "test_key");
    }
}