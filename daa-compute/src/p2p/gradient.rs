//! Gradient sharing and all-reduce implementation
//!
//! This module provides efficient gradient aggregation algorithms
//! for distributed training, including compression and fault tolerance.

use anyhow::{anyhow, Result};
use libp2p::{gossipsub::IdentTopic, PeerId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::RwLock;
use tracing::{info, warn};

use super::compression::CompressionMethod;

lazy_static::lazy_static! {
    pub static ref GRADIENT_TOPIC: IdentTopic = IdentTopic::new("gradients");
}

/// Message containing gradient update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradientMessage {
    #[serde(with = "peer_id_serde")]
    pub peer_id: PeerId,
    pub round: u64,
    pub compressed_gradient: Vec<u8>,
    pub timestamp: SystemTime,
}

mod peer_id_serde {
    use super::*;
    use serde::{Deserializer, Serializer};
    use std::str::FromStr;

    pub fn serialize<S>(peer_id: &PeerId, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&peer_id.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<PeerId, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        PeerId::from_str(&s).map_err(serde::de::Error::custom)
    }
}

/// All-reduce algorithm for gradient aggregation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllReduceAlgorithm {
    /// Ring all-reduce (bandwidth efficient)
    Ring,
    /// Tree all-reduce (latency efficient)
    Tree,
    /// Butterfly all-reduce (balanced)
    Butterfly,
    /// Hierarchical all-reduce (for geo-distributed)
    Hierarchical,
}

/// Gradients received per round, keyed by the contributing peer.
type GradientsByRound = HashMap<u64, HashMap<PeerId, Vec<f32>>>;

/// Gradient manager for handling aggregation
pub struct GradientManager {
    peer_id: PeerId,
    current_round: Arc<RwLock<u64>>,
    gradients: Arc<RwLock<GradientsByRound>>,
    compression_method: CompressionMethod,
    algorithm: AllReduceAlgorithm,
    min_peers_for_aggregation: usize,
}

impl GradientManager {
    pub fn new(peer_id: PeerId, compression_level: u32) -> Self {
        Self {
            peer_id,
            current_round: Arc::new(RwLock::new(0)),
            gradients: Arc::new(RwLock::new(HashMap::new())),
            compression_method: CompressionMethod::Zstd {
                level: compression_level as i32,
            },
            algorithm: AllReduceAlgorithm::Ring,
            min_peers_for_aggregation: 2,
        }
    }

    pub fn current_round(&self) -> u64 {
        // Note: This is a synchronous method but we need async to read the lock
        // In production, we'd handle this differently
        0 // Placeholder
    }

    /// Compress gradient using configured method
    pub fn compress_gradient(&self, gradient: &[f32]) -> Result<Vec<u8>> {
        // Quantize to int8 for 4x compression (as mentioned in Prime)
        let quantized = quantize_gradient(gradient)?;
        self.compression_method.compress(&quantized)
    }

    /// Decompress gradient
    pub fn decompress_gradient(&self, compressed: &[u8]) -> Result<Vec<f32>> {
        let quantized = self.compression_method.decompress(compressed)?;
        dequantize_gradient(&quantized)
    }

    /// Handle incoming gradient message
    pub async fn handle_gradient_message(&self, message: GradientMessage) -> Result<()> {
        let gradient = self.decompress_gradient(&message.compressed_gradient)?;

        // The write guard must be released before aggregating, which takes a
        // read lock on the same map.
        let held = {
            let mut gradients = self.gradients.write().await;
            let round_gradients = gradients.entry(message.round).or_insert_with(HashMap::new);
            round_gradients.insert(message.peer_id, gradient);
            round_gradients.len()
        };

        info!(
            "Received gradient from {} for round {}",
            message.peer_id, message.round
        );

        // Check if we have enough gradients to aggregate. A round that cannot
        // be reduced yet (e.g. a peer sent a malformed gradient) must not cause
        // this, possibly valid, message to be rejected.
        if held >= self.min_peers_for_aggregation {
            if let Err(e) = self.try_aggregate_round(message.round).await {
                warn!("Round {} not aggregated: {}", message.round, e);
            }
        }

        Ok(())
    }

    /// Gradients currently held for `round`, keyed by contributing peer.
    pub async fn gradients_for_round(&self, round: u64) -> HashMap<PeerId, Vec<f32>> {
        self.gradients
            .read()
            .await
            .get(&round)
            .cloned()
            .unwrap_or_default()
    }

    /// Try to aggregate gradients for a round
    async fn try_aggregate_round(&self, round: u64) -> Result<()> {
        let gradients = self.gradients.read().await;

        if let Some(round_gradients) = gradients.get(&round) {
            if round_gradients.len() >= self.min_peers_for_aggregation
                // All-reduce is computed from this peer's position, so it can
                // only run once we have contributed to the round ourselves.
                && round_gradients.contains_key(&self.peer_id)
            {
                info!(
                    "Aggregating {} gradients for round {}",
                    round_gradients.len(),
                    round
                );

                self.all_reduce(self.algorithm, round_gradients).await?;
            }
        }

        Ok(())
    }

    /// Reduce one round of per-peer gradients to their average using `algorithm`.
    ///
    /// `gradients` must contain an entry for this manager's own peer ID, and
    /// every gradient must have the same length and only finite values.
    pub async fn all_reduce(
        &self,
        algorithm: AllReduceAlgorithm,
        gradients: &HashMap<PeerId, Vec<f32>>,
    ) -> Result<Vec<f32>> {
        let own = gradients
            .get(&self.peer_id)
            .ok_or_else(|| anyhow!("Our peer ID not found in gradient list"))?;
        for (peer, gradient) in gradients {
            if gradient.len() != own.len() {
                return Err(anyhow!(
                    "Gradient from {} has {} values, expected {}",
                    peer,
                    gradient.len(),
                    own.len()
                ));
            }
            if gradient.iter().any(|v| !v.is_finite()) {
                return Err(anyhow!("Gradient from {} has non-finite values", peer));
            }
        }

        match algorithm {
            AllReduceAlgorithm::Ring => self.ring_allreduce(gradients).await,
            AllReduceAlgorithm::Tree => self.tree_allreduce(gradients).await,
            AllReduceAlgorithm::Butterfly => self.butterfly_allreduce(gradients).await,
            AllReduceAlgorithm::Hierarchical => self.hierarchical_allreduce(gradients).await,
        }
    }

    /// Ring all-reduce implementation
    async fn ring_allreduce(&self, gradients: &HashMap<PeerId, Vec<f32>>) -> Result<Vec<f32>> {
        // Sort peers to ensure consistent ordering
        let mut peers: Vec<_> = gradients.keys().cloned().collect();
        peers.sort();

        // Find our position in the ring
        let our_position = peers
            .iter()
            .position(|p| p == &self.peer_id)
            .ok_or_else(|| anyhow!("Our peer ID not found in gradient list"))?;

        // Initialize with our gradient
        let mut accumulated = gradients[&self.peer_id].clone();
        let n_peers = peers.len();

        // Ring reduce phase: each peer accumulates gradients
        for step in 1..n_peers {
            let source_idx = (our_position + n_peers - step) % n_peers;
            let source_peer = &peers[source_idx];

            if let Some(gradient) = gradients.get(source_peer) {
                // Add gradient to accumulator
                for (acc, grad) in accumulated.iter_mut().zip(gradient.iter()) {
                    *acc += grad;
                }
            }
        }

        // Average the accumulated gradients
        let scale = 1.0 / n_peers as f32;
        for value in &mut accumulated {
            *value *= scale;
        }

        Ok(accumulated)
    }

    /// Tree all-reduce implementation
    async fn tree_allreduce(&self, gradients: &HashMap<PeerId, Vec<f32>>) -> Result<Vec<f32>> {
        // Build a binary tree structure
        let mut peers: Vec<_> = gradients.keys().cloned().collect();
        peers.sort();

        // Reduce phase: aggregate up the tree
        let mut level_gradients = gradients.clone();
        let mut level_size = peers.len();

        while level_size > 1 {
            let mut next_level = HashMap::new();

            for i in (0..level_size).step_by(2) {
                if i + 1 < level_size {
                    // Aggregate pairs
                    let peer1 = &peers[i];
                    let peer2 = &peers[i + 1];

                    if let (Some(grad1), Some(grad2)) =
                        (level_gradients.get(peer1), level_gradients.get(peer2))
                    {
                        let mut aggregated = grad1.clone();
                        for (a, b) in aggregated.iter_mut().zip(grad2.iter()) {
                            *a = (*a + *b) / 2.0;
                        }
                        next_level.insert(*peer1, aggregated);
                    }
                } else {
                    // Odd peer passes through
                    if let Some(grad) = level_gradients.get(&peers[i]) {
                        next_level.insert(peers[i], grad.clone());
                    }
                }
            }

            level_gradients = next_level;
            level_size = level_size.div_ceil(2);
        }

        // The root contains the aggregated gradient
        level_gradients
            .values()
            .next()
            .cloned()
            .ok_or_else(|| anyhow!("No aggregated gradient found"))
    }

    /// Butterfly all-reduce implementation
    async fn butterfly_allreduce(&self, gradients: &HashMap<PeerId, Vec<f32>>) -> Result<Vec<f32>> {
        // Butterfly pattern for balanced latency and bandwidth
        let mut peers: Vec<_> = gradients.keys().cloned().collect();
        peers.sort();

        let n = peers.len();
        let our_idx = peers
            .iter()
            .position(|p| p == &self.peer_id)
            .ok_or_else(|| anyhow!("Our peer ID not found"))?;

        let mut current = gradients[&self.peer_id].clone();

        // Butterfly stages
        let stages = (n as f64).log2().ceil() as usize;

        for stage in 0..stages {
            let distance = 1 << stage;
            let partner_idx = our_idx ^ distance;

            if partner_idx < n {
                let partner = &peers[partner_idx];
                if let Some(partner_grad) = gradients.get(partner) {
                    // Exchange and aggregate
                    for (curr, part) in current.iter_mut().zip(partner_grad.iter()) {
                        *curr = (*curr + *part) / 2.0;
                    }
                }
            }
        }

        Ok(current)
    }

    /// Hierarchical all-reduce for geo-distributed nodes
    async fn hierarchical_allreduce(
        &self,
        gradients: &HashMap<PeerId, Vec<f32>>,
    ) -> Result<Vec<f32>> {
        // Group nodes by region (simplified: by peer ID prefix)
        let mut regions: HashMap<u8, Vec<PeerId>> = HashMap::new();

        for peer in gradients.keys() {
            let region = peer.to_bytes()[0]; // Simple region assignment
            regions.entry(region).or_default().push(*peer);
        }

        // First level: aggregate within regions
        let mut regional_aggregates = HashMap::new();

        for (region, region_peers) in regions {
            let mut region_sum = vec![0.0; gradients.values().next().unwrap().len()];
            let mut count = 0;

            for peer in region_peers {
                if let Some(grad) = gradients.get(&peer) {
                    for (sum, val) in region_sum.iter_mut().zip(grad.iter()) {
                        *sum += val;
                    }
                    count += 1;
                }
            }

            if count > 0 {
                for val in &mut region_sum {
                    *val /= count as f32;
                }
                regional_aggregates.insert(region, region_sum);
            }
        }

        // Second level: aggregate across regions
        let mut global_sum = vec![0.0; gradients.values().next().unwrap().len()];
        let region_count = regional_aggregates.len();

        for (_, regional_grad) in regional_aggregates {
            for (sum, val) in global_sum.iter_mut().zip(regional_grad.iter()) {
                *sum += val;
            }
        }

        for val in &mut global_sum {
            *val /= region_count as f32;
        }

        Ok(global_sum)
    }

    /// Get the current aggregated gradient if available
    pub async fn get_aggregated_gradient(&self) -> Result<Option<Vec<f32>>> {
        let current_round = *self.current_round.read().await;
        let gradients = self.gradients.read().await;

        if let Some(round_gradients) = gradients.get(&current_round) {
            if round_gradients.len() >= self.min_peers_for_aggregation
                && round_gradients.contains_key(&self.peer_id)
            {
                Ok(Some(
                    self.all_reduce(self.algorithm, round_gradients).await?,
                ))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }
}

/// Quantize float32 gradient to int8 for compression
pub fn quantize_gradient(gradient: &[f32]) -> Result<Vec<u8>> {
    // Find min and max for quantization
    let min = gradient.iter().fold(f32::INFINITY, |a, &b| a.min(b));
    let max = gradient.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));

    let mut quantized = Vec::with_capacity(gradient.len() + 8);

    // Store min and max for dequantization
    quantized.extend_from_slice(&min.to_le_bytes());
    quantized.extend_from_slice(&max.to_le_bytes());

    if min == max {
        // All values are the same; the header alone reconstructs them
        quantized.resize(gradient.len() + 8, 0);
        return Ok(quantized);
    }

    let scale = 255.0 / (max - min);

    // Quantize values
    for &value in gradient {
        let normalized = (value - min) * scale;
        let quantized_value = normalized.round() as u8;
        quantized.push(quantized_value);
    }

    Ok(quantized)
}

/// Dequantize int8 gradient back to float32
pub fn dequantize_gradient(quantized: &[u8]) -> Result<Vec<f32>> {
    if quantized.len() < 8 {
        return Err(anyhow!("Invalid quantized gradient"));
    }

    // Extract min and max
    let min = f32::from_le_bytes([quantized[0], quantized[1], quantized[2], quantized[3]]);
    let max = f32::from_le_bytes([quantized[4], quantized[5], quantized[6], quantized[7]]);

    if min == max {
        // All values were the same
        return Ok(vec![min; quantized.len() - 8]);
    }

    let scale = (max - min) / 255.0;
    let mut gradient = Vec::with_capacity(quantized.len() - 8);

    // Dequantize values
    for &quantized_value in &quantized[8..] {
        let value = min + (quantized_value as f32) * scale;
        gradient.push(value);
    }

    Ok(gradient)
}

/// All-reduce trait for different implementations
#[async_trait::async_trait]
pub trait AllReduce: Send + Sync {
    /// Perform all-reduce on the given values
    async fn all_reduce(&self, values: Vec<f32>) -> Result<Vec<f32>>;

    /// Get the algorithm name
    fn algorithm_name(&self) -> &'static str;
}
