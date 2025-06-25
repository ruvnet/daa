//! Model integrity verification using post-quantum signatures and merkle trees

use super::{SecureIdentity, SecurityError};
use qudag_crypto::{
    ml_dsa::{MlDsa, MlDsaPublicKey},
    hash::HashFunction,
    fingerprint::Fingerprint,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Model checkpoint with integrity guarantees
#[derive(Clone, Serialize, Deserialize)]
pub struct ModelCheckpoint {
    /// Model version
    pub version: u64,
    
    /// Model parameters hash
    pub parameters_hash: Vec<u8>,
    
    /// Training metadata
    pub metadata: ModelMetadata,
    
    /// Merkle root of model layers
    pub merkle_root: Vec<u8>,
    
    /// Post-quantum signature
    pub signature: Vec<u8>,
    
    /// Signer's fingerprint
    pub signer_fingerprint: Fingerprint,
}

/// Model metadata for verification
#[derive(Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    /// Training round
    pub round: u64,
    
    /// Number of samples
    pub num_samples: usize,
    
    /// Loss value
    pub loss: f64,
    
    /// Accuracy metrics
    pub accuracy: f64,
    
    /// Privacy budget used
    pub privacy_budget_used: f64,
    
    /// Contributing participants
    pub contributors: Vec<Fingerprint>,
}

/// Merkle tree for model layer verification
pub struct ModelMerkleTree {
    /// Layer hashes
    pub layers: Vec<LayerHash>,
    
    /// Internal nodes
    nodes: Vec<Vec<u8>>,
    
    /// Tree height
    height: usize,
}

/// Hash of a model layer
#[derive(Clone, Serialize, Deserialize)]
pub struct LayerHash {
    /// Layer index
    pub index: usize,
    
    /// Layer name
    pub name: String,
    
    /// Parameter count
    pub param_count: usize,
    
    /// Layer hash
    pub hash: Vec<u8>,
}

impl ModelMerkleTree {
    /// Build merkle tree from model layers
    pub fn build(layers: Vec<LayerHash>) -> Result<Self, SecurityError> {
        if layers.is_empty() {
            return Err(SecurityError::VerificationError(
                "Cannot build tree from empty layers".to_string(),
            ));
        }
        
        let n = layers.len();
        let height = (n as f64).log2().ceil() as usize;
        let padded_size = 2_usize.pow(height as u32);
        
        // Initialize nodes with layer hashes
        let mut nodes = vec![vec![0u8; 32]; 2 * padded_size - 1];
        
        // Copy layer hashes to leaves
        for (i, layer) in layers.iter().enumerate() {
            nodes[padded_size - 1 + i] = layer.hash.clone();
        }
        
        // Build tree bottom-up
        for i in (0..padded_size - 1).rev() {
            let left_child = &nodes[2 * i + 1];
            let right_child = &nodes[2 * i + 2];
            
            let mut combined = Vec::new();
            combined.extend_from_slice(left_child);
            combined.extend_from_slice(right_child);
            
            nodes[i] = HashFunction::hash(&combined);
        }
        
        Ok(Self {
            layers,
            nodes,
            height,
        })
    }
    
    /// Get merkle root
    pub fn root(&self) -> Vec<u8> {
        self.nodes[0].clone()
    }
    
    /// Generate merkle proof for a layer
    pub fn generate_proof(&self, layer_index: usize) -> Result<MerkleProof, SecurityError> {
        if layer_index >= self.layers.len() {
            return Err(SecurityError::VerificationError(
                "Layer index out of bounds".to_string(),
            ));
        }
        
        let mut proof_path = Vec::new();
        let padded_size = 2_usize.pow(self.height as u32);
        let mut index = padded_size - 1 + layer_index;
        
        while index > 0 {
            let sibling_index = if index % 2 == 1 { index + 1 } else { index - 1 };
            proof_path.push(ProofNode {
                hash: self.nodes[sibling_index].clone(),
                is_right: index % 2 == 1,
            });
            index = (index - 1) / 2;
        }
        
        Ok(MerkleProof {
            layer_index,
            layer_hash: self.layers[layer_index].hash.clone(),
            proof_path,
        })
    }
}

/// Merkle proof for layer verification
#[derive(Clone, Serialize, Deserialize)]
pub struct MerkleProof {
    /// Index of the layer
    pub layer_index: usize,
    
    /// Hash of the layer
    pub layer_hash: Vec<u8>,
    
    /// Proof path
    pub proof_path: Vec<ProofNode>,
}

/// Node in merkle proof path
#[derive(Clone, Serialize, Deserialize)]
pub struct ProofNode {
    /// Hash value
    pub hash: Vec<u8>,
    
    /// Whether this is a right sibling
    pub is_right: bool,
}

impl MerkleProof {
    /// Verify proof against merkle root
    pub fn verify(&self, root: &[u8]) -> bool {
        let mut current_hash = self.layer_hash.clone();
        
        for node in &self.proof_path {
            let mut combined = Vec::new();
            
            if node.is_right {
                combined.extend_from_slice(&current_hash);
                combined.extend_from_slice(&node.hash);
            } else {
                combined.extend_from_slice(&node.hash);
                combined.extend_from_slice(&current_hash);
            }
            
            current_hash = HashFunction::hash(&combined);
        }
        
        current_hash == root
    }
}

/// Model integrity verifier
pub struct ModelIntegrityVerifier {
    /// Trusted checkpoints
    checkpoints: HashMap<u64, ModelCheckpoint>,
    
    /// Trusted signers
    trusted_signers: HashMap<Fingerprint, MlDsaPublicKey>,
}

impl ModelIntegrityVerifier {
    /// Create a new verifier
    pub fn new() -> Self {
        Self {
            checkpoints: HashMap::new(),
            trusted_signers: HashMap::new(),
        }
    }
    
    /// Add a trusted signer
    pub fn add_trusted_signer(
        &mut self,
        fingerprint: Fingerprint,
        public_key: MlDsaPublicKey,
    ) {
        self.trusted_signers.insert(fingerprint, public_key);
    }
    
    /// Create and sign a model checkpoint
    pub fn create_checkpoint(
        &mut self,
        identity: &SecureIdentity,
        model_data: &[u8],
        metadata: ModelMetadata,
        layer_hashes: Vec<LayerHash>,
    ) -> Result<ModelCheckpoint, SecurityError> {
        // Build merkle tree
        let merkle_tree = ModelMerkleTree::build(layer_hashes)?;
        let merkle_root = merkle_tree.root();
        
        // Hash model parameters
        let parameters_hash = HashFunction::hash(model_data);
        
        // Create checkpoint
        let version = self.checkpoints.len() as u64 + 1;
        let mut checkpoint = ModelCheckpoint {
            version,
            parameters_hash,
            metadata,
            merkle_root,
            signature: vec![],
            signer_fingerprint: identity.fingerprint.clone(),
        };
        
        // Sign checkpoint
        let checkpoint_data = bincode::serialize(&checkpoint)
            .map_err(|e| SecurityError::VerificationError(e.to_string()))?;
        
        checkpoint.signature = identity.sign(&checkpoint_data)?;
        
        // Store checkpoint
        self.checkpoints.insert(version, checkpoint.clone());
        
        Ok(checkpoint)
    }
    
    /// Verify a model checkpoint
    pub fn verify_checkpoint(
        &self,
        checkpoint: &ModelCheckpoint,
    ) -> Result<bool, SecurityError> {
        // Get trusted signer
        let public_key = self.trusted_signers
            .get(&checkpoint.signer_fingerprint)
            .ok_or_else(|| SecurityError::VerificationError(
                "Unknown signer".to_string()
            ))?;
        
        // Prepare data for verification (without signature)
        let mut checkpoint_copy = checkpoint.clone();
        checkpoint_copy.signature = vec![];
        
        let checkpoint_data = bincode::serialize(&checkpoint_copy)
            .map_err(|e| SecurityError::VerificationError(e.to_string()))?;
        
        // Verify signature
        SecureIdentity::verify(public_key, &checkpoint_data, &checkpoint.signature)
    }
    
    /// Verify model layer integrity
    pub fn verify_layer(
        &self,
        layer_data: &[u8],
        proof: &MerkleProof,
        checkpoint: &ModelCheckpoint,
    ) -> Result<bool, SecurityError> {
        // Verify checkpoint first
        if !self.verify_checkpoint(checkpoint)? {
            return Ok(false);
        }
        
        // Hash layer data
        let layer_hash = HashFunction::hash(layer_data);
        
        // Verify hash matches proof
        if layer_hash != proof.layer_hash {
            return Ok(false);
        }
        
        // Verify merkle proof
        Ok(proof.verify(&checkpoint.merkle_root))
    }
}

/// Attestation for remote model verification
#[derive(Clone, Serialize, Deserialize)]
pub struct ModelAttestation {
    /// Checkpoint being attested
    pub checkpoint: ModelCheckpoint,
    
    /// Hardware attestation (if available)
    pub hardware_attestation: Option<Vec<u8>>,
    
    /// Attestor identity
    pub attestor: Fingerprint,
    
    /// Attestation signature
    pub signature: Vec<u8>,
    
    /// Timestamp
    pub timestamp: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_merkle_tree() {
        let layers = vec![
            LayerHash {
                index: 0,
                name: "input".to_string(),
                param_count: 100,
                hash: vec![1; 32],
            },
            LayerHash {
                index: 1,
                name: "hidden".to_string(),
                param_count: 200,
                hash: vec![2; 32],
            },
            LayerHash {
                index: 2,
                name: "output".to_string(),
                param_count: 10,
                hash: vec![3; 32],
            },
        ];
        
        let tree = ModelMerkleTree::build(layers).unwrap();
        let root = tree.root();
        
        // Generate and verify proof
        let proof = tree.generate_proof(1).unwrap();
        assert!(proof.verify(&root));
    }
    
    #[test]
    fn test_model_checkpoint() {
        let mut verifier = ModelIntegrityVerifier::new();
        let identity = SecureIdentity::new(1000).unwrap();
        
        // Add as trusted signer
        verifier.add_trusted_signer(
            identity.fingerprint.clone(),
            identity.signing_keys.public_key.clone(),
        );
        
        // Create checkpoint
        let metadata = ModelMetadata {
            round: 1,
            num_samples: 1000,
            loss: 0.1,
            accuracy: 0.95,
            privacy_budget_used: 0.5,
            contributors: vec![identity.fingerprint.clone()],
        };
        
        let layers = vec![LayerHash {
            index: 0,
            name: "test".to_string(),
            param_count: 100,
            hash: vec![1; 32],
        }];
        
        let checkpoint = verifier
            .create_checkpoint(&identity, b"model data", metadata, layers)
            .unwrap();
        
        // Verify checkpoint
        assert!(verifier.verify_checkpoint(&checkpoint).unwrap());
    }
}