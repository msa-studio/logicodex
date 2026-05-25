# Appendix B: Timeline Evolusi

Garis masa pembangunan Logicodex dari v1.21 hingga v1.45.

---

## 2026 Q1: Foundation

| Tarikh | Peristiwa |
|---|---|
| Januari | v1.21.0-alpha — Compiler core: lexer, parser, AST, semantic analyzer, LLVM backend |
| Januari | Sprint 1.1 — TypeRegistry, CoercionEngine, Raylib FFI (28 functions) |
| Januari | Sprint 1.2 — Parser Type Injection, TypeChecker, bilingual errors |
| Februari | Sprint 2 — LayoutEngine, struct layout, memory offsets |
| Februari | Sprint 2.5 — Struct Literals, struct constructors |
| Februari | Sprint 3 — Codegen Calls, function calls, LLVM call generation |

## 2026 Q2: Platform (v1.30-v1.34)

| Tarikh | Peristiwa |
|---|---|
| Mac | v1.30.0-alpha — Threading + IO + Audio (3 phases) |
| Mac | Phase 1 — Actor & Channel, topology validation (8/8 tests) |
| Mac | Phase 2 — Zero-copy ownership transfer via SPSC ring buffer (6/6 tests) |
| Mac | Phase 3 — Backpressure policies + scheduler (10/10 tests) |
| Mac | K1-K4 IO Architecture — Slice<T>, Buffer<T>, Result<T,E>, file handle + syscall |
| April | v1.31.0-alpha — Streaming Engine, 2-Pass Engine, SemanticSummary (~64B/symbol) |
| April | v1.32.0-alpha — Capability Fabric, Gate/Door split, 3 gate types, topology verify |
| April | v1.33.0-alpha — Network Reactor, RAII Connection, Taint FSM, Service manifest |
| Mei | v1.34.0-alpha — Sharded Reactor, per-CPU-core instances, CPU affinity, cross-shard doors |

## 2026 Q2: Translation Layer (v1.35-v1.36)

| Tarikh | Peristiwa |
|---|---|
| Mei | v1.35.0-alpha — CapabilityGraph IR, Single Source of Truth, 6 verify() checks |
| Mei | v1.36.0-alpha — CTL Mapper, WIT generation, 6 domain mappings, host reactor stubs |

## 2026 Q2: Runtime (v1.37-v1.41)

| Tarikh | Peristiwa |
|---|---|
| Mei | v1.37.0-alpha — Network Runtime LIVE, epoll event loop, direct syscalls |
| Mei | v1.38.0-alpha — Deferred Cleanup, 8 items resolved (A6, D1, E1-E2, F1, G1-G2, I1) |
| Mei | v1.39.0-alpha — Sharded Runtime LIVE, real OS threads, CPU affinity via sched_setaffinity |
| Mei | v1.40.0-alpha — WASM Backend, wasm32-unknown-unknown target, LLVM features |
| Mei | v1.41.0-alpha — Host Reactor, Guest↔Host HW mediation, GatePermissions, HardwareZone |

## 2026 Q2: Graphics + Audio (v1.42-v1.43)

| Tarikh | Peristiwa |
|---|---|
| Mei | v1.42.0-alpha — Raylib FFI, 8 pending items resolved, struct-by-value, math shims |
| Mei | v1.43.0-alpha — Raylib Audio, 22 functions, StrictAudioContext integration |

## 2026 Q2: Freestanding + Stabilization (v1.44-v1.45)

| Tarikh | Peristiwa |
|---|---|
| Mei | v1.44.0-alpha — Freestanding Compiler, 15 gaps resolved, 3 architectures |
| Mei | v1.44.1-alpha — Foundation Polish, validator tiering (A/B/C), maintenance report |
| Mei | v1.45.0-alpha — Quantitative Benchmark Framework, 4 layers, BASELINE.json |
| Mei | **Architecture Freeze** — RFC process formalized, 4 mandatory alignment checks |

## 2026+ (Planned)

| Jangkaan | Peristiwa |
|---|---|
| 2026 H2 | v1.46.0-alpha — Streaming WASM, WASI capability verification |
| 2026 H2 | ldx-fmt formatter, LSP diagnostics |
| 2027 | v2.00.0-alpha — Pointer Provenance Engine (5-level) |
| 2027+ | Logicodex Migrator, AI Repair Loop, Global Token Registry |

---

## Statistik Kumulatif Mengikut Release

| Release | LOC | Tests | Validators | Deferred Resolved |
|---|---|---|---|---|
| v1.21 | ~5,000 | 9 | 9 | — |
| v1.30 | ~12,000 | 104 | 104 | — |
| v1.31 | ~14,000 | 110 | 110 | — |
| v1.32 | ~16,000 | 120 | 120 | — |
| v1.33 | ~20,000 | 133 | 133 | — |
| v1.34 | ~24,000 | 144 | 144 | — |
| v1.35 | ~26,000 | 160 | 160 | — |
| v1.36 | ~28,000 | 172 | 172 | — |
| v1.37 | ~31,000 | 185 | 185 | — |
| v1.38 | ~33,000 | 197 | 197 | 8 resolved |
| v1.39 | ~35,000 | 207 | 207 | — |
| v1.40 | ~37,000 | 220 | 220 | — |
| v1.41 | ~39,000 | 232 | 232 | — |
| v1.42 | ~40,000 | 241 | 241 | — |
| v1.43 | ~41,000 | 250 | 250 | — |
| v1.44 | ~42,000 | 265 | 265 | 15 resolved |
| v1.44.1 | ~42,500 | 275 | 275 | — |
| **v1.45** | **~43,600** | **400+** | **148** | **25 resolved** |
