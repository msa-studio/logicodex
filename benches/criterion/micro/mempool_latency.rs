// =========================================================================
// Logicodex v1.45 — Layer 1 Micro-Benchmark: MemoryPool Latency
//
// Measures: Bump allocator acquire/release (v1.44 G4)
// Target: < 20ns mean, < 30ns p99
// Architecture: Bump allocator with atomic CAS
// =========================================================================

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::alloc::{GlobalAlloc, Layout};
use std::time::Duration;

/// Simulated bump allocator (mirrors src/os/allocator.rs).
struct SimBumpAlloc {
    next: std::sync::atomic::AtomicUsize,
    end: usize,
}

impl SimBumpAlloc {
    fn new(start: usize, size: usize) -> Self {
        Self {
            next: std::sync::atomic::AtomicUsize::new(start),
            end: start + size,
        }
    }

    fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut current = self.next.load(std::sync::atomic::Ordering::Relaxed);
        loop {
            let aligned = (current + layout.align() - 1) & !(layout.align() - 1);
            let new_next = aligned + layout.size();
            if new_next > self.end {
                return std::ptr::null_mut();
            }
            match self.next.compare_exchange_weak(
                current,
                new_next,
                std::sync::atomic::Ordering::SeqCst,
                std::sync::atomic::Ordering::Relaxed,
            ) {
                Ok(_) => return aligned as *mut u8,
                Err(actual) => current = actual,
            }
        }
    }
}

fn bench_mempool(c: &mut Criterion) {
    let mut group = c.benchmark_group("mempool_latency");
    group.measurement_time(Duration::from_secs(5));
    group.warm_up_time(Duration::from_secs(1));
    group.sample_size(1000);

    // Benchmark: allocate varying sizes
    for size in [8, 64, 256, 1024, 4096] {
        group.bench_with_input(
            BenchmarkId::new("acquire", format!("{}B", size)),
            &size,
            |b, &sz| {
                let pool = SimBumpAlloc::new(0x400000, 0x100000); // 1MB pool
                let layout = Layout::from_size_align(sz, 8).unwrap();
                b.iter(|| {
                    let ptr = black_box(pool.alloc(layout));
                    black_box(ptr);
                });
            },
        );
    }

    // Benchmark: allocate + use (touch memory)
    group.bench_function("acquire_use_64B", |b| {
        let pool = SimBumpAlloc::new(0x400000, 0x100000);
        let layout = Layout::from_size_align(64, 8).unwrap();
        b.iter(|| {
            let ptr = pool.alloc(layout);
            if !ptr.is_null() {
                unsafe { std::ptr::write_volatile(ptr.offset(32), 0xDEu8); }
            }
            black_box(ptr);
        });
    });

    group.finish();
}

criterion_group!(benches, bench_mempool);
criterion_main!(benches);
