//! Network stub implementations for WASM
//!
//! These are placeholder implementations for network functionality
//! when building for WASM targets where real networking isn't available.

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

/// Stub implementation of network node for WASM
#[wasm_bindgen]
pub struct NetworkNode {
    node_id: String,
    port: u16,
}

#[wasm_bindgen]
impl NetworkNode {
    /// Create a new network node (stub)
    #[wasm_bindgen(constructor)]
    pub fn new(port: u16) -> Self {
        Self {
            node_id: format!("wasm-node-{}", uuid::Uuid::new_v4()),
            port,
        }
    }

    /// Get node ID
    #[wasm_bindgen(js_name = "getNodeId")]
    pub fn get_node_id(&self) -> String {
        self.node_id.clone()
    }

    /// Start the node (no-op in WASM)
    pub async fn start(&self) -> Result<(), JsError> {
        web_sys::console::log_1(&format!("Network node {} started (stub)", self.node_id).into());
        Ok(())
    }

    /// Connect to peer (no-op in WASM)
    #[wasm_bindgen(js_name = "connectToPeer")]
    pub async fn connect_to_peer(&self, _peer_address: &str) -> Result<(), JsError> {
        Err(JsError::new(
            "P2P networking not available in WASM. Use WebRTC or WebSocket proxy.",
        ))
    }
}

/// Peer information stub
#[wasm_bindgen]
#[derive(Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    peer_id: String,
    address: String,
    reputation: f64,
}

#[wasm_bindgen]
impl PeerInfo {
    /// Get peer ID
    #[wasm_bindgen(js_name = "getPeerId")]
    pub fn get_peer_id(&self) -> String {
        self.peer_id.clone()
    }

    /// Get peer address
    pub fn get_address(&self) -> String {
        self.address.clone()
    }

    /// Get reputation score
    pub fn get_reputation(&self) -> f64 {
        self.reputation
    }
}

/// Network manager stub for WASM
#[wasm_bindgen]
pub struct NetworkManager {
    is_webrtc_enabled: bool,
    websocket_url: Option<String>,
}

#[wasm_bindgen]
impl NetworkManager {
    /// Create new network manager
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            is_webrtc_enabled: false,
            websocket_url: None,
        }
    }

    /// Enable WebRTC for browser P2P
    #[wasm_bindgen(js_name = "enableWebRTC")]
    pub fn enable_webrtc(&mut self) {
        self.is_webrtc_enabled = true;
        web_sys::console::log_1(&"WebRTC enabled for P2P communication".into());
    }

    /// Set WebSocket proxy URL
    #[wasm_bindgen(js_name = "setWebSocketProxy")]
    pub fn set_websocket_proxy(&mut self, url: String) {
        self.websocket_url = Some(url);
        web_sys::console::log_1(&format!("WebSocket proxy set to: {}", url).into());
    }

    /// Get network stats (stub data)
    #[wasm_bindgen(js_name = "getNetworkStats")]
    pub fn get_network_stats(&self) -> Result<JsValue, JsError> {
        let stats = serde_json::json!({
            "connected_peers": 0,
            "total_bandwidth": 0,
            "messages_sent": 0,
            "messages_received": 0,
            "webrtc_enabled": self.is_webrtc_enabled,
            "websocket_proxy": self.websocket_url,
            "note": "Network features require WebRTC or WebSocket proxy in WASM"
        });

        Ok(serde_wasm_bindgen::to_value(&stats)?)
    }
}

/// Onion routing stub
#[wasm_bindgen]
pub struct OnionRouter {
    layers: u8,
}

#[wasm_bindgen]
impl OnionRouter {
    /// Create new onion router
    #[wasm_bindgen(constructor)]
    pub fn new(layers: u8) -> Self {
        Self { layers }
    }

    /// Create onion message (returns mock encrypted data)
    #[wasm_bindgen(js_name = "createOnionMessage")]
    pub fn create_onion_message(
        &self,
        message: &str,
        _path: Vec<String>,
    ) -> Result<Vec<u8>, JsError> {
        // In real implementation, this would create layered encryption
        // For WASM stub, just return a mock encrypted message
        let mock_encrypted = format!("ONION[{}]:{}", self.layers, message);
        Ok(mock_encrypted.as_bytes().to_vec())
    }

    /// Decrypt onion layer (stub)
    #[wasm_bindgen(js_name = "decryptLayer")]
    pub fn decrypt_layer(&self, _data: &[u8]) -> Result<Vec<u8>, JsError> {
        Err(JsError::new(
            "Onion routing requires full network stack. Use server relay in WASM.",
        ))
    }
}

/// Export network-related types and functions for internal use
pub mod internal {
    use super::*;

    /// Create a mock peer for testing
    pub fn create_mock_peer(id: &str) -> PeerInfo {
        PeerInfo {
            peer_id: id.to_string(),
            address: format!("wasm://mock/{}", id),
            reputation: 1.0,
        }
    }

    /// Check if WebRTC is available in the browser
    pub fn is_webrtc_available() -> bool {
        // Check if RTCPeerConnection is available
        js_sys::Reflect::has(&js_sys::global(), &JsValue::from_str("RTCPeerConnection"))
            .unwrap_or(false)
    }
}
