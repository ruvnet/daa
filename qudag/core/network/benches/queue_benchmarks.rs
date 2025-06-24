use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use futures::future::join_all;
use qudag_network::MessageQueue;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use std::time::Duration;
use tokio::sync::mpsc;

const MSG_SIZES: [usize; 3] = [64, 1024, 65536]; // 64B, 1KB, 64KB
const BATCH_SIZES: [usize; 3] = [32, 128, 512];

async fn bench_message_throughput(
    queue: &MessageQueue,
    msg_size: usize,
    batch_size: usize,
    duration: Duration,
) -> f64 {
    let start = std::time::Instant::now();
    let msg_count = Arc::new(AtomicU64::new(0));
    let msg = vec![0u8; msg_size];

    let msg_count_clone = msg_count.clone();
    let send_task = tokio::spawn(async move {
        while start.elapsed() < duration {
            // Send message batch
            for _ in 0..batch_size {
                if queue.send(msg.clone()).await.is_ok() {
                    msg_count_clone.fetch_add(1, Ordering::Relaxed);
                }
            }
        }
    });

    let recv_task = tokio::spawn(async move {
        while start.elapsed() < duration {
            if let Some(_) = queue.receive().await {
                continue;
            }
        }
    });

    // Wait for tasks to complete
    let _ = join_all(vec![send_task, recv_task]).await;

    let count = msg_count.load(Ordering::Relaxed);
    let elapsed = start.elapsed().as_secs_f64();
    count as f64 / elapsed
}

fn bench_queue_performance(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("message_queue");
    group.measurement_time(Duration::from_secs(10));

    for &msg_size in MSG_SIZES.iter() {
        for &batch_size in BATCH_SIZES.iter() {
            group.bench_with_input(
                BenchmarkId::new(format!("throughput_{}b", msg_size), batch_size),
                &batch_size,
                |b, &batch_size| {
                    b.to_async(&rt).iter(|| async {
                        let (queue, _rx) = MessageQueue::with_batch_size(batch_size);
                        black_box(
                            bench_message_throughput(
                                &queue,
                                msg_size,
                                batch_size,
                                Duration::from_secs(1),
                            )
                            .await,
                        )
                    })
                },
            );
        }
    }

    group.finish();
}

fn bench_latency(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("message_latency");

    for &msg_size in MSG_SIZES.iter() {
        group.bench_with_input(
            BenchmarkId::new("latency", msg_size),
            &msg_size,
            |b, &msg_size| {
                let (queue, _rx) = MessageQueue::new();
                let msg = vec![0u8; msg_size];

                b.to_async(&rt).iter(|| async {
                    let start = std::time::Instant::now();
                    black_box(queue.send(msg.clone()).await.unwrap());
                    black_box(start.elapsed())
                })
            },
        );
    }

    group.finish();
}

fn bench_batch_processing(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("batch_processing");

    for &batch_size in BATCH_SIZES.iter() {
        group.bench_with_input(
            BenchmarkId::new("batch", batch_size),
            &batch_size,
            |b, &batch_size| {
                let (queue, mut rx) = MessageQueue::with_batch_size(batch_size);
                let msg = vec![0u8; 1024]; // 1KB messages

                b.to_async(&rt).iter(|| async {
                    // Send batch
                    for _ in 0..batch_size {
                        black_box(queue.send(msg.clone()).await.unwrap());
                    }

                    // Receive batch
                    for _ in 0..batch_size {
                        black_box(rx.recv().await.unwrap());
                    }
                })
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_queue_performance,
    bench_latency,
    bench_batch_processing
);
criterion_main!(benches);
