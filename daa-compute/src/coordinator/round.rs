use crate::{
    mesh::elastic::{ElasticDeviceMesh, NodeInfo},
    protocols::aggregation::GradientAggregator,
    training::{Gradient, ModelParameters},
};
use futures::future::join_all;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, Mutex, RwLock};
use tokio::time::timeout;
use tracing::{debug, error, info, warn};

/// Coordinates training rounds across distributed nodes
pub struct RoundCoordinator {
    mesh: Arc<RwLock<ElasticDeviceMesh>>,
    round_state: Arc<Mutex<RoundState>>,
    participant_manager: Arc<ParticipantManager>,
    consensus_threshold: f32, // Percentage of nodes needed for consensus
}

#[derive(Debug)]
struct RoundState {
    current_round: u64,
    phase: RoundPhase,
    start_time: Instant,
    participants: Vec<String>,
    contributions: HashMap<String, RoundContribution>,
}

#[derive(Debug, PartialEq)]
enum RoundPhase {
    Idle,
    LocalTraining,
    GatheringUpdates,
    Aggregating,
    Broadcasting,
    Complete,
}

#[derive(Clone)]
struct RoundContribution {
    gradient: Option<Gradient>,
    model_hash: String,
    submitted_at: Instant,
    verified: bool,
}

struct ParticipantManager {
    active_participants: RwLock<HashMap<String, ParticipantInfo>>,
    contribution_timeout: Duration,
}

#[derive(Clone)]
struct ParticipantInfo {
    node_id: String,
    rounds_participated: u64,
    successful_contributions: u64,
    last_contribution_time: Option<Instant>,
}

impl RoundCoordinator {
    pub async fn new(mesh: Arc<RwLock<ElasticDeviceMesh>>) -> anyhow::Result<Self> {
        Ok(Self {
            mesh,
            round_state: Arc::new(Mutex::new(RoundState {
                current_round: 0,
                phase: RoundPhase::Idle,
                start_time: Instant::now(),
                participants: Vec::new(),
                contributions: HashMap::new(),
            })),
            participant_manager: Arc::new(ParticipantManager {
                active_participants: RwLock::new(HashMap::new()),
                contribution_timeout: Duration::from_secs(300), // 5 minutes
            }),
            consensus_threshold: 0.51, // Simple majority
        })
    }

    /// Coordinate a training round
    pub async fn coordinate_round(
        &self,
        round_number: u64,
        local_params: ModelParameters,
        aggregator: &GradientAggregator,
    ) -> anyhow::Result<(ModelParameters, u64)> {
        info!("Starting coordination for round {}", round_number);
        
        // Initialize round
        self.initialize_round(round_number).await?;
        
        // Gather participants
        let participants = self.gather_participants().await?;
        info!("Round {} has {} participants", round_number, participants.len());
        
        // Broadcast round start
        self.broadcast_round_start(round_number, &participants).await?;
        
        // Collect gradients with timeout
        let gradients = self.collect_gradients(round_number, &participants).await?;
        
        // Aggregate gradients
        let (aggregated_gradient, comm_bytes) = aggregator
            .aggregate(gradients, round_number)
            .await?;
        
        // Apply gradient to get new parameters
        let new_params = self.apply_gradient_to_params(local_params, aggregated_gradient).await?;
        
        // Broadcast new parameters
        self.broadcast_parameters(&new_params, &participants).await?;
        
        // Complete round
        self.complete_round().await?;
        
        Ok((new_params, comm_bytes))
    }

    /// Initialize a new round
    async fn initialize_round(&self, round_number: u64) -> anyhow::Result<()> {
        let mut state = self.round_state.lock().await;
        
        if state.phase != RoundPhase::Idle && state.phase != RoundPhase::Complete {
            return Err(anyhow::anyhow!("Previous round not complete"));
        }
        
        state.current_round = round_number;
        state.phase = RoundPhase::LocalTraining;
        state.start_time = Instant::now();
        state.participants.clear();
        state.contributions.clear();
        
        Ok(())
    }

    /// Gather eligible participants for the round
    async fn gather_participants(&self) -> anyhow::Result<Vec<NodeInfo>> {
        let mesh = self.mesh.read().await;
        let active_nodes = mesh.get_active_nodes().await;
        
        // Filter nodes based on criteria
        let eligible_nodes: Vec<NodeInfo> = active_nodes.into_iter()
            .filter(|node| {
                // Could add additional filters here
                // e.g., minimum compute capability, reliability score
                node.reliability_score > 0.5
            })
            .collect();
        
        if eligible_nodes.len() < 2 {
            return Err(anyhow::anyhow!("Insufficient participants for round"));
        }
        
        // Update round state with participants
        {
            let mut state = self.round_state.lock().await;
            state.participants = eligible_nodes.iter().map(|n| n.id.clone()).collect();
        }
        
        Ok(eligible_nodes)
    }

    /// Broadcast round start to participants
    async fn broadcast_round_start(
        &self,
        round_number: u64,
        participants: &[NodeInfo],
    ) -> anyhow::Result<()> {
        let mut state = self.round_state.lock().await;
        state.phase = RoundPhase::GatheringUpdates;
        
        // In real implementation, would send network messages
        // For now, log the action
        for node in participants {
            debug!("Broadcasting round {} start to node {}", round_number, node.id);
        }
        
        Ok(())
    }

    /// Collect gradients from participants
    async fn collect_gradients(
        &self,
        round_number: u64,
        participants: &[NodeInfo],
    ) -> anyhow::Result<Vec<Gradient>> {
        let timeout_duration = self.participant_manager.contribution_timeout;
        let (tx, mut rx) = mpsc::channel::<Gradient>(participants.len());
        
        // Spawn tasks to collect from each participant
        let collection_tasks: Vec<_> = participants.iter()
            .map(|node| {
                let node_id = node.id.clone();
                let tx = tx.clone();
                async move {
                    // In real implementation, would receive via network
                    // For now, create mock gradient
                    let gradient = Gradient {
                        values: vec![0.1; 1000], // Mock gradient
                        node_id: node_id.clone(),
                        round: round_number,
                        compressed: false,
                    };
                    
                    // Simulate network delay
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    
                    let _ = tx.send(gradient).await;
                }
            })
            .collect();
        
        // Drop original sender so channel closes when all tasks complete
        drop(tx);
        
        // Wait for contributions with timeout
        let collection_handle = tokio::spawn(async move {
            join_all(collection_tasks).await;
        });
        
        let _ = timeout(timeout_duration, collection_handle).await;
        
        // Collect received gradients
        let mut gradients = Vec::new();
        while let Ok(gradient) = rx.try_recv() {
            gradients.push(gradient);
        }
        
        // Update round state
        {
            let mut state = self.round_state.lock().await;
            state.phase = RoundPhase::Aggregating;
            
            for grad in &gradients {
                state.contributions.insert(
                    grad.node_id.clone(),
                    RoundContribution {
                        gradient: Some(grad.clone()),
                        model_hash: "mock-hash".to_string(),
                        submitted_at: Instant::now(),
                        verified: true,
                    },
                );
            }
        }
        
        // Check if we have enough contributions
        let contribution_rate = gradients.len() as f32 / participants.len() as f32;
        if contribution_rate < self.consensus_threshold {
            warn!(
                "Only {}/{} nodes contributed ({}%)",
                gradients.len(),
                participants.len(),
                contribution_rate * 100.0
            );
        }
        
        // Update participant stats
        self.update_participant_stats(&gradients).await?;
        
        Ok(gradients)
    }

    /// Apply gradient to parameters
    async fn apply_gradient_to_params(
        &self,
        mut params: ModelParameters,
        gradient: Gradient,
    ) -> anyhow::Result<ModelParameters> {
        // Simple gradient application (in practice would use learning rate, optimizer state)
        let learning_rate = 0.01;
        
        for (i, grad_value) in gradient.values.iter().enumerate() {
            if i < params.weights.len() {
                params.weights[i] -= learning_rate * grad_value;
            }
        }
        
        // Update version and hash
        params.version += 1;
        params.hash = self.calculate_param_hash(&params.weights);
        
        Ok(params)
    }

    /// Broadcast new parameters to participants
    async fn broadcast_parameters(
        &self,
        params: &ModelParameters,
        participants: &[NodeInfo],
    ) -> anyhow::Result<()> {
        let mut state = self.round_state.lock().await;
        state.phase = RoundPhase::Broadcasting;
        
        // In real implementation, would send via network
        // Could use different strategies:
        // 1. Direct broadcast from coordinator
        // 2. P2P gossip propagation
        // 3. BitTorrent-style distribution
        
        for node in participants {
            debug!(
                "Broadcasting parameters v{} to node {}",
                params.version, node.id
            );
        }
        
        Ok(())
    }

    /// Complete the round
    async fn complete_round(&self) -> anyhow::Result<()> {
        let mut state = self.round_state.lock().await;
        state.phase = RoundPhase::Complete;
        
        let round_duration = state.start_time.elapsed();
        info!(
            "Round {} completed in {:?} with {}/{} contributions",
            state.current_round,
            round_duration,
            state.contributions.len(),
            state.participants.len()
        );
        
        Ok(())
    }

    /// Update participant statistics
    async fn update_participant_stats(&self, gradients: &[Gradient]) -> anyhow::Result<()> {
        let mut participants = self.participant_manager.active_participants.write().await;
        
        for grad in gradients {
            let participant = participants.entry(grad.node_id.clone())
                .or_insert_with(|| ParticipantInfo {
                    node_id: grad.node_id.clone(),
                    rounds_participated: 0,
                    successful_contributions: 0,
                    last_contribution_time: None,
                });
            
            participant.rounds_participated += 1;
            participant.successful_contributions += 1;
            participant.last_contribution_time = Some(Instant::now());
        }
        
        Ok(())
    }

    /// Calculate hash of parameters for verification
    fn calculate_param_hash(&self, weights: &[f32]) -> String {
        use sha2::{Sha256, Digest};
        
        let mut hasher = Sha256::new();
        for weight in weights {
            hasher.update(weight.to_le_bytes());
        }
        
        format!("{:x}", hasher.finalize())
    }

    /// Handle round failure and recovery
    pub async fn handle_round_failure(&self, error: anyhow::Error) -> anyhow::Result<()> {
        error!("Round failed: {}", error);
        
        let mut state = self.round_state.lock().await;
        
        // Reset to idle state
        state.phase = RoundPhase::Idle;
        
        // Could implement recovery strategies:
        // 1. Retry with same participants
        // 2. Re-select participants
        // 3. Rollback to previous checkpoint
        // 4. Switch to different aggregation strategy
        
        Ok(())
    }

    /// Get round statistics
    pub async fn get_round_stats(&self) -> RoundStatistics {
        let state = self.round_state.lock().await;
        let participants = self.participant_manager.active_participants.read().await;
        
        RoundStatistics {
            current_round: state.current_round,
            phase: format!("{:?}", state.phase),
            participants_count: state.participants.len(),
            contributions_count: state.contributions.len(),
            total_participants: participants.len(),
            round_duration: state.start_time.elapsed(),
        }
    }
}

#[derive(Debug)]
pub struct RoundStatistics {
    pub current_round: u64,
    pub phase: String,
    pub participants_count: usize,
    pub contributions_count: usize,
    pub total_participants: usize,
    pub round_duration: Duration,
}