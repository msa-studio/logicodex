# Logicodex Language — v1.45.0-alpha
## Quantitative Benchmark Framework

> v1.21 Compiler Baseline → v1.30 Threading + IO + Audio → v1.31 Streaming Engine → v1.32 Capability Fabric → v1.33 Network Reactor → v1.34 Sharded Reactor → v1.35 Capability IR → v1.36 CTL Mapper → v1.37 Network Runtime → v1.38 Deferred Cleanup → v1.39 Sharded Runtime → v1.40 WASM Codegen → v1.41 Host Reactor → v1.42 Raylib FFI → v1.43 Raylib Audio → **v1.44 Freestanding Compiler**

Logicodex is a systems programming language with **zero runtime mediation** — all security, scheduling, and hardware access verified at compile time. It compiles to both **Native (ELF)** and **WebAssembly (WASM)** via LLVM.

> **"Project capability model INTO the WASM ecosystem, not borrow from it."**
> 
> **"WASM Guest = Unit Logik — NO direct hardware access."**

---

## Quick Stats

| Metric | Value |
|---|---|
| **Total LOC** | ~43,600 |
| **Validators** | **148/148 ✅** |
| **Deferred Items** | **25/25 resolved** (1 by design) |
| **Releases** | v1.21 → v1.45 (14 releases) |
| **Backends** | Native (ELF), WASM (wasm32-unknown-unknown), Freestanding (x86_64/aarch64/riscv64) |

---

## v1.30-v1.41 Capability Overview

Logicodex has evolved from a compiler-core prototype into a **deterministic systems platform with WASM integration** through 10 consecutive alpha releases:

| Release | Focus | Key Innovation |
|---|---|---|
| **v1.30.1-alpha** | Threading + IO + Audio | Actor-model concurrency (`actor`/`channel`), zero-copy ownership transfer, 4-Ketuk IO architecture, hardware-safe audio engine |
| **v1.31.0-alpha** | Streaming Compiler | 2-Pass Engine — RAM stays flat regardless of program size; SemanticSummary (~64B/symbol) replaces full AST |
| **v1.32.0-alpha** | Capability Security | Static Capability Fabric — Gate/Door split, compile-time topology verification, supply-chain `.cap` files, privilege escalation detection |
| **v1.33.0-alpha** | Network Reactor | Deterministic event-driven networking — RAII auto-cleanup (no socket leaks), taint state machine, backpressure policies, service manifest syntax |
| **v1.34.0-alpha** | Sharded Multi-Core Reactor | Per-CPU-core reactor instances, static affinity mapping, cross-shard SPSC doors, memory budgeting |
| **v1.35.0-alpha** | CapabilityGraph IR | Single Source of Truth IR — unifies SemanticSummary + CapabilityTopology + ShardTopology; generates Native/`.cap`/WIT |
| **v1.36.0-alpha** | CTL Mapper | Auto-generates WIT from CapabilityGraph — 6 domain mappings, manual overrides, HW gate host reactor stubs |
| **v1.37.0-alpha** | Network Runtime | Live epoll event loop, SYS_RECV/SYS_SEND syscalls, monotonic timestamp, continuous event processing |
| **v1.38.0-alpha** | Deferred Cleanup | CallableRegistry predeclare, topology fix, enum layout, Windows fallback, secure attestation, freestanding target, semantic gatekeeper |
| **v1.39.0-alpha** | Sharded Runtime | Real OS thread per shard, parallel execution, CPU affinity via direct syscall (Linux: sched_setaffinity) |
| **v1.40.0-alpha** | WASM Codegen Backend | LLVM → .wasm via wasm32-unknown-unknown target — `--target wasm` CLI |
| **v1.41.0-alpha** | Host Reactor | Guest ↔ Host HW mediation: GPIO, Timer, DMA. Permission-based pin allowlists. HostFunction dispatch protocol |
| **v1.42.0-alpha** | Raylib FFI — 8 Pending Items | Struct-by-value Color, Vector2/Rectangle constructors, math utilities (clamp/lerp/remap), StrictAudioContext, WASM blocks Raylib, FfiGatekeeper coercion |
| **v1.43.0-alpha** | Raylib Audio — 22 Functions | Sound/Music/Wave/AudioStream types, 22 audio functions, StrictAudioContext integration with capability gates |
| **v1.44.0-alpha** | Freestanding Compiler | Bare-metal support: _start, panic handler, linker script, bump allocator, UART/VGA, IDT/PIC, MMIO volatile codegen, multiboot. 3 architectures: x86_64/aarch64/riscv64 |
| **v1.44.1-alpha** | Foundation Polish | Validator tiering (A/B/C), maintenance report, dead code audit, security micro-audit, documentation drift fix |
| **v1.45.0-alpha** | **Quantitative Benchmark Framework** | **Architecture-correlated benchmarks: 6 micro (criterion), reactor throughput, RSS stability, security stress. BASELINE.json + regression detection + RFC template** |

---

## Architecture: Door + Gate + Service + IR + CTL + Host

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        APPLICATION LAYER                                 │
│                                                                          │
│    actor Worker {          service WebServer {          ┌───────────┐    │
│        let ch: Channel<...>      port: 443              │ WIT Output│    │
│        ch.send(data)             requires: Net.Admin    │  (WASM)   │    │
│    }                             handler: WebHandler    └─────┬─────┘    │
│                                  policy: Block                │           │
│    spawn Worker()            }                                │ CTL       │
│                                                               │ Mapper    │
├──────────────┬──────────────┬──────────────────┬──────────────┴───────────┤
│    DOOR      │     GATE     │     SERVICE      │     HostReactor          │
│  (SPSC Ring  │  (Compile-   │  (Port Actor     │  (Guest → Host HW       │
│   Buffer)    │   time Cap)  │   + Reactor)     │   Mediation)             │
│ Zero-copy    │ Zero Runtime │ RAII Cleanup     │ Permission-based         │
│ Lock-free    │  Mediation   │ Taint FSM        │ Pin claim/release        │
├──────────────┴──────────────┴──────────────────┴─────────────────────────┤
│                    CAPABILITY LAYER (v1.32-v1.36)                        │
│                                                                          │
│   CapabilityGraph IR ──► CTL Mapper ──► WIT / Native / .cap             │
│   (Single Source of    (6 domain        (all outputs from                │
│    Truth)               mappings)        one unified IR)                 │
├─────────────────────────────────────────────────────────────────────────┤
│                    SHARDED LAYER (v1.34, v1.39)                          │
│                                                                          │
│   ShardTopology ──► ShardedReactor ──► Vec<ShardInstance>               │
│   (compile-time     (v1.39: real        (per-core threads +             │
│    verify)          OS threads)          CPU affinity)                    │
├─────────────────────────────────────────────────────────────────────────┤
│                    NETWORK LAYER (v1.33, v1.37)                          │
│                                                                          │
│   Reactor (epoll)  Connection (fd+Taint)  Backpressure                 │
│   event loop       RAII auto-cleanup       Block/DropOldest/Error       │
├─────────────────────────────────────────────────────────────────────────┤
│                    COMPILER LAYER                                        │
│                                                                          │
│   Tier 1: Lexer → Parser → AST → Semantic → HIR                        │
│   Tier 2: CapabilityGraph IR → Streaming Engine                        │
│   Tier 3: LLVM Codegen → Native / WASM / Freestanding                  │
├─────────────────────────────────────────────────────────────────────────┤
│                    RUNTIME PRIMITIVES                                    │
│                                                                          │
│   Syscalls: SYS_RECV/SYS_SEND/SYS_CLOSE/EPOLL/CLOCK                   │
│   Ring Buffer: SPSC with Block/DropOldest/Error policies              │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Usage

### Native Target (default)
```bash
logicodex input.ldx -o output.o
```

### WebAssembly Target
```bash
logicodex --target wasm input.ldx -o output.wasm
wasm-ld --no-entry -o final.wasm output.wasm --export-all
```

### Freestanding Target
```bash
logicodex --target freestanding input.ldx -o kernel.o
```

### Secure Build
```bash
logicodex --secure input.ldx -o output.o
```

---

## Validation

**102/102 checks passing — zero regression across all versions.**

```
v1.41 Host Reactor:         12/12 ✅
v1.40 WASM Backend:         13/13 ✅
v1.39 Sharded Runtime:      11/11 ✅
v1.38 Deferred Cleanup:     12/12 ✅
v1.37 Network Runtime:      13/13 ✅
v1.36 CTL Mapper:           12/12 ✅
v1.35 Capability IR:        16/16 ✅
v1.34 Sharded Reactor:      11/11 ✅
v1.33 Network Reactor:      13/13 ✅
v1.32 Capability Fabric:    10/10 ✅
v1.31 Streaming Engine:       6/6  ✅
v1.21 Baseline:               9/9  ✅
─────────────────────────────────────
TOTAL:                     102/102 ✅
```

Run all validators:
```bash
for v in scripts/validate_*.py; do python3 "$v"; done
```

---

## Project Stats

| Component | Files | LOC |
|---|---|---|
| Rust Source (`src/`) | 43 | 17,083 |
| Tests (`tests/`) | 31 | 9,230 |
| Validators (`scripts/`) | 34 | 6,675 |
| Documentation (`docs/` + root) | 17 | 3,578 |
| Library (`lib/`) | 13 | — |
| **TOTAL** | **139** | **~37,500** |

---

## Documentation

| Document | Description |
|---|---|
| `docs/AUDIT.md` | **Full project audit** (this release) |
| `docs/ARCHITECTURE.md` | Complete architecture overview |
| `docs/v1.30-THREADING.md` | Threading Phases 1-3 |
| `docs/v1.31-STREAMING.md` | Tier 2 Streaming Semantic Compiler |
| `docs/v1.32-CAPABILITY.md` | Static Capability Fabric |
| `docs/v1.33-REACTOR.md` | Deterministic Network Reactor |
| `docs/v1.34-SHARDED.md` | Sharded Deterministic Reactor |
| `docs/v1.35-CAPABILITY-IR.md` | CapabilityGraph IR |
| `docs/v1.36-CTL-MAPPER.md` | CTL Mapper |
| `docs/v1.37-NETWORK-RUNTIME.md` | Network Runtime — epoll, socket I/O |
| `ROADMAP.md` | Future milestones |
| `CHANGELOG.md` | Release history |
| `DEFERRED.md` | All deferred items (25/26 resolved) |

---

## Roadmap

| Version | Focus | Status |
|---|---|---|
| v1.36 | Codegen A1-A5 | ✅ COMPLETED |
| v1.37 | Network Runtime B1-B6 | ✅ COMPLETED |
| v1.38 | Deferred Cleanup | ✅ COMPLETED |
| v1.39 | Sharded Runtime C1-C5 | ✅ COMPLETED |
| v1.40 | WASM Backend | ✅ COMPLETED |
| v1.41 | Host Reactor | ✅ COMPLETED |
| v1.42 | Streaming WASM + Runtime Capability Verification | 🔬 RESEARCH |
| v2.00 | Pointer Provenance Engine | 🔬 RESEARCH |

---

*Logicodex Language — v1.41.0-alpha*
*Architect: Mohamad Supardi Abdul (mymsastudio@gmail.com)*
*2026-05-25*
