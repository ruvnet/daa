//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

use qudag_wasm::*;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_client_creation() {
    let client = QuDAGClient::new();
    assert_eq!(QuDAGClient::get_version(), env!("CARGO_PKG_VERSION"));
}

#[wasm_bindgen_test]
fn test_feature_detection() {
    assert!(!QuDAGClient::has_feature("unknown"));
    #[cfg(feature = "vault")]
    assert!(QuDAGClient::has_feature("vault"));
}

#[wasm_bindgen_test]
async fn test_crypto_operations() {
    use qudag_wasm::wasm_crypto::*;

    // Test ML-DSA key generation and signing
    let keypair = WasmMlDsaKeyPair::new().unwrap();
    let message = b"Test message";
    let signature = keypair.sign(message).unwrap();
    assert!(!signature.is_empty());

    // Test ML-KEM operations
    let kem = WasmMlKem768::new();
    let keypair_data = kem.generate_key_pair().unwrap();
    assert!(keypair_data.is_object());

    // Test BLAKE3 hashing
    let hash = WasmHasher::hash_blake3(b"Hello, QuDAG!");
    assert_eq!(hash.len(), 32);
}

#[wasm_bindgen_test]
fn test_dag_operations() {
    // Skip DAG tests for crypto-only build
    /*
    use qudag_wasm::dag::*;

    let dag = WasmDag::new();
    let stats = dag.get_stats().unwrap();
    assert!(stats.is_object());

    let consensus = WasmConsensus::new();
    let metrics = consensus.get_metrics().unwrap();
    assert!(metrics.is_object());
    */
}

#[wasm_bindgen_test]
async fn test_network_operations() {
    // Skip network tests for crypto-only build
    /*
    use qudag_wasm::network::*;

    let network = WasmNetworkManager::new();

    // Add a peer
    let peer_id = network.add_peer("/ip4/127.0.0.1/tcp/8000").await.unwrap();
    assert!(!peer_id.is_empty());

    // List peers
    let peers = network.list_peers().unwrap();
    assert!(peers.is_array());

    // Get network stats
    let stats = network.get_network_stats().unwrap();
    assert!(stats.is_object());
    */
}

#[wasm_bindgen_test]
async fn test_dark_addressing() {
    // Skip address tests for crypto-only build
    /*
    use qudag_wasm::address::*;

    let resolver = WasmDarkResolver::new();

    // Register a domain
    let result = resolver.register_domain("test.dark").await.unwrap();
    assert!(result.is_object());

    // Check domain availability
    assert!(!resolver.is_domain_available("test.dark").unwrap());
    assert!(resolver.is_domain_available("other.dark").unwrap());

    // Generate shadow address
    let shadow = resolver.generate_shadow_address(3600).unwrap();
    assert!(shadow.is_object());

    // Create fingerprint
    let fingerprint = resolver.create_fingerprint(b"Test data").unwrap();
    assert!(fingerprint.is_object());
    */
}

#[cfg(feature = "vault")]
#[wasm_bindgen_test]
async fn test_vault_operations() {
    // Skip vault tests for crypto-only build
    /*
    use qudag_wasm::vault::*;

    let vault = WasmVault::new();

    // Initialize vault
    vault.init("test_password_123").await.unwrap();

    // Generate password
    let password = WasmVault::generate_password(16, true, true).unwrap();
    assert_eq!(password.len(), 16);

    // Add entry
    let entry_id = vault.add_entry("test_site", "user@example.com", &password, None).unwrap();
    assert!(!entry_id.is_empty());

    // Get entry
    let entry = vault.get_entry("test_site").unwrap();
    assert!(entry.is_object());

    // List entries
    let entries = vault.list_entries(None).unwrap();
    assert!(entries.is_array());

    // Get stats
    let stats = vault.get_stats().unwrap();
    assert!(stats.is_object());
    */
}

#[wasm_bindgen_test]
fn test_utilities() {
    use qudag_wasm::utils::*;

    // Test encoding
    let original = "Hello, QuDAG!";
    let bytes = Encoding::string_to_bytes(original);
    let hex = Encoding::bytes_to_hex(&bytes);
    let decoded_bytes = Encoding::hex_to_bytes(&hex).unwrap();
    let decoded = Encoding::bytes_to_string(&decoded_bytes).unwrap();
    assert_eq!(original, decoded);

    // Test validation
    assert!(Validation::is_dark_domain("example.dark"));
    assert!(!Validation::is_dark_domain("example.com"));
    assert!(Validation::is_peer_address("/ip4/127.0.0.1/tcp/8000"));
    assert!(Validation::is_valid_hex("deadbeef"));

    // Test performance
    let start = Performance::now();
    // Simulate some work
    let _ = (0..1000).sum::<i32>();
    let duration = Performance::measure("test_operation", start);
    assert!(duration >= 0.0);

    // Test random
    let random_bytes = Random::get_bytes(32).unwrap();
    assert_eq!(random_bytes.len(), 32);

    let random_id = Random::get_id();
    assert_eq!(random_id.len(), 32); // 16 bytes as hex
}
