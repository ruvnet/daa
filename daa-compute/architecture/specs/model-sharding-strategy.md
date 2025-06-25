# Model Sharding Strategy

## Overview

This document defines the model sharding strategy for distributed training across heterogeneous nodes in the DAA-Compute system. The strategy enables efficient training of large models that exceed single-node memory capacity while optimizing for the diverse capabilities of cloud, edge, and browser nodes.

## Sharding Architecture

### Core Sharding Types

```rust
pub enum ShardingStrategy {
    /// Data Parallel - Each node has full model, different data
    DataParallel {
        replication_factor: u32,
        gradient_accumulation_steps: u32,
    },
    
    /// Model Parallel - Model split across nodes
    ModelParallel {
        partition_strategy: PartitionStrategy,
        pipeline_stages: Option<u32>,
    },
    
    /// Hybrid Parallel - Combination of data and model parallel
    HybridParallel {
        model_parallel_size: u32,
        data_parallel_size: u32,
        partition_dims: Vec<Dimension>,
    },
    
    /// Expert Parallel - For mixture of experts models
    ExpertParallel {
        num_experts: u32,
        expert_capacity: u32,
        routing_strategy: RoutingStrategy,
    },
}

pub struct ModelShard {
    pub shard_id: ShardId,
    pub shard_type: ShardType,
    pub layers: Vec<LayerId>,
    pub parameters: ParameterMap,
    pub size_bytes: u64,
    pub compute_requirements: ComputeRequirements,
    pub dependencies: Vec<ShardId>,
}
```

### Partition Strategies

```rust
pub enum PartitionStrategy {
    /// Horizontal partitioning by layers
    LayerWise {
        layers_per_shard: u32,
        overlap_layers: u32,
    },
    
    /// Vertical partitioning within layers
    TensorWise {
        split_dimensions: Vec<Dimension>,
        min_tensor_size: u64,
    },
    
    /// Pipeline partitioning for sequential execution
    Pipeline {
        num_stages: u32,
        recomputation: bool,
    },
    
    /// Automatic partitioning based on profiling
    Automatic {
        target_memory_per_shard: u64,
        balance_computation: bool,
        minimize_communication: bool,
    },
}
```

## Sharding Algorithm

### Model Analysis

```rust
pub struct ModelAnalyzer {
    pub model_graph: ComputationGraph,
    pub parameter_stats: ParameterStatistics,
    pub memory_profiler: MemoryProfiler,
    pub compute_profiler: ComputeProfiler,
}

impl ModelAnalyzer {
    pub async fn analyze_model(&mut self, model: &Model) -> Result<ModelAnalysis> {
        // Build computation graph
        let graph = self.build_computation_graph(model)?;
        
        // Analyze memory requirements
        let memory_profile = self.profile_memory_usage(&graph)?;
        
        // Analyze compute requirements
        let compute_profile = self.profile_compute_requirements(&graph)?;
        
        // Identify communication patterns
        let comm_patterns = self.analyze_communication_patterns(&graph)?;
        
        Ok(ModelAnalysis {
            total_parameters: self.count_parameters(model),
            memory_footprint: memory_profile,
            compute_requirements: compute_profile,
            communication_volume: comm_patterns.total_volume(),
            critical_path: graph.find_critical_path(),
        })
    }
    
    pub fn suggest_sharding(&self, analysis: &ModelAnalysis, constraints: &ShardingConstraints) -> ShardingPlan {
        match constraints.optimization_goal {
            OptimizationGoal::MinimizeLatency => {
                self.minimize_latency_sharding(analysis, constraints)
            },
            OptimizationGoal::MinimizeMemory => {
                self.minimize_memory_sharding(analysis, constraints)
            },
            OptimizationGoal::MaximizeThroughput => {
                self.maximize_throughput_sharding(analysis, constraints)
            },
            OptimizationGoal::BalanceLoad => {
                self.balance_load_sharding(analysis, constraints)
            },
        }
    }
}
```

### Sharding Planner

```rust
pub struct ShardingPlanner {
    pub strategy: ShardingStrategy,
    pub node_capabilities: HashMap<NodeId, NodeCapabilities>,
    pub network_topology: NetworkTopology,
    pub optimization_solver: OptimizationSolver,
}

impl ShardingPlanner {
    pub async fn create_sharding_plan(
        &mut self,
        model: &Model,
        available_nodes: Vec<NodeId>
    ) -> Result<ShardingPlan> {
        // Analyze model structure
        let analysis = self.analyze_model(model).await?;
        
        // Get node capabilities
        let capabilities = self.collect_node_capabilities(&available_nodes).await?;
        
        // Create constraint matrix
        let constraints = self.build_constraints(&analysis, &capabilities)?;
        
        // Solve optimization problem
        let solution = self.optimization_solver.solve(constraints)?;
        
        // Generate sharding plan
        Ok(self.generate_plan_from_solution(solution, model))
    }
    
    fn build_constraints(
        &self,
        analysis: &ModelAnalysis,
        capabilities: &HashMap<NodeId, NodeCapabilities>
    ) -> ConstraintMatrix {
        let mut constraints = ConstraintMatrix::new();
        
        // Memory constraints
        for (node_id, cap) in capabilities {
            constraints.add_constraint(
                Constraint::Memory {
                    node: *node_id,
                    max_bytes: cap.available_memory_gb * 1_073_741_824,
                }
            );
        }
        
        // Compute constraints
        for (node_id, cap) in capabilities {
            constraints.add_constraint(
                Constraint::Compute {
                    node: *node_id,
                    max_flops: cap.compute_tflops * 1e12,
                }
            );
        }
        
        // Communication constraints
        constraints.add_constraint(
            Constraint::Communication {
                max_bandwidth_gbps: self.network_topology.aggregate_bandwidth(),
                latency_budget_ms: 100,
            }
        );
        
        constraints
    }
}
```

## Layer-wise Sharding

### Horizontal Partitioning

```rust
pub struct LayerWiseSharding {
    pub shard_boundaries: Vec<LayerBoundary>,
    pub activation_checkpointing: bool,
    pub gradient_accumulation: bool,
}

impl LayerWiseSharding {
    pub fn partition_model(&self, model: &Model, num_shards: u32) -> Result<Vec<ModelShard>> {
        let layers = model.get_layers();
        let total_layers = layers.len();
        let base_layers_per_shard = total_layers / num_shards as usize;
        let extra_layers = total_layers % num_shards as usize;
        
        let mut shards = Vec::new();
        let mut layer_idx = 0;
        
        for shard_id in 0..num_shards {
            let shard_layers = if (shard_id as usize) < extra_layers {
                base_layers_per_shard + 1
            } else {
                base_layers_per_shard
            };
            
            let shard = ModelShard {
                shard_id: ShardId::new(shard_id),
                shard_type: ShardType::Sequential,
                layers: layers[layer_idx..layer_idx + shard_layers].to_vec(),
                parameters: self.extract_parameters(&layers[layer_idx..layer_idx + shard_layers]),
                size_bytes: self.calculate_shard_size(&layers[layer_idx..layer_idx + shard_layers]),
                compute_requirements: self.estimate_compute(&layers[layer_idx..layer_idx + shard_layers]),
                dependencies: if shard_id > 0 {
                    vec![ShardId::new(shard_id - 1)]
                } else {
                    vec![]
                },
            };
            
            shards.push(shard);
            layer_idx += shard_layers;
        }
        
        Ok(shards)
    }
    
    pub fn create_pipeline_schedule(
        &self,
        shards: &[ModelShard],
        micro_batch_size: usize
    ) -> PipelineSchedule {
        PipelineSchedule {
            stages: shards.len(),
            micro_batches: self.calculate_optimal_micro_batches(shards, micro_batch_size),
            forward_schedule: self.generate_forward_schedule(shards),
            backward_schedule: self.generate_backward_schedule(shards),
            bubble_time: self.estimate_pipeline_bubble(shards),
        }
    }
}
```

### Pipeline Parallel Execution

```rust
pub struct PipelineExecutor {
    pub schedule: PipelineSchedule,
    pub shard_executors: HashMap<ShardId, ShardExecutor>,
    pub activation_buffer: ActivationBuffer,
    pub gradient_buffer: GradientBuffer,
}

impl PipelineExecutor {
    pub async fn execute_pipeline_step(
        &mut self,
        input_batch: Batch
    ) -> Result<PipelineOutput> {
        // Split batch into micro-batches
        let micro_batches = input_batch.split(self.schedule.micro_batches);
        
        // Execute pipeline schedule
        let mut outputs = Vec::new();
        
        for (step, operations) in self.schedule.forward_schedule.iter().enumerate() {
            // Execute operations in parallel
            let futures: Vec<_> = operations.iter().map(|op| {
                self.execute_operation(op, &micro_batches)
            }).collect();
            
            let results = futures::future::join_all(futures).await;
            outputs.extend(results);
            
            // Manage activation memory
            if self.activation_checkpointing {
                self.checkpoint_activations(step).await?;
            }
        }
        
        Ok(PipelineOutput { outputs })
    }
}
```

## Tensor-wise Sharding

### Tensor Partitioning

```rust
pub struct TensorSharding {
    pub partition_strategy: TensorPartitionStrategy,
    pub communication_backend: CommunicationBackend,
    pub tensor_parallel_size: u32,
}

pub enum TensorPartitionStrategy {
    /// Split along batch dimension
    BatchSplit,
    
    /// Split along feature dimension
    FeatureSplit {
        split_dimension: usize,
    },
    
    /// Split attention heads
    AttentionHeadSplit {
        num_heads: u32,
    },
    
    /// 2D mesh partitioning
    Mesh2D {
        row_size: u32,
        col_size: u32,
    },
}

impl TensorSharding {
    pub fn partition_tensor(
        &self,
        tensor: &Tensor,
        num_partitions: u32
    ) -> Result<Vec<TensorShard>> {
        match &self.partition_strategy {
            TensorPartitionStrategy::BatchSplit => {
                self.split_batch_dimension(tensor, num_partitions)
            },
            TensorPartitionStrategy::FeatureSplit { split_dimension } => {
                self.split_feature_dimension(tensor, num_partitions, *split_dimension)
            },
            TensorPartitionStrategy::AttentionHeadSplit { num_heads } => {
                self.split_attention_heads(tensor, num_partitions, *num_heads)
            },
            TensorPartitionStrategy::Mesh2D { row_size, col_size } => {
                self.mesh_partition(tensor, *row_size, *col_size)
            },
        }
    }
    
    pub async fn all_gather_tensor(
        &self,
        shards: Vec<TensorShard>
    ) -> Result<Tensor> {
        match &self.communication_backend {
            CommunicationBackend::Nccl => {
                self.nccl_all_gather(shards).await
            },
            CommunicationBackend::Gloo => {
                self.gloo_all_gather(shards).await
            },
            CommunicationBackend::Custom => {
                self.custom_all_gather(shards).await
            },
        }
    }
}
```

### Communication Optimization

```rust
pub struct CommunicationOptimizer {
    pub overlap_computation: bool,
    pub gradient_compression: bool,
    pub communication_schedule: CommSchedule,
}

impl CommunicationOptimizer {
    pub async fn optimize_all_reduce(
        &mut self,
        gradients: Vec<Gradient>,
        computation_stream: &ComputeStream
    ) -> Result<Gradient> {
        if self.overlap_computation {
            // Start all-reduce while computation continues
            let all_reduce_handle = self.start_async_all_reduce(gradients);
            
            // Continue backward pass computation
            computation_stream.continue_backward().await?;
            
            // Wait for all-reduce to complete
            all_reduce_handle.await
        } else {
            // Traditional synchronous all-reduce
            self.synchronous_all_reduce(gradients).await
        }
    }
    
    pub fn create_communication_schedule(
        &self,
        model_shards: &[ModelShard]
    ) -> CommSchedule {
        let mut schedule = CommSchedule::new();
        
        // Identify communication points
        for (i, shard) in model_shards.iter().enumerate() {
            for dep in &shard.dependencies {
                schedule.add_communication(
                    CommPoint {
                        from_shard: *dep,
                        to_shard: shard.shard_id,
                        data_size: self.estimate_activation_size(shard),
                        priority: Priority::High,
                    }
                );
            }
        }
        
        // Optimize schedule to minimize wait time
        schedule.optimize_for_latency();
        schedule
    }
}
```

## Expert Parallelism (MoE)

### Mixture of Experts Sharding

```rust
pub struct MoESharding {
    pub num_experts: u32,
    pub expert_capacity: u32,
    pub top_k: u32,
    pub load_balancing: LoadBalancingStrategy,
}

impl MoESharding {
    pub fn shard_experts(
        &self,
        experts: Vec<Expert>,
        available_nodes: Vec<NodeId>
    ) -> Result<ExpertPlacement> {
        let mut placement = ExpertPlacement::new();
        
        // Distribute experts across nodes
        let experts_per_node = (experts.len() / available_nodes.len()).max(1);
        
        for (chunk_idx, expert_chunk) in experts.chunks(experts_per_node).enumerate() {
            if chunk_idx < available_nodes.len() {
                placement.assign_experts_to_node(
                    available_nodes[chunk_idx],
                    expert_chunk.to_vec()
                );
            }
        }
        
        // Setup routing table
        placement.build_routing_table();
        
        Ok(placement)
    }
    
    pub async fn route_tokens(
        &self,
        tokens: &Tensor,
        gating_output: &Tensor
    ) -> Result<TokenRouting> {
        // Get top-k experts for each token
        let expert_indices = gating_output.topk(self.top_k as i64, -1, true, true);
        
        // Apply load balancing
        let balanced_routing = match &self.load_balancing {
            LoadBalancingStrategy::AuxiliaryLoss(aux_weight) => {
                self.apply_auxiliary_loss_balancing(&expert_indices, *aux_weight)
            },
            LoadBalancingStrategy::CapacityFactor(factor) => {
                self.apply_capacity_factor_balancing(&expert_indices, *factor)
            },
            LoadBalancingStrategy::RandomRouting(prob) => {
                self.apply_random_routing(&expert_indices, *prob)
            },
        };
        
        Ok(balanced_routing)
    }
}
```

## Dynamic Resharding

### Adaptive Sharding

```rust
pub struct AdaptiveSharding {
    pub monitoring_interval: Duration,
    pub resharding_threshold: ReshardingThreshold,
    pub migration_strategy: MigrationStrategy,
}

pub struct ReshardingThreshold {
    pub load_imbalance: f32,
    pub memory_pressure: f32,
    pub communication_overhead: f32,
}

impl AdaptiveSharding {
    pub async fn monitor_and_adapt(
        &mut self,
        current_sharding: &ShardingPlan
    ) -> Result<Option<ShardingPlan>> {
        // Collect metrics
        let metrics = self.collect_sharding_metrics(current_sharding).await?;
        
        // Check if resharding is needed
        if self.should_reshard(&metrics) {
            // Generate new sharding plan
            let new_plan = self.generate_adaptive_plan(current_sharding, &metrics)?;
            
            // Plan migration
            let migration_plan = self.plan_migration(current_sharding, &new_plan)?;
            
            // Execute migration
            self.execute_migration(migration_plan).await?;
            
            Ok(Some(new_plan))
        } else {
            Ok(None)
        }
    }
    
    fn should_reshard(&self, metrics: &ShardingMetrics) -> bool {
        metrics.load_imbalance > self.resharding_threshold.load_imbalance ||
        metrics.memory_pressure > self.resharding_threshold.memory_pressure ||
        metrics.communication_overhead > self.resharding_threshold.communication_overhead
    }
}
```

### State Migration

```rust
pub struct StateMigration {
    pub migration_protocol: MigrationProtocol,
    pub checkpoint_based: bool,
    pub zero_downtime: bool,
}

impl StateMigration {
    pub async fn migrate_shard(
        &self,
        shard: &ModelShard,
        from_node: NodeId,
        to_node: NodeId
    ) -> Result<()> {
        if self.zero_downtime {
            // Live migration with continuous service
            self.live_migrate_shard(shard, from_node, to_node).await
        } else {
            // Stop-and-copy migration
            self.stop_and_copy_migrate(shard, from_node, to_node).await
        }
    }
    
    async fn live_migrate_shard(
        &self,
        shard: &ModelShard,
        from_node: NodeId,
        to_node: NodeId
    ) -> Result<()> {
        // Phase 1: Copy while serving
        let snapshot = self.create_shard_snapshot(shard, from_node).await?;
        self.transfer_snapshot(snapshot, to_node).await?;
        
        // Phase 2: Incremental sync
        loop {
            let delta = self.get_shard_delta(shard, from_node).await?;
            if delta.is_small_enough() {
                break;
            }
            self.apply_delta(delta, to_node).await?;
        }
        
        // Phase 3: Atomic switchover
        self.atomic_switchover(shard, from_node, to_node).await?;
        
        Ok(())
    }
}
```

## Heterogeneous Node Support

### Node-Aware Sharding

```rust
pub struct HeterogeneousSharding {
    pub node_profiler: NodeProfiler,
    pub capability_matcher: CapabilityMatcher,
    pub fallback_strategy: FallbackStrategy,
}

impl HeterogeneousSharding {
    pub async fn assign_shards_to_nodes(
        &self,
        shards: Vec<ModelShard>,
        nodes: Vec<NodeInfo>
    ) -> Result<ShardAssignment> {
        // Profile node capabilities
        let profiles = self.node_profiler.profile_all(&nodes).await?;
        
        // Create assignment matrix
        let mut assignment = ShardAssignment::new();
        
        // Sort shards by compute requirements
        let mut sorted_shards = shards;
        sorted_shards.sort_by_key(|s| s.compute_requirements.total_flops);
        
        // Assign shards to best-fit nodes
        for shard in sorted_shards.iter().rev() {
            let best_node = self.find_best_node(shard, &profiles)?;
            
            // Check if node can handle shard
            if self.can_node_handle_shard(&best_node, shard) {
                assignment.assign(shard.shard_id, best_node.node_id);
            } else {
                // Apply fallback strategy
                match &self.fallback_strategy {
                    FallbackStrategy::SplitShard => {
                        let sub_shards = self.split_shard_further(shard)?;
                        for sub_shard in sub_shards {
                            let node = self.find_capable_node(&sub_shard, &profiles)?;
                            assignment.assign(sub_shard.shard_id, node.node_id);
                        }
                    },
                    FallbackStrategy::OffloadCompute => {
                        assignment.assign_with_offload(shard.shard_id, best_node.node_id);
                    },
                }
            }
        }
        
        Ok(assignment)
    }
}
```

## Performance Optimization

### Shard Placement Optimization

```rust
pub struct PlacementOptimizer {
    pub cost_model: CostModel,
    pub optimization_algorithm: OptimizationAlgorithm,
    pub constraints: PlacementConstraints,
}

impl PlacementOptimizer {
    pub fn optimize_placement(
        &self,
        shards: &[ModelShard],
        nodes: &[NodeInfo],
        network: &NetworkTopology
    ) -> Result<OptimalPlacement> {
        // Build cost matrix
        let cost_matrix = self.build_cost_matrix(shards, nodes, network)?;
        
        // Apply optimization algorithm
        let placement = match &self.optimization_algorithm {
            OptimizationAlgorithm::Hungarian => {
                self.hungarian_algorithm(cost_matrix)
            },
            OptimizationAlgorithm::SimulatedAnnealing => {
                self.simulated_annealing(cost_matrix, 1000)
            },
            OptimizationAlgorithm::GeneticAlgorithm => {
                self.genetic_algorithm(cost_matrix, 100, 1000)
            },
        };
        
        Ok(placement?)
    }
    
    fn build_cost_matrix(
        &self,
        shards: &[ModelShard],
        nodes: &[NodeInfo],
        network: &NetworkTopology
    ) -> Result<CostMatrix> {
        let mut matrix = CostMatrix::new(shards.len(), nodes.len());
        
        for (i, shard) in shards.iter().enumerate() {
            for (j, node) in nodes.iter().enumerate() {
                let compute_cost = self.cost_model.compute_cost(shard, node);
                let memory_cost = self.cost_model.memory_cost(shard, node);
                let comm_cost = self.cost_model.communication_cost(shard, node, network);
                
                matrix.set(i, j, compute_cost + memory_cost + comm_cost);
            }
        }
        
        Ok(matrix)
    }
}
```