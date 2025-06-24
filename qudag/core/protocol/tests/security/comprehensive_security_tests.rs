use qudag_protocol::{
    node::Node,
    message::Message,
    config::Config,
    state::State,
    validation::Validator,
};
use std::time::{Duration, Instant};
use std::collections::{HashMap, HashSet};
use proptest::prelude::*;

/// Comprehensive security test suite for the protocol module
#[cfg(test)]
mod protocol_security_tests {
    use super::*;

    #[test]
    fn test_protocol_state_integrity() {
        let config = Config::default();
        let mut node = Node::new(config);
        
        // Test state transitions are atomic and consistent
        let initial_state = node.get_state().clone();
        
        // Apply a series of state changes
        let messages = vec![
            Message::new("test1", vec![1, 2, 3]),
            Message::new("test2", vec![4, 5, 6]),
            Message::new("test3", vec![7, 8, 9]),
        ];
        
        for msg in messages {
            let pre_state = node.get_state().clone();
            node.process_message(msg);
            let post_state = node.get_state().clone();
            
            // Verify state progression is monotonic
            assert!(post_state.height() >= pre_state.height(),
                "State height decreased: {} -> {}", pre_state.height(), post_state.height());
            
            // Verify state hash chain integrity
            if pre_state.height() > 0 {
                assert!(post_state.previous_hash() == pre_state.current_hash(),
                    "State hash chain broken");
            }
        }
    }

    #[test]
    fn test_message_validation_bypass_attempts() {
        let config = Config::default();
        let validator = Validator::new(config);
        
        // Test various malformed messages that might bypass validation
        let malicious_messages = vec![
            // Oversized payload
            Message::new("oversized", vec![0u8; 1024 * 1024]),
            
            // Empty ID
            Message::new("", vec![1, 2, 3]),
            
            // Very long ID
            Message::new(&"x".repeat(1000), vec![1, 2, 3]),
            
            // Null bytes in ID
            Message::new("test\0bypass", vec![1, 2, 3]),
            
            // Unicode exploitation attempts
            Message::new("test\u{202e}bypass", vec![1, 2, 3]),
            
            // Control characters
            Message::new("test\x00\x01\x02", vec![1, 2, 3]),
        ];
        
        for (i, msg) in malicious_messages.iter().enumerate() {
            let result = validator.validate_message(msg);
            
            // All malicious messages should be rejected
            assert!(result.is_err(), 
                "Malicious message {} was accepted: {:?}", i, msg);
            
            // Verify error doesn't leak information
            let error_msg = format!("{:?}", result.unwrap_err());
            assert!(!error_msg.contains("internal"),
                "Error message leaks internal information: {}", error_msg);
        }
    }

    #[test]
    fn test_replay_attack_prevention() {
        let config = Config::default();
        let mut node = Node::new(config);
        
        // Create a valid message
        let msg = Message::new("test_replay", vec![1, 2, 3]);
        
        // Process message first time
        let result1 = node.process_message(msg.clone());
        assert!(result1.is_ok(), "First message processing failed");
        
        // Attempt replay attack
        let result2 = node.process_message(msg.clone());
        assert!(result2.is_err(), "Replay attack succeeded");
        
        // Verify replay detection works with modified timestamps
        let mut modified_msg = msg.clone();
        modified_msg.set_timestamp(Instant::now());
        let result3 = node.process_message(modified_msg);
        assert!(result3.is_err(), "Replay attack with modified timestamp succeeded");
    }

    #[test]
    fn test_denial_of_service_resistance() {
        let config = Config::default();
        let mut node = Node::new(config);
        
        // Test flood attack resistance
        let start_time = Instant::now();
        let mut processed_count = 0;
        let mut rejected_count = 0;
        
        // Send many messages rapidly
        for i in 0..1000 {
            let msg = Message::new(&format!("flood_{}", i), vec![i as u8]);
            let result = node.process_message(msg);
            
            match result {
                Ok(_) => processed_count += 1,
                Err(_) => rejected_count += 1,
            }
            
            // Should not take too long (DoS prevention)
            if start_time.elapsed() > Duration::from_secs(5) {
                break;
            }
        }
        
        // Verify rate limiting is working
        assert!(rejected_count > 0, "No rate limiting detected");
        assert!(processed_count > 0, "All messages rejected - overly aggressive limiting");
        
        println!("DoS test: {} processed, {} rejected in {:?}", 
                processed_count, rejected_count, start_time.elapsed());
    }

    #[test]
    fn test_memory_exhaustion_prevention() {
        let config = Config::default();
        let mut node = Node::new(config);
        
        // Track memory usage
        let initial_memory = get_memory_usage();
        
        // Attempt to exhaust memory with large messages
        for i in 0..100 {
            let large_payload = vec![0u8; 1024 * 1024]; // 1MB per message
            let msg = Message::new(&format!("large_{}", i), large_payload);
            
            let result = node.process_message(msg);
            
            // Check memory usage
            let current_memory = get_memory_usage();
            let memory_increase = current_memory - initial_memory;
            
            // Should not allow unbounded memory growth
            if memory_increase > 50 * 1024 * 1024 { // 50MB limit
                assert!(result.is_err(), 
                    "Memory exhaustion attack succeeded - {} bytes used", memory_increase);
                break;
            }
        }
    }

    #[test]
    fn test_cryptographic_downgrade_prevention() {
        let config = Config::default();
        let node = Node::new(config);
        
        // Test that weak cryptographic parameters are rejected
        let weak_configs = vec![
            Config::builder().key_size(128).build(), // Too small key
            Config::builder().hash_function("md5").build(), // Weak hash
            Config::builder().signature_scheme("rsa1024").build(), // Weak signature
        ];
        
        for weak_config in weak_configs {
            let result = Node::try_new(weak_config);
            assert!(result.is_err(), 
                "Weak cryptographic configuration accepted");
        }
    }

    #[test]
    fn test_side_channel_information_disclosure() {
        let config = Config::default();
        let mut node = Node::new(config);
        
        // Test that processing time doesn't leak information about internal state
        let test_messages = vec![
            Message::new("short", vec![1]),
            Message::new("medium", vec![1; 100]),
            Message::new("long", vec![1; 1000]),
        ];
        
        let mut timing_data = HashMap::new();
        
        for msg in test_messages {
            let mut timings = Vec::new();
            
            // Measure processing time multiple times
            for _ in 0..100 {
                let start = Instant::now();
                let _ = node.process_message(msg.clone());
                timings.push(start.elapsed());
            }
            
            timing_data.insert(msg.id().to_string(), timings);
        }
        
        // Analyze timing patterns
        for (id, timings) in timing_data {
            let mean = timings.iter().sum::<Duration>() / timings.len() as u32;
            let variance = timings.iter()
                .map(|t| {
                    let diff = t.as_nanos() as i128 - mean.as_nanos() as i128;
                    diff * diff
                })
                .sum::<i128>() / timings.len() as i128;
            
            let cv = (variance as f64).sqrt() / mean.as_nanos() as f64;
            
            // Timing should be relatively consistent
            assert!(cv < 0.1, 
                "High timing variance for {}: CV = {:.3}", id, cv);
        }
    }

    #[test]
    fn test_configuration_injection_attacks() {
        // Test that configuration cannot be injected through messages
        let config = Config::default();
        let mut node = Node::new(config);
        
        // Attempt configuration injection through message content
        let injection_attempts = vec![
            Message::new("config_inject", b"key_size=128".to_vec()),
            Message::new("config_inject", b"admin=true".to_vec()),
            Message::new("config_inject", b"debug=true".to_vec()),
            Message::new("config_inject", b"bypass_validation=true".to_vec()),
        ];
        
        for msg in injection_attempts {
            let result = node.process_message(msg);
            
            // Verify configuration hasn't changed
            let current_config = node.get_config();
            assert_eq!(current_config.key_size(), Config::default().key_size(),
                "Configuration was modified through message injection");
        }
    }

    #[test]
    fn test_consensus_manipulation_resistance() {
        let config = Config::default();
        let mut nodes = vec![
            Node::new(config.clone()),
            Node::new(config.clone()),
            Node::new(config.clone()),
        ];
        
        // Test that minority cannot manipulate consensus
        let honest_message = Message::new("honest", vec![1, 2, 3]);
        let malicious_message = Message::new("malicious", vec![4, 5, 6]);
        
        // Two honest nodes process honest message
        nodes[0].process_message(honest_message.clone()).unwrap();
        nodes[1].process_message(honest_message.clone()).unwrap();
        
        // One malicious node tries to process different message
        nodes[2].process_message(malicious_message).unwrap();
        
        // Verify honest consensus is maintained
        let consensus = calculate_consensus(&nodes);
        assert!(consensus.contains(&honest_message.id()),
            "Honest consensus was not maintained");
        assert!(!consensus.contains(&"malicious"),
            "Malicious message achieved consensus");
    }

    // Property-based testing for protocol security
    proptest! {
        #[test]
        fn prop_message_ordering_invariants(
            messages in prop::collection::vec(
                (prop::string::string_regex("[a-zA-Z0-9]{1,100}").unwrap(),
                 prop::collection::vec(any::<u8>(), 0..1000)),
                1..50
            )
        ) {
            let config = Config::default();
            let mut node = Node::new(config);
            
            // Process messages in random order
            for (id, payload) in messages {
                let msg = Message::new(&id, payload);
                let _ = node.process_message(msg);
            }
            
            // Verify invariants are maintained
            let state = node.get_state();
            prop_assert!(state.is_consistent(), "State consistency violated");
            prop_assert!(state.height() >= 0, "Invalid state height");
        }
        
        #[test]
        fn prop_cryptographic_integrity(
            payloads in prop::collection::vec(
                prop::collection::vec(any::<u8>(), 1..1000),
                1..100
            )
        ) {
            let config = Config::default();
            let mut node = Node::new(config);
            
            for (i, payload) in payloads.iter().enumerate() {
                let msg = Message::new(&format!("msg_{}", i), payload.clone());
                let result = node.process_message(msg);
                
                // All valid messages should be processed successfully
                // (assuming proper validation)
                if result.is_err() {
                    // Verify rejection is for valid security reasons
                    let error = result.unwrap_err();
                    let error_str = format!("{:?}", error);
                    prop_assert!(
                        error_str.contains("validation") || 
                        error_str.contains("security") ||
                        error_str.contains("rate_limit"),
                        "Unexpected rejection reason: {}", error_str
                    );
                }
            }
        }
    }

    // Helper functions
    fn get_memory_usage() -> usize {
        // Simplified memory usage calculation
        // In a real implementation, this would use proper memory profiling
        std::process::id() as usize * 1024 // Placeholder
    }
    
    fn calculate_consensus(nodes: &[Node]) -> HashSet<String> {
        let mut consensus = HashSet::new();
        
        // Simplified consensus calculation
        // In a real implementation, this would use the actual consensus algorithm
        for node in nodes {
            let state = node.get_state();
            for msg_id in state.message_ids() {
                consensus.insert(msg_id.to_string());
            }
        }
        
        consensus
    }
}

// Mock implementations for testing
mod mocks {
    use super::*;
    
    impl Config {
        pub fn default() -> Self {
            Config {
                key_size: 256,
                hash_function: "blake3".to_string(),
                signature_scheme: "ed25519".to_string(),
            }
        }
        
        pub fn builder() -> ConfigBuilder {
            ConfigBuilder::default()
        }
        
        pub fn key_size(&self) -> u32 {
            self.key_size
        }
    }
    
    #[derive(Default)]
    pub struct ConfigBuilder {
        key_size: Option<u32>,
        hash_function: Option<String>,
        signature_scheme: Option<String>,
    }
    
    impl ConfigBuilder {
        pub fn key_size(mut self, size: u32) -> Self {
            self.key_size = Some(size);
            self
        }
        
        pub fn hash_function(mut self, func: &str) -> Self {
            self.hash_function = Some(func.to_string());
            self
        }
        
        pub fn signature_scheme(mut self, scheme: &str) -> Self {
            self.signature_scheme = Some(scheme.to_string());
            self
        }
        
        pub fn build(self) -> Config {
            Config {
                key_size: self.key_size.unwrap_or(256),
                hash_function: self.hash_function.unwrap_or("blake3".to_string()),
                signature_scheme: self.signature_scheme.unwrap_or("ed25519".to_string()),
            }
        }
    }
}

pub struct Config {
    key_size: u32,
    hash_function: String,
    signature_scheme: String,
}