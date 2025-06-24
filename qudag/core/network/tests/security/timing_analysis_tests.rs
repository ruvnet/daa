use qudag_network::{Router, PeerId, NetworkMessage, MessagePriority, RoutingStrategy, MessageEnvelope};
use std::time::{Duration, Instant};
use std::collections::HashMap;
use statistical::*;

/// Comprehensive timing attack analysis for network operations
#[cfg(test)]
mod timing_security_tests {
    use super::*;

    const TIMING_SAMPLES: usize = 1000;
    const TIMING_THRESHOLD: f64 = 0.05; // 5% coefficient of variation max

    /// Helper function to perform statistical timing analysis
    fn analyze_timing_distribution(timings: &[Duration]) -> TimingAnalysis {
        let nanos: Vec<f64> = timings.iter().map(|d| d.as_nanos() as f64).collect();
        
        let mean = nanos.iter().sum::<f64>() / nanos.len() as f64;
        let variance = nanos.iter()
            .map(|t| (t - mean).powi(2))
            .sum::<f64>() / nanos.len() as f64;
        let std_dev = variance.sqrt();
        let cv = std_dev / mean;
        
        // Calculate percentiles
        let mut sorted = nanos.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let p50 = sorted[sorted.len() / 2];
        let p95 = sorted[(sorted.len() * 95) / 100];
        let p99 = sorted[(sorted.len() * 99) / 100];
        
        TimingAnalysis {
            mean,
            std_dev,
            coefficient_of_variation: cv,
            p50,
            p95,
            p99,
            sample_count: nanos.len(),
        }
    }

    #[derive(Debug)]
    struct TimingAnalysis {
        mean: f64,
        std_dev: f64,
        coefficient_of_variation: f64,
        p50: f64,
        p95: f64,
        p99: f64,
        sample_count: usize,
    }

    #[tokio::test]
    async fn test_routing_timing_consistency() {
        let router = Router::new();
        
        // Setup test network
        let peers: Vec<_> = (0..20).map(|_| PeerId::random()).collect();
        for peer in &peers {
            router.add_peer(*peer).await;
        }

        let source = peers[0];
        let dest = peers[19];
        
        // Test different routing strategies for timing consistency
        let strategies = vec![
            RoutingStrategy::Anonymous { hops: 3 },
            RoutingStrategy::Anonymous { hops: 5 },
            RoutingStrategy::Anonymous { hops: 7 },
        ];

        for strategy in strategies {
            let mut timings = Vec::with_capacity(TIMING_SAMPLES);
            
            for i in 0..TIMING_SAMPLES {
                let msg = NetworkMessage {
                    id: format!("test_{}", i),
                    source: source.to_bytes().to_vec(),
                    destination: dest.to_bytes().to_vec(),
                    payload: vec![0; 32],
                    priority: MessagePriority::Normal,
                    ttl: Duration::from_secs(60),
                };
                
                let start = Instant::now();
                let _ = router.route(&msg, strategy.clone()).await;
                timings.push(start.elapsed());
            }
            
            let analysis = analyze_timing_distribution(&timings);
            
            println!("Routing timing analysis for {:?}:", strategy);
            println!("  Mean: {:.2} ns", analysis.mean);
            println!("  Std Dev: {:.2} ns", analysis.std_dev);
            println!("  CV: {:.4}", analysis.coefficient_of_variation);
            println!("  P95: {:.2} ns", analysis.p95);
            println!("  P99: {:.2} ns", analysis.p99);
            
            // Assert timing consistency
            assert!(analysis.coefficient_of_variation < TIMING_THRESHOLD,
                "Routing timing variation too high: {:.4} > {:.4}",
                analysis.coefficient_of_variation, TIMING_THRESHOLD);
        }
    }

    #[tokio::test]
    async fn test_message_processing_timing() {
        let router = Router::new();
        
        // Test timing consistency across different message sizes
        let message_sizes = vec![32, 64, 128, 256, 512, 1024];
        
        for size in message_sizes {
            let mut timings = Vec::with_capacity(TIMING_SAMPLES);
            
            for i in 0..TIMING_SAMPLES {
                let msg = NetworkMessage {
                    id: format!("test_{}", i),
                    source: vec![1],
                    destination: vec![2],
                    payload: vec![0; size],
                    priority: MessagePriority::Normal,
                    ttl: Duration::from_secs(60),
                };
                
                let mut envelope = MessageEnvelope::new(msg);
                
                let start = Instant::now();
                let _ = envelope.validate();
                timings.push(start.elapsed());
            }
            
            let analysis = analyze_timing_distribution(&timings);
            
            println!("Message processing timing for size {}:", size);
            println!("  CV: {:.4}", analysis.coefficient_of_variation);
            
            // Message processing should be constant-time regardless of size
            // (within reasonable bounds for different sizes)
            assert!(analysis.coefficient_of_variation < TIMING_THRESHOLD * 2.0,
                "Message processing timing variation too high for size {}: {:.4}",
                size, analysis.coefficient_of_variation);
        }
    }

    #[tokio::test]
    async fn test_peer_lookup_timing() {
        let router = Router::new();
        
        // Add peers with different patterns
        let mut peer_timings = HashMap::new();
        
        // Add peers in a predictable pattern
        for i in 0..100 {
            let peer = PeerId::random();
            router.add_peer(peer).await;
            
            // Time peer lookup
            let mut lookup_times = Vec::with_capacity(100);
            for _ in 0..100 {
                let start = Instant::now();
                let _ = router.has_peer(&peer).await;
                lookup_times.push(start.elapsed());
            }
            
            let analysis = analyze_timing_distribution(&lookup_times);
            peer_timings.insert(peer, analysis);
        }
        
        // Analyze timing patterns across all peers
        let cvs: Vec<f64> = peer_timings.values()
            .map(|a| a.coefficient_of_variation)
            .collect();
        
        let mean_cv = cvs.iter().sum::<f64>() / cvs.len() as f64;
        let max_cv = cvs.iter().fold(0.0f64, |a, &b| a.max(b));
        
        println!("Peer lookup timing analysis:");
        println!("  Mean CV: {:.4}", mean_cv);
        println!("  Max CV: {:.4}", max_cv);
        
        // Peer lookup should be consistent
        assert!(mean_cv < TIMING_THRESHOLD,
            "Peer lookup timing inconsistent: mean CV {:.4}", mean_cv);
        assert!(max_cv < TIMING_THRESHOLD * 2.0,
            "Peer lookup timing has outliers: max CV {:.4}", max_cv);
    }

    #[tokio::test]
    async fn test_message_signature_timing() {
        // Test timing consistency of signature operations
        
        let rng = ring::rand::SystemRandom::new();
        let pkcs8_bytes = ring::signature::Ed25519KeyPair::generate_pkcs8(&rng).unwrap();
        let key_pair = ring::signature::Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref()).unwrap();
        
        // Test different message content for timing leaks
        let test_messages = vec![
            vec![0u8; 32],                                    // All zeros
            vec![0xFFu8; 32],                                // All ones
            (0..32).map(|i| i as u8).collect(),             // Sequential
            vec![0x55u8; 32],                                // Pattern 1
            vec![0xAAu8; 32],                                // Pattern 2
        ];
        
        for (pattern_id, payload) in test_messages.iter().enumerate() {
            let mut sign_timings = Vec::with_capacity(TIMING_SAMPLES);
            let mut verify_timings = Vec::with_capacity(TIMING_SAMPLES);
            
            for i in 0..TIMING_SAMPLES {
                let msg = NetworkMessage {
                    id: format!("test_{}_{}", pattern_id, i),
                    source: vec![1],
                    destination: vec![2],
                    payload: payload.clone(),
                    priority: MessagePriority::Normal,
                    ttl: Duration::from_secs(60),
                };
                
                let mut envelope = MessageEnvelope::new(msg);
                
                // Time signing
                let start = Instant::now();
                envelope.sign(key_pair.signing_key()).unwrap();
                sign_timings.push(start.elapsed());
                
                // Time verification
                let start = Instant::now();
                let _ = envelope.verify_signature(key_pair.public_key().as_ref());
                verify_timings.push(start.elapsed());
            }
            
            let sign_analysis = analyze_timing_distribution(&sign_timings);
            let verify_analysis = analyze_timing_distribution(&verify_timings);
            
            println!("Signature timing for pattern {}:", pattern_id);
            println!("  Sign CV: {:.4}", sign_analysis.coefficient_of_variation);
            println!("  Verify CV: {:.4}", verify_analysis.coefficient_of_variation);
            
            // Signature operations should be constant-time
            assert!(sign_analysis.coefficient_of_variation < TIMING_THRESHOLD,
                "Signing timing varies with message content: pattern {} CV {:.4}",
                pattern_id, sign_analysis.coefficient_of_variation);
            assert!(verify_analysis.coefficient_of_variation < TIMING_THRESHOLD,
                "Verification timing varies with message content: pattern {} CV {:.4}",
                pattern_id, verify_analysis.coefficient_of_variation);
        }
    }

    #[tokio::test]
    async fn test_onion_routing_timing() {
        let router = Router::new();
        
        // Setup network with varying distances
        let peers: Vec<_> = (0..50).map(|_| PeerId::random()).collect();
        for peer in &peers {
            router.add_peer(*peer).await;
        }
        
        // Test timing for different hop counts
        let hop_counts = vec![3, 5, 7, 10];
        
        for hops in hop_counts {
            let mut route_timings = Vec::with_capacity(TIMING_SAMPLES);
            
            for i in 0..TIMING_SAMPLES {
                let msg = NetworkMessage {
                    id: format!("onion_test_{}", i),
                    source: peers[0].to_bytes().to_vec(),
                    destination: peers[peers.len()-1].to_bytes().to_vec(),
                    payload: vec![0; 64],
                    priority: MessagePriority::Normal,
                    ttl: Duration::from_secs(60),
                };
                
                let start = Instant::now();
                let route = router.route(&msg, RoutingStrategy::Anonymous { hops }).await;
                route_timings.push(start.elapsed());
                
                // Verify route was created
                if let Ok(route) = route {
                    assert_eq!(route.len(), hops, "Route length mismatch");
                }
            }
            
            let analysis = analyze_timing_distribution(&route_timings);
            
            println!("Onion routing timing for {} hops:", hops);
            println!("  Mean: {:.2} μs", analysis.mean / 1000.0);
            println!("  CV: {:.4}", analysis.coefficient_of_variation);
            
            // Onion routing should scale predictably with hop count
            // but timing should be consistent for same hop count
            assert!(analysis.coefficient_of_variation < TIMING_THRESHOLD * 1.5,
                "Onion routing timing inconsistent for {} hops: CV {:.4}",
                hops, analysis.coefficient_of_variation);
        }
    }

    #[tokio::test]
    async fn test_network_congestion_timing() {
        let router = Router::new();
        
        // Simulate network congestion by processing many messages simultaneously
        let peers: Vec<_> = (0..10).map(|_| PeerId::random()).collect();
        for peer in &peers {
            router.add_peer(*peer).await;
        }
        
        // Test timing under different load conditions
        let load_levels = vec![1, 10, 50, 100];
        
        for load in load_levels {
            let mut processing_times = Vec::with_capacity(TIMING_SAMPLES);
            
            // Process messages in batches
            for batch in 0..(TIMING_SAMPLES / load) {
                let mut batch_messages = Vec::new();
                
                // Create batch of messages
                for i in 0..load {
                    let msg = NetworkMessage {
                        id: format!("load_test_{}_{}", batch, i),
                        source: peers[0].to_bytes().to_vec(),
                        destination: peers[peers.len()-1].to_bytes().to_vec(),
                        payload: vec![0; 32],
                        priority: MessagePriority::Normal,
                        ttl: Duration::from_secs(60),
                    };
                    batch_messages.push(msg);
                }
                
                // Time batch processing
                let start = Instant::now();
                for msg in batch_messages {
                    let _ = router.route(&msg, RoutingStrategy::Anonymous { hops: 3 }).await;
                }
                let batch_time = start.elapsed();
                
                // Calculate per-message time
                let per_message_time = Duration::from_nanos(
                    batch_time.as_nanos() as u64 / load as u64
                );
                processing_times.push(per_message_time);
            }
            
            let analysis = analyze_timing_distribution(&processing_times);
            
            println!("Network load timing for {} concurrent messages:", load);
            println!("  Mean per message: {:.2} μs", analysis.mean / 1000.0);
            println!("  CV: {:.4}", analysis.coefficient_of_variation);
            
            // Processing time should remain consistent under load
            // (within reasonable bounds for resource contention)
            assert!(analysis.coefficient_of_variation < TIMING_THRESHOLD * 3.0,
                "Network timing under load {} inconsistent: CV {:.4}",
                load, analysis.coefficient_of_variation);
        }
    }
}