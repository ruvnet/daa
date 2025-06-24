use qudag_network::{
    Router,
    PeerId,
    NetworkMessage,
    MessagePriority,
    RoutingStrategy,
    NetworkError,
    MessageEnvelope,
};
use std::time::Duration;
use ring::signature::Ed25519KeyPair;
use tracing::info;
use uuid;
// use test_utils::network::*;

use test_log::test;

#[cfg(test)]
mod security_tests {
    use super::*;

    #[tokio::test]
    async fn test_route_anonymity() {
        let router = Router::new();
        
        // Add test peers
        let peers: Vec<_> = (0..5).map(|_| PeerId::random()).collect();
        for peer in &peers {
            router.add_peer(*peer).await;
        }
        
        // Test message with anonymous routing
        let msg = NetworkMessage {
            id: "test".into(),
            source: peers[0].to_bytes().to_vec(),
            destination: peers[4].to_bytes().to_vec(),
            payload: vec![1, 2, 3],
            priority: MessagePriority::High,
            ttl: Duration::from_secs(60),
        };
        
        let route = router.route(&msg, RoutingStrategy::Anonymous { hops: 3 })
            .await
            .unwrap();
            
        // Verify route properties
        assert_eq!(route.len(), 3, "Route should have exactly 3 hops");
        assert!(route.iter().all(|p| peers.contains(p)), "Route should only use known peers");
        assert!(!route.contains(&peers[0]), "Route should not include source");
        assert!(!route.contains(&peers[4]), "Route should not include destination");
        
        // Test constant time properties
        let mut timings = Vec::new();
        for _ in 0..100 {
            let start = std::time::Instant::now();
            let _ = router.route(&msg, RoutingStrategy::Anonymous { hops: 3 }).await;
            timings.push(start.elapsed());
        }
        
        // Statistical timing analysis
        let mut sorted_timings: Vec<_> = timings.iter().map(|d| d.as_nanos()).collect();
        sorted_timings.sort_unstable();
        
        // Calculate quartiles and IQR
        let q1 = sorted_timings[sorted_timings.len() / 4];
        let q3 = sorted_timings[3 * sorted_timings.len() / 4];
        let iqr = q3 - q1;
        
        // Define outlier bounds
        let lower_bound = q1.saturating_sub(iqr * 2); // 2 * IQR below Q1
        let upper_bound = q3.saturating_add(iqr * 2); // 2 * IQR above Q3
        
        // Check for outliers
        let outliers: Vec<_> = sorted_timings.iter()
            .filter(|&&t| t < lower_bound || t > upper_bound)
            .collect();
        
        // Calculate basic statistics
        let avg = sorted_timings.iter().sum::<u128>() as f64 / sorted_timings.len() as f64;
        let std_dev = (sorted_timings.iter()
            .map(|&t| {
                let diff = t as f64 - avg;
                diff * diff
            })
            .sum::<f64>() / sorted_timings.len() as f64)
            .sqrt();
        
        // Coefficient of variation should be small for constant-time operations
        let cv = std_dev / avg;
        
        // Assert timing consistency with detailed diagnostics
        assert!(cv < 0.05, 
            "Timing variation too high:\n\
             - Coefficient of variation: {:.3}\n\
             - Mean: {:.2} ns\n\
             - Std Dev: {:.2} ns\n\
             - Outliers: {}\n\
             - IQR: {} ns", 
            cv, avg, std_dev, outliers.len(), iqr);
    }

    #[tokio::test]
    async fn test_message_integrity() {
        // Create and sign a message
        let rng = ring::rand::SystemRandom::new();
        let pkcs8_bytes = Ed25519KeyPair::generate_pkcs8(&rng).unwrap();
        let key_pair = Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref()).unwrap();
        
        let msg = NetworkMessage {
            id: "test".into(),
            source: vec![1],
            destination: vec![2],
            payload: vec![1, 2, 3],
            priority: MessagePriority::High,
            ttl: Duration::from_secs(60),
        };
        
        let mut envelope = MessageEnvelope::new(msg);
        
        // Sign message
        envelope.sign(key_pair.signing_key()).unwrap();
        
        // Verify signature
        assert!(envelope.verify_signature(key_pair.public_key().as_ref()).unwrap());
        
        // Tamper with message
        envelope.message.payload.push(4);
        
        // Verify signature fails
        assert!(!envelope.verify_signature(key_pair.public_key().as_ref()).unwrap());
    }

    #[tokio::test]
    async fn test_route_privacy() {
        let router = Router::new();
        
        // Add test peers
        let peers: Vec<_> = (0..10).map(|_| PeerId::random()).collect();
        for peer in &peers {
            router.add_peer(*peer).await;
        }
        
        let source = peers[0];
        let dest = peers[9];
        
        // Create message with onion routing
        let msg = NetworkMessage {
            id: "test".into(),
            source: source.to_bytes().to_vec(),
            destination: dest.to_bytes().to_vec(),
            payload: vec![1, 2, 3],
            priority: MessagePriority::High,
            ttl: Duration::from_secs(60),
        };
        
        // Get encrypted route
        let encrypted_route = router.route(&msg, RoutingStrategy::Anonymous { hops: 5 })
            .await
            .unwrap();
        
        // Verify route encryption properties
        assert_eq!(encrypted_route.len(), 5, "Route should have 5 hops");
        
        // Each hop should only know its predecessor and successor
        for (i, hop) in encrypted_route.iter().enumerate() {
            let hop_info = router.get_hop_info(hop).await.unwrap();
            
            // Verify hop can only decrypt its layer
            assert!(hop_info.can_decrypt_layer(i), "Hop should decrypt its layer");
            
            // Verify hop cannot decrypt other layers
            for j in 0..encrypted_route.len() {
                if j != i {
                    assert!(!hop_info.can_decrypt_layer(j), "Hop should not decrypt other layers");
                }
            }
            
            // Verify hop only knows immediate neighbors
            if i > 0 {
                assert!(hop_info.knows_peer(&encrypted_route[i-1]), "Hop should know predecessor");
            }
            if i < encrypted_route.len()-1 {
                assert!(hop_info.knows_peer(&encrypted_route[i+1]), "Hop should know successor");
            }
            
            // Verify hop does not know other peers
            for j in 0..encrypted_route.len() {
                if j != i-1 && j != i && j != i+1 {
                    assert!(!hop_info.knows_peer(&encrypted_route[j]), "Hop should not know non-neighbor peers");
                }
            }
        }
    }

    #[tokio::test]
    async fn test_message_unlinkability() {
        let router = Router::new();
        let dest = PeerId::random();
        
        // Add test peers
        for _ in 0..10 {
            router.add_peer(PeerId::random()).await;
        }
        
        // Generate multiple routes to the same destination
        let mut routes = Vec::new();
        for _ in 0..5 {
            let msg = NetworkMessage {
                id: uuid::Uuid::new_v4().to_string(),
                source: PeerId::random().to_bytes().to_vec(),
                destination: dest.to_bytes().to_vec(),
                payload: vec![0; 32],
                priority: MessagePriority::Normal,
                ttl: Duration::from_secs(60),
            };
            
            let route = router.route(&msg, RoutingStrategy::Anonymous { hops: 3 })
                .await
                .unwrap();
            routes.push(route);
        }
        
        // Verify route diversity and statistical properties
        
        // Track peer frequency in routes
        let mut peer_counts = std::collections::HashMap::new();
        for route in &routes {
            for peer in route {
                *peer_counts.entry(*peer).or_insert(0) += 1;
            }
        }
        
        // Verify basic route diversity
        for i in 0..routes.len() {
            for j in (i + 1)..routes.len() {
                assert_ne!(routes[i], routes[j], "Routes should be different");
            }
        }
        
        // Statistical analysis
        let total_hops = routes.len() * 3; // 3 hops per route
        let expected_frequency = total_hops as f64 / peer_counts.len() as f64;
        
        // Chi-square test for uniform distribution
        let chi_square: f64 = peer_counts.values()
            .map(|&count| {
                let diff = count as f64 - expected_frequency;
                (diff * diff) / expected_frequency
            })
            .sum();
        
        // Degrees of freedom = number of peers - 1
        let df = peer_counts.len() - 1;
        
        // Critical value for p=0.05 with df degrees of freedom
        // This is a simplified check - in practice use a proper statistics library
        let critical_value = df as f64 * 1.5; // Simplified threshold
        
        assert!(chi_square < critical_value, 
            "Peer selection not uniform enough: chi_square={}, critical_value={}", 
            chi_square, critical_value);
    }

    #[tokio::test]
    async fn test_routing_performance() {
        let router = Router::new();
        let mut peer_count = 20;
        
        // Add test peers
        let peers: Vec<_> = (0..peer_count).map(|_| PeerId::random()).collect();
        for peer in &peers {
            router.add_peer(*peer).await;
        }
        
        // Benchmark route computation with varying parameters
        let test_cases = vec![
            (3, MessagePriority::Low),
            (3, MessagePriority::Normal),
            (3, MessagePriority::High),
            (5, MessagePriority::Normal),
            (7, MessagePriority::Normal),
        ];
        
        for (hop_count, priority) in test_cases {
            let mut route_times = Vec::with_capacity(100);
            let source = peers[0];
            let dest = peers[peer_count-1];
            
            // Time 100 route computations
            for _ in 0..100 {
                let msg = NetworkMessage {
                    id: uuid::Uuid::new_v4().to_string(),
                    source: source.to_bytes().to_vec(),
                    destination: dest.to_bytes().to_vec(),
                    payload: vec![0; 32],
                    priority,
                    ttl: Duration::from_secs(60),
                };
                
                let start = std::time::Instant::now();
                let route = router.route(&msg, RoutingStrategy::Anonymous { hops: hop_count })
                    .await
                    .unwrap();
                let elapsed = start.elapsed();
                
                route_times.push(elapsed);
                
                // Basic validation
                assert_eq!(route.len(), hop_count, "Route length mismatch");
            }
            
            // Calculate statistics
            let avg = route_times.iter().sum::<Duration>() / route_times.len() as u32;
            let max = route_times.iter().max().unwrap();
            let p95 = route_times.iter()
                .map(|d| d.as_micros())
                .nth(95)
                .unwrap_or_default();
            
            // Log performance metrics
            info!(
                "Route computation performance:\n\
                 - Hops: {}\n\
                 - Priority: {:?}\n\
                 - Average: {:?}\n\
                 - Max: {:?}\n\
                 - P95: {} µs\n\
                 - Routes/sec: {:.2}",
                hop_count,
                priority,
                avg,
                max,
                p95,
                1_000_000f64 / avg.as_micros() as f64
            );
            
            // Performance assertions
            assert!(avg < Duration::from_millis(10), 
                "Route computation too slow: {:?} average", avg);
            assert!(max < Duration::from_millis(50),
                "Route computation max time too high: {:?}", max);
        }
    }
}