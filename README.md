# Logicodex Language — v1.21-alpha
## v1.21-alpha Phase 2 Deployment Integration

The **v1.21-alpha** milestone synchronizes the formal language specification baseline with the Undefined Behavior and Pointer Provenance model. It adds a canonical four-layer EBNF grammar, a layered C/C++/Rust-derived memory-safety classification, and a zero-overhead Critical/Medium/Low severity architecture intended for direct LLVM IR lowering without runtime speed penalties.

```text
=========================================================
  _                 _               _                 
 | |    ___   __ _ (_)  ___  ___   __| |  ___ __  __  
 | |   / _ \ / _` || | / __|/ _ \ / _` | / _ \ \/ /  
 | |__| (_) | (_| || || (__| (_) | (_| ||  __/ >  <   
 |_____\___/ \__, ||_| \___|\___/ \__,_| \___|/_/\_\  
             |___/                                    
             [ LOGICODEX COMPILER v1.21-alpha ]
             [ DUAL-SYNTAX LLVM SYSTEMS LANGUAGE ]
=========================================================
Architect & Creator: Mohamad Supardi Abdul (mymsastudio@gmail.com)
```

## Executive Summary

Logicodex is a dual-syntax, LLVM-backed systems programming language created by **Mohamad Supardi Abdul** (`mymsastudio@gmail.com`). It is designed to reduce the cognitive gap between readable human intent and native machine execution by allowing novice-oriented pseudocode and expert shorthand to compile through one deterministic compiler pipeline.

The current **Phase 1** alpha delivers the verified working core compiler infrastructure: the `dict/core_map.json` dictionary loader, Lexer, Parser, AST construction, Semantic Analyzer, and LLVM-Inkwell backend path for native-oriented output. Roadmap capabilities including the **WebAssembly Target**, **Logicodex Migrator Engine**, and **Continuous Runtime Memory Attestation** are formally defined engineering specifications for **Phase 2/3** and should not be read as production-complete Phase 1 features.

## Compiler Pipeline

```text
[ Novice Code (.ldx) ] ──► (Lexer + core_map.json) ──► [ Unified Token Stream ]
                                                              │
[ Expert Code (.ldx) ] ──► (Lexer + core_map.json) ──► [ Abstract Syntax Tree ]
                                                              │
[ Native Binary ] ◄── (LLVM Backend Optimization O3) ◄── [ LLVM IR Generation ]
```

The dictionary is consumed strictly during lexing. Surface forms such as `MULA`, `BEGIN`, and `{` normalize into canonical token identities such as `TokenKind::Start` before parsing begins. The parser therefore consumes a uniform token stream rather than performing macro rewriting or grammar-level dialect conversion.

## Architectural Highlights

| Area | v1.21-alpha Status | Engineering Direction |
|---|---|---|
| Dual syntax | Implemented in the Phase 1 frontend through dictionary-aware tokenization. | Expand localized and domain-specific token families while preserving deterministic builds. |
| Static semantics | Implemented for the Phase 1 core language. | Extend toward formal EBNF, type-system boundaries, pointer provenance, and UB catalog definitions. |
| LLVM backend | Implemented through the Rust Inkwell path for core expressions and native object generation. | Mature target triples, ABI contracts, and linker policies. |
| WebAssembly target | Architectural roadmap for Phase 2/3. | Add a Wasm target prototype, browser playground, and sandboxed package execution. |
| Migrator Engine | Architectural roadmap for Phase 2/3. | Convert legacy source into readable Logicodex with explicit semantic review. |
| Continuous runtime memory attestation | Architectural roadmap for Phase 2/3. | Convert the current security plan contract into concrete digest insertion, verifier stubs, and target-specific fail-stop mitigation. |
| Freestanding support | Alpha target profile and plan generation. | Add linker scripts, bootloader examples, raw-pointer gates, and hardware-region policies. |

## Freestanding and Hosted Safety Boundary

The `--target freestanding` profile is intended for OS-less integration contexts such as kernels, bootloaders, firmware, and hypervisors. Examples that write to physical addresses such as VGA text memory at `0xB8000` are valid only under freestanding execution or equivalent kernel-space mapping authority. Hosted operating systems such as Linux and Windows normally protect processes with virtual memory paging and ASLR, so direct physical address manipulation from user space should be expected to fail through page-fault defenses unless mediated by mechanisms such as `/dev/mem` or Ring-0 drivers.

## Security Roadmap Boundary

The active self-defense model defines a fail-stop invariant for executable-code tampering. In hosted environments, hard self-destruction means process termination through native abort behavior. In freestanding environments, it means a target-specific termination primitive such as a CPU Triple Fault, `hlt` halt loop, or hardware watchdog reset. The Phase 1 tree documents this contract; Phase 2/3 work must implement the production verifier and mitigation runtime.

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

## Governance and Licensing

Logicodex is distributed under permissive dual licensing through **MIT License** and **Apache License 2.0**. The name **Logicodex**, **Logicodex Language**, and official branding assets remain protected project identifiers to preserve ecosystem clarity and avoid misleading forks.

## Collaboration

Contributors interested in compiler engineering, LLVM optimization, operating-system targets, formal specification, documentation, or AI-assisted migration are invited to coordinate with **MSA Studio** through `mymsastudio@gmail.com`.
