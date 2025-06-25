//! Gradient sharing and all-reduce implementation
//!
//! This module provides efficient gradient aggregation algorithms
//! for distributed training, including compression and fault tolerance.

use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use std::sync::Arc;
use tokio::sync::RwLock;
use libp2p::{PeerId, gossipsub::IdentTopic};
use serde::{Serialize, Deserialize};
use anyhow::{Result, anyhow};
use tracing::info;

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

/// Gradient manager for handling aggregation
pub struct GradientManager {
    peer_id: PeerId,
    current_round: Arc<RwLock<u64>>,
    gradients: Arc<RwLock<HashMap<u64, HashMap<PeerId, Vec<f32>>>>>,
    compression_method: CompressionMethod,
    algorithm: AllReduceAlgorithm,
    round_timeout: Duration,
    min_peers_for_aggregation: usize,
}

impl GradientManager {
    pub fn new(peer_id: PeerId, compression_level: u32) -> Self {
        Self {
            peer_id,
            current_round: Arc::new(RwLock::new(0)),
            gradients: Arc::new(RwLock::new(HashMap::new())),
            compression_method: CompressionMethod::Zstd { level: compression_level as i32 },
            algorithm: AllReduceAlgorithm::Ring,
            round_timeout: Duration::from_secs(30),
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
        
        let mut gradients = self.gradients.write().await;
        let round_gradients = gradients.entry(message.round).or_insert_with(HashMap::new);
        round_gradients.insert(message.peer_id, gradient);
        
        info!("Received gradient from {} for round {}", message.peer_id, message.round);
        
        // Check if we have enough gradients to aggregate
        if round_gradients.len() >= self.min_peers_for_aggregation {
            self.try_aggregate_round(message.round).await?;
        }
        
        Ok(())
    }

    /// Try to aggregate gradients for a round
    async fn try_aggregate_round(&self, round: u64) -> Result<()> {
        let gradients = self.gradients.read().await;
        
        if let Some(round_gradients) = gradients.get(&round) {
            if round_gradients.len() >= self.min_peers_for_aggregation {
                info!("Aggregating {} gradients for round {}", round_gradients.len(), round);
                
                match self.algorithm {
                    AllReduceAlgorithm::Ring => {
                        self.ring_allreduce(round_gradients).await?;
                    }
                    AllReduceAlgorithm::Tree => {
                        self.tree_allreduce(round_gradients).await?;
                    }
                    AllReduceAlgorithm::Butterfly => {
                        self.butterfly_allreduce(round_gradients).await?;
                    }
                    AllReduceAlgorithm::Hierarchical => {
                        self.hierarchical_allreduce(round_gradients).await?;
                    }
                }
            }
        }
        
        Ok(())
    }

    /// Ring all-reduce implementation
    async fn ring_allreduce(&self, gradients: &HashMap<PeerId, Vec<f32>>) -> Result<Vec<f32>> {
        // Sort peers to ensure consistent ordering
        let mut peers: Vec<_> = gradients.keys().cloned().collect();
        peers.sort();
        
        // Find our position in the ring
        let our_position = peers.iter().position(|p| p == &self.peer_id)
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
        
        let our_idx = peers.iter().position(|p| p == &self.peer_id)
            .ok_or_else(|| anyhow!("Our peer ID not found"))?;
        
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
                    
                    if let (Some(grad1), Some(grad2)) = (level_gradients.get(peer1), level_gradients.get(peer2)) {
                        let mut aggregated = grad1.clone();
                        for (a, b) in aggregated.iter_mut().zip(grad2.iter()) {
                            *a = (*a + *b) / 2.0;
                        }
                        next_level.insert(peer1.clone(), aggregated);
                    }
                } else {
                    // Odd peer passes through
                    if let Some(grad) = level_gradients.get(&peers[i]) {
                        next_level.insert(peers[i].clone(), grad.clone());
                    }
                }
            }
            
            level_gradients = next_level;
            level_size = (level_size + 1) / 2;
        }
        
        // The root contains the aggregated gradient
        level_gradients.values().next().cloned()
            .ok_or_else(|| anyhow!("No aggregated gradient found"))
    }

    /// Butterfly all-reduce implementation
    async fn butterfly_allreduce(&self, gradients: &HashMap<PeerId, Vec<f32>>) -> Result<Vec<f32>> {
        // Butterfly pattern for balanced latency and bandwidth
        let mut peers: Vec<_> = gradients.keys().cloned().collect();
        peers.sort();
        
        let n = peers.len();
        let our_idx = peers.iter().position(|p| p == &self.peer_id)
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
    async fn hierarchical_allreduce(&self, gradients: &HashMap<PeerId, Vec<f32>>) -> Result<Vec<f32>> {
        // Group nodes by region (simplified: by peer ID prefix)
        let mut regions: HashMap<u8, Vec<PeerId>> = HashMap::new();
        
        for peer in gradients.keys() {
            let region = peer.to_bytes()[0]; // Simple region assignment
            regions.entry(region).or_insert_with(Vec::new).push(peer.clone());
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
            if round_gradients.len() >= self.min_peers_for_aggregation {
                match self.algorithm {
                    AllReduceAlgorithm::Ring => Ok(Some(self.ring_allreduce(round_gradients).await?)),
                    AllReduceAlgorithm::Tree => Ok(Some(self.tree_allreduce(round_gradients).await?)),
                    AllReduceAlgorithm::Butterfly => Ok(Some(self.butterfly_allreduce(round_gradients).await?)),
                    AllReduceAlgorithm::Hierarchical => Ok(Some(self.hierarchical_allreduce(round_gradients).await?)),
                }
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
    
    if min == max {
        // All values are the same
        return Ok(vec![0; gradient.len() + 8]);
    }
    
    let scale = 255.0 / (max - min);
    let mut quantized = Vec::with_capacity(gradient.len() + 8);
    
    // Store min and max for dequantization
    quantized.extend_from_slice(&min.to_le_bytes());
    quantized.extend_from_slice(&max.to_le_bytes());
    
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