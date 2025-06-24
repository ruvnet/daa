use qudag_protocol::{consensus::DAGConsensus, node::Node, message::Message};
use test_utils::protocol::*;

#[cfg(test)]
mod protocol_security_tests {
    use super::*;

    #[test]
    fn test_consensus_byzantine_resistance() {
        let mut network = TestNetwork::new(10);
        
        // Introduce Byzantine nodes
        network.set_byzantine_nodes(3);
        
        // Run consensus rounds
        let consensus = DAGConsensus::new();
        let result = consensus.run_rounds(&network, 10);
        
        assert!(result.is_valid(), "Consensus failed with Byzantine nodes");
        assert!(result.agreement_reached(), "Agreement not reached with Byzantine nodes");
    }

    #[test]
    fn test_message_ordering_integrity() {
        let node = Node::new();
        let messages = generate_test_messages(100);
        
        // Submit messages out of order
        let shuffled = shuffle_messages(&messages);
        for msg in shuffled {
            node.submit_message(msg);
        }
        
        // Verify DAG preserves casual ordering
        let dag = node.get_dag();
        assert!(dag.maintains_casual_ordering(), "Casual ordering violated");
        assert!(dag.no_cycles_present(), "Cycles detected in DAG");
    }

    #[test]
    fn test_sybil_resistance() {
        let network = TestNetwork::new(20);
        
        // Attempt Sybil attack
        let attack = SybilAttack::new(&network);
        attack.execute();
        
        // Verify network resistance
        assert!(network.sybil_resistance_score() > 0.95, 
            "Network vulnerable to Sybil attacks");
        
        // Check node validation
        assert!(network.all_nodes_validated(), 
            "Node validation mechanism failed");
    }
}