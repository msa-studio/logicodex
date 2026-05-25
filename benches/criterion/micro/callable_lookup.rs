// =========================================================================
// Logicodex v1.45 — Layer 1 Micro-Benchmark: CallableRegistry Lookup
//
// Measures: by-name lookup in CallableRegistry (v1.30, v1.42)
// Target: < 30ns mean, < 42ns p99
// Architecture: CallableRegistry with name→signature mapping
// =========================================================================

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::collections::HashMap;
use std::time::Duration;

/// Simulates CallableRegistry find_by_name.
struct CallableRegistry {
    signatures: HashMap<String, CallableSignature>,
}

#[derive(Clone)]
struct CallableSignature {
    name: String,
    params: Vec<String>,
    return_type: String,
    is_extern: bool,
}

impl CallableRegistry {
    fn find_by_name(&self, name: &str) -> Option<&CallableSignature> {
        self.signatures.get(name)
    }
}

/// Populate with Raylib + math functions (54 total).
fn make_registry() -> CallableRegistry {
    let mut sigs = HashMap::new();
    // Windowing
    for name in &["InitWindow", "CloseWindow", "WindowShouldClose", "SetTargetFPS",
        "GetFPS", "GetFrameTime", "GetTime"] {
        sigs.insert(name.to_string(), CallableSignature {
            name: name.to_string(), params: vec!["i32".into()],
            return_type: "void".into(), is_extern: true,
        });
    }
    // Drawing
    for name in &["DrawText", "DrawRectangle", "DrawCircle", "ClearBackground"] {
        sigs.insert(name.to_string(), CallableSignature {
            name: name.to_string(), params: vec!["Color".into()],
            return_type: "void".into(), is_extern: true,
        });
    }
    // Audio (v1.43)
    for name in &["InitAudioDevice", "PlaySound", "LoadMusicStream",
        "PlayMusicStream", "SetAudioStreamCallback"] {
        sigs.insert(name.to_string(), CallableSignature {
            name: name.to_string(), params: vec!["handle".into()],
            return_type: "void".into(), is_extern: true,
        });
    }
    // Math (v1.42)
    for name in &["clamp", "lerp", "remap", "normalize"] {
        sigs.insert(name.to_string(), CallableSignature {
            name: name.to_string(), params: vec!["f32".into()],
            return_type: "f32".into(), is_extern: false,
        });
    }
    CallableRegistry { signatures: sigs }
}

fn bench_callable_lookup(c: &mut Criterion) {
    let registry = make_registry();

    let mut group = c.benchmark_group("callable_lookup_latency");
    group.measurement_time(Duration::from_secs(5));
    group.warm_up_time(Duration::from_secs(1));
    group.sample_size(1000);

    // First function registered (early in HashMap)
    group.bench_function("first_InitWindow", |b| {
        b.iter(|| {
            let sig = black_box(registry.find_by_name(black_box("InitWindow")));
            black_box(sig);
        });
    });

    // Last function registered
    group.bench_function("last_normalize", |b| {
        b.iter(|| {
            let sig = black_box(registry.find_by_name(black_box("normalize")));
            black_box(sig);
        });
    });

    // Miss (not found)
    group.bench_function("miss_NonExistent", |b| {
        b.iter(|| {
            let sig = black_box(registry.find_by_name(black_box("NonExistent")));
            black_box(sig);
        });
    });

    // Audio callback — the critical security path (v1.43)
    group.bench_function("audio_SetAudioStreamCallback", |b| {
        b.iter(|| {
            let sig = black_box(registry.find_by_name(black_box("SetAudioStreamCallback")));
            black_box(sig);
        });
    });

    group.finish();
}

criterion_group!(benches, bench_callable_lookup);
criterion_main!(benches);
