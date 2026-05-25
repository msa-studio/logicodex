// =========================================================================
// Logicodex v1.45 — Layer 1 Micro-Benchmark: Gate Invocation Latency
//
// Measures: Capability Fabric gate check cost (v1.32)
// Target: < 50ns mean, < 68ns p99
// Architecture: Capability Fabric — Gate/Door split
// =========================================================================

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::time::Duration;

/// Minimal gate check simulation.
/// Mirrors the logic in `src/tier2/gate.rs::check_access()`.
fn gate_check(capability: &str, gate: &str, topology: &[(String, Vec<String>)]) -> bool {
    for (domain, gates) in topology {
        if domain == capability {
            return gates.iter().any(|g| g == gate);
        }
    }
    false
}

fn bench_gate_latency(c: &mut Criterion) {
    // Simulate capability topology (Audio domain with 2 gates)
    let topology: Vec<(String, Vec<String>)> = vec![
        ("Audio".into(), vec!["Main".into(), "Rakam".into()]),
        ("Net".into(), vec!["Admin".into(), "User".into()]),
        ("Gpio".into(), vec!["Read".into(), "Write".into()]),
    ];

    let mut group = c.benchmark_group("gate_invoke_latency");
    group.measurement_time(Duration::from_secs(5));
    group.warm_up_time(Duration::from_secs(1));
    group.sample_size(1000);

    // Benchmark: Audio.Main gate check (hit)
    group.bench_with_input(
        BenchmarkId::new("hit", "Audio.Main"),
        &("Audio", "Main"),
        |b, (cap, gate)| {
            b.iter(|| gate_check(black_box(cap), black_box(gate), black_box(&topology)));
        },
    );

    // Benchmark: Audio.Admin gate check (miss — fallthrough)
    group.bench_with_input(
        BenchmarkId::new("miss", "Audio.Admin"),
        &("Audio", "Admin"),
        |b, (cap, gate)| {
            b.iter(|| gate_check(black_box(cap), black_box(gate), black_box(&topology)));
        },
    );

    // Benchmark: Crypto.Sign gate check (domain not found)
    group.bench_with_input(
        BenchmarkId::new("not_found", "Crypto.Sign"),
        &("Crypto", "Sign"),
        |b, (cap, gate)| {
            b.iter(|| gate_check(black_box(cap), black_box(gate), black_box(&topology)));
        },
    );

    group.finish();
}

criterion_group!(benches, bench_gate_latency);
criterion_main!(benches);
