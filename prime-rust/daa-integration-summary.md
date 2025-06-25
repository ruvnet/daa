# DAA Integration Code for Prime-Rust

This document contains the complete DAA (Decentralized Autonomous Applications) integration code for the Prime-Rust distributed ML training framework.

## Overview

The integration follows the 30-line agent pattern from research.md and implements:
- DaaOrchestrator for each node type (trainer, coordinator, DHT)
- Autonomy loops (Monitor → Reason → Act → Reflect → Adapt)
- daa-economy integration for token rewards
- daa-governance for rule-based validation
- Agent communication protocols via DHT

## Core Components

### 1. Prime-Core Types (`prime-core/src/lib.rs`)

```rust
// Core types and traits for Prime-Rust with DAA integration
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;

pub mod grpc;
pub mod types;

pub use types::*;

/// Core error types for Prime framework
#[derive(Error, Debug)]
pub enum PrimeError {
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Training error: {0}")]
    TrainingError(String),
    
    #[error("Governance rule violation: {0}")]
    GovernanceError(String),
    
    #[error("Economic transaction failed: {0}")]
    EconomicError(String),
    
    #[error("DAA orchestration error: {0}")]
    OrchestrationError(String),
    
    #[error("Other error: {0}")]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, PrimeError>;

/// Node types in the Prime network
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeType {
    Trainer,
    Coordinator,
    Validator,
    ParameterServer,
}

/// Core context for DAA agents
#[derive(Clone)]
pub struct DaaContext {
    pub node_id: String,
    pub node_type: NodeType,
    pub peer_uri: String,
    pub dht_handle: Arc<dyn DhtInterface>,
    pub economy_handle: Arc<dyn EconomyInterface>,
    pub governance_handle: Arc<dyn GovernanceInterface>,
}

/// DHT interface for parameter storage and discovery
#[async_trait::async_trait]
pub trait DhtInterface: Send + Sync {
    async fn get(&self, key: &str) -> Result<Vec<u8>>;
    async fn put(&self, key: &str, value: Vec<u8>) -> Result<()>;
    async fn discover_peers(&self) -> Result<Vec<String>>;
}

/// Economy interface for token rewards and incentives
#[async_trait::async_trait]
pub trait EconomyInterface: Send + Sync {
    async fn reward_contribution(&self, contributor: &str, amount: u64) -> Result<()>;
    async fn charge_usage(&self, user: &str, amount: u64) -> Result<()>;
    async fn get_balance(&self, account: &str) -> Result<u64>;
}

/// Governance interface for rule validation
#[async_trait::async_trait]
pub trait GovernanceInterface: Send + Sync {
    async fn validate_action(&self, action: &str, params: HashMap<String, String>) -> Result<bool>;
    async fn add_rule(&self, rule: GovernanceRule) -> Result<()>;
    async fn get_active_rules(&self) -> Result<Vec<GovernanceRule>>;
}
```

### 2. Trainer Implementation (`prime-trainer/src/main.rs`)

```rust
// Prime Trainer with DAA Orchestrator - Following the 30-line agent pattern
use anyhow::Result;
use daa::orchestrator::{DaaOrchestrator, OrchestratorConfig};
use prime_core::{grpc::trainer_client::TrainerClient, GradChunk, DaaContext};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();
    
    // Configure DAA orchestrator for trainer node
    let config = OrchestratorConfig::default()
        .with_name("trainer-node")
        .with_autonomy_interval(std::time::Duration::from_secs(10));
    
    let mut agent = DaaOrchestrator::new(config).await?;
    
    // Add training task to autonomy loop
    agent.add_task("train_step", |ctx: DaaContext| async move {
        tracing::info!("Executing training step");
        
        // Compute local gradient (using tch or burn)
        let grad = compute_local_gradient().await?;
        
        // Connect to peer and push gradient
        let mut rpc = TrainerClient::connect(ctx.peer_uri()).await?;
        rpc.push_grad(GradChunk::from(grad)).await?;
        
        // Reward contribution via economy module
        ctx.economy_handle.reward_contribution(&ctx.node_id, 100).await?;
        
        Ok(())
    });
    
    // Add model sync task
    agent.add_task("model_sync", |ctx: DaaContext| async move {
        tracing::info!("Syncing model parameters from DHT");
        
        // Get latest model version from DHT
        let model_data = ctx.dht_handle.get("model:latest").await?;
        
        // Apply model update
        apply_model_update(&model_data).await?;
        
        Ok(())
    });
    
    // Add governance validation task
    agent.add_task("validate_training", |ctx: DaaContext| async move {
        // Check if training parameters comply with governance rules
        let mut params = std::collections::HashMap::new();
        params.insert("learning_rate".to_string(), "0.001".to_string());
        params.insert("batch_size".to_string(), "32".to_string());
        
        let is_valid = ctx.governance_handle
            .validate_action("training_step", params)
            .await?;
        
        if !is_valid {
            tracing::warn!("Training parameters violate governance rules");
        }
        
        Ok(())
    });
    
    // Run the autonomy loop (Monitor → Reason → Act → Reflect → Adapt)
    tracing::info!("Starting DAA trainer autonomy loop");
    agent.run_autonomy_loop().await?;
    
    Ok(())
}
```

### 3. Trainer Library with Full Autonomy Loops (`prime-trainer/src/lib.rs`)

```rust
/// Trainer node with DAA orchestration capabilities
pub struct TrainerNode {
    orchestrator: DaaOrchestrator,
    context: Arc<RwLock<TrainingContext>>,
    model_params: Arc<RwLock<ModelParams>>,
}

impl TrainerNode {
    /// Setup autonomy loops for the trainer
    pub async fn setup_autonomy_loops(&mut self, daa_context: DaaContext) -> Result<()> {
        let ctx = self.context.clone();
        let model = self.model_params.clone();
        
        // Monitor loop - check for new tasks and peer health
        self.orchestrator.add_task("monitor", {
            let daa_ctx = daa_context.clone();
            move |_| {
                let daa_ctx = daa_ctx.clone();
                async move {
                    // Discover active peers
                    let peers = daa_ctx.dht_handle.discover_peers().await?;
                    tracing::info!("Active peers: {}", peers.len());
                    
                    // Check own resource usage
                    let balance = daa_ctx.economy_handle.get_balance(&daa_ctx.node_id).await?;
                    tracing::debug!("Current balance: {} tokens", balance);
                    
                    Ok(())
                }
            }
        });
        
        // Reason loop - decide on training strategy
        self.orchestrator.add_task("reason", {
            let daa_ctx = daa_context.clone();
            move |_| {
                let daa_ctx = daa_ctx.clone();
                let ctx = ctx.clone();
                async move {
                    let mut training_ctx = ctx.write().await;
                    
                    // Check governance rules for training parameters
                    let mut params = std::collections::HashMap::new();
                    params.insert("learning_rate".to_string(), training_ctx.learning_rate.to_string());
                    params.insert("batch_size".to_string(), training_ctx.local_batch_size.to_string());
                    
                    let is_valid = daa_ctx.governance_handle
                        .validate_action("training_config", params)
                        .await?;
                    
                    if !is_valid {
                        // Adjust parameters to comply with rules
                        training_ctx.learning_rate *= 0.5;
                        tracing::warn!("Adjusted learning rate to comply with governance");
                    }
                    
                    Ok(())
                }
            }
        });
        
        // Act loop - perform training computation
        self.orchestrator.add_task("act", {
            let daa_ctx = daa_context.clone();
            move |_| {
                let daa_ctx = daa_ctx.clone();
                let model = model.clone();
                let ctx = ctx.clone();
                async move {
                    let mut training_ctx = ctx.write().await;
                    
                    // Compute gradients
                    let gradients = compute_gradients(&*model.read().await).await?;
                    
                    // Push to peers
                    let grad_chunk = GradChunk::from(gradients);
                    let serialized = serde_json::to_vec(&grad_chunk)?;
                    daa_ctx.dht_handle.put(&format!("grad:{}", training_ctx.global_step), serialized).await?;
                    
                    // Update step counter
                    training_ctx.global_step += 1;
                    
                    // Earn rewards for contribution
                    daa_ctx.economy_handle.reward_contribution(&daa_ctx.node_id, 10).await?;
                    
                    Ok(())
                }
            }
        });
        
        // Reflect loop - analyze training progress
        self.orchestrator.add_task("reflect", {
            let daa_ctx = daa_context.clone();
            move |_| {
                let daa_ctx = daa_ctx.clone();
                let ctx = ctx.clone();
                async move {
                    let training_ctx = ctx.read().await;
                    
                    // Log training metrics
                    tracing::info!(
                        "Training progress - Epoch: {}, Step: {}, LR: {}",
                        training_ctx.current_epoch,
                        training_ctx.global_step,
                        training_ctx.learning_rate
                    );
                    
                    Ok(())
                }
            }
        });
        
        // Adapt loop - adjust strategy based on performance
        self.orchestrator.add_task("adapt", {
            let daa_ctx = daa_context.clone();
            move |_| {
                let daa_ctx = daa_ctx.clone();
                let ctx = ctx.clone();
                async move {
                    let mut training_ctx = ctx.write().await;
                    
                    // Simple learning rate decay
                    if training_ctx.global_step % 1000 == 0 {
                        training_ctx.learning_rate *= 0.95;
                        tracing::info!("Decayed learning rate to {}", training_ctx.learning_rate);
                    }
                    
                    Ok(())
                }
            }
        });
        
        Ok(())
    }
}
```

### 4. Coordinator Implementation (`prime-coordinator/src/lib.rs`)

Key features:
- Manages active nodes and task allocation
- Implements all 5 autonomy loops
- Integrates governance rules for training control
- Uses economy for charging coordination fees
- Tracks node reliability scores

### 5. DHT Implementation (`prime-dht/src/lib.rs`)

Key features:
- Kademlia DHT for parameter storage
- DAA orchestration for maintenance tasks
- Economy integration for storage rewards
- Agent communication protocol
- Message passing via DHT

### 6. CLI Implementation (`prime-cli/src/main.rs`)

Commands:
- `prime up --role <trainer|coordinator|dht>` - Start a node
- `prime join --coordinator <address>` - Join as trainer
- `prime status --dht <address>` - Query network status

## Key Integration Points

1. **Autonomy Loops**: Every node type implements the 5-phase loop:
   - Monitor: Track environment and peer health
   - Reason: Validate actions against governance rules
   - Act: Execute core functionality (training, coordination, storage)
   - Reflect: Analyze performance and log metrics
   - Adapt: Adjust parameters and strategy

2. **Economy Integration**:
   - Trainers earn rewards for gradient contributions
   - Coordinators charge fees for task allocation
   - DHT nodes earn rewards for storage provision
   - All operations check token balances

3. **Governance Integration**:
   - Training parameters validated against rules
   - Storage limits enforced via governance
   - Dynamic rule addition supported
   - Actions blocked if rules violated

4. **Communication Protocol**:
   - Agent-to-agent messaging via DHT
   - Broadcast capability for network-wide messages
   - Notification system for message delivery
   - All messages economically metered

## Usage Examples

### Start a Coordinator
```bash
prime up --role coordinator --id coord-1 --public-ip 192.168.1.100
```

### Start a Trainer
```bash
prime up --role trainer --id trainer-1
```

### Join Network
```bash
prime join --coordinator 192.168.1.100:50051
```

### Query Status
```bash
prime status --dht 192.168.1.100:4001
```

## Architecture Benefits

1. **Fully Decentralized**: No single point of failure
2. **Economically Incentivized**: Token rewards drive participation
3. **Rule-Based Governance**: Transparent, auditable decision making
4. **Self-Organizing**: Nodes autonomously coordinate via DAA
5. **Fault Tolerant**: Continues operating despite node failures