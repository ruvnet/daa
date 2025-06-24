use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use qudag_network::{message::Message, peer::Peer, PeerId};
use rand::RngCore;
use std::time::Duration;

const PEER_COUNTS: [usize; 3] = [100, 1000, 10000];
const MSG_SIZES: [usize; 3] = [64, 1024, 65536];

fn generate_random_data(size: usize) -> Vec<u8> {
    let mut data = vec![0u8; size];
    rand::thread_rng().fill_bytes(&mut data);
    data
}

fn bench_peer_messaging(c: &mut Criterion) {
    let mut group = c.benchmark_group("peer_messaging");

    for &peer_count in PEER_COUNTS.iter() {
        for &msg_size in MSG_SIZES.iter() {
            group.bench_with_input(
                BenchmarkId::new(format!("peers_{}", peer_count), msg_size),
                &(peer_count, msg_size),
                |b, &(peer_count, msg_size)| {
                    let mut peers = Vec::with_capacity(peer_count);
                    for i in 0..peer_count {
                        peers.push(Peer::new(format!("peer{}", i)));
                    }

                    let data = generate_random_data(msg_size);
                    let msg = Message::new(&data);

                    b.iter(|| {
                        for peer in peers.iter() {
                            black_box(peer.send_message(msg.clone()));
                        }
                    });
                },
            );
        }
    }

    group.finish();
}

fn bench_peer_discovery(c: &mut Criterion) {
    let mut group = c.benchmark_group("peer_discovery");

    for &peer_count in PEER_COUNTS.iter() {
        group.bench_with_input(
            BenchmarkId::new("discovery", peer_count),
            &peer_count,
            |b, &peer_count| {
                let mut peers = Vec::with_capacity(peer_count);
                for i in 0..peer_count {
                    peers.push(Peer::new(format!("peer{}", i)));
                }

                b.iter(|| {
                    for peer in peers.iter() {
                        black_box(peer.discover_peers());
                    }
                });
            },
        );
    }

    group.finish();
}

fn bench_peer_handshake(c: &mut Criterion) {
    let mut group = c.benchmark_group("peer_handshake");

    for &peer_count in PEER_COUNTS.iter() {
        group.bench_with_input(
            BenchmarkId::new("handshake", peer_count),
            &peer_count,
            |b, &peer_count| {
                let mut peers = Vec::with_capacity(peer_count);
                for i in 0..peer_count {
                    peers.push(Peer::new(format!("peer{}", i)));
                }

                b.iter(|| {
                    for i in 0..peer_count.saturating_sub(1) {
                        black_box(peers[i].handshake(&peers[i + 1]));
                    }
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_peer_messaging,
    bench_peer_discovery,
    bench_peer_handshake
);
criterion_main!(benches);
