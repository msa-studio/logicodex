## Unreleased

### Diagnostics

- Converted the clearest unsupported codegen fallback-zero paths into fail-fast
  errors instead of silently producing `0`.
- Added regression coverage for unsupported direct array literal expressions.
- Classified remaining zero-producing codegen paths into allowed, deferred, or
  ABI-policy categories.

### Lifecycle

- Resolved the two reviewed lifecycle orphans by removing the undocumented,
  duplicate AST `storage_width_bits` layout helper and the redundant primitive
  `is_signed_int` helper; lifecycle validation now prevents reintroduction and
  preserves the legacy AST analyzer boundary.
- Consolidated current authority behind one agent-facing entry point, demoted
  detailed roadmap/lifecycle documents to scoped evidence records, and added a
  fail-closed authority validator to the canonical integrity gates.

# Logicodex Changelog

All notable changes to the Logicodex compiler are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/), and this project adheres to [Semantic Versioning](https://semver.org/) for release versions.

---

## Versioning realignment (1.x -> 0.x)

As of 0.46.0-alpha, Logicodex is versioned under the **0.x** series. The earlier
`1.x` numbering wrongly implied a stable, production-ready, API-committed release
(SemVer: 1.0 means a committed public API). Logicodex is alpha research-grade
software that is still partial in many areas, so a `0.x` version is the honest
label (SemVer: 0.x means pre-1.0, unstable, API may change at any time).

The development-history number is preserved: the work previously tracked as
`1.46` is now `0.46.0-alpha`. The `1.21`-`1.46` entries below are kept unchanged
as a historical record; their numbers map onto the `0.x` series (1.46 -> 0.46,
1.45 -> 0.45, and so on). No past entry has been rewritten - only the current and
future versioning is corrected.

---

## [Unreleased]

Branch: `feature/stdlib-core` (target `main`). Not yet merged. This is the CPB
Phase 1 foundation: a contract-backed standard-library layer plus the generic
language capabilities it required. All stdlib modules are normal `.ldx` files
with `.std.toml` contracts, discovered through the ordinary import path; the
compiler stays generic (no module-specific wiring).

### Added
- **Module system (Stage 0).** Generic dotted-path imports via `import`,
  `public`, `begin`/`end` keywords and a `QualifiedCall` node (`core.x.fn(...)`),
  with `src/module_loader.rs` (topological graph, cycle detection,
  filesystem-relative resolution), name mangling, and private-by-default
  visibility. Proven by `tests/module_system_stage0.rs` and
  `tests/module_dotted_paths.rs`.
- **`lod` Stage 0** (`src/lod.rs`): a hand-written zero-dependency `logicodex.toml`
  reader for manifest-driven C-ABI linking (`[ffi] allow` + `[dependencies.c.*]`)
  that feeds the capability policy and linker.
- **FfiGatekeeper capability gate** (`src/semantic_gate.rs`, `src/ffi.rs`):
  default-deny `extern "C"`, runtime builtins auto-allowed, other foreign
  symbols require explicit `ffi.allow`.
- **Stage 0 contract-backed stdlib**: `core.math`, `core.assert`, `core.bits`,
  `core.bool`, `core.compare`, `core.range` (pure `I64` helpers), each with an
  `.ldx` source, a `.std.toml` contract, and a `tests/stdlib_core_*.rs` e2e test.
- **`core.prelude`** (explicit-import baseline): `id_i64`, `zero_i64`, `one_i64`,
  `truthy_i64`, `fallback_i64`. Not an automatic language prelude.
- **`core.text` (Stage 1)**: `is_empty_text_i64`, `not_empty_text_i64`,
  `same_emptiness_i64`, `select_by_empty_i64` (`String` in / `I64` out; only
  `== ""` / `!= ""` proven).
- **Stage 1 contract-backed `core.option`** for the `Option<I64>` slice
  (`is_some_i64`, `is_none_i64`, `unwrap_or_i64`), paired with
  `lib/core/option.std.toml` and proven by `tests/stdlib_core_result_option.rs`.
- **Stage 1 contract-backed `core.result`** for the `Result<I64, I64>` slice
  (`is_ok_i64`, `is_err_i64`, `unwrap_or_i64`, `unwrap_err_or_i64`), paired with
  `lib/core/result.std.toml`.
- **Result/Option compiler foundation** proven by
  `tests/compiler_result_option_foundation.rs`: enum-to-`i64` variant tags,
  typed `Ok`/`Err`/`Some`/`None` construction, `Result<I64, I64>` /
  `Option<I64>` returns, and `match` destructuring.
- **Collections foundation** (generic language capability): array-literal
  construction, fixed local arrays, and index read/write for `I64`, via the
  `ArrayLiteral` AST node and `Type::Array`. Proven by
  `tests/compiler_collections_foundation.rs`. No heap `Vec`/`List` and no slice
  passing yet (deferred).
- **`Modulo` and `BitXor` binary operators** (generic), with constant
  divide-by-zero checking extended to `Modulo`. Proven by
  `tests/modulo_operator.rs` and `tests/bitxor.rs`.
- **Actor runtime backend** (`src/runtime/runtime_actor.c`): audited pthread
  spawn/join/channel C backend completing the previously codegen-only ABI, plus
  cross-actor channel capture (actor `params`) and `ChannelCreate`.
- **Contract-verification infrastructure**: `tools/verify_stdlib_contracts.py`
  (metadata validation + bounded run-cases), `scripts/dev/` verify scripts, and
  the `.std.toml` contract format. Wired into CI (`check` validates contracts;
  `test` runs `--run-cases`).
- **`tests/stdlib_root_resolution.rs`** for std-root module path resolution.

### Changed
- `lib/core/result.ldx` was narrowed from the older aspirational generic
  `Result<T, E>` sketch (which advertised `expect`, `map`, etc. in the PR #25
  entry below) down to the compiler-proven Stage 1 `Result<I64, I64>` API. The
  generic surface is intentionally **not** shipped yet.
- Parser now accepts `result` as a dotted stdlib module leaf (`core.result`)
  even though `Result` remains a type keyword; `option`/`Some`/`None` stay plain
  identifiers.
- Frozen core files received additive, documented changes: `src/ast.rs` (new
  nodes/fields: `is_public`, actor `params`, `QualifiedCall`, `ArrayLiteral`,
  Option `Some`/`None`, `ChannelCreate`, `Modulo`/`BitXor`, `Type::Array`/
  `Type::Option`) and `src/semantic.rs` (rest-pattern tolerance, Modulo
  divide-by-zero, allowed-operator list). No existing semantics removed.
- Documentation aligned to the proven state: `result-option-foundation.md`
  (foundation marked proven, transitional scalar encoding documented),
  `stdlib-migration-status.md` (`core.result` moved out of the legacy lists;
  `core.option` + `core.result` added as ContractVerified Stage 1),
  `compiler-subset.md` P1-B2, `SPECIFICATION.md` (Option/Result generic
  qualifier + diagnostics-status note), and the merge/stability audits under
  `docs/audit/`.

### Notes
- The Result/Option Stage 1 slice uses a transitional scalar `i64` encoding
  (`Ok`/`Some(v) = (v << 1) | 1`, `Err(e) = e << 1`, `None = 0`), not the final
  tagged-union layout. Payloads are effectively 63-bit and `match` lowers to an
  if-chain (no exhaustiveness checking yet). Enum tagged-union layout
  (`TypeRegistry` size/align for `Enum`) remains deferred (Sprint 2.5) and is
  currently unreachable.
- Deferred: generic `Option<T>` / `Result<T, E>`, `map` / `and_then` /
  `expect` / panic helpers, String / IO error payloads, nested Result/Option,
  heap-backed collections (`Vec`/`List`) and slice passing, custom-enum `match`,
  high-level file/network IO, and the LDX-DIP diagnostic-intelligence layer.
- Merging this branch to `main` trips the size gate (>500 lines) and the
  architecture-freeze gate (`src/ast.rs`, `src/semantic.rs`); both require a
  maintainer override. See `docs/audit/stdlib-core-to-main-merge-readiness.md`.

---

## [0.46.0-alpha] - 2026-06-14 - End-to-End .ldx -> Kernel + Single-Engine Cleanup + Honest Versioning

### Changed
- **Versioning realigned from `1.x` to `0.x`** to honestly reflect pre-1.0,
  alpha status (see the note above). Current version: `0.46.0-alpha`.

### Added
- End-to-end `.ldx` -> bootable x86_64 kernel: a `.ldx` program compiles to a
  freestanding object, links with the `logicodex-os` runtime, and boots in QEMU
  running its own code. CI-guarded via `make boot-e2e` (tested: structs,
  recursion, fib, multi-arg functions, if/else, nested loops).
- `examples/freestanding/{minimal,showcase}.ldx` + `make boot-e2e` regression guard.

### Changed (engine)
- Single-engine cleanup: removed the `v1_30` feature flag entirely (49 cfg sites
  + dead legacy non-HIR arms). HIR is now the unconditional, sole engine.
- Rewrote CI for single-engine reality (check / test / freestanding boot+e2e);
  bumped actions to Node 24.

### Removed
- Archived 26 pre-HIR validators that tested the retired pre-HIR architecture
  (kept under `scripts/validators/_archive_pre_hir/` as history). Live QA is now
  `cargo test` + `make boot-evidence`/`boot-e2e`.

### Fixed
- `logicodex_clamp_f32` multiply-defined under release LTO (dead `math_shims`).
- Repo hygiene: removed empty paste artifacts, Windows Zone.Identifier metadata,
  and stale build/test logs; cleaned stale era references in source comments
  (now all English, reflecting the single HIR engine); fixed Makefile validator
  targets to point at the live validator set.

---

## [v1.46.0-alpha] — 2026-06-10 — HIR Activated: Single Engine

### Summary
The HIR pipeline is now the **sole execution path** (Issue #02 resolved: ACTIVATE).
`.ldx` compiles through Lexer → Parser → AST → HIR lowering → semantic_gate → LLVM IR
→ native binary. The legacy v1.21 AST-codegen path and v1.21 Analyzer gate were retired.
See `docs/architecture/hir-decision.md`.

### Added — executing end-to-end (HIR + LLVM)
- Variables, assignment, print; functions, params, calls, recursion
- Arithmetic/comparisons; if-else, while (`SELAGI`), loop (`ULANG`), break/continue
- Structs: construction, field read, field assignment, struct-as-parameter
- Enums + pattern matching: variant refs (`Warna::Merah` → tag), `MATCH` → if/else chain, wildcard `_`

### Added — type inference, struct returns, postfix chaining
- Call return-type inference: each callable's return type is resolved in a pre-pass and attached to `Call` expressions (previously typed `Unknown`), enabling precise typing of call results
- Struct returns via the **sret ABI**: a struct-returning function takes a hidden caller-allocated buffer pointer, copies its fields into it on return, and returns that pointer
- Chained postfix field access: `buat().x` (field access on a call result) and `a.b.c` now parse via a `parse_postfix` loop in `unary()`
- Fixed-width integers (I8/I16/I32/I64/U8/U16/U32/U64): true wrapping via `wrap_to_width` (truncate to width then sign/zero-extend) at every boundary — literal/let init, assignment, struct-field store, each arithmetic/unary op (per-op wrap), incoming parameters, return values, and call results. Uniform i64 storage/working model retained (no layout/ABI change); only values wrap. All 8 widths enabled via `named_type_id` uppercase aliases
- Capability verification (Issue #07): the dormant tier2 Capability Fabric is now wired into `check` — each `Service` `requires` clause is validated against the standard gate vocabulary (`all_standard_domains`). A malformed gate (not `Domain.Operation`) is an error; a well-formed gate outside the vocabulary is a warning. Compile-time only, consistent with the fabric's zero-runtime-mediation design
- Foreign-type translation scaffold: `LegacyType` enum (full C scalar family + Pascal) with `canonical_native()` mapping to native primitives — inert, translation-only; plus integer width helpers (`int_bits`, `is_signed_int`). See `docs/architecture/foreign-types.md`

### Fixed
- Returning a struct by value no longer yields a dangling pointer (second field previously read garbage); struct returns now copy into a caller-provided buffer (sret)
- Service declarations now parse: field names (`port`/`requires`/`handler`/`policy`) and bareword values that lex as keywords are accepted (previously the parser required identifiers and rejected its own field keywords)
- Member access accepts keyword-named members: `expr.NAME` where `NAME` collides with a keyword now parses (via `consume_member_name`), enabling gate operations like `Net.Send`, `Storage.Baca`, `UI.Papar`
- Keyword namespace bases: a word-like keyword immediately followed by `.` is parsed as a `Variable` base (e.g. the `HW` domain in `HW.GPIO`); bare keywords in expression position still error

### Changed
- `check` runs full v1.30 validation (parse → lower → semantic_gate)
- Duplicate-function & division-by-zero checks ported from v1.21 Analyzer into `semantic_gate`
- `lower_v121_program` → `lower_program`

### Removed
- v1.21 AST codegen (~646 lines): compile_to_object, emit_program/stmt/expr, hardware/MMIO emitters
- v1.21 Analyzer retired from pipeline (module kept for reference)
- `CompilerPipeline::V121` variant (`--pipeline v1.21` now a deprecated alias)
- Dead `unimplemented_feature` + `UnimplementedFeature` variant

### Architecture
- Adopted the **tiered runtime doctrine** (`docs/architecture/runtime-doctrine.md`): *Zero Runtime Core, Optional Runtime Profiles, Portable Semantic IR*. The core stays runtime-free (everything compile-away: types, fixed-width ints, structs/enums, control flow, FFI boundary, static capability validation); runtime features (actors, channels, sandbox I/O, runtime capability enforcement) are opt-in profiles layered above the backend. Resolves the #07 zero-runtime-vs-enforcement tension (validation=core, enforcement=profile)

### Notes
- Compiler-guided dead-code removal; `cargo fix` for mechanical lints; zero regression
- Roadmap subsystems (OS/freestanding, capability/sharded/actor runtime) intact — dead_code = honest "not yet wired"

### Known stopgaps
- Method calls on expression results (`buat().m()`) not yet supported (`Expr::MethodCall` carries a name, not an expression)

## [v1.45.0-alpha] — 2026-05-25 — Quantitative Benchmark Framework (Layers 1-3)

### Summary
Architecture-correlated benchmark framework with 4 layers. 20 new files (~2,200 LOC). Layer 4 (security) stubs created. 11/11 Tier C validator checks passing.

### Layer 1: Micro-Benchmarks (6 criterion benchmarks)
- `gate_latency.rs` — Capability Fabric gate check (target: < 50ns)
- `door_latency.rs` — Channel<T> send/recv/roundtrip (target: < 100ns)
- `mempool_latency.rs` — Bump allocator acquire (target: < 20ns)
- `callable_lookup.rs` — CallableRegistry by-name lookup (target: < 30ns)
- `hir_lower.rs` — AST → HIR expression lowering (target: < 200ns)
- `llvm_emit.rs` — LLVM IR generation (target: < 500ns)

### Layer 2: Reactor Throughput
- `echo_server.rs` — epoll-based TCP echo server with CPU affinity
- `flood_client.rs` — Multi-threaded flood generator
- `throughput.sh` — 1/2/4/8 core scaling test with efficiency calculation
- Target: > 85% scaling efficiency at 8 cores

### Layer 3: System Stability
- `rss_monitor.py` — /proc/[pid]/status snapshotter with linear regression creep detection
- `valgrind_check.sh` — Full leak check (target: 0 leaks)
- `longrun.sh` — 1h/6h/24h automated stability test
- Acceptance: Linear regression slope ≤ 0.001 KB/hour

### Layer 4: Security Stress (stubs)
- `slowloris.py` — Partial read attack (tests Taint FSM)
- `syn_flood.py` — Connection flood (tests backpressure)
- `malformed.py` — Random byte injection (tests EPOLLERR cleanup)
- `fd_exhaustion.py` — EMFILE boundary test (tests graceful degradation)

### Infrastructure
- `benches/BASELINE.json` — Gold standard with regression thresholds (5% warn, 10% fail)
- `benches/harness/run_all.sh` — All-layers runner (quick/full/compare modes)
- `benches/harness/compare_baseline.py` — Regression detection tool
- `docs/RFC_TEMPLATE.md` — Architecture Freeze enforcement template

### Validation

> **Correction (2026-06):** the previously stated "148/148 ✅" was not reproducible. A full `cargo test --features v1_30` inventory shows **5 of 34 test targets compile**; the remaining 29 fail to compile due to test-suite drift (removed `net` module, AST evolution, visibility changes). The suite requires rehabilitation (Issue #13 / ROADMAP P1-D2). No verified aggregate pass count is claimed until the suite compiles.

---

## [v1.44.1-alpha] — 2026-05-25 — Foundation Polish: Validator Tiering + Maintenance Report

### Summary
First maintenance release under Architecture Freeze. Seven maintenance items completed across validator health, dead code audit, test coverage analysis, security micro-audit, documentation drift correction, and minor polish. All 27 validators now pass (was 3/27). Foundation integrity: 137/137 checks.

### Changes

#### 1. Validator Tiering (3-tier system)
- Created `scripts/validators/{tier_a_core,tier_b_feature,tier_c_stress}/`
- Tier A (7): Core integrity — build stops on failure
- Tier B (13): Feature correctness — warning on failure
- Tier C (7): Platform/stress — CI only

#### 2. Validator Health Fixes
- `validate_v121_executable_logic.py`: Version-agnostic semver check (was hardcoded `1.21.0-alpha`)
- `validate_v121_alpha_deployment.py`: Flexible header check (was exact string match)
- `validate_v130_pipeline.py`: README check now version-agnostic

#### 3. Dead Code Audit
- TODO: 1 (ctl_mapper.rs:460 — BY DESIGN, WIT template placeholder)
- `#[allow(unused)]`: 0 | `#[allow(dead_code)]`: 0 | `todo!()`: 0

#### 4. Test Coverage Analysis
- 5 modules without dedicated tests identified (all low risk)
- Test-to-source ratio: 69% (34/49 files)

#### 5. Security Micro-Audit
- `.unwrap()` in production: **0** (all 7 are in `#[cfg(test)]`)
- `unsafe` blocks: 141 — all documented with safety preconditions
- `as` casts: 134 — all safe (widening or pointer-to-int)

#### 6. Documentation Drift Fixed
- README.md: Version `v1.44.0-alpha`, stats `~40,700 LOC | 137/137 checks`
- ARCHITECTURE.md: Validation table updated to 137/137
- docs/MAINTENANCE_v1441.md: Comprehensive maintenance report

### Validation: 137/137 ✅ (was 102/102)

---

## [v1.44.0-alpha] — 2026-05-25 — Freestanding Compiler — All 15 Gaps Resolved

### Summary
Resolved all 15 gaps preventing Logicodex from being a true freestanding (bare-metal) compiler. Freestanding readiness: **100%** (Tier 1: 5/5, Tier 2: 5/5, Tier 3: 5/5). Total: ~2,000+ new LOC across 11 new files.

### Tier 1: MUST HAVE (G1-G5) — Without these, can't link/run

| Gap | File | Description | LOC |
|---|---|---|---|
| **G1** | `src/os/startup.rs` | `_start` entry: set stack (2MB), zero BSS, copy data, call main, halt | 120 |
| **G2** | `src/os/panic.rs` | `#[panic_handler]`: clear SSE registers (xmm0-3), UART output, halt loop | 70 |
| **G3** | `lib/linker_scripts/x86_64-freestanding.ld` | Memory layout: code at 1MB, stack 1-2MB, heap after BSS | 50 |
| **G4** | `src/os/allocator.rs` | Bump allocator: AtomicUsize CAS, `#[global_allocator]`, OOM returns null | 180 |
| **G5** | `src/os/uart.rs` | x86_64 port I/O: `uart_putc/puts/hex`, `VgaWriter` (0xB8000), `uart_print!` macros | 280 |

### Tier 2: HIGH (G6-G10) — Can't compile without these

| Gap | File | Description | LOC |
|---|---|---|---|
| **G6** | `src/lib.rs` | `#![no_std]` + `extern crate alloc` + conditional re-exports (Vec, String, HashMap) | 15 |
| **G7** | `src/os/source_provider.rs` | `SourceProvider` trait: `FileSystemProvider` (hosted), `EmbeddedProvider`, `BinaryProvider` | 120 |
| **G8** | `src/os/target.rs` | `TargetArch` enum (x86_64/aarch64/riscv64), `build_target_machine_with_arch()`, CLI `--target freestanding-<arch>` | 80 |
| **G9** | `src/os/target.rs` | Fixed `+soft-float` → `+sse2` for x86_64 (x86-64 CPUs have SSE2 by default) | 5 |
| **G10** | `src/os/startup.rs` | BSS zeroing (`write_bytes`) + data copy (`copy_nonoverlapping`) in `_start` | (in G1) |

### Tier 3: MEDIUM (G11-G15) — Run but limited without these

| Gap | File | Description | LOC |
|---|---|---|---|
| **G11** | `src/os/interrupts.rs` | IDT (256 entries), 32 CPU exception handlers, PIC remap (IRQ 32-47), `irq_enable/disable` | 320 |
| **G12** | `src/codegen.rs` | `emit_hardware_zone()` + `emit_mmio_volatile_write/read()` — volatile store/load for MMIO | 80 |
| **G13** | `lib/startup/multiboot_header.rs` | Multiboot header (0x1BADB002), GRUB-compatible, `.multiboot` linker section | 80 |
| **G14** | `src/os/startup.rs` | Stack pointer init: `mov rsp, 0x200000` in `_start` | (in G1) |
| **G15** | `build.rs` | Raylib detection (pkg-config, RAYLIB_DIR, platform paths) + graceful fallback | 80 |

### Architecture Support

| Architecture | LLVM Triple | Features | Code Model |
|---|---|---|---|
| x86_64 (default) | `x86_64-unknown-none` | `+sse2` | Kernel |
| aarch64 | `aarch64-unknown-none` | (default) | Small |
| riscv64 | `riscv64gc-unknown-none-elf` | (default) | Medium |

### Validation
- v1.44 Freestanding Gaps: **15/15 ✅**
- v1.43 Raylib Audio: 80/80 | v1.42 Raylib Pending: 9/9 | Host Reactor: 20/20 | WASM: 13/13
- **Total: 137/137 ✅**

---

## [Merged] — 2026-05-25 — v1.43.0-alpha: Raylib Audio — 22 Functions + StrictAudioContext Integration

### Summary
Added 22 Raylib audio functions to the FFI layer, integrated with the existing bare metal audio capability system. No conflict between the two systems — they are complementary: Raylib provides the **implementation** (how to play audio), the capability system provides **security** (who can access audio).

### Audio Types (`src/ffi/raylib_sys.rs`)
- `Wave` — raw audio data (samples + format)
- `Sound` — loaded short audio (fully in memory)
- `Music` — streaming long audio (decoded on the fly)
- `AudioStream` — custom real-time audio stream
- `AudioCallback` — function pointer signature for `SetAudioStreamCallback`

### Audio Functions — 22 Registered in CallableRegistry

| Category | Functions | Count |
|---|---|---|
| Device | `InitAudioDevice`, `CloseAudioDevice`, `IsAudioDeviceReady`, `SetMasterVolume` | 4 |
| Sound | `LoadSound`, `UnloadSound`, `PlaySound`, `StopSound`, `IsSoundPlaying` | 5 |
| Music | `LoadMusicStream`, `UnloadMusicStream`, `PlayMusicStream`, `StopMusicStream`, `IsMusicStreamPlaying`, `UpdateMusicStream`, `SetMusicVolume`, `SeekMusicStream` | 8 |
| Stream | `LoadAudioStream`, `UnloadAudioStream`, `PlayAudioStream`, `StopAudioStream`, `IsAudioStreamPlaying` | 5 |

All audio functions: `UnsafeRequired`, C ABI. Sound/Music/Stream use `i64` handles (opaque).

### Integration with StrictAudioContext
- `SetAudioStreamCallback(stream, callback)` → triggers `Analyzer::register_audio_callback(func_name)`
- Callback function validated by `verify_audio_safety()` against 4 violation types:
  - `AudioViolationIo` — no Print/DrawText/InitWindow in audio ISR
  - `AudioViolationRecursion` — no self-calling
  - `AudioViolationUnboundedLoop` — no unbounded `loop { }`
  - `AudioViolationForbiddenCall` — no malloc/free/spawn

### Integration with Capability System
- Audio functions map to `Audio.Main` capability gate (`lib/core/capability.ldx`)
- CTL Mapper: `Audio` domain → `wasi:io/custom` for WASM targets
- Host Reactor mediates audio access for WASM guests

### Safe Wrappers (`src/ffi/raylib.rs`)
22 safe wrapper functions with proper documentation and safety notes.

### Validation
- v1.43 Audio Integration: 80/80 | v1.42 Raylib Pending: 9/9 | Host Reactor: 20/20 | WASM Backend: 13/13 | **Total: 122/122 ✅**

---

## [Merged] — 2026-05-25 — v1.42.0-alpha: Raylib FFI — 8 Pending Items Resolved

### Summary
All 8 long-standing Raylib FFI pending items from the architecture review have been resolved. This closes the gap between compile-time validation and runtime linking for graphics applications.

### P1: `build.rs` — Raylib Detection + Graceful Fallback
- `build.rs`: Auto-detect Raylib via `pkg-config`, `RAYLIB_DIR` env var, or platform-specific paths.
- `RAYLIB_NO_LINK=1`: Opt-out flag for builds without Raylib installed.
- Graceful fallback: warning emitted, build continues (no link error).
- Supported: Linux (`apt install libraylib-dev`), macOS (`brew install raylib`), Windows (`RAYLIB_DIR`).

### P2: Color Struct-by-Value Passing
- `register_raylib_functions()`: Drawing functions now take `Color` struct type (not packed `u32`).
- `ClearBackground`, `DrawText`, `DrawRectangle`, `DrawCircle`, `DrawLine`, `DrawRectangleLines`, `DrawPixel` — all use struct type.
- Texture functions: `LoadTexture` returns `Texture2D` struct, `UnloadTexture` takes `Texture2D` struct (not `i64` handle).

### P3: Vector2/Rectangle/Texture2D Struct Constructors
- `is_struct_constructor()`: Detects `Color`, `Vector2`, `Rectangle`.
- `struct_constructor_arity()`: Returns param count (Color=4, Vector2=2, Rectangle=4).
- `emit_hir_struct_constructor()`: LLVM codegen for `Vector2(x, y)` and `Rectangle(x, y, w, h)` constructors.

### P4: Math Utilities in CallableRegistry
- `clamp(v, min, max)`, `lerp(a, b, t)`, `remap(v, l1, h1, l2, h2)`, `normalize(v, low, high)`.
- All registered as `CallableSafety::Safe` (no unsafe required).
- `math_shims` module: `extern "C"` wrappers for LLVM-generated code to call.

### P5: Runtime Linking Integration
- 28 Raylib functions + 4 math functions = 32 total registered functions.
- All functions: `CallableSafety::UnsafeRequired`, C ABI.
- `register_raylib_functions_compat()`: Backward-compatible wrapper for existing tests.

### P6: StrictAudioContext — Hardware-Safe Audio Guards
- 4 violation types: `AudioViolationIo`, `AudioViolationRecursion`, `AudioViolationUnboundedLoop`, `AudioViolationForbiddenCall`.
- `register_audio_callback(name)`: Mark function as audio ISR.
- `verify_audio_safety()`: Walks AST, validates against all 4 violation types.
- Forbidden: `Print`, `DrawText`, `InitWindow` in callbacks; self-recursion; unbounded `loop { }`; `malloc`/`free`/`spawn` calls.

### P7: WASM Target Blocks Raylib
- `compile_v130_pipeline()`: When `target.is_wasm()`, Raylib functions are detected and removed from `CallableRegistry`.
- Error message: "WASM target does not support Raylib graphics functions — use WebGL or Canvas API via the WASM host instead."
- Math utilities (`clamp`, `lerp`, `remap`, `normalize`) are NOT blocked — they are pure Rust.

### P8: FfiGatekeeper Coercion Support
- `is_compatible_with_coercion()`: Widening coercion matrix.
- Allowed: `I32 → I64`, `I32 → F32/F64`, `I64 → F64`, `F32 → F64`, `U8 → I32/I64`.
- Bilingual error messages with type names in diagnostics.

### Validation
- v1.42 Raylib Pending: 9/9 | Host Reactor: 20/20 | WASM Backend: 13/13 | Sharded Runtime: 10/10 | Network Runtime: 16/16 | CTL Mapper: 12/12 | Capability IR: 16/16 | Capability Fabric: 10/10 | Streaming: 6/6 | v1.21: 9/9 | **Total: 111/111 ✅ + runtime live + sharded + wasm + host**

---

## [Merged] — 2026-05-25 — v1.41.0-alpha: Host Reactor Integration — Guest ↔ Host HW Mediation

### Added
- **HostReactor** (`src/net/host_reactor.rs`): Central struct mediating all HW gate access.
  - `GatePermissions`: Per-operation pin allowlists — which pins each WASM guest can access.
  - `HardwareZone`: Pin claim/release tracking — prevents double-use conflicts.
  - `with_hardware_zone()`: Validates permission → claims pin → executes callback → releases pin (always, even on error).
- **HW Gate Implementations**:
  - `gpio_control(pin, mode)`: GPIO pin control — input/output/pullup/pulldown/high/low.
  - `timer_set(pin, micros)`: Hardware timer configuration.
  - `dma_transfer(channel, src, dst, len)`: DMA data movement between addresses.
- **Guest → Host Dispatch Protocol**:
  - `HostFunction` enum: `GpioControl`, `TimerSet`, `DmaTransfer`.
  - `register_host_function(name)`: Maps WIT import name to `HostFunction`.
  - `dispatch(func, args)`: Called by WASM runtime when guest imports are invoked.
  - `GuestRequest` / `HostResponse`: Serialization envelopes for guest-host communication.
- **Permission Denied Handling**: All HW operations check `GatePermissions` before execution. Unauthorized access returns `HostReactorError::PermissionDenied` — HW is never exposed to unprivileged guests.

### Architecture
```
WASM Guest                      Host (Native)
──────────                      ─────────────
import "logicodex:host-reactor/gpio-control"
        │
        ▼
WASM Runtime (wasmtime/wasmer) ──► HostFunction::GpioControl
                                          │
                                          ▼
                                   HostReactor.gpio_control()
                                          │
                                          ▼
                                   GatePermissions.check()
                                   HardwareZone.claim()
                                   [actual GPIO driver]
                                   HardwareZone.release()
                                          │
                                          ▼
                                   Return u32 to guest
```

### Validation
- Host Reactor: 12/12 | WASM Backend: 13/13 | Sharded Reactor: 11/11 | Capability IR: 16/16 | CTL Mapper: 12/12 | Capability Fabric: 10/10 | Streaming: 6/6 | v1.21: 9/9 | **Total: 102/102 ✅ + runtime live + sharded + wasm + host**

---

## [Merged] — 2026-05-25 — v1.40.0-alpha: WASM Codegen Backend — LLVM → .wasm

### Added
- **CompilationTarget::Wasm**: New target variant parsed from `wasm` and `wasm32`.
  - `entry_symbol()`: `_start` (WASM convention)
  - `llvm_triple()`: `wasm32-unknown-unknown`
  - `is_wasm()`: Check for WASM targets
- **OutputKind::WasmModule**: New output kind for WASM generation.
- **build_target_machine() WASM support**: LLVM WASM backend with:
  - Target triple: `wasm32-unknown-unknown`
  - CPU: `generic`
  - Features: `+bulk-memory,+mutable-globals,+sign-ext`
  - Relocation: Static
  - Optimization: Default (size-conscious)
- **Codegen WASM paths**: Both v1.21 (`compile`) and v1.30 (`compile_v130`) detect `is_wasm()` and select `OutputKind::WasmModule`.
- **CLI `--target wasm`**: Recognized in argument parser. WASM-specific output messages and `wasm-ld` linking hints.
- **Syscall**: `syscall0()` helper (no-argument syscall, used for `sched_getcpu`).

### Usage
```bash
logicodex --target wasm input.ldx -o output.wasm
wasm-ld --no-entry -o final.wasm output.wasm --export-all
```

### Validation
- WASM Backend: 13/13 | Network Reactor: 13/13 | Sharded Reactor: 11/11 | Capability IR: 16/16 | CTL Mapper: 12/12 | Capability Fabric: 10/10 | Streaming: 6/6 | v1.21: 9/9 | **Total: 90/90 ✅ + runtime live + sharded + wasm**

---

## [Merged] — 2026-05-25 — v1.39.0-alpha: Sharded Runtime — Real Threads + CPU Affinity

### Summary
**ALL 26 DEFERRED ITEMS NOW RESOLVED.** 25 implemented, 1 by design (H1 Edition Routing).

### C1 — Thread Spawning
- `ShardedReactor::start()`: Spawns `std::thread` per shard via `spawn()`.
- Each thread: sets CPU affinity → runs reactor event loop.
- Thread handles stored in `Vec<Option<JoinHandle<()>>>`.
- `active_threads()`: Returns count of spawned threads.

### C2 — Parallel Execution
- All shards run simultaneously in their own OS threads.
- No more sequential execution — replaced with parallel `Vec<JoinHandle>`.
- `stop()`: Joins all threads on shutdown.

### C3 — CPU Affinity Linux
- `set_cpu_affinity()`: `sched_setaffinity` syscall (`SYS_SCHED_SETAFFINITY=203`).
- Builds `cpu_set_t` bitmap (512 bytes), sets bit for target core.
- `num_cpus()`: `std::thread::available_parallelism()` (not hardcoded 4).
- `current_core_id()`: `sched_getcpu` syscall (`SYS_SCHED_GETCPU=309`).
- `affinity_info()`: Diagnostic string with cores/current/platform.

### C4 — CPU Affinity macOS
- `set_cpu_affinity()`: Returns `UnsupportedPlatform` with diagnostic.
- Notes `thread_policy_set` requirement for future Mach framework integration.

### C5 — CPU Affinity Windows
- `set_cpu_affinity()`: Returns `UnsupportedPlatform` with diagnostic.
- Notes `SetThreadAffinityMask` requirement + CallableRegistry FFI path.

### Validation
- Network Reactor: 13/13 | Sharded Reactor: 11/11 | Capability IR: 16/16 | CTL Mapper: 12/12 | Capability Fabric: 10/10 | Streaming: 6/6 | v1.21: 9/9 | **Total: 77/77 ✅ + runtime live + sharded**

---

## [Merged] — 2026-05-25 — v1.38.0-alpha: Deferred Items Cleanup — 8 Items Resolved

### Summary
Closed 8 long-standing deferred items. **20 of 26 total deferred items are now resolved.** Remaining: C1-C5 (Sharded Runtime) → v1.39.

### A6 — CallableRegistry Integration
- `predeclare_callables()`: Iterates all registered callables, declares them in LLVM module before HIR codegen begins. Prevents "CallableRegistry not attached" errors.
- Integrated at start of `compile_v130()`.

### D1 — from_topology() Fix
- Added accessor methods to `CapabilityTopology`: `contracts()`, `providers_of()`, `consumers_of()`, `all_providers()`, `all_consumers()`, `module_symbol()`.
- `from_topology()`: Now imports all `GateContract` entries as `IRGateEdge` into CapabilityGraph.

### E1 — Struct Type Resolution
- Clarified design: struct constructors returning `I64` (packed value) is intentional — value types packed into integer registers.

### E2 — Enum Layout
- Added `enum_layouts: Vec<EnumLayout>` to `TypeRegistry` with `register_enum_layout()` and `get_enum_layout()` methods.
- `layout.rs`: `TypeKind::Enum` now looks up cached layout (fallback to `u32` for unregistered enums).

### F1 — Windows Syscall Fallback
- `open_file()`: Returns `Err(-1)` with diagnostic instead of `unimplemented!()` panic.
- `win_recv_fallback()` + `win_send_fallback()`: Graceful error returns.

### G1 — Memory Attestation (--secure)
- `compute_module_hash()`: Simple folding hash (placeholder for future SHA-256 over `.text` section).
- `--secure` flag now includes computed hash in security plan document.

### G2 — Freestanding Target (--target freestanding)
- `select_freestanding_target_triple()`: Returns `x86_64-unknown-none-elf`, `aarch64-unknown-none`, or `riscv64gc-unknown-none-elf` based on host arch.
- `--target freestanding` now includes selected LLVM triple in plan document.

### I1 — Semantic Gatekeeper Activation
- Removed `#![allow(dead_code)]`, added module documentation.
- `validate_module()`: Public API for final validation pass.
- `validate_module_with_reporting()`: Convenience function with diagnostics.
- Integrated into `compile_v130()`: Runs as final validation pass before LLVM codegen (non-fatal).

### Validation
- Network Reactor: 13/13 | Sharded Reactor: 11/11 | Capability IR: 16/16 | CTL Mapper: 12/12 | Capability Fabric: 10/10 | Streaming: 6/6 | v1.21: 9/9 | **Total: 77/77 ✅ + runtime live**

---

## [Merged] — 2026-05-25 — v1.37.0-alpha: Deterministic Network Runtime — From Compile-Time to Live

### The Gap Being Closed
The v1.33 Network Reactor provided compile-time verification (syntax, topology, taint analysis, backpressure policies) but all runtime operations were stubs. v1.37 closes this gap — the reactor now runs live with real syscalls.

### Implemented
- **B1 — epoll Event Loop** (`src/net/reactor.rs`): Real epoll via `epoll_create1(0)`, `epoll_ctl` for ADD/MOD/DEL, `epoll_wait` for event collection. Event loop runs until `stop()` called.
- **B2 — Connection I/O** (`src/net/connection.rs`): `read()` → `SYS_RECV`, `write()` → `SYS_SEND` via `src/os/syscall.rs`.
- **B3 — Monotonic Timestamp** (`src/net/connection.rs`): `clock_gettime(CLOCK_MONOTONIC)` → millisecond timestamp for taint timeout.
- **B4 — Event Processing** (`src/net/reactor.rs`): `process_events()` dispatches `EPOLLIN`/`EPOLLOUT`/`EPOLLERR` to connection handlers.
- **B5 — Taint FSM** (`src/net/reactor.rs`): `Healthy→Suspicious→Closing` transitions on error threshold + idle timeout. `is_trustworthy()` gates all I/O.
- **B6 — Backpressure** (`src/net/reactor.rs`): Runtime policies — `Block` (spin-wait on full), `DropOldest` (overwrite oldest), `Error` (return false).

### Validation
- Network Reactor: 13/13 | Sharded Reactor: 11/11 | Capability IR: 16/16 | CTL Mapper: 12/12 | Capability Fabric: 10/10 | Streaming: 6/6 | v1.21: 9/9 | **Total: 77/77 ✅ + runtime live**

### Deferred Items Closed
- B1-B6: All 6 network runtime stubs from DEFERRED.md resolved.

---

## [Merged via PR #38] — 2026-05-25 — v1.36.0-alpha: CTL Mapper — WIT Auto-Generation (Fasa B)

### Added
- **CTL Mapper** (`src/tier2/ctl_mapper.rs`): Capability Translation Layer — bridges Logicodex capability world to WASM ecosystem
  - `WitDomain` enum: 6 standard mappings — `Storage→wasi:filesystem`, `Net→wasi:sockets`, `UI→wasi:cli`, `HW→HostReactor`, `Audio→wasi:io/custom`, `Crypto→wasi:crypto`
  - `WitOperation`: WIT function signature generation with typed parameters and return values
  - `get_wit_operations()`: domain-specific operation lookup (3 ops per standard domain)
  - `CtlMapper`: core mapper struct — `map_capability()`, `map_graph()`, `generate_wit()`, `generate_host_reactor_stub()`
  - Manual overrides: `add_override()` lets users define custom WIT mappings that take precedence over auto-mapping
  - HW gate detection: HW gates are routed through Host Reactor, NEVER exposed to WASM guest
  - Unknown domain fallback: maps to `logicodex:custom` interface
  - `CtlMappingStats`: reports mappings applied, HW gates detected, unknown domains, overrides used
  - Pipeline functions: `map_and_generate_wit()` (one-shot), `map_and_generate_wit_with_overrides()`
- **Module exports** (`src/tier2/mod.rs`): `CtlMapper`, `CtlMappingStats`, `WitDomain`, `WitOperation`, `get_wit_operations`, `map_and_generate_wit`
- **Tests**: `tests/ctl_mapper.rs` (16 test groups — all 6 domains + overrides + HW + pipeline)
- **Validator**: `scripts/validate_ctl_mapper.py` (12 checks)

### Design Philosophy
> "Project INTO, not borrow FROM" — Logicodex domains are primary; WASI is a projection target.

### Validation
- CTL Mapper: 12/12 | Capability IR: 16/16 | Sharded Reactor: 11/11 | Network Reactor: 13/13 | Capability Fabric: 10/10 | Streaming: 6/6 | v1.21: 9/9 | **Total: 77/77 ✅**

---

## [Merged via PR #37] — 2026-05-25 — v1.35.0-alpha: CapabilityGraph IR — Single Source of Truth (Fasa A)

### Added
- **CapabilityGraph IR** (`src/tier2/capability_ir.rs`): Unified "Single Source of Truth" language-agnostic capability representation
  - `CompileTarget`: `Native` (ELF), `Wasm`, `All` (dual artifacts) — determines output safety rules
  - `CapabilityRef`: `domain.operation + GateType + optional WIT mapping` — canonical capability reference
  - `IRServiceNode`: unified service node merging v1.31 SemanticSummary + v1.32 gates + v1.34 shard info
  - `IRShardNode`: `core_id + budget_mb + allowed_gates + service IDs`
  - `IRDoorEdge + IRGateEdge`: unified edge types (cross-shard communication + capability contracts)
  - `CapabilityGraph`: THE IR — services, shards, doors, gates, target — generates all output targets
  - `verify()`: 6 unified checks — `EmptyGraph`, `WasmHardwareGate`, `InvalidShardAssignment`, `UnknownServiceInDoor`, `UnknownServiceInGate`, `EmptyShard`
  - `to_cap()`: `.cap` audit manifest generation (SERVICES/SHARDS/DOORS/GATES sections)
  - `to_wit_stub()`: WIT string generation stub — foundation for Fasa B CTL Mapper
  - Integration: `from_semantic_summaries()` (v1.31), `from_topology()` (v1.32), `from_shard_topology()` (v1.34)
- **Module exports** (`src/tier2/mod.rs`): `CapabilityGraph`, `CapabilityRef`, `CompileTarget`, `IR*`, `IRVerifyResult`, `IRViolation`
- **Tests**: `tests/capability_ir.rs` (22 assertions — CompileTarget, CapabilityRef, all node types, verify all 6 checks, to_cap, to_wit_stub, integration)
- **Validator**: `scripts/validate_capability_ir.py` (16 checks)

### Guard Rail
> WASM Guest = Unit Logik — NO direct hardware access. All hardware access through Capability Gates → Host Reactor.

### Validation
- Capability IR: 16/16 | Sharded Reactor: 11/11 | Network Reactor: 13/13 | Capability Fabric: 10/10 | Streaming: 6/6 | v1.21: 9/9 | **Total: 65/65 ✅**

---

## [Merged via PR #36] — 2026-05-25 — v1.34.0-alpha: Sharded Deterministic Reactor

### Added
- **ShardTopology** (`src/tier2/shard.rs`): Compile-time service topology sharding
  - `ShardAssignment`: `shard_id + core_id + services + budget_mb + gates` — static mapping oleh kompiler
  - `ShardTopology`: assignments + service graph + cross-shard doors + 5 verify checks
  - `ShardVerifyResult` + `ShardViolation`: unassigned service, duplicate assignment, forbidden direct cross-shard, budget overflow, empty shard, core conflict
  - `ServiceGraph` + `ServiceNode`: named service nodes with ports, gates, handlers, policies
  - `CommEdge` + `CommType`: Door (SPSC, cross-shard) vs Direct (intra-shard only)
  - `DoorRef`: cross-shard SPSC channel reference — `from_shard → to_shard` with message type + capacity
  - `to_manifest_json()`: JSON serialization untuk visualisasi / audit
- **ShardedReactor** (`src/net/sharded_reactor.rs`): Runtime sharded reactor — `Vec<ShardInstance>` dengan per-core event loops
- **ShardLocalPool** (`src/net/shard_local_pool.rs`): Per-shard memory pool dengan budget tracking
- **CPU Affinity** (`src/net/affinity.rs`): Wrapper untuk `sched_setaffinity` — static mapping service → core
- **Tests**: `tests/sharded_reactor.rs` + `tests/shard_topology.rs` (combined assertions)
- **Validator**: `scripts/validate_v134_sharded_reactor.py` (11 checks)

### Design Principle
> "Shard Isolation: Setiap CPU Core = satu ReactorInstance + LocalPool. Cross-Shard = Door Only."

### Validation
- Sharded Reactor: 11/11 | Network Reactor: 13/13 | Capability Fabric: 10/10 | Streaming: 6/6 | v1.21: 9/9 | **Total: 49/49 ✅**

---

## [Merged via PR #31] — 2026-05-25 — v1.31.0-alpha: Tier 2 — 2-Pass Streaming Engine

### Added
- **Tier 2 Module** (`src/tier2/`): Streaming Semantic Compiler foundation
  - `src/tier2/metadata.rs` — Core data structures for semantic compression
    - `SemanticSummary` (~64 bytes/symbol): compressed semantic essence replacing full AST
    - `MetadataGraph`: persistent lightweight index — name→ID lookup, call graph, actor registry, channel topology
    - `Capability` (u8 bitflags): Pure, IO, Unsafe, Concurrent, Hardware, Diverging — inferred per-function
    - `InlineCost`: Trivial/Small/Medium/Large/Recursive — estimated from statement count + recursion
    - `MemoryReport`: compare metadata vs. AST memory usage, compute compression ratio
  - `src/tier2/pass.rs` — 2-Pass Streaming Engine
    - `pass1_predeclare()`: lightning scan — collects all signatures, builds call graph, detects mutual recursion
    - `pass2_streaming()`: deep analysis per function — infers capabilities, estimates inline cost
    - `compile_streaming()`: full pipeline — Pass 1 → Pass 2 → StreamingResult
  - `CompileMode`: `Pantas` (aggressive streaming) / `Pakar { max_ram_mb }` (adaptive window)
- **Tests**: `tests/streaming_pass_engine.rs` (12 assertions)
- **Validator**: `scripts/validate_streaming_pass.py` (6 checks)

### Validation
- Streaming Engine: 6/6 | Phase 3: 10/10 | Phase 2: 6/6 | Phase 1: 8/8 | v1.21: 9/9 | **Total: 41/41 ✅**

---

## [Merged via PR #30] — 2026-05-25 — v1.30.1-alpha Phase 3: Backpressure + Scheduler

### Added
- **AST** (`src/ast.rs`): `Expr::TrySend`, `Expr::TryRecv`, `Expr::Yield`, `Expr::Sleep`, `Expr::TimeoutRecv`
- **Lexer** (`src/lexer.rs`): `TokenKind::TrySend`, `TryRecv`, `Yield`, `Sleep` + default aliases
- **Parser** (`src/parser.rs`): `channel.try_send(v)`, `channel.try_recv()`, `yield()`, `sleep(ms)`, `channel.timeout_recv(ms)`
- **Semantic** (`src/semantic.rs`):
  - `ChannelFull { name }` error — backpressure when channel buffer is full
  - `RecvTimeout { name, timeout_ms }` error — recv exceeded timeout
  - Type checking for all Phase 3 expressions (ownership transfer on try_send, numeric duration validation)
- **Codegen** (`src/codegen.rs`): Phase 3 stubs with backpressure-aware + scheduler comments
- **Native Library** (`lib/core/ring_buffer.ldx`): `ring_try_send` (Result<bool, IoError>), `ring_try_recv` (Option<T>), `ring_timeout_recv` (Result<T, RecvTimeout>)
- **Native Library** (`lib/core/scheduler.ldx`): Cooperative scheduler with round-robin — `sched_new`, `sched_register`, `sched_unregister`, `sched_next_actor`, `sched_all_done`, `sched_run`, `sched_yield_threshold`
- **Tests**: `tests/threading_fasa3.rs` (14 assertions)
- **Validator**: `scripts/validate_threading_fasa3.py` (10 checks)

### Validation
- Phase 3 Backpressure: 10/10 | Phase 2 Ownership: 6/6 | Phase 1 Threading: 8/8 | v1.21: 9/9 | **Total: 33/33 ✅**

## [Merged via PR #29] — 2026-05-25 — BREAKING CHANGE: Malay Syntax → English for International Acceptance

### Changed
All threading syntax keywords renamed from Malay to English for international standards compliance.

| Malay (old) | English (new) | Purpose |
|---|---|---|
| `kotak` | `actor` | Concurrency unit (actor-model) |
| `pintu` | `channel` | SPSC communication channel |
| `lahirkan` | `spawn` | Create actor instance |
| `hantar` | `send` | Send value through channel |
| `terima` | `recv` | Receive value from channel |
| `tunggu` | `join` | Wait for actor completion |

### Internal Renames
- `Stmt::Kotak` → `Stmt::Actor`, `Type::Pintu` → `Type::Channel`, `Expr::Hantar` → `Expr::Send`, `Expr::Terima` → `Expr::Recv`, `Expr::Tunggu` → `Expr::Join`
- `TokenKind::{Kotak,Pintu,Lahirkan,Hantar,Terima,Tunggu}` → English equivalents
- `kotak_registry` → `actor_registry`, `pintu_registry` → `channel_registry`, `moved_via_pintu` → `moved_via_channel`
- `UseAfterHantar` → `UseAfterSend`, `KotakNotFound` → `ActorNotFound`, `InvalidPintuTopology` → `InvalidChannelTopology`, `DuplicateKotak` → `DuplicateActor`, `SpawnNonKotak` → `SpawnNonActor`

### Files Modified (12 files, ~875 lines)
`src/ast.rs`, `src/lexer.rs`, `src/parser.rs`, `src/semantic.rs`, `src/codegen.rs`, `tests/threading_foundation.rs`, `tests/threading_fasa2.rs`, `lib/core/ring_buffer.ldx`, `lib/core/thread.ldx`, `scripts/validate_threading_foundation.py`, `scripts/validate_threading_fasa2.py`

### Validation
- Threading Foundation: 8/8 ✅ | Phase 2 Ownership: 6/6 ✅ | v1.21: 9/9 ✅

## [Merged via PR #28] — 2026-05-25 — v1.30.1-alpha Phase 2: Zero-Copy Ownership Transfer

### Added
- **Semantic** (`src/semantic.rs`): Zero-copy ownership transfer via Pintu `hantar()`
  - `moved_via_pintu: HashSet<String>` — tracks variables moved through Pintu
  - `UseAfterHantar { name }` error — bilingual Malay/English diagnostic
  - Move triggered only on `hantar(variable)`, not on `hantar(literal)` or `hantar(expr)`
  - Double-hantar same variable → compile-time error
- **Codegen** (`src/codegen.rs`): `emit_hantar` / `emit_terima` stubs
  - `emit_hantar`: Release semantics for zero-copy send (runtime: `pintu_send_release`)
  - `emit_terima`: Acquire semantics for zero-copy receive (runtime: `pintu_recv_acquire`)
  - `Spawn`/`Tunggu` expression stubs (completes Fasa 1 coverage)
- **Native Library** (`lib/core/ring_buffer.ldx`): SPSC ring buffer with memory ordering
  - `ring_baru<T>(kapasiti)` — allocates power-of-2 ring buffer
  - `ring_hantar<T>()` — Producer write with **Release** tail update
  - `ring_terima<T>()` — Consumer read with **Acquire** head read
  - `ring_kosong()`, `ring_penuh()`, `ring_saiz()` — utility queries
- **Tests**: `tests/threading_fasa2.rs` (12 assertions)
- **Validator**: `scripts/validate_threading_fasa2.py` (6 checks)

### Validation
- v1.21: 9/9 | Sprint 1.1: 32/32 | Sprint 1.2: 20/20 | Sprint 2: 34/34 | Sprint 2.5: 25/25 | Sprint 3: 28/28 | Demo: 11/11 | K1: 17/17 | K2: 9/9 | K3+K4: 12/12 | Audio: 14/14 | **F1 Threading: 12/12 ✅ | F2 Ownership: 12/12 ✅**

## [Merged via PR #27] — 2026-05-25 — v1.30.1-alpha Fasa 1: Threading Foundation — Kotak & Pintu

### Added
- **AST** (`src/ast.rs`): Actor-model concurrency types and expressions
  - `Type::Pintu { from, to, message_type }` — typed SPSC channel
  - `Stmt::Kotak { name, body }` — actor definition (1 OS thread)
  - `Expr::Spawn { kotak_name, args }` — spawn actor (lahirkan)
  - `Expr::Hantar { pintu_name, value }` — send through Pintu
  - `Expr::Terima { pintu_name }` — receive from Pintu
  - `Expr::Tunggu { kotak_name }` — wait for actor (tunggu)
  - `is_pintu()`, `pintu_capability()` helpers
- **Lexer** (`src/lexer.rs`): `Kotak`, `Pintu`, `Lahirkan`, `Hantar`, `Terima`, `Tunggu` tokens
- **Parser** (`src/parser.rs`): `kotak N { ... }`, `Pintu<F, T, M>`, `lahirkan N()`, `pintu.hantar(v)`, `pintu.terima()`, `tunggu N`
- **Semantic** (`src/semantic.rs`): Topology validation
  - `KotakNotFound` — spawn of non-existent Kotak
  - `DuplicateKotak` — duplicate actor definition
  - `InvalidPintuTopology` — Pintu endpoint mismatch
  - `SpawnNonKotak` — spawn on non-Kotak name
  - `kotak_registry: HashSet<String>`, `pintu_registry: Vec<(String, String, String)>`
- **Native Library** (`lib/core/thread.ldx`): Kotak & Pintu documentation, usage patterns, topology examples
- **Native Library** (`lib/core/sync.ldx`): `Mutex`, `RwLock`, `AtomicI32` synchronization primitives
- **Tests**: `tests/threading_foundation.rs` (12 assertions)
- **Validator**: `scripts/validate_threading_foundation.py` (8 checks)

### Validation
- v1.21: 9/9 | Sprint 1.1: 32/32 | Sprint 1.2: 20/20 | Sprint 2: 34/34 | Sprint 2.5: 25/25 | Sprint 3: 28/28 | Demo: 11/11 | K1: 17/17 | K2: 9/9 | K3+K4: 12/12 | Audio: 14/14 | **F1 Threading: 12/12 ✅**

## [Merged via PR #26] — 2026-05-24 — Ketuk 3 + 4: File Handle ABI & Syscall Backend

### Added
- **AST** (`src/ast.rs`): `Type::Opaque { name }` — opaque handle type
  - `Expr::MethodCall { object, method, args }` — `h.read(1024)` syntax
  - `is_opaque()`, `is_file_handle()` helpers
- **Lexer** (`src/lexer.rs`): `FileHandle`, `Close`, `Read`, `Write`, `Seek`, `IsOpen` tokens
- **Parser** (`src/parser.rs`): `FileHandle` type, `h.read()` / `h.close()` / `h.write()` / `h.seek()` method calls
- **Semantic** (`src/semantic.rs`): File handle lifecycle validation
  - `HandleNotOpen` — operation on closed handle
  - `HandlePermissionDenied` — unauthorized access
  - `handle_permissions: HashMap<String, FilePermission>`
- **Native Library** (`lib/core/file.ldx`): `open`, `close`, `read`, `write`, `seek` with bilingual docs
- **Syscall Backend** (`src/os/syscall.rs`): Linux x86_64 direct syscall
  - `SYS_OPEN`, `SYS_CLOSE`, `SYS_READ`, `SYS_WRITE`, `SYS_LSEEK`, `SYS_MMAP` constants
  - `emit_file_syscall()` — generates `syscall` instruction inline
- **Runtime** (`lib/runtime/io_syscalls.ldx`): Runtime syscall wrappers
- **Tests**: `tests/io_file_syscall.rs` (12 assertions)
- **Validator**: `scripts/validate_io_file_syscall.py` (10 checks)

### Validation
- v1.21: 9/9 | Sprint 1.1: 32/32 | Sprint 1.2: 20/20 | Sprint 2: 34/34 | Sprint 2.5: 25/25 | Sprint 3: 28/28 | Demo: 11/11 | K1: 17/17 | K2: 9/9 | **K3+K4 File Syscall: 12/12 ✅**

## [Merged via PR #25] — 2026-05-24 — Ketuk 2: Result<T, E> Abstraction — Ok/Err, match, IO Guard

### Added
- **AST** (`src/ast.rs`): `Type::Result { ok, err }`, `Expr::Ok { value }`, `Expr::Err { value }`
  `Stmt::Match { value, arms }`, `MatchArm { pattern, body }`
  `MatchPattern::Ok { binding }`, `MatchPattern::Err { binding }`, `MatchPattern::Wildcard`
- **Lexer** (`src/lexer.rs`): `Result`, `Ok`, `Err`, `Match`, `ArrowFat` (=>), `Underscore` (_) tokens
- **Parser** (`src/parser.rs`): `Result<T, E>` type syntax, `Ok()`/`Err()` constructors
  `match expr { Ok(v) => body, Err(e) => body }` statement + arm parsing
- **Semantic** (`src/semantic.rs`): Match exhaustiveness validation
  `MatchOnNonResult` error — match on non-Result type
  `NonExhaustiveMatch` error — missing Ok or Err arm
- **Native Library** (`lib/core/result.ldx`): `unwrap_or`, `expect`, `is_ok`, `is_err`, `map`
- **Native Library** (`lib/core/io_error.ldx`): `IoError` enum — `FileNotFound`, `PermissionDenied`, `InvalidPath`, `BufferTooSmall`, `DiskFull`, `Unknown`
- **Tests**: `tests/result_abstraction.rs` (9 assertions)
- **Validator**: `scripts/validate_result_abstraction.py` (8 checks)

### Validation
- v1.21: 9/9 | Sprint 1.1: 32/32 | Sprint 1.2: 20/20 | Sprint 2: 34/34 | Sprint 2.5: 25/25 | Sprint 3: 28/28 | Demo: 11/11 | K1: 17/17 | **K2 Result: 9/9 ✅**

## [Merged via PR #24] — 2026-05-24 — Fix: 5 Critical Bugs in Buffer Overflow & Use-After-Move

### Fixed
- **BUG #1 CRITICAL**: `Stmt::Let` tak register buffer ke `buffer_registry` → `register_buffer()` call semasa Let process `Buffer<T>`
- **BUG #2 CRITICAL**: Parser tak support `buf[index] = value` → `peek_index_assignment()` + `index_assignment_statement()`
- **BUG #2b CRITICAL**: `Stmt::Assign` tak handled dalam semantic analyzer → Full Assign handling dengan Index target validation + provenance check
- **BUG #3 HIGH**: `moved_vars` tak clear bila scope keluar → `scoped_block()` cleanup `moved_vars` + `buffer_registry`
- **BUG #4 MEDIUM**: `mark_moved` tak pernah dipanggil → Let detect ownership transfer (`let buf2 = buf`)
- **BUG #5 LOW**: Error misleading untuk unregistered buffer → `NotABuffer` error variant (Malay + English)

### Added
- `Buffer<f32, 1024>` capacity syntax dalam parser
- `tests/buffer_provenance_bugfixes.rs` — 9 assertions
- `scripts/validate_buffer_bugfixes.py` — 9 checks

### Validation
- v1.21: 9/9 | Sprint 1.1: 32/32 | Sprint 1.2: 20/20 | Sprint 2: 34/34 | Sprint 2.5: 25/25 | Sprint 3: 28/28 | Demo: 11/11 | K1: 17/17 | **Bug Fixes: 9/9 ✅**

## [Merged via PR #23] — 2026-05-24 — Ketuk 1: Core Memory Model (Slice, Buffer, Ownership & Provenance)

### Added
- **AST** (`src/ast.rs`): `Type::Slice { element }`, `Type::Buffer { element }`, `Expr::Index { base, index }`
  - `is_slice()`, `is_buffer()`, `is_contiguous()`, `element_type()` helpers
- **Lexer** (`src/lexer.rs`): `LeftBracket`, `RightBracket`, `Buffer` tokens
- **Parser** (`src/parser.rs`): `[]T` slice type, `Buffer<T>` buffer type, `buf[index]` indexing
- **Semantic** (`src/semantic.rs`): Buffer provenance + ownership tracking
  - `BufferOverflow { name, index, capacity }` — compile-time bounds check
  - `UseAfterMove { name }` — ownership violation detection
  - `ElementTypeMismatch { elem, expected, actual }`
  - `validate_buffer_index()`, `register_buffer()`, `mark_moved()`, `is_moved()`
- **Native Library** (`lib/core/memori.ldx`): `panjang`, `kapasiti`, `kosongkan`, `salin`, `isi`, `sub`
- **Tests**: `tests/core_memory_model.rs` (17 assertions)
- **Validator**: `scripts/validate_core_memory.py` (7 checks)

### Validation
- v1.21: 9/9 | Sprint 1.1: 32/32 | Sprint 1.2: 20/20 | Sprint 2: 34/34 | Sprint 2.5: 25/25 | Sprint 3: 28/28 | Demo: 11/11 | **K1 Core Memory: 17/17 ✅**

## [Merged via PR #22] — 2026-05-24 — Audio Engine: Hardware-Safe Audio Guards with Function Pointers

### Added
- **AST** (`src/ast.rs`): `Type::FunctionPointer { params, return_type }` — function pointer type
  - `is_function_pointer()` — check if type is a function pointer
  - `is_audio_callback_fp()` — detect audio ISR signature `fn(*mut f32, i32)`
- **Parser** (`src/parser.rs`): `parse_type()` handles `fn(params) -> ret` syntax
- **StrictAudioContext** (`src/semantic.rs`): Hardware-safe audio callback verification
  - `verify_audio_safety()` — walks function body, validates all statements/expressions
  - `AudioViolationIo` — rejects `Print`, `DrawText`, `InitWindow` in callbacks
  - `AudioViolationRecursion` — rejects self-calling in audio ISR
  - `AudioViolationUnboundedLoop` — rejects `loop { }` (watchdog risk)
  - `AudioViolationForbiddenCall` — rejects unsafe function calls
  - `mark_audio_callback_if_applicable()` — detects `SetAudioStreamCallback(func)`
- **Native Library** (`lib/std/audio.ldx`): `tulis_selamat()` hardware clipper clamping `[-1.0, 1.0]`, `kepit()`, `gelombang_sinus()`
- **Demo** (`examples/audio_sine.ldx`): 72-line function pointer callback demo
- **Tests**: `tests/audio_engine_hardware_safe.rs` (14 assertions)
- **Validator**: `scripts/validate_audio_engine.py` (8 checks)

### Validation
- v1.21: 9/9 | Sprint 1.1: 32/32 | Sprint 1.2: 20/20 | Sprint 2: 34/34 | Sprint 2.5: 25/25 | Sprint 3: 28/28 | Demo: 11/11 | **Audio: 14/14 ✅**

## [Merged via PR #21] — 2026-05-24 — Demo: Raylib Spinning Box (compile-ready example)

### Added
- **`examples/raylib_spinning_box.ldx`** — 53-line interactive demo program:
  - 6x `Color(r, g, b, a)` struct constructors (packed RGBA)
  - Raylib FFI calls: `InitWindow`, `DrawRectangle`, `DrawText`, `ClearBackground`, `BeginDrawing`, `EndDrawing`
  - Input handling: `IsMouseButtonPressed(0)`, `IsKeyPressed(KEY_SPACE)`
  - Game loop: `while (!WindowShouldClose())` with `break`
  - `unsafe { ... }` FFI safety gate
- **Integration test** (`tests/demo_raylib_spinning_box.rs`): 11 assertions:
  - Parser: all `Color(...)` recognized as `Expr::Call` with 4 args
  - TypeChecker: validates all 6 color constructors
  - CallableRegistry: all 12 Raylib functions used are registered with correct signatures
  - HIR lowering: demo program lowers to `HirModule` without errors
  - Color packing: `Color(255,0,0,255)` → `0xFF0000FF`
- **Validator** (`scripts/validate_demo_raylib_box.py`): 4 checks PASSED

### Compile
```bash
logicodex --pipeline v1.30 examples/raylib_spinning_box.ldx -o spinning_box
```

### Validation
- v1.21 executable logic: 9/9 checks PASSED
- Sprint 1.1: 32/32 checks PASSED
- Sprint 1.2: 20/20 checks PASSED
- Sprint 2: 34/34 checks PASSED
- Sprint 2.5: 25/25 checks PASSED
- Sprint 3: 28/28 checks PASSED
- **Demo Spinning Box: 11/11 assertions PASSED**

## [Merged via PR #20] — 2026-05-24 — Sprint 3: Codegen Backend for Function Calls

### Added
- **LlvmCompiler CallableRegistry integration** (`src/codegen.rs`): `with_callables()` attaches `CallableRegistry` + `TypeRegistry` for function call codegen
- **TypeId → LLVM mapping** (`src/codegen.rs`): `type_id_to_llvm()` maps all `PrimitiveType` variants → `inkwell::BasicTypeEnum`
- **LLVM extern function declaration** (`src/codegen.rs`): `declare_extern_func()` creates LLVM function declarations with `Linkage::External` and caching
- **Function call codegen** (`src/codegen.rs`): `emit_expr(Expr::Call)` — CallableRegistry lookup → declare → `builder.build_call()` → extract return value
- **Struct constructor codegen** (`src/codegen.rs`): `try_struct_constructor()` — `Color(255,0,0,255)` → packed u32 `0xFF0000FF`
- **v1.21 → HIR lowering** (`src/hir.rs`): `lower_v121_program()` converts `ast::Program` → `HirModule` with callable registration
- **AST conversion helpers** (`src/hir.rs`): `lower_type_ast`, `lower_stmt_ast`, `lower_expr_ast`, `lower_binary_op` — v1.21 AST → HIR AST bridge
- **V130 compile pipeline** (`src/main.rs`): `compile_v130_pipeline()` — parse → Raylib type/function registration → HIR lowering → semantic check → `compile_v130()`
- **Tests** (`tests/codegen_function_calls.rs`): 28 assertions — CallableRegistry, type mapping, Raylib registration, HIR lowering, Color packing
- **Validator** (`scripts/validate_sprint3_codegen_calls.py`): 28/28 checks PASSED

### Changed
- `compile_v130()`: Updated signature to accept `(CallableRegistry, TypeRegistry)`
- `compile()`: Branches on `CompilerPipeline::V130` → `compile_v130_pipeline()` vs `V121` → `compile_to_object()`

### Validation
- v1.21 executable logic: 9/9 checks PASSED
- Sprint 1.1: 32/32 checks PASSED
- Sprint 1.2: 20/20 checks PASSED
- Sprint 2: 34/34 checks PASSED
- Sprint 2.5: 25/25 checks PASSED
- Sprint 3: 28/28 checks PASSED

## [Merged via PR #19] — 2026-05-24 — Sprint 2.5: Struct Literals & Function Call Parser

### Added
- **Expr::Call** (`src/ast.rs`): New AST variant `Call { callee: Box<Expr>, args: Vec<Expr> }` for struct constructors and function calls
- **Parser call detection** (`src/parser.rs`): `primary()` detects `Identifier(` → parses as `Expr::Call` with comma-separated argument list
- **HIR Call lowering** (`src/semantic.rs`): `ExprAst::Call` → `HirExprKind::Call` with Sprint 3 codegen placeholder
- **TypeChecker::check_call()** (`src/semantic/type_checker.rs`): Validates struct constructor argument count against registered `StructLayout` fields
- **Tests** (`tests/parser_struct_literals.rs`): 25 assertions — struct literals `Color(255,0,0,255)`, nested constructors, function calls `print("hello")`, error cases
- **Validator** (`scripts/validate_sprint2_5_struct_literals.py`): 25/25 checks PASSED

### Architecture Notes
- `check_call()` returns `Type::I64` placeholder — full struct TypeId resolution deferred to Sprint 3 (LLVM struct value emission)
- Complex callees (e.g., `obj.method()`) return descriptive error — deferred to Sprint 3

### Validation
- v1.21 executable logic: 9/9 checks PASSED
- Sprint 1.1: 32/32 checks PASSED
- Sprint 1.2: 20/20 checks PASSED
- Sprint 2: 34/34 checks PASSED
- Sprint 2.5: 25/25 checks PASSED

## [Merged via PR #18] — 2026-05-24 — Sprint 2: LayoutEngine

### Added
- **Struct layout types** (`src/types.rs`): `StructLayout`, `StructFieldLayout` moved from `layout.rs`
- **TypeRegistry struct cache**: `struct_layouts: Vec<StructLayout>`, `intern_struct()`, `get_struct_layout()`, `find_struct_by_name()`
- **get_size/get_align for Struct**: Uses cached layout instead of panic
- **LayoutEngine struct lookup** (`src/layout.rs`): `size_and_align` resolves Struct via cache
- **Raylib struct types** (`src/ffi/raylib.rs`): `register_raylib_types()` registers Color(4B), Vector2(8B), Rectangle(16B), Texture2D(20B)
- **Tests** (`tests/layout_engine_integration.rs`): 29 assertions — layout, cache, Raylib types, nested structs

### Changed
- `src/layout.rs`: Import `StructLayout`/`StructFieldLayout` from `types.rs` (not local)

### Validation
- v1.21 executable logic: 9/9 checks PASSED
- Sprint 1.1: 32/32 checks PASSED
- Sprint 1.2: 20/20 checks PASSED
- Sprint 2: 34/34 checks PASSED

## [Merged via PR #17] — 2026-05-24 — Stage 1 Quickfix: Raylib Color + Math

### Fixed
- **Color registration mismatch**: 7 drawing functions registered Color as `I64`, now `U32` (packed RGBA `0xRRGGBBAA`)
- ClearBackground, DrawText, DrawRectangle, DrawCircle, DrawLine, DrawRectangleLines, DrawPixel

### Added
- **Math utilities** (`src/ffi/math.rs`): `clamp()`, `lerp()`, `remap()`, `normalize()`, `float_equals()`, `float_zero()`
- **Integration test** (`tests/ffi_stage1_integration.rs`): 25 assertions validating full user story

### Validation
- v1.21 executable logic: 9/9 checks PASSED
- Sprint 1.1 structural: 32/32 checks PASSED
- Sprint 1.2 structural: 20/20 checks PASSED

## [Merged via PR #16] — 2026-05-24 — Sprint 1 Bugfixes (8 critical fixes)

### Fixed
- **CRITICAL #1**: `ast_type_to_id` returned invalid `TypeId(1000+)` — would panic on `resolve()`
- **CRITICAL #2**: Circular dependency `types.rs ↔ semantic/coercion.rs` broke compilation
- **CRITICAL #3**: `use` statement inside method body — invalid Rust syntax
- **HIGH #4**: `coercion.rs` test `setup()` — self-referential lifetime issue
- **HIGH #5**: `type_checker.rs` test — same self-referential pattern
- **MEDIUM #6**: `infer_default_type` returned `I64` for all complex expressions → now `Option<Type>`
- **MEDIUM #7**: `c_void_ptr`/`c_const_char_ptr` needed `&mut self` — inconsistent API → added `void_ptr()`/`const_char_ptr()` with `&self`
- **LOW #8**: `explain_incompatibility` only covered Bool/String → expanded to all cases

### Changed
- `src/types.rs`: Removed AST bridge (→ TypeChecker), added `&self` pointer accessors
- `src/semantic/type_checker.rs`: Added bridge functions, fixed lifetimes, `Option<Type>` inference
- `src/semantic/coercion.rs`: Fixed test helper lifetime

### Validation
- v1.21 executable logic: 9/9 checks PASSED
- Sprint 1.1 structural: 32/32 checks PASSED
- Sprint 1.2 structural: 20/20 checks PASSED

## [Merged via PR #15] — 2026-05-24 — Sprint 1.2: Parser Type Injection

### Added
- **TypeChecker** (`src/semantic/type_checker.rs`):
  - `check_assignment(declared, actual) -> TypeCheckResult` — uses CoercionEngine
  - `TypeCheckResult` enum: `Ok`, `ImplicitWidening`, `RequiresExplicitCast`, `Incompatible`
  - `infer_default_type(Expr) -> Type` — I64 (int), F64 (float), String, Bool
  - `format_error()` — bilingual Malay/English diagnostics with cast suggestions
- **AST Type Bridge** (`src/types.rs`):
  - `ast_type_to_id()` — converts `ast::Type` to `TypeId`
  - `type_id_to_ast()` — converts `TypeId` back to `ast::Type`
  - `ast_types_compatible()` — CoercionEngine-based compatibility check
- **Tests** (`tests/parser_type_test.rs`): 25 assertions

### Validation
- v1.21 executable logic: 9/9 checks PASSED
- Sprint 1.1 structural: 32/32 checks PASSED
- Sprint 1.2 structural: 20/20 checks PASSED

## [Merged via PR #14] — 2026-05-24 — Sprint 1.1: Type System Foundation

### Added
- **TypeRegistry enhancements** (`src/types.rs`):
  - `get_size(TypeId) -> usize` — deterministic byte sizes (I32=4, I64=8, Ptr=8)
  - `get_align(TypeId) -> usize` — C ABI alignment
  - `resolve(TypeId) -> &TypeKind` — infallible lookup
  - `c_abi_info(TypeId) -> CAbiInfo` — combined size+align for FFI
  - FFI type aliases: `c_int()`, `c_double()`, `c_void_ptr()`, `c_const_char_ptr()`
- **TypeInspector** (`src/semantic/registry.rs`): High-level type queries
  - `is_integer`, `is_float`, `is_numeric`, `is_pointer`, `is_bool`
  - `type_name()` for diagnostic messages
  - `validate_ffi_type()` for FFI boundary checks
  - `is_lossless_conversion()` for widening checks
- **CoercionEngine** (`src/semantic/coercion.rs`): Full coercion matrix
  - `CoercionResult` enum: `Identity`, `Implicit`, `RequiresCast`, `Incompatible`
  - `can_coerce(from, to)` — complete coercion rules
  - `common_type(left, right)` — binary operation type inference
  - Widening: I32→I64, I32→F64, F32→F64, String→*const I8 (implicit)
  - Narrowing: I64→I32, F64→I32 (requires explicit cast)
- **Raylib FFI** (`src/ffi/raylib_sys.rs` + `src/ffi/raylib.rs`):
  - Raw `extern "C"` declarations for 20 core functions
  - C types: `Color` (4 bytes), `Vector2` (8 bytes), `Texture2D` (20 bytes)
  - Safe wrapper layer with null-checks
  - `CallableRegistry` integration (28 functions, all `UnsafeRequired`)
  - Coverage: windowing, drawing, textures, input
- **Library target** (`src/lib.rs` + `Cargo.toml`): `[lib]` section for integration tests
- **Tests** (`tests/type_registry_test.rs` + `tests/raylib_ffi_test.rs`):
  - 38 assertions covering sizes, alignment, idempotency, FFI, coercion, layouts
- **Validator** (`scripts/validate_sprint1_type_registry.py`): 32-check structural validator

### Validation
- v1.21 executable logic: 9/9 checks PASSED
- Sprint 1.1 structural: 32/32 checks PASSED

## [Merged via PR #12] — 2026-05-24 — Version Gate Integration (v1.30 Pipeline)

### Added — Edition Routing / Version Gate Architecture
- **New CLI flag**: `--pipeline <v1.21|v1.30>` on both `compile` and `check` commands.
  - Default: `v1.21` (stable, backward-compatible).
  - Opt-in: `v1.30` activates experimental parsing for advanced constructs.
- **New `CompilerPipeline` enum** in `src/parser.rs` with `FromStr` implementation for clean CLI parsing.
- **Parser pipeline gating**: `Parser::with_pipeline()` allows per-instance pipeline selection.
  - `v1.21` pipeline: tokens like `struct`, `enum`, `unsafe`, `extern` are trapped with `UnimplementedFeature` error.
  - `v1.30` pipeline: these tokens are parsed into proper AST nodes (`StructDecl`, `EnumDecl`, `UnsafeBlock`, `ExternBlock`).
- **New AST variants** in `src/ast.rs`:
  - `Stmt::StructDecl { name, fields }` — structure type declarations.
  - `Stmt::EnumDecl { name, variants }` — enumeration type declarations.
  - `Stmt::UnsafeBlock { body }` — unsafe code blocks.
  - `Stmt::ExternBlock { abi, functions }` — foreign function interface blocks.
  - `ExternFnDecl` struct for individual extern function signatures.
- **HIR enhancements** in `src/hir.rs`:
  - Added `StmtAst::If` and `HirStmt::If` with condition, then-branch, and optional else-branch.
  - Added `LoweringContext::types` field carrying `TypeRegistry` reference.
  - Replaced hardcoded `TypeId` values with `TypeRegistry::primitive()` lookups.
  - Fixed `AddressOf` to use proper pointer type interning via `TypeKind::Pointer`.
  - Fixed `ExternBlock` lowering bug — now correctly processes all extern functions instead of only the last one.
- **Codegen safety net** in `src/codegen.rs`:
  - `LlvmCompiler::emit_v130_ast_in_v21()` emits `unreachable!()` panic with informative message if v1.30 AST nodes leak into v1.21 codegen.
  - `compile_v130()` entry point for v1.30 HIR-to-object compilation.
  - `CodegenBackend` trait defining the contract for version-gated codegen backends.
- **Semantic gate update** in `src/semantic_gate.rs`:
  - `check_statement()` now handles `HirStmt::If` with proper scope management for both branches.

### Changed
- **Parser `declaration_or_statement()`**: Replaced monolithic v1.21 trap with pipeline-dispatched `match` arms. Cleaner, faster, and enables LLVM jump-table optimization.
- **`named_type_id()`**: Now takes `&TypeRegistry` parameter instead of returning hardcoded `TypeId` values. Eliminates fragile numeric constants.
- **`LoweringContext` construction**: Now requires both `symbols: &mut SymbolTable` and `types: &mut TypeRegistry`.

### Fixed
- **ExternBlock lowering bug** (`src/hir.rs`): Previously only the last extern function in a block was preserved. Now all functions are correctly lowered.
- **AddressOf type bug** (`src/hir.rs`): Previously hardcoded `TypeId(15)` for all pointer types. Now each pointer gets a unique `TypeId` via proper type interning.

### Security / Defense-in-Depth
- **Fail-fast codegen**: v1.21 codegen will panic with a descriptive message (via `unreachable!()`) if it receives v1.30-only AST nodes. This prevents silent corruption and makes pipeline misconfigurations immediately visible.

### Zero Regression Guarantee
- **Default pipeline**: `v1.21` (backward-compatible, no behavior change).
- v1.21 code paths are **untouched**.
- v1.21 does **not** pass through HIR lowering.
- Fail-fast `unreachable!()` safety nets prevent silent pipeline leaks.

### Validation
- All 9 `validate_v121_executable_logic.py` checks pass:
  - AST supports executable v1.21-alpha declarations ✅
  - Lexer exposes canonical v1.21-alpha tokens ✅
  - Parser enforces executable grammar layout ✅
  - Semantic analyzer implements static safety checks ✅
  - Code generator accepts expanded AST ✅
  - CLI wires target and secure flags ✅
  - Dictionary token surface ✅
  - Version-label policy ✅
  - Known regression guards ✅

---

## [1.21.0-alpha] — 2026-05-XX

### Added
- Initial v1.21-alpha compiler core with LLVM backend.
- Malay/English bilingual alias system via `dict/core_map.json`.
- Hardware-zone provenance gates (`ZON_PERKAKASAN` / `hw_unsafe`).
- Reflex-engine example suite covering arithmetic, functions, loops, bitwise operations, hardware-zone provenance, and Boolean conditionals.
- Three-tier error severity classification (Critical / Medium / Low).
- Dormant v1.30.0-alpha subsystem with HIR, layout engine, semantic gate, and codegen contracts.

---

*For older releases, see the Git history.*
