//! Test fixtures and data generators

use fake::{Fake, Faker};
use proptest::prelude::*;
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

/// Generate test model parameters
pub struct ModelFixtures;

impl ModelFixtures {
    pub fn simple_model() -> ModelParams {
        ModelParams {
            name: "test_model".to_string(),
            layers: vec![
                Layer { neurons: 784, activation: "relu".to_string() },
                Layer { neurons: 128, activation: "relu".to_string() },
                Layer { neurons: 10, activation: "softmax".to_string() },
            ],
            learning_rate: 0.01,
            batch_size: 32,
        }
    }

    pub fn large_model() -> ModelParams {
        ModelParams {
            name: "large_test_model".to_string(),
            layers: vec![
                Layer { neurons: 10000, activation: "relu".to_string() },
                Layer { neurons: 5000, activation: "relu".to_string() },
                Layer { neurons: 2000, activation: "relu".to_string() },
                Layer { neurons: 1000, activation: "relu".to_string() },
                Layer { neurons: 100, activation: "softmax".to_string() },
            ],
            learning_rate: 0.001,
            batch_size: 128,
        }
    }

    pub fn random_model() -> ModelParams {
        let num_layers: usize = (2..10).fake();
        let mut layers = Vec::new();
        let mut prev_neurons = (100..1000).fake();
        
        for _ in 0..num_layers {
            let neurons = (10..prev_neurons).fake();
            let activation = ["relu", "tanh", "sigmoid"].choose(&mut rand::thread_rng()).unwrap();
            layers.push(Layer {
                neurons,
                activation: activation.to_string(),
            });
            prev_neurons = neurons;
        }

        ModelParams {
            name: Faker.fake(),
            layers,
            learning_rate: (0.0001..0.1).fake(),
            batch_size: [16, 32, 64, 128].choose(&mut rand::thread_rng()).copied().unwrap(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ModelParams {
    pub name: String,
    pub layers: Vec<Layer>,
    pub learning_rate: f32,
    pub batch_size: usize,
}

#[derive(Debug, Clone)]
pub struct Layer {
    pub neurons: usize,
    pub activation: String,
}

/// Generate test training data
pub struct DataFixtures;

impl DataFixtures {
    pub fn mnist_batch(size: usize) -> TrainingBatch {
        let mut data = Vec::new();
        let mut labels = Vec::new();
        
        for _ in 0..size {
            // 28x28 image flattened
            let image: Vec<f32> = (0..784).map(|_| rand::random()).collect();
            let label = (0..10).fake();
            data.push(image);
            labels.push(label);
        }

        TrainingBatch { data, labels }
    }

    pub fn random_batch(input_size: usize, output_size: usize, batch_size: usize) -> TrainingBatch {
        let mut data = Vec::new();
        let mut labels = Vec::new();
        
        for _ in 0..batch_size {
            let sample: Vec<f32> = (0..input_size).map(|_| rand::random()).collect();
            let label = (0..output_size).fake();
            data.push(sample);
            labels.push(label);
        }

        TrainingBatch { data, labels }
    }
}

#[derive(Debug, Clone)]
pub struct TrainingBatch {
    pub data: Vec<Vec<f32>>,
    pub labels: Vec<usize>,
}

/// Generate test network configurations
pub struct NetworkFixtures;

impl NetworkFixtures {
    pub fn bootstrap_nodes(count: usize) -> Vec<NodeConfig> {
        (0..count)
            .map(|i| NodeConfig {
                peer_id: format!("bootstrap_{}", i),
                address: SocketAddr::new(
                    IpAddr::V4(Ipv4Addr::LOCALHOST),
                    8000 + i as u16,
                ),
                role: NodeRole::Bootstrap,
                capabilities: vec!["dht".to_string(), "relay".to_string()],
            })
            .collect()
    }

    pub fn trainer_nodes(count: usize) -> Vec<NodeConfig> {
        (0..count)
            .map(|i| NodeConfig {
                peer_id: format!("trainer_{}", i),
                address: SocketAddr::new(
                    IpAddr::V4(Ipv4Addr::LOCALHOST),
                    9000 + i as u16,
                ),
                role: NodeRole::Trainer,
                capabilities: vec!["compute".to_string(), "storage".to_string()],
            })
            .collect()
    }

    pub fn validator_nodes(count: usize) -> Vec<NodeConfig> {
        (0..count)
            .map(|i| NodeConfig {
                peer_id: format!("validator_{}", i),
                address: SocketAddr::new(
                    IpAddr::V4(Ipv4Addr::LOCALHOST),
                    10000 + i as u16,
                ),
                role: NodeRole::Validator,
                capabilities: vec!["consensus".to_string(), "verification".to_string()],
            })
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct NodeConfig {
    pub peer_id: String,
    pub address: SocketAddr,
    pub role: NodeRole,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NodeRole {
    Bootstrap,
    Trainer,
    Validator,
    Coordinator,
}

/// Property-based test generators
pub mod generators {
    use super::*;
    use proptest::prelude::*;

    pub fn valid_peer_id() -> impl Strategy<Value = String> {
        "[a-zA-Z0-9]{20,60}".prop_map(|s| format!("peer_{}", s))
    }

    pub fn valid_port() -> impl Strategy<Value = u16> {
        49152u16..65535u16
    }

    pub fn valid_address() -> impl Strategy<Value = SocketAddr> {
        valid_port().prop_map(|port| {
            SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port)
        })
    }

    pub fn model_params() -> impl Strategy<Value = ModelParams> {
        (
            "[a-zA-Z0-9_]{5,20}",
            prop::collection::vec(layer_spec(), 1..10),
            0.0001f32..0.1f32,
            prop::sample::select(vec![16, 32, 64, 128]),
        ).prop_map(|(name, layers, lr, batch)| {
            ModelParams {
                name,
                layers,
                learning_rate: lr,
                batch_size: batch,
            }
        })
    }

    fn layer_spec() -> impl Strategy<Value = Layer> {
        (
            10usize..1000usize,
            prop::sample::select(vec!["relu", "tanh", "sigmoid", "softmax"]),
        ).prop_map(|(neurons, activation)| {
            Layer {
                neurons,
                activation: activation.to_string(),
            }
        })
    }

    pub fn training_batch(max_size: usize) -> impl Strategy<Value = TrainingBatch> {
        (1usize..max_size).prop_flat_map(|size| {
            (
                prop::collection::vec(
                    prop::collection::vec(0.0f32..1.0f32, 784),
                    size,
                ),
                prop::collection::vec(0usize..10usize, size),
            ).prop_map(|(data, labels)| TrainingBatch { data, labels })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_fixtures() {
        let simple = ModelFixtures::simple_model();
        assert_eq!(simple.layers.len(), 3);
        assert_eq!(simple.layers[0].neurons, 784);

        let large = ModelFixtures::large_model();
        assert_eq!(large.layers.len(), 5);
        assert!(large.layers[0].neurons > 5000);

        let random = ModelFixtures::random_model();
        assert!(random.layers.len() >= 2);
    }

    #[test]
    fn test_data_fixtures() {
        let mnist = DataFixtures::mnist_batch(10);
        assert_eq!(mnist.data.len(), 10);
        assert_eq!(mnist.labels.len(), 10);
        assert_eq!(mnist.data[0].len(), 784);

        let random = DataFixtures::random_batch(100, 10, 32);
        assert_eq!(random.data.len(), 32);
        assert_eq!(random.data[0].len(), 100);
    }

    #[test]
    fn test_network_fixtures() {
        let bootstrap = NetworkFixtures::bootstrap_nodes(3);
        assert_eq!(bootstrap.len(), 3);
        assert_eq!(bootstrap[0].role, NodeRole::Bootstrap);

        let trainers = NetworkFixtures::trainer_nodes(5);
        assert_eq!(trainers.len(), 5);
        assert_eq!(trainers[0].role, NodeRole::Trainer);
    }

    proptest! {
        #[test]
        fn test_model_params_generator(params in generators::model_params()) {
            assert!(!params.name.is_empty());
            assert!(!params.layers.is_empty());
            assert!(params.learning_rate > 0.0);
            assert!(params.batch_size > 0);
        }

        #[test]
        fn test_training_batch_generator(batch in generators::training_batch(100)) {
            assert_eq!(batch.data.len(), batch.labels.len());
            for sample in &batch.data {
                assert_eq!(sample.len(), 784);
            }
        }
    }
}