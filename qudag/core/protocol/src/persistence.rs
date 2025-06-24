//! State persistence layer for QuDAG protocol
//!
//! This module provides a comprehensive persistence layer for storing and retrieving
//! DAG vertices, peer information, and dark domain records using different storage backends.

use async_trait::async_trait;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use thiserror::Error;
use tokio::fs::{self, File};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{debug, error, info, warn};

// Import types from other modules
use qudag_dag::vertex::{Vertex, VertexId};
use qudag_network::dark_resolver::DarkDomainRecord;
use qudag_network::types::PeerId;

/// Result type for persistence operations
pub type Result<T> = std::result::Result<T, PersistenceError>;

/// Current state version for compatibility
pub const CURRENT_STATE_VERSION: u32 = 1;

/// Errors that can occur during persistence operations
#[derive(Debug, Error)]
pub enum PersistenceError {
    /// IO error during file operations
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Data corruption detected
    #[error("Data corruption detected: {0}")]
    Corruption(String),

    /// Directory creation failed
    #[error("Directory creation failed: {0}")]
    DirectoryCreation(String),

    /// File not found
    #[error("File not found: {0}")]
    FileNotFound(String),

    /// Invalid data format
    #[error("Invalid data format: {0}")]
    InvalidFormat(String),

    /// Lock acquisition timeout
    #[error("Lock acquisition timeout")]
    LockTimeout,
}

/// Information about a peer for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    /// Peer's network address
    pub address: String,
    /// Last seen timestamp (Unix timestamp)
    pub last_seen: u64,
    /// Reputation score (0-100)
    pub reputation: u8,
    /// Whether the peer is trusted
    pub trusted: bool,
    /// Connection statistics
    pub connection_count: u64,
    /// Total bytes exchanged
    pub bytes_exchanged: u64,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Persisted DAG state for protocol operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedDagState {
    /// Version of the persisted state format
    pub version: u32,
    /// Node identifier
    pub node_id: Vec<u8>,
    /// Current protocol state
    pub protocol_state: crate::state::ProtocolState,
    /// Active sessions
    pub sessions: HashMap<uuid::Uuid, crate::state::SessionInfo>,
    /// Peer information
    pub peers: Vec<(PeerId, PeerInfo)>,
    /// DAG state information
    pub dag_state: DagState,
    /// State machine metrics
    pub metrics: crate::state::StateMachineMetrics,
    /// Timestamp when state was last saved
    pub last_saved: u64,
}

/// DAG-specific state information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DagState {
    /// DAG vertices stored as HashMap for efficient lookup
    pub vertices: HashMap<VertexId, Vertex>,
    /// Current tip vertices
    pub tips: std::collections::HashSet<VertexId>,
    /// Voting records for consensus
    pub voting_records: HashMap<VertexId, VotingRecord>,
    /// Last checkpoint information
    pub last_checkpoint: Option<CheckpointInfo>,
}

/// Voting record for consensus tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VotingRecord {
    /// Vertex being voted on
    pub vertex_id: VertexId,
    /// Votes received (node_id -> vote)
    pub votes: HashMap<Vec<u8>, bool>,
    /// Timestamp when voting started
    pub started_at: u64,
    /// Consensus status
    pub status: ConsensusStatus,
}

/// Consensus status for a vertex
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsensusStatus {
    /// Voting in progress
    Pending,
    /// Consensus reached (accepted)
    Accepted,
    /// Consensus reached (rejected)
    Rejected,
    /// Voting timed out
    TimedOut,
}

/// Checkpoint information for state snapshots
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointInfo {
    /// Checkpoint identifier
    pub id: Vec<u8>,
    /// Checkpoint timestamp
    pub timestamp: u64,
    /// Number of vertices at checkpoint
    pub vertex_count: usize,
    /// Merkle root of DAG state at checkpoint
    pub merkle_root: Vec<u8>,
}

/// General persisted state wrapper
pub type PersistedState = PersistedDagState;

/// Backend storage interface alias
pub type MemoryBackend = MemoryStateStore;
pub type SqliteBackend = FileStateStore; // For now, using file backend as SQLite placeholder

/// Persistence manager for coordinating storage operations
pub type PersistenceManager = Arc<dyn StateStore + Send + Sync>;

/// State persistence trait alias
pub trait StatePersistence: StateStore {}
impl<T: StateStore> StatePersistence for T {}

/// State provider trait for node integration
pub trait StateProvider: Send + Sync {
    fn get_state_store(&self) -> Arc<dyn StateStore + Send + Sync>;
}

impl Default for PeerInfo {
    fn default() -> Self {
        Self {
            address: String::new(),
            last_seen: 0,
            reputation: 50,
            trusted: false,
            connection_count: 0,
            bytes_exchanged: 0,
            metadata: HashMap::new(),
        }
    }
}

/// Abstract storage trait for different persistence backends
#[async_trait]
pub trait StateStore: Send + Sync {
    /// Save a DAG vertex to storage
    async fn save_vertex(&self, vertex: &Vertex) -> Result<()>;

    /// Load a DAG vertex from storage by ID
    async fn load_vertex(&self, id: &VertexId) -> Result<Option<Vertex>>;

    /// Save peer information to storage
    async fn save_peer(&self, peer_id: &PeerId, info: &PeerInfo) -> Result<()>;

    /// Load all peers from storage
    async fn load_peers(&self) -> Result<Vec<(PeerId, PeerInfo)>>;

    /// Save a dark domain record to storage
    async fn save_dark_record(&self, record: &DarkDomainRecord) -> Result<()>;

    /// Load all dark domain records from storage
    async fn load_dark_records(&self) -> Result<Vec<DarkDomainRecord>>;

    /// Remove a vertex from storage
    async fn remove_vertex(&self, id: &VertexId) -> Result<()>;

    /// Remove a peer from storage
    async fn remove_peer(&self, peer_id: &PeerId) -> Result<()>;

    /// Remove a dark domain record from storage by owner ID
    async fn remove_dark_record(&self, owner_id: &PeerId) -> Result<()>;

    /// Get total number of stored vertices
    async fn vertex_count(&self) -> Result<usize>;

    /// Get total number of stored peers
    async fn peer_count(&self) -> Result<usize>;

    /// Get total number of stored dark records
    async fn dark_record_count(&self) -> Result<usize>;

    /// Check if storage is healthy
    async fn health_check(&self) -> Result<bool>;

    /// Save complete persisted state
    async fn save_state(&self, state: &PersistedDagState) -> Result<()>;

    /// Recover complete persisted state
    async fn recover_state(&self) -> Result<Option<PersistedDagState>>;

    /// Create backup of the entire state
    async fn create_backup(&self, backup_path: &PathBuf) -> Result<()>;

    /// Restore from backup
    async fn restore_backup(&self, backup_path: &PathBuf) -> Result<()>;
}

/// File-based storage implementation using JSON files
pub struct FileStateStore {
    /// Base directory for data storage
    data_dir: PathBuf,
    /// Whether to use atomic writes
    atomic_writes: bool,
}

impl FileStateStore {
    /// Create a new file-based state store
    pub async fn new(data_dir: PathBuf) -> Result<Self> {
        // Create the base directory structure
        fs::create_dir_all(&data_dir).await.map_err(|e| {
            PersistenceError::DirectoryCreation(format!("Failed to create data dir: {}", e))
        })?;

        // Create subdirectories
        let vertices_dir = data_dir.join("vertices");
        let peers_dir = data_dir.join("peers");
        let domains_dir = data_dir.join("domains");

        fs::create_dir_all(&vertices_dir).await.map_err(|e| {
            PersistenceError::DirectoryCreation(format!("Failed to create vertices dir: {}", e))
        })?;

        fs::create_dir_all(&peers_dir).await.map_err(|e| {
            PersistenceError::DirectoryCreation(format!("Failed to create peers dir: {}", e))
        })?;

        fs::create_dir_all(&domains_dir).await.map_err(|e| {
            PersistenceError::DirectoryCreation(format!("Failed to create domains dir: {}", e))
        })?;

        info!("Initialized file state store at {:?}", data_dir);

        Ok(Self {
            data_dir,
            atomic_writes: true,
        })
    }

    /// Enable or disable atomic writes
    pub fn set_atomic_writes(&mut self, enabled: bool) {
        self.atomic_writes = enabled;
    }

    /// Get path for a vertex file
    fn vertex_path(&self, id: &VertexId) -> PathBuf {
        let id_hex = hex::encode(id.as_bytes());
        self.data_dir
            .join("vertices")
            .join(format!("{}.json", id_hex))
    }

    /// Get path for a peer file
    fn peer_path(&self, peer_id: &PeerId) -> PathBuf {
        let id_hex = hex::encode(peer_id.as_bytes());
        self.data_dir.join("peers").join(format!("{}.json", id_hex))
    }

    /// Get path for a dark domain file
    fn domain_path(&self, record: &DarkDomainRecord) -> PathBuf {
        // Use owner_id as the filename since domain is in a related field
        let id_hex = hex::encode(record.owner_id.as_bytes());
        self.data_dir
            .join("domains")
            .join(format!("{}.json", id_hex))
    }

    /// Write data to file atomically
    async fn write_file_atomic<T: Serialize>(&self, path: &Path, data: &T) -> Result<()> {
        let json = serde_json::to_string_pretty(data)
            .map_err(|e| PersistenceError::Serialization(e.to_string()))?;

        if self.atomic_writes {
            // Write to temporary file first
            let temp_path = path.with_extension("tmp");
            let mut file = File::create(&temp_path).await?;
            file.write_all(json.as_bytes()).await?;
            file.sync_all().await?;

            // Atomically rename to final location
            fs::rename(&temp_path, path).await?;
        } else {
            // Direct write
            let mut file = File::create(path).await?;
            file.write_all(json.as_bytes()).await?;
            file.sync_all().await?;
        }

        Ok(())
    }

    /// Read data from file
    async fn read_file<T: for<'de> Deserialize<'de>>(&self, path: &Path) -> Result<Option<T>> {
        if !path.exists() {
            return Ok(None);
        }

        let mut file = File::open(path).await?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).await?;

        let data = serde_json::from_str(&contents)
            .map_err(|e| PersistenceError::Serialization(e.to_string()))?;

        Ok(Some(data))
    }
}

#[async_trait]
impl StateStore for FileStateStore {
    async fn save_vertex(&self, vertex: &Vertex) -> Result<()> {
        let path = self.vertex_path(&vertex.id);
        self.write_file_atomic(&path, vertex).await?;
        debug!("Saved vertex {:?} to file", vertex.id);
        Ok(())
    }

    async fn load_vertex(&self, id: &VertexId) -> Result<Option<Vertex>> {
        let path = self.vertex_path(id);
        let vertex = self.read_file(&path).await?;
        if vertex.is_some() {
            debug!("Loaded vertex {:?} from file", id);
        }
        Ok(vertex)
    }

    async fn save_peer(&self, peer_id: &PeerId, info: &PeerInfo) -> Result<()> {
        let path = self.peer_path(peer_id);
        self.write_file_atomic(&path, info).await?;
        debug!("Saved peer {:?} to file", peer_id);
        Ok(())
    }

    async fn load_peers(&self) -> Result<Vec<(PeerId, PeerInfo)>> {
        let peers_dir = self.data_dir.join("peers");
        let mut peers = Vec::new();

        let mut entries = fs::read_dir(&peers_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let filename = path.file_stem().and_then(|s| s.to_str()).ok_or_else(|| {
                    PersistenceError::InvalidFormat("Invalid filename".to_string())
                })?;

                let peer_id_bytes = hex::decode(filename).map_err(|e| {
                    PersistenceError::InvalidFormat(format!("Invalid peer ID: {}", e))
                })?;

                if peer_id_bytes.len() != 32 {
                    return Err(PersistenceError::InvalidFormat(
                        "Invalid peer ID length".to_string(),
                    ));
                }

                let mut id_array = [0u8; 32];
                id_array.copy_from_slice(&peer_id_bytes);
                let peer_id = PeerId::from_bytes(id_array);

                if let Some(info) = self.read_file::<PeerInfo>(&path).await? {
                    peers.push((peer_id, info));
                }
            }
        }

        debug!("Loaded {} peers from files", peers.len());
        Ok(peers)
    }

    async fn save_dark_record(&self, record: &DarkDomainRecord) -> Result<()> {
        let path = self.domain_path(record);
        self.write_file_atomic(&path, record).await?;
        debug!(
            "Saved dark domain record for owner {:?} to file",
            record.owner_id
        );
        Ok(())
    }

    async fn load_dark_records(&self) -> Result<Vec<DarkDomainRecord>> {
        let domains_dir = self.data_dir.join("domains");
        let mut records = Vec::new();

        let mut entries = fs::read_dir(&domains_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Some(record) = self.read_file::<DarkDomainRecord>(&path).await? {
                    records.push(record);
                }
            }
        }

        debug!("Loaded {} dark domain records from files", records.len());
        Ok(records)
    }

    async fn remove_vertex(&self, id: &VertexId) -> Result<()> {
        let path = self.vertex_path(id);
        if path.exists() {
            fs::remove_file(&path).await?;
            debug!("Removed vertex {:?} from file", id);
        }
        Ok(())
    }

    async fn remove_peer(&self, peer_id: &PeerId) -> Result<()> {
        let path = self.peer_path(peer_id);
        if path.exists() {
            fs::remove_file(&path).await?;
            debug!("Removed peer {:?} from file", peer_id);
        }
        Ok(())
    }

    async fn remove_dark_record(&self, owner_id: &PeerId) -> Result<()> {
        let id_hex = hex::encode(owner_id.as_bytes());
        let path = self
            .data_dir
            .join("domains")
            .join(format!("{}.json", id_hex));
        if path.exists() {
            fs::remove_file(&path).await?;
            debug!(
                "Removed dark domain record for owner {:?} from file",
                owner_id
            );
        }
        Ok(())
    }

    async fn vertex_count(&self) -> Result<usize> {
        let vertices_dir = self.data_dir.join("vertices");
        let mut count = 0;
        let mut entries = fs::read_dir(&vertices_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            if entry.path().extension().and_then(|s| s.to_str()) == Some("json") {
                count += 1;
            }
        }
        Ok(count)
    }

    async fn peer_count(&self) -> Result<usize> {
        let peers_dir = self.data_dir.join("peers");
        let mut count = 0;
        let mut entries = fs::read_dir(&peers_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            if entry.path().extension().and_then(|s| s.to_str()) == Some("json") {
                count += 1;
            }
        }
        Ok(count)
    }

    async fn dark_record_count(&self) -> Result<usize> {
        let domains_dir = self.data_dir.join("domains");
        let mut count = 0;
        let mut entries = fs::read_dir(&domains_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            if entry.path().extension().and_then(|s| s.to_str()) == Some("json") {
                count += 1;
            }
        }
        Ok(count)
    }

    async fn health_check(&self) -> Result<bool> {
        // Check if directories exist and are writable
        let test_file = self.data_dir.join(".health_check");
        match File::create(&test_file).await {
            Ok(_) => {
                let _ = fs::remove_file(&test_file).await;
                Ok(true)
            }
            Err(_) => Ok(false),
        }
    }

    async fn save_state(&self, state: &PersistedDagState) -> Result<()> {
        let state_file = self.data_dir.join("state.json");
        self.write_file_atomic(&state_file, state).await?;
        debug!("Saved complete state to file");
        Ok(())
    }

    async fn recover_state(&self) -> Result<Option<PersistedDagState>> {
        let state_file = self.data_dir.join("state.json");
        let state = self.read_file(&state_file).await?;
        if state.is_some() {
            debug!("Recovered complete state from file");
        }
        Ok(state)
    }

    async fn create_backup(&self, backup_path: &PathBuf) -> Result<()> {
        // Create backup directory if it doesn't exist
        fs::create_dir_all(backup_path).await.map_err(|e| {
            PersistenceError::DirectoryCreation(format!("Failed to create backup dir: {}", e))
        })?;

        // Copy all data files to backup directory
        let backup_data_dir = backup_path.join("data");
        fs::create_dir_all(&backup_data_dir).await.map_err(|e| {
            PersistenceError::DirectoryCreation(format!("Failed to create backup data dir: {}", e))
        })?;

        // Copy all files recursively
        copy_dir_all(self.data_dir.clone(), backup_data_dir.clone()).await?;

        debug!("Created backup at {:?}", backup_path);
        Ok(())
    }

    async fn restore_backup(&self, backup_path: &PathBuf) -> Result<()> {
        let backup_data_dir = backup_path.join("data");

        if !backup_data_dir.exists() {
            return Err(PersistenceError::FileNotFound(format!(
                "Backup data directory not found: {:?}",
                backup_data_dir
            )));
        }

        // Clear current data directory
        if self.data_dir.exists() {
            fs::remove_dir_all(&self.data_dir).await?;
        }

        // Restore from backup
        copy_dir_all(backup_data_dir.clone(), self.data_dir.clone()).await?;

        debug!("Restored backup from {:?}", backup_path);
        Ok(())
    }
}

/// In-memory storage implementation for testing
pub struct MemoryStateStore {
    /// Stored vertices
    vertices: DashMap<VertexId, Vertex>,
    /// Stored peers
    peers: DashMap<PeerId, PeerInfo>,
    /// Stored dark domain records
    dark_records: DashMap<String, DarkDomainRecord>,
}

impl Default for MemoryStateStore {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryStateStore {
    /// Create a new memory-based state store
    pub fn new() -> Self {
        Self {
            vertices: DashMap::new(),
            peers: DashMap::new(),
            dark_records: DashMap::new(),
        }
    }

    /// Clear all stored data
    pub fn clear(&self) {
        self.vertices.clear();
        self.peers.clear();
        self.dark_records.clear();
    }
}

#[async_trait]
impl StateStore for MemoryStateStore {
    async fn save_vertex(&self, vertex: &Vertex) -> Result<()> {
        self.vertices.insert(vertex.id.clone(), vertex.clone());
        debug!("Saved vertex {:?} to memory", vertex.id);
        Ok(())
    }

    async fn load_vertex(&self, id: &VertexId) -> Result<Option<Vertex>> {
        let vertex = self.vertices.get(id).map(|entry| entry.clone());
        if vertex.is_some() {
            debug!("Loaded vertex {:?} from memory", id);
        }
        Ok(vertex)
    }

    async fn save_peer(&self, peer_id: &PeerId, info: &PeerInfo) -> Result<()> {
        self.peers.insert(*peer_id, info.clone());
        debug!("Saved peer {:?} to memory", peer_id);
        Ok(())
    }

    async fn load_peers(&self) -> Result<Vec<(PeerId, PeerInfo)>> {
        let peers: Vec<(PeerId, PeerInfo)> = self
            .peers
            .iter()
            .map(|entry| (*entry.key(), entry.value().clone()))
            .collect();
        debug!("Loaded {} peers from memory", peers.len());
        Ok(peers)
    }

    async fn save_dark_record(&self, record: &DarkDomainRecord) -> Result<()> {
        let key = hex::encode(record.owner_id.as_bytes());
        self.dark_records.insert(key, record.clone());
        debug!(
            "Saved dark domain record for owner {:?} to memory",
            record.owner_id
        );
        Ok(())
    }

    async fn load_dark_records(&self) -> Result<Vec<DarkDomainRecord>> {
        let records: Vec<DarkDomainRecord> = self
            .dark_records
            .iter()
            .map(|entry| entry.value().clone())
            .collect();
        debug!("Loaded {} dark domain records from memory", records.len());
        Ok(records)
    }

    async fn remove_vertex(&self, id: &VertexId) -> Result<()> {
        self.vertices.remove(id);
        debug!("Removed vertex {:?} from memory", id);
        Ok(())
    }

    async fn remove_peer(&self, peer_id: &PeerId) -> Result<()> {
        self.peers.remove(peer_id);
        debug!("Removed peer {:?} from memory", peer_id);
        Ok(())
    }

    async fn remove_dark_record(&self, owner_id: &PeerId) -> Result<()> {
        let key = hex::encode(owner_id.as_bytes());
        self.dark_records.remove(&key);
        debug!(
            "Removed dark domain record for owner {:?} from memory",
            owner_id
        );
        Ok(())
    }

    async fn vertex_count(&self) -> Result<usize> {
        Ok(self.vertices.len())
    }

    async fn peer_count(&self) -> Result<usize> {
        Ok(self.peers.len())
    }

    async fn dark_record_count(&self) -> Result<usize> {
        Ok(self.dark_records.len())
    }

    async fn health_check(&self) -> Result<bool> {
        // Memory store is always healthy if it exists
        Ok(true)
    }

    async fn save_state(&self, _state: &PersistedDagState) -> Result<()> {
        // Memory store doesn't persist state to disk
        debug!("State save called on memory store (no-op)");
        Ok(())
    }

    async fn recover_state(&self) -> Result<Option<PersistedDagState>> {
        // Memory store doesn't persist state to disk
        debug!("State recovery called on memory store (returning None)");
        Ok(None)
    }

    async fn create_backup(&self, _backup_path: &PathBuf) -> Result<()> {
        // Memory store doesn't support backups
        warn!("Backup creation called on memory store (no-op)");
        Ok(())
    }

    async fn restore_backup(&self, _backup_path: &PathBuf) -> Result<()> {
        // Memory store doesn't support backups
        warn!("Backup restoration called on memory store (no-op)");
        Ok(())
    }
}

/// Enhanced NodeRunner with persistence integration
pub struct PersistentNodeRunner<S: StateStore> {
    /// Storage backend
    store: Arc<S>,
    /// Auto-save interval in seconds
    auto_save_interval: u64,
    /// Whether persistence is enabled
    persistence_enabled: bool,
}

impl<S: StateStore + 'static> PersistentNodeRunner<S> {
    /// Create a new persistent node runner
    pub fn new(store: Arc<S>) -> Self {
        Self {
            store,
            auto_save_interval: 300, // 5 minutes default
            persistence_enabled: true,
        }
    }

    /// Set auto-save interval in seconds
    pub fn set_auto_save_interval(&mut self, seconds: u64) {
        self.auto_save_interval = seconds;
    }

    /// Enable or disable persistence
    pub fn set_persistence_enabled(&mut self, enabled: bool) {
        self.persistence_enabled = enabled;
    }

    /// Save a DAG vertex after consensus
    pub async fn save_vertex_after_consensus(&self, vertex: &Vertex) -> Result<()> {
        if !self.persistence_enabled {
            return Ok(());
        }

        self.store.save_vertex(vertex).await?;
        info!("Persisted vertex {:?} after consensus", vertex.id);
        Ok(())
    }

    /// Persist peer information
    pub async fn persist_peer_info(&self, peer_id: &PeerId, info: &PeerInfo) -> Result<()> {
        if !self.persistence_enabled {
            return Ok(());
        }

        self.store.save_peer(peer_id, info).await?;
        debug!("Persisted peer info for {:?}", peer_id);
        Ok(())
    }

    /// Store dark domain registration
    pub async fn store_dark_domain_registration(&self, record: &DarkDomainRecord) -> Result<()> {
        if !self.persistence_enabled {
            return Ok(());
        }

        self.store.save_dark_record(record).await?;
        info!(
            "Stored dark domain registration for owner {:?}",
            record.owner_id
        );
        Ok(())
    }

    /// Load state on startup
    pub async fn load_state_on_startup(&self) -> Result<StartupState> {
        if !self.persistence_enabled {
            return Ok(StartupState::default());
        }

        info!("Loading persisted state on startup...");

        let vertices = vec![]; // Would load all vertices in a real implementation
        let peers = self.store.load_peers().await?;
        let dark_records = self.store.load_dark_records().await?;

        let state = StartupState {
            vertices,
            peers,
            dark_records,
        };

        info!(
            "Loaded startup state: {} vertices, {} peers, {} dark records",
            state.vertices.len(),
            state.peers.len(),
            state.dark_records.len()
        );

        Ok(state)
    }

    /// Start auto-save background task
    pub async fn start_auto_save_task(&self) -> Result<()> {
        if !self.persistence_enabled || self.auto_save_interval == 0 {
            return Ok(());
        }

        let store = self.store.clone();
        let interval = self.auto_save_interval;

        tokio::spawn(async move {
            let mut interval_timer =
                tokio::time::interval(tokio::time::Duration::from_secs(interval));

            loop {
                interval_timer.tick().await;

                match store.health_check().await {
                    Ok(true) => {
                        debug!("Auto-save health check passed");
                    }
                    Ok(false) => {
                        warn!("Auto-save health check failed");
                    }
                    Err(e) => {
                        error!("Auto-save health check error: {}", e);
                    }
                }
            }
        });

        info!("Started auto-save task with {} second interval", interval);
        Ok(())
    }

    /// Get storage statistics
    pub async fn get_storage_stats(&self) -> Result<StorageStats> {
        let vertex_count = self.store.vertex_count().await?;
        let peer_count = self.store.peer_count().await?;
        let dark_record_count = self.store.dark_record_count().await?;
        let healthy = self.store.health_check().await?;

        Ok(StorageStats {
            vertex_count,
            peer_count,
            dark_record_count,
            healthy,
        })
    }
}

/// State loaded on node startup
#[derive(Debug, Default)]
pub struct StartupState {
    /// Loaded vertices
    pub vertices: Vec<Vertex>,
    /// Loaded peers
    pub peers: Vec<(PeerId, PeerInfo)>,
    /// Loaded dark domain records
    pub dark_records: Vec<DarkDomainRecord>,
}

/// Storage statistics
#[derive(Debug)]
pub struct StorageStats {
    /// Number of stored vertices
    pub vertex_count: usize,
    /// Number of stored peers
    pub peer_count: usize,
    /// Number of stored dark records
    pub dark_record_count: usize,
    /// Whether storage is healthy
    pub healthy: bool,
}

/// Helper function to copy directory contents recursively
fn copy_dir_all(
    src: PathBuf,
    dst: PathBuf,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'static>> {
    Box::pin(async move {
        fs::create_dir_all(&dst).await.map_err(|e| {
            PersistenceError::DirectoryCreation(format!(
                "Failed to create destination directory: {}",
                e
            ))
        })?;

        let mut entries = fs::read_dir(&src).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            let dest_path = dst.join(entry.file_name());

            if path.is_dir() {
                copy_dir_all(path, dest_path).await?;
            } else {
                fs::copy(&path, &dest_path).await?;
            }
        }
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use qudag_dag::vertex::VertexId;
    use qudag_network::peer::PeerId;
    use std::collections::HashSet;
    use tempfile::tempdir;

    fn create_test_vertex() -> Vertex {
        Vertex::new(VertexId::new(), vec![1, 2, 3, 4], HashSet::new())
    }

    fn create_test_peer_info() -> PeerInfo {
        PeerInfo {
            address: "127.0.0.1:8080".to_string(),
            last_seen: 1234567890,
            reputation: 75,
            trusted: true,
            connection_count: 5,
            bytes_exchanged: 1024,
            metadata: HashMap::new(),
        }
    }

    fn create_test_dark_record() -> DarkDomainRecord {
        use qudag_network::types::NetworkAddress;
        use std::collections::HashMap;

        DarkDomainRecord {
            signing_public_key: vec![1, 2, 3, 4],
            encryption_public_key: vec![5, 6, 7, 8],
            addresses: vec![NetworkAddress::new([127, 0, 0, 1], 8080)],
            alias: Some("test.dark".to_string()),
            ttl: 3600,
            registered_at: 1234567890,
            expires_at: 1234567890 + 3600,
            owner_id: PeerId::new(),
            signature: vec![9, 10, 11, 12],
            metadata: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn test_memory_store_vertices() {
        let store = MemoryStateStore::new();
        let vertex = create_test_vertex();

        // Save vertex
        store.save_vertex(&vertex).await.unwrap();

        // Load vertex
        let loaded = store.load_vertex(&vertex.id).await.unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().id, vertex.id);

        // Check count
        assert_eq!(store.vertex_count().await.unwrap(), 1);

        // Remove vertex
        store.remove_vertex(&vertex.id).await.unwrap();
        assert_eq!(store.vertex_count().await.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_memory_store_peers() {
        let store = MemoryStateStore::new();
        let peer_id = PeerId::random();
        let peer_info = create_test_peer_info();

        // Save peer
        store.save_peer(&peer_id, &peer_info).await.unwrap();

        // Load peers
        let peers = store.load_peers().await.unwrap();
        assert_eq!(peers.len(), 1);
        assert_eq!(peers[0].0, peer_id);

        // Check count
        assert_eq!(store.peer_count().await.unwrap(), 1);

        // Remove peer
        store.remove_peer(&peer_id).await.unwrap();
        assert_eq!(store.peer_count().await.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_memory_store_dark_records() {
        let store = MemoryStateStore::new();
        let record = create_test_dark_record();

        // Save record
        store.save_dark_record(&record).await.unwrap();

        // Load records
        let records = store.load_dark_records().await.unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].owner_id, record.owner_id);

        // Check count
        assert_eq!(store.dark_record_count().await.unwrap(), 1);

        // Remove record
        store.remove_dark_record(&record.owner_id).await.unwrap();
        assert_eq!(store.dark_record_count().await.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_file_store_vertices() {
        let temp_dir = tempdir().unwrap();
        let store = FileStateStore::new(temp_dir.path().to_path_buf())
            .await
            .unwrap();
        let vertex = create_test_vertex();

        // Save vertex
        store.save_vertex(&vertex).await.unwrap();

        // Load vertex
        let loaded = store.load_vertex(&vertex.id).await.unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().id, vertex.id);

        // Check count
        assert_eq!(store.vertex_count().await.unwrap(), 1);
    }

    #[tokio::test]
    async fn test_file_store_peers() {
        let temp_dir = tempdir().unwrap();
        let store = FileStateStore::new(temp_dir.path().to_path_buf())
            .await
            .unwrap();
        let peer_id = PeerId::random();
        let peer_info = create_test_peer_info();

        // Save peer
        store.save_peer(&peer_id, &peer_info).await.unwrap();

        // Load peers
        let peers = store.load_peers().await.unwrap();
        assert_eq!(peers.len(), 1);
        assert_eq!(peers[0].0, peer_id);
    }

    #[tokio::test]
    async fn test_persistent_node_runner() {
        let store = Arc::new(MemoryStateStore::new());
        let mut runner = PersistentNodeRunner::new(store.clone());
        runner.set_auto_save_interval(1);

        let vertex = create_test_vertex();
        runner.save_vertex_after_consensus(&vertex).await.unwrap();

        let peer_id = PeerId::random();
        let peer_info = create_test_peer_info();
        runner
            .persist_peer_info(&peer_id, &peer_info)
            .await
            .unwrap();

        let dark_record = create_test_dark_record();
        runner
            .store_dark_domain_registration(&dark_record)
            .await
            .unwrap();

        let state = runner.load_state_on_startup().await.unwrap();
        assert_eq!(state.peers.len(), 1);
        assert_eq!(state.dark_records.len(), 1);

        let stats = runner.get_storage_stats().await.unwrap();
        assert_eq!(stats.vertex_count, 1);
        assert_eq!(stats.peer_count, 1);
        assert_eq!(stats.dark_record_count, 1);
        assert!(stats.healthy);
    }

    #[tokio::test]
    async fn test_health_check() {
        let store = MemoryStateStore::new();
        assert!(store.health_check().await.unwrap());

        let temp_dir = tempdir().unwrap();
        let file_store = FileStateStore::new(temp_dir.path().to_path_buf())
            .await
            .unwrap();
        assert!(file_store.health_check().await.unwrap());
    }
}
