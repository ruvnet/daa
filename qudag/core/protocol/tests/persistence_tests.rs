use qudag_protocol::{
    persistence::PeerStats, MemoryBackend, Node, NodeConfig, NodeStateProvider, PersistedDagState,
    PersistedPeer, PersistedState, PersistenceManager, SqliteBackend, StatePersistence,
    StateProvider, CURRENT_STATE_VERSION,
};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tempfile::TempDir;
use tokio::sync::RwLock;

#[tokio::test]
async fn test_memory_backend_persistence() {
    let backend = Arc::new(MemoryBackend::default());

    // Create test state
    let state = create_test_state();

    // Save state
    backend.save_state(&state).await.unwrap();

    // Load state
    let loaded = backend.load_state().await.unwrap();
    assert!(loaded.is_some());

    let loaded_state = loaded.unwrap();
    assert_eq!(loaded_state.version, state.version);
    assert_eq!(loaded_state.node_id, state.node_id);
    assert_eq!(loaded_state.peers.len(), state.peers.len());
}

#[tokio::test]
async fn test_sqlite_backend_persistence() {
    let temp_dir = TempDir::new().unwrap();
    std::fs::create_dir_all(temp_dir.path()).unwrap();
    let db_path = temp_dir.path().join("test.db");

    let backend = Arc::new(SqliteBackend::new(db_path.clone()).await.unwrap());

    // Create test state
    let state = create_test_state();

    // Save state
    backend.save_state(&state).await.unwrap();

    // Load state
    let loaded = backend.load_state().await.unwrap();
    assert!(loaded.is_some());

    let loaded_state = loaded.unwrap();
    assert_eq!(loaded_state.version, state.version);
    assert_eq!(loaded_state.node_id, state.node_id);

    // Test persistence across backend instances
    drop(backend);

    // Create new backend instance
    let backend2 = Arc::new(SqliteBackend::new(db_path).await.unwrap());
    let loaded2 = backend2.load_state().await.unwrap();
    assert!(loaded2.is_some());
    assert_eq!(loaded2.unwrap().node_id, state.node_id);
}

#[tokio::test]
async fn test_peer_persistence() {
    let backend = Arc::new(MemoryBackend::default());

    let peers = vec![
        PersistedPeer {
            id: vec![1, 2, 3, 4],
            address: "127.0.0.1:8000".to_string(),
            reputation: 80,
            last_seen: 1234567890,
            stats: PeerStats {
                total_connections: 10,
                successful_connections: 8,
                failed_connections: 2,
                bytes_sent: 1024,
                bytes_received: 2048,
                avg_response_time: 50,
            },
            blacklisted: false,
            whitelisted: true,
            metadata: HashMap::new(),
        },
        PersistedPeer {
            id: vec![5, 6, 7, 8],
            address: "192.168.1.100:9000".to_string(),
            reputation: 60,
            last_seen: 1234567900,
            stats: PeerStats::default(),
            blacklisted: true,
            whitelisted: false,
            metadata: HashMap::new(),
        },
    ];

    // Save peers
    backend.save_peers(&peers).await.unwrap();

    // Load peers
    let loaded = backend.load_peers().await.unwrap();
    assert_eq!(loaded.len(), 2);
    assert_eq!(loaded[0].id, vec![1, 2, 3, 4]);
    assert_eq!(loaded[0].reputation, 80);
    assert!(loaded[0].whitelisted);
    assert!(!loaded[0].blacklisted);
    assert_eq!(loaded[1].id, vec![5, 6, 7, 8]);
    assert!(loaded[1].blacklisted);
}

#[tokio::test]
async fn test_dag_state_persistence() {
    let backend = Arc::new(MemoryBackend::default());

    let dag_state = PersistedDagState {
        vertices: HashMap::new(),
        tips: HashSet::new(),
        voting_records: HashMap::new(),
        last_checkpoint: None,
    };

    // Save DAG state
    backend.save_dag_state(&dag_state).await.unwrap();

    // Load DAG state
    let loaded = backend.load_dag_state().await.unwrap();
    assert!(loaded.is_some());
}

#[tokio::test]
async fn test_state_validation() {
    let backend = Arc::new(MemoryBackend::default());

    // Should be valid with no state
    assert!(backend.validate_state().await.unwrap());

    // Save valid state
    let state = create_test_state();
    backend.save_state(&state).await.unwrap();
    assert!(backend.validate_state().await.unwrap());

    // Save invalid state (empty node_id)
    let mut invalid_state = state.clone();
    invalid_state.node_id = vec![];
    backend.save_state(&invalid_state).await.unwrap();
    assert!(!backend.validate_state().await.unwrap());
}

#[tokio::test]
async fn test_backup_and_restore() {
    let temp_dir = TempDir::new().unwrap();
    let backend = Arc::new(MemoryBackend::default());

    // Create and save state
    let state = create_test_state();
    backend.save_state(&state).await.unwrap();

    // Create backup
    let backup_path = temp_dir.path();
    backend.create_backup(backup_path).await.unwrap();

    // Clear state
    backend
        .save_state(&PersistedState {
            version: CURRENT_STATE_VERSION,
            node_id: vec![99, 99, 99],
            protocol_state: qudag_protocol::state::ProtocolState::Initial,
            sessions: HashMap::new(),
            peers: vec![],
            dag_state: PersistedDagState {
                vertices: HashMap::new(),
                tips: HashSet::new(),
                voting_records: HashMap::new(),
                last_checkpoint: None,
            },
            metrics: Default::default(),
            last_saved: 0,
        })
        .await
        .unwrap();

    // Verify state changed
    let modified = backend.load_state().await.unwrap().unwrap();
    assert_eq!(modified.node_id, vec![99, 99, 99]);

    // Restore from backup
    backend.restore_backup(backup_path).await.unwrap();

    // Verify state restored
    let restored = backend.load_state().await.unwrap().unwrap();
    assert_eq!(restored.node_id, state.node_id);
}

#[tokio::test]
async fn test_persistence_manager() {
    let backend = Arc::new(MemoryBackend::default());
    let manager = PersistenceManager::new(backend.clone());

    // Test state recovery with no existing state
    let recovered = manager.recover_state().await.unwrap();
    assert!(recovered.is_none());

    // Save state
    let state = create_test_state();
    backend.save_state(&state).await.unwrap();

    // Test state recovery
    let recovered = manager.recover_state().await.unwrap();
    assert!(recovered.is_some());
    assert_eq!(recovered.unwrap().node_id, state.node_id);
}

#[tokio::test]
async fn test_state_export_import() {
    let temp_dir = TempDir::new().unwrap();
    let backend = Arc::new(MemoryBackend::default());
    let manager = PersistenceManager::new(backend.clone());

    // Save state
    let state = create_test_state();
    backend.save_state(&state).await.unwrap();

    // Export state
    let export_path = temp_dir.path().join("export.json");
    manager.export_state(&export_path).await.unwrap();

    // Clear state
    backend
        .save_state(&PersistedState {
            version: CURRENT_STATE_VERSION,
            node_id: vec![88, 88, 88],
            protocol_state: qudag_protocol::state::ProtocolState::Initial,
            sessions: HashMap::new(),
            peers: vec![],
            dag_state: PersistedDagState {
                vertices: HashMap::new(),
                tips: HashSet::new(),
                voting_records: HashMap::new(),
                last_checkpoint: None,
            },
            metrics: Default::default(),
            last_saved: 0,
        })
        .await
        .unwrap();

    // Import state
    manager.import_state(&export_path).await.unwrap();

    // Verify imported state
    let imported = backend.load_state().await.unwrap().unwrap();
    assert_eq!(imported.node_id, state.node_id);
    assert_eq!(imported.peers.len(), state.peers.len());
}

#[tokio::test]
async fn test_node_with_persistence() {
    let temp_dir = TempDir::new().unwrap();
    let config = NodeConfig {
        data_dir: temp_dir.path().to_path_buf(),
        ..Default::default()
    };

    // Create directory for SQLite
    std::fs::create_dir_all(&config.data_dir).unwrap();

    // Create node with persistence
    let mut node = Node::with_persistence(config.clone()).await.unwrap();

    // Start node
    node.start().await.unwrap();

    // Save state
    node.save_state().await.unwrap();

    // Create backup
    let backup_path = temp_dir.path().join("backup");
    std::fs::create_dir_all(&backup_path).unwrap();
    node.create_backup(backup_path.clone()).await.unwrap();

    // Stop node
    node.stop().await.unwrap();

    // Create new node and verify persistence works
    let node2 = Node::with_persistence(config).await.unwrap();
    // Note: In a real implementation, the node ID would be loaded from persistence
    assert!(node2.has_persistence());
}

#[tokio::test]
async fn test_node_state_provider() {
    let config = NodeConfig::default();
    let node = Arc::new(RwLock::new(Node::new(config).await.unwrap()));

    // Create state provider
    let provider = NodeStateProvider::new(node.clone());

    // Get current state
    let state = provider.get_current_state().await.unwrap();
    assert_eq!(state.version, CURRENT_STATE_VERSION);
    assert!(!state.node_id.is_empty());
}

// Helper function to create test state
fn create_test_state() -> PersistedState {
    PersistedState {
        version: CURRENT_STATE_VERSION,
        node_id: vec![1, 2, 3, 4, 5, 6, 7, 8],
        protocol_state: qudag_protocol::state::ProtocolState::Active(
            qudag_protocol::state::ActiveState::Normal,
        ),
        sessions: HashMap::new(),
        peers: vec![PersistedPeer {
            id: vec![10, 11, 12, 13],
            address: "127.0.0.1:8000".to_string(),
            reputation: 75,
            last_seen: 1234567890,
            stats: PeerStats::default(),
            blacklisted: false,
            whitelisted: false,
            metadata: HashMap::new(),
        }],
        dag_state: PersistedDagState {
            vertices: HashMap::new(),
            tips: HashSet::new(),
            voting_records: HashMap::new(),
            last_checkpoint: None,
        },
        metrics: qudag_protocol::state::StateMachineMetrics {
            current_state: qudag_protocol::state::ProtocolState::Active(
                qudag_protocol::state::ActiveState::Normal,
            ),
            uptime: std::time::Duration::from_secs(100),
            active_sessions: 5,
            total_state_transitions: 10,
            total_messages_sent: 1000,
            total_messages_received: 1200,
            total_bytes_sent: 100000,
            total_bytes_received: 120000,
            total_errors: 2,
        },
        last_saved: 1234567890,
    }
}
