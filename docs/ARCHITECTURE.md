# Logicodex Systems Architecture: Door + Gate + Service + IR + CTL

> **"The Capability Translation Layer"**

Architect: Mohamad Supardi Abdul (mymsastudio@gmail.com)

---

## Philosophy

Logicodex bukan sekadar bahasa pengaturcaraan — ia adalah **"Hardware-Integrated Systems Platform"** yang menggabungkan 6 tiang utama:

1. **Provenance Memory** (K1-K4) — Memori dengan jejak keaslian
2. **Deterministic Concurrency** (Actor & Channel) — Zero-copy actor model
3. **Capability Fabric** (Security Gate) — Compile-time security
4. **Network Reactor** (Deterministic I/O) — Event-driven networking
5. **Sharded Reactor** (Multi-Core) — Per-CPU-core deterministic instances
6. **Capability Translation** (WASM) — Project INTO, not borrow FROM

> **"Mustahil untuk mengalami race condition atau memory leak"** — kerana semuanya diverifikasi pada masa kompil.
>
> **"WASM Guest = Unit Logik — NO direct hardware access"** — All hardware through Capability Gates → Host Reactor.

---

## The Unified Model: Door + Gate + Service

```
┌─────────────────────────────────────────────────────────────────┐
│                     APPLICATION LAYER                            │
│                                                                  │
│    actor Worker {          service WebServer {                   │
│        let ch: Channel<...>      port: 443                       │
│        ch.send(data)             requires: Net.Admin             │
│    }                             handler: WebHandler             │
│                                  policy: Block                   │
│    spawn Worker()            }                                   │
│                                                                  │
├──────────────────────┬──────────────────────┬───────────────────┤
│        DOOR          │        GATE          │      SERVICE      │
│    (Data Transport)  │   (Capability)       │  (Event Loop)     │
│                      │                      │                   │
│  SPSC Ring Buffer    │  Compile-time        │  Port-based Actor │
│  Zero-copy           │  Security Contract   │  + Reactor        │
│  Lock-free           │  Zero Runtime        │  + Connection     │
│                      │  Mediation           │  + Taint FSM      │
├──────────────────────┴──────────────────────┴───────────────────┤
│                     COMPILER LAYER                               │
│                                                                  │
│   Tier 1: Parser (AST) → Tier 2: Metadata → Tier 3: Codegen    │
│   Full AST           →  SemanticSummary   →  LLVM IR           │
│   (temporary)          (persistent)          (streamed)         │
│                                                                  │
│   2-Pass Engine:                                                 │
│   Pass 1: Pre-declare (lightning scan)                           │
│   Pass 2: Deep streaming (analyze + codegen + discard)           │
│                                                                  │
├─────────────────────────────────────────────────────────────────┤
│                     RUNTIME LAYER                                │
│                                                                  │
│   RAII Connection Drop → close(fd) deterministik                 │
│   Taint State Machine  → Healthy → Suspicious → Closing          │
│   Backpressure Policy  → Block / DropOldest / Error              │
│   epoll/kqueue/IOCP    → Event-driven I/O                        │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## Door — Data Transport

| Attribute | Value |
|---|---|
| **What** | SPSC Ring Buffer — lock-free, zero-copy |
| **When** | Semua komunikasi antara actors/channels |
| **Memory Ordering** | Producer: Release, Consumer: Acquire |
| **File** | `lib/core/ring_buffer.ldx` |
| **Introduced** | v1.30.1-alpha Phase 3 |

**Functions:**
- `ring_send()` — blocking send
- `ring_recv()` — blocking receive
- `ring_try_send()` — non-blocking (Result)
- `ring_try_recv()` — non-blocking (Option)
- `ring_timeout_recv()` — with timeout

---

## Gate — Capability Contract

| Attribute | Value |
|---|---|
| **What** | Compile-time capability verification |
| **When** | Semua akses kepada resources (fail, rangkaian, hardware) |
| **Runtime Cost** | **ZERO** — all checks at compile time |
| **Files** | `src/tier2/gate.rs`, `src/tier2/topology.rs` |
| **Introduced** | v1.32.0-alpha |

**Three Types:**
- `DirectCall` — inline-able sync (math, crypto)
- `Message` — async SPSC (sensor, network)
- `Hardware` — bare-metal only (GPIO, DMA)

**Supply-Chain Security:**
- `.cap` file per compile → audit trail
- `diff_topology()` → detect privilege escalation

---

## Service — Event Loop + Connection

| Attribute | Value |
|---|---|
| **What** | Port-based actor dengan RAII connection |
| **When** | Semua operasi rangkaian (TCP/UDP) |
| **Cleanup** | RAII Drop — `close(fd)` deterministik |
| **Files** | `src/net/*.rs` |
| **Introduced** | v1.33.0-alpha |

**Manifest:**
```ldx
service MyService {
    port: 8080,
    requires: Net.Send,
    handler: MyHandler,
    policy: Block,
}
```

**Taint State Machine:**
```
Healthy → Suspicious → Closing
   ↑______________|
```

---

## Complete Component Map

| Version | Component | Status | Tests |
|---|---|---|---|
| v1.21 | Baseline Compiler | ✅ Merged | 9/9 |
| v1.21 | Sprint 1.1 TypeRegistry | ✅ Merged | 32/32 |
| v1.21 | Sprint 1.2 Parser Types | ✅ Merged | 20/20 |
| v1.21 | Sprint 2 LayoutEngine | ✅ Merged | 34/34 |
| v1.21 | Sprint 2.5 Struct Literals | ✅ Merged | 25/25 |
| v1.21 | Sprint 3 Codegen Calls | ✅ Merged | 28/28 |
| v1.30 | Demo Raylib Spinning Box | ✅ Merged | 11/11 |
| v1.30 | K1 Core Memory (Slice/Buffer) | ✅ Merged | 17/17 |
| v1.30 | K2 Result<T,E> + Match | ✅ Merged | 9/9 |
| v1.30 | K3+4 File Handle + Syscall | ✅ Merged | 12/12 |
| v1.30 | Audio Engine (Hardware-Safe) | ✅ Merged | 14/14 |
| v1.30 | Phase 1: Threading Foundation | ✅ Merged | 8/8 |
| v1.30 | Phase 2: Zero-Copy Ownership | ✅ Merged | 6/6 |
| v1.30 | Phase 3: Backpressure + Scheduler | ✅ Merged | 10/10 |
| v1.31 | Tier 2 Streaming Engine | ✅ Merged | 6/6 |
| v1.32 | Static Capability Fabric | ✅ Merged | 10/10 |
| v1.33 | Deterministic Network Reactor | ✅ Merged | 13/13 |
| v1.34 | Sharded Multi-Core Reactor | ✅ Merged | 11/11 |
| v1.35 | CapabilityGraph IR | ✅ Merged | 22/22 |
| v1.36 | CTL Mapper (WIT Generation) | ✅ Merged | 16/16 |
| v1.37 | Network Runtime (epoll + syscalls) | ✅ LIVE | 16/16 |
| v1.38 | Deferred Items (8 items) | ✅ Merged | — |
| v1.39 | Sharded Runtime (threads + affinity) | ✅ Merged | 10/10 |
| v1.40 | WASM Codegen Backend | ✅ Merged | 13/13 |
| v1.41 | Host Reactor Integration | ✅ Merged | 20/20 |
| | **TOTAL** | | **400+** |

---

## New Architecture: CapabilityGraph IR + CTL Mapper

### v1.35.0-alpha: CapabilityGraph IR — Single Source of Truth

The CapabilityGraph IR unifies three previously separate structures into one language-agnostic representation:

| Source Structure | IR Component | File |
|---|---|---|
| v1.31 `SemanticSummary` | `IRServiceNode` (effects, inline_cost) | `src/tier2/capability_ir.rs` |
| v1.32 `CapabilityTopology` | `IRGateEdge` + `CapabilityRef` | `src/tier2/capability_ir.rs` |
| v1.34 `ShardTopology` | `IRShardNode` + `IRDoorEdge` | `src/tier2/capability_ir.rs` |

**Output Targets:**
- `CompileTarget::Native` → ELF with inlined capability checks
- `CompileTarget::Wasm` → Sandboxed, maps to WASI via CTL
- `CompileTarget::All` → Dual artifacts from one CapabilityGraph

**Verification:** 6 unified checks (`verify()`) — `EmptyGraph`, `WasmHardwareGate`, `InvalidShardAssignment`, `UnknownServiceInDoor`, `UnknownServiceInGate`, `EmptyShard`

### v1.36.0-alpha: CTL Mapper — "Project INTO, not borrow FROM"

The CTL Mapper auto-generates WIT from CapabilityGraph, projecting Logicodex's capability model INTO the WASM ecosystem:

| Logicodex Domain | WIT Target | Hardware? |
|---|---|---|
| `Storage` | `wasi:filesystem` | No |
| `Net` | `wasi:sockets` | No |
| `UI` | `wasi:cli` | No |
| `HW` | `logicodex:host-reactor` | **Host-mediated only** |
| `Audio` | `wasi:io/custom` | No |
| `Crypto` | `wasi:crypto` | No |

**Key Features:**
- Manual overrides via `add_override(domain.op, custom_wit)`
- HW gates NEVER reach WASM guest — always routed through Host Reactor
- Unknown domains fallback to `logicodex:custom`
- Host reactor stubs auto-generated in Rust

---

## Completed Work (v1.37-v1.41)

### v1.37.0-alpha: Deterministic Network Runtime ✅ COMPLETED
- epoll event loop: `epoll_create1`, `epoll_ctl`, `epoll_wait` via direct syscalls
- Live socket I/O: `SYS_RECV`, `SYS_SEND` via `src/os/syscall.rs` (no libc)
- Monotonic timestamp: `clock_gettime(CLOCK_MONOTONIC)` for taint timeout
- Event processing: `EPOLLIN`/`EPOLLOUT`/`EPOLLERR`/`EPOLLHUP` dispatch
- Taint FSM: `Healthy→Suspicious→Closing` transitions at runtime
- Backpressure: `Block`/`DropOldest`/`Error` policies applied to ring buffer

### v1.38.0-alpha: Deferred Items Cleanup ✅ COMPLETED
- CallableRegistry predeclaration, `from_topology()` fix, enum layout
- Windows syscall fallback, `--secure` memory attestation, `--target freestanding`
- Semantic gatekeeper activation in `compile_v130()`

### v1.39.0-alpha: Sharded Runtime ✅ COMPLETED
- `std::thread::spawn` per shard with `JoinHandle` tracking
- CPU affinity: `sched_setaffinity` (Linux), `thread_policy_set` (macOS), `SetThreadAffinityMask` (Windows)
- `available_parallelism()` for dynamic core detection, `sched_getcpu` for current core

### v1.40.0-alpha: WASM Codegen Backend ✅ COMPLETED
- `CompilationTarget::Wasm` with `wasm32-unknown-unknown` triple
- LLVM features: `+bulk-memory,+mutable-globals,+sign-ext`
- CLI `--target wasm` with `wasm-ld` linking hints

### v1.41.0-alpha: Host Reactor Integration ✅ COMPLETED
- `HostReactor` mediates all HW gate access from WASM guests
- `GatePermissions`: per-operation pin allowlists
- `HardwareZone`: pin claim/release tracking prevents double-use
- `HostFunction` enum: `GpioControl`, `TimerSet`, `DmaTransfer`
- Guest → Host dispatch via `GuestRequest`/`HostResponse` serialization

---

## Future Work

### v1.42.0-alpha: Streaming WASM + WASI Verification (RESEARCH)
- `verify()` extended for WASM-specific constraints: memory limits, no hardware gates
- WASI import completeness validation
- Streaming compilation to WASM (current: full-module only)

### v2.00.0-alpha: 5-Level Pointer Provenance Engine (RESEARCH)
- Level 1: Strict linear provenance (current baseline)
- Level 2: Strict sub-bounded provenance (aggregates, slices)
- Level 3: Hardware view-only provenance (peripheral read patterns)
- Level 4: Hardware mutex-isolated provenance (mutable HW access)
- Level 5: Wild/untrusted provenance (FFI inputs, raw pointers)

### Long-term Objectives
- Full freestanding: bootloader examples, linker scripts, OS-less target profile
- `ldx-fmt` formatter: canonical style without changing meaning
- LSP diagnostics: syntax and semantic feedback in editors
- Global Token Registry: offline-first sync with project lockfile

---

## Validation

**137/137 checks passing + runtime live + sharded + wasm + host + freestanding** — zero regression across all versions.

```
Host Reactor (v1.41):      20/20 ✅  (GatePermissions, HardwareZone, HW dispatch)
WASM Backend (v1.40):      13/13 ✅  (wasm32-unknown-unknown, features, CLI)
Sharded Runtime (v1.39):   10/10 ✅  (thread spawn, CPU affinity, parallel exec)
Network Runtime (v1.37):   16/16 ✅  (epoll, I/O, taint, backpressure — LIVE)
CTL Mapper (v1.36):        12/12 ✅
Capability IR (v1.35):     16/16 ✅
Sharded Reactor (v1.34):   11/11 ✅
Network Reactor (v1.33):   13/13 ✅
Capability Fabric (v1.32):  10/10 ✅
Streaming Engine (v1.31):    6/6  ✅
Threading Phase 3:          10/10 ✅
Threading Phase 2:           6/6  ✅
Threading Phase 1:           8/8  ✅
v1.21 baseline:              9/9  ✅
──────────────────────────────────
TOTAL:                       148/148 ✅ + all platforms
```

### v1.45: Quantitative Benchmark Framework
Architecture-correlated benchmarks across 4 layers:
- **Layer 1 Micro (6):** Gate invoke (<50ns), Door send/recv (<100ns), MemoryPool (<20ns), Callable lookup (<30ns), HIR lower (<200ns), LLVM emit (<500ns)
- **Layer 2 Reactor:** Echo server + flood client, 1/2/4/8 core scaling (target: >85% efficiency)
- **Layer 3 Stability:** RSS monitor with linear regression (0KB creep target), valgrind (0 leaks), 1h/6h/24h longrun
- **Layer 4 Security:** Slowloris, SYN flood, malformed packet, FD exhaustion (all: fail-stop target)
- **BASELINE.json:** Gold standard with 5% warn / 10% fail regression thresholds
- **RFC_TEMPLATE.md:** Architecture Freeze enforcement (4 mandatory alignment checks)

### v1.44.1: Maintenance — Validator Tiering
All 27 validators organized into 3 tiers:
- **Tier A Core (7):** Baseline integrity — build stops on failure
- **Tier B Feature (13):** Feature correctness — warning on failure  
- **Tier C Platform (8):** Platform/stress/performance — CI only
