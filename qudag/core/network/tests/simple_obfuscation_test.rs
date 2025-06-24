//! Simple test to check if traffic obfuscation module works

#[cfg(test)]
mod tests {
    use std::time::Duration;

    #[test]
    fn test_standard_message_sizes() {
        use qudag_network::traffic_obfuscation::{DEFAULT_MESSAGE_SIZE, STANDARD_MESSAGE_SIZES};

        assert_eq!(DEFAULT_MESSAGE_SIZE, 4096);
        assert_eq!(STANDARD_MESSAGE_SIZES.len(), 8);
        assert!(STANDARD_MESSAGE_SIZES.contains(&4096));
    }

    #[test]
    fn test_obfuscation_config_default() {
        use qudag_network::traffic_obfuscation::TrafficObfuscationConfig;

        let config = TrafficObfuscationConfig::default();
        assert!(config.enable_size_normalization);
        assert_eq!(config.standard_message_size, 4096);
        assert!(config.enable_dummy_traffic);
        assert!(config.dummy_traffic_ratio > 0.0);
        assert!(config.enable_traffic_shaping);
    }

    #[test]
    fn test_obfuscation_patterns() {
        use qudag_network::traffic_obfuscation::ObfuscationPattern;

        let patterns = vec![
            ObfuscationPattern::Http,
            ObfuscationPattern::Https,
            ObfuscationPattern::WebSocket,
            ObfuscationPattern::Dns,
            ObfuscationPattern::Custom(vec![1, 2, 3, 4]),
        ];

        assert_eq!(patterns.len(), 5);
    }

    #[tokio::test]
    async fn test_traffic_obfuscator_creation() {
        use qudag_network::traffic_obfuscation::{TrafficObfuscationConfig, TrafficObfuscator};

        let config = TrafficObfuscationConfig::default();
        let _obfuscator = TrafficObfuscator::new(config);

        // If we get here, creation succeeded
        assert!(true);
    }

    #[tokio::test]
    async fn test_message_queue_with_obfuscation() {
        use qudag_network::{message::MessageQueue, traffic_obfuscation::TrafficObfuscationConfig};

        let config = TrafficObfuscationConfig::default();
        let (_queue, _rx) = MessageQueue::with_obfuscation(config);

        // If we get here, creation succeeded
        assert!(true);
    }
}
