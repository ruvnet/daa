use blake3::Hash;
use dashmap::DashMap;
use lru::LruCache;
use parking_lot::RwLock;
use rayon::prelude::*;
use std::collections::{HashSet, VecDeque};
use std::num::NonZeroUsize;
use std::time::Instant;
use tracing::info;

use crate::{DagError, Edge, Node, Result};

/// Graph performance metrics
#[derive(Debug, Default)]
pub struct GraphMetrics {
    /// Average vertex processing time in nanoseconds
    pub avg_vertex_time_ns: u64,
    /// Number of vertices processed
    pub vertices_processed: u64,
    /// Cache hit rate for vertex lookups
    pub cache_hit_rate: f64,
    /// Memory usage in bytes
    pub memory_usage_bytes: usize,
    /// Number of pruned vertices
    pub pruned_vertices: u64,
}

/// Storage configuration for the DAG
#[derive(Debug, Clone)]
pub struct StorageConfig {
    /// Maximum number of vertices to keep in memory
    pub max_vertices: usize,
    /// Maximum number of edges to cache
    pub max_edges: usize,
    /// Pruning threshold (vertices to keep after pruning)
    pub pruning_threshold: usize,
    /// Maximum depth to keep in fast access cache
    pub cache_depth: usize,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            max_vertices: 100_000,
            max_edges: 500_000,
            pruning_threshold: 10_000,
            cache_depth: 1000,
        }
    }
}

/// Vertex storage with efficient caching and pruning
struct VertexStorage {
    /// Primary storage for all vertices
    vertices: DashMap<Hash, Node>,
    /// Fast access cache for recent vertices
    cache: RwLock<LruCache<Hash, Node>>,
    /// Pruning queue for old vertices
    pruning_queue: RwLock<VecDeque<Hash>>,
    /// Configuration
    config: StorageConfig,
    /// Cache statistics
    cache_hits: std::sync::atomic::AtomicU64,
    cache_misses: std::sync::atomic::AtomicU64,
}

impl VertexStorage {
    fn new(config: StorageConfig) -> Self {
        let cache_size = NonZeroUsize::new(config.cache_depth).unwrap();
        Self {
            vertices: DashMap::with_capacity(config.max_vertices),
            cache: RwLock::new(LruCache::new(cache_size)),
            pruning_queue: RwLock::new(VecDeque::new()),
            config,
            cache_hits: std::sync::atomic::AtomicU64::new(0),
            cache_misses: std::sync::atomic::AtomicU64::new(0),
        }
    }

    fn get(&self, hash: &Hash) -> Option<Node> {
        // Try cache first
        if let Some(node) = self.cache.write().get(hash) {
            self.cache_hits
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            return Some(node.clone());
        }

        // Try main storage
        if let Some(node) = self.vertices.get(hash) {
            let node = node.clone();
            // Update cache
            self.cache.write().put(*hash, node.clone());
            self.cache_misses
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            Some(node)
        } else {
            None
        }
    }

    fn insert(&self, hash: Hash, node: Node) -> Result<()> {
        // Check capacity
        if self.vertices.len() >= self.config.max_vertices {
            self.prune_old_vertices()?;
        }

        // Insert into main storage
        self.vertices.insert(hash, node.clone());

        // Update cache
        self.cache.write().put(hash, node);

        // Add to pruning queue
        self.pruning_queue.write().push_back(hash);

        Ok(())
    }

    fn prune_old_vertices(&self) -> Result<()> {
        let mut pruning_queue = self.pruning_queue.write();
        let target_size = self.config.pruning_threshold;
        let current_size = self.vertices.len();

        if current_size <= target_size {
            return Ok(());
        }

        let to_remove = current_size - target_size;
        let mut removed = 0;

        while removed < to_remove && !pruning_queue.is_empty() {
            if let Some(hash) = pruning_queue.pop_front() {
                // Only remove if vertex is in Final state
                if let Some(node) = self.vertices.get(&hash) {
                    if matches!(node.state(), crate::NodeState::Final) {
                        self.vertices.remove(&hash);
                        removed += 1;
                    }
                }
            }
        }

        info!("Pruned {} vertices from storage", removed);
        Ok(())
    }

    fn cache_hit_rate(&self) -> f64 {
        let hits = self.cache_hits.load(std::sync::atomic::Ordering::Relaxed);
        let misses = self.cache_misses.load(std::sync::atomic::Ordering::Relaxed);
        let total = hits + misses;
        if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64
        }
    }

    fn memory_usage(&self) -> usize {
        // Rough estimate: each node ~256 bytes + hash overhead
        self.vertices.len() * 256
    }
}

/// Represents the DAG data structure with high-performance concurrent access
pub struct Graph {
    /// Efficient vertex storage with caching
    storage: VertexStorage,
    /// Edges in the DAG with concurrent access
    edges: DashMap<Hash, HashSet<Edge>>,
    /// Performance metrics
    metrics: RwLock<GraphMetrics>,
}

impl Default for Graph {
    fn default() -> Self {
        Self::new()
    }
}

impl Graph {
    /// Creates a new empty DAG
    pub fn new() -> Self {
        Self::with_config(StorageConfig::default())
    }

    /// Creates a new Graph with specified initial capacity
    pub fn with_capacity(capacity: usize) -> Self {
        let config = StorageConfig {
            max_vertices: capacity,
            max_edges: capacity * 5,
            ..StorageConfig::default()
        };
        Self::with_config(config)
    }

    /// Creates a new Graph with custom storage configuration
    pub fn with_config(config: StorageConfig) -> Self {
        Self {
            storage: VertexStorage::new(config.clone()),
            edges: DashMap::with_capacity(config.max_edges),
            metrics: RwLock::new(GraphMetrics::default()),
        }
    }

    /// Returns true if the DAG contains no nodes
    pub fn is_empty(&self) -> bool {
        self.storage.vertices.is_empty()
    }

    /// Returns the number of nodes in the DAG
    pub fn len(&self) -> usize {
        self.storage.vertices.len()
    }

    /// Adds a new node to the DAG
    pub fn add_node(&self, node: Node) -> Result<()> {
        let start = Instant::now();
        let node_hash = node.hash();

        // Check if node already exists
        if self.storage.get(&node_hash).is_some() {
            return Err(DagError::NodeExists(format!("{:?}", node_hash)));
        }

        // Verify all parents exist concurrently
        let parents = node.parents();
        let missing_parent = parents
            .par_iter()
            .find_first(|parent| self.storage.get(parent).is_none());

        if let Some(parent) = missing_parent {
            return Err(DagError::MissingParent(format!("{:?}", parent)));
        }

        // Add node to storage
        self.storage.insert(node_hash, node)?;

        // Initialize edge set
        self.edges.entry(node_hash).or_default();

        // Add edges from parents in parallel
        if let Some(node) = self.storage.get(&node_hash) {
            let parents = node.parents();
            parents.par_iter().for_each(|parent| {
                let edge = Edge::new(*parent, node_hash);
                if let Some(mut parent_edges) = self.edges.get_mut(parent) {
                    parent_edges.insert(edge);
                }
            });
        }

        // Update metrics
        let elapsed = start.elapsed().as_nanos() as u64;
        let mut metrics = self.metrics.write();
        metrics.vertices_processed += 1;
        metrics.avg_vertex_time_ns =
            (metrics.avg_vertex_time_ns * (metrics.vertices_processed - 1) + elapsed)
                / metrics.vertices_processed;
        metrics.cache_hit_rate = self.storage.cache_hit_rate();
        metrics.memory_usage_bytes = self.storage.memory_usage();

        Ok(())
    }

    /// Returns a reference to a node by its hash
    pub fn get_node(&self, hash: &Hash) -> Option<Node> {
        self.storage.get(hash)
    }

    /// Returns all edges connected to a node
    pub fn get_edges(&self, hash: &Hash) -> Option<HashSet<Edge>> {
        // Fast concurrent lookup
        self.edges.get(hash).map(|edges| edges.clone())
    }

    /// Updates the state of a node
    pub fn update_node_state(&self, hash: &Hash, new_state: crate::node::NodeState) -> Result<()> {
        // Get node from storage
        let mut node = self
            .storage
            .get(hash)
            .ok_or_else(|| DagError::NodeNotFound(format!("{:?}", hash)))?;

        // Update state
        node.update_state(new_state)?;

        // Store updated node
        self.storage.insert(*hash, node)?;

        Ok(())
    }

    /// Checks if adding an edge would create a cycle
    #[allow(dead_code)]
    fn would_create_cycle(&self, from: &Hash, to: &Hash, visited: &mut HashSet<Hash>) -> bool {
        if from == to {
            return true;
        }

        if !visited.insert(*from) {
            return false;
        }

        if let Some(edges) = self.edges.get(from) {
            for edge in edges.iter() {
                let edge_to = edge.to();
                if self.would_create_cycle(&edge_to, to, visited) {
                    return true;
                }
            }
        }

        false
    }

    /// Triggers pruning of old vertices
    pub fn prune(&self) -> Result<()> {
        self.storage.prune_old_vertices()?;

        // Update metrics
        let mut metrics = self.metrics.write();
        metrics.pruned_vertices += 1;
        metrics.memory_usage_bytes = self.storage.memory_usage();

        Ok(())
    }

    /// Gets current performance metrics
    pub fn metrics(&self) -> GraphMetrics {
        let metrics_guard = self.metrics.read();
        GraphMetrics {
            avg_vertex_time_ns: metrics_guard.avg_vertex_time_ns,
            vertices_processed: metrics_guard.vertices_processed,
            cache_hit_rate: self.storage.cache_hit_rate(),
            memory_usage_bytes: self.storage.memory_usage(),
            pruned_vertices: metrics_guard.pruned_vertices,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::NodeState;

    #[test]
    fn test_graph_basic_operations() {
        let graph = Graph::new();
        assert!(graph.is_empty());
        assert_eq!(graph.len(), 0);

        // Create root node
        let root = Node::new(vec![1], vec![]);
        let root_hash = root.hash();
        assert!(graph.add_node(root).is_ok());
        assert!(!graph.is_empty());
        assert_eq!(graph.len(), 1);

        // Create child node
        let child = Node::new(vec![2], vec![root_hash]);
        let child_hash = child.hash();
        assert!(graph.add_node(child).is_ok());
        assert_eq!(graph.len(), 2);

        // Verify edges
        let root_edges = graph.get_edges(&root_hash).unwrap();
        assert_eq!(root_edges.len(), 1);
        assert!(root_edges.iter().any(|e| e.to() == child_hash));
    }

    #[test]
    fn test_node_state_updates() {
        let graph = Graph::new();
        let node = Node::new(vec![1], vec![]);
        let hash = node.hash();

        graph.add_node(node).unwrap();

        // Valid transition
        assert!(graph.update_node_state(&hash, NodeState::Verified).is_ok());

        let node = graph.get_node(&hash).unwrap();
        assert_eq!(node.state(), NodeState::Verified);

        // Invalid transition
        assert!(graph.update_node_state(&hash, NodeState::Pending).is_err());
    }

    #[test]
    fn test_cycle_prevention() {
        let graph = Graph::new();

        // Create nodes a -> b -> c
        let a = Node::new(vec![1], vec![]);
        let a_hash = a.hash();
        graph.add_node(a).unwrap();

        let b = Node::new(vec![2], vec![a_hash]);
        let b_hash = b.hash();
        graph.add_node(b).unwrap();

        let c = Node::new(vec![3], vec![b_hash]);
        let c_hash = c.hash();
        graph.add_node(c).unwrap();

        // Attempt to create cycle by adding edge c -> a
        let cycle_node = Node::new(vec![4], vec![c_hash, a_hash]);
        assert!(graph.add_node(cycle_node).is_ok());
    }

    #[test]
    fn test_missing_parent() {
        let graph = Graph::new();
        let missing_hash = blake3::hash(b"missing");
        let node = Node::new(vec![1], vec![missing_hash]);

        assert!(matches!(
            graph.add_node(node),
            Err(DagError::MissingParent(_))
        ));
    }
}
