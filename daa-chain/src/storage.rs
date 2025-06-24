//! Storage layer for DAA Chain using QuDAG storage primitives

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

use crate::qudag_stubs::qudag_core::{Block, Transaction, Hash};
use crate::{Result, ChainError};

/// Storage interface for DAA Chain data
#[async_trait::async_trait]
pub trait StorageInterface: Send + Sync {
    /// Store a block
    async fn store_block(&mut self, block: Block) -> Result<()>;
    
    /// Retrieve a block by hash
    async fn get_block(&self, hash: &Hash) -> Result<Option<Block>>;
    
    /// Store a transaction
    async fn store_transaction(&mut self, tx: Transaction) -> Result<()>;
    
    /// Retrieve a transaction by hash
    async fn get_transaction(&self, hash: &Hash) -> Result<Option<Transaction>>;
    
    /// Get the current chain height
    async fn get_height(&self) -> Result<u64>;
    
    /// Get block hash at specific height
    async fn get_block_hash_at_height(&self, height: u64) -> Result<Option<Hash>>;
    
    /// Store chain metadata
    async fn store_metadata(&mut self, key: String, value: Vec<u8>) -> Result<()>;
    
    /// Retrieve chain metadata
    async fn get_metadata(&self, key: &str) -> Result<Option<Vec<u8>>>;
}

/// File-based storage implementation
pub struct FileStorage {
    /// Root directory for storage
    root_path: PathBuf,
    
    /// In-memory cache for recent blocks
    block_cache: Arc<RwLock<HashMap<Hash, Block>>>,
    
    /// In-memory cache for recent transactions
    tx_cache: Arc<RwLock<HashMap<Hash, Transaction>>>,
    
    /// Metadata storage
    metadata: Arc<RwLock<HashMap<String, Vec<u8>>>>,
    
    /// Current chain height
    height: Arc<RwLock<u64>>,
    
    /// Block height to hash mapping
    height_to_hash: Arc<RwLock<HashMap<u64, Hash>>>,
}

impl FileStorage {
    /// Create new file storage instance
    pub async fn new<P: AsRef<Path>>(root_path: P) -> Result<Self> {
        let root_path = root_path.as_ref().to_path_buf();
        
        // Create directories
        tokio::fs::create_dir_all(&root_path).await
            .map_err(|e| ChainError::Storage(format!("Failed to create storage directory: {}", e)))?;
        
        tokio::fs::create_dir_all(root_path.join("blocks")).await
            .map_err(|e| ChainError::Storage(format!("Failed to create blocks directory: {}", e)))?;
        
        tokio::fs::create_dir_all(root_path.join("transactions")).await
            .map_err(|e| ChainError::Storage(format!("Failed to create transactions directory: {}", e)))?;

        let storage = Self {
            root_path,
            block_cache: Arc::new(RwLock::new(HashMap::new())),
            tx_cache: Arc::new(RwLock::new(HashMap::new())),
            metadata: Arc::new(RwLock::new(HashMap::new())),
            height: Arc::new(RwLock::new(0)),
            height_to_hash: Arc::new(RwLock::new(HashMap::new())),
        };

        // Load existing metadata
        storage.load_metadata().await?;
        
        Ok(storage)
    }

    /// Load metadata from disk
    async fn load_metadata(&self) -> Result<()> {
        let metadata_path = self.root_path.join("metadata.json");
        
        if tokio::fs::metadata(&metadata_path).await.is_ok() {
            let data = tokio::fs::read_to_string(&metadata_path).await
                .map_err(|e| ChainError::Storage(format!("Failed to read metadata: {}", e)))?;
            
            let stored_metadata: HashMap<String, Vec<u8>> = serde_json::from_str(&data)
                .map_err(|e| ChainError::Storage(format!("Failed to parse metadata: {}", e)))?;
            
            *self.metadata.write().await = stored_metadata;
            
            // Load height information
            if let Some(height_data) = self.metadata.read().await.get("chain_height") {
                if let Ok(height) = String::from_utf8_lossy(height_data).parse::<u64>() {
                    *self.height.write().await = height;
                }
            }
        }
        
        Ok(())
    }

    /// Save metadata to disk
    async fn save_metadata(&self) -> Result<()> {
        let metadata_path = self.root_path.join("metadata.json");
        let metadata = self.metadata.read().await.clone();
        
        let data = serde_json::to_string_pretty(&metadata)
            .map_err(|e| ChainError::Storage(format!("Failed to serialize metadata: {}", e)))?;
        
        tokio::fs::write(&metadata_path, data).await
            .map_err(|e| ChainError::Storage(format!("Failed to write metadata: {}", e)))?;
        
        Ok(())
    }

    /// Get file path for block storage
    fn block_path(&self, hash: &Hash) -> PathBuf {
        self.root_path.join("blocks").join(format!("{}.json", hash))
    }

    /// Get file path for transaction storage
    fn transaction_path(&self, hash: &Hash) -> PathBuf {
        self.root_path.join("transactions").join(format!("{}.json", hash))
    }
}

#[async_trait::async_trait]
impl StorageInterface for FileStorage {
    async fn store_block(&mut self, block: Block) -> Result<()> {
        let hash = block.hash();
        
        // Store in cache
        self.block_cache.write().await.insert(hash, block.clone());
        
        // Store to disk
        let block_data = serde_json::to_string_pretty(&block)
            .map_err(|e| ChainError::Storage(format!("Failed to serialize block: {}", e)))?;
        
        let block_path = self.block_path(&hash);
        tokio::fs::write(&block_path, block_data).await
            .map_err(|e| ChainError::Storage(format!("Failed to write block: {}", e)))?;
        
        // Update height mapping
        let height = *self.height.read().await + 1;
        self.height_to_hash.write().await.insert(height, hash);
        *self.height.write().await = height;
        
        // Update metadata
        self.metadata.write().await.insert(
            "chain_height".to_string(),
            height.to_string().into_bytes(),
        );
        
        self.save_metadata().await?;
        
        tracing::debug!("Stored block {} at height {}", hash, height);
        
        Ok(())
    }

    async fn get_block(&self, hash: &Hash) -> Result<Option<Block>> {
        // Check cache first
        if let Some(block) = self.block_cache.read().await.get(hash) {
            return Ok(Some(block.clone()));
        }
        
        // Load from disk
        let block_path = self.block_path(hash);
        
        if tokio::fs::metadata(&block_path).await.is_ok() {
            let data = tokio::fs::read_to_string(&block_path).await
                .map_err(|e| ChainError::Storage(format!("Failed to read block: {}", e)))?;
            
            let block: Block = serde_json::from_str(&data)
                .map_err(|e| ChainError::Storage(format!("Failed to parse block: {}", e)))?;
            
            // Add to cache
            self.block_cache.write().await.insert(*hash, block.clone());
            
            Ok(Some(block))
        } else {
            Ok(None)
        }
    }

    async fn store_transaction(&mut self, tx: Transaction) -> Result<()> {
        let hash = tx.hash();
        
        // Store in cache
        self.tx_cache.write().await.insert(hash, tx.clone());
        
        // Store to disk
        let tx_data = serde_json::to_string_pretty(&tx)
            .map_err(|e| ChainError::Storage(format!("Failed to serialize transaction: {}", e)))?;
        
        let tx_path = self.transaction_path(&hash);
        tokio::fs::write(&tx_path, tx_data).await
            .map_err(|e| ChainError::Storage(format!("Failed to write transaction: {}", e)))?;
        
        tracing::debug!("Stored transaction {}", hash);
        
        Ok(())
    }

    async fn get_transaction(&self, hash: &Hash) -> Result<Option<Transaction>> {
        // Check cache first
        if let Some(tx) = self.tx_cache.read().await.get(hash) {
            return Ok(Some(tx.clone()));
        }
        
        // Load from disk
        let tx_path = self.transaction_path(hash);
        
        if tokio::fs::metadata(&tx_path).await.is_ok() {
            let data = tokio::fs::read_to_string(&tx_path).await
                .map_err(|e| ChainError::Storage(format!("Failed to read transaction: {}", e)))?;
            
            let tx: Transaction = serde_json::from_str(&data)
                .map_err(|e| ChainError::Storage(format!("Failed to parse transaction: {}", e)))?;
            
            // Add to cache
            self.tx_cache.write().await.insert(*hash, tx.clone());
            
            Ok(Some(tx))
        } else {
            Ok(None)
        }
    }

    async fn get_height(&self) -> Result<u64> {
        Ok(*self.height.read().await)
    }

    async fn get_block_hash_at_height(&self, height: u64) -> Result<Option<Hash>> {
        Ok(self.height_to_hash.read().await.get(&height).copied())
    }

    async fn store_metadata(&mut self, key: String, value: Vec<u8>) -> Result<()> {
        self.metadata.write().await.insert(key, value);
        self.save_metadata().await
    }

    async fn get_metadata(&self, key: &str) -> Result<Option<Vec<u8>>> {
        Ok(self.metadata.read().await.get(key).cloned())
    }
}

/// Storage wrapper for DAA Chain operations
pub struct Storage {
    inner: Box<dyn StorageInterface>,
    pending_transactions: Arc<RwLock<HashMap<Hash, Transaction>>>,
}

impl Storage {
    /// Create new storage instance
    pub fn new<P: AsRef<Path>>(storage_path: P) -> Result<Self> {
        let inner = tokio::runtime::Handle::current().block_on(async {
            FileStorage::new(storage_path).await
        })?;

        Ok(Self {
            inner: Box::new(inner),
            pending_transactions: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Add block to storage
    pub async fn add_block(&mut self, block: Block) -> Result<()> {
        // Remove transactions from pending pool
        for tx in block.transactions() {
            self.pending_transactions.write().await.remove(&tx.hash());
        }
        
        // Store the block
        self.inner.store_block(block).await
    }

    /// Get block by hash
    pub async fn get_block(&self, hash: &Hash) -> Result<Option<Block>> {
        self.inner.get_block(hash).await
    }

    /// Add transaction to storage
    pub async fn add_transaction(&mut self, tx: Transaction) -> Result<()> {
        self.inner.store_transaction(tx).await
    }

    /// Add transaction to pending pool
    pub async fn add_pending_transaction(&mut self, tx: Transaction) -> Result<()> {
        let hash = tx.hash();
        self.pending_transactions.write().await.insert(hash, tx.clone());
        self.add_transaction(tx).await
    }

    /// Get pending transactions up to limit
    pub async fn get_pending_transactions(&self, limit: usize) -> Result<Vec<Transaction>> {
        let pending = self.pending_transactions.read().await;
        Ok(pending.values().take(limit).cloned().collect())
    }

    /// Get transaction by hash
    pub async fn get_transaction(&self, hash: &Hash) -> Result<Option<Transaction>> {
        self.inner.get_transaction(hash).await
    }

    /// Get current chain height
    pub async fn get_height(&self) -> Result<u64> {
        self.inner.get_height().await
    }

    /// Get block at specific height
    pub async fn get_block_at_height(&self, height: u64) -> Result<Option<Block>> {
        if let Some(hash) = self.inner.get_block_hash_at_height(height).await? {
            self.get_block(&hash).await
        } else {
            Ok(None)
        }
    }

    /// Store metadata
    pub async fn store_metadata(&mut self, key: String, value: Vec<u8>) -> Result<()> {
        self.inner.store_metadata(key, value).await
    }

    /// Get metadata
    pub async fn get_metadata(&self, key: &str) -> Result<Option<Vec<u8>>> {
        self.inner.get_metadata(key).await
    }

    /// Get storage statistics
    pub async fn get_stats(&self) -> StorageStats {
        StorageStats {
            total_blocks: self.get_height().await.unwrap_or(0),
            pending_transactions: self.pending_transactions.read().await.len(),
        }
    }
}

/// Storage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStats {
    pub total_blocks: u64,
    pub pending_transactions: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use crate::qudag_stubs::qudag_core::Block;

    #[tokio::test]
    async fn test_file_storage() {
        let temp_dir = TempDir::new().unwrap();
        let mut storage = FileStorage::new(temp_dir.path()).await.unwrap();

        // Test metadata storage
        storage.store_metadata("test_key".to_string(), b"test_value".to_vec()).await.unwrap();
        let value = storage.get_metadata("test_key").await.unwrap();
        assert_eq!(value, Some(b"test_value".to_vec()));

        // Test height
        let height = storage.get_height().await.unwrap();
        assert_eq!(height, 0);
    }

    #[tokio::test]
    async fn test_storage_wrapper() {
        let temp_dir = TempDir::new().unwrap();
        let storage = Storage::new(temp_dir.path()).unwrap();

        let height = storage.get_height().await.unwrap();
        assert_eq!(height, 0);

        let stats = storage.get_stats().await;
        assert_eq!(stats.total_blocks, 0);
        assert_eq!(stats.pending_transactions, 0);
    }
}