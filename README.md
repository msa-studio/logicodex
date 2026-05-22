# Logicodex Language — v1.21-alpha
## v1.21-alpha Practical Compiler Baseline

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

Logicodex is a dual-syntax, LLVM-backed systems programming language created by **Mohamad Supardi Abdul** (`mymsastudio@gmail.com`). Its practical aim is to reduce the cognitive gap between readable human intent and native-oriented compiler output by allowing novice-oriented pseudocode and expert shorthand to flow through one deterministic frontend.

The current **Phase 1** alpha focuses on a working compiler core: the `dict/core_map.json` dictionary loader, lexer, parser, AST construction, semantic analyzer, and LLVM-Inkwell backend path for native-oriented object generation. Roadmap capabilities including the **WebAssembly target**, **Logicodex Migrator Engine**, continuous runtime memory attestation, and deeper freestanding support are **objectives to be built and validated over time**, not completed implementation claims in the current alpha.

## Compiler Pipeline

```text
[ Novice Code (.ldx) ] ──► (Lexer + core_map.json) ──► [ Unified Token Stream ]
                                                              │
[ Expert Code (.ldx) ] ──► (Lexer + core_map.json) ──► [ Abstract Syntax Tree ]
                                                              │
[ Native-Oriented Object Output ] ◄── (LLVM Backend) ◄── [ LLVM IR Generation ]
```

The dictionary is consumed strictly during lexing. Surface forms such as `MULA`, `BEGIN`, and `{` normalize into canonical token identities such as `TokenKind::Start` before parsing begins. The parser therefore consumes a uniform token stream rather than performing macro rewriting or grammar-level dialect conversion.

## Current Capability Boundary

The project should be read as an **alpha compiler and specification prototype**. It is suitable for compiler-core experimentation, syntax design, semantic-analysis iteration, LLVM backend development, and documentation of future systems-programming goals. It should not yet be presented as a hardened production compiler, a complete freestanding operating-system toolchain, or a formally verified security platform.

| Area | Current v1.21-alpha status | Practical next objective |
|---|---|---|
| Dual syntax | Implemented in the Phase 1 frontend through dictionary-aware tokenization. | Expand localized and domain-specific token families while preserving deterministic builds. |
| Static semantics | Implemented for the Phase 1 core language and selected safety checks. | Tighten type-system boundaries, diagnostics, pointer-provenance rules, and UB catalog coverage. |
| LLVM backend | Implemented through the Rust Inkwell path for core expressions and native-oriented object generation. | Mature target triples, ABI contracts, linker policies, and executable examples. |
| WebAssembly target | Long-term roadmap objective. | Build a small Wasm prototype after native examples and tests are stable. |
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

## Practical Roadmap Summary

The next useful work is not to expand claims, but to improve proof. The project should prioritize a small set of native examples, reliable build instructions, stable diagnostics, and repeatable validation scripts. After that foundation is stable, the roadmap can safely move into WebAssembly, migration tooling, runtime attestation, and freestanding hardware experiments.

| Horizon | Emphasis | Success signal |
|---|---|---|
| Short term | Stabilize compiler core and examples. | `cargo check`, release build, validators, and representative `.ldx` programs pass reproducibly. |
| Medium term | Improve tooling and language boundaries. | Formatter, LSP diagnostics, stronger type rules, and documented unsafe capability gates exist. |
| Long term | Research-grade systems and security features. | Wasm target, migration assistant, attestation prototype, freestanding examples, and measured overhead data are available. |

## Governance and Licensing

Logicodex is distributed under permissive dual licensing through **MIT License** and **Apache License 2.0**. The name **Logicodex**, **Logicodex Language**, and official branding assets remain protected project identifiers to preserve ecosystem clarity and avoid misleading forks.

## Collaboration

Contributors interested in compiler engineering, LLVM optimization, operating-system targets, formal specification, documentation, or AI-assisted migration are invited to coordinate with **MSA Studio** through `mymsastudio@gmail.com`.
