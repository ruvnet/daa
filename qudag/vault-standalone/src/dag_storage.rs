//! DAG-based storage for vault secrets with optional QuDAG integration.

// For initial standalone publishing, we'll use String as VertexId
/// Simple vertex ID for standalone mode
pub type VertexId = String;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use tracing::{debug, info, trace};
use chrono::{DateTime, Utc};

use crate::{
    crypto::VaultCrypto,
    error::{VaultError, VaultResult},
    secret::{EncryptedSecret, SecretEntry},
};

/// DAG-based storage for vault secrets.
pub struct VaultDag {
    /// The root vertex ID of the vault.
    root_id: VertexId,
    /// Map of vertex IDs to encrypted secrets.
    secrets: HashMap<VertexId, EncryptedSecret>,
    /// Map of labels to vertex IDs for quick lookup.
    label_index: HashMap<String, VertexId>,
    /// Map of vertex IDs to their parent vertices (categories).
    parent_map: HashMap<VertexId, HashSet<VertexId>>,
    /// Map of vertex IDs to their child vertices.
    child_map: HashMap<VertexId, HashSet<VertexId>>,
    /// Map of vertex IDs to their creation timestamp.
    created_at_map: HashMap<VertexId, DateTime<Utc>>,
    /// Map of vertex IDs to their last update timestamp.
    updated_at_map: HashMap<VertexId, DateTime<Utc>>,
}

/// Serializable representation of the vault DAG.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedVaultDag {
    /// The root vertex ID in hex format.
    pub root_id: String,
    /// All vertices in the DAG.
    pub vertices: Vec<SerializedVertex>,
}

/// Serializable vertex containing encrypted secret data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedVertex {
    /// The vertex ID in hex format.
    pub id: String,
    /// Parent vertex IDs in hex format.
    pub parents: Vec<String>,
    /// The encrypted secret data.
    pub encrypted_secret: EncryptedSecret,
}

impl VaultDag {
    /// Create a new empty vault DAG.
    pub fn new() -> Self {
        let root_id = VertexId::new();
        let now = Utc::now();
        info!("Created new vault DAG with root ID: {:?}", root_id);
        
        let mut created_at_map = HashMap::new();
        let mut updated_at_map = HashMap::new();
        created_at_map.insert(root_id.clone(), now);
        updated_at_map.insert(root_id.clone(), now);
        
        Self {
            root_id,
            secrets: HashMap::new(),
            label_index: HashMap::new(),
            parent_map: HashMap::new(),
            child_map: HashMap::new(),
            created_at_map,
            updated_at_map,
        }
    }

    /// Add a secret to the vault DAG.
    pub fn add_secret(
        &mut self,
        secret: SecretEntry,
        crypto: &VaultCrypto,
        parent_labels: Vec<String>,
    ) -> VaultResult<VertexId> {
        // Check if label already exists
        if self.label_index.contains_key(&secret.label) {
            return Err(VaultError::Generic(format!(
                "Secret with label '{}' already exists",
                secret.label
            )));
        }

        // Encrypt the secret
        let serialized = bincode::serialize(&secret)?;
        let encrypted_data = crypto.encrypt(&serialized)?;
        
        let encrypted_secret = EncryptedSecret {
            encrypted_data,
            metadata: secret.metadata.clone(),
            label: secret.label.clone(),
        };

        // Create new vertex
        let vertex_id = VertexId::new();
        debug!("Adding secret '{}' with ID: {:?}", secret.label, vertex_id);

        // Find parent vertices
        let mut parent_ids = HashSet::new();
        if parent_labels.is_empty() {
            // If no parents specified, attach to root
            parent_ids.insert(self.root_id.clone());
        } else {
            for parent_label in &parent_labels {
                if let Some(parent_id) = self.label_index.get(parent_label) {
                    parent_ids.insert(parent_id.clone());
                } else {
                    // Create category node if it doesn't exist
                    let category_id = self.create_category(parent_label, crypto)?;
                    parent_ids.insert(category_id);
                }
            }
        }

        // Store label before moving secret
        let label = secret.label.clone();
        
        // Update maps
        let now = Utc::now();
        self.secrets.insert(vertex_id.clone(), encrypted_secret);
        self.label_index.insert(secret.label, vertex_id.clone());
        self.parent_map.insert(vertex_id.clone(), parent_ids.clone());
        self.created_at_map.insert(vertex_id.clone(), now);
        self.updated_at_map.insert(vertex_id.clone(), now);
        
        // Update child maps and parent timestamps
        for parent_id in &parent_ids {
            self.child_map
                .entry(parent_id.clone())
                .or_insert_with(HashSet::new)
                .insert(vertex_id.clone());
            self.updated_at_map.insert(parent_id.clone(), now);
        }

        info!("Successfully added secret with label '{}'", label);
        Ok(vertex_id)
    }

    /// Create a category node.
    fn create_category(&mut self, label: &str, crypto: &VaultCrypto) -> VaultResult<VertexId> {
        if let Some(existing_id) = self.label_index.get(label) {
            return Ok(existing_id.clone());
        }

        let category = SecretEntry::new(
            label.to_string(),
            String::new(), // Empty username for categories
            String::new(), // Empty password for categories
        );

        let serialized = bincode::serialize(&category)?;
        let encrypted_data = crypto.encrypt(&serialized)?;
        
        let encrypted_secret = EncryptedSecret {
            encrypted_data,
            metadata: category.metadata,
            label: label.to_string(),
        };

        let vertex_id = VertexId::new();
        let now = Utc::now();
        self.secrets.insert(vertex_id.clone(), encrypted_secret);
        self.label_index.insert(label.to_string(), vertex_id.clone());
        self.created_at_map.insert(vertex_id.clone(), now);
        self.updated_at_map.insert(vertex_id.clone(), now);
        
        // Categories attach to root by default
        let mut parents = HashSet::new();
        parents.insert(self.root_id.clone());
        self.parent_map.insert(vertex_id.clone(), parents);
        
        self.child_map
            .entry(self.root_id.clone())
            .or_insert_with(HashSet::new)
            .insert(vertex_id.clone());
        
        // Update root timestamp
        self.updated_at_map.insert(self.root_id.clone(), now);

        debug!("Created category '{}' with ID: {:?}", label, vertex_id);
        Ok(vertex_id)
    }

    /// Get a secret by its label.
    pub fn get_secret(
        &self,
        label: &str,
        crypto: &VaultCrypto,
    ) -> VaultResult<SecretEntry> {
        let vertex_id = self
            .label_index
            .get(label)
            .ok_or_else(|| VaultError::SecretNotFound(label.to_string()))?;

        let encrypted = self
            .secrets
            .get(vertex_id)
            .ok_or_else(|| VaultError::SecretNotFound(label.to_string()))?;

        let decrypted = crypto.decrypt(&encrypted.encrypted_data)?;
        let secret: SecretEntry = bincode::deserialize(&decrypted)?;
        
        // Update access time
        trace!("Retrieved secret '{}'", label);
        Ok(secret)
    }

    /// List all secrets, optionally filtered by category.
    pub fn list_secrets(&self, category: Option<&str>) -> VaultResult<Vec<String>> {
        if let Some(cat) = category {
            // List secrets under a specific category
            let category_id = self
                .label_index
                .get(cat)
                .ok_or_else(|| VaultError::Generic(format!("Category '{}' not found", cat)))?;

            let children = self.child_map.get(category_id);
            Ok(children
                .map(|ids| {
                    ids.iter()
                        .filter_map(|id| {
                            self.secrets
                                .get(id)
                                .map(|encrypted| encrypted.label.clone())
                        })
                        .collect()
                })
                .unwrap_or_default())
        } else {
            // List all secrets
            Ok(self
                .secrets
                .values()
                .filter(|s| !s.label.is_empty() && !s.encrypted_data.is_empty())
                .map(|s| s.label.clone())
                .collect())
        }
    }

    /// Update an existing secret.
    pub fn update_secret(
        &mut self,
        label: &str,
        new_secret: SecretEntry,
        crypto: &VaultCrypto,
    ) -> VaultResult<()> {
        let vertex_id = self
            .label_index
            .get(label)
            .ok_or_else(|| VaultError::SecretNotFound(label.to_string()))?
            .to_owned();

        // Encrypt the updated secret
        let serialized = bincode::serialize(&new_secret)?;
        let encrypted_data = crypto.encrypt(&serialized)?;
        
        let encrypted_secret = EncryptedSecret {
            encrypted_data,
            metadata: new_secret.metadata.clone(),
            label: new_secret.label.clone(),
        };

        self.secrets.insert(vertex_id.clone(), encrypted_secret);
        
        // Update label index if label changed
        if label != &new_secret.label {
            self.label_index.remove(label);
            self.label_index.insert(new_secret.label, vertex_id);
        }

        info!("Updated secret '{}'", label);
        Ok(())
    }

    /// Delete a secret from the vault.
    pub fn delete_secret(&mut self, label: &str) -> VaultResult<()> {
        let vertex_id = self
            .label_index
            .get(label)
            .ok_or_else(|| VaultError::SecretNotFound(label.to_string()))?
            .to_owned();

        // Remove from all maps
        self.secrets.remove(&vertex_id);
        self.label_index.remove(label);
        self.created_at_map.remove(&vertex_id);
        self.updated_at_map.remove(&vertex_id);
        
        // Remove from parent-child relationships
        if let Some(parents) = self.parent_map.remove(&vertex_id) {
            for parent_id in parents {
                if let Some(children) = self.child_map.get_mut(&parent_id) {
                    children.remove(&vertex_id);
                }
            }
        }

        // Remove as parent from any children
        if let Some(children) = self.child_map.remove(&vertex_id) {
            for child_id in children {
                if let Some(parents) = self.parent_map.get_mut(&child_id) {
                    parents.remove(&vertex_id);
                    // If child has no parents, attach to root
                    if parents.is_empty() {
                        parents.insert(self.root_id.clone());
                        self.child_map
                            .entry(self.root_id.clone())
                            .or_insert_with(HashSet::new)
                            .insert(child_id);
                    }
                }
            }
        }

        info!("Deleted secret '{}'", label);
        Ok(())
    }

    /// Find all descendants of a node.
    pub fn find_descendants(&self, id: &VertexId) -> VaultResult<HashSet<VertexId>> {
        let mut descendants = HashSet::new();
        let mut queue = VecDeque::new();
        
        if let Some(children) = self.child_map.get(id) {
            queue.extend(children.iter().cloned());
        } else {
            // It's valid for a node to have no children
            return Ok(descendants);
        }
        
        while let Some(node_id) = queue.pop_front() {
            if descendants.insert(node_id.clone()) {
                if let Some(children) = self.child_map.get(&node_id) {
                    queue.extend(children.iter().cloned());
                }
            }
        }
        
        trace!("Found {} descendants for node", descendants.len());
        Ok(descendants)
    }
    
    /// Find all ancestors of a node.
    pub fn find_ancestors(&self, id: &VertexId) -> VaultResult<HashSet<VertexId>> {
        let mut ancestors = HashSet::new();
        let mut queue = VecDeque::new();
        
        if let Some(parents) = self.parent_map.get(id) {
            queue.extend(parents.iter().cloned());
        } else {
            // It's valid for a node to have no parents (except root)
            return Ok(ancestors);
        }
        
        while let Some(node_id) = queue.pop_front() {
            if ancestors.insert(node_id.clone()) {
                if let Some(parents) = self.parent_map.get(&node_id) {
                    queue.extend(parents.iter().cloned());
                }
            }
        }
        
        trace!("Found {} ancestors for node", ancestors.len());
        Ok(ancestors)
    }
    
    /// Check if adding an edge would create a cycle.
    pub fn would_create_cycle(&self, from: &VertexId, to: &VertexId) -> VaultResult<bool> {
        // Check if 'to' is an ancestor of 'from'
        if from == to {
            return Ok(true);
        }
        
        let ancestors = self.find_ancestors(from)?;
        Ok(ancestors.contains(to))
    }
    
    /// List all nodes at a specific depth from root.
    pub fn list_at_depth(&self, depth: usize) -> Vec<VertexId> {
        let mut current_level = vec![self.root_id.clone()];
        
        for _ in 0..depth {
            let mut next_level = Vec::new();
            for node_id in current_level {
                if let Some(children) = self.child_map.get(&node_id) {
                    next_level.extend(children.iter().cloned());
                }
            }
            current_level = next_level;
            
            if current_level.is_empty() {
                break;
            }
        }
        
        debug!("Found {} nodes at depth {}", current_level.len(), depth);
        current_level
    }
    
    /// Get the creation timestamp of a node.
    pub fn get_created_at(&self, id: &VertexId) -> Option<&DateTime<Utc>> {
        self.created_at_map.get(id)
    }
    
    /// Get the last update timestamp of a node.
    pub fn get_updated_at(&self, id: &VertexId) -> Option<&DateTime<Utc>> {
        self.updated_at_map.get(id)
    }
    
    /// Get the total number of nodes including root.
    pub fn node_count(&self) -> usize {
        self.secrets.len() + 1 // +1 for root
    }
    
    /// Get all labeled nodes with their metadata.
    pub fn labeled_nodes(&self) -> Vec<(String, DateTime<Utc>, DateTime<Utc>)> {
        self.label_index
            .iter()
            .filter_map(|(label, id)| {
                match (self.created_at_map.get(id), self.updated_at_map.get(id)) {
                    (Some(created), Some(updated)) => {
                        Some((label.clone(), *created, *updated))
                    }
                    _ => None,
                }
            })
            .collect()
    }

    /// Serialize the vault DAG for storage.
    pub fn serialize(&self) -> VaultResult<SerializedVaultDag> {
        let mut vertices = Vec::new();

        for (vertex_id, encrypted_secret) in &self.secrets {
            let parents = self
                .parent_map
                .get(vertex_id)
                .map(|p| p.iter().map(|id| hex::encode(id.as_bytes())).collect())
                .unwrap_or_default();

            vertices.push(SerializedVertex {
                id: hex::encode(vertex_id.as_bytes()),
                parents,
                encrypted_secret: encrypted_secret.clone(),
            });
        }

        Ok(SerializedVaultDag {
            root_id: hex::encode(self.root_id.as_bytes()),
            vertices,
        })
    }

    /// Deserialize a vault DAG from storage.
    pub fn deserialize(data: SerializedVaultDag) -> VaultResult<Self> {
        // Convert hex string to bytes for VertexId
        let root_bytes = hex::decode(&data.root_id)
            .map_err(|_| VaultError::InvalidFormat("Invalid root ID".to_string()))?;
        let root_id = String::from_utf8(root_bytes.to_vec())
            .map_err(|e| VaultError::InvalidFormat(format!("Invalid root ID: {}", e)))?;

        let now = Utc::now();
        let mut created_at_map = HashMap::new();
        let mut updated_at_map = HashMap::new();
        created_at_map.insert(root_id.clone(), now);
        updated_at_map.insert(root_id.clone(), now);
        
        let mut dag = Self {
            root_id,
            secrets: HashMap::new(),
            label_index: HashMap::new(),
            parent_map: HashMap::new(),
            child_map: HashMap::new(),
            created_at_map,
            updated_at_map,
        };

        for vertex in data.vertices {
            let vertex_bytes = hex::decode(&vertex.id)
                .map_err(|_| VaultError::InvalidFormat("Invalid vertex ID".to_string()))?;
            let vertex_id = String::from_utf8(vertex_bytes.to_vec())
                .map_err(|e| VaultError::InvalidFormat(format!("Invalid vertex ID: {}", e)))?;

            let parent_ids: HashSet<VertexId> = vertex
                .parents
                .iter()
                .filter_map(|p| hex::decode(p).ok())
                .filter_map(|bytes| String::from_utf8(bytes.to_vec()).ok())
                .collect();

            dag.secrets.insert(vertex_id.clone(), vertex.encrypted_secret.clone());
            dag.label_index.insert(vertex.encrypted_secret.label.clone(), vertex_id.clone());
            dag.parent_map.insert(vertex_id.clone(), parent_ids.clone());
            
            // Add timestamps for deserialized nodes (using current time since we don't persist them)
            dag.created_at_map.insert(vertex_id.clone(), now);
            dag.updated_at_map.insert(vertex_id.clone(), now);

            // Rebuild child map
            for parent_id in &parent_ids {
                dag.child_map
                    .entry(parent_id.clone())
                    .or_insert_with(HashSet::new)
                    .insert(vertex_id.clone());
            }
        }

        Ok(dag)
    }
}

impl Default for VaultDag {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vault_dag_creation() {
        let dag = VaultDag::new();
        assert!(dag.secrets.is_empty());
        assert!(dag.label_index.is_empty());
    }

    #[test]
    fn test_add_and_get_secret() {
        let mut dag = VaultDag::new();
        let crypto = VaultCrypto::new().unwrap();
        
        let secret = SecretEntry::new(
            "test/secret".to_string(),
            "user".to_string(),
            "password".to_string(),
        );
        
        dag.add_secret(secret.clone(), &crypto, vec![]).unwrap();
        
        let retrieved = dag.get_secret("test/secret", &crypto).unwrap();
        assert_eq!(retrieved.label, secret.label);
        assert_eq!(retrieved.username, secret.username);
    }

    #[test]
    fn test_list_secrets() {
        let mut dag = VaultDag::new();
        let crypto = VaultCrypto::new().unwrap();
        
        // Add secrets
        for i in 1..=3 {
            let secret = SecretEntry::new(
                format!("secret{}", i),
                "user".to_string(),
                "password".to_string(),
            );
            dag.add_secret(secret, &crypto, vec![]).unwrap();
        }
        
        let secrets = dag.list_secrets(None).unwrap();
        assert_eq!(secrets.len(), 3);
        assert!(secrets.contains(&"secret1".to_string()));
        assert!(secrets.contains(&"secret2".to_string()));
        assert!(secrets.contains(&"secret3".to_string()));
    }

    #[test]
    fn test_dag_operations() {
        let mut dag = VaultDag::new();
        let crypto = VaultCrypto::new().unwrap();
        
        // Add category
        let category_id = dag.create_category("work", &crypto).unwrap();
        
        // Add secrets under category
        let secret1 = SecretEntry::new("email".to_string(), "user1".to_string(), "pass1".to_string());
        let secret2 = SecretEntry::new("vpn".to_string(), "user2".to_string(), "pass2".to_string());
        
        let id1 = dag.add_secret(secret1, &crypto, vec!["work".to_string()]).unwrap();
        let id2 = dag.add_secret(secret2, &crypto, vec!["work".to_string()]).unwrap();
        
        // Test descendants
        let descendants = dag.find_descendants(&category_id).unwrap();
        assert_eq!(descendants.len(), 2);
        assert!(descendants.contains(&id1));
        assert!(descendants.contains(&id2));
        
        // Test ancestors (should include category and root)
        let ancestors = dag.find_ancestors(&id1).unwrap();
        assert_eq!(ancestors.len(), 2); // category + root
        assert!(ancestors.contains(&category_id));
        assert!(ancestors.contains(&dag.root_id));
        
        // Test cycle detection
        assert!(dag.would_create_cycle(&id1, &category_id).unwrap()); // Would create cycle
        assert!(!dag.would_create_cycle(&category_id, &id1).unwrap()); // Would not create cycle
        
        // Test depth listing
        let depth_0 = dag.list_at_depth(0);
        assert_eq!(depth_0.len(), 1); // Only root
        
        let depth_1 = dag.list_at_depth(1);
        assert!(depth_1.contains(&category_id));
        
        let depth_2 = dag.list_at_depth(2);
        assert_eq!(depth_2.len(), 2);
        assert!(depth_2.contains(&id1));
        assert!(depth_2.contains(&id2));
    }

    #[test]
    fn test_timestamps() {
        let mut dag = VaultDag::new();
        let crypto = VaultCrypto::new().unwrap();
        
        let secret = SecretEntry::new("test".to_string(), "user".to_string(), "pass".to_string());
        let id = dag.add_secret(secret, &crypto, vec![]).unwrap();
        
        // Check timestamps exist
        assert!(dag.get_created_at(&id).is_some());
        assert!(dag.get_updated_at(&id).is_some());
        
        // Check labeled nodes include timestamps
        let labeled = dag.labeled_nodes();
        assert_eq!(labeled.len(), 1);
        assert_eq!(labeled[0].0, "test");
    }
}