# Logicodex Practical Roadmap — v1.45.0-alpha Quantitative Benchmark Framework

This roadmap describes how **Logicodex** progresses from an alpha compiler baseline toward a deterministic systems platform with WASM integration, combining actor-model concurrency, streaming compilation, capability-based security, sharded event-driven networking, and capability-native WIT generation.

> **Roadmap principle:** Logicodex should earn stronger claims through reproducible builds, executable examples, validation scripts, measured performance, and clearly documented safety boundaries.

| Horizon | Primary focus | Status |
|---|---|---|
| v1.21 | Compiler-core baseline | ✅ **COMPLETED** — 9/9 checks passing |
| v1.30 | Threading + IO + Audio | ✅ **COMPLETED** — 104/104 checks passing |
| v1.31 | Streaming compiler | ✅ **COMPLETED** — 6/6 checks passing |
| v1.32 | Capability security | ✅ **COMPLETED** — 10/10 checks passing |
| v1.33 | Network reactor | ✅ **COMPLETED** — 13/13 checks passing |
| v1.34 | Sharded multi-core reactor | ✅ **COMPLETED** — 11/11 checks passing |
| v1.35 | CapabilityGraph IR (Fasa A) | ✅ **COMPLETED** — 16/16 checks passing |
| v1.36 | CTL Mapper — WIT Generation (Fasa B) | ✅ **COMPLETED** — 12/12 checks passing |
| v1.37 | **Network Runtime — epoll, socket I/O, taint FSM** | ✅ **COMPLETED** |
| v1.38 | **Deferred Items Cleanup — A6, D1, E1-E2, F1, G1, G2, I1** | ✅ **COMPLETED** |
| v1.39 | **Sharded Runtime — C1-C5: thread spawn, CPU affinity** | ✅ **COMPLETED** |
| v1.40 | **WASM Codegen Backend — wasm32-unknown-unknown** | ✅ **COMPLETED** |
| v1.41 | **Host Reactor Integration — Guest ↔ Host HW mediation** | ✅ **COMPLETED** |
| v1.42 | **Raylib FFI — 8 pending items resolved** | ✅ **COMPLETED** |
| v1.43 | **Raylib Audio — 22 functions + StrictAudioContext** | ✅ **COMPLETED** |
| v1.44 | **Freestanding Compiler — 15 gaps, 3 architectures** | ✅ **COMPLETED** |
| v1.44.1 | **Foundation Polish — Validator tiering, maintenance report** | ✅ **COMPLETED** |
| v1.45 | **Quantitative Benchmark Framework — 4 layers, BASELINE.json** | ✅ **COMPLETED** |
| v1.46 | Streaming WASM + WASI Capability Verification at Runtime | 🔬 **RESEARCH** |
| v2.00 | Pointer provenance engine (5-level) | 🔬 **RESEARCH** |

## Milestone 1: Stabilize the Alpha Compiler Core

The first priority is to keep the current compiler pipeline reproducible. This means preserving Rust Edition 2021 compatibility, retaining the Rust 1.75 validation floor, maintaining the pinned LLVM 15 dependency path, and expanding executable examples that demonstrate lexing, parsing, semantic checks, LLVM IR generation, and native-oriented object output.

| Issue | Status | Owner | Practical acceptance signal |
|---|---|---|---|
| Sprint 1.2 — Parser Type Injection (TypeChecker, CoercionEngine integration, bilingual errors) | **[X] COMPLETED / MERGED #15** | Mohamad Supardi Abdul | TypeChecker with CoercionEngine, default type inference (I64/F64/String/Bool), bilingual diagnostics. See CHANGELOG.md for details. |
| Sprint 1.1 — Type System Foundation (TypeRegistry, CoercionEngine, Raylib FFI) | **[X] COMPLETED / MERGED #14** | Mohamad Supardi Abdul | TypeRegistry with C ABI sizes, CoercionEngine with widening/narrowing rules, Raylib FFI bindings (28 functions), 38 test assertions. See CHANGELOG.md for details. |
| Issue #01 — Grammar baseline | Completed for v1.21-alpha baseline | Mohamad Supardi Abdul | The grammar document matches lexer/parser behavior for the currently implemented language subset. |
| Issue #02 — UB and provenance design note | Baseline hardware-zone gate implemented and logically verified | Mohamad Supardi Abdul | The specification now records `ZON_PERKAKASAN` / `hw_unsafe` lexical gating, and the semantic analyzer rejects raw address pointer bindings outside that safe zone while leaving deeper hardware I/O work for later milestones. |
| Issue #03 — Native example suite | Partially complete | Mohamad Supardi Abdul | The refreshed reflex-engine `.ldx` suite passes `check` and `v130-check`; remaining work is expected-output fixtures and backend/object-output parity checks. |
| Issue #04 — CI-oriented validation | ✅ **COMPLETED** v1.44.1 | Mohamad Supardi Abdul | 3-tier validator pipeline (Tier A: 7 core, Tier B: 13 feature, Tier C: 8 stress) with `cargo test`. |

## Milestone 1b: Threading + IO + Audio (v1.30.1-alpha)

Deterministic concurrency, 4-Ketuk IO architecture, and hardware-safe audio engine. All integrated with the v1.21 baseline with zero regression.

| Issue | Status | Owner | Practical acceptance signal |
|---|---|---|---|
| Phase 1 — Threading Foundation (Actor & Channel, topology validation) | **[X] COMPLETED / MERGED #27** | Mohamad Supardi Abdul | 8/8 validator checks, `lib/core/thread.ldx`, `lib/core/sync.ldx`. See `docs/v1.30-THREADING.md`. |
| Phase 2 — Zero-Copy Ownership Transfer (RAII move via channel, UseAfterSend) | **[X] COMPLETED / MERGED #28** | Mohamad Supardi Abdul | 6/6 validator checks, `lib/core/ring_buffer.ldx`. See `docs/v1.30-THREADING.md`. |
| Phase 3 — Backpressure + Scheduler (try_send/try_recv/yield/sleep/timeout_recv) | **[X] COMPLETED / MERGED #30** | Mohamad Supardi Abdul | 10/10 validator checks, `lib/core/scheduler.ldx`. See `docs/v1.30-THREADING.md`. |
| IO Architecture K1 — Core Memory Model (Slice<T>, Buffer<T>, provenance) | **[X] COMPLETED / MERGED #23** | Mohamad Supardi Abdul | 17/17 validator checks, `lib/core/memori.ldx`. See `docs/architecture/IO_ARCHITECTURE_4KETUK.md`. |
| IO Architecture K2 — Result<T,E> Abstraction (Ok/Err/Match) | **[X] COMPLETED / MERGED #25** | Mohamad Supardi Abdul | 9/9 validator checks, `lib/core/result.ldx`. |
| IO Architecture K3+4 — File Handle ABI + Syscall Backend | **[X] COMPLETED / MERGED #26** | Mohamad Supardi Abdul | 12/12 validator checks, `src/os/syscall.rs`, `lib/core/file.ldx`. |
| Audio Engine — Hardware-Safe Audio Guards (StrictAudioContext, 4 violation types) | **[X] COMPLETED / MERGED #22** | Mohamad Supardi Abdul | 14/14 validator checks, `lib/std/audio.ldx`. |
| Buffer Provenance Bug Fixes (5 critical fixes) | **[X] COMPLETED / MERGED #24** | Mohamad Supardi Abdul | 9/9 validator checks. |

## Milestone 1c: The Deterministic Systems Platform (v1.31-v1.34)

Transform Logicodex from a compiler into a hardware-integrated systems platform.

| Issue | Status | Owner | Practical acceptance signal |
|---|---|---|---|
| v1.31 — Tier 2 Streaming Engine (2-Pass Engine, SemanticSummary ~64B, MetadataGraph) | **[X] COMPLETED / MERGED #31** | Mohamad Supardi Abdul | 6/6 validator checks, `src/tier2/`. RAM stays flat regardless of program size. See `docs/v1.31-STREAMING.md`. |
| v1.32 — Static Capability Fabric (Gate/Door split, 3 gate types, topology verify, .cap file, privilege escalation detection) | **[X] COMPLETED / MERGED #32** | Mohamad Supardi Abdul | 10/10 validator checks, `src/tier2/gate.rs`, `src/tier2/topology.rs`. Zero runtime mediation. See `docs/v1.32-CAPABILITY.md`. |
| v1.33 — Deterministic Network Reactor (RAII Connection, Taint FSM, Service manifest, backpressure policies) | **[X] COMPLETED / MERGED #33** | Mohamad Supardi Abdul | 13/13 validator checks, `src/net/`. No socket leaks. See `docs/v1.33-REACTOR.md`. |
| v1.34 — Sharded Deterministic Reactor (ShardTopology, per-core instances, CPU affinity, cross-shard doors, memory budgeting) | **[X] COMPLETED / MERGED #35** | Mohamad Supardi Abdul | 11/11 validator checks, `src/tier2/shard.rs`, `src/net/sharded_reactor.rs`. See `docs/v1.34-SHARDED.md`. |

## Milestone 1d: The Capability Translation Layer (v1.35-v1.36)

Project Logicodex's capability-native world INTO the WASM ecosystem. "Project INTO, not borrow FROM."

| Issue | Status | Owner | Practical acceptance signal |
|---|---|---|---|
| v1.35 — CapabilityGraph IR (CompileTarget, CapabilityRef, IRServiceNode, IRShardNode, verify, to_cap, to_wit_stub) | **[X] COMPLETED / MERGED #37** | Mohamad Supardi Abdul | 16/16 validator checks, `src/tier2/capability_ir.rs`. Single Source of Truth unifying v1.31+v1.32+v1.34. See `docs/v1.35-CAPABILITY-IR.md`. |
| v1.36 — CTL Mapper (WitDomain, WitOperation, CtlMapper, 6 domain mappings, manual overrides, host reactor stubs) | **[X] COMPLETED / MERGED #38** | Mohamad Supardi Abdul | 12/12 validator checks, `src/tier2/ctl_mapper.rs`. Auto-generates WIT from CapabilityGraph. See `docs/v1.36-CTL-MAPPER.md`. |

## Milestone 1f: Deferred Items Cleanup (v1.38)

Close 8 long-standing deferred items from DEFERRED.md.

| Issue | Status | Owner | Practical acceptance signal |
|---|---|---|---|
| v1.38 A6 — CallableRegistry predeclaration (predeclare_callables before HIR codegen) | **[X] COMPLETED** | Mohamad Supardi Abdul | No more "CallableRegistry not attached" errors during codegen. |
| v1.38 D1 — from_topology() fix (CapabilityTopology accessors + IRGateEdge import) | **[X] COMPLETED** | Mohamad Supardi Abdul | CapabilityTopology data correctly merges into CapabilityGraph. |
| v1.38 E1 — Struct type resolution (clarified I64 packed value design) | **[X] COMPLETED** | Mohamad Supardi Abdul | Struct constructors return correct packed values. |
| v1.38 E2 — Enum layout (enum_layouts in TypeRegistry, layout.rs lookup) | **[X] COMPLETED** | Mohamad Supardi Abdul | TypeKind::Enum resolves to cached layout (u32 fallback). |
| v1.38 F1 — Windows syscall fallback (graceful errors instead of panic) | **[X] COMPLETED** | Mohamad Supardi Abdul | Windows builds don't panic on I/O — return diagnostic errors. |
| v1.38 G1 — Memory attestation (compute_module_hash placeholder SHA-256) | **[X] COMPLETED** | Mohamad Supardi Abdul | `--secure` flag produces hash in security plan document. |
| v1.38 G2 — Freestanding target (select_freestanding_target_triple) | **[X] COMPLETED** | Mohamad Supardi Abdul | `--target freestanding` selects correct LLVM triple (x86_64/aarch64/riscv64). |
| v1.38 I1 — Semantic gatekeeper activation (validate_module in compile_v130) | **[X] COMPLETED** | Mohamad Supardi Abdul | Final validation pass runs before LLVM codegen. |

## Milestone 1e: The Deterministic Network Runtime (v1.37)

Close the compile-time → runtime gap. The v1.33 reactor had compile-time verification (syntax, topology, taint analysis) but all runtime operations were stubs. v1.37 makes the reactor live.

| Issue | Status | Owner | Practical acceptance signal |
|---|---|---|---|
| v1.37 B1 — epoll event loop (`epoll_create1`, `epoll_ctl` ADD/MOD/DEL, `epoll_wait`) | **[X] IMPLEMENTED** | Mohamad Supardi Abdul | `epoll_fd` is a real file descriptor, not -1. Event loop runs until `stop()` called. |
| v1.37 B2 — Connection read/write (`recv`, `send` syscalls) | **[X] IMPLEMENTED** | Mohamad Supardi Abdul | `Connection::read()` calls `SYS_RECV`, `Connection::write()` calls `SYS_SEND`. |
| v1.37 B3 — Timestamp (`clock_gettime` CLOCK_MONOTONIC) | **[X] IMPLEMENTED** | Mohamad Supardi Abdul | `current_timestamp_ms()` returns actual wall-clock ms, not 0. |
| v1.37 B4 — Event processing (`process_events` processes epoll events) | **[X] IMPLEMENTED** | Mohamad Supardi Abdul | `Reactor::run()` loops, calling `process_events()` which dispatches to handler. |
| v1.37 B5 — Connection taint FSM (Healthy→Suspicious→Closing transitions) | **[X] IMPLEMENTED** | Mohamad Supardi Abdul | `check_taint()` transitions state based on error count + timeout. `is_trustworthy()` gates I/O. |
| v1.37 B6 — Backpressure at runtime (`Block`/`DropOldest`/`Error` policies) | **[X] IMPLEMENTED** | Mohamad Supardi Abdul | `enqueue()` applies policy: Block (spin-wait), DropOldest (overwrite), Error (return false). |

## Milestone 2: Tighten Language Semantics and Diagnostics

The second priority is to make the language easier to reason about. The project should document nominal typing, inference boundaries, casts, unsafe capability gates, and target-specific restrictions before adding broad new language features.

| Issue | Status | Owner | Practical acceptance signal |
|---|---|---|---|
| Issue #05 — Nominal type system boundaries | Open | TBD | Type rules distinguish inference, explicit annotations, casts, and compiler-enforced invariants. |
| Issue #06 — Pointer and hardware-region gates | Open | TBD | Hosted and freestanding memory operations are separated by explicit syntax, diagnostics, and examples. |
| Issue #07 — Diagnostic quality pass | Open | TBD | Common user errors produce actionable messages with source locations and suggested fixes. |
| Issue #07b — Version gate (Edition Routing) architecture | **[X] COMPLETED / MERGED #12** | Mohamad Supardi Abdul | `CompilerPipeline` enum gates parser behavior; v1.21 remains default with zero regression; v1.30 is opt-in via `--pipeline v1.30`; fail-fast `unreachable!()` safety nets prevent silent pipeline leaks. See CHANGELOG.md for full details. |

## Milestone 3: Build Developer Tooling

Developer tooling should follow the stabilized language subset rather than racing ahead of it. Formatting and editor feedback will make the alias-to-canonical model easier to maintain in collaborative use.

| Issue | Status | Owner | Practical acceptance signal |
|---|---|---|---|
| Issue #08 — `ldx-fmt` formatter | Open | TBD | Representative Logicodex examples are formatted into a canonical style without changing meaning. |
| Issue #09 — LSP diagnostics | Open | TBD | Syntax and semantic feedback work in at least one supported editor. |
| Issue #10 — Documentation examples | Partially complete | Mohamad Supardi Abdul | README, manual, grammar notes, repository context, and reflex-example documentation describe the current validated example suite; remaining work is to keep release notes and future specs synchronized as behavior changes. |

## Milestone 4: Prototype Portable Targets

WebAssembly and cross-platform targets remain valuable objectives, but they should be introduced through small prototypes with clear limitations. The project should avoid describing a target as supported until a representative program can be built and executed in that environment.

| Issue | Status | Owner | Practical acceptance signal |
|---|---|---|---|
| Issue #11 — WebAssembly prototype | ✅ **COMPLETED** v1.40 | Mohamad Supardi Abdul | LLVM `wasm32-unknown-unknown` with `+bulk-memory,+mutable-globals,+sign-ext`. CLI `--target wasm`. |
| Issue #12 — Cross-platform benchmark harness | ✅ **COMPLETED** v1.45 | Mohamad Supardi Abdul | 4-layer framework (micro/reactor/stability/security), BASELINE.json, regression detection. |
| Issue #13 — Release artifact refresh workflow | Open | TBD | Archives are regenerated only after build and validator evidence is captured. |

## Milestone 5: Treat Security and Freestanding Work as Research Objectives

Runtime memory attestation, Golden Hash planning, hard fail-stop behavior, and freestanding hardware access are ambitious directions. They should be framed as **research and engineering objectives** until implemented, reviewed, and measured. The near-term task is to define threat models, safe opt-in flags, target-specific behavior, and tests that prove the compiler emits the expected runtime hooks.

| Objective | Current status | Long-term success signal |
|---|---|---|
| Runtime memory attestation | Design contract and plan generation | Digest insertion, verifier stubs, threat model, overhead measurements, and tamper tests exist. |
| Stop-on-failure mitigation | Roadmap model | Hosted process abort and freestanding halt/reset behavior are implemented only where safe, documented, and explicitly selected. |
| Freestanding support | Experimental target profile | Linker scripts, bootloader integration notes, hardware-region policies, and minimal examples are validated. |
| Migration assistant | Conceptual roadmap | Translation output is reviewable, testable, and clearly marked as assisted migration rather than automatic proof of correctness. |

## Milestone 5b: Platform Targets (v1.37-v1.41) — ALL COMPLETED

WASM codegen backend, host reactor integration, and freestanding target support — all completed across v1.37 through v1.41.

| Objective | Status | Practical acceptance signal |
|---|---|---|
| v1.37 — Network Runtime | ✅ **COMPLETED** | epoll event loop via direct syscall, `SYS_RECV`/`SYS_SEND` socket I/O, taint FSM at runtime, monotonic timestamps. |
| v1.38 — Deferred Items Cleanup | ✅ **COMPLETED** | 8 deferred items resolved: callable predeclaration, topology fix, enum layout, Windows fallback, memory attestation, freestanding target, semantic gatekeeper. |
| v1.39 — Sharded Runtime | ✅ **COMPLETED** | `std::thread::spawn` per shard, CPU affinity via `sched_setaffinity`, `available_parallelism()`, `sched_getcpu`. |
| v1.40 — WASM Codegen Backend | ✅ **COMPLETED** | LLVM `wasm32-unknown-unknown` with `+bulk-memory,+mutable-globals,+sign-ext`. CLI `--target wasm`. |
| v1.41 — Host Reactor Integration | ✅ **COMPLETED** | `logicodex:host-reactor` interface for HW gate mediation. `GatePermissions` + `HardwareZone` for guest ↔ host access control. |
| v1.42 — Raylib FFI Resolution | ✅ **COMPLETED** | 8 pending items resolved: build.rs, struct-by-value, constructors, math shims, audio guards, WASM safety, coercion. |
| v1.42b — WASI Capability Verification | 🔬 **RESEARCH** | `verify()` extended for WASM-specific constraints: memory limits, no hardware gates, WASI import completeness. |
| v2.00 — Pointer Provenance Engine | 🔬 **RESEARCH** | 5-level provenance: linear → sub-bounded → hardware view-only → hardware mutex-isolated → wild/untrusted. |

## Milestone 6: Prepare the Logicodex v2.0 Pointer Provenance Research Track

The **5-Level Pointer Provenance Engine** is a proposed Logicodex v2.0 contributor track. It should be treated as a staged research and engineering objective, not as a completed v1.21-alpha guarantee. The current logicodex v 1.21 alpha baseline already documents ownership, provenance, and unsafe-boundary intent, but each additional level needs parser support, semantic-analysis rules, diagnostics, examples, and validation evidence before it is described as an implemented security feature.

The practical goal is to turn pointer provenance into a contributor-friendly roadmap that can be implemented incrementally in `src/lexer.rs`, `src/parser.rs`, `src/semantic.rs`, and `src/codegen.rs`. Strong security language should be reserved for behavior that has executable tests and target-specific evidence. Until then, the roadmap should describe these items as intended mitigations, design constraints, or long-term compiler-analysis objectives.

| Proposed provenance level | Roadmap status | Practical acceptance signal |
|---|---|---|
| Level 1 — Strict linear provenance | Current design baseline | Lexical ownership and drop semantics are documented, parser/semantic behavior is covered by examples, and regressions are checked by validation fixtures. |
| Level 2 — Strict sub-bounded provenance | Long-term v2.0 objective | Aggregate fields, slices, and array sub-ranges have explicit bounds metadata; diagnostics reject proven out-of-range sub-object access without claiming broad vulnerability elimination. |
| Level 3 — Hardware view-only provenance | Long-term v2.0 objective | `hw addr`-style physical-address vocabulary is gated by target profile and read-only attributes, with examples showing safe peripheral-read patterns. |
| Level 4 — Hardware mutex-isolated provenance | Long-term v2.0 objective | Mutable hardware access requires explicit synchronization policy, atomic capability, or unique access proof before code generation accepts the operation. |
| Level 5 — Wild or untrusted provenance | Current design direction | FFI inputs, raw pointers, and integer-to-pointer casts remain isolated behind explicit syntax, diagnostics, and unsafe-boundary documentation. |

Contributor work should start with specification and diagnostics before LLVM optimization metadata. Attribute syntax such as `#[bounded]`, `#[read_only]`, or future Malay aliases should first be represented in the lexer and parser, then enforced in `src/semantic.rs`, and only later mapped to LLVM metadata such as alias-analysis or type-based alias-analysis hints in `src/codegen.rs`. Metadata should be treated as an optimization aid, not the sole enforcement layer, because the compiler's semantic checks must carry the primary safety contract.

| Contribution area | First practical task | Evidence required before stronger claims |
|---|---|---|
| Lexer and parser attributes | Parse bounded/read-only annotations without changing existing program behavior. | Syntax fixtures and parser snapshots show accepted and rejected forms. |
| Semantic enforcement | Track sub-object bounds and unsafe provenance categories in a conservative analysis pass. | Negative tests prove invalid flows are rejected with clear diagnostics. |
| Hardware access gates | Require explicit target/profile selection before hardware-address tokens become meaningful. | Hosted builds reject hardware-only constructs unless the correct profile is selected. |
| Code generation metadata | Emit LLVM metadata only after semantic acceptance. | IR snapshots show metadata placement, and tests confirm source-level checks still work without relying on optimizer behavior. |

This v2.0 track should remain open to contributors, but each implementation step must preserve deterministic builds and avoid claiming production-grade memory-security guarantees until the compiler has measured, repeatable evidence.

## Milestone 7: Plan the Logicodex Global Token Registry as an Offline-First Objective

The **Logicodex Global Token Registry** is a long-term ecosystem objective for the current logicodex v 1.21 alpha roadmap. The concept is to let future Logicodex installations discover expanded token dictionaries from a centrally maintained `global_map.json`, while still preserving deterministic compilation, reproducible builds, and local project control.

This feature should not be implemented as automatic HTTP lookup inside `src/lexer.rs`. Network access during lexing would make compilation depend on server availability, registry state, network latency, and policy changes outside the source tree. The practical design should instead use an explicit registry-management command that runs before compilation, validates registry data, writes a local cache, and records a project-pinned lockfile. The compiler should then read only local, pinned token maps during normal compilation.

| Registry objective | Current status | Practical acceptance signal |
|---|---|---|
| `global_map.json` registry format | Long-term objective | A versioned schema defines `identity`, expert canonical shorthand, `primary_ms` Malay alias, English/pseudocode aliases, namespace, stability, version-added metadata, checksum, and compatibility policy. |
| Offline-first sync command | Long-term objective | A command such as `logicodex registry sync --version 1.21-alpha` fetches, validates, and caches registry data before compilation. |
| Project lockfile | Long-term objective | A `logicodex.lock` or equivalent file pins registry version, checksum, and selected namespaces so repeated builds use the same token map. |
| Immutable token policy | Long-term objective | Published token identities are append-only or version-gated; conflicting changes are rejected rather than silently replacing existing meaning. |
| Audit and safety controls | Long-term objective | Registry updates produce an audit trail, reject malformed entries, and support checksum or signature verification before use. |
| Enterprise or premium policy layer | Optional future objective | License-gated namespaces are handled outside the open compiler core and must not make standard language compilation dependent on a live network call. |

The intended flow is therefore `registry sync -> schema/hash verification -> local cache and lockfile -> deterministic compiler read`. This keeps the global registry concept available for future ecosystem growth without weakening the stability of the current compiler pipeline.

## Tracking Notes

The roadmap should be updated only when implementation evidence changes. Completed items should cite the files, tests, examples, or release assets that prove the claim. When an item remains a goal, the documentation should use terms such as **planned**, **prototype**, **experimental**, **research objective**, or **long-term objective** rather than implying production readiness.

## Completed Vocabulary Milestone: Dictionary Vocabulary Expansion

The current logicodex v 1.21 alpha repository now includes a schema v2 token vocabulary model in `dict/core_map.json`, where `expert` is the compiler reference surface, `primary_ms` is the official Malay human alias, and `aliases` contains English pseudocode or compatibility spellings. This milestone is complete at the dictionary and lexer-recognition level after logical/static verification. The next practical milestones are to decide which of the new vocabulary families should become executable language features, then add parser, semantic-analysis, code-generation, examples, and tests for those features one at a time.

| Vocabulary family | Current status | Future implementation requirement |
|---|---|---|
| Program structure, bindings, conditionals, routines, and core types | Available in the dictionary and compatible with the lexer map | Keep examples synchronized with parser behavior. |
| Loops and bitwise markers | Available as token vocabulary and executable parser/semantic/codegen subset | Keep examples, tests, and `v130-check` compatibility synchronized as the subset expands. |
| Mutability, FFI, C interop, resources, and string type marker | Available as token vocabulary | Add parser and semantic rules before claiming executable feature support. |
| Hardware/address vocabulary | Available as token vocabulary, current hardware-zone provenance examples, and design direction | Add explicit target gates, deeper provenance rules, and freestanding backend examples before claiming runtime support. |
