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

# Logicodex Language Engine

**Version:** v1.0.1-alpha (Internal Security & OS Freestanding Test)  
**Official Source Extension:** `.ldx`  
**Compiler Command:** `logicodex`  
**Licensing:** Dual licensed under MIT and Apache License 2.0, with separate trademark restrictions for the **Logicodex** name and logo.

## Project Identity

Logicodex is a Phase 1 native compiler research project designed as a dual-syntax programming language engine. Its language layer accepts beginner-oriented vocabulary and expert symbolic vocabulary through a dynamic dictionary, then normalizes both forms into a common abstract syntax tree for semantic analysis and LLVM-backed object generation. The v1.0.1-alpha upgrade focuses on the foundations for **active runtime memory integrity**, **bare-metal freestanding compilation**, and explicit documentation of raw physical-memory access patterns.

| Attribute | Current Specification |
|---|---|
| Release | `v1.0.1-alpha` |
| Security profile | Internal Security & OS Freestanding Test |
| Official contact | `mymsastudio@gmail.com` |
| Author block | `Architect & Creator: Mohamad Supardi Abdul (mymsastudio@gmail.com)` |
| Primary backend | LLVM object generation through Rust and Inkwell |
| Native path | Host-linked executable using the repository runtime assembly |
| Freestanding path | `logicodex compile --target freestanding` object generation for bootloader or kernel integration |

## Installation Dependencies

Logicodex is written in Rust and uses LLVM through the `inkwell` crate. LLVM provides the compiler infrastructure for generating and optimizing machine-level artifacts, while Rust supplies the implementation language, package manager, and safety-oriented development ecosystem.[^llvm] [^rust]

| Dependency | Purpose |
|---|---|
| Rust `rustc` and `cargo` | Build the compiler and resolve Rust crate dependencies. |
| LLVM development libraries | Provide target initialization, module verification, IR handling, and object-file emission. |
| C toolchain or linker | Link native-mode object output and runtime assembly into an executable. |
| `zip`, `tar`, `sha256sum` | Regenerate release archives and checksums. |

On Ubuntu-like systems, the intended setup is:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
rustup default stable
sudo apt update
sudo apt install -y build-essential clang lld llvm-17 llvm-17-dev zip tar
```

If the operating system does not ship LLVM 17, install LLVM from the official LLVM package repository and export the LLVM prefix before building.

```bash
export LLVM_SYS_170_PREFIX=/usr/lib/llvm-17
export PATH=/usr/lib/llvm-17/bin:$PATH
cargo build --release
```

## Command Overview

The compiler exposes a small command surface intended for transparent research evaluation.

| Command | Purpose |
|---|---|
| `logicodex logo` | Prints the official ASCII identity banner. |
| `logicodex check file.ldx` | Parses and semantically validates a Logicodex program. |
| `logicodex tokens file.ldx` | Prints the token stream produced by the dynamic dictionary lexer. |
| `logicodex compile file.ldx` | Emits an object file and links a native executable using the repository runtime. |
| `logicodex compile -s file.ldx` | Generates the native artifact and writes a security attestation design plan. |
| `logicodex compile --target freestanding file.ldx` | Emits a freestanding object using the `_start` entry contract and OS-independent target settings. |

## Runtime Memory Integrity Verification Engine

The `--secure` or `-s` flag activates the architectural security path for **active runtime memory self-attestation**. The compiler records a plan for a hardened backend in which the immutable `.text` segment of a binary is hashed at compile time to form a **Golden Hash**. At runtime, a lightweight verifier repeatedly hashes the executable memory region in RAM and compares the live digest against the Golden Hash. The v1.0.1-alpha codebase prepares internal representations for this contract through `MemoryIntegrityPlan` and secure compile-option plumbing.

> The security strategy is not a claim that the current alpha build is production hardened. It is an explicit architectural contract for progressive implementation: compile-time Golden Hash generation, protected digest embedding, CPU SHA/AES-NI accelerated hashing through LLVM intrinsic lowering where supported, periodic runtime comparison, and immediate self-defense if memory tampering is detected.

If a mismatch occurs, the required mitigation model is **immediate panic**, **sensitive-register clearing**, and **hard process self-destruction**. The threat model includes process injection, fileless malware modification, rootkit patching, and unauthorized runtime binary rewriting.

## Freestanding and Beyond-OS Compilation

Logicodex v1.0.1-alpha introduces an explicit freestanding layout framework through `logicodex compile --target freestanding`. The target bypasses ordinary operating-system runtime linkage, avoids Linux `crt0` or Windows subsystem entry expectations, and emits an object with the `_start` entry symbol for integration into bootloaders, kernels, hypervisors, or firmware-level environments. This aligns with common systems-development practice where freestanding programs cannot assume a hosted standard library, operating-system process model, or default application entry point.[^freestanding]

| Mode | Entry Symbol | Runtime Assumption | Intended Use |
|---|---|---|---|
| `native` | `main` | Host OS process and native linker | Normal executable experiments. |
| `freestanding` | `_start` | No hosted OS runtime | Bootloader, microkernel, hypervisor, firmware, and embedded experiments. |

## Raw Physical Memory and Pointer Manipulation

The freestanding architecture documents a future `*int` raw pointer representation for memory-mapped I/O. The planned backend contract permits direct writes to hardware-mapped memory such as VGA text memory at `0xB8000` and direct serial UART communication ports such as `0x3F8`. These features are intentionally gated behind the freestanding backend model because direct physical addressing is powerful, unsafe, and inappropriate for ordinary hosted applications.

## Repository Structure

```text
logicodex/
├── Cargo.toml
├── README.md
├── WHITE_PAPER.md
├── LICENSE-MIT
├── LICENSE-APACHE
├── TRADEMARK.md
├── NOTICE
├── dict/core_map.json
├── examples/*.ldx
├── stdlib/*.ldx
├── src/*.rs
└── scripts/update_release_archives.sh
```

Every `.rs`, `.json`, `.ldx`, and standard-library source asset carries the required v1.0.1-alpha header.

## Governance, Licensing, and Trademark Safeguards

Logicodex source code is distributed under a permissive MIT/Apache-2.0 dual-license model. This allows broad study, modification, and redistribution of code while preserving compatibility with modern open-source ecosystems. The **Logicodex** name, ASCII logo, and project identity remain protected by trademark guidance. Forks may use the source licenses, but they must not imply official endorsement or present modified products as the official Logicodex language engine without permission.

## Release Packaging

Regenerate release archives from the repository root with:

```bash
./scripts/update_release_archives.sh
```

The script produces:

```text
logicodex-v1.0.1-alpha.zip
logicodex-v1.0.1-alpha.tar.gz
logicodex-v1.0.1-alpha.sha256
```

## References

[^llvm]: [LLVM Project, official project site](https://llvm.org/).
[^rust]: [Rust Programming Language, official documentation](https://www.rust-lang.org/).
[^freestanding]: [OSDev Wiki, Bare Bones and freestanding kernel development concepts](https://wiki.osdev.org/Bare_Bones).
