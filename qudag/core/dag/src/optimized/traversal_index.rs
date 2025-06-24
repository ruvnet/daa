//! High-performance traversal index for DAG operations

use crate::vertex::{Vertex, VertexId};
use crate::dag::DAG;
use dashmap::DashMap;
use parking_lot::RwLock;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::algo::{toposort, dijkstra};
use petgraph::visit::EdgeRef;

/// Traversal index for efficient DAG operations
pub struct TraversalIndex {
    /// Ancestor index: vertex -> set of ancestors
    ancestor_index: Arc<DashMap<VertexId, HashSet<VertexId>>>,
    /// Descendant index: vertex -> set of descendants
    descendant_index: Arc<DashMap<VertexId, HashSet<VertexId>>>,
    /// Depth index: vertex -> depth from genesis
    depth_index: Arc<DashMap<VertexId, u32>>,
    /// Children index: vertex -> direct children
    children_index: Arc<DashMap<VertexId, HashSet<VertexId>>>,
    /// Tip index: set of current tips
    tip_index: Arc<RwLock<HashSet<VertexId>>>,
    /// Common ancestor cache
    common_ancestor_cache: Arc<DashMap<(VertexId, VertexId), Option<VertexId>>>,
    /// Path cache for frequently accessed paths
    path_cache: Arc<DashMap<(VertexId, VertexId), Vec<VertexId>>>,
    /// Statistics
    stats: IndexStats,
}

/// Index statistics
#[derive(Debug, Default)]
struct IndexStats {
    /// Total indexed vertices
    indexed_vertices: AtomicU64,
    /// Cache hits
    cache_hits: AtomicU64,
    /// Cache misses
    cache_misses: AtomicU64,
}

impl TraversalIndex {
    /// Create a new traversal index
    pub fn new() -> Self {
        Self {
            ancestor_index: Arc::new(DashMap::new()),
            descendant_index: Arc::new(DashMap::new()),
            depth_index: Arc::new(DashMap::new()),
            children_index: Arc::new(DashMap::new()),
            tip_index: Arc::new(RwLock::new(HashSet::new())),
            common_ancestor_cache: Arc::new(DashMap::with_capacity(10000)),
            path_cache: Arc::new(DashMap::with_capacity(1000)),
            stats: IndexStats::default(),
        }
    }

    /// Add a vertex to the index
    pub fn add_vertex(&self, vertex: &Vertex) {
        let vertex_id = vertex.id.clone();
        
        // Initialize collections
        self.ancestor_index.insert(vertex_id.clone(), HashSet::new());
        self.descendant_index.insert(vertex_id.clone(), HashSet::new());
        self.children_index.insert(vertex_id.clone(), HashSet::new());
        
        // Update parent-child relationships
        for parent_id in &vertex.parents {
            // Add vertex as child of parent
            self.children_index
                .entry(parent_id.clone())
                .or_insert_with(HashSet::new)
                .insert(vertex_id.clone());
            
            // Remove parent from tips
            self.tip_index.write().remove(parent_id);
        }
        
        // Add as tip (will be removed if it gets children)
        self.tip_index.write().insert(vertex_id.clone());
        
        // Update ancestor/descendant indexes
        self.update_ancestry_indexes(&vertex_id, &vertex.parents);
        
        // Update depth
        self.update_depth(&vertex_id, &vertex.parents);
        
        // Increment stats
        self.stats.indexed_vertices.fetch_add(1, Ordering::Relaxed);
    }

    /// Update ancestry indexes when adding a vertex
    fn update_ancestry_indexes(&self, vertex_id: &VertexId, parents: &[VertexId]) {
        let mut ancestors = HashSet::new();
        
        // Collect all ancestors from parents
        for parent_id in parents {
            ancestors.insert(parent_id.clone());
            
            // Add parent's ancestors
            if let Some(parent_ancestors) = self.ancestor_index.get(parent_id) {
                ancestors.extend(parent_ancestors.iter().cloned());
            }
        }
        
        // Store ancestors for this vertex
        self.ancestor_index.insert(vertex_id.clone(), ancestors.clone());
        
        // Update descendant index for all ancestors
        for ancestor_id in &ancestors {
            self.descendant_index
                .entry(ancestor_id.clone())
                .or_insert_with(HashSet::new)
                .insert(vertex_id.clone());
        }
    }

    /// Update depth index
    fn update_depth(&self, vertex_id: &VertexId, parents: &[VertexId]) {
        let depth = if parents.is_empty() {
            0 // Genesis vertex
        } else {
            // Depth is max parent depth + 1
            parents.iter()
                .filter_map(|p| self.depth_index.get(p))
                .map(|d| *d)
                .max()
                .unwrap_or(0) + 1
        };
        
        self.depth_index.insert(vertex_id.clone(), depth);
    }

    /// Get ancestors of a vertex
    pub fn get_ancestors(&self, vertex_id: &VertexId) -> Option<HashSet<VertexId>> {
        self.stats.cache_hits.fetch_add(1, Ordering::Relaxed);
        self.ancestor_index.get(vertex_id).map(|v| v.clone())
    }

    /// Get descendants of a vertex
    pub fn get_descendants(&self, vertex_id: &VertexId) -> Option<HashSet<VertexId>> {
        self.stats.cache_hits.fetch_add(1, Ordering::Relaxed);
        self.descendant_index.get(vertex_id).map(|v| v.clone())
    }

    /// Get depth of a vertex
    pub fn get_depth(&self, vertex_id: &VertexId) -> Option<u32> {
        self.depth_index.get(vertex_id).map(|v| *v)
    }

    /// Get current tips
    pub fn get_tips(&self) -> HashSet<VertexId> {
        self.tip_index.read().clone()
    }

    /// Find common ancestor of two vertices (cached)
    pub fn find_common_ancestor(&self, a: &VertexId, b: &VertexId) -> Option<VertexId> {
        // Check cache first
        let cache_key = if a < b { (a.clone(), b.clone()) } else { (b.clone(), a.clone()) };
        
        if let Some(cached) = self.common_ancestor_cache.get(&cache_key) {
            self.stats.cache_hits.fetch_add(1, Ordering::Relaxed);
            return cached.clone();
        }
        
        self.stats.cache_misses.fetch_add(1, Ordering::Relaxed);
        
        // Compute common ancestor
        let ancestors_a = self.get_ancestors(a)?;
        let ancestors_b = self.get_ancestors(b)?;
        
        // Find intersection with highest depth
        let common: Vec<_> = ancestors_a.intersection(&ancestors_b).cloned().collect();
        
        let result = common.into_iter()
            .max_by_key(|v| self.get_depth(v).unwrap_or(0))
            .or_else(|| {
                // If no common ancestor in index, check if one is ancestor of other
                if ancestors_a.contains(b) {
                    Some(b.clone())
                } else if ancestors_b.contains(a) {
                    Some(a.clone())
                } else {
                    None
                }
            });
        
        // Cache result
        self.common_ancestor_cache.insert(cache_key, result.clone());
        
        result
    }

    /// Find shortest path between two vertices (cached)
    pub fn find_path(&self, from: &VertexId, to: &VertexId) -> Option<Vec<VertexId>> {
        let cache_key = (from.clone(), to.clone());
        
        // Check cache
        if let Some(cached) = self.path_cache.get(&cache_key) {
            self.stats.cache_hits.fetch_add(1, Ordering::Relaxed);
            return Some(cached.clone());
        }
        
        self.stats.cache_misses.fetch_add(1, Ordering::Relaxed);
        
        // Use BFS to find shortest path
        let path = self.bfs_path(from, to)?;
        
        // Cache result
        self.path_cache.insert(cache_key, path.clone());
        
        Some(path)
    }

    /// BFS to find path between vertices
    fn bfs_path(&self, from: &VertexId, to: &VertexId) -> Option<Vec<VertexId>> {
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        let mut parent_map = HashMap::new();
        
        queue.push_back(from.clone());
        visited.insert(from.clone());
        
        while let Some(current) = queue.pop_front() {
            if current == *to {
                // Reconstruct path
                let mut path = Vec::new();
                let mut node = current;
                
                while node != *from {
                    path.push(node.clone());
                    node = parent_map.get(&node)?.clone();
                }
                
                path.push(from.clone());
                path.reverse();
                
                return Some(path);
            }
            
            // Check children
            if let Some(children) = self.children_index.get(&current) {
                for child in children.iter() {
                    if !visited.contains(child) {
                        visited.insert(child.clone());
                        parent_map.insert(child.clone(), current.clone());
                        queue.push_back(child.clone());
                    }
                }
            }
        }
        
        None
    }

    /// Invalidate caches for a vertex
    pub fn invalidate(&self, vertex_id: &VertexId) {
        // Clear common ancestor cache entries involving this vertex
        self.common_ancestor_cache.retain(|(a, b), _| a != vertex_id && b != vertex_id);
        
        // Clear path cache entries involving this vertex
        self.path_cache.retain(|(from, to), _| from != vertex_id && to != vertex_id);
    }

    /// Get index statistics
    pub fn get_stats(&self) -> (u64, u64, u64) {
        (
            self.stats.indexed_vertices.load(Ordering::Relaxed),
            self.stats.cache_hits.load(Ordering::Relaxed),
            self.stats.cache_misses.load(Ordering::Relaxed),
        )
    }
}

/// Indexed DAG wrapper that uses TraversalIndex
pub struct IndexedDAG {
    /// Underlying DAG
    dag: Arc<RwLock<DAG>>,
    /// Traversal index
    index: Arc<TraversalIndex>,
    /// Graph representation for advanced algorithms
    graph: Arc<RwLock<DiGraph<VertexId, ()>>>,
    /// Node index mapping
    node_map: Arc<DashMap<VertexId, NodeIndex>>,
}

impl IndexedDAG {
    /// Create a new indexed DAG
    pub fn new(dag: DAG) -> Self {
        let index = Arc::new(TraversalIndex::new());
        let graph = Arc::new(RwLock::new(DiGraph::new()));
        let node_map = Arc::new(DashMap::new());
        
        // Build initial index
        let indexed_dag = Self {
            dag: Arc::new(RwLock::new(dag)),
            index: index.clone(),
            graph,
            node_map,
        };
        
        indexed_dag.rebuild_index();
        indexed_dag
    }

    /// Rebuild the entire index
    fn rebuild_index(&self) {
        let dag = self.dag.read();
        
        // Clear existing index
        self.index.ancestor_index.clear();
        self.index.descendant_index.clear();
        self.index.depth_index.clear();
        self.index.children_index.clear();
        self.index.tip_index.write().clear();
        
        // Rebuild from DAG
        // This is a placeholder - actual implementation would iterate through DAG vertices
    }

    /// Add vertex with automatic indexing
    pub fn add_vertex(&self, vertex: Vertex) -> Result<(), crate::error::DagError> {
        // Add to DAG
        self.dag.write().add_vertex(vertex.clone())?;
        
        // Update index
        self.index.add_vertex(&vertex);
        
        // Update graph
        let mut graph = self.graph.write();
        let node_idx = graph.add_node(vertex.id.clone());
        self.node_map.insert(vertex.id.clone(), node_idx);
        
        // Add edges from parents
        for parent_id in &vertex.parents {
            if let Some(parent_idx) = self.node_map.get(parent_id) {
                graph.add_edge(*parent_idx, node_idx, ());
            }
        }
        
        Ok(())
    }

    /// Get ancestors using index
    pub fn get_ancestors(&self, vertex_id: &VertexId) -> Option<HashSet<VertexId>> {
        self.index.get_ancestors(vertex_id)
    }

    /// Get descendants using index
    pub fn get_descendants(&self, vertex_id: &VertexId) -> Option<HashSet<VertexId>> {
        self.index.get_descendants(vertex_id)
    }

    /// Find common ancestor using index
    pub fn find_common_ancestor(&self, a: &VertexId, b: &VertexId) -> Option<VertexId> {
        self.index.find_common_ancestor(a, b)
    }

    /// Get topological ordering
    pub fn topological_sort(&self) -> Option<Vec<VertexId>> {
        let graph = self.graph.read();
        
        match toposort(&*graph, None) {
            Ok(nodes) => {
                let vertex_ids: Vec<_> = nodes.into_iter()
                    .filter_map(|idx| {
                        graph.node_weight(idx).cloned()
                    })
                    .collect();
                Some(vertex_ids)
            }
            Err(_) => None, // Cycle detected
        }
    }

    /// Find shortest path using Dijkstra
    pub fn shortest_path(&self, from: &VertexId, to: &VertexId) -> Option<Vec<VertexId>> {
        let from_idx = self.node_map.get(from)?;
        let to_idx = self.node_map.get(to)?;
        
        let graph = self.graph.read();
        let paths = dijkstra(&*graph, *from_idx, Some(*to_idx), |_| 1);
        
        if paths.contains_key(&to_idx) {
            // Reconstruct path
            self.index.find_path(from, to)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_traversal_index() {
        let index = TraversalIndex::new();
        
        // Create test vertices
        let v1 = Vertex::new(
            VertexId::from_bytes(vec![1]),
            vec![1],
            HashSet::new(), // Genesis
        );
        
        let v2 = Vertex::new(
            VertexId::from_bytes(vec![2]),
            vec![2],
            vec![v1.id.clone()].into_iter().collect(),
        );
        
        let v3 = Vertex::new(
            VertexId::from_bytes(vec![3]),
            vec![3],
            vec![v1.id.clone()].into_iter().collect(),
        );
        
        // Add vertices to index
        index.add_vertex(&v1);
        index.add_vertex(&v2);
        index.add_vertex(&v3);
        
        // Test ancestor queries
        assert_eq!(index.get_ancestors(&v1.id).unwrap().len(), 0);
        assert_eq!(index.get_ancestors(&v2.id).unwrap().len(), 1);
        assert!(index.get_ancestors(&v2.id).unwrap().contains(&v1.id));
        
        // Test descendant queries
        assert_eq!(index.get_descendants(&v1.id).unwrap().len(), 2);
        assert!(index.get_descendants(&v1.id).unwrap().contains(&v2.id));
        assert!(index.get_descendants(&v1.id).unwrap().contains(&v3.id));
        
        // Test depth
        assert_eq!(index.get_depth(&v1.id).unwrap(), 0);
        assert_eq!(index.get_depth(&v2.id).unwrap(), 1);
        assert_eq!(index.get_depth(&v3.id).unwrap(), 1);
        
        // Test tips
        let tips = index.get_tips();
        assert!(tips.contains(&v2.id));
        assert!(tips.contains(&v3.id));
        assert!(!tips.contains(&v1.id));
    }

    #[test]
    fn test_common_ancestor() {
        let index = TraversalIndex::new();
        
        // Create diamond structure: v1 -> v2, v3 -> v4
        let v1 = Vertex::new(
            VertexId::from_bytes(vec![1]),
            vec![1],
            HashSet::new(),
        );
        
        let v2 = Vertex::new(
            VertexId::from_bytes(vec![2]),
            vec![2],
            vec![v1.id.clone()].into_iter().collect(),
        );
        
        let v3 = Vertex::new(
            VertexId::from_bytes(vec![3]),
            vec![3],
            vec![v1.id.clone()].into_iter().collect(),
        );
        
        let v4 = Vertex::new(
            VertexId::from_bytes(vec![4]),
            vec![4],
            vec![v2.id.clone(), v3.id.clone()].into_iter().collect(),
        );
        
        index.add_vertex(&v1);
        index.add_vertex(&v2);
        index.add_vertex(&v3);
        index.add_vertex(&v4);
        
        // Test common ancestor
        let common = index.find_common_ancestor(&v2.id, &v3.id);
        assert_eq!(common, Some(v1.id.clone()));
    }
}