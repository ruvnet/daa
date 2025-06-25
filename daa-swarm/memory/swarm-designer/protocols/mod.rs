//! Hybrid MoE-Swarm Coordination Protocols
//! 
//! Revolutionary swarm intelligence protocols combining quantum mechanics,
//! biological inspiration, and emergent behaviors for distributed AI systems.

pub mod quantum_entanglement_protocol;
pub mod stigmergic_gpu_protocol;
pub mod emergent_consensus_protocol;
pub mod bio_routing_protocol;
pub mod hybrid_moe_swarm_patterns;

// Re-export core types for convenience
pub use quantum_entanglement_protocol::{
    QuantumEntanglementProtocol, QuantumExpertState, EntanglementLink,
    BellState, QuantumEvent, EmergentBehavior as QuantumEmergentBehavior,
};

pub use stigmergic_gpu_protocol::{
    StigmergicGPUProtocol, GPUStigmergicMemory, NavigationState,
    NavigationStrategy, PheromoneCell, EmergentPattern,
};

pub use emergent_consensus_protocol::{
    EmergentConsensusProtocol, EmergentConsensusState, OpinionVector,
    AttractorBasin, ConsensusEvent, ConsensusResult,
};

pub use bio_routing_protocol::{
    BioRoutingProtocol, PheromoneRoutingNetwork, ExpertNode,
    Route, RouteRequirements, RoutingEvent,
};

pub use hybrid_moe_swarm_patterns::{
    HybridMoESwarm, QuantumExpert, HybridPattern, HybridPatternType,
    SwarmConfig, SwarmTask, TaskResult, EmergentProperty,
};

/// Protocol version information
pub const PROTOCOL_VERSION: &str = "1.0.0";
pub const PROTOCOL_NAME: &str = "Hybrid MoE-Swarm Coordination";

/// Initialize all protocols with default configuration
pub async fn initialize_protocols() -> Result<HybridMoESwarm, String> {
    let config = SwarmConfig {
        consensus_dimensions: 10,
        initial_topology: hybrid_moe_swarm_patterns::TopologyType::Adaptive,
        evolution_rate: 0.1,
        resource_constraints: hybrid_moe_swarm_patterns::ResourceConstraints {
            max_energy: 1000.0,
            max_memory: 100_000.0,
            max_bandwidth: 10_000.0,
        },
    };
    
    Ok(HybridMoESwarm::new(config).await)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_protocol_initialization() {
        let swarm = initialize_protocols().await;
        assert!(swarm.is_ok());
    }
}