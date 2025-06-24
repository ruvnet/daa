//! Event streaming and notifications for QuDAG MCP.

use crate::error::{Error, Result};
use crate::types::{EventType, McpEvent};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::{broadcast, RwLock};
use tokio::time::{interval, Duration};

/// Event manager for handling real-time events
#[derive(Debug, Clone)]
pub struct EventManager {
    /// Event broadcaster
    sender: broadcast::Sender<Event>,
    /// Event configuration
    config: EventConfig,
    /// Active subscriptions
    subscriptions: Arc<RwLock<HashMap<String, EventSubscription>>>,
}

/// Event configuration
#[derive(Debug, Clone)]
pub struct EventConfig {
    /// Maximum number of events to buffer
    pub buffer_size: usize,
    /// Event retention duration in seconds
    pub retention_duration: u64,
    /// Whether to enable event batching
    pub enable_batching: bool,
    /// Batch size for batched events
    pub batch_size: usize,
    /// Batch timeout in milliseconds
    pub batch_timeout_ms: u64,
}

/// Event subscription
#[derive(Debug, Clone)]
pub struct EventSubscription {
    /// Subscription ID
    pub id: String,
    /// Event types to subscribe to
    pub event_types: Vec<EventType>,
    /// Event filters
    pub filters: HashMap<String, String>,
    /// Subscription created time
    pub created_at: SystemTime,
    /// Last activity
    pub last_activity: SystemTime,
}

/// Internal event type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// Event ID
    pub id: String,
    /// Event type
    pub event_type: EventType,
    /// Event source
    pub source: String,
    /// Event data
    pub data: serde_json::Value,
    /// Event timestamp
    pub timestamp: u64,
    /// Event metadata
    pub metadata: HashMap<String, String>,
}

/// Event notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventNotification {
    /// Notification method
    pub method: String,
    /// Event data
    pub params: EventParams,
}

/// Event parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventParams {
    /// Events (single event or batch)
    pub events: Vec<Event>,
    /// Subscription ID
    pub subscription_id: String,
}

/// Event filter criteria
#[derive(Debug, Clone)]
pub struct EventFilter {
    /// Event types to include
    pub event_types: Option<Vec<EventType>>,
    /// Source filter
    pub source: Option<String>,
    /// Time range filter
    pub time_range: Option<(u64, u64)>,
    /// Custom metadata filters
    pub metadata_filters: HashMap<String, String>,
}

impl EventManager {
    /// Create new event manager
    pub fn new(config: EventConfig) -> Self {
        let (sender, _) = broadcast::channel(config.buffer_size);

        Self {
            sender,
            config,
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create event manager with default config
    pub fn with_default_config() -> Self {
        Self::new(EventConfig::default())
    }

    /// Subscribe to events
    pub async fn subscribe(
        &self,
        subscription_id: String,
        event_types: Vec<EventType>,
        filters: HashMap<String, String>,
    ) -> Result<broadcast::Receiver<Event>> {
        let subscription = EventSubscription {
            id: subscription_id.clone(),
            event_types,
            filters,
            created_at: SystemTime::now(),
            last_activity: SystemTime::now(),
        };

        let mut subscriptions = self.subscriptions.write().await;
        subscriptions.insert(subscription_id, subscription);

        Ok(self.sender.subscribe())
    }

    /// Unsubscribe from events
    pub async fn unsubscribe(&self, subscription_id: &str) -> Result<()> {
        let mut subscriptions = self.subscriptions.write().await;
        subscriptions.remove(subscription_id);
        Ok(())
    }

    /// Publish an event
    pub async fn publish_event(
        &self,
        event_type: EventType,
        source: String,
        data: serde_json::Value,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<()> {
        let event = Event {
            id: uuid::Uuid::new_v4().to_string(),
            event_type,
            source,
            data,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            metadata: metadata.unwrap_or_default(),
        };

        // Send to all subscribers
        let _ = self.sender.send(event);
        Ok(())
    }

    /// Publish system event
    pub async fn publish_system_event(
        &self,
        message: String,
        data: serde_json::Value,
    ) -> Result<()> {
        self.publish_event(
            EventType::System,
            "qudag-mcp-server".to_string(),
            serde_json::json!({
                "message": message,
                "data": data
            }),
            None,
        )
        .await
    }

    /// Publish security event
    pub async fn publish_security_event(
        &self,
        event_type: String,
        severity: String,
        description: String,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<()> {
        self.publish_event(
            EventType::Security,
            "security".to_string(),
            serde_json::json!({
                "event_type": event_type,
                "severity": severity,
                "description": description
            }),
            metadata,
        )
        .await
    }

    /// Publish network event
    pub async fn publish_network_event(
        &self,
        event_type: String,
        peer_info: serde_json::Value,
        data: serde_json::Value,
    ) -> Result<()> {
        self.publish_event(
            EventType::Network,
            "network".to_string(),
            serde_json::json!({
                "event_type": event_type,
                "peer_info": peer_info,
                "data": data
            }),
            None,
        )
        .await
    }

    /// Publish DAG event
    pub async fn publish_dag_event(
        &self,
        event_type: String,
        vertex_id: Option<String>,
        data: serde_json::Value,
    ) -> Result<()> {
        let mut event_data = serde_json::json!({
            "event_type": event_type,
            "data": data
        });

        if let Some(vertex_id) = vertex_id {
            event_data["vertex_id"] = serde_json::Value::String(vertex_id);
        }

        self.publish_event(EventType::Dag, "dag".to_string(), event_data, None)
            .await
    }

    /// Get active subscriptions
    pub async fn get_subscriptions(&self) -> HashMap<String, EventSubscription> {
        self.subscriptions.read().await.clone()
    }

    /// Cleanup expired subscriptions
    pub async fn cleanup_expired_subscriptions(&self) {
        let now = SystemTime::now();
        let retention_duration = Duration::from_secs(self.config.retention_duration);

        let mut subscriptions = self.subscriptions.write().await;
        subscriptions.retain(|_, subscription| {
            now.duration_since(subscription.last_activity)
                .map(|age| age < retention_duration)
                .unwrap_or(false)
        });
    }

    /// Start background cleanup task
    pub fn start_cleanup_task(&self) {
        let event_manager = self.clone();
        tokio::spawn(async move {
            let mut cleanup_interval = interval(Duration::from_secs(300)); // Cleanup every 5 minutes

            loop {
                cleanup_interval.tick().await;
                event_manager.cleanup_expired_subscriptions().await;
            }
        });
    }

    /// Create MCP event notification
    pub fn create_notification(
        &self,
        events: Vec<Event>,
        subscription_id: String,
    ) -> EventNotification {
        EventNotification {
            method: "notifications/events".to_string(),
            params: EventParams {
                events,
                subscription_id,
            },
        }
    }

    /// Filter events based on subscription criteria
    pub fn filter_events(&self, events: &[Event], subscription: &EventSubscription) -> Vec<Event> {
        events
            .iter()
            .filter(|event| {
                // Check event type filter
                if !subscription.event_types.is_empty()
                    && !subscription.event_types.contains(&event.event_type)
                {
                    return false;
                }

                // Check custom filters
                for (key, value) in &subscription.filters {
                    match key.as_str() {
                        "source" => {
                            if event.source != *value {
                                return false;
                            }
                        }
                        "min_timestamp" => {
                            if let Ok(min_ts) = value.parse::<u64>() {
                                if event.timestamp < min_ts {
                                    return false;
                                }
                            }
                        }
                        "max_timestamp" => {
                            if let Ok(max_ts) = value.parse::<u64>() {
                                if event.timestamp > max_ts {
                                    return false;
                                }
                            }
                        }
                        _ => {
                            // Check metadata filters
                            if let Some(metadata_value) = event.metadata.get(key) {
                                if metadata_value != value {
                                    return false;
                                }
                            } else {
                                return false;
                            }
                        }
                    }
                }

                true
            })
            .cloned()
            .collect()
    }
}

impl Default for EventConfig {
    fn default() -> Self {
        Self {
            buffer_size: 1000,
            retention_duration: 3600, // 1 hour
            enable_batching: false,
            batch_size: 10,
            batch_timeout_ms: 1000,
        }
    }
}

impl From<Event> for McpEvent {
    fn from(event: Event) -> Self {
        Self {
            event_type: format!("{:?}", event.event_type),
            data: event.data,
            timestamp: event.timestamp,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_event_subscription() {
        let manager = EventManager::with_default_config();

        let subscription_id = "test_sub".to_string();
        let event_types = vec![EventType::System];
        let filters = HashMap::new();

        let mut receiver = manager
            .subscribe(subscription_id.clone(), event_types, filters)
            .await
            .unwrap();

        // Publish an event
        manager
            .publish_system_event("Test event".to_string(), serde_json::json!({"test": true}))
            .await
            .unwrap();

        // Receive the event
        let received_event = receiver.recv().await.unwrap();
        assert_eq!(received_event.event_type, EventType::System);
        assert_eq!(received_event.source, "qudag-mcp-server");

        // Unsubscribe
        manager.unsubscribe(&subscription_id).await.unwrap();
    }

    #[tokio::test]
    async fn test_event_filtering() {
        let manager = EventManager::with_default_config();

        let subscription = EventSubscription {
            id: "test".to_string(),
            event_types: vec![EventType::Security],
            filters: {
                let mut filters = HashMap::new();
                filters.insert("source".to_string(), "security".to_string());
                filters
            },
            created_at: SystemTime::now(),
            last_activity: SystemTime::now(),
        };

        let events = vec![
            Event {
                id: "1".to_string(),
                event_type: EventType::Security,
                source: "security".to_string(),
                data: serde_json::json!({}),
                timestamp: 1234567890,
                metadata: HashMap::new(),
            },
            Event {
                id: "2".to_string(),
                event_type: EventType::System,
                source: "system".to_string(),
                data: serde_json::json!({}),
                timestamp: 1234567890,
                metadata: HashMap::new(),
            },
            Event {
                id: "3".to_string(),
                event_type: EventType::Security,
                source: "other".to_string(),
                data: serde_json::json!({}),
                timestamp: 1234567890,
                metadata: HashMap::new(),
            },
        ];

        let filtered = manager.filter_events(&events, &subscription);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "1");
    }

    #[tokio::test]
    async fn test_different_event_types() {
        let manager = EventManager::with_default_config();

        // Test security event
        manager
            .publish_security_event(
                "authentication_failed".to_string(),
                "high".to_string(),
                "Authentication attempt failed".to_string(),
                None,
            )
            .await
            .unwrap();

        // Test network event
        manager
            .publish_network_event(
                "peer_connected".to_string(),
                serde_json::json!({"peer_id": "test_peer"}),
                serde_json::json!({"connection_time": 1234567890}),
            )
            .await
            .unwrap();

        // Test DAG event
        manager
            .publish_dag_event(
                "vertex_added".to_string(),
                Some("vertex_123".to_string()),
                serde_json::json!({"payload": "test data"}),
            )
            .await
            .unwrap();
    }
}
