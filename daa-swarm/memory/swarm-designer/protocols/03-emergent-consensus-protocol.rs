//! Emergent Consensus Protocol for Dynamic Expert Allocation
//! 
//! This protocol implements self-organizing consensus mechanisms inspired by
//! biological systems, chaos theory, and complex adaptive systems. Experts
//! reach consensus through emergent behaviors without centralized control.

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use serde::{Deserialize, Serialize};
use tokio::sync::{RwLock, Mutex, broadcast};

/// Emergent consensus state for the swarm
#[derive(Debug, Clone)]
pub struct EmergentConsensusState {
    /// Opinion landscape - multi-dimensional opinion space
    pub opinion_landscape: OpinionLandscape,
    /// Attractor basins in the consensus space
    pub attractors: Vec<AttractorBasin>,
    /// Phase space trajectories
    pub trajectories: HashMap<String, PhaseTrajectory>,
    /// Bifurcation points for consensus shifts
    pub bifurcations: Vec<BifurcationPoint>,
    /// Current consensus emergence level
    pub emergence_level: f64,
    /// Lyapunov exponents for chaos measurement
    pub lyapunov_exponents: Vec<f64>,
}

/// Multi-dimensional opinion landscape
#[derive(Debug, Clone)]
pub struct OpinionLandscape {
    /// Dimensions of the opinion space
    pub dimensions: usize,
    /// Expert opinions as points in the space
    pub expert_opinions: HashMap<String, OpinionVector>,
    /// Potential field governing opinion dynamics
    pub potential_field: PotentialField,
    /// Opinion clusters
    pub clusters: Vec<OpinionCluster>,
    /// Landscape topology
    pub topology: LandscapeTopology,
}

/// Opinion vector in multi-dimensional space
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpinionVector {
    /// Opinion values across dimensions
    pub values: Vec<f64>,
    /// Confidence weights
    pub confidence: Vec<f64>,
    /// Momentum vector for opinion dynamics
    pub momentum: Vec<f64>,
    /// Influence radius
    pub influence_radius: f64,
    /// Timestamp of last update
    pub last_update: SystemTime,
}

/// Potential field for opinion dynamics
#[derive(Debug, Clone)]
pub struct PotentialField {
    /// Field strength at grid points
    pub field_values: Vec<Vec<f64>>,
    /// Gradient vectors
    pub gradients: Vec<Vec<Vec<f64>>>,
    /// Critical points (maxima, minima, saddles)
    pub critical_points: Vec<CriticalPoint>,
    /// Field parameters
    pub parameters: FieldParameters,
}

/// Critical points in the potential field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CriticalPoint {
    pub position: Vec<f64>,
    pub point_type: CriticalPointType,
    pub stability: f64,
    pub eigenvalues: Vec<f64>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CriticalPointType {
    LocalMinimum,
    LocalMaximum,
    SaddlePoint,
    FlatRegion,
}

/// Opinion cluster formation
#[derive(Debug, Clone)]
pub struct OpinionCluster {
    pub id: String,
    pub centroid: Vec<f64>,
    pub members: HashSet<String>,
    pub cohesion: f64,
    pub stability: f64,
    pub formation_time: SystemTime,
}

/// Landscape topology characteristics
#[derive(Debug, Clone)]
pub struct LandscapeTopology {
    pub connectivity: f64,
    pub roughness: f64,
    pub fractal_dimension: f64,
    pub percolation_threshold: f64,
}

/// Attractor basin in consensus space
#[derive(Debug, Clone)]
pub struct AttractorBasin {
    pub id: String,
    pub attractor_type: AttractorType,
    pub center: Vec<f64>,
    pub basin_of_attraction: Vec<Vec<f64>>,
    pub strength: f64,
    pub experts_captured: HashSet<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttractorType {
    FixedPoint,
    LimitCycle,
    StrangeAttractor,
    Chaotic,
}

/// Phase space trajectory of expert opinions
#[derive(Debug, Clone)]
pub struct PhaseTrajectory {
    pub expert_id: String,
    pub positions: VecDeque<Vec<f64>>,
    pub velocities: VecDeque<Vec<f64>>,
    pub recurrence_map: RecurrenceMap,
    pub trajectory_type: TrajectoryType,
}

/// Recurrence analysis for trajectory patterns
#[derive(Debug, Clone)]
pub struct RecurrenceMap {
    pub matrix: Vec<Vec<bool>>,
    pub recurrence_rate: f64,
    pub determinism: f64,
    pub entropy: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TrajectoryType {
    Converging,
    Oscillating,
    Chaotic,
    Escaping,
}

/// Bifurcation point in consensus dynamics
#[derive(Debug, Clone)]
pub struct BifurcationPoint {
    pub parameter_value: f64,
    pub bifurcation_type: BifurcationType,
    pub pre_states: Vec<Vec<f64>>,
    pub post_states: Vec<Vec<f64>>,
    pub critical_experts: HashSet<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum BifurcationType {
    PitchforkBifurcation,
    HopfBifurcation,
    SaddleNodeBifurcation,
    TranscriticalBifurcation,
}

/// Field parameters for potential field
#[derive(Debug, Clone)]
pub struct FieldParameters {
    pub coupling_strength: f64,
    pub noise_amplitude: f64,
    pub anharmonicity: f64,
    pub dissipation: f64,
}

/// Emergent consensus coordinator
pub struct EmergentConsensusProtocol {
    /// Consensus state
    consensus_state: Arc<RwLock<EmergentConsensusState>>,
    /// Expert allocation engine
    allocation_engine: Arc<AllocationEngine>,
    /// Dynamics simulator
    dynamics_simulator: Arc<DynamicsSimulator>,
    /// Pattern recognizer
    pattern_recognizer: Arc<PatternRecognizer>,
    /// Consensus event channel
    event_channel: broadcast::Sender<ConsensusEvent>,
}

/// Dynamic expert allocation engine
pub struct AllocationEngine {
    /// Allocation strategies
    strategies: Vec<Box<dyn AllocationStrategy>>,
    /// Resource constraints
    constraints: Arc<RwLock<ResourceConstraints>>,
    /// Allocation history
    history: Arc<RwLock<AllocationHistory>>,
}

/// Allocation strategy trait
pub trait AllocationStrategy: Send + Sync {
    fn allocate(
        &self,
        consensus_state: &EmergentConsensusState,
        constraints: &ResourceConstraints,
    ) -> HashMap<String, Vec<String>>;
    
    fn name(&self) -> &str;
}

/// Resource constraints for allocation
#[derive(Debug, Clone)]
pub struct ResourceConstraints {
    pub max_experts_per_task: usize,
    pub min_experts_per_task: usize,
    pub expert_capacities: HashMap<String, f64>,
    pub task_requirements: HashMap<String, f64>,
}

/// Allocation history tracking
#[derive(Debug, Clone)]
pub struct AllocationHistory {
    pub allocations: VecDeque<AllocationRecord>,
    pub performance_metrics: HashMap<String, f64>,
    pub adaptation_rate: f64,
}

#[derive(Debug, Clone)]
pub struct AllocationRecord {
    pub timestamp: SystemTime,
    pub allocations: HashMap<String, Vec<String>>,
    pub consensus_level: f64,
    pub efficiency: f64,
}

/// Dynamics simulator for consensus evolution
pub struct DynamicsSimulator {
    /// Integration method
    integrator: Box<dyn Integrator>,
    /// Force calculators
    force_calculators: Vec<Box<dyn ForceCalculator>>,
    /// Noise generator
    noise_generator: NoiseGenerator,
    /// Simulation parameters
    parameters: SimulationParameters,
}

/// Integration method trait
pub trait Integrator: Send + Sync {
    fn step(
        &self,
        state: &mut OpinionLandscape,
        forces: &HashMap<String, Vec<f64>>,
        dt: f64,
    );
}

/// Force calculator trait
pub trait ForceCalculator: Send + Sync {
    fn calculate(&self, state: &OpinionLandscape) -> HashMap<String, Vec<f64>>;
    fn name(&self) -> &str;
}

/// Noise generator for stochastic dynamics
#[derive(Debug, Clone)]
pub struct NoiseGenerator {
    pub noise_type: NoiseType,
    pub amplitude: f64,
    pub correlation_time: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum NoiseType {
    WhiteNoise,
    PinkNoise,
    BrownianNoise,
    LevyNoise,
}

/// Simulation parameters
#[derive(Debug, Clone)]
pub struct SimulationParameters {
    pub time_step: f64,
    pub temperature: f64,
    pub damping: f64,
    pub coupling_matrix: Vec<Vec<f64>>,
}

/// Pattern recognition for emergent behaviors
pub struct PatternRecognizer {
    /// Pattern detectors
    detectors: Vec<Box<dyn PatternDetector>>,
    /// Pattern history
    history: Arc<RwLock<Vec<EmergentPattern>>>,
    /// Recognition thresholds
    thresholds: HashMap<String, f64>,
}

/// Pattern detector trait
pub trait PatternDetector: Send + Sync {
    fn detect(&self, state: &EmergentConsensusState) -> Option<EmergentPattern>;
    fn pattern_type(&self) -> &str;
}

/// Emergent pattern description
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergentPattern {
    pub pattern_type: String,
    pub strength: f64,
    pub participating_experts: HashSet<String>,
    pub spatial_extent: f64,
    pub temporal_persistence: Duration,
    pub description: String,
}

/// Consensus events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsensusEvent {
    OpinionConvergence {
        cluster_id: String,
        experts: HashSet<String>,
        convergence_point: Vec<f64>,
    },
    BifurcationDetected {
        bifurcation_type: BifurcationType,
        parameter_value: f64,
        affected_experts: HashSet<String>,
    },
    AttractorFormation {
        attractor_type: AttractorType,
        basin_size: f64,
        captured_experts: HashSet<String>,
    },
    ConsensusReached {
        consensus_vector: Vec<f64>,
        agreement_level: f64,
        participating_experts: usize,
    },
    ChaoticRegimeEntered {
        lyapunov_exponents: Vec<f64>,
        affected_dimensions: Vec<usize>,
    },
    EmergentAllocation {
        task_id: String,
        allocated_experts: Vec<String>,
        emergence_score: f64,
    },
}

impl EmergentConsensusProtocol {
    /// Create new emergent consensus protocol
    pub async fn new(dimensions: usize) -> Self {
        let (event_tx, _) = broadcast::channel(1000);
        
        let consensus_state = EmergentConsensusState {
            opinion_landscape: OpinionLandscape {
                dimensions,
                expert_opinions: HashMap::new(),
                potential_field: PotentialField::new(dimensions),
                clusters: Vec::new(),
                topology: LandscapeTopology {
                    connectivity: 0.0,
                    roughness: 0.0,
                    fractal_dimension: 0.0,
                    percolation_threshold: 0.0,
                },
            },
            attractors: Vec::new(),
            trajectories: HashMap::new(),
            bifurcations: Vec::new(),
            emergence_level: 0.0,
            lyapunov_exponents: vec![0.0; dimensions],
        };

        Self {
            consensus_state: Arc::new(RwLock::new(consensus_state)),
            allocation_engine: Arc::new(AllocationEngine::new()),
            dynamics_simulator: Arc::new(DynamicsSimulator::new()),
            pattern_recognizer: Arc::new(PatternRecognizer::new()),
            event_channel: event_tx,
        }
    }

    /// Initialize expert in consensus space
    pub async fn initialize_expert(
        &self,
        expert_id: &str,
        initial_opinion: Vec<f64>,
        influence_radius: f64,
    ) -> Result<(), String> {
        let mut state = self.consensus_state.write().await;
        
        if initial_opinion.len() != state.opinion_landscape.dimensions {
            return Err("Opinion dimension mismatch".to_string());
        }

        let opinion = OpinionVector {
            values: initial_opinion.clone(),
            confidence: vec![0.5; initial_opinion.len()],
            momentum: vec![0.0; initial_opinion.len()],
            influence_radius,
            last_update: SystemTime::now(),
        };

        state.opinion_landscape.expert_opinions.insert(expert_id.to_string(), opinion);
        
        // Initialize trajectory
        state.trajectories.insert(
            expert_id.to_string(),
            PhaseTrajectory {
                expert_id: expert_id.to_string(),
                positions: VecDeque::from(vec![initial_opinion]),
                velocities: VecDeque::from(vec![vec![0.0; state.opinion_landscape.dimensions]]),
                recurrence_map: RecurrenceMap::new(100),
                trajectory_type: TrajectoryType::Converging,
            },
        );

        Ok(())
    }

    /// Update consensus dynamics
    pub async fn update_dynamics(&self, dt: f64) -> Result<(), String> {
        let mut state = self.consensus_state.write().await;
        
        // Calculate forces on each expert
        let forces = self.dynamics_simulator.calculate_forces(&state.opinion_landscape).await;
        
        // Update opinions
        self.dynamics_simulator.integrate(&mut state.opinion_landscape, &forces, dt).await;
        
        // Update trajectories
        for (expert_id, opinion) in &state.opinion_landscape.expert_opinions {
            if let Some(trajectory) = state.trajectories.get_mut(expert_id) {
                trajectory.positions.push_back(opinion.values.clone());
                trajectory.velocities.push_back(opinion.momentum.clone());
                
                // Keep trajectory buffer bounded
                if trajectory.positions.len() > 1000 {
                    trajectory.positions.pop_front();
                    trajectory.velocities.pop_front();
                }
                
                // Update recurrence map
                trajectory.recurrence_map.update(&trajectory.positions);
            }
        }

        // Detect attractors
        self.detect_attractors(&mut state).await;
        
        // Calculate Lyapunov exponents
        self.calculate_lyapunov_exponents(&mut state).await;
        
        // Update emergence level
        state.emergence_level = self.calculate_emergence_level(&state);

        Ok(())
    }

    /// Perform emergent expert allocation
    pub async fn allocate_experts(
        &self,
        task_id: &str,
        required_expertise: Vec<f64>,
        num_experts: usize,
    ) -> Result<Vec<String>, String> {
        let state = self.consensus_state.read().await;
        
        // Find experts in consensus regions matching required expertise
        let mut candidates = Vec::new();
        
        for (expert_id, opinion) in &state.opinion_landscape.expert_opinions {
            let similarity = self.calculate_similarity(&opinion.values, &required_expertise);
            if similarity > 0.7 {
                candidates.push((expert_id.clone(), similarity));
            }
        }

        // Sort by similarity and consensus participation
        candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        // Select top experts based on emergent clustering
        let selected = self.select_by_emergence(&candidates, &state, num_experts).await;
        
        // Broadcast allocation event
        if !selected.is_empty() {
            let _ = self.event_channel.send(ConsensusEvent::EmergentAllocation {
                task_id: task_id.to_string(),
                allocated_experts: selected.clone(),
                emergence_score: state.emergence_level,
            });
        }

        Ok(selected)
    }

    /// Detect consensus formation
    pub async fn detect_consensus(&self) -> Option<ConsensusResult> {
        let state = self.consensus_state.read().await;
        
        // Check for dominant cluster
        if let Some(largest_cluster) = state.opinion_landscape.clusters.iter()
            .max_by_key(|c| c.members.len())
        {
            let participation = largest_cluster.members.len() as f64 
                / state.opinion_landscape.expert_opinions.len() as f64;
                
            if participation > 0.8 && largest_cluster.cohesion > 0.9 {
                return Some(ConsensusResult {
                    consensus_vector: largest_cluster.centroid.clone(),
                    agreement_level: largest_cluster.cohesion,
                    participating_experts: largest_cluster.members.clone(),
                    stability: largest_cluster.stability,
                });
            }
        }

        None
    }

    /// Detect emergent patterns
    pub async fn detect_patterns(&self) -> Vec<EmergentPattern> {
        let state = self.consensus_state.read().await;
        self.pattern_recognizer.detect(&state).await
    }

    // Helper methods

    async fn detect_attractors(&self, state: &mut EmergentConsensusState) {
        // Simplified attractor detection
        let mut attractors = Vec::new();
        
        // Fixed point detection
        for (expert_id, trajectory) in &state.trajectories {
            if self.is_converged(trajectory) {
                attractors.push(AttractorBasin {
                    id: format!("fixed_{}", expert_id),
                    attractor_type: AttractorType::FixedPoint,
                    center: trajectory.positions.back().unwrap().clone(),
                    basin_of_attraction: Vec::new(),
                    strength: 0.8,
                    experts_captured: HashSet::from([expert_id.clone()]),
                });
            }
        }

        state.attractors = attractors;
    }

    async fn calculate_lyapunov_exponents(&self, state: &mut EmergentConsensusState) {
        // Simplified Lyapunov exponent calculation
        for i in 0..state.opinion_landscape.dimensions {
            state.lyapunov_exponents[i] = 0.1; // Placeholder
        }
    }

    fn calculate_emergence_level(&self, state: &EmergentConsensusState) -> f64 {
        let cluster_score = state.opinion_landscape.clusters.len() as f64 / 10.0;
        let attractor_score = state.attractors.len() as f64 / 5.0;
        let chaos_score = state.lyapunov_exponents.iter().filter(|&&x| x > 0.0).count() as f64 / state.opinion_landscape.dimensions as f64;
        
        (cluster_score + attractor_score + chaos_score) / 3.0
    }

    fn calculate_similarity(&self, vec1: &[f64], vec2: &[f64]) -> f64 {
        let mut sum = 0.0;
        let mut norm1 = 0.0;
        let mut norm2 = 0.0;
        
        for i in 0..vec1.len().min(vec2.len()) {
            sum += vec1[i] * vec2[i];
            norm1 += vec1[i] * vec1[i];
            norm2 += vec2[i] * vec2[i];
        }
        
        if norm1 > 0.0 && norm2 > 0.0 {
            sum / (norm1.sqrt() * norm2.sqrt())
        } else {
            0.0
        }
    }

    async fn select_by_emergence(
        &self,
        candidates: &[(String, f64)],
        state: &EmergentConsensusState,
        num_experts: usize,
    ) -> Vec<String> {
        let mut selected = Vec::new();
        
        // Prefer experts from stable clusters
        for cluster in &state.opinion_landscape.clusters {
            if cluster.stability > 0.8 {
                for expert_id in &cluster.members {
                    if candidates.iter().any(|(id, _)| id == expert_id) {
                        selected.push(expert_id.clone());
                        if selected.len() >= num_experts {
                            return selected;
                        }
                    }
                }
            }
        }

        // Fill remaining from top candidates
        for (expert_id, _) in candidates {
            if !selected.contains(expert_id) {
                selected.push(expert_id.clone());
                if selected.len() >= num_experts {
                    break;
                }
            }
        }

        selected
    }

    fn is_converged(&self, trajectory: &PhaseTrajectory) -> bool {
        if trajectory.positions.len() < 10 {
            return false;
        }

        // Check if recent positions are stable
        let recent: Vec<_> = trajectory.positions.iter().rev().take(10).collect();
        let first = recent[0];
        
        recent.iter().all(|pos| {
            self.calculate_distance(pos, first) < 0.1
        })
    }

    fn calculate_distance(&self, vec1: &[f64], vec2: &[f64]) -> f64 {
        vec1.iter().zip(vec2.iter())
            .map(|(a, b)| (a - b).powi(2))
            .sum::<f64>()
            .sqrt()
    }
}

/// Consensus result
#[derive(Debug, Clone)]
pub struct ConsensusResult {
    pub consensus_vector: Vec<f64>,
    pub agreement_level: f64,
    pub participating_experts: HashSet<String>,
    pub stability: f64,
}

// Implementations for helper structs

impl PotentialField {
    fn new(dimensions: usize) -> Self {
        Self {
            field_values: vec![vec![0.0; 100]; dimensions],
            gradients: vec![vec![vec![0.0; dimensions]; 100]; dimensions],
            critical_points: Vec::new(),
            parameters: FieldParameters {
                coupling_strength: 1.0,
                noise_amplitude: 0.1,
                anharmonicity: 0.0,
                dissipation: 0.1,
            },
        }
    }
}

impl RecurrenceMap {
    fn new(size: usize) -> Self {
        Self {
            matrix: vec![vec![false; size]; size],
            recurrence_rate: 0.0,
            determinism: 0.0,
            entropy: 0.0,
        }
    }

    fn update(&mut self, positions: &VecDeque<Vec<f64>>) {
        // Update recurrence matrix
        let n = positions.len().min(self.matrix.len());
        for i in 0..n {
            for j in 0..n {
                let dist = positions[i].iter()
                    .zip(positions[j].iter())
                    .map(|(a, b)| (a - b).powi(2))
                    .sum::<f64>()
                    .sqrt();
                self.matrix[i][j] = dist < 0.1;
            }
        }
    }
}

impl AllocationEngine {
    fn new() -> Self {
        Self {
            strategies: Vec::new(),
            constraints: Arc::new(RwLock::new(ResourceConstraints {
                max_experts_per_task: 10,
                min_experts_per_task: 1,
                expert_capacities: HashMap::new(),
                task_requirements: HashMap::new(),
            })),
            history: Arc::new(RwLock::new(AllocationHistory {
                allocations: VecDeque::new(),
                performance_metrics: HashMap::new(),
                adaptation_rate: 0.1,
            })),
        }
    }
}

impl DynamicsSimulator {
    fn new() -> Self {
        Self {
            integrator: Box::new(RungeKutta4Integrator),
            force_calculators: Vec::new(),
            noise_generator: NoiseGenerator {
                noise_type: NoiseType::WhiteNoise,
                amplitude: 0.1,
                correlation_time: 1.0,
            },
            parameters: SimulationParameters {
                time_step: 0.01,
                temperature: 1.0,
                damping: 0.1,
                coupling_matrix: Vec::new(),
            },
        }
    }

    async fn calculate_forces(&self, landscape: &OpinionLandscape) -> HashMap<String, Vec<f64>> {
        let mut all_forces = HashMap::new();
        
        for calculator in &self.force_calculators {
            let forces = calculator.calculate(landscape);
            for (expert_id, force) in forces {
                all_forces.entry(expert_id)
                    .or_insert(vec![0.0; landscape.dimensions])
                    .iter_mut()
                    .zip(force.iter())
                    .for_each(|(a, b)| *a += b);
            }
        }
        
        all_forces
    }

    async fn integrate(
        &self,
        landscape: &mut OpinionLandscape,
        forces: &HashMap<String, Vec<f64>>,
        dt: f64,
    ) {
        self.integrator.step(landscape, forces, dt);
    }
}

/// Example integrator implementation
struct RungeKutta4Integrator;

impl Integrator for RungeKutta4Integrator {
    fn step(
        &self,
        state: &mut OpinionLandscape,
        forces: &HashMap<String, Vec<f64>>,
        dt: f64,
    ) {
        // Simplified RK4 implementation
        for (expert_id, opinion) in state.expert_opinions.iter_mut() {
            if let Some(force) = forces.get(expert_id) {
                for i in 0..opinion.values.len() {
                    opinion.momentum[i] += force[i] * dt;
                    opinion.values[i] += opinion.momentum[i] * dt;
                }
            }
        }
    }
}

impl PatternRecognizer {
    fn new() -> Self {
        Self {
            detectors: Vec::new(),
            history: Arc::new(RwLock::new(Vec::new())),
            thresholds: HashMap::new(),
        }
    }

    async fn detect(&self, state: &EmergentConsensusState) -> Vec<EmergentPattern> {
        let mut patterns = Vec::new();
        
        for detector in &self.detectors {
            if let Some(pattern) = detector.detect(state) {
                patterns.push(pattern);
            }
        }
        
        patterns
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_consensus_initialization() {
        let protocol = EmergentConsensusProtocol::new(3).await;
        
        protocol.initialize_expert(
            "expert1",
            vec![0.5, 0.5, 0.5],
            1.0,
        ).await.unwrap();

        let state = protocol.consensus_state.read().await;
        assert!(state.opinion_landscape.expert_opinions.contains_key("expert1"));
    }

    #[tokio::test]
    async fn test_dynamics_update() {
        let protocol = EmergentConsensusProtocol::new(2).await;
        
        // Initialize multiple experts
        for i in 0..5 {
            protocol.initialize_expert(
                &format!("expert{}", i),
                vec![rand::random(), rand::random()],
                1.0,
            ).await.unwrap();
        }

        // Update dynamics
        protocol.update_dynamics(0.1).await.unwrap();

        let state = protocol.consensus_state.read().await;
        assert_eq!(state.trajectories.len(), 5);
    }
}