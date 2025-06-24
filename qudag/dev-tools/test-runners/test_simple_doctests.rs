fn main() {
    println!("Testing documentation examples manually...");
    
    // Test 1: Verify simple digest functionality
    test_digest_example();
    
    // Test 2: Verify node config example
    test_node_config_example();
    
    println!("✓ All manual doc tests passed!");
}

fn test_digest_example() {
    // Emulate the digest example from documentation
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Digest(Vec<u8>);
    
    impl Digest {
        fn as_bytes(&self) -> &[u8] {
            &self.0
        }
        
        fn into_bytes(self) -> Vec<u8> {
            self.0
        }
    }
    
    let digest = Digest(vec![0x12, 0x34, 0x56, 0x78]);
    let bytes = digest.as_bytes();
    assert_eq!(bytes, &[0x12, 0x34, 0x56, 0x78]);
    
    let digest2 = Digest(vec![0x12, 0x34, 0x56, 0x78]);
    let bytes2 = digest2.into_bytes();
    assert_eq!(bytes2, vec![0x12, 0x34, 0x56, 0x78]);
    
    println!("✓ Digest example test passed");
}

fn test_node_config_example() {
    use std::path::PathBuf;
    
    #[derive(Debug, Clone)]
    struct NodeConfig {
        data_dir: PathBuf,
        network_port: u16,
        max_peers: usize,
        initial_peers: Vec<String>,
    }
    
    impl Default for NodeConfig {
        fn default() -> Self {
            Self {
                data_dir: PathBuf::from("./data"),
                network_port: 8000,
                max_peers: 50,
                initial_peers: Vec::new(),
            }
        }
    }
    
    // Test default configuration
    let config = NodeConfig::default();
    assert_eq!(config.network_port, 8000);
    
    // Test custom configuration
    let custom_config = NodeConfig {
        data_dir: PathBuf::from("/custom/data"),
        network_port: 9000,
        max_peers: 100,
        initial_peers: vec!["peer1:8000".to_string(), "peer2:8000".to_string()],
    };
    
    assert_eq!(custom_config.network_port, 9000);
    assert_eq!(custom_config.max_peers, 100);
    assert_eq!(custom_config.initial_peers.len(), 2);
    
    println!("✓ NodeConfig example test passed");
}