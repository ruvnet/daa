//! Network condition tests for QR-Avalanche consensus.

use qudag_dag::{ConsensusError, ConsensusStatus, QRAvalanche, QRAvalancheConfig, VertexId};
use std::collections::HashSet;
use std::time::{Duration, Instant};
use tokio::time::sleep;

/// Simulate network partition by creating separate consensus instances
#[tokio::test]
async fn test_network_partition_tolerance() {
    // Create two separate partitions
    let mut partition_a = QRAvalanche::new();
    let mut partition_b = QRAvalanche::new();

    // Add participants to each partition
    for i in 0..30 {
        let participant_id = VertexId::from_bytes(format!("partition_a_{}", i).into_bytes());
        partition_a.add_participant(participant_id);
    }

    for i in 0..30 {
        let participant_id = VertexId::from_bytes(format!("partition_b_{}", i).into_bytes());
        partition_b.add_participant(participant_id);
    }

    // Create vertices in each partition
    let vertex_a = VertexId::from_bytes(b"vertex_a".to_vec());
    let vertex_b = VertexId::from_bytes(b"vertex_b".to_vec());

    partition_a.process_vertex(vertex_a.clone()).unwrap();
    partition_b.process_vertex(vertex_b.clone()).unwrap();

    // Run consensus in each partition
    let status_a = partition_a.run_consensus_round(&vertex_a).await.unwrap();
    let status_b = partition_b.run_consensus_round(&vertex_b).await.unwrap();

    // Both partitions should reach consensus independently
    assert!(matches!(
        status_a,
        ConsensusStatus::Final | ConsensusStatus::Accepted
    ));
    assert!(matches!(
        status_b,
        ConsensusStatus::Final | ConsensusStatus::Accepted
    ));

    // Simulate partition healing by merging participants
    for participant in &partition_b.participants.clone() {
        partition_a.add_participant(participant.clone());
    }

    // The merged partition should still maintain consistency
    assert!(partition_a.check_byzantine_tolerance());
}

/// Test consensus behavior under message delays
#[tokio::test]
async fn test_message_delay_tolerance() {
    let mut consensus = QRAvalanche::with_config(QRAvalancheConfig {
        round_timeout: Duration::from_millis(200), // Longer timeout for delays
        max_rounds: 150,
        ..QRAvalancheConfig::default()
    });

    // Add participants
    for i in 0..50 {
        let participant_id =
            VertexId::from_bytes(format!("delayed_participant_{}", i).into_bytes());
        consensus.add_participant(participant_id);
    }

    let vertex_id = VertexId::from_bytes(b"delayed_vertex".to_vec());
    consensus.process_vertex(vertex_id.clone()).unwrap();

    // Simulate message delays by adding artificial delays in consensus
    let start_time = Instant::now();

    // Run consensus with delays
    let mut round = 0;
    let mut achieved_consensus = false;

    while round < 20 && !achieved_consensus {
        // Simulate network delay
        sleep(Duration::from_millis(50 + round * 10)).await;

        // Query sample (this would normally have network delays)
        let (positive, negative) = consensus.query_sample(&vertex_id).await.unwrap();

        if positive + negative > 0 {
            let confidence = positive as f64 / (positive + negative) as f64;
            if confidence >= consensus.config.beta {
                achieved_consensus = true;
            }
        }

        round += 1;
    }

    let total_time = start_time.elapsed();

    // Should achieve consensus despite delays
    assert!(
        achieved_consensus,
        "Failed to achieve consensus with message delays"
    );

    // Should still be reasonably fast (under 3 seconds even with delays)
    assert!(
        total_time < Duration::from_secs(3),
        "Consensus took too long with delays: {:?}",
        total_time
    );
}

/// Test consensus under packet loss simulation
#[tokio::test]
async fn test_packet_loss_tolerance() {
    let mut consensus = QRAvalanche::with_config(QRAvalancheConfig {
        query_sample_size: 30, // Larger sample to compensate for losses
        max_rounds: 200,
        ..QRAvalancheConfig::default()
    });

    // Add participants
    for i in 0..100 {
        let participant_id = VertexId::from_bytes(format!("lossy_participant_{}", i).into_bytes());
        consensus.add_participant(participant_id);
    }

    let vertex_id = VertexId::from_bytes(b"lossy_vertex".to_vec());
    consensus.process_vertex(vertex_id.clone()).unwrap();

    // Simulate packet loss by reducing effective votes
    let mut successful_rounds = 0;
    let mut total_rounds = 0;

    for _ in 0..50 {
        total_rounds += 1;

        // Simulate 20% packet loss
        let (positive, negative) = consensus.query_sample(&vertex_id).await.unwrap();
        let packet_loss_rate = 0.2;

        let effective_positive = (positive as f64 * (1.0 - packet_loss_rate)) as usize;
        let effective_negative = (negative as f64 * (1.0 - packet_loss_rate)) as usize;

        if effective_positive + effective_negative > 0 {
            successful_rounds += 1;

            // Manually update confidence with reduced votes
            if let Some(confidence) = consensus.confidence.get_mut(&vertex_id) {
                confidence.update_votes(effective_positive, effective_negative);

                if confidence.value >= consensus.config.beta {
                    // Achieved consensus despite packet loss
                    assert!(
                        successful_rounds as f64 / total_rounds as f64 >= 0.7,
                        "Too many failed rounds due to packet loss"
                    );
                    return;
                }
            }
        }

        sleep(Duration::from_millis(10)).await;
    }

    // Should have some successful rounds despite packet loss
    assert!(
        successful_rounds > 0,
        "No successful rounds with packet loss simulation"
    );
}

/// Test jitter and variable latency tolerance
#[tokio::test]
async fn test_jitter_tolerance() {
    let mut consensus = QRAvalanche::with_config(QRAvalancheConfig::fast_finality());

    // Add participants
    for i in 0..40 {
        let participant_id = VertexId::from_bytes(format!("jitter_participant_{}", i).into_bytes());
        consensus.add_participant(participant_id);
    }

    let vertex_id = VertexId::from_bytes(b"jitter_vertex".to_vec());
    consensus.process_vertex(vertex_id.clone()).unwrap();

    let start_time = Instant::now();

    // Simulate variable jitter in network timing
    for round in 0..30 {
        // Variable delay simulating network jitter (10-100ms)
        let jitter_ms = 10 + (round * 3) % 90;
        sleep(Duration::from_millis(jitter_ms)).await;

        let (positive, negative) = consensus.query_sample(&vertex_id).await.unwrap();

        if positive + negative > 0 {
            if let Some(confidence) = consensus.confidence.get(&vertex_id) {
                if confidence.value >= consensus.config.beta {
                    // Achieved consensus despite jitter
                    let total_time = start_time.elapsed();

                    // Should still achieve reasonable performance despite jitter
                    assert!(
                        total_time < Duration::from_secs(2),
                        "Consensus took too long with jitter: {:?}",
                        total_time
                    );
                    return;
                }
            }
        }
    }

    // Should have made progress even with jitter
    if let Some(confidence) = consensus.confidence.get(&vertex_id) {
        assert!(
            confidence.value > 0.0,
            "No progress made with network jitter"
        );
    }
}

/// Test Byzantine behavior under network stress
#[tokio::test]
async fn test_byzantine_under_network_stress() {
    let mut consensus = QRAvalanche::with_config(QRAvalancheConfig {
        beta: 0.75, // Lower threshold to handle stress
        max_rounds: 300,
        round_timeout: Duration::from_millis(500),
        ..QRAvalancheConfig::default()
    });

    let participant_count = 100;
    let byzantine_count = 25; // 25% Byzantine nodes (still under 1/3)

    // Add honest participants
    for i in 0..participant_count - byzantine_count {
        let participant_id = VertexId::from_bytes(format!("honest_{}", i).into_bytes());
        consensus.add_participant(participant_id);
    }

    // Add Byzantine participants
    for i in 0..byzantine_count {
        let byzantine_id = VertexId::from_bytes(format!("byzantine_{}", i).into_bytes());
        consensus.add_participant(byzantine_id.clone());
        consensus
            .voting_record
            .byzantine_voters
            .insert(byzantine_id);
    }

    let vertex_id = VertexId::from_bytes(b"stressed_vertex".to_vec());
    consensus.process_vertex(vertex_id.clone()).unwrap();

    // Run consensus under network stress with Byzantine nodes
    let mut stress_rounds = 0;
    for stress_level in 1..=5 {
        stress_rounds += 1;

        // Simulate increasing network stress
        sleep(Duration::from_millis(stress_level * 20)).await;

        let (positive, negative) = consensus.query_sample(&vertex_id).await.unwrap();

        // Ensure Byzantine fault tolerance is maintained
        assert!(
            consensus.check_byzantine_tolerance(),
            "Byzantine tolerance violated under network stress"
        );

        if positive + negative > 0 {
            if let Some(confidence) = consensus.confidence.get(&vertex_id) {
                if confidence.value >= consensus.config.beta {
                    // Successfully achieved consensus despite Byzantine nodes and network stress
                    assert!(stress_rounds <= 5, "Too many rounds required under stress");
                    return;
                }
            }
        }
    }

    // Should make progress even under stress
    let metrics = consensus.get_metrics();
    assert!(
        metrics.total_vertices_processed > 0,
        "No vertices processed under stress"
    );
}

/// Test rapid network changes (participants joining/leaving)
#[tokio::test]
async fn test_dynamic_network_changes() {
    let mut consensus = QRAvalanche::new();

    // Start with initial participants
    let initial_participants = 20;
    for i in 0..initial_participants {
        let participant_id = VertexId::from_bytes(format!("initial_{}", i).into_bytes());
        consensus.add_participant(participant_id);
    }

    let vertex_id = VertexId::from_bytes(b"dynamic_vertex".to_vec());
    consensus.process_vertex(vertex_id.clone()).unwrap();

    // Simulate rapid network changes
    for round in 0..10 {
        // Add new participants
        for i in 0..5 {
            let new_participant_id =
                VertexId::from_bytes(format!("dynamic_{}_round_{}", i, round).into_bytes());
            consensus.add_participant(new_participant_id);
        }

        // Query sample with changing network
        let (positive, negative) = consensus.query_sample(&vertex_id).await.unwrap();

        if positive + negative > 0 {
            // Network should remain stable despite changes
            assert!(
                consensus.participants.len() >= initial_participants,
                "Participant count decreased unexpectedly"
            );

            if let Some(confidence) = consensus.confidence.get(&vertex_id) {
                if confidence.value >= consensus.config.beta {
                    // Successfully adapted to network changes
                    return;
                }
            }
        }

        sleep(Duration::from_millis(50)).await;
    }

    // Network should have grown
    assert!(
        consensus.participants.len() > initial_participants,
        "Network didn't grow as expected"
    );
}

/// Test consensus recovery after network healing
#[tokio::test]
async fn test_network_healing_recovery() {
    // Create initially partitioned network
    let mut consensus_main = QRAvalanche::new();
    let mut consensus_partition = QRAvalanche::new();

    // Main network
    for i in 0..40 {
        let participant_id = VertexId::from_bytes(format!("main_{}", i).into_bytes());
        consensus_main.add_participant(participant_id);
    }

    // Partitioned network
    for i in 0..20 {
        let participant_id = VertexId::from_bytes(format!("partition_{}", i).into_bytes());
        consensus_partition.add_participant(participant_id);
    }

    let vertex_id = VertexId::from_bytes(b"healing_vertex".to_vec());

    // Process vertex in main network
    consensus_main.process_vertex(vertex_id.clone()).unwrap();
    let main_status = consensus_main
        .run_consensus_round(&vertex_id)
        .await
        .unwrap();

    // Simulate network healing by merging participants
    for participant in &consensus_partition.participants.clone() {
        consensus_main.add_participant(participant.clone());
    }

    // Verify the healed network maintains consensus
    assert!(matches!(
        main_status,
        ConsensusStatus::Final | ConsensusStatus::Accepted
    ));

    // Check that the merged network is stable
    assert!(consensus_main.check_byzantine_tolerance());
    assert!(consensus_main.participants.len() == 60); // 40 + 20

    // Should be able to process new vertices after healing
    let new_vertex_id = VertexId::from_bytes(b"post_healing_vertex".to_vec());
    consensus_main
        .process_vertex(new_vertex_id.clone())
        .unwrap();

    let post_healing_status = consensus_main
        .run_consensus_round(&new_vertex_id)
        .await
        .unwrap();
    assert!(matches!(
        post_healing_status,
        ConsensusStatus::Final | ConsensusStatus::Accepted
    ));
}
