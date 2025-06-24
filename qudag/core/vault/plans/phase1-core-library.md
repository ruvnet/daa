# Phase 1: Core Library Implementation Details

## Overview
This document provides detailed implementation specifications for Phase 1 of the QuDAG Vault library, focusing on the core cryptographic and storage functionality.

## Module Structure

### 1. Cryptographic Module (`crypto/`)

#### 1.1 Key Derivation (`kdf.rs`)
```rust
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2
};
use zeroize::Zeroize;

pub const KDF_SALT_LEN: usize = 32;
pub const MASTER_KEY_LEN: usize = 32;

#[derive(Debug)]
pub struct KdfParams {
    pub memory_cost: u32,      // 256 MB
    pub time_cost: u32,        // 3 iterations
    pub parallelism: u32,      // 4 threads
}

impl Default for KdfParams {
    fn default() -> Self {
        Self {
            memory_cost: 256 * 1024,  // 256 MB
            time_cost: 3,
            parallelism: 4,
        }
    }
}

pub fn derive_master_key(
    password: &[u8],
    salt: &[u8; KDF_SALT_LEN],
    params: &KdfParams,
) -> Result<[u8; MASTER_KEY_LEN], CryptoError> {
    let argon2 = Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        argon2::Params::new(
            params.memory_cost,
            params.time_cost,
            params.parallelism,
            Some(MASTER_KEY_LEN),
        )?,
    );
    
    let mut output = [0u8; MASTER_KEY_LEN];
    argon2.hash_password_into(password, salt, &mut output)?;
    Ok(output)
}
```

#### 1.2 AEAD Encryption (`aead.rs`)
```rust
use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use zeroize::Zeroize;

pub const NONCE_LEN: usize = 12;
pub const TAG_LEN: usize = 16;

pub struct AeadCipher {
    cipher: Aes256Gcm,
}

impl AeadCipher {
    pub fn new(key: &[u8; 32]) -> Self {
        let key = Key::<Aes256Gcm>::from_slice(key);
        Self {
            cipher: Aes256Gcm::new(key),
        }
    }
    
    pub fn encrypt(&self, plaintext: &[u8], aad: Option<&[u8]>) -> Result<Vec<u8>, CryptoError> {
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        
        let ciphertext = if let Some(aad) = aad {
            self.cipher.encrypt(&nonce, aead::Payload {
                msg: plaintext,
                aad,
            })
        } else {
            self.cipher.encrypt(&nonce, plaintext)
        }?;
        
        // Format: [nonce(12)] || [ciphertext + tag]
        let mut result = Vec::with_capacity(NONCE_LEN + ciphertext.len());
        result.extend_from_slice(&nonce);
        result.extend_from_slice(&ciphertext);
        Ok(result)
    }
    
    pub fn decrypt(&self, ciphertext: &[u8], aad: Option<&[u8]>) -> Result<Vec<u8>, CryptoError> {
        if ciphertext.len() < NONCE_LEN + TAG_LEN {
            return Err(CryptoError::InvalidCiphertext);
        }
        
        let (nonce, ct) = ciphertext.split_at(NONCE_LEN);
        let nonce = Nonce::from_slice(nonce);
        
        let plaintext = if let Some(aad) = aad {
            self.cipher.decrypt(nonce, aead::Payload {
                msg: ct,
                aad,
            })
        } else {
            self.cipher.decrypt(nonce, ct)
        }?;
        
        Ok(plaintext)
    }
}

impl Drop for AeadCipher {
    fn drop(&mut self) {
        // Cipher key is already zeroized by aes-gcm
    }
}
```

#### 1.3 Post-Quantum KEM (`kem.rs`)
```rust
use qudag_crypto::kem::{KemKeyPair, KemPublicKey, KemSecretKey};
use qudag_crypto::kem::kyber::{encapsulate, decapsulate};

pub struct VaultKem {
    keypair: KemKeyPair,
}

impl VaultKem {
    pub fn generate() -> Result<Self, CryptoError> {
        let keypair = KemKeyPair::generate()?;
        Ok(Self { keypair })
    }
    
    pub fn from_keypair(keypair: KemKeyPair) -> Self {
        Self { keypair }
    }
    
    pub fn public_key(&self) -> &KemPublicKey {
        &self.keypair.public
    }
    
    pub fn encapsulate_for(&self, recipient: &KemPublicKey) -> Result<(Vec<u8>, [u8; 32]), CryptoError> {
        let (ciphertext, shared_secret) = encapsulate(recipient)?;
        Ok((ciphertext, shared_secret))
    }
    
    pub fn decapsulate(&self, ciphertext: &[u8]) -> Result<[u8; 32], CryptoError> {
        let shared_secret = decapsulate(&self.keypair.secret, ciphertext)?;
        Ok(shared_secret)
    }
}
```

### 2. Storage Module (`storage/`)

#### 2.1 File-based Storage (`file.rs`)
```rust
use std::fs::{File, OpenOptions};
use std::io::{Read, Write, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use bincode;
use zeroize::Zeroize;

const VAULT_MAGIC: &[u8; 8] = b"QUDAG-V1";
const HEADER_SIZE: usize = 512;

#[derive(Debug, Serialize, Deserialize)]
pub struct VaultHeader {
    pub magic: [u8; 8],
    pub version: u32,
    pub created_at: u64,
    pub last_modified: u64,
    pub kdf_params: KdfParams,
    pub salt: [u8; 32],
    pub encrypted_vault_key: Vec<u8>,
    pub kem_public_key: Option<Vec<u8>>,
    pub checksum: [u8; 32],
}

pub struct FileStorage {
    path: PathBuf,
    file: File,
    header: VaultHeader,
}

impl FileStorage {
    pub fn create(path: &Path, header: VaultHeader) -> Result<Self, StorageError> {
        if path.exists() {
            return Err(StorageError::AlreadyExists);
        }
        
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .read(true)
            .mode(0o600) // Owner read/write only
            .open(path)?;
        
        // Write header
        let header_bytes = bincode::serialize(&header)?;
        if header_bytes.len() > HEADER_SIZE {
            return Err(StorageError::HeaderTooLarge);
        }
        
        let mut padded_header = vec![0u8; HEADER_SIZE];
        padded_header[..header_bytes.len()].copy_from_slice(&header_bytes);
        
        file.write_all(&padded_header)?;
        file.sync_all()?;
        
        Ok(Self {
            path: path.to_owned(),
            file,
            header,
        })
    }
    
    pub fn open(path: &Path) -> Result<Self, StorageError> {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)?;
        
        // Read header
        let mut header_bytes = vec![0u8; HEADER_SIZE];
        file.read_exact(&mut header_bytes)?;
        
        let header: VaultHeader = bincode::deserialize(&header_bytes)?;
        
        // Verify magic
        if header.magic != *VAULT_MAGIC {
            return Err(StorageError::InvalidFormat);
        }
        
        Ok(Self {
            path: path.to_owned(),
            file,
            header,
        })
    }
    
    pub fn write_data(&mut self, encrypted_dag: &[u8]) -> Result<(), StorageError> {
        self.file.seek(SeekFrom::Start(HEADER_SIZE as u64))?;
        self.file.write_all(encrypted_dag)?;
        self.file.sync_all()?;
        
        // Update header
        self.header.last_modified = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        self.update_header()?;
        Ok(())
    }
    
    pub fn read_data(&mut self) -> Result<Vec<u8>, StorageError> {
        self.file.seek(SeekFrom::Start(HEADER_SIZE as u64))?;
        let mut data = Vec::new();
        self.file.read_to_end(&mut data)?;
        Ok(data)
    }
}
```

### 3. DAG Module (`dag/`)

#### 3.1 Node Structure (`node.rs`)
```rust
use uuid::Uuid;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub Uuid);

impl NodeId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DagNode {
    pub id: NodeId,
    pub node_type: NodeType,
    pub parents: Vec<NodeId>,
    pub children: Vec<NodeId>,
    pub encrypted_data: Vec<u8>,
    pub metadata: NodeMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeType {
    Root,
    Category { 
        name: String,
        parent_category: Option<NodeId>,
    },
    Secret {
        entry_id: Uuid,
        current_version: u32,
    },
    Version {
        entry_id: Uuid,
        version: u32,
        previous_version: Option<NodeId>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetadata {
    pub created_at: u64,
    pub modified_at: u64,
    pub created_by: Option<UserId>,
    pub modified_by: Option<UserId>,
    pub access_count: u32,
    pub last_accessed: Option<u64>,
    pub tags: Vec<String>,
}
```

#### 3.2 DAG Operations (`graph.rs`)
```rust
use std::collections::{HashMap, HashSet, VecDeque};
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::algo::is_cyclic_directed;

pub struct VaultDag {
    graph: DiGraph<NodeId, EdgeType>,
    node_map: HashMap<NodeId, NodeIndex>,
    nodes: HashMap<NodeId, DagNode>,
    root_id: NodeId,
    categories: HashMap<String, NodeId>,
}

#[derive(Debug, Clone)]
pub enum EdgeType {
    Parent,
    Category,
    Version,
}

impl VaultDag {
    pub fn new() -> Self {
        let mut graph = DiGraph::new();
        let root_id = NodeId::new();
        
        let root_node = DagNode {
            id: root_id.clone(),
            node_type: NodeType::Root,
            parents: vec![],
            children: vec![],
            encrypted_data: vec![],
            metadata: NodeMetadata::default(),
        };
        
        let root_idx = graph.add_node(root_id.clone());
        let mut node_map = HashMap::new();
        node_map.insert(root_id.clone(), root_idx);
        
        let mut nodes = HashMap::new();
        nodes.insert(root_id.clone(), root_node);
        
        Self {
            graph,
            node_map,
            nodes,
            root_id,
            categories: HashMap::new(),
        }
    }
    
    pub fn add_node(&mut self, node: DagNode) -> Result<NodeId, DagError> {
        // Check for cycles
        let node_idx = self.graph.add_node(node.id.clone());
        
        // Add edges
        for parent_id in &node.parents {
            if let Some(&parent_idx) = self.node_map.get(parent_id) {
                self.graph.add_edge(parent_idx, node_idx, EdgeType::Parent);
            }
        }
        
        // Verify no cycles
        if is_cyclic_directed(&self.graph) {
            // Rollback
            self.graph.remove_node(node_idx);
            return Err(DagError::CycleDetected);
        }
        
        // Update parent's children
        for parent_id in &node.parents {
            if let Some(parent) = self.nodes.get_mut(parent_id) {
                parent.children.push(node.id.clone());
            }
        }
        
        let node_id = node.id.clone();
        self.node_map.insert(node_id.clone(), node_idx);
        self.nodes.insert(node_id.clone(), node);
        
        Ok(node_id)
    }
    
    pub fn get_node(&self, id: &NodeId) -> Option<&DagNode> {
        self.nodes.get(id)
    }
    
    pub fn traverse_from(&self, start_id: &NodeId, max_depth: Option<usize>) -> Vec<NodeId> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut result = Vec::new();
        
        queue.push_back((start_id.clone(), 0));
        
        while let Some((node_id, depth)) = queue.pop_front() {
            if let Some(max_d) = max_depth {
                if depth > max_d {
                    continue;
                }
            }
            
            if visited.insert(node_id.clone()) {
                result.push(node_id.clone());
                
                if let Some(node) = self.nodes.get(&node_id) {
                    for child_id in &node.children {
                        queue.push_back((child_id.clone(), depth + 1));
                    }
                }
            }
        }
        
        result
    }
}
```

### 4. Vault Core (`vault.rs`)

```rust
use std::path::Path;
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Debug)]
pub struct Vault {
    storage: Box<dyn VaultStorage>,
    crypto: VaultCrypto,
    dag: VaultDag,
    config: VaultConfig,
    session: VaultSession,
}

#[derive(Debug, ZeroizeOnDrop)]
struct VaultCrypto {
    #[zeroize(skip)]
    master_key_hash: [u8; 32],
    vault_key: [u8; 32],
    cipher: AeadCipher,
    kem: Option<VaultKem>,
    sign: Option<VaultSign>,
}

#[derive(Debug)]
struct VaultSession {
    opened_at: std::time::Instant,
    last_save: std::time::Instant,
    changes_count: usize,
    auto_save_threshold: usize,
}

impl Vault {
    pub fn create(path: &Path, master_password: &str) -> Result<Self, VaultError> {
        // Generate salt
        let mut salt = [0u8; KDF_SALT_LEN];
        OsRng.fill_bytes(&mut salt);
        
        // Derive master key
        let kdf_params = KdfParams::default();
        let master_key = derive_master_key(
            master_password.as_bytes(),
            &salt,
            &kdf_params
        )?;
        
        // Generate vault key
        let mut vault_key = [0u8; 32];
        OsRng.fill_bytes(&mut vault_key);
        
        // Encrypt vault key with master key
        let master_cipher = AeadCipher::new(&master_key);
        let encrypted_vault_key = master_cipher.encrypt(&vault_key, Some(b"vault_key"))?;
        
        // Create header
        let header = VaultHeader {
            magic: *VAULT_MAGIC,
            version: 1,
            created_at: current_timestamp(),
            last_modified: current_timestamp(),
            kdf_params,
            salt,
            encrypted_vault_key,
            kem_public_key: None,
            checksum: [0u8; 32], // Will be updated on save
        };
        
        // Create storage
        let storage = Box::new(FileStorage::create(path, header)?);
        
        // Create crypto
        let crypto = VaultCrypto {
            master_key_hash: blake3::hash(master_password.as_bytes()).into(),
            vault_key,
            cipher: AeadCipher::new(&vault_key),
            kem: None,
            sign: None,
        };
        
        // Create DAG
        let dag = VaultDag::new();
        
        // Create config
        let config = VaultConfig::default();
        
        // Create session
        let session = VaultSession {
            opened_at: std::time::Instant::now(),
            last_save: std::time::Instant::now(),
            changes_count: 0,
            auto_save_threshold: 10,
        };
        
        let mut vault = Self {
            storage,
            crypto,
            dag,
            config,
            session,
        };
        
        // Save initial state
        vault.save()?;
        
        Ok(vault)
    }
    
    pub fn add_secret(&mut self, mut entry: SecretEntry) -> Result<NodeId, VaultError> {
        // Set timestamps
        let now = current_timestamp();
        entry.created_at = now;
        entry.modified_at = now;
        
        // Serialize and encrypt
        let plaintext = bincode::serialize(&entry)?;
        let encrypted = self.crypto.cipher.encrypt(&plaintext, Some(b"secret_entry"))?;
        
        // Create DAG node
        let entry_id = Uuid::new_v4();
        let node = DagNode {
            id: NodeId::new(),
            node_type: NodeType::Secret {
                entry_id,
                current_version: 1,
            },
            parents: vec![self.dag.root_id.clone()],
            children: vec![],
            encrypted_data: encrypted,
            metadata: NodeMetadata {
                created_at: now,
                modified_at: now,
                created_by: None,
                modified_by: None,
                access_count: 0,
                last_accessed: None,
                tags: entry.tags.clone(),
            },
        };
        
        // Add to DAG
        let node_id = self.dag.add_node(node)?;
        
        // Update session
        self.session.changes_count += 1;
        
        // Auto-save if threshold reached
        if self.session.changes_count >= self.session.auto_save_threshold {
            self.save()?;
        }
        
        Ok(node_id)
    }
    
    pub fn get_secret(&self, id: &NodeId) -> Result<SecretEntry, VaultError> {
        let node = self.dag.get_node(id)
            .ok_or(VaultError::SecretNotFound)?;
        
        // Decrypt
        let plaintext = self.crypto.cipher.decrypt(
            &node.encrypted_data,
            Some(b"secret_entry")
        )?;
        
        // Deserialize
        let entry: SecretEntry = bincode::deserialize(&plaintext)?;
        
        Ok(entry)
    }
    
    pub fn save(&mut self) -> Result<(), VaultError> {
        // Serialize DAG
        let dag_data = bincode::serialize(&self.dag)?;
        
        // Encrypt DAG
        let encrypted_dag = self.crypto.cipher.encrypt(&dag_data, Some(b"vault_dag"))?;
        
        // Write to storage
        self.storage.write_data(&encrypted_dag)?;
        
        // Update session
        self.session.last_save = std::time::Instant::now();
        self.session.changes_count = 0;
        
        Ok(())
    }
}
```

## Testing Infrastructure

### Unit Test Examples

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_vault_creation_and_reopen() {
        let dir = tempdir().unwrap();
        let vault_path = dir.path().join("test.vault");
        let password = "test_password_123";
        
        // Create vault
        {
            let vault = Vault::create(&vault_path, password).unwrap();
            assert!(vault_path.exists());
        }
        
        // Reopen vault
        {
            let vault = Vault::open(&vault_path, password).unwrap();
            assert_eq!(vault.dag.nodes.len(), 1); // Only root node
        }
        
        // Wrong password should fail
        assert!(Vault::open(&vault_path, "wrong_password").is_err());
    }
    
    #[test]
    fn test_secret_operations() {
        let dir = tempdir().unwrap();
        let vault_path = dir.path().join("test.vault");
        let password = "test_password_123";
        
        let mut vault = Vault::create(&vault_path, password).unwrap();
        
        // Add secret
        let entry = SecretEntry {
            label: "test_secret".to_string(),
            username: "user@example.com".to_string(),
            password: SecureString::from("secret_password"),
            ..Default::default()
        };
        
        let node_id = vault.add_secret(entry.clone()).unwrap();
        
        // Get secret
        let retrieved = vault.get_secret(&node_id).unwrap();
        assert_eq!(retrieved.label, entry.label);
        assert_eq!(retrieved.username, entry.username);
        
        // Update secret
        let mut updated_entry = retrieved;
        updated_entry.username = "new_user@example.com".to_string();
        
        let new_node_id = vault.update_secret(&node_id, updated_entry).unwrap();
        assert_ne!(node_id, new_node_id);
        
        // List secrets
        let secrets = vault.list_secrets(None).unwrap();
        assert_eq!(secrets.len(), 1);
    }
    
    #[test]
    fn test_encryption_integrity() {
        let key = [0u8; 32];
        let cipher = AeadCipher::new(&key);
        
        let plaintext = b"sensitive data";
        let aad = b"additional data";
        
        // Encrypt
        let ciphertext = cipher.encrypt(plaintext, Some(aad)).unwrap();
        
        // Decrypt with correct AAD
        let decrypted = cipher.decrypt(&ciphertext, Some(aad)).unwrap();
        assert_eq!(&decrypted, plaintext);
        
        // Decrypt with wrong AAD should fail
        assert!(cipher.decrypt(&ciphertext, Some(b"wrong aad")).is_err());
        
        // Tampered ciphertext should fail
        let mut tampered = ciphertext.clone();
        tampered[20] ^= 0xFF;
        assert!(cipher.decrypt(&tampered, Some(aad)).is_err());
    }
}
```

### Integration Test Examples

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use qudag_crypto::kem::KemKeyPair;
    
    #[tokio::test]
    async fn test_concurrent_access() {
        let dir = tempdir().unwrap();
        let vault_path = dir.path().join("test.vault");
        
        // Create vault
        let vault = Arc::new(RwLock::new(
            Vault::create(&vault_path, "password").unwrap()
        ));
        
        // Spawn multiple tasks
        let mut handles = vec![];
        
        for i in 0..10 {
            let vault_clone = Arc::clone(&vault);
            let handle = tokio::spawn(async move {
                let entry = SecretEntry {
                    label: format!("secret_{}", i),
                    username: format!("user_{}", i),
                    password: SecureString::from(format!("pass_{}", i)),
                    ..Default::default()
                };
                
                let mut vault = vault_clone.write().await;
                vault.add_secret(entry).unwrap()
            });
            handles.push(handle);
        }
        
        // Wait for all tasks
        for handle in handles {
            handle.await.unwrap();
        }
        
        // Verify all secrets added
        let vault = vault.read().await;
        let secrets = vault.list_secrets(None).unwrap();
        assert_eq!(secrets.len(), 10);
    }
    
    #[test]
    fn test_kem_key_sharing() {
        // Generate two keypairs
        let alice_kem = VaultKem::generate().unwrap();
        let bob_kem = VaultKem::generate().unwrap();
        
        // Alice encapsulates for Bob
        let (ciphertext, shared_secret_alice) = alice_kem
            .encapsulate_for(bob_kem.public_key())
            .unwrap();
        
        // Bob decapsulates
        let shared_secret_bob = bob_kem.decapsulate(&ciphertext).unwrap();
        
        // Shared secrets should match
        assert_eq!(shared_secret_alice, shared_secret_bob);
    }
}
```

## Performance Benchmarks

```rust
#[cfg(test)]
mod benches {
    use super::*;
    use criterion::{black_box, criterion_group, criterion_main, Criterion};
    
    fn bench_kdf(c: &mut Criterion) {
        let password = b"test_password_123";
        let salt = [0u8; 32];
        let params = KdfParams::default();
        
        c.bench_function("argon2id_derive", |b| {
            b.iter(|| {
                derive_master_key(black_box(password), black_box(&salt), &params)
            })
        });
    }
    
    fn bench_encryption(c: &mut Criterion) {
        let key = [0u8; 32];
        let cipher = AeadCipher::new(&key);
        let plaintext = vec![0u8; 1024]; // 1KB
        
        c.bench_function("aes256gcm_encrypt_1kb", |b| {
            b.iter(|| {
                cipher.encrypt(black_box(&plaintext), None)
            })
        });
    }
    
    fn bench_dag_operations(c: &mut Criterion) {
        let mut dag = VaultDag::new();
        
        // Add 1000 nodes
        for _ in 0..1000 {
            let node = DagNode {
                id: NodeId::new(),
                node_type: NodeType::Secret {
                    entry_id: Uuid::new_v4(),
                    current_version: 1,
                },
                parents: vec![dag.root_id.clone()],
                children: vec![],
                encrypted_data: vec![0u8; 256],
                metadata: NodeMetadata::default(),
            };
            dag.add_node(node).unwrap();
        }
        
        c.bench_function("dag_traverse_1000_nodes", |b| {
            b.iter(|| {
                dag.traverse_from(black_box(&dag.root_id), None)
            })
        });
    }
    
    criterion_group!(benches, bench_kdf, bench_encryption, bench_dag_operations);
    criterion_main!(benches);
}
```

## Security Considerations

### Memory Safety
1. All sensitive data uses `zeroize` for secure cleanup
2. Minimal plaintext exposure time
3. No sensitive data in logs or error messages

### Cryptographic Best Practices
1. Constant-time operations where applicable
2. Secure random number generation
3. Proper nonce/IV handling
4. Authentication for all encrypted data

### File System Security
1. Restrictive file permissions (0600)
2. Atomic writes with fsync
3. Secure temporary file handling
4. Protection against path traversal

## Next Steps

1. Complete core module implementation
2. Extensive security testing
3. Performance optimization
4. Documentation generation
5. Prepare for CLI integration in Phase 2