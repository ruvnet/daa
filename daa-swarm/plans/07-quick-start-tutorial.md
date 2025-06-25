# DAA Swarm Quick Start Tutorial

## Welcome to DAA Swarm! 🚀

This tutorial will get you running a distributed AI swarm with GPU acceleration in under 30 minutes. By the end, you'll have:
- A local swarm node running with GPU support
- Connected to the DAA test network
- Executed your first distributed inference
- Monitored swarm performance in real-time

## Prerequisites

### System Requirements
- Ubuntu 22.04 or macOS (with Docker)
- NVIDIA GPU (GTX 1080 or better) or Apple Silicon
- 16GB RAM minimum
- 50GB free disk space
- Docker installed
- Rust 1.75+ (we'll install if needed)

### Quick Check
```bash
# Check GPU availability
nvidia-smi  # For NVIDIA GPUs

# Check Docker
docker --version

# Check Rust (optional)
rustc --version
```

## 1. Quick Install (5 minutes)

### Option A: Docker Quick Start (Recommended)
```bash
# Clone the repository
git clone https://github.com/daa-network/daa-swarm.git
cd daa-swarm

# Pull pre-built Docker image
docker pull daanetwork/swarm-node:latest

# Run with GPU support
docker run --gpus all -p 50051:50051 -p 8080:8080 \
  -v $(pwd)/data:/data \
  --name daa-swarm-node \
  daanetwork/swarm-node:latest
```

### Option B: Build from Source
```bash
# Install Rust if needed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Clone and build
git clone https://github.com/daa-network/daa-swarm.git
cd daa-swarm
cargo build --release --features gpu

# Run the node
./target/release/daa-swarm serve
```

## 2. First Swarm Connection (2 minutes)

### Start Your Node
```bash
# Using Docker
docker exec -it daa-swarm-node daa-swarm init

# Or using binary
./daa-swarm init
```

You'll see:
```
🚀 DAA Swarm Node v1.0.0
📍 Node ID: 12D3KooWRkYhHnMZ...
🌐 Listening on: /ip4/0.0.0.0/tcp/50051
🔗 Bootstrap complete! Connected to 3 peers
✅ Node ready for swarm operations
```

### Join the Test Swarm
```bash
# Connect to DAA testnet
./daa-swarm join testnet

# Or join a specific swarm
./daa-swarm join --swarm-id daa-ml-research
```

## 3. Your First Expert (5 minutes)

### Deploy a Pre-trained Expert
```bash
# List available experts
./daa-swarm expert list --available

# Deploy a language expert
./daa-swarm expert deploy language-expert-small \
  --gpu-memory 4GB \
  --specialization "code-generation"
```

### Create a Custom Expert
```rust
// my_expert.rs
use daa_swarm::prelude::*;

#[derive(Expert)]
pub struct MyCodeExpert {
    model: TransformerModel,
    specialization: Specialization,
}

impl MyCodeExpert {
    pub fn new() -> Self {
        Self {
            model: TransformerModel::load("models/code-expert-v1"),
            specialization: Specialization::CodeGeneration {
                languages: vec!["rust", "python", "typescript"],
            },
        }
    }
}

// Register with swarm
#[swarm_main]
async fn main() -> Result<()> {
    let expert = MyCodeExpert::new();
    SwarmNode::new()
        .register_expert(expert)
        .join_swarm("daa-ml-research")
        .serve()
        .await
}
```

## 4. Distributed Inference (3 minutes)

### Command Line Interface
```bash
# Simple inference request
./daa-swarm infer "Explain quantum computing in simple terms"

# Code generation with specific expert
./daa-swarm infer \
  --expert-type code-generation \
  --prompt "Write a Rust function to calculate fibonacci numbers"

# Multi-expert ensemble
./daa-swarm infer \
  --mode ensemble \
  --experts 3 \
  --prompt "Design a distributed cache system"
```

### Python Client
```python
from daa_swarm import SwarmClient

# Connect to local node
client = SwarmClient("localhost:50051")

# Simple inference
response = client.infer(
    "What are the key principles of distributed systems?",
    num_experts=3,
    consensus_mode="weighted_vote"
)

print(f"Response: {response.text}")
print(f"Consensus confidence: {response.confidence}")
print(f"Experts used: {response.expert_ids}")
```

### Rust Client
```rust
use daa_swarm::client::SwarmClient;

#[tokio::main]
async fn main() -> Result<()> {
    // Connect to swarm
    let client = SwarmClient::connect("http://localhost:50051").await?;
    
    // Request inference with multiple experts
    let response = client
        .infer("Implement a binary search tree in Rust")
        .with_experts(3)
        .with_consensus_mode(ConsensusMode::WeightedVote)
        .execute()
        .await?;
    
    println!("Response: {}", response.text());
    println!("Latency: {:?}", response.latency());
    
    Ok(())
}
```

## 5. Real-time Monitoring (2 minutes)

### Web Dashboard
```bash
# Open monitoring dashboard
open http://localhost:8080

# Or use CLI monitoring
./daa-swarm monitor --real-time
```

### Metrics to Watch
```yaml
Key Metrics:
- Active Experts: Number of experts in your swarm
- GPU Utilization: Should be 60-80% for optimal performance
- Network Latency: P95 should be < 500ms
- Consensus Time: Time to reach expert agreement
- Cache Hit Rate: Higher is better (aim for > 70%)
```

### Grafana Dashboard (Optional)
```bash
# Start Grafana with pre-configured dashboards
docker-compose up -d grafana

# Access at http://localhost:3000
# Default login: admin/admin
```

## 6. Common Patterns

### Pattern 1: Local Expert Ensemble
```bash
# Start multiple experts on one GPU
./daa-swarm expert deploy ensemble \
  --experts "language-small,code-small,math-small" \
  --gpu-memory-split "2GB,2GB,2GB"
```

### Pattern 2: Specialized Swarm
```bash
# Create a code-review swarm
./daa-swarm swarm create code-review \
  --required-experts "code-analysis,security-audit,style-check" \
  --min-consensus 2
```

### Pattern 3: Federated Learning
```bash
# Join federated training
./daa-swarm train join \
  --federation "daa-fl-experiment-1" \
  --local-data "./data/code_samples" \
  --privacy-budget 1.0
```

## 7. Production Deployment

### Deploy to Fly.io
```bash
# Install Fly CLI
curl -L https://fly.io/install.sh | sh

# Login to Fly
fly auth login

# Deploy your swarm node
fly launch --config fly.toml
fly deploy

# Scale to multiple regions
fly scale count 3 --region ord,iad,sjc
```

### fly.toml Configuration
```toml
app = "my-daa-swarm"
primary_region = "ord"

[build]
image = "daanetwork/swarm-node:latest"

[experimental]
auto_stop_machines = false

[[vm]]
size = "a10"  # or "a100-40gb" for production
memory = "32gb"

[services]
  [[services.ports]]
    port = 443
    handlers = ["tls", "http"]
  [[services.ports]]
    port = 50051
    handlers = ["tls"]

[[mounts]]
source = "models"
destination = "/data/models"
```

## 8. Troubleshooting

### Common Issues

#### GPU Not Detected
```bash
# Check CUDA installation
nvidia-smi
ldconfig -p | grep cuda

# For Docker
docker run --gpus all nvidia/cuda:12.2-base nvidia-smi
```

#### Connection Issues
```bash
# Check connectivity
./daa-swarm peers list
./daa-swarm network diagnose

# Reset and rejoin
./daa-swarm reset
./daa-swarm join testnet --bootstrap
```

#### Performance Issues
```bash
# Run diagnostics
./daa-swarm diagnose performance

# Optimize settings
./daa-swarm config set \
  --gpu-batch-size 32 \
  --cache-size 10GB \
  --max-concurrent-requests 100
```

## 9. Advanced Features

### Custom Routing
```rust
// Implement custom expert routing
use daa_swarm::routing::*;

struct PriorityRouter {
    priority_map: HashMap<ExpertType, f32>,
}

impl Router for PriorityRouter {
    fn route(&self, request: &Request) -> Vec<(ExpertId, f32)> {
        // Custom routing logic
        self.select_by_priority(request)
    }
}
```

### Privacy-Preserving Inference
```bash
# Enable differential privacy
./daa-swarm config privacy \
  --epsilon 1.0 \
  --delta 1e-5 \
  --noise-multiplier 1.1
```

### Multi-Modal Experts
```python
# Deploy vision + language expert
client.deploy_expert(
    name="multimodal-expert",
    model_configs=[
        {"type": "vision", "model": "clip-large"},
        {"type": "language", "model": "gpt-small"},
    ],
    fusion_strategy="cross_attention"
)
```

## 10. Next Steps

### Join the Community
- Discord: https://discord.gg/daa-network
- GitHub: https://github.com/daa-network/daa-swarm
- Documentation: https://docs.daa.network

### Contribute
1. **Run a Node**: Help decentralize the network
2. **Train Experts**: Contribute specialized models
3. **Build Apps**: Create applications using the swarm
4. **Improve Core**: Contribute to the codebase

### Learn More
- [Architecture Deep Dive](./05-integrated-architecture.md)
- [Parallel Implementation Guide](./06-parallel-implementation-guide.md)
- [API Reference](https://docs.daa.network/api)
- [Expert Training Guide](https://docs.daa.network/training)

## Congratulations! 🎉

You're now running a distributed AI swarm node! Your node is contributing to a decentralized network of AI experts, enabling:
- Collaborative intelligence through expert consensus
- Privacy-preserving distributed computation  
- Incentivized participation in AI research
- Democratic access to advanced AI capabilities

Welcome to the future of distributed AI! 🚀

---

## Quick Reference Card

```bash
# Essential Commands
daa-swarm init                    # Initialize node
daa-swarm join testnet           # Join test network
daa-swarm expert list            # List experts
daa-swarm infer "prompt"         # Run inference
daa-swarm monitor                # View metrics
daa-swarm help                   # Get help

# Configuration
daa-swarm config get             # View config
daa-swarm config set KEY VALUE   # Update config

# Debugging
daa-swarm logs -f                # Follow logs
daa-swarm diagnose               # Run diagnostics
daa-swarm peers list             # List connected peers
```

---

*Built with ❤️ by the DAA Network community*