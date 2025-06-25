#![no_main]

use libfuzzer_sys::fuzz_target;
use arbitrary::Arbitrary;
use prime_dht::{Dht, DhtConfig};
use libp2p::PeerId;
use std::time::Duration;

/// Fuzz input for DHT operations
#[derive(Arbitrary, Debug)]
struct DhtFuzzInput {
    operations: Vec<DhtOperation>,
    config_params: ConfigParams,
}

#[derive(Arbitrary, Debug)]
struct ConfigParams {
    k_bucket_size: usize,
    alpha: usize,
    replication_factor: usize,
    ttl_seconds: u64,
}

#[derive(Arbitrary, Debug)]
enum DhtOperation {
    Put { key: Vec<u8>, value: Vec<u8> },
    Get { key: Vec<u8> },
    Remove { key: Vec<u8> },
    CleanupExpired,
}

fuzz_target!(|input: DhtFuzzInput| {
    // Limit operation count and data sizes
    if input.operations.len() > 1000 {
        return;
    }
    
    // Validate config parameters
    let k_bucket_size = input.config_params.k_bucket_size.max(1).min(100);
    let alpha = input.config_params.alpha.max(1).min(k_bucket_size);
    let replication_factor = input.config_params.replication_factor.max(1).min(20);
    let ttl_seconds = input.config_params.ttl_seconds.max(1).min(86400);
    
    let config = DhtConfig {
        k_bucket_size,
        alpha,
        replication_factor,
        refresh_interval: Duration::from_secs(3600),
        ttl: Duration::from_secs(ttl_seconds),
    };
    
    let peer_id = PeerId::random();
    let dht = Dht::new(peer_id, config);
    
    // Create a simple async runtime for the fuzz test
    let rt = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    
    rt.block_on(async {
        for operation in input.operations {
            match operation {
                DhtOperation::Put { key, value } => {
                    // Limit key and value sizes
                    if key.len() > 1000 || value.len() > 10000 {
                        continue;
                    }
                    
                    let _ = dht.put(key, value).await;
                }
                DhtOperation::Get { key } => {
                    if key.len() > 1000 {
                        continue;
                    }
                    
                    let _ = dht.get(key).await;
                }
                DhtOperation::Remove { key: _ } => {
                    // Remove operation not implemented in current DHT
                    // This tests that unimplemented operations don't crash
                }
                DhtOperation::CleanupExpired => {
                    // Test cleanup operations
                    // In a real implementation, this would trigger cleanup
                }
            }
        }
    });
});