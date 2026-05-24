# Logicodex Phase 1 MVP Developer Manual

Architect & Creator: Mohamad Supardi Abdul (mymsastudio@gmail.com)

```text
=========================================================
  _                 _               _                 
 | |    ___   __ _ (_)  ___  ___   __| |  ___ __  __  
 | |   / _ \ / _` || | / __|/ _ \ / _` | / _ \\ \/ /  
 | |__| (_) | (_| || || (__| (_) | (_| ||  __/ >  <   
 |_____\___/ \__, ||_| \___|\___/ \__,_| \___|/_/\_\  
             |___/                                    
                  [ COMPILER PHASE 1 - experimental freestanding ]
=========================================================
```

**Repository:** `logicodex`  
**Compiler executable:** `logicodex`  
**Official source extension:** `.ldx`  
**Version:** 1.21-alpha
**Security Profile:** Internal Security & OS Freestanding Test

## Overview

Logicodex is a native programming language compiler implemented in Rust. The Phase 1 MVP demonstrates an alias-to-canonical frontend in which Malay/English pseudocode aliases and expert canonical shorthand are normalized through `dict/core_map.json` into the same compiler token identities. Once lexing is complete, all supported surface styles produce the same AST, pass through the same semantic analyzer, and are lowered to LLVM machine code.

## Compiler Frontend and Architecture

```text
[ Malay/English Alias Code (.ldx) ] ──► (Lexer + core_map.json) ──► [ Unified Token Stream ]
                                                                           │
[ Expert Canonical Code (.ldx) ] ──► (Lexer + core_map.json) ──► [ Abstract Syntax Tree ]
                                                              │
[ Native Binary ] ◄── (LLVM Backend Optimization O3) ◄── [ LLVM IR Generation ]
```

## Build Requirements

| Dependency | Purpose |
|---|---|
| Rust and Cargo | Build the compiler executable. |
| LLVM 15 development libraries | Required by the configured `inkwell = 0.4.0` backend feature `llvm15-0`. |
| C-compatible linker | Links generated object files and the platform runtime bridge. |

## Build and Use

```bash
cd logicodex
python3 scripts/validate_v121_executable_logic.py
cargo test --target x86_64-unknown-linux-gnu
RUSTFLAGS='-D warnings' cargo build --target x86_64-unknown-linux-gnu
./target/x86_64-unknown-linux-gnu/debug/logicodex logo
./target/x86_64-unknown-linux-gnu/debug/logicodex tokens examples/01_tambah_pemula.ldx
./target/x86_64-unknown-linux-gnu/debug/logicodex check examples/01_tambah_pemula.ldx
```

Set `LOGICODEX_LINKER` to override the linker used by the compiler. For machine setup details, use `ENVIRONMENT_SETUP.md`; for grammar, dictionary, aliases, and executable examples, use `GrammarandDictionary.md`.

## Current Example Compatibility Suite

The refreshed `examples/` directory is the maintained reflex-engine compatibility suite for **current Logicodex v1.21-alpha** plus the dormant **v1.30.0-alpha** probe. It includes expert canonical and Malay beginner programs for arithmetic, functions, loops, bitwise operators, hardware-zone provenance, and Boolean conditionals. Maintainers should validate the full suite rather than a single sample file when changing parser, semantic, CLI, or documentation behavior.

| Example group | Files | Compatibility expectation |
|---|---|---|
| Legacy smoke examples | `hello.ldx`, `matematik.ldx`, `perkakasan.ldx` | Continue to pass the default `check` path. |
| Reflex arithmetic examples | `01_tambah_pakar.ldx`, `01_tambah_pemula.ldx` | Pass both `check` and `v130-check` after the syntax refresh. |
| Reflex feature examples | `02_fungsi_matematik.ldx` through `06_logik_bersyarat.ldx` | Pass both `check` and `v130-check` while avoiding recognized-but-blocked roadmap constructs. |

```bash
for file in examples/*.ldx; do
  cargo run --quiet -- check "$file"
  cargo run --quiet -- v130-check "$file"
done
```

The detailed file-by-file inventory is maintained in `docs/examples/REFLEX_ENGINE_EXAMPLES.md`.

## v1.21-alpha Split-Implementation Boundary

The executable v1.21-alpha subset now supports `while`, `loop`, `break`, `continue`, logical operators, bitwise operators, and shift operators through the AST, parser, semantic analyzer, and LLVM backend. The lexer and dictionary also recognize complex roadmap tokens such as `struct`, `enum`, `unsafe`, and `extern`, but these are intentionally stopped at parser level with an unimplemented diagnostic until their type-layout, ABI, and safety semantics are designed and validated.

Compiler diagnostics are emitted in **bilingual Malay + English** form, using the pattern `Malay message / English message`. Prose documentation remains **English-only** so that the repository has a single reviewable documentation language while still keeping user-facing errors accessible to Malay-first and English-speaking users.

## Runtime Bridge

The compiler lowers `PAPAR` and `print` to `logicodex_print_i64`. The Linux bridge writes through native syscall-oriented assembly, while the Windows bridge is structured around Win32 console output. This keeps Phase 1 free from a mandatory virtual machine or garbage collector.


## Peer-Review Alignment Notes for v1.21-alpha

The Phase 1 compiler implements the verified core path: dictionary loading, lexing, parsing, AST construction, semantic analysis, and LLVM-Inkwell backend generation. WebAssembly targeting, the Logicodex Migrator Engine, and Continuous Runtime Memory Attestation are Phase 2/3 roadmap specifications. The dictionary is consumed during lexing only; parser behavior is based on canonical `TokenKind` values rather than macro rewriting. Freestanding memory examples such as `0xB8000` are OS-less or kernel-authority examples and are not valid hosted user-space memory operations under Linux or Windows without explicit kernel-space mapping.
