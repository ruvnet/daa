//! DAG tip selection implementation.

use crate::vertex::{Vertex, VertexId};
use rand::{thread_rng, Rng};
use std::collections::{HashMap, HashSet};
use thiserror::Error;

/// Errors that can occur during tip selection.
#[derive(Debug, Error)]
pub enum TipSelectionError {
    /// No valid tips available
    #[error("No valid tips available")]
    NoValidTips,

    /// Invalid tip reference
    #[error("Invalid tip reference")]
    InvalidTip,

    /// Selection failure
    #[error("Selection failure")]
    SelectionFailed,

    /// MCMC walk failed
    #[error("MCMC walk failed: {0}")]
    McmcWalkFailed(String),

    /// Weight calculation failed
    #[error("Weight calculation failed")]
    WeightCalculationFailed,
}

/// Tip selection algorithm configuration.
#[derive(Debug, Clone)]
pub struct TipSelectionConfig {
    /// Number of tips to select
    pub tip_count: usize,

    /// Maximum tip age (in seconds)
    pub max_age: u64,

    /// Minimum confidence score
    pub min_confidence: f64,

    /// MCMC walk length
    pub mcmc_walk_length: usize,

    /// Alpha parameter for weighted selection
    pub alpha: f64,

    /// Maximum number of attempts
    pub max_attempts: usize,
}

impl Default for TipSelectionConfig {
    fn default() -> Self {
        Self {
            tip_count: 2,
            max_age: 3600, // 1 hour
            min_confidence: 0.5,
            mcmc_walk_length: 1000,
            alpha: 0.001,
            max_attempts: 50,
        }
    }
}

/// Parent selection algorithm type
#[derive(Debug, Clone, PartialEq)]
pub enum ParentSelectionAlgorithm {
    /// Random selection from tips
    Random,
    /// Weighted random selection based on vertex weight
    WeightedRandom,
    /// Monte Carlo Markov Chain (MCMC) walk
    McmcWalk,
}

/// Vertex weight information for parent selection
#[derive(Debug, Clone)]
pub struct VertexWeight {
    /// Cumulative weight of the vertex
    pub cumulative_weight: f64,
    /// Direct weight of the vertex
    pub direct_weight: f64,
    /// Number of approvers
    pub approvers: usize,
    /// Last update timestamp
    pub last_updated: u64,
}

/// DAG tip selection trait defining the interface for tip selection algorithms.
pub trait TipSelection {
    /// Initialize tip selection with configuration.
    fn init(config: TipSelectionConfig) -> Result<(), TipSelectionError>;

    /// Select tips for a new vertex.
    fn select_tips(&self) -> Result<Vec<VertexId>, TipSelectionError>;

    /// Check if a vertex is eligible as a tip.
    fn is_valid_tip(&self, vertex: &Vertex) -> bool;

    /// Calculate confidence score for a tip.
    fn calculate_confidence(&self, tip: &VertexId) -> f64;

    /// Update tip pool with new vertex.
    fn update_tips(&mut self, vertex: &Vertex) -> Result<(), TipSelectionError>;
}

/// Advanced tip selection implementation with MCMC and weighted selection
pub struct AdvancedTipSelection {
    /// Configuration
    config: TipSelectionConfig,

    /// Current tips
    tips: HashSet<VertexId>,

    /// Vertex weights for weighted selection
    weights: HashMap<VertexId, VertexWeight>,

    /// Vertex adjacency information
    adjacency: HashMap<VertexId, HashSet<VertexId>>,

    /// Reverse adjacency (children)
    reverse_adjacency: HashMap<VertexId, HashSet<VertexId>>,

    /// Algorithm to use
    algorithm: ParentSelectionAlgorithm,
}

impl AdvancedTipSelection {
    /// Create a new advanced tip selection instance
    pub fn new(config: TipSelectionConfig, algorithm: ParentSelectionAlgorithm) -> Self {
        Self {
            config,
            tips: HashSet::new(),
            weights: HashMap::new(),
            adjacency: HashMap::new(),
            reverse_adjacency: HashMap::new(),
            algorithm,
        }
    }

    /// Add a vertex to the DAG structure
    pub fn add_vertex(&mut self, vertex: &Vertex) -> Result<(), TipSelectionError> {
        let vertex_id = vertex.id.clone();
        let parents = vertex.parents();

        // Add to adjacency lists
        self.adjacency.insert(vertex_id.clone(), parents.clone());

        // Update reverse adjacency
        for parent in &parents {
            self.reverse_adjacency
                .entry(parent.clone())
                .or_default()
                .insert(vertex_id.clone());
        }

        // Remove parents from tips (they now have children)
        for parent in &parents {
            self.tips.remove(parent);
        }

        // Add this vertex as a new tip
        self.tips.insert(vertex_id.clone());

        // Update weights
        self.update_vertex_weight(&vertex_id)?;

        Ok(())
    }

    /// Update weight for a vertex
    fn update_vertex_weight(&mut self, vertex_id: &VertexId) -> Result<(), TipSelectionError> {
        let approvers = self
            .reverse_adjacency
            .get(vertex_id)
            .map(|children| children.len())
            .unwrap_or(0);

        let direct_weight = 1.0;
        let cumulative_weight = self.calculate_cumulative_weight(vertex_id)?;

        let weight = VertexWeight {
            cumulative_weight,
            direct_weight,
            approvers,
            last_updated: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        self.weights.insert(vertex_id.clone(), weight);
        Ok(())
    }

    /// Calculate cumulative weight using DFS
    fn calculate_cumulative_weight(&self, vertex_id: &VertexId) -> Result<f64, TipSelectionError> {
        let mut visited = HashSet::new();
        self.calculate_cumulative_weight_recursive(vertex_id, &mut visited)
    }

    fn calculate_cumulative_weight_recursive(
        &self,
        vertex_id: &VertexId,
        visited: &mut HashSet<VertexId>,
    ) -> Result<f64, TipSelectionError> {
        if visited.contains(vertex_id) {
            return Ok(0.0); // Avoid cycles
        }

        visited.insert(vertex_id.clone());

        let direct_weight = self
            .weights
            .get(vertex_id)
            .map(|w| w.direct_weight)
            .unwrap_or(1.0);

        let mut cumulative = direct_weight;

        if let Some(children) = self.reverse_adjacency.get(vertex_id) {
            for child in children {
                cumulative += self.calculate_cumulative_weight_recursive(child, visited)?;
            }
        }

        Ok(cumulative)
    }

    /// Perform MCMC walk for tip selection
    fn mcmc_walk(&self, start: &VertexId) -> Result<VertexId, TipSelectionError> {
        let mut current = start.clone();
        let mut rng = thread_rng();

        for _ in 0..self.config.mcmc_walk_length {
            // Get children of current vertex
            let children = self
                .reverse_adjacency
                .get(&current)
                .cloned()
                .unwrap_or_default();

            if children.is_empty() {
                // Reached a tip
                return Ok(current);
            }

            // Calculate transition probabilities based on weights
            let mut transition_weights = Vec::new();
            let mut candidates = Vec::new();

            for child in &children {
                let weight = self
                    .weights
                    .get(child)
                    .map(|w| w.cumulative_weight)
                    .unwrap_or(1.0);

                // Apply exponential transformation for better selection
                let transition_weight = (-self.config.alpha * weight).exp();
                transition_weights.push(transition_weight);
                candidates.push(child.clone());
            }

            // Select next vertex based on weights
            let total_weight: f64 = transition_weights.iter().sum();
            if total_weight == 0.0 {
                // Uniform selection if all weights are zero
                let idx = rng.gen_range(0..candidates.len());
                current = candidates[idx].clone();
            } else {
                let mut cumulative = 0.0;
                let target = rng.gen::<f64>() * total_weight;

                for (i, &weight) in transition_weights.iter().enumerate() {
                    cumulative += weight;
                    if cumulative >= target {
                        current = candidates[i].clone();
                        break;
                    }
                }
            }
        }

        Ok(current)
    }

    /// Weighted random selection from tips
    fn weighted_random_selection(&self) -> Result<Vec<VertexId>, TipSelectionError> {
        if self.tips.is_empty() {
            return Err(TipSelectionError::NoValidTips);
        }

        let mut rng = thread_rng();
        let mut selected = Vec::new();
        let mut available_tips: Vec<_> = self.tips.iter().cloned().collect();

        for _ in 0..self.config.tip_count.min(available_tips.len()) {
            if available_tips.is_empty() {
                break;
            }

            // Calculate weights for remaining tips
            let mut weights = Vec::new();
            for tip in &available_tips {
                let weight = self
                    .weights
                    .get(tip)
                    .map(|w| w.cumulative_weight)
                    .unwrap_or(1.0);
                weights.push(weight);
            }

            // Select based on weights
            let total_weight: f64 = weights.iter().sum();
            if total_weight == 0.0 {
                // Uniform selection
                let idx = rng.gen_range(0..available_tips.len());
                selected.push(available_tips.remove(idx));
            } else {
                let mut cumulative = 0.0;
                let target = rng.gen::<f64>() * total_weight;

                for (i, &weight) in weights.iter().enumerate() {
                    cumulative += weight;
                    if cumulative >= target {
                        selected.push(available_tips.remove(i));
                        break;
                    }
                }
            }
        }

        Ok(selected)
    }

    /// Random selection from tips
    fn random_selection(&self) -> Result<Vec<VertexId>, TipSelectionError> {
        if self.tips.is_empty() {
            return Err(TipSelectionError::NoValidTips);
        }

        let mut rng = thread_rng();
        let mut tips: Vec<_> = self.tips.iter().cloned().collect();

        // Shuffle and take the required number
        for i in 0..tips.len() {
            let j = rng.gen_range(i..tips.len());
            tips.swap(i, j);
        }

        Ok(tips.into_iter().take(self.config.tip_count).collect())
    }
}

impl TipSelection for AdvancedTipSelection {
    fn init(config: TipSelectionConfig) -> Result<(), TipSelectionError> {
        // Validation
        if config.tip_count == 0 {
            return Err(TipSelectionError::SelectionFailed);
        }

        if config.mcmc_walk_length == 0 {
            return Err(TipSelectionError::McmcWalkFailed(
                "Walk length must be positive".to_string(),
            ));
        }

        Ok(())
    }

    fn select_tips(&self) -> Result<Vec<VertexId>, TipSelectionError> {
        match self.algorithm {
            ParentSelectionAlgorithm::Random => self.random_selection(),
            ParentSelectionAlgorithm::WeightedRandom => self.weighted_random_selection(),
            ParentSelectionAlgorithm::McmcWalk => {
                // For MCMC, start from genesis and walk to tips
                if self.tips.is_empty() {
                    return Err(TipSelectionError::NoValidTips);
                }

                let mut selected = Vec::new();
                let mut rng = thread_rng();

                for _ in 0..self.config.tip_count {
                    // Find a genesis or low-weight vertex to start from
                    let start_candidates: Vec<_> = self
                        .weights
                        .iter()
                        .filter(|(_, w)| w.approvers == 0) // Genesis vertices
                        .map(|(id, _)| id.clone())
                        .collect();

                    let start = if start_candidates.is_empty() {
                        // Use random tip if no genesis found
                        let tips: Vec<_> = self.tips.iter().collect();
                        tips[rng.gen_range(0..tips.len())].clone()
                    } else {
                        start_candidates[rng.gen_range(0..start_candidates.len())].clone()
                    };

                    match self.mcmc_walk(&start) {
                        Ok(tip) => {
                            if !selected.contains(&tip) {
                                selected.push(tip);
                            }
                        }
                        Err(_) => {
                            // Fallback to random selection
                            let tips: Vec<_> = self.tips.iter().collect();
                            let random_tip = tips[rng.gen_range(0..tips.len())].clone();
                            if !selected.contains(&random_tip) {
                                selected.push(random_tip);
                            }
                        }
                    }
                }

                Ok(selected)
            }
        }
    }

    fn is_valid_tip(&self, vertex: &Vertex) -> bool {
        let vertex_id = &vertex.id;

        // Check if vertex has no children (is a tip)
        if let Some(children) = self.reverse_adjacency.get(vertex_id) {
            if !children.is_empty() {
                return false;
            }
        }

        // Check age constraint
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        if current_time - vertex.timestamp > self.config.max_age {
            return false;
        }

        // Check confidence constraint
        if let Some(weight) = self.weights.get(vertex_id) {
            if weight.cumulative_weight < self.config.min_confidence {
                return false;
            }
        }

        true
    }

    fn calculate_confidence(&self, tip: &VertexId) -> f64 {
        self.weights
            .get(tip)
            .map(|w| w.cumulative_weight)
            .unwrap_or(0.0)
    }

    fn update_tips(&mut self, vertex: &Vertex) -> Result<(), TipSelectionError> {
        self.add_vertex(vertex)
    }
}
