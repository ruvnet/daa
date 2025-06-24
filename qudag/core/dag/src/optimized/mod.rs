//! Optimized DAG operations with caching and indexing

pub mod validation_cache;
pub mod traversal_index;

pub use validation_cache::{ValidationCache, ValidationResult};
pub use traversal_index::{TraversalIndex, IndexedDAG};