//! QuDAG Exchange WASM Bindings
//!
//! WebAssembly interface for the QuDAG Exchange system.

use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};
use qudag_exchange_core::{AccountId, Balance};

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global allocator
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// Initialize panic hook for better debugging
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default();
}

/// WASM-friendly account information
#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmAccount {
    id: String,
    public_key: String,
    balance: u64,
}

#[wasm_bindgen]
impl WasmAccount {
    #[wasm_bindgen(constructor)]
    pub fn new(id: String, public_key: String, balance: u64) -> Self {
        Self { id, public_key, balance }
    }

    #[wasm_bindgen(getter)]
    pub fn id(&self) -> String {
        self.id.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn public_key(&self) -> String {
        self.public_key.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn balance(&self) -> u64 {
        self.balance
    }
}

/// WASM-friendly transaction
#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmTransaction {
    id: String,
    from: String,
    to: String,
    amount: u64,
    status: String,
}

#[wasm_bindgen]
impl WasmTransaction {
    #[wasm_bindgen(constructor)]
    pub fn new(from: String, to: String, amount: u64) -> Self {
        Self {
            id: format!("tx_{}", js_sys::Date::now() as u64),
            from,
            to,
            amount,
            status: "pending".to_string(),
        }
    }

    #[wasm_bindgen(getter)]
    pub fn id(&self) -> String {
        self.id.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn from(&self) -> String {
        self.from.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn to(&self) -> String {
        self.to.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn amount(&self) -> u64 {
        self.amount
    }

    #[wasm_bindgen(getter)]
    pub fn status(&self) -> String {
        self.status.clone()
    }
}

/// Main WASM interface for QuDAG Exchange
#[wasm_bindgen]
pub struct QuDAGExchange {
    // In WASM, we'll use browser storage instead of direct file access
}

#[wasm_bindgen]
impl QuDAGExchange {
    /// Create a new QuDAG Exchange instance
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<QuDAGExchange, JsValue> {
        Ok(QuDAGExchange {})
    }

    /// Create a new account
    #[wasm_bindgen]
    pub async fn create_account(&mut self, name: String) -> Result<WasmAccount, JsValue> {
        // Generate mock keys (in real implementation, use Web Crypto API)
        let public_key = format!("pk_{}", &name);
        let account = WasmAccount::new(name, public_key, 1000);
        
        // Store in browser localStorage
        self.store_account(&account)?;
        
        Ok(account)
    }

    /// Get account balance
    #[wasm_bindgen]
    pub async fn get_balance(&self, account_id: String) -> Result<u64, JsValue> {
        // Retrieve from browser storage
        let account = self.load_account(&account_id)?;
        Ok(account.balance)
    }

    /// Transfer rUv tokens
    #[wasm_bindgen]
    pub async fn transfer(
        &mut self,
        from: String,
        to: String,
        amount: u64,
    ) -> Result<WasmTransaction, JsValue> {
        // Validate balances
        let from_account = self.load_account(&from)?;
        if from_account.balance < amount {
            return Err(JsValue::from_str("Insufficient balance"));
        }
        
        // Create transaction
        let tx = WasmTransaction::new(from, to, amount);
        
        // In a real implementation, this would submit to consensus
        // For now, just update local storage
        
        Ok(tx)
    }

    /// Get resource costs
    #[wasm_bindgen]
    pub fn get_resource_costs(&self) -> Result<JsValue, JsValue> {
        let costs = serde_json::json!({
            "create_account": 10,
            "transfer": 1,
            "store_data_per_kb": 5,
            "compute_per_ms": 2,
        });
        
        Ok(serde_wasm_bindgen::to_value(&costs)?)
    }

    // Helper methods (not exposed to WASM)
    
    fn store_account(&self, account: &WasmAccount) -> Result<(), JsValue> {
        let window = web_sys::window().ok_or("No window object")?;
        let storage = window.local_storage()?.ok_or("No local storage")?;
        
        let key = format!("qudag_account_{}", account.id);
        let value = serde_json::to_string(account)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
            
        storage.set_item(&key, &value)?;
        Ok(())
    }
    
    fn load_account(&self, id: &str) -> Result<WasmAccount, JsValue> {
        let window = web_sys::window().ok_or("No window object")?;
        let storage = window.local_storage()?.ok_or("No local storage")?;
        
        let key = format!("qudag_account_{}", id);
        let value = storage.get_item(&key)?
            .ok_or_else(|| JsValue::from_str("Account not found"))?;
            
        let account: WasmAccount = serde_json::from_str(&value)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
            
        Ok(account)
    }
}

// Utility functions

/// Get version information
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Log a message to the browser console
#[wasm_bindgen]
pub fn log(message: &str) {
    web_sys::console::log_1(&JsValue::from_str(message));
}