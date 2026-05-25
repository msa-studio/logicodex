# Logicodex Language — v1.38.0-alpha
## The Deferred Items Cleanup

> v1.21 Compiler Baseline → v1.30 Threading + IO + Audio → v1.31 Streaming Engine → v1.32 Capability Fabric → v1.33 Network Reactor → v1.34 Sharded Reactor → v1.35 Capability IR → v1.36 CTL Mapper → v1.37 Network Runtime → **v1.38 Deferred Cleanup**

The **current logicodex v1.38 alpha** closes 8 long-standing deferred items: CallableRegistry integration (A6), CapabilityTopology fix (D1), struct/enum type resolution (E1-E2), Windows syscall fallback (F1), memory attestation (G1), freestanding target (G2), and semantic gatekeeper activation (I1). **20 of 26 deferred items are now resolved.**

> v1.21 Compiler Baseline → v1.30 Threading + IO + Audio → v1.31 Streaming Engine → v1.32 Capability Fabric → v1.33 Network Reactor → v1.34 Sharded Reactor → v1.35 Capability IR → v1.36 CTL Mapper → **v1.37 Network Runtime**

The **current logicodex v1.37 alpha** milestone implements the **Deterministic Network Runtime** — transforming the compile-time network reactor (v1.33) into a fully operational runtime with live epoll event loops, syscall-based socket I/O, taint-state FSM, and backpressure policies. This closes the gap between compile-time verification and runtime execution.

The **current logicodex v1.36 alpha** milestone completes the **Capability Translation Layer (CTL)** — a unified IR that projects Logicodex's capability-native world INTO the WASM ecosystem. It combines deterministic concurrency, streaming compilation, capability-based security, sharded event-driven networking, and now **WIT auto-generation** — all verified at compile time with **zero runtime mediation**.

The **current logicodex v 1.21 alpha** milestone establishes a practical compiler-core baseline and a documented security research direction. It includes a four-layer grammar baseline, an Undefined Behavior and Pointer Provenance design note, and a Critical/Medium/Low severity taxonomy. The stronger security, freestanding, and measured-overhead goals are treated as **long-term engineering objectives** until they are implemented, benchmarked, and validated by repeatable tests.

```text
=========================================================
  _                 _               _                 
 | |    ___   __ _ (_)  ___  ___   __| |  ___ __  __  
 | |   / _ \ / _` || | / __|/ _ \ / _` | / _ \ \/ /  
 | |__| (_) | (_| || || (__| (_) | (_| ||  __/ >  <   
 |_____\___/ \__, ||_| \___|\___/ \__,_| \___|/_/\_\  
             |___/                                    
             [ LOGICODEX COMPILER v1.21-alpha ]
             [ PRACTICAL LLVM COMPILER BASELINE ]
=========================================================
Architect & Creator: Mohamad Supardi Abdul (mymsastudio@gmail.com)
```

## Executive Summary

Logicodex is an alias-to-canonical, LLVM-backed systems programming language created by **Mohamad Supardi Abdul** (`mymsastudio@gmail.com`). Its practical aim is to reduce the cognitive gap between readable human intent and native-oriented compiler output by allowing Malay/English pseudocode aliases and expert canonical shorthand to flow through one deterministic frontend.

The current **Phase 1** alpha focuses on a working compiler core: the `dict/core_map.json` dictionary loader, lexer, parser, AST construction, semantic analyzer, and LLVM-Inkwell backend path for native-oriented object generation. Roadmap capabilities including the **WebAssembly target**, **Logicodex Migrator Engine**, continuous runtime memory attestation, and deeper freestanding support are **objectives to be built and validated over time**, not completed implementation claims in the current alpha.

## Compiler Pipeline

```text
[ Malay/English Alias Code (.ldx) ] ──► (Lexer + core_map.json) ──► [ Unified Token Stream ]
                                                                           │
[ Expert Canonical Code (.ldx) ] ──► (Lexer + core_map.json) ──► [ Abstract Syntax Tree ]
                                                              │
[ Native-Oriented Object Output ] ◄── (LLVM Backend) ◄── [ LLVM IR Generation ]
```

The dictionary is consumed strictly during lexing. Surface forms such as `MULA`, `BEGIN`, and `{` normalize into canonical token identities such as `TokenKind::Start` before parsing begins. The parser therefore consumes a uniform token stream rather than performing macro rewriting or grammar-level dialect conversion.

## v1.30-v1.36 Capability Overview

Logicodex has evolved from a compiler-core prototype into a **deterministic systems platform with WASM integration** through 7 consecutive alpha releases:

| Release | Focus | Key Innovation |
|---|---|---|
| **v1.30.1-alpha** | Threading + IO + Audio | Actor-model concurrency (`actor`/`channel`), zero-copy ownership transfer, 4-Ketuk IO architecture, hardware-safe audio engine |
| **v1.31.0-alpha** | Streaming Compiler | 2-Pass Engine — RAM stays flat regardless of program size; SemanticSummary (~64B/symbol) replaces full AST |
| **v1.32.0-alpha** | Capability Security | Static Capability Fabric — Gate/Door split, compile-time topology verification, supply-chain `.cap` files, privilege escalation detection |
| **v1.33.0-alpha** | Network Reactor | Deterministic event-driven networking — RAII auto-cleanup (no socket leaks), taint state machine, backpressure policies, service manifest syntax |
| **v1.34.0-alpha** | Sharded Multi-Core Reactor | Per-CPU-core reactor instances, static affinity mapping, cross-shard SPSC doors, memory budgeting |
| **v1.35.0-alpha** | CapabilityGraph IR | Single Source of Truth IR — unifies SemanticSummary + CapabilityTopology + ShardTopology; generates Native/`.cap`/WIT |
| **v1.36.0-alpha** | CTL Mapper | Auto-generates WIT from CapabilityGraph — 6 domain mappings, manual overrides, HW gate host reactor stubs |
| **v1.37.0-alpha** | **Network Runtime** | **Deterministic event loop — epoll, live socket I/O, taint FSM, RAII auto-cleanup, backpressure at runtime** |
| **v1.38.0-alpha** | **Deferred Cleanup** | **CallableRegistry predeclare, topology fix, enum layout, Windows fallback, secure attestation, freestanding target, semantic gatekeeper** |

### Architecture: Door + Gate + Service + IR + CTL

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        APPLICATION LAYER                                 │
│                                                                          │
│    actor Worker {          service WebServer {          ┌───────────┐    │
│        let ch: Channel<...>      port: 443              │ WIT Output│    │
│        ch.send(data)             requires: Net.Admin    │  (WASM)   │    │
│    }                             handler: WebHandler    └───────────┘    │
│                                  policy: Block              ▲            │
│    spawn Worker()            }                              │            │
│                                                             │ CTL Mapper │
├──────────────┬──────────────┬──────────────────┬────────────┴───────────┤
│    DOOR      │     GATE     │     SERVICE      │     CapabilityGraph    │
│  (SPSC Ring  │  (Compile-   │  (Port Actor     │      (THE IR)          │
│   Buffer)    │   time Cap)  │   + Reactor)     │  Single Source of      │
│ Zero-copy    │ Zero Runtime │ RAII Cleanup     │  Truth — unifies       │
│ Lock-free    │  Mediation   │ Taint FSM        │  v1.31+v1.32+v1.34    │
├──────────────┴──────────────┴──────────────────┴────────────────────────┤
│                         COMPILER LAYER                                   │
│                                                                          │
│   Tier 1: Parser (AST) → Tier 2: CapabilityGraph → Tier 3: Codegen     │
│   Full AST           →  Unified IR          →  Native / WASM           │
│   (temporary)          (persistent)            (streamed)               │
│                                                                          │
│   2-Pass Engine + Capability Verification + Sharding                   │
├─────────────────────────────────────────────────────────────────────────┤
│                          RUNTIME LAYER                                   │
│                                                                          │
│   RAII Connection Drop → close(fd) deterministik                        │
│   Taint State Machine  → Healthy → Suspicious → Closing                 │
│   Backpressure Policy  → Block / DropOldest / Error                     │
│   Shard Local Pool     → Per-core memory budgeting                      │
│   Host Reactor         → HW gate mediation (WASM safety)                │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

- **Door** = Data transport (v1.30 Phase 3) — `lib/core/ring_buffer.ldx`
- **Gate** = Security contract (v1.32) — `src/tier2/gate.rs`, `src/tier2/topology.rs`
- **Service** = Event loop + connection (v1.33) — `src/net/*.rs`
- **CapabilityGraph IR** = Unified IR (v1.35) — `src/tier2/capability_ir.rs`
- **CTL Mapper** = WIT generator (v1.36) — `src/tier2/ctl_mapper.rs`

**Validation: 77/77 checks passing + runtime live — zero regression across all versions.**

### Documentation

| Document | Description |
|---|---|
| `docs/ARCHITECTURE.md` | Complete Door+Gate+Service+IR+CTL architecture overview |
| `docs/v1.30-THREADING.md` | Threading Phases 1-3 (Actor/Channel/Backpressure) |
| `docs/v1.31-STREAMING.md` | Tier 2 Streaming Semantic Compiler |
| `docs/v1.32-CAPABILITY.md` | Static Capability Fabric (Gate/Door/Topology) |
| `docs/v1.33-REACTOR.md` | Deterministic Network Reactor (RAII/Service/Taint) |
| `docs/v1.34-SHARDED.md` | Sharded Deterministic Reactor (Per-core/Doors/Budget) |
| `docs/v1.35-CAPABILITY-IR.md` | CapabilityGraph IR — Single Source of Truth |
| `docs/v1.36-CTL-MAPPER.md` | CTL Mapper — WIT Auto-Generation from CapabilityGraph |
| `docs/v1.37-NETWORK-RUNTIME.md` | **Network Runtime — epoll, socket I/O, taint FSM, RAII cleanup** |

---

## Current Capability Boundary

The project should be read as an **alpha systems platform and specification prototype**. It is suitable for compiler-core experimentation, deterministic concurrency design, capability-security architecture, network reactor development, and systems-programming research. The v1.30-v1.33 threading, streaming, capability, and reactor features are implemented and validated, but should be understood as a maturing alpha rather than a hardened production platform.

| Area | Current v1.21-alpha status | Practical next objective |
|---|---|---|
| Alias-to-canonical syntax | Implemented in the Phase 1 frontend through dictionary-aware tokenization. | Expand Malay, English pseudocode, and domain-specific alias families while preserving deterministic builds. |
| Static semantics | Implemented for the Phase 1 core language and selected safety checks. | Tighten type-system boundaries, diagnostics, pointer-provenance rules, and UB catalog coverage. |
| LLVM backend | Implemented through the Rust Inkwell path for core expressions and native-oriented object generation. | Mature target triples, ABI contracts, linker policies, and executable-output checks. |
| Reflex engine examples | Refreshed suite in `examples/` now covers expert and Malay arithmetic, functions, loops, bitwise operations, hardware-zone provenance, and Boolean conditionals. | Add expected-output fixtures and backend parity checks as the next proof layer. |
| WebAssembly target | **Foundation laid via CTL Mapper (v1.36)** — WIT auto-generation from CapabilityGraph with 6 domain mappings. Full `.wasm` codegen remains a medium-term objective. | Complete end-to-end: CapabilityGraph → WIT → WASM component → host reactor integration. |
| Migrator Engine | Long-term roadmap objective. | Start with assisted source translation experiments that require explicit human review. |
| Runtime memory attestation | Long-term security research objective. | Convert the current plan contract into prototype digest insertion, verifier stubs, and target-specific fail-stop behavior. |
| Freestanding support | Alpha target profile and plan generation. | Add linker scripts, bootloader examples, raw-pointer gates, and hardware-region policies before claiming full freestanding readiness. |

## Freestanding and Hosted Safety Boundary

The `--target freestanding` profile is an **experimental direction** for OS-less integration contexts such as kernels, bootloaders, firmware, and hypervisors. Examples that write to physical addresses such as VGA text memory at `0xB8000` are valid only under freestanding execution or equivalent kernel-space mapping authority. Hosted operating systems such as Linux and Windows normally protect processes with virtual memory paging and ASLR, so direct physical address manipulation from user space should be expected to fail through page-fault defenses unless mediated by mechanisms such as `/dev/mem` or Ring-0 drivers.

## Security Roadmap Boundary

The security model is currently best described as a **roadmap and design contract**. The desired long-term behavior is fail-stop handling for executable-code tampering, with hosted environments using normal process termination and freestanding environments using target-appropriate halt or reset mechanisms where such behavior is safe and explicitly configured. The current repository documents and prepares this direction; future milestones must implement verifier code, benchmark overhead, document threat models, and test target-specific mitigation behavior before stronger production security claims are made.

## Stable Rust 1.75 Build Compatibility

The **v1.21-alpha** repository is pinned to Rust Edition 2021 and is intended to resolve and compile under stable Rust **1.75.0 through 1.78.0** without requiring Edition 2024 dependency features. The repository-level `rust-toolchain.toml` pins the canonical validation channel to `1.75.0`; newer stable toolchains in the supported range may be used after confirming that the generated `Cargo.lock` remains compatible with the same dependency floor.

| Component | Compatibility setting | Reason |
|---|---|---|
| Rust edition | `edition = "2021"` | Prevents accidental adoption of Edition 2024-only dependency features. |
| Cargo validation channel | `rust-toolchain.toml` channel `1.75.0` | Establishes the lowest supported stable toolchain for reproducible builds. |
| CLI dependency | `clap = "=4.4.18"` with `clap_lex = "=0.6.0"` | Prevents Cargo 1.75 from resolving newer clap lines that can pull Edition 2024-only `clap_lex` manifests. |
| LLVM binding | `inkwell = "0.4.0"` with feature `llvm15-0` | Aligns the backend with LLVM 15 development libraries available through common stable Linux package repositories. |

A clean native build should be performed with LLVM 15 development libraries installed, then Cargo should regenerate `Cargo.lock` from the pinned manifest constraints:

```bash
sudo apt-get update
sudo apt-get install -y llvm-15-dev llvm-15-tools llvm-15-runtime clang-15 lld-15 libclang-15-dev liblld-15-dev
cargo clean
cargo update
cargo check
cargo build --release
```

If a machine does not provide LLVM 15 development libraries, maintainers should still run the executable-logic validator to confirm that the v1.21-alpha compiler-core implementation, grammar specifications, and release metadata remain synchronized:

```bash
python3 scripts/validate_v121_executable_logic.py
python3 scripts/validate_v121_alpha_deployment.py
```

## Version-Gated Compiler Pipeline (Edition Routing)

Logicodex now implements **Edition Routing** — a version-gated pipeline architecture that allows experimental v1.30 constructs to be developed without destabilizing the v1.21-alpha baseline.

```bash
# Default: stable v1.21 pipeline (traps struct/enum/unsafe/extern as unimplemented)
logicodex compile program.ldx
logicodex check program.ldx

# Opt-in: experimental v1.30 pipeline (parses struct/enum/unsafe/extern)
logicodex compile program.ldx --pipeline v1.30
logicodex check program.ldx --pipeline v1.30
```

| Principle | Guarantee |
|---|---|
| Zero regression | v1.21 code paths untouched; default remains v1.21 |
| Zero overhead | v1.21 does not pass through HIR lowering |
| Fail-fast | `unreachable!()` safety nets prevent silent pipeline leaks |
| Scalable | Future versions use the same `CompilerPipeline` pattern |

See `CHANGELOG.md` for the full technical implementation details.

## Documentation Map

The current documentation set now separates language grammar, environment setup, and roadmap boundaries so maintainers can validate each concern independently.

| Document | Purpose |
|---|---|
| `GrammarandDictionary.md` | Explains the current grammar surface, token dictionary, expert canonical shorthand mode, primary Malay aliases, English pseudocode aliases, semicolon policy, hardware-zone boundary policy, and executable examples generated from `dict/core_map.json`. |
| `ENVIRONMENT_SETUP.md` | Records the confirmed WSL2/Linux baseline for building and validating **current logicodex v 1.21 alpha** with LLVM 15-oriented dependencies. |
| `MANUAL.md` | Provides concise compiler usage, build commands, frontend architecture notes, and the current example validation loop. |
| `docs/examples/REFLEX_ENGINE_EXAMPLES.md` | Records the refreshed reflex-engine example set and the paired `check` / `v130-check` compatibility boundary. |
| `docs/VS_CODE_EXTENSION.md` | Explains how to run the Logicodex VS Code Side View MVP for best-effort Malay/English pseudocode alias to expert canonical shorthand preview without changing Rust. |
| `v121_execution_design.md` | Captures the executable-logic and provenance design track for the v1.21-alpha milestone. |
| `spec/v1.30.0-alpha/v130_architecture_design.md` | Defines the documentation-first v1.30 architecture baseline with Rust skeleton structures for spans, HIR, type registry, struct layout, enum representation, FFI signatures, unsafe gatekeeping, semantic validation, and codegen contracts. |

## Practical Roadmap Summary

The next useful work is not to expand claims, but to improve proof. The repository now includes a refreshed reflex-engine example set that passes both the default v1.21-alpha `check` path and the opt-in v1.30.0-alpha `v130-check` probe. The next proof layer should add expected-output fixtures, backend parity checks, reliable build instructions, stable diagnostics, and repeatable validation scripts before broader WebAssembly, migration tooling, runtime attestation, or freestanding hardware experiments.

| Horizon | Emphasis | Success signal |
|---|---|---|
| Short term | Stabilize compiler core and examples. | `cargo check`, release build, validators, and representative `.ldx` programs pass reproducibly. |
| Medium term | Improve tooling and language boundaries. | Formatter, LSP diagnostics, stronger type rules, documented unsafe capability gates, and the v1.30 HIR/type-layout skeleton are available for implementation review. |
| Long term | Research-grade systems and security features. | Wasm target, migration assistant, attestation prototype, freestanding examples, and measured overhead data are available. |

## Governance and Licensing

Logicodex is distributed under permissive dual licensing through **MIT License** and **Apache License 2.0**. The name **Logicodex**, **Logicodex Language**, and official branding assets remain protected project identifiers to preserve ecosystem clarity and avoid misleading forks.

## Collaboration

Contributors interested in compiler engineering, LLVM optimization, operating-system targets, formal specification, documentation, or AI-assisted migration are invited to coordinate with **MSA Studio** through `mymsastudio@gmail.com`.

## v1.21-alpha Three-Tier Token Dictionary Expansion

The current logicodex v 1.21 alpha dictionary now records a practical three-tier token vocabulary for the requested program-structure, binding, control-flow, FFI vocabulary, resource vocabulary, type-family, hardware vocabulary, and bitwise-operator families. Each requested `TOKEN_*` entry in `dict/core_map.json` has a stable universal identifier, one expert canonical spelling, one primary Malay alias, and English pseudocode aliases.

This is now a **split-implementation capability upgrade**. The lexer and dictionary recognize the broader token vocabulary, while the compiler frontend, semantic analyzer, and LLVM backend execute only the safe, bounded subset that has implementation evidence and repeatable tests. Complex declaration tokens remain recognized but intentionally blocked with clear bilingual diagnostics until their type-system, layout, FFI, and safety semantics are designed and validated.

| Capability boundary | Current status | Practical note |
|---|---|---|
| Alias-to-canonical token families | Expanded at dictionary and lexer-recognition level | The token vocabulary is broader and logically verified. |
| `while`, `loop`, `break`, and `continue` | Implemented across AST, parser, semantic analysis, and LLVM code generation | Control-flow aliases such as `selagi`, `ulang`, `henti`, and `langkau` compile through the executable subset. |
| Logical, bitwise, and shift operators | Implemented across parsing, semantic typing, and code generation | Boolean logic returns `Bool`; integer bitwise and shift operators remain numeric. |
| `struct`, `enum`, `unsafe`, and `extern` | Recognized by the lexer and dictionary, then stopped at parser level with an unimplemented diagnostic | This prevents false production claims while preserving the token-completeness roadmap. |
| Embedded and freestanding vocabulary | Dictionary support plus existing address/hardware concepts | Hardware behavior still requires explicit target gates, examples, and validation. |

## Language Policy

All repository prose documentation is maintained in **English** so external contributors can read one consistent specification baseline. Compiler diagnostics and error messages must remain **bilingual Malay + English** whenever they are emitted to users, using the pattern `Malay message / English message`; this preserves accessibility for Malay-first users while keeping diagnostics searchable and reviewable by English-speaking maintainers.
