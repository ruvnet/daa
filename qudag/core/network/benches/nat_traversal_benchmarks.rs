//! Performance benchmarks for NAT traversal functionality

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use qudag_network::{
    ConnectionManager, HolePunchCoordinator, NatTraversalConfig, NatTraversalManager,
    PortMappingProtocol, RelayManager, StunClient, StunServer,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;

fn benchmark_nat_detection(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("nat_detection", |b| {
        b.to_async(&rt).iter(|| async {
            let stun_servers = vec![StunServer::new("127.0.0.1:3478".parse().unwrap(), 1)];

            let client = StunClient::new(stun_servers);

            // This will likely fail but we're measuring the attempt time
            let _result = client.detect_nat().await;

            black_box(())
        });
    });
}

fn benchmark_stun_server_creation(c: &mut Criterion) {
    c.bench_function("stun_server_creation", |b| {
        b.iter(|| {
            let server = StunServer::new(black_box("8.8.8.8:3478".parse().unwrap()), black_box(1));
            black_box(server)
        });
    });
}

fn benchmark_hole_punch_setup(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("hole_punch_setup", |b| {
        b.to_async(&rt).iter(|| async {
            let coordinator = HolePunchCoordinator::new(Duration::from_millis(100));

            let peer_id = qudag_network::types::PeerId::random();
            let local_candidates = vec![
                "127.0.0.1:8080".parse().unwrap(),
                "127.0.0.1:8081".parse().unwrap(),
            ];
            let remote_candidates = vec![
                "127.0.0.1:9080".parse().unwrap(),
                "127.0.0.1:9081".parse().unwrap(),
            ];

            // This will timeout quickly, we're measuring setup time
            let _result = coordinator
                .start_hole_punch(peer_id, local_candidates, remote_candidates)
                .await;

            black_box(())
        });
    });
}

fn benchmark_relay_manager_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("relay_establish", |b| {
        b.to_async(&rt).iter(|| async {
            let manager = RelayManager::new(10);

            let relay_server = qudag_network::RelayServer {
                id: qudag_network::types::PeerId::random(),
                address: "/ip4/127.0.0.1/tcp/8080".parse().unwrap(),
                capacity: 100,
                load: Arc::new(std::sync::atomic::AtomicU32::new(0)),
                is_available: true,
                last_health_check: None,
            };

            manager.add_relay_server(relay_server).await;

            let peer_id = qudag_network::types::PeerId::random();
            let _result = manager.establish_relay(peer_id).await;

            black_box(())
        });
    });
}

fn benchmark_port_mapping_creation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("port_mapping_creation", |b| {
        b.to_async(&rt).iter(|| async {
            let config = NatTraversalConfig::default();
            let connection_manager = Arc::new(ConnectionManager::new(50));
            let nat_manager = NatTraversalManager::new(config, connection_manager);

            let _result = nat_manager
                .create_port_mapping(
                    black_box(8080),
                    black_box(8080),
                    black_box(PortMappingProtocol::TCP),
                )
                .await;

            black_box(())
        });
    });
}

fn benchmark_nat_manager_initialization(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("nat_manager_init", |b| {
        b.to_async(&rt).iter(|| async {
            let config = NatTraversalConfig {
                enable_stun: false, // Disable STUN for faster init
                enable_turn: false,
                enable_upnp: false,
                enable_nat_pmp: false,
                enable_hole_punching: true,
                enable_relay: true,
                enable_ipv6: false,
                stun_servers: vec![],
                turn_servers: vec![],
                max_relay_connections: 10,
                hole_punch_timeout: Duration::from_millis(100),
                detection_interval: Duration::from_secs(60),
                upgrade_interval: Duration::from_secs(30),
                port_mapping_lifetime: Duration::from_secs(300),
            };

            let connection_manager = Arc::new(ConnectionManager::new(50));
            let nat_manager = NatTraversalManager::new(config, connection_manager);

            let _result = nat_manager.initialize().await;

            black_box(())
        });
    });
}

fn benchmark_concurrent_connections(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("concurrent_connections");

    for connection_count in [1, 5, 10, 20, 50].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(connection_count),
            connection_count,
            |b, &connection_count| {
                b.to_async(&rt).iter(|| async move {
                    let config = NatTraversalConfig::default();
                    let connection_manager = Arc::new(ConnectionManager::new(100));
                    let nat_manager =
                        Arc::new(NatTraversalManager::new(config, connection_manager));

                    let tasks: Vec<_> = (0..connection_count)
                        .map(|_| {
                            let nat_manager = Arc::clone(&nat_manager);
                            tokio::spawn(async move {
                                let peer_id = qudag_network::types::PeerId::random();
                                nat_manager.connect_peer(peer_id).await
                            })
                        })
                        .collect();

                    let _results = futures::future::join_all(tasks).await;

                    black_box(())
                });
            },
        );
    }

    group.finish();
}

fn benchmark_statistics_collection(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("statistics_collection", |b| {
        b.to_async(&rt).iter(|| async {
            let config = NatTraversalConfig::default();
            let connection_manager = Arc::new(ConnectionManager::new(50));
            let nat_manager = NatTraversalManager::new(config, connection_manager);

            // Collect statistics multiple times to simulate monitoring
            for _ in 0..10 {
                let _stats = nat_manager.get_stats();
            }

            black_box(())
        });
    });
}

fn benchmark_config_creation(c: &mut Criterion) {
    c.bench_function("config_creation", |b| {
        b.iter(|| {
            let config = NatTraversalConfig {
                enable_stun: black_box(true),
                enable_turn: black_box(true),
                enable_upnp: black_box(true),
                enable_nat_pmp: black_box(true),
                enable_hole_punching: black_box(true),
                enable_relay: black_box(true),
                enable_ipv6: black_box(true),
                stun_servers: vec![
                    StunServer::new("stun1.l.google.com:19302".parse().unwrap(), 1),
                    StunServer::new("stun2.l.google.com:19302".parse().unwrap(), 2),
                ],
                turn_servers: vec![],
                max_relay_connections: black_box(50),
                hole_punch_timeout: Duration::from_secs(30),
                detection_interval: Duration::from_secs(300),
                upgrade_interval: Duration::from_secs(60),
                port_mapping_lifetime: Duration::from_secs(3600),
            };

            black_box(config)
        });
    });
}

fn benchmark_peer_id_operations(c: &mut Criterion) {
    c.bench_function("peer_id_generation", |b| {
        b.iter(|| {
            let peer_id = qudag_network::types::PeerId::random();
            black_box(peer_id)
        });
    });
}

fn benchmark_memory_usage(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("memory_footprint", |b| {
        b.to_async(&rt).iter(|| async {
            // Create multiple NAT managers to test memory usage
            let managers: Vec<_> = (0..10)
                .map(|_| {
                    let config = NatTraversalConfig::default();
                    let connection_manager = Arc::new(ConnectionManager::new(10));
                    NatTraversalManager::new(config, connection_manager)
                })
                .collect();

            // Perform some operations
            for manager in &managers {
                let _stats = manager.get_stats();
                let _nat_info = manager.get_nat_info();
            }

            black_box(managers)
        });
    });
}

criterion_group!(
    benches,
    benchmark_nat_detection,
    benchmark_stun_server_creation,
    benchmark_hole_punch_setup,
    benchmark_relay_manager_operations,
    benchmark_port_mapping_creation,
    benchmark_nat_manager_initialization,
    benchmark_concurrent_connections,
    benchmark_statistics_collection,
    benchmark_config_creation,
    benchmark_peer_id_operations,
    benchmark_memory_usage,
);

criterion_main!(benches);
