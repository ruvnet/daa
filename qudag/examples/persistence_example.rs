//! Example demonstrating QuDAG node state persistence functionality

use qudag_protocol::{
    Node, NodeConfig, NodeStateProvider, MemoryBackend, PersistenceManager, 
    SqliteBackend, StatePersistence, StateProvider,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tempfile::TempDir;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("QuDAG State Persistence Example");
    println!("==============================\n");

    // Example 1: Using memory backend (for testing)
    println!("1. Memory Backend Example");
    memory_backend_example().await?;
    
    // Example 2: Using SQLite backend (for lightweight deployments)
    println!("\n2. SQLite Backend Example");
    sqlite_backend_example().await?;
    
    // Example 3: Node with automatic persistence
    println!("\n3. Node with Automatic Persistence");
    node_persistence_example().await?;

    Ok(())
}

async fn memory_backend_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating memory backend...");
    
    // Create memory backend
    let backend = Arc::new(MemoryBackend::default());
    let manager = PersistenceManager::new(backend.clone());
    
    // Create a node
    let config = NodeConfig::default();
    let mut node = Node::new(config).await?;
    
    // Start the node
    node.start().await?;
    println!("Node started with ID: {:?}", hex::encode(&node.node_id[..8]));
    
    // Save current state
    let state = node.get_current_state().await?;
    backend.save_state(&state).await?;
    println!("State saved to memory");
    
    // Verify we can load it back
    let loaded_state = backend.load_state().await?;
    if let Some(state) = loaded_state {
        println!("State loaded successfully, node ID: {:?}", 
                 hex::encode(&state.node_id[..8]));
    }
    
    // Stop the node
    node.stop().await?;
    println!("Node stopped");
    
    Ok(())
}

async fn sqlite_backend_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating SQLite backend...");
    
    // Create temporary directory for database
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("node_state.db");
    
    // Create SQLite backend
    let backend = Arc::new(SqliteBackend::new(db_path.clone()).await?);
    let manager = PersistenceManager::new(backend.clone());
    
    // Create and start a node
    let config = NodeConfig {
        data_dir: temp_dir.path().to_path_buf(),
        ..Default::default()
    };
    let mut node = Node::new(config).await?;
    node.start().await?;
    
    println!("Node started with ID: {:?}", hex::encode(&node.node_id[..8]));
    
    // Save state
    let state = node.get_current_state().await?;
    backend.save_state(&state).await?;
    println!("State saved to SQLite database");
    
    // Save some peer information
    let peers = vec![
        qudag_protocol::PersistedPeer {
            id: vec![1, 2, 3, 4, 5, 6, 7, 8],
            address: "127.0.0.1:8001".to_string(),
            reputation: 100,
            last_seen: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            stats: Default::default(),
            blacklisted: false,
            whitelisted: true,
            metadata: std::collections::HashMap::new(),
        },
    ];
    backend.save_peers(&peers).await?;
    println!("Saved {} peer(s)", peers.len());
    
    // Create a backup
    let backup_path = temp_dir.path().join("backup");
    std::fs::create_dir_all(&backup_path)?;
    backend.create_backup(&backup_path).await?;
    println!("Backup created at: {:?}", backup_path);
    
    // Stop the node
    node.stop().await?;
    
    // Simulate node restart - create new backend instance
    println!("\nSimulating node restart...");
    let backend2 = Arc::new(SqliteBackend::new(db_path).await?);
    
    // Load state
    let loaded_state = backend2.load_state().await?;
    if let Some(state) = loaded_state {
        println!("State recovered successfully, node ID: {:?}", 
                 hex::encode(&state.node_id[..8]));
    }
    
    // Load peers
    let loaded_peers = backend2.load_peers().await?;
    println!("Recovered {} peer(s)", loaded_peers.len());
    
    Ok(())
}

async fn node_persistence_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating node with automatic persistence...");
    
    // Create temporary directory for node data
    let temp_dir = TempDir::new()?;
    let config = NodeConfig {
        data_dir: temp_dir.path().to_path_buf(),
        network_port: 8080,
        max_peers: 100,
        initial_peers: vec!["peer1.example.com:8080".to_string()],
    };
    
    // Ensure data directory exists
    std::fs::create_dir_all(&config.data_dir)?;
    
    // Create node with persistence
    let mut node = Node::with_persistence(config.clone()).await?;
    println!("Node created with persistence support");
    
    // Start the node
    node.start().await?;
    println!("Node started with ID: {:?}", hex::encode(&node.node_id[..8]));
    
    // Save state manually
    node.save_state().await?;
    println!("State saved manually");
    
    // Create backup
    let backup_path = temp_dir.path().join("backup");
    std::fs::create_dir_all(&backup_path)?;
    node.create_backup(backup_path.clone()).await?;
    println!("Backup created");
    
    // Stop the node
    node.stop().await?;
    println!("Node stopped");
    
    // Create new node instance and verify persistence
    println!("\nCreating new node instance...");
    let node2 = Node::with_persistence(config).await?;
    println!("New node created, ID should match: {:?}", 
             hex::encode(&node2.node_id[..8]));
    
    // The node IDs won't match in this example because we generate new IDs,
    // but in a real implementation, the node ID would be loaded from persistence
    
    Ok(())
}

// Helper function to format bytes as hex
fn hex_format(bytes: &[u8]) -> String {
    bytes.iter()
        .take(4)
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<_>>()
        .join("")
}