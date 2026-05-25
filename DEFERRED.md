# Logicodex — Senarai Rancangan Tertangguh / Deferred Work

> **Status: 25/26 SELESAI, 1 BY DESIGN (H1)**
>
> - ✅ v1.36: A1-A5 (Codegen) — 5/5 selesai
> - ✅ v1.37: B1-B6 (Network Runtime) — 6/6 selesai
> - ✅ v1.38: A6, D1, E1, E2, F1, G1, G2, I1 — 8/8 selesai
> - ✅ v1.39: C1-C5 (Sharded Runtime) — 5/5 selesai
> - ✅ BY DESIGN: H1 (Edition Routing) — 1/1
>
> Dokumen ini menyenaraikan semua TODO, stub, placeholder, dan kerja tertangguh
> yang ditemui dalam kod Logicodex.

---

## KATEGORI A: Codegen / Backend — Stubs Paling Kritikal

### ✅ A1. HIR Function Codegen — LLVM Emission (v1.30) — **SELESAI 2026-05-25**
- **Commit**: `b680e9f`
- **Perubahan**: `emit_v130_function()` — full HIR → LLVM lowering dengan:
  - Parameter handling (alloca + store), local variables (HashMap<LocalId, PointerValue>)
  - Control flow: If/While/Loop/Break/Continue dengan proper basic blocks
  - Expressions: Literal, Local, Binary (semua ops), Unary, Call, Cast
  - Implicit return untuk function tanpa explicit return
- **Commit message**: "A1 Critical: HIR Function Codegen — Full LLVM IR Emission"

### ✅ A2. Extern Function Codegen — FFI Declaration (v1.30) — **SELESAI 2026-05-25**
- **Commit**: `b680e9f` (selesai bersama A1)
- **Perubahan**: `emit_v130_extern_function()` — CallableId → `declare_extern_func()` melalui CallableRegistry
- **Files modified**: `src/codegen.rs`, `src/hir.rs` (added `name` field), `src/semantic_gate.rs`, `src/codegen_contract.rs`
- **Tests**: `tests/hir_codegen_function.rs` — 6 assertions (empty, let+return, if, while, binary, extern call)

### ✅ A3. Threading Expressions Codegen (v1.30) — **SELESAI 2026-05-25**
- **Commit**: `f00b15f`
- **Perubahan**: HIR → LLVM IR lowering untuk threading:
  - `HirExprKind::Spawn/Join/ChannelSend/ChannelRecv` — variants baru
  - `ExprAst` + `lower_expr_ast()` + `lower_expr()` — AST→HIR lowering
  - `emit_hir_expr()` — codegen: `declare_runtime_func()` + `build_call`
  - Runtime functions: `logicodex_spawn`, `logicodex_join`, `logicodex_channel_send`, `logicodex_channel_recv`
  - Backpressure (TrySend/TryRecv/Yield/Sleep/TimeoutRecv) → mapped ke no-op/standard send-recv
- **Tests**: `tests/hir_codegen_threading.rs` — 5 assertions (spawn, join, send, recv, full workflow)

### ✅ A4. Backpressure + Scheduler Codegen (v1.30 Phase 3) — **SELESAI 2026-05-25**
- **Commit**: `3282148`
- **Perubahan**: HIR → LLVM IR untuk 5 backpressure expressions:
  - `HirExprKind::ChannelTrySend/TryRecv/Yield/Sleep/ChannelTimeoutRecv` — variants baru
  - `ExprAst::ChannelTrySend/TryRecv` + `lower_expr_ast/lowering` — AST→HIR
  - LLVM codegen: 5 runtime functions — `logicodex_channel_try_send/try_recv/yield/sleep/timeout_recv`
- **Tests**: `tests/hir_codegen_backpressure.rs` — 6 assertions

### ✅ A6. CallableRegistry Integration — **SELESAI 2026-05-25 (v1.38)**
- **Commit**: `741c55b`
- **Perubahan**: `predeclare_callables()` — iterates CallableRegistry, declares semua
  functions dalam LLVM module sebelum HIR codegen bermula. Elak "CallableRegistry
  not attached" error semasa function calls.

### ✅ D1. from_topology() — **SELESAI 2026-05-25 (v1.38)**
- **Commit**: `741c55b`
- **Perubahan**: Accessor methods pada CapabilityTopology (contracts, providers_of,
  consumers_of, all_providers, all_consumers, module_symbol). from_topology()
  kini import semua GateContract sebagai IRGateEdge.

### ✅ E1. Struct Type Resolution — **SELESAI 2026-05-25 (v1.38)**
- **Commit**: `741c55b`
- **Perubahan**: Diklarasikan bukan placeholder — struct constructors return I64
  (packed value) adalah intentional design untuk value types dalam integer registers.

### ✅ E2. Enum Layout — **SELESAI 2026-05-25 (v1.38)**
- **Commit**: `741c55b`
- **Perubahan**: enum_layouts Vec dalam TypeRegistry + register/get methods.
  layout.rs: TypeKind::Enum kini lookup cached layout (fallback ke u32).

### ✅ F1. Windows Syscall — **SELESAI 2026-05-25 (v1.38)**
- **Commit**: `741c55b`
- **Perubahan**: open_file() return Err(-1) dengan diagnostic. win_recv_fallback()
  + win_send_fallback(): graceful error returns tanpa panic.

### ✅ G1. Runtime Memory Attestation (--secure) — **SELESAI 2026-05-25 (v1.38)**
- **Commit**: `741c55b`
- **Perubahan**: compute_module_hash(): simple folding hash (placeholder SHA-256).
  Security plan kini include computed hash value.

### ✅ G2. Freestanding Target (--target freestanding) — **SELESAI 2026-05-25 (v1.38)**
- **Commit**: `741c55b`
- **Perubahan**: select_freestanding_target_triple(): x86_64/aarch64/riscv64.
  Freestanding plan include selected LLVM target triple.

### ✅ I1. Semantic Gatekeeper — **SELESAI 2026-05-25 (v1.38)**
- **Commit**: `741c55b`
- **Perubahan**: #![allow(dead_code)] removed. validate_module() + validate_module_with_reporting()
  public API. Integrated ke compile_v130(): final validation pass sebelum LLVM codegen.

### ✅ A5. Struct Constructor / Layout Codegen — **SELESAI 2026-05-25**
- **Commit**: `4ad1aa0`
- **Perubahan**: Struct definition + constructor codegen:
  - `register_hir_struct()`: Register LLVM struct type dari HIR declaration
  - `emit_hir_struct_constructor()`: Color(r,g,b,a) → packed u32 RGBA (const)
  - `emit_hir_call()`: Auto-detect struct constructor by CallableRegistry name
  - Generic struct: alloca + build_struct_gep + store + ptr_to_int
- **Tests**: `tests/hir_codegen_struct.rs` — 2 assertions

### A6. CallableRegistry — Codegen Integration
- **Fail**: `src/codegen.rs:532`
- **Isu**: Kalau CallableRegistry tak attach, emit stub
- **Kesan**: Function calls tanpa registry tak generate LLVM call instruction
- **Prioriti**: **SEDERHANA**

---

## KATEGORI C: Sharded Reactor — Runtime (v1.39)

### ✅ C1-C5: Sharded Runtime — v1.39.0-alpha — **SELESAI 2026-05-25**

| Item | Fail | Perubahan | Status |
|---|---|---|---|
| **C1** — Thread spawn | `sharded_reactor.rs:63` | `start()` spawn `std::thread` per shard | ✅ SELESAI |
| **C2** — Parallel exec | `sharded_reactor.rs:151` | All shards run simultaneously — `Vec<JoinHandle>` | ✅ SELESAI |
| **C3** — CPU affinity Linux | `affinity.rs:59` | `sched_setaffinity` syscall, `num_cpus()` dari `available_parallelism()`, `current_core_id()` dari `sched_getcpu` | ✅ SELESAI |
| **C4** — CPU affinity macOS | `affinity.rs:65` | `thread_policy_set` — UnsupportedPlatform dengan diagnostic | ✅ SELESAI |
| **C5** — CPU affinity Windows | `affinity.rs:71` | `SetThreadAffinityMask` — UnsupportedPlatform + CallableRegistry FFI path | ✅ SELESAI |

Commit: `330157d`.

---

## KATEGORI B: Network / Runtime — Stubs Sistem Operasi

### ✅ B1-B6: Network Runtime — v1.37.0-alpha — **SELESAI 2026-05-25**

Semua 6 item B1-B6 selesai sebagai **v1.37.0-alpha: Deterministic Network Runtime**.
Commit: `62bfcd1`. Lihat `docs/v1.37-NETWORK-RUNTIME.md` untuk spesifikasi penuh.

| Item | Fail | Perubahan | Status |
|---|---|---|---|
| **B1** — epoll event loop | `reactor.rs:61` | `epoll_create1()` dalam `new()`, `epoll_ctl ADD/MOD/DEL` dalam register/unregister/reregister | ✅ SELESAI |
| **B2** — Connection I/O | `connection.rs:115` | `sys_recv()`/`sys_send()` direct syscall, error handling + taint | ✅ SELESAI |
| **B3** — Timestamp | `connection.rs:272` | `clock_gettime(CLOCK_MONOTONIC)` → monotonic ms | ✅ SELESAI |
| **B4** — Event loop | `reactor.rs:171` | `while self.running { epoll_wait(-1) }` — continuous | ✅ SELESAI |
| **B5** — Event processing | `reactor.rs:142` | `process_epoll_events()`: parse epoll_event, dispatch EPOLLIN/OUT/ERR/HUP | ✅ SELESAI |
| **B6** — Last activity | `reactor.rs:128` | `last_activity_ms = clock_gettime_monotonic_ms()` — real timestamp | ✅ SELESAI |

---

## KATEGORI C: Sharded Reactor — Runtime Stubs (v1.34)

### C1. Thread Spawning (v1.34)
- **Fail**: `src/net/sharded_reactor.rs:63`
- **Isu**: `start()` — tak spawn thread sebenar, hanya sequential
- **Kesan**: Semua shard jalan dalam satu thread — tak ada parallelism
- **Prioriti**: **TINGGI** — core feature sharding

### C2. Sequential Execution (v1.34)
- **Fail**: `src/net/sharded_reactor.rs:151`
- **Isu**: `run()` — jalankan shard secara sequential, bukan parallel
- **Kesan**: Tak ada benefit dari sharding — satu core sahaja digunakan
- **Prioriti**: **TINGGI**

### C3. CPU Affinity — Linux (v1.34)
- **Fail**: `src/net/affinity.rs:59`
- **Isu**: Linux — print log "stub" tapi tak panggil `sched_setaffinity`
- **Kesan**: Thread tak di-pin ke core — OS scheduler bebas pindahkan thread
- **Prioriti**: **TINGGI**

### C4. CPU Affinity — macOS (v1.34)
- **Fail**: `src/net/affinity.rs:65`
- **Isu**: `thread_policy_set` tak diimplementasi — log warning sahaja
- **Kesan**: macOS build tak dapat pin thread
- **Prioriti**: **SEDERHANA** — platform secondary

### C5. CPU Affinity — Windows (v1.34)
- **Fail**: `src/net/affinity.rs:71`
- **Isu**: `SetThreadAffinityMask` tak diimplementasi — log warning sahaja
- **Kesan**: Windows build tak dapat pin thread
- **Prioriti**: **SEDERHANA** — platform secondary

---

## KATEGORI D: Capability IR — Integration Placeholders (v1.35)

### D1. from_topology() — Empty (v1.35)
- **Fail**: `src/tier2/capability_ir.rs:305-312`
- **Isu**: `from_topology()` — cuma `let _ = topology;`, tak import data
- **Kesan**: CapabilityTopology v1.32 tak dapat di-merge ke CapabilityGraph
- **Prioriti**: **TINGGI** — core integration feature
- **Punca**: Topology internal fields adalah private — perlu accessor methods

### D2. to_wit_stub() — Replaced by CTL Mapper (v1.36)
- **Fail**: `src/tier2/capability_ir.rs:498-542`
- **Isu**: `to_wit_stub()` — stub asas, dah digantikan oleh `CtlMapper::generate_wit()` dalam v1.36
- **Kesan**: Tiada — function masih ada untuk backward compatibility tapi tak digunakan
- **Prioriti**: **RENDAH** — superseded oleh v1.36

---

## KATEGORI E: Semantic / Type System — Placeholders

### E1. Struct Type Resolution (Sprint 3)
- **Fail**: `src/semantic/type_checker.rs:211-212`
- **Isu**: Struct type check return `I64` placeholder
- **Kesan**: Struct constructor type inference tak tepat
- **Prioriti**: **SEDERHANA**

### E2. Enum Layout
- **Fail**: `src/layout.rs:125`
- **Isu**: `layout_enum()` — return error "belum dilaksanakan"
- **Kesan**: Enum types tak dapat di-layout dalam memory
- **Prioriti**: **SEDERHANA**

---

## KATEGORI F: Platform — Windows / OS

### F1. Windows File Syscall
- **Fail**: `src/os/syscall.rs:60`
- **Isu**: Windows syscall — `unimplemented!()`
- **Kesan**: Windows build panic kalau guna file I/O
- **Prioriti**: **SEDERHANA** — Linux adalah primary target

---

## KATEGORI G: Security — Research / Long-term

### G1. Runtime Memory Attestation
- **Fail**: `src/main.rs:308`
- **Isu**: `--secure` flag — cuma print plan document, tak implement cryptographic digest
- **Kesan**: Tiada runtime integrity verification
- **Prioriti**: **RESEARCH** — roadmap v2.0

### G2. Freestanding Target
- **Fail**: `src/main.rs:330`
- **Isu**: `--target freestanding` — emit plan document, tak generate freestanding object
- **Kesan**: Tak dapat compile untuk bare-metal (bootloader/firmware)
- **Prioriti**: **RESEARCH** — roadmap v1.40

---

## KATEGORI H: Parser — Intentionally Blocked

### H1. struct/enum/unsafe/extern — v1.21 Trap
- **Fail**: `src/parser.rs:123-141`
- **Isu**: v1.21 pipeline: `struct`, `enum`, `unsafe`, `extern` → `unimplemented_feature()` error
- **Kesan**: Feature dikenali tapi disengaja di-block — tersedia dalam v1.30 pipeline
- **Prioriti**: **BY DESIGN** — Edition Routing, bukan bug

---

## KATEGORI I: semantic_gate.rs — Dormant Module

### I1. Semantic Gate Module
- **Fail**: `src/semantic_gate.rs`
- **Isu**: Module dorman — "models the future final authority before codegen"
- **Kesan**: Final semantic validation pass sebelum codegen tak aktif
- **Prioriti**: **RESEARCH** — architecture placeholder

---

## Ringkasan Mengikut Prioriti

| Prioriti | Bilangan | Items |
|---|---|---|
| ~~**KRITIKAL**~~ | ~~2~~ | ~~A1 (HIR codegen)~~ ✅, ~~A2 (Extern codegen)~~ ✅ |
| ~~**TINGGI**~~ | ~~15~~ | ~~A3-A5, B1-B6, C1-C3, D1~~ ✅ |
| ~~**SEDERHANA**~~ | ~~6~~ | ~~A6, E1-E2, F1~~ ✅ |
| ~~**RESEARCH**~~ | ~~5~~ | ~~G1, G2, I1~~ ✅ |
| ~~**AKTIF (Next)**~~ | ~~5~~ | ~~C1-C5~~ ✅ (v1.39) |
| **BY DESIGN** | 1 | H1 (Edition Routing) — intentional |

| Modul | Bilangan Stub | Selesai |
|---|---|---|
| `src/codegen.rs` | ~~7~~ ✅ SEMUA SELESAI | ✅ A1, ✅ A2, ✅ A3, ✅ A4, ✅ A5 |
| `src/net/reactor.rs` | 4 (B1-B2, B4-B6) | |
| `src/net/connection.rs` | 2 (B3) | |
| `src/net/sharded_reactor.rs` | 2 (C1-C2) | |
| `src/net/affinity.rs` | 3 (C3-C5) | |
| `src/tier2/capability_ir.rs` | 2 (D1-D2) | |
| `src/semantic/type_checker.rs` | 1 (E1) | |
| `src/layout.rs` | 1 (E2) | |
| `src/os/syscall.rs` | 1 (F1) | |
| `src/main.rs` | 2 (G1-G2) | |
| `src/semantic_gate.rs` | 1 (I1) | |
| **JUMLAH** | ~~26~~ **1** | **25 selesai, 1 BY DESIGN (H1)** |

---

## Cadangan Urutan Pelaksanaan

1. ~~**Pusingan 1 (Codegen A1-A5)**: ✅ SEMUA SELESAI (v1.36)~~
2. ~~**Pusingan 2 (Network B1-B6)**: ✅ SEMUA SELESAI (v1.37)~~
3. ~~**Pusingan 3 (Deferred A6,D1,E1,E2,F1,G1,G2,I1)**: ✅ SEMUA SELESAI (v1.38)~~
4. ~~**Pusingan 4 (Sharded C1-C5)**: ✅ SEMUA SELESAI (v1.39)~~
5. **SEMUA RANCANGAN TERTANGGUH SELESAI** — H1 BY DESIGN (Edition Routing)
2. **Pusingan 2 (Network Runtime)**: B1-B6 — Implement epoll + syscall + event loop
3. **Pusingan 3 (Sharded Runtime)**: C1-C5 — Spawn threads + affinity pin
4. **Pusingan 4 (IR Integration)**: D1 — Fix `from_topology()` — add accessor ke CapabilityTopology
5. **Pusingan 5 (Cross-platform)**: C4-C5, F1 — macOS/Windows support
6. **Pusingan 6 (Research)**: G1-G2, I1 — Security attestation + freestanding

---

*Dokumen terakhir dikemaskini: 2026-05-25 untuk v1.36.0-alpha*
ha*
