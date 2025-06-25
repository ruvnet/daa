use prime_trainer::{Trainer, TrainerConfig, GradientAggregator, ModelShard};
use prime_core::TrainingConfig;
use tch::{nn, Device, Tensor};

#[tokio::test]
async fn test_trainer_initialization() {
    let config = TrainerConfig {
        device: Device::Cpu,
        checkpoint_interval: 100,
        gradient_compression: true,
    };
    
    let trainer = Trainer::new(config).await.unwrap();
    assert_eq!(trainer.device(), Device::Cpu);
}

#[tokio::test]
async fn test_gradient_aggregation() {
    let mut aggregator = GradientAggregator::new(4);
    
    // Simulate gradients from different nodes
    let grad1 = vec![1.0, 2.0, 3.0];
    let grad2 = vec![2.0, 3.0, 4.0];
    let grad3 = vec![3.0, 4.0, 5.0];
    let grad4 = vec![4.0, 5.0, 6.0];
    
    aggregator.add_gradient("node1", grad1).await;
    aggregator.add_gradient("node2", grad2).await;
    aggregator.add_gradient("node3", grad3).await;
    aggregator.add_gradient("node4", grad4).await;
    
    let averaged = aggregator.compute_average().await.unwrap();
    assert_eq!(averaged, vec![2.5, 3.5, 4.5]);
}

#[tokio::test]
async fn test_model_sharding() {
    let model_size = 1000;
    let num_shards = 4;
    
    let shards = ModelShard::create_shards(model_size, num_shards);
    assert_eq!(shards.len(), num_shards);
    
    let total_size: usize = shards.iter().map(|s| s.size()).sum();
    assert_eq!(total_size, model_size);
}

#[tokio::test]
async fn test_fsdp_training_step() {
    let config = TrainerConfig::default();
    let mut trainer = Trainer::new(config).await.unwrap();
    
    let training_config = TrainingConfig {
        batch_size: 16,
        learning_rate: 0.001,
        epochs: 1,
        gradient_accumulation_steps: 4,
    };
    
    // Mock training data
    let inputs = Tensor::randn(&[16, 10], (tch::Kind::Float, Device::Cpu));
    let targets = Tensor::randn(&[16, 1], (tch::Kind::Float, Device::Cpu));
    
    let loss = trainer.train_step(&inputs, &targets, &training_config).await.unwrap();
    assert!(loss > 0.0);
}

#[tokio::test]
async fn test_checkpoint_saving() {
    let config = TrainerConfig::default();
    let trainer = Trainer::new(config).await.unwrap();
    
    let checkpoint_path = "/tmp/test_checkpoint.pt";
    trainer.save_checkpoint(checkpoint_path).await.unwrap();
    
    // Verify checkpoint exists
    assert!(std::path::Path::new(checkpoint_path).exists());
    
    // Clean up
    std::fs::remove_file(checkpoint_path).ok();
}