# Logicodex Language — v1.0.1-alpha
```text
=========================================================
  _                 _               _                 
 | |    ___   __ _ (_)  ___  ___   __| |  ___ __  __  
 | |   / _ \ / _` || | / __|/ _ \ / _` | / _ \ \/ /  
 | |__| (_) | (_| || || (__| (_) | (_| ||  __/ >  <   
 |_____\___/ \__, ||_| \___|\___/ \__,_| \___|/_/\_\  
             |___/                                    
             [ LOGICODEX COMPILER v1.0.1-alpha ]
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

| Area | v1.0.1-alpha Status | Engineering Direction |
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

## Governance and Licensing

Logicodex is distributed under permissive dual licensing through **MIT License** and **Apache License 2.0**. The name **Logicodex**, **Logicodex Language**, and official branding assets remain protected project identifiers to preserve ecosystem clarity and avoid misleading forks.

## Collaboration

Contributors interested in compiler engineering, LLVM optimization, operating-system targets, formal specification, documentation, or AI-assisted migration are invited to coordinate with **MSA Studio** through `mymsastudio@gmail.com`.
