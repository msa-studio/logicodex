```text
=========================================================
  _                 _               _                 
 | |    ___   __ _ (_)  ___  ___   __| |  ___ __  __  
 | |   / _ \ / _` || | / __|/ _ \ / _` | / _ \ \/ /  
 | |__| (_) | (_| || || (__| (_) | (_| ||  __/ >  <   
 |_____\___/ \__, ||_| \___|\___/ \__,_| \___|/_/\_\  
             |___/                                    
             [ LOGICODEX COMPILER v1.0.1-alpha ]
             [ SECURITY ENHANCED - BARE-METAL  ]
=========================================================
Architect & Creator: Mohamad Supardi Abdul (mymsastudio@gmail.com)
```

# Logicodex White Paper

**Version:** v1.0.1-alpha (Internal Security & OS Freestanding Test)  
**Official Contact:** `mymsastudio@gmail.com`  
**Document Status:** Academic-grade alpha architecture paper for local evaluation and security research review.

## Executive Summary & Architectural Vision

Logicodex is a native systems-language experiment that joins a **dual-syntax human interface** with a **single deterministic compiler pipeline**. Its central hypothesis is that one language can serve both early learners and expert systems programmers by allowing multiple surface vocabularies to map into one canonical compiler representation. Beginner phrases and expert tokens are resolved through `dict/core_map.json`, tokenized into a shared token stream, parsed into a common abstract syntax tree, checked by `semantic.rs`, and lowered through LLVM-oriented code generation into native machine artifacts.

The v1.0.1-alpha upgrade changes the project from a simple Phase 1 MVP language demonstrator into a security-oriented systems research platform. It adds architectural hooks for runtime memory self-attestation, a freestanding target mode for beyond-OS execution, and formal documentation for raw physical memory access. LLVM is an appropriate foundation for this direction because it provides a collection of reusable compiler and toolchain technologies, including intermediate representation, optimization, target support, and machine-code emission facilities.[^llvm]

| Layer | Responsibility | v1.0.1-alpha Upgrade |
|---|---|---|
| Lexical layer | Maps beginner and expert vocabulary to normalized tokens. | Header-comment aware dictionary and source handling. |
| Parser | Constructs a single AST independent of surface syntax. | Maintains deterministic syntax normalization. |
| Semantic analysis | Rejects invalid control flow, undefined identifiers, and unsafe structural patterns. | Documents the static-check foundation for memory-safe evolution. |
| Code generation | Emits LLVM-backed object files. | Adds target-mode plumbing for native and freestanding profiles. |
| Security architecture | Defines active runtime self-defense. | Introduces Golden Hash memory integrity planning and panic mitigation semantics. |

The long-term vision is not merely to create another educational programming language. Logicodex is designed as a **native bare-metal binary pipeline** with an unusually accessible syntax layer. It aims to make low-level systems construction approachable without surrendering the compiler architecture required for kernels, hypervisors, IoT firmware, and high-assurance native tools.

## Dual-Syntax Compiler Model

Most programming languages bind a single syntactic culture to a compiler. Logicodex instead separates **surface vocabulary** from **semantic identity**. A novice may write a localized or descriptive form, while an expert may use compact symbols. Both are resolved into the same AST and therefore must obey the same static rules.

This design has two engineering advantages. First, it preserves one compiler implementation rather than fragmenting the language into beginner and advanced dialects. Second, it creates a path for localized pedagogy, domain-specific vocabulary, and professional-grade code generation to coexist without weakening the backend. The dictionary is not treated as a macro processor; it is a controlled lexical map that feeds a normal parser and semantic analyzer.

| Example Concern | Novice Surface | Expert Surface | Canonical Meaning |
|---|---|---|---|
| Variable binding | readable keyword form | symbolic keyword form | `let` statement in AST. |
| Output | beginner print verb | compact print command | runtime integer print call. |
| Branching | descriptive conditional | compact conditional | `if` node with optional else branch. |
| Arithmetic | readable operator aliases | standard operators | LLVM integer operations. |

## Radical Runtime Self-Attestation: Security Strategy

The most important v1.0.1-alpha security addition is the **Runtime Memory Integrity Verification Engine**. The engine is defined as an active self-attestation loop that protects the executable `.text` segment after program launch. The compiler-side contract is straightforward: produce a compile-time digest of immutable executable code, store it as a protected Golden Hash, and schedule a runtime verifier that continuously or periodically recomputes the digest from live memory.

Mathematically, the strategy can be expressed as follows. Let `T_compile` be the immutable byte sequence of the executable text region at compile or link finalization time, and let `H` be a cryptographic hash function. The compiler records `G = H(T_compile)`. At runtime, the verifier reads the current executable memory bytes `T_runtime` and computes `R = H(T_runtime)`. The integrity invariant is `R == G`. If the invariant fails, the runtime must assume memory tampering until proven otherwise.

| Symbol | Meaning |
|---|---|
| `T_compile` | Expected executable text segment bytes. |
| `G` | Golden Hash generated from expected text bytes. |
| `T_runtime` | Actual in-memory executable bytes during execution. |
| `R` | Live runtime hash. |
| `R != G` | Evidence of runtime code tampering, injection, or patching. |

The v1.0.1-alpha source tree prepares this model through `CodegenOptions.secure`, `MemoryIntegrityPlan`, secure CLI plumbing with `--secure` and `-s`, and generated attestation plan files. The planned hardened implementation uses CPU SHA/AES-NI extensions through LLVM intrinsic lowering where supported. Hardware acceleration matters because a runtime integrity loop must minimize overhead while still checking high-value memory frequently enough to detect process injection, fileless malware modification, and rootkit-style patching.[^intel]

> **Security invariant:** A Logicodex binary compiled under the secure profile should treat modification of its executable `.text` segment as a catastrophic integrity failure and respond with immediate panic, sensitive-register clearing, and hard process self-destruction.

The semantic layer also contributes to safety. In the current MVP, `semantic.rs` enforces identifier and structural correctness. The documented roadmap extends this foundation toward bounds-aware memory access, restricted raw pointer capabilities, and deterministic ownership or RAII-style scope cleanup. Therefore, Logicodex's security model is dual: **static prevention** through semantic checks and **dynamic detection** through runtime self-attestation.

## Active Self-Defense Mitigation Model

A passive integrity check that merely logs tampering is insufficient for adversarial systems. Logicodex defines self-defense as a three-stage mitigation sequence. First, the runtime enters an immediate panic path, halting normal control flow. Second, sensitive registers and volatile secrets are cleared to reduce post-compromise extraction. Third, the process triggers hard self-destruction, isolating the suspected compromised execution context.

| Stage | Action | Purpose |
|---|---|---|
| Panic | Stop normal execution immediately. | Prevent attacker-controlled code continuation. |
| Register clearing | Zero sensitive volatile state. | Reduce leakage of secrets or cryptographic material. |
| Hard self-destruction | Abort the process or freestanding execution context. | Isolate the threat and preserve the integrity boundary. |

This model is intentionally strict because the threat signal is direct code-integrity failure. In ordinary software, a crash is undesirable. In high-assurance runtime defense, continuing after verified code tampering may be worse than termination.

## Operating System Pervasiveness & Potential

Logicodex is positioned for OS and kernel experimentation because it is intentionally native, compact, and increasingly freestanding. A language designed for microkernels, IoT hypervisors, and distributed OS components must not assume the presence of a hosted runtime. The compiler must be able to emit objects that bootloaders and kernel linkers can consume, and it must provide a way to reason about memory-mapped devices directly.

The `--target freestanding` mode is the first explicit step. It selects the `x86_64-unknown-none` target concept, a static relocation posture, a kernel code model, and an `_start` entry symbol. It avoids platform-specific entry conventions such as Linux `crt0` or Windows GUI subsystem startup. This does not yet make Logicodex a complete kernel language, but it creates the necessary shape for bootloader-compatible artifacts.

| OS-Level Use Case | Logicodex Potential |
|---|---|
| Secure microkernels | Small statically checked code regions with runtime code-integrity attestation. |
| IoT hypervisors | Low-overhead native objects and direct hardware register access planning. |
| Boot services | Freestanding `_start` objects that can be linked into bootloader environments. |
| Distributed OS agents | Compact native binaries with self-defense against memory tampering. |

## Freestanding Compilation Layer

Hosted applications inherit substantial assumptions from the operating system: process startup, stack layout, standard library availability, dynamic loader behavior, and termination semantics. Freestanding programs cannot assume these services. C and C++ standards distinguish hosted and freestanding implementation environments, and systems developers commonly use freestanding modes to construct kernels or firmware.[^cpp]

Logicodex v1.0.1-alpha mirrors this distinction through an explicit compiler target parameter:

```bash
logicodex compile --target freestanding examples/01_tambah_pakar.ldx --object-only
```

In this profile, the backend emits an object intended for later integration by a bootloader, kernel linker script, hypervisor build, or firmware image generator. The compiler does not claim to provide a complete bootable image at this stage. It provides the **layout framework** required for operating-system development: target selection, entry-symbol control, runtime bypass, and physical-memory access documentation.

## Physical Memory Mapping and Raw Pointer Architecture

Operating-system code must frequently interact with memory-mapped hardware registers. Examples include text-mode VGA memory at physical address `0xB8000` on legacy x86 environments and serial UART I/O ports such as `0x3F8`. Logicodex documents a planned `*int` raw pointer representation for this class of work. The pointer model is deliberately not exposed as a casual hosted-language feature; it belongs to the freestanding backend and must be gated by explicit compiler mode and semantic rules.

```text
# Conceptual future Logicodex freestanding operation
let screen: *int = 0xB8000
*screen = 0x0741
```

The example expresses a direct write to a hardware-visible memory location. Such operations must bypass standard library abstractions, but they also bypass ordinary safety boundaries. Therefore, the compiler roadmap reserves them for a controlled unsafe backend gate, supported by semantic checks, target-mode validation, and documented physical address policies.

## LLVM Optimization and Bare-Metal Binary Pipeline

LLVM's value to Logicodex is not limited to object-file output. LLVM provides an intermediate representation and a broad optimization pipeline that can serve hosted and freestanding targets alike.[^llvm] A long-term Logicodex backend can lower the same AST into optimized LLVM IR, apply aggressive optimization passes, choose a target machine, and emit object files for native linking or kernel-level composition.

| Pipeline Step | Native Mode | Freestanding Mode |
|---|---|---|
| AST generation | Shared | Shared |
| Semantic analysis | Shared | Shared plus physical-memory gates |
| LLVM IR emission | `main` entry | `_start` entry |
| Target machine | Host default triple | Freestanding triple concept |
| Linking | Host linker and runtime assembly | External linker script or boot image tool |
| Runtime security | Process attestation | Kernel or firmware attestation policy |

## Open-Source Governance, Dual-Licensing, and Trademark Safeguards

Logicodex uses a permissive dual-license framework: MIT and Apache License 2.0. This mirrors a common open-source pattern in Rust-oriented ecosystems because it supports broad reuse while giving downstream users practical legal clarity. The source code may be studied, modified, and redistributed under the license terms included in the repository.

Trademark rights are separate from copyright licenses. The **Logicodex** name, ASCII logo, and official language identity are governed by trademark safeguards. A fork may use the licensed code, but it may not misrepresent itself as the official Logicodex compiler or imply endorsement by the project creator. This distinction protects users from confusion while preserving open-source freedoms.

| Asset | Usage Model |
|---|---|
| Source code | MIT/Apache-2.0 dual licensing. |
| Documentation | Distributed with the project for evaluation, study, and adaptation with attribution. |
| Name `Logicodex` | Protected project identity. |
| ASCII logo | Protected brand indicator for official releases. |
| Modified forks | Must clearly distinguish themselves from official Logicodex releases. |

## Research Roadmap

The current alpha repository is a foundation. The next engineering milestones are to replace plan-file generation with actual secure backend insertion, implement cryptographic digest construction at final link time, add target-specific runtime verifier stubs, define a precise raw pointer type system, and introduce linker-script examples for bootable freestanding artifacts.

| Milestone | Description |
|---|---|
| Secure verifier runtime | Emit concrete attestation routines and digest storage. |
| Intrinsic lowering | Map secure hash operations to SHA/AES-NI where the target supports them. |
| Raw pointer semantics | Add gated `*int` syntax and semantic rules for physical memory access. |
| Boot integration | Provide linker scripts and bootloader examples. |
| RAII scope cleanup | Formalize deterministic cleanup for resource-sensitive systems code. |

## Conclusion

Logicodex v1.0.1-alpha defines a clear direction: a dual-syntax language that remains approachable to learners while moving toward the requirements of secure native systems engineering. The upgrade introduces architectural definitions for runtime memory self-attestation, explicit freestanding compilation, and raw physical-memory access. These changes position Logicodex as a candidate research platform for secure microkernels, IoT hypervisors, firmware experiments, and distributed native agents. The implementation remains alpha, but the specification now points decisively beyond ordinary hosted applications and toward **security-enhanced bare-metal computation**.

## References

[^llvm]: [LLVM Project, official project site](https://llvm.org/).
[^intel]: [Intel Intrinsics Guide, SHA and AES instruction families](https://www.intel.com/content/www/us/en/docs/intrinsics-guide/index.html).
[^cpp]: [cppreference, freestanding and hosted implementations](https://en.cppreference.com/w/cpp/freestanding).
