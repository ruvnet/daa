//! Ethereum blockchain adapter implementation

use async_trait::async_trait;
use ethers::prelude::*;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{
    Address, AdapterError, Balance, Block as DaaBlock, BlockchainAdapter, Transaction, TxHash,
};

/// Ethereum adapter for blockchain operations
pub struct EthereumAdapter {
    rpc_url: String,
    client: Arc<Provider<Http>>,
    wallet: LocalWallet,
    connected: Arc<Mutex<bool>>,
}

impl EthereumAdapter {
    /// Create a new Ethereum adapter
    pub fn new(rpc_url: impl Into<String>, private_key: impl AsRef<str>) -> Self {
        let rpc_url = rpc_url.into();
        let provider = Provider::<Http>::try_from(&rpc_url)
            .expect("Invalid RPC URL");
        
        let wallet = private_key.as_ref()
            .parse::<LocalWallet>()
            .expect("Invalid private key");
        
        EthereumAdapter {
            rpc_url,
            client: Arc::new(provider),
            wallet,
            connected: Arc::new(Mutex::new(false)),
        }
    }
    
    fn convert_block(&self, block: ethers::types::Block<H256>) -> DaaBlock {
        DaaBlock {
            number: block.number.unwrap_or_default().as_u64(),
            hash: format!("{:?}", block.hash.unwrap_or_default()),
            parent_hash: format!("{:?}", block.parent_hash),
            timestamp: block.timestamp.as_u64(),
            transactions: block.transactions.iter()
                .map(|tx| TxHash::new(format!("{:?}", tx)))
                .collect(),
        }
    }
}

#[async_trait]
impl BlockchainAdapter for EthereumAdapter {
    async fn connect(&self) -> Result<(), AdapterError> {
        // Test connectivity by getting chain ID
        match self.client.get_chainid().await {
            Ok(chain_id) => {
                log::info!("Connected to Ethereum chain ID: {}", chain_id);
                *self.connected.lock().await = true;
                Ok(())
            }
            Err(e) => Err(AdapterError::ConnectionError(e.to_string())),
        }
    }
    
    async fn send_transaction(&self, tx: Transaction) -> Result<TxHash, AdapterError> {
        if !*self.connected.lock().await {
            return Err(AdapterError::ConnectionError("Not connected".to_string()));
        }
        
        // Convert to Ethereum transaction
        let to_addr = ethers::types::Address::from_slice(tx.to.as_bytes());
        let value = U256::from(tx.value.value);
        
        let eth_tx = TransactionRequest::new()
            .to(to_addr)
            .value(value)
            .data(tx.data);
        
        // Sign and send
        let signed_tx = self.wallet.sign_transaction(&eth_tx.into())
            .await
            .map_err(|e| AdapterError::TransactionError(e.to_string()))?;
        
        let pending = self.client
            .send_raw_transaction(signed_tx.rlp().into())
            .await
            .map_err(|e| AdapterError::TransactionError(e.to_string()))?;
        
        Ok(TxHash::new(format!("{:?}", pending.tx_hash())))
    }
    
    async fn query_balance(&self, account: &Address) -> Result<Balance, AdapterError> {
        if !*self.connected.lock().await {
            return Err(AdapterError::ConnectionError("Not connected".to_string()));
        }
        
        let addr = ethers::types::Address::from_slice(account.as_bytes());
        let balance = self.client
            .get_balance(addr, None)
            .await
            .map_err(|e| AdapterError::QueryError(e.to_string()))?;
        
        Ok(Balance::from_wei(balance.as_u128()))
    }
    
    async fn subscribe_blocks<F>(&self, handler: F) -> Result<(), AdapterError>
    where
        F: Fn(DaaBlock) + Send + 'static,
    {
        if !*self.connected.lock().await {
            return Err(AdapterError::ConnectionError("Not connected".to_string()));
        }
        
        let client = self.client.clone();
        
        tokio::spawn(async move {
            let mut stream = client.watch_blocks().await.unwrap();
            
            while let Some(block_hash) = stream.next().await {
                if let Ok(Some(block)) = client.get_block(block_hash).await {
                    let daa_block = DaaBlock {
                        number: block.number.unwrap_or_default().as_u64(),
                        hash: format!("{:?}", block.hash.unwrap_or_default()),
                        parent_hash: format!("{:?}", block.parent_hash),
                        timestamp: block.timestamp.as_u64(),
                        transactions: block.transactions.iter()
                            .map(|tx| TxHash::new(format!("{:?}", tx)))
                            .collect(),
                    };
                    handler(daa_block);
                }
            }
        });
        
        Ok(())
    }
    
    async fn get_block_number(&self) -> Result<u64, AdapterError> {
        if !*self.connected.lock().await {
            return Err(AdapterError::ConnectionError("Not connected".to_string()));
        }
        
        let block_number = self.client
            .get_block_number()
            .await
            .map_err(|e| AdapterError::QueryError(e.to_string()))?;
        
        Ok(block_number.as_u64())
    }
    
    async fn get_block(&self, number: u64) -> Result<Option<DaaBlock>, AdapterError> {
        if !*self.connected.lock().await {
            return Err(AdapterError::ConnectionError("Not connected".to_string()));
        }
        
        let block = self.client
            .get_block(number)
            .await
            .map_err(|e| AdapterError::QueryError(e.to_string()))?;
        
        Ok(block.map(|b| self.convert_block(b)))
    }
}