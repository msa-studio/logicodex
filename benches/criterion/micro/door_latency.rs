// =========================================================================
// Logicodex v1.45 — Layer 1 Micro-Benchmark: Door (Channel) Latency
//
// Measures: Channel<T> send and receive overhead (v1.30 zero-copy)
// Target: < 100ns mean, < 120ns p99 for both send and recv
// Architecture: Actor-model concurrency — zero-copy ownership transfer
// =========================================================================

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::sync::mpsc;
use std::time::Duration;

/// Single-threaded channel send benchmark.
fn bench_door_send(c: &mut Criterion) {
    let mut group = c.benchmark_group("door_send_latency");
    group.measurement_time(Duration::from_secs(5));
    group.warm_up_time(Duration::from_secs(1));
    group.sample_size(1000);

    for payload_size in [8, 64, 256, 1024] {
        let payload = vec![0u8; payload_size];
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}B", payload_size)),
            &payload,
            |b, data| {
                let (tx, _rx) = mpsc::channel::<Vec<u8>>();
                let data = data.clone();
                b.iter(|| {
                    let _ = tx.send(black_box(data.clone()));
                });
            },
        );
    }

    group.finish();
}

/// Single-threaded channel recv benchmark.
fn bench_door_recv(c: &mut Criterion) {
    let mut group = c.benchmark_group("door_recv_latency");
    group.measurement_time(Duration::from_secs(5));
    group.warm_up_time(Duration::from_secs(1));
    group.sample_size(1000);

    for payload_size in [8, 64, 256, 1024] {
        let payload = vec![0u8; payload_size];
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}B", payload_size)),
            &payload,
            |b, data| {
                b.iter_custom(|iters| {
                    let (tx, rx) = mpsc::channel::<Vec<u8>>();
                    // Pre-fill channel
                    for _ in 0..iters {
                        tx.send(data.clone()).unwrap();
                    }
                    let start = std::time::Instant::now();
                    for _ in 0..iters {
                        let _ = black_box(rx.recv().unwrap());
                    }
                    start.elapsed()
                });
            },
        );
    }

    group.finish();
}

/// Round-trip: send + recv latency.
fn bench_door_roundtrip(c: &mut Criterion) {
    let mut group = c.benchmark_group("door_roundtrip_latency");
    group.measurement_time(Duration::from_secs(5));
    group.warm_up_time(Duration::from_secs(1));
    group.sample_size(1000);

    group.bench_function("8B_pingpong", |b| {
        b.iter_custom(|iters| {
            let (tx1, rx1) = mpsc::channel::<u64>();
            let (tx2, rx2) = mpsc::channel::<u64>();

            // Spawn receiver-echo thread
            std::thread::spawn(move || {
                while let Ok(v) = rx1.recv() {
                    if tx2.send(v).is_err() { break; }
                }
            });

            let start = std::time::Instant::now();
            for i in 0..iters {
                tx1.send(black_box(i as u64)).unwrap();
                let _ = black_box(rx2.recv().unwrap());
            }
            start.elapsed()
        });
    });

    group.finish();
}

criterion_group!(benches, bench_door_send, bench_door_recv, bench_door_roundtrip);
criterion_main!(benches);
