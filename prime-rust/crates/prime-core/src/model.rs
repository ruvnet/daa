//! Model structures and serialization for DiLoCo

use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tch::{nn, Device, Tensor};
use sha2::{Sha256, Digest};

/// Model parameter with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelParameter {
    /// Parameter name (e.g., "layer1.weight")
    pub name: String,
    
    /// Tensor shape
    pub shape: Vec<i64>,
    
    /// Serialized tensor data
    pub data: Vec<u8>,
    
    /// Data type
    pub dtype: DataType,
    
    /// Whether this parameter is trainable
    pub trainable: bool,
    
    /// Parameter statistics
    pub stats: Option<ParameterStats>,
}

/// Parameter statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterStats {
    /// Mean value
    pub mean: f32,
    
    /// Standard deviation
    pub std: f32,
    
    /// Minimum value
    pub min: f32,
    
    /// Maximum value
    pub max: f32,
    
    /// L2 norm
    pub norm: f32,
}

/// Data types for tensors
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataType {
    Float32,
    Float16,
    BFloat16,
    Int64,
    Int32,
    Int8,
}

impl DataType {
    /// Convert to tch::Kind
    pub fn to_kind(&self) -> tch::Kind {
        match self {
            DataType::Float32 => tch::Kind::Float,
            DataType::Float16 => tch::Kind::Half,
            DataType::BFloat16 => tch::Kind::BFloat16,
            DataType::Int64 => tch::Kind::Int64,
            DataType::Int32 => tch::Kind::Int,
            DataType::Int8 => tch::Kind::Int8,
        }
    }
    
    /// Create from tch::Kind
    pub fn from_kind(kind: tch::Kind) -> Result<Self> {
        match kind {
            tch::Kind::Float => Ok(DataType::Float32),
            tch::Kind::Half => Ok(DataType::Float16),
            tch::Kind::BFloat16 => Ok(DataType::BFloat16),
            tch::Kind::Int64 => Ok(DataType::Int64),
            tch::Kind::Int => Ok(DataType::Int32),
            tch::Kind::Int8 => Ok(DataType::Int8),
            _ => Err(Error::Model(format!("Unsupported tensor kind: {:?}", kind))),
        }
    }
}

/// Complete model state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelState {
    /// Model architecture identifier
    pub architecture: String,
    
    /// Model version (hash of parameters)
    pub version: String,
    
    /// All model parameters
    pub parameters: Vec<ModelParameter>,
    
    /// Model metadata
    pub metadata: ModelMetadata,
}

impl ModelState {
    /// Create a new model state
    pub fn new(architecture: String) -> Self {
        Self {
            architecture,
            version: String::new(),
            parameters: Vec::new(),
            metadata: ModelMetadata::default(),
        }
    }
    
    /// Add a parameter
    pub fn add_parameter(&mut self, param: ModelParameter) {
        self.metadata.total_parameters += param.shape.iter().product::<i64>();
        if param.trainable {
            self.metadata.trainable_parameters += param.shape.iter().product::<i64>();
        }
        self.metadata.size_bytes += param.data.len() as i64;
        
        self.parameters.push(param);
    }
    
    /// Calculate and update version hash
    pub fn update_version(&mut self) {
        let mut hasher = Sha256::new();
        
        // Hash architecture
        hasher.update(self.architecture.as_bytes());
        
        // Hash each parameter
        for param in &self.parameters {
            hasher.update(param.name.as_bytes());
            hasher.update(&param.shape.iter().map(|&x| x.to_le_bytes()).flatten().collect::<Vec<_>>());
            hasher.update(&param.data);
        }
        
        self.version = hex::encode(hasher.finalize());
        self.metadata.modified_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }
    
    /// Get parameter by name
    pub fn get_parameter(&self, name: &str) -> Option<&ModelParameter> {
        self.parameters.iter().find(|p| p.name == name)
    }
    
    /// Get mutable parameter by name
    pub fn get_parameter_mut(&mut self, name: &str) -> Option<&mut ModelParameter> {
        self.parameters.iter_mut().find(|p| p.name == name)
    }
    
    /// Create a parameter map
    pub fn parameter_map(&self) -> HashMap<String, &ModelParameter> {
        self.parameters.iter().map(|p| (p.name.clone(), p)).collect()
    }
}

/// Model metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    /// Total number of parameters
    pub total_parameters: i64,
    
    /// Number of trainable parameters
    pub trainable_parameters: i64,
    
    /// Model size in bytes
    pub size_bytes: i64,
    
    /// Creation timestamp
    pub created_at: u64,
    
    /// Last modified timestamp
    pub modified_at: u64,
    
    /// Training configuration
    pub training_config: HashMap<String, String>,
}

impl Default for ModelMetadata {
    fn default() -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
            
        Self {
            total_parameters: 0,
            trainable_parameters: 0,
            size_bytes: 0,
            created_at: now,
            modified_at: now,
            training_config: HashMap::new(),
        }
    }
}

/// Model delta for incremental updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelDelta {
    /// Base model version
    pub base_version: String,
    
    /// New model version after applying delta
    pub new_version: String,
    
    /// Parameter updates
    pub deltas: Vec<ParameterDelta>,
    
    /// Global training step
    pub global_step: u64,
}

impl ModelDelta {
    /// Apply delta to model state
    pub fn apply_to(&self, model: &mut ModelState) -> Result<()> {
        if model.version != self.base_version {
            return Err(Error::Model(format!(
                "Version mismatch: model has {}, delta expects {}",
                model.version, self.base_version
            )));
        }
        
        for delta in &self.deltas {
            if let Some(param) = model.get_parameter_mut(&delta.name) {
                delta.apply_to_parameter(param)?;
            } else {
                return Err(Error::Model(format!(
                    "Parameter not found: {}",
                    delta.name
                )));
            }
        }
        
        model.update_version();
        Ok(())
    }
}

/// Parameter delta
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterDelta {
    /// Parameter name
    pub name: String,
    
    /// Delta values
    pub delta_data: Vec<u8>,
    
    /// Sparse indices (optional)
    pub sparse_indices: Option<Vec<i64>>,
}

impl ParameterDelta {
    /// Apply delta to parameter
    pub fn apply_to_parameter(&self, param: &mut ModelParameter) -> Result<()> {
        // Deserialize current parameter
        let tensor = deserialize_tensor(&param.data, param.dtype.to_kind(), &param.shape)?;
        
        // Deserialize delta
        let delta = deserialize_tensor(&self.delta_data, param.dtype.to_kind(), &param.shape)?;
        
        // Apply delta
        let updated = if let Some(indices) = &self.sparse_indices {
            // Sparse update
            apply_sparse_delta(&tensor, &delta, indices)?
        } else {
            // Dense update
            tensor + delta
        };
        
        // Serialize back
        param.data = serialize_tensor(&updated)?;
        
        Ok(())
    }
}

/// High-level model wrapper
pub struct Model {
    /// PyTorch variable store
    pub vs: nn::VarStore,
    
    /// Model architecture name
    pub architecture: String,
    
    /// Device
    pub device: Device,
}

impl Model {
    /// Create a new model
    pub fn new(architecture: String, device: Device) -> Self {
        Self {
            vs: nn::VarStore::new(device),
            architecture,
            device,
        }
    }
    
    /// Export model state
    pub fn export_state(&self) -> Result<ModelState> {
        let mut state = ModelState::new(self.architecture.clone());
        
        // Export all variables
        for (name, tensor) in self.vs.variables() {
            let param = export_parameter(name, &tensor)?;
            state.add_parameter(param);
        }
        
        state.update_version();
        Ok(state)
    }
    
    /// Import model state
    pub fn import_state(&mut self, state: &ModelState) -> Result<()> {
        // Verify architecture
        if self.architecture != state.architecture {
            return Err(Error::Model(format!(
                "Architecture mismatch: expected {}, got {}",
                self.architecture, state.architecture
            )));
        }
        
        // Import parameters
        for param in &state.parameters {
            let tensor = deserialize_tensor(&param.data, param.dtype.to_kind(), &param.shape)?;
            self.vs.root().var(&param.name, &param.shape, |_| tensor);
        }
        
        Ok(())
    }
    
    /// Calculate model delta from another state
    pub fn calculate_delta(&self, base_state: &ModelState) -> Result<ModelDelta> {
        let current_state = self.export_state()?;
        
        let mut delta = ModelDelta {
            base_version: base_state.version.clone(),
            new_version: current_state.version.clone(),
            deltas: Vec::new(),
            global_step: 0, // To be set by caller
        };
        
        // Calculate parameter deltas
        for param in &current_state.parameters {
            if let Some(base_param) = base_state.get_parameter(&param.name) {
                let current = deserialize_tensor(&param.data, param.dtype.to_kind(), &param.shape)?;
                let base = deserialize_tensor(&base_param.data, base_param.dtype.to_kind(), &base_param.shape)?;
                
                let diff = current - base;
                let delta_data = serialize_tensor(&diff)?;
                
                delta.deltas.push(ParameterDelta {
                    name: param.name.clone(),
                    delta_data,
                    sparse_indices: None,
                });
            }
        }
        
        Ok(delta)
    }
}

/// Export a parameter from tensor
fn export_parameter(name: &str, tensor: &Tensor) -> Result<ModelParameter> {
    let shape = tensor.size();
    let dtype = DataType::from_kind(tensor.kind())?;
    let data = serialize_tensor(tensor)?;
    
    // Calculate statistics
    let stats = Some(ParameterStats {
        mean: tensor.mean(tch::Kind::Float).double_value(&[]) as f32,
        std: tensor.std(false).double_value(&[]) as f32,
        min: tensor.min().double_value(&[]) as f32,
        max: tensor.max().double_value(&[]) as f32,
        norm: tensor.norm().double_value(&[]) as f32,
    });
    
    Ok(ModelParameter {
        name: name.to_string(),
        shape,
        data,
        dtype,
        trainable: true, // Assume all exported parameters are trainable
        stats,
    })
}

/// Serialize tensor to bytes
fn serialize_tensor(tensor: &Tensor) -> Result<Vec<u8>> {
    // Ensure tensor is contiguous
    let tensor = tensor.contiguous();
    
    // Get size information
    let numel = tensor.numel() as usize;
    let element_size = tensor.element_size();
    let total_size = numel * element_size;
    
    // Create byte vector
    let mut bytes = vec![0u8; total_size];
    
    // Copy data
    unsafe {
        let src = tensor.data_ptr() as *const u8;
        std::ptr::copy_nonoverlapping(src, bytes.as_mut_ptr(), total_size);
    }
    
    Ok(bytes)
}

/// Deserialize tensor from bytes
fn deserialize_tensor(bytes: &[u8], kind: tch::Kind, shape: &[i64]) -> Result<Tensor> {
    // Calculate expected size
    let numel: i64 = shape.iter().product();
    let element_size = kind.element_size();
    let expected_size = (numel as usize) * element_size;
    
    if bytes.len() != expected_size {
        return Err(Error::Model(format!(
            "Size mismatch: expected {} bytes, got {}",
            expected_size,
            bytes.len()
        )));
    }
    
    // Create tensor from bytes
    let tensor = unsafe {
        Tensor::from_blob(
            bytes.as_ptr() as *const std::ffi::c_void,
            shape,
            &[1; 7], // Dummy strides
            kind,
            Device::Cpu,
        )
    };
    
    Ok(tensor.contiguous())
}

/// Apply sparse delta to tensor
fn apply_sparse_delta(tensor: &Tensor, delta: &Tensor, indices: &[i64]) -> Result<Tensor> {
    // For now, just do dense update
    // TODO: Implement proper sparse update
    Ok(tensor + delta)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_model_state_serialization() {
        let mut state = ModelState::new("test_model".to_string());
        
        // Add some parameters
        let tensor = Tensor::randn(&[10, 20], (tch::Kind::Float, Device::Cpu));
        let param = export_parameter("layer1.weight", &tensor).unwrap();
        state.add_parameter(param);
        
        state.update_version();
        
        assert!(!state.version.is_empty());
        assert_eq!(state.parameters.len(), 1);
        assert_eq!(state.metadata.total_parameters, 200);
    }
    
    #[test]
    fn test_model_delta() {
        let device = Device::Cpu;
        
        // Create two models
        let mut model1 = Model::new("test_arch".to_string(), device);
        let mut model2 = Model::new("test_arch".to_string(), device);
        
        // Add parameters
        model1.vs.root().var("weight", &[5, 5], |t| t.randn_standard());
        model2.vs.root().var("weight", &[5, 5], |t| t.randn_standard());
        
        // Export states
        let state1 = model1.export_state().unwrap();
        let state2 = model2.export_state().unwrap();
        
        // Calculate delta
        let delta = model2.calculate_delta(&state1).unwrap();
        
        assert_eq!(delta.base_version, state1.version);
        assert_eq!(delta.new_version, state2.version);
        assert_eq!(delta.deltas.len(), 1);
    }
}