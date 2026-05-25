# Logicodex Systems Architecture: Door + Gate + Service

> **"The Deterministic Systems Platform"**

Architect: Mohamad Supardi Abdul (mymsastudio@gmail.com)

---

## Philosophy

Logicodex bukan sekadar bahasa pengaturcaraan — ia adalah **"Hardware-Integrated Systems Platform"** yang menggabungkan 4 tiang utama:

1. **Provenance Memory** (K1-K4) — Memori dengan jejak keaslian
2. **Deterministic Concurrency** (Actor & Channel) — Zero-copy actor model
3. **Capability Fabric** (Security Gate) — Compile-time security
4. **Network Reactor** (Deterministic I/O) — Event-driven networking

> **"Mustahil untuk mengalami race condition atau memory leak"** — kerana semuanya diverifikasi pada masa kompil.

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
| | **TOTAL** | | **260+** |

---

## Future Work

### v1.34.0-alpha: Sharded Multi-Core Reactor
- Per-CPU-core reactor instance
- Static CPU affinity untuk servis
- Cross-core communication melalui dedicated Door

### v1.35.0-alpha: WebAssembly Target
- Wasm code generation daripada LLVM IR
- Capability gate untuk browser APIs

### v1.40.0-alpha: Full Freestanding
- Bootloader examples
- Raw pointer gates
- Hardware-region policies
- OS-less target profile

---

## Validation

**60/60 checks passing** — zero regression across all versions.

```
Network Reactor:     13/13 ✅
Capability Fabric:   10/10 ✅
Streaming Engine:     6/6  ✅
Threading Phase 3:   10/10 ✅
Threading Phase 2:    6/6  ✅
Threading Phase 1:    8/8  ✅
v1.21 baseline:       9/9  ✅
─────────────────────────────
TOTAL:               60/60 ✅
```
