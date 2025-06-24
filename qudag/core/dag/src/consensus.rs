//! DAG consensus implementation with QR-Avalanche algorithm.

use crate::vertex::{Vertex, VertexId};
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};
use thiserror::Error;

/// Errors that can occur during consensus operations.
#[derive(Debug, Error)]
pub enum ConsensusError {
    /// Invalid vertex reference
    #[error("Invalid vertex reference")]
    InvalidVertex,

    /// Conflicting vertices
    #[error("Conflicting vertices")]
    ConflictingVertices,

    /// Failed to reach consensus
    #[error("Failed to reach consensus")]
    ConsensusFailure,

    /// Invalid system state
    #[error("Invalid system state")]
    InvalidState,

    /// Fork detected in the DAG
    #[error("Fork detected: {0}")]
    ForkDetected(String),

    /// Validation error
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// Byzantine behavior detected
    #[error("Byzantine behavior detected: {0}")]
    ByzantineBehavior(String),

    /// Insufficient votes for consensus
    #[error("Insufficient votes for consensus")]
    InsufficientVotes,

    /// Timeout during consensus
    #[error("Consensus timeout")]
    Timeout,
}

/// Consensus status for a vertex.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConsensusStatus {
    /// Vertex is pending consensus
    Pending,

    /// Vertex has achieved consensus
    Accepted,

    /// Vertex has been rejected
    Rejected,

    /// Vertex has achieved finality (for test compatibility)
    Final,
}

/// Confidence level for a vertex in the QR-Avalanche protocol
#[derive(Debug, Clone, PartialEq)]
pub struct Confidence {
    /// Current confidence value (0.0 to 1.0)
    pub value: f64,
    /// Number of positive votes
    pub positive_votes: usize,
    /// Number of negative votes
    pub negative_votes: usize,
    /// Last update timestamp
    pub last_updated: Instant,
}

impl Default for Confidence {
    fn default() -> Self {
        Self::new()
    }
}

impl Confidence {
    /// Creates a new confidence instance with zero initial confidence
    pub fn new() -> Self {
        Self {
            value: 0.0,
            positive_votes: 0,
            negative_votes: 0,
            last_updated: Instant::now(),
        }
    }

    /// Updates the vote counts and recalculates confidence value
    pub fn update_votes(&mut self, positive: usize, negative: usize) {
        self.positive_votes = positive;
        self.negative_votes = negative;
        let total_votes = positive + negative;
        if total_votes > 0 {
            self.value = positive as f64 / total_votes as f64;
        }
        self.last_updated = Instant::now();
    }
}

/// QR-Avalanche configuration parameters
#[derive(Debug, Clone)]
pub struct QRAvalancheConfig {
    /// Beta parameter - threshold for accepting a vertex (typically 0.8)
    pub beta: f64,
    /// Alpha parameter - threshold for querying (typically 0.6)
    pub alpha: f64,
    /// Sample size for queries
    pub query_sample_size: usize,
    /// Maximum number of consensus rounds
    pub max_rounds: usize,
    /// Finality threshold
    pub finality_threshold: f64,
    /// Timeout for consensus rounds
    pub round_timeout: Duration,
}

impl Default for QRAvalancheConfig {
    fn default() -> Self {
        Self {
            beta: 0.8,
            alpha: 0.6,
            query_sample_size: 20,
            max_rounds: 100,
            finality_threshold: 0.9,
            round_timeout: Duration::from_millis(100),
        }
    }
}

impl QRAvalancheConfig {
    /// Create a configuration optimized for sub-second finality
    pub fn fast_finality() -> Self {
        Self {
            beta: 0.75,                               // Lower threshold for faster acceptance
            alpha: 0.55,                              // Lower query threshold
            query_sample_size: 15,                    // Smaller samples for speed
            max_rounds: 50,                           // Fewer rounds to prevent timeout
            finality_threshold: 0.85,                 // Lower finality threshold
            round_timeout: Duration::from_millis(50), // Faster round timeouts
        }
    }

    /// Create a configuration optimized for high security (slower but more secure)
    pub fn high_security() -> Self {
        Self {
            beta: 0.9,                                 // Higher threshold for security
            alpha: 0.7,                                // Higher query threshold
            query_sample_size: 30,                     // Larger samples for security
            max_rounds: 200,                           // More rounds for consensus
            finality_threshold: 0.95,                  // Higher finality threshold
            round_timeout: Duration::from_millis(200), // Longer timeouts
        }
    }
}

/// Voting record for Byzantine fault tolerance
#[derive(Debug, Clone)]
pub struct VotingRecord {
    /// Votes for each vertex
    pub votes: HashMap<VertexId, HashMap<VertexId, bool>>, // vertex_id -> (voter_id -> vote)
    /// Known Byzantine voters
    pub byzantine_voters: HashSet<VertexId>,
    /// Conflicting vertices detected
    pub conflicts: HashMap<VertexId, HashSet<VertexId>>,
}

impl Default for VotingRecord {
    fn default() -> Self {
        Self::new()
    }
}

impl VotingRecord {
    /// Creates a new empty voting record
    pub fn new() -> Self {
        Self {
            votes: HashMap::new(),
            byzantine_voters: HashSet::new(),
            conflicts: HashMap::new(),
        }
    }

    /// Records a vote for a vertex and detects Byzantine behavior
    pub fn record_vote(
        &mut self,
        vertex_id: VertexId,
        voter_id: VertexId,
        vote: bool,
    ) -> Result<(), ConsensusError> {
        // Check for Byzantine behavior (conflicting votes)
        if let Some(vertex_votes) = self.votes.get(&vertex_id) {
            if let Some(&previous_vote) = vertex_votes.get(&voter_id) {
                if previous_vote != vote {
                    // Byzantine behavior detected
                    self.byzantine_voters.insert(voter_id.clone());
                    return Err(ConsensusError::ByzantineBehavior(format!(
                        "Voter {:?} changed vote for vertex {:?}",
                        voter_id, vertex_id
                    )));
                }
            }
        }

        self.votes
            .entry(vertex_id)
            .or_default()
            .insert(voter_id, vote);
        Ok(())
    }

    /// Gets the positive and negative vote counts for a vertex
    pub fn get_vote_counts(&self, vertex_id: &VertexId) -> (usize, usize) {
        if let Some(vertex_votes) = self.votes.get(vertex_id) {
            let positive = vertex_votes.values().filter(|&&v| v).count();
            let negative = vertex_votes.values().filter(|&&v| !v).count();
            (positive, negative)
        } else {
            (0, 0)
        }
    }
}

/// Consensus metrics for monitoring performance
#[derive(Debug, Clone)]
pub struct ConsensusMetrics {
    /// Total vertices processed
    pub total_vertices_processed: usize,
    /// Average finality time
    pub average_finality_time: Duration,
    /// Total finality times for averaging
    pub total_finality_time: Duration,
    /// Number of finalized vertices
    pub finalized_count: usize,
    /// Number of Byzantine behaviors detected
    pub byzantine_behaviors_detected: usize,
    /// Number of forks resolved
    pub forks_resolved: usize,
    /// Current throughput (vertices/second)
    pub current_throughput: f64,
    /// Start time for throughput calculation
    pub start_time: Instant,
}

impl Default for ConsensusMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl ConsensusMetrics {
    /// Creates a new metrics instance
    pub fn new() -> Self {
        Self {
            total_vertices_processed: 0,
            average_finality_time: Duration::ZERO,
            total_finality_time: Duration::ZERO,
            finalized_count: 0,
            byzantine_behaviors_detected: 0,
            forks_resolved: 0,
            current_throughput: 0.0,
            start_time: Instant::now(),
        }
    }

    /// Records that a vertex has been processed
    pub fn record_vertex_processed(&mut self) {
        self.total_vertices_processed += 1;
        let elapsed = self.start_time.elapsed();
        if elapsed.as_secs() > 0 {
            self.current_throughput = self.total_vertices_processed as f64 / elapsed.as_secs_f64();
        }
    }

    /// Records finality achievement for a vertex
    pub fn record_finality(&mut self, finality_time: Duration) {
        self.finalized_count += 1;
        self.total_finality_time += finality_time;
        self.average_finality_time = self.total_finality_time / self.finalized_count as u32;
    }

    /// Records detection of Byzantine behavior
    pub fn record_byzantine_behavior(&mut self) {
        self.byzantine_behaviors_detected += 1;
    }

    /// Records resolution of a fork
    pub fn record_fork_resolved(&mut self) {
        self.forks_resolved += 1;
    }
}

/// DAG consensus trait defining the interface for consensus operations.
pub trait Consensus {
    /// Initialize consensus system with genesis vertex.
    fn init(&mut self, genesis: Vertex) -> Result<(), ConsensusError>;

    /// Process a new vertex for consensus.
    fn process_vertex(&mut self, vertex: &Vertex) -> Result<ConsensusStatus, ConsensusError>;

    /// Check if consensus has been reached for a vertex.
    fn is_consensus_reached(&self, vertex_id: &VertexId) -> Result<bool, ConsensusError>;

    /// Get the current tip set (vertices with no children).
    fn get_tips(&self) -> Vec<VertexId>;

    /// Prune old vertices that have achieved consensus.
    fn prune(&mut self) -> Result<(), ConsensusError>;
}

/// QR-Avalanche consensus implementation
#[derive(Debug)]
pub struct QRAvalanche {
    /// Vertices and their consensus status
    pub vertices: HashMap<VertexId, ConsensusStatus>,
    /// Tip set (vertices with no children)
    pub tips: HashSet<VertexId>,
    /// Confidence tracking for vertices
    pub confidence: HashMap<VertexId, Confidence>,
    /// Configuration parameters
    pub config: QRAvalancheConfig,
    /// Voting records for Byzantine fault tolerance
    pub voting_record: VotingRecord,
    /// Consensus metrics
    pub metrics: ConsensusMetrics,
    /// Start time for each vertex consensus
    pub vertex_start_times: HashMap<VertexId, Instant>,
    /// Network participants
    pub participants: HashSet<VertexId>,
}

impl QRAvalanche {
    /// Creates a new QR-Avalanche consensus instance
    pub fn new() -> Self {
        Self {
            vertices: HashMap::new(),
            tips: HashSet::new(),
            confidence: HashMap::new(),
            config: QRAvalancheConfig::default(),
            voting_record: VotingRecord::new(),
            metrics: ConsensusMetrics::new(),
            vertex_start_times: HashMap::new(),
            participants: HashSet::new(),
        }
    }

    /// Creates a new QR-Avalanche consensus instance with custom configuration
    pub fn with_config(config: QRAvalancheConfig) -> Self {
        Self {
            vertices: HashMap::new(),
            tips: HashSet::new(),
            confidence: HashMap::new(),
            config,
            voting_record: VotingRecord::new(),
            metrics: ConsensusMetrics::new(),
            vertex_start_times: HashMap::new(),
            participants: HashSet::new(),
        }
    }

    /// Process a vertex ID for consensus using QR-Avalanche algorithm
    pub fn process_vertex(
        &mut self,
        vertex_id: VertexId,
    ) -> Result<ConsensusStatus, ConsensusError> {
        // Record start time
        self.vertex_start_times
            .insert(vertex_id.clone(), Instant::now());

        // Initialize confidence
        self.confidence.insert(vertex_id.clone(), Confidence::new());

        // Start with pending status
        let status = ConsensusStatus::Pending;
        self.vertices.insert(vertex_id.clone(), status.clone());

        // Add to tips initially
        self.tips.insert(vertex_id.clone());

        // Record metrics
        self.metrics.record_vertex_processed();

        Ok(status)
    }

    /// Record a vote for a vertex (implements Byzantine fault tolerance)
    pub fn record_vote(
        &mut self,
        vertex_id: VertexId,
        voter_id: VertexId,
        vote: bool,
    ) -> Result<(), ConsensusError> {
        // Record the vote
        self.voting_record
            .record_vote(vertex_id.clone(), voter_id.clone(), vote)?;

        // Update confidence based on votes
        let (positive, negative) = self.voting_record.get_vote_counts(&vertex_id);

        if let Some(confidence) = self.confidence.get_mut(&vertex_id) {
            confidence.update_votes(positive, negative);

            // Check for finality based on beta threshold
            if confidence.value >= self.config.beta {
                self.finalize_vertex(vertex_id)?;
            } else if confidence.value <= (1.0 - self.config.beta) {
                // Reject if confidence is too low
                self.vertices
                    .insert(vertex_id.clone(), ConsensusStatus::Rejected);
                self.tips.remove(&vertex_id);
            }
        }

        Ok(())
    }

    /// Finalize a vertex (achieve consensus)
    fn finalize_vertex(&mut self, vertex_id: VertexId) -> Result<(), ConsensusError> {
        // Update status to final
        self.vertices
            .insert(vertex_id.clone(), ConsensusStatus::Final);

        // Record finality time
        if let Some(start_time) = self.vertex_start_times.get(&vertex_id) {
            let finality_time = start_time.elapsed();
            self.metrics.record_finality(finality_time);
        }

        // Remove from tips as it's now finalized
        self.tips.remove(&vertex_id);

        Ok(())
    }

    /// Get confidence for a vertex
    pub fn get_confidence(&self, vertex_id: &VertexId) -> Option<&Confidence> {
        self.confidence.get(vertex_id)
    }

    /// Detect and resolve forks in the DAG
    pub fn detect_and_resolve_forks(&mut self) -> Result<Vec<VertexId>, ConsensusError> {
        let mut resolved_forks = Vec::new();

        // Detect potential fork conflicts
        let conflicts = self.detect_fork_conflicts();

        for conflict_set in conflicts {
            if conflict_set.len() > 1 {
                // Resolve conflict by choosing vertex with highest confidence
                let winner = self.resolve_conflict_set(&conflict_set)?;

                // Reject all other vertices in the conflict set
                for vertex_id in &conflict_set {
                    if vertex_id != &winner {
                        self.vertices
                            .insert(vertex_id.clone(), ConsensusStatus::Rejected);
                        self.tips.remove(vertex_id);
                        resolved_forks.push(vertex_id.clone());

                        // Record the conflict for future reference
                        self.voting_record
                            .conflicts
                            .entry(winner.clone())
                            .or_default()
                            .insert(vertex_id.clone());
                    }
                }
            }
        }

        // Record metrics
        for _ in &resolved_forks {
            self.metrics.record_fork_resolved();
        }

        Ok(resolved_forks)
    }

    /// Detect fork conflicts in the DAG
    fn detect_fork_conflicts(&self) -> Vec<Vec<VertexId>> {
        let mut conflicts = Vec::new();
        let mut visited = HashSet::new();

        // Group vertices that might be in conflict
        let pending_vertices: Vec<_> = self
            .vertices
            .iter()
            .filter(|(_, status)| **status == ConsensusStatus::Pending)
            .map(|(id, _)| id.clone())
            .collect();

        for vertex_id in &pending_vertices {
            if visited.contains(vertex_id) {
                continue;
            }

            let mut conflict_set = vec![vertex_id.clone()];
            visited.insert(vertex_id.clone());

            // Find other vertices that conflict with this one
            for other_vertex_id in &pending_vertices {
                if other_vertex_id != vertex_id
                    && !visited.contains(other_vertex_id)
                    && self.vertices_conflict(vertex_id, other_vertex_id)
                {
                    conflict_set.push(other_vertex_id.clone());
                    visited.insert(other_vertex_id.clone());
                }
            }

            if conflict_set.len() > 1 {
                conflicts.push(conflict_set);
            }
        }

        conflicts
    }

    /// Check if two vertices are in conflict
    fn vertices_conflict(&self, vertex1: &VertexId, vertex2: &VertexId) -> bool {
        // Simplified conflict detection based on similarity
        // In a real implementation, this would check for actual conflicts
        // such as double-spending or conflicting state transitions

        let bytes1 = vertex1.as_bytes();
        let bytes2 = vertex2.as_bytes();

        // Consider vertices conflicting if they have similar "content"
        // This is a placeholder - real conflict detection would be more sophisticated
        if bytes1.len() != bytes2.len() {
            return false;
        }

        let mut differences = 0;
        for (b1, b2) in bytes1.iter().zip(bytes2.iter()) {
            if b1 != b2 {
                differences += 1;
            }
        }

        // Consider conflicting if they differ in less than 25% of bytes
        let similarity_threshold = bytes1.len() / 4;
        differences <= similarity_threshold
    }

    /// Resolve a conflict set by choosing the best vertex
    fn resolve_conflict_set(&self, conflict_set: &[VertexId]) -> Result<VertexId, ConsensusError> {
        if conflict_set.is_empty() {
            return Err(ConsensusError::InvalidState);
        }

        if conflict_set.len() == 1 {
            return Ok(conflict_set[0].clone());
        }

        // Find vertex with highest confidence
        let mut best_vertex = &conflict_set[0];
        let mut best_confidence = self
            .confidence
            .get(best_vertex)
            .map(|c| c.value)
            .unwrap_or(0.0);

        for vertex_id in &conflict_set[1..] {
            let confidence = self
                .confidence
                .get(vertex_id)
                .map(|c| c.value)
                .unwrap_or(0.0);
            if confidence > best_confidence {
                best_confidence = confidence;
                best_vertex = vertex_id;
            }
        }

        Ok(best_vertex.clone())
    }

    /// Advanced fork resolution with additional criteria
    pub fn advanced_fork_resolution(
        &mut self,
        vertex_id: &VertexId,
    ) -> Result<ConsensusStatus, ConsensusError> {
        // Check if vertex is involved in any conflicts
        if let Some(conflicts) = self.voting_record.conflicts.get(vertex_id) {
            if !conflicts.is_empty() {
                // This vertex has known conflicts, check if they're resolved
                for conflict_vertex in conflicts {
                    if let Some(status) = self.vertices.get(conflict_vertex) {
                        if *status == ConsensusStatus::Pending {
                            // Conflict still pending, continue monitoring
                            return Ok(ConsensusStatus::Pending);
                        }
                    }
                }
            }
        }

        // Check confidence level
        if let Some(confidence) = self.confidence.get(vertex_id) {
            if confidence.value >= self.config.finality_threshold {
                return Ok(ConsensusStatus::Final);
            } else if confidence.value >= self.config.beta {
                return Ok(ConsensusStatus::Accepted);
            } else if confidence.value <= (1.0 - self.config.beta) {
                return Ok(ConsensusStatus::Rejected);
            }
        }

        Ok(ConsensusStatus::Pending)
    }

    /// Add a participant to the network
    pub fn add_participant(&mut self, participant_id: VertexId) {
        self.participants.insert(participant_id);
    }

    /// Get current consensus metrics
    pub fn get_metrics(&self) -> &ConsensusMetrics {
        &self.metrics
    }

    /// Check if the system can tolerate Byzantine faults (f < n/3)
    pub fn check_byzantine_tolerance(&self) -> bool {
        let total_participants = self.participants.len();
        let byzantine_count = self.voting_record.byzantine_voters.len();

        if total_participants < 3 {
            return false;
        }

        byzantine_count < total_participants / 3
    }

    /// Query a sample of nodes for their vote on a vertex (QR-Avalanche protocol)
    pub async fn query_sample(
        &mut self,
        vertex_id: &VertexId,
    ) -> Result<(usize, usize), ConsensusError> {
        let sample_size = std::cmp::min(self.config.query_sample_size, self.participants.len());

        if sample_size == 0 {
            return Ok((0, 0));
        }

        // Simulate querying random sample of participants
        let mut positive_votes = 0;
        let mut negative_votes = 0;

        // Use deterministic sampling based on vertex ID for consistency
        let vertex_bytes = vertex_id.as_bytes();
        let mut sample_participants: Vec<_> = self.participants.iter().collect();

        // Sort participants by their "distance" from vertex ID for deterministic sampling
        sample_participants.sort_by_key(|p| {
            let p_bytes = p.as_bytes();
            let mut distance = 0u64;
            for (i, &byte) in vertex_bytes.iter().enumerate() {
                if i < p_bytes.len() {
                    distance += (byte as u64).wrapping_sub(p_bytes[i] as u64).pow(2);
                }
            }
            distance
        });

        // Take the closest sample_size participants
        for participant in sample_participants.iter().take(sample_size) {
            // Skip Byzantine voters
            if self.voting_record.byzantine_voters.contains(participant) {
                continue;
            }

            // Simulate vote based on some criteria (placeholder logic)
            // In a real implementation, this would be network calls
            let vote = self.simulate_participant_vote(vertex_id, participant);

            if vote {
                positive_votes += 1;
            } else {
                negative_votes += 1;
            }

            // Record the vote
            if let Err(_e) =
                self.voting_record
                    .record_vote(vertex_id.clone(), (*participant).clone(), vote)
            {
                // If Byzantine behavior detected, skip this voter
                self.metrics.record_byzantine_behavior();
                continue;
            }
        }

        Ok((positive_votes, negative_votes))
    }

    /// Simulate a participant's vote (placeholder for actual network query)
    fn simulate_participant_vote(&self, vertex_id: &VertexId, participant_id: &VertexId) -> bool {
        // Simple simulation: vote based on hash of vertex and participant
        let vertex_bytes = vertex_id.as_bytes();
        let participant_bytes = participant_id.as_bytes();

        let mut hash_value = 0u64;
        for (i, &v_byte) in vertex_bytes.iter().enumerate() {
            if i < participant_bytes.len() {
                hash_value = hash_value
                    .wrapping_add((v_byte as u64).wrapping_mul(participant_bytes[i] as u64));
            }
        }

        // Return true if hash is even (50% probability)
        hash_value % 2 == 0
    }

    /// Run a full consensus round using QR-Avalanche protocol
    pub async fn run_consensus_round(
        &mut self,
        vertex_id: &VertexId,
    ) -> Result<ConsensusStatus, ConsensusError> {
        let mut current_confidence = 0.0;
        let mut consecutive_strong_rounds = 0;
        let start_time = Instant::now();

        for round in 0..self.config.max_rounds {
            // Check if we've exceeded round timeout
            if start_time.elapsed() > self.config.round_timeout * self.config.max_rounds as u32 {
                break;
            }

            // Query sample of participants
            let (positive, negative) = self.query_sample(vertex_id).await?;
            let total_votes = positive + negative;

            if total_votes == 0 {
                return Err(ConsensusError::InsufficientVotes);
            }

            let round_confidence = positive as f64 / total_votes as f64;

            // Update vertex confidence with momentum-based smoothing for faster convergence
            if let Some(confidence) = self.confidence.get_mut(vertex_id) {
                let old_confidence = confidence.value;
                confidence.update_votes(positive, negative);

                // Apply momentum to accelerate convergence
                let momentum = 0.1; // 10% momentum factor
                confidence.value = confidence.value * (1.0 - momentum) + old_confidence * momentum;
                current_confidence = confidence.value;
            }

            // Optimized early termination conditions for sub-second finality
            if round_confidence >= self.config.alpha {
                consecutive_strong_rounds += 1;

                // Fast-track finality with adaptive thresholds
                let adaptive_threshold = if consecutive_strong_rounds >= 2 {
                    self.config.beta * 0.95 // Lower threshold after strong consecutive rounds
                } else {
                    self.config.beta
                };

                if current_confidence >= adaptive_threshold {
                    self.finalize_vertex(vertex_id.clone())?;
                    return Ok(ConsensusStatus::Final);
                }
            } else if round_confidence <= (1.0 - self.config.alpha) {
                // Strong rejection with fast termination
                consecutive_strong_rounds = 0;
                if current_confidence <= (1.0 - self.config.beta) || round > 10 {
                    self.vertices
                        .insert(vertex_id.clone(), ConsensusStatus::Rejected);
                    self.tips.remove(vertex_id);
                    return Ok(ConsensusStatus::Rejected);
                }
            } else {
                // Weak vote, reset consecutive counter but don't penalize as much
                consecutive_strong_rounds = std::cmp::max(0, consecutive_strong_rounds - 1);
            }

            // Adaptive delay based on confidence level
            let delay_ms = if current_confidence > 0.7 {
                1 // Minimal delay when confidence is high
            } else if current_confidence > 0.5 {
                5 // Short delay for moderate confidence
            } else {
                10 // Longer delay for low confidence
            };

            tokio::time::sleep(Duration::from_millis(delay_ms)).await;
        }

        // If we've exhausted all rounds without achieving finality
        if current_confidence >= self.config.beta {
            Ok(ConsensusStatus::Accepted)
        } else {
            Err(ConsensusError::Timeout)
        }
    }

    /// Optimized consensus round with parallel querying for faster finality
    pub async fn run_fast_consensus_round(
        &mut self,
        vertex_id: &VertexId,
    ) -> Result<ConsensusStatus, ConsensusError> {
        let start_time = Instant::now();
        let target_finality_time = Duration::from_millis(500); // 500ms target

        let mut current_confidence = 0.0;
        let mut consecutive_strong_rounds = 0;
        let mut round = 0;

        while start_time.elapsed() < target_finality_time && round < self.config.max_rounds {
            // Use smaller sample sizes initially for speed, increase if needed
            let dynamic_sample_size = if round < 5 {
                std::cmp::min(10, self.config.query_sample_size)
            } else {
                self.config.query_sample_size
            };

            // Override sample size temporarily
            let original_sample_size = self.config.query_sample_size;
            self.config.query_sample_size = dynamic_sample_size;

            let (positive, negative) = self.query_sample(vertex_id).await?;

            // Restore original sample size
            self.config.query_sample_size = original_sample_size;

            let total_votes = positive + negative;
            if total_votes == 0 {
                round += 1;
                tokio::time::sleep(Duration::from_millis(1)).await;
                continue;
            }

            let round_confidence = positive as f64 / total_votes as f64;

            // Update confidence with exponential smoothing for faster convergence
            if let Some(confidence) = self.confidence.get_mut(vertex_id) {
                confidence.update_votes(positive, negative);
                current_confidence = confidence.value;
            }

            // Aggressive early termination for speed
            if round_confidence >= self.config.alpha * 0.95 {
                // Slightly lower threshold for speed
                consecutive_strong_rounds += 1;

                if current_confidence >= self.config.beta * 0.9 && consecutive_strong_rounds >= 2 {
                    self.finalize_vertex(vertex_id.clone())?;
                    return Ok(ConsensusStatus::Final);
                }
            } else if round_confidence <= (1.0 - self.config.alpha * 0.95)
                && current_confidence <= (1.0 - self.config.beta * 0.9)
            {
                self.vertices
                    .insert(vertex_id.clone(), ConsensusStatus::Rejected);
                self.tips.remove(vertex_id);
                return Ok(ConsensusStatus::Rejected);
            }

            round += 1;

            // Minimal delay for maximum speed
            if round < 10 {
                // No delay for first 10 rounds
            } else {
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
        }

        // Final decision based on current confidence
        if current_confidence >= self.config.beta * 0.85 {
            self.finalize_vertex(vertex_id.clone())?;
            Ok(ConsensusStatus::Final)
        } else if current_confidence >= self.config.beta * 0.7 {
            Ok(ConsensusStatus::Accepted)
        } else {
            Err(ConsensusError::Timeout)
        }
    }

    /// Detect Byzantine behavior patterns
    pub fn detect_byzantine_patterns(&mut self) -> Vec<VertexId> {
        let mut detected_byzantine = Vec::new();

        // Look for voters with inconsistent voting patterns
        for (vertex_id, vertex_votes) in &self.voting_record.votes {
            for (voter_id, &_vote) in vertex_votes {
                // Check if this voter has conflicting votes on related vertices
                if self.has_conflicting_vote_pattern(voter_id, vertex_id)
                    && !self.voting_record.byzantine_voters.contains(voter_id)
                {
                    self.voting_record.byzantine_voters.insert(voter_id.clone());
                    detected_byzantine.push(voter_id.clone());
                    self.metrics.record_byzantine_behavior();
                }
            }
        }

        detected_byzantine
    }

    /// Check if a voter has conflicting voting patterns (simplified logic)  
    fn has_conflicting_vote_pattern(&self, voter_id: &VertexId, _vertex_id: &VertexId) -> bool {
        // Simplified Byzantine detection: check for too much inconsistency
        // In a real implementation, this would use more sophisticated analysis

        let voter_votes: Vec<_> = self
            .voting_record
            .votes
            .values()
            .filter_map(|vertex_votes| vertex_votes.get(voter_id))
            .collect();

        if voter_votes.len() < 5 {
            return false; // Not enough data
        }

        let positive_count = voter_votes.iter().filter(|&&v| *v).count();
        let _negative_count = voter_votes.len() - positive_count;

        // If voting pattern is too erratic (close to 50/50), might be Byzantine
        let positive_ratio = positive_count as f64 / voter_votes.len() as f64;

        // Flag as potentially Byzantine if voting is too random
        positive_ratio > 0.4 && positive_ratio < 0.6 && voter_votes.len() > 10
    }

    /// Synchronize with another consensus instance
    pub fn sync(&mut self) -> Result<(), ConsensusError> {
        // Simple sync implementation - nothing to do for now
        Ok(())
    }
}

impl Default for QRAvalanche {
    fn default() -> Self {
        Self::new()
    }
}
