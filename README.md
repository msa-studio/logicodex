# Logicodex

```
+------------------------------------------------------------------+
|  ⚠️  v1.45.0-alpha — Research-grade systems language prototype   |
|      Not for production. See ROADMAP.md for current phase and    |
|      maturity status.                                            |
+------------------------------------------------------------------+
```

**A deterministic systems programming language with compile-time capability security and bilingual (Malay/English) syntax.**

---

## Table of Contents

- [What's Actually Working](#whats-actually-working)
- [Maturity Matrix](#maturity-matrix)
- [Quick Start](#quick-start)
- [Current Limitations](#current-limitations)
- [Documentation Map](#documentation-map)
- [Governance](#governance)
- [License](#license)

---

## What's Actually Working

### Compiler Core (Alpha)

| Stage | Status | Detail |
|---|---|---|
| Lexer | ✅ Functional | Full Malay/English bilingual lexing — 60 Malay keywords, 200+ keyword aliases |
| Parser | ✅ Functional | Expression parsing, declarations, function definitions, type annotations |
| AST | ✅ Functional | Complete AST representation with source-location tracking |
| Semantic Analysis | ✅ Functional | Name resolution, scope checking, type inference |
| Type Checker | ✅ Functional | `I32` / `I64` / `F32` / `F64` / `Bool` with 32 typed error variants |
| LLVM IR Generation | ✅ Functional | Produces native `.o` object files via LLVM backend |
| HIR | ⚠️ Dormant | Types and structure defined; bypassed in production pipeline |

The compiler successfully compiles source code through the full pipeline to native object files on supported targets.

### Security — Capability System (Alpha)

- ✅ **Compile-time capability gates** — topology verification runs at compile time
- ✅ **Capability lattice** — grant, revoke, and transfer operations are semantically checked
- ⚠️ **Hardware gate** — compile-time check validates intent; runtime enforcement is a stub (no actual hardware integration yet)

### Concurrency — Actor Model (Alpha)

- ✅ **Actor types** — `Aktor`, `Mesej`, `Kotak` (Mailbox) types are defined and parse correctly
- ✅ **Semantic checks** — actor isolation and message-passing rules are validated
- ❌ **Runtime actor thread pool** — **NOT IMPLEMENTED** (0 lines of code). The actor model exists only as a type system and semantic checker; there is no executing runtime.

### Platform Targets

| Target | Status | Detail |
|---|---|---|
| `x86_64-unknown-linux-gnu` | ✅ Native ELF | Produces working `.o` object files |
| `x86_64-freestanding-elf` | ⚠️ Code complete | ELF generation implemented; not yet booted on real hardware |
| `wasm32-wasi` | ⚠️ LLVM IR works | IR generation verified; manual linking required for `.wasm` output |
| `aarch64-freestanding-elf` | ❌ Skeleton | LLVM triple configured only; no code generation |
| `riscv64-freestanding-elf` | ❌ Skeleton | LLVM triple configured only; no code generation |

### FFI — Foreign Function Interface (Alpha)

- ⚠️ **Raylib bindings** — 55 wrapper functions registered, covering ~11% of the full Raylib API. Basic windowing and drawing primitives available; most of the API surface is not yet wrapped.

### Infrastructure

| Component | Status | Detail |
|---|---|---|
| Benchmark framework | ✅ Functional | 4-layer benchmark suite (micro, component, integration, end-to-end) |
| Validator tiering | ✅ Functional | Tier A (6 tests), Tier B (13 tests), Tier C (8 tests) |
| CI / Testing | ⚠️ Partial | Tier A + B run in CI; some tests currently failing |
| Dual licensing | ✅ Active | MIT OR Apache-2.0 at user option |
| Security policy | ✅ Active | See `SECURITY.md` |
| Contributing guide | ✅ Active | See `CONTRIBUTING.md` |

---

## Maturity Matrix

Legend: 🟢 Alpha — functional and tested in CI &nbsp;|&nbsp; 🟡 Partial — works for some cases, gaps exist &nbsp;|&nbsp; 🔴 Skeleton — types defined, not functional &nbsp;|&nbsp; ⚪ Planned — not started

| Capability | Status | Notes |
|---|---|---|
| Bilingual lexer (MS/EN) | 🟢 Alpha | 60 keywords, 200+ aliases, CI tested |
| Parser | 🟢 Alpha | Full expression/decl/function grammar |
| AST construction | 🟢 Alpha | Source-located, traversable |
| Semantic analysis | 🟢 Alpha | Name resolution, scope checking |
| Type checker (5 base types) | 🟢 Alpha | I32/I64/F32/F64/Bool + 32 errors |
| LLVM IR → `.o` code generation | 🟢 Alpha | x86_64-linux-gnu verified |
| Compile-time capability gates | 🟢 Alpha | Topology verification active |
| Benchmark framework (4 layers) | 🟢 Alpha | Runs in CI |
| Validator tiering (A/B/C) | 🟢 Alpha | 6/13/8 tests respectively |
| HIR (high-level IR) | 🟡 Partial | Structure complete, dormant in pipeline |
| Capability hardware gate | 🟡 Partial | Compile-time check works; runtime stub |
| x86_64 freestanding target | 🟡 Partial | Code complete, not hardware-booted |
| WASM target | 🟡 Partial | LLVM IR emitted, manual linking needed |
| Actor semantic checks | 🟡 Partial | Types + isolation rules enforced |
| CI pipeline | 🟡 Partial | Tier A+B execute; some failures |
| Raylib FFI | 🟡 Partial | 55 wrappers (~11% coverage) |
| Actor runtime (thread pool) | 🔴 Skeleton | 0 LOC — types exist, no execution |
| aarch64 freestanding target | 🔴 Skeleton | LLVM triple only |
| riscv64 freestanding target | 🔴 Skeleton | LLVM triple only |
| Actor mailbox scheduler | 🔴 Skeleton | Type defined, no runtime logic |
| Self-hosted compilation | ⚪ Planned | Not started |
| Package manager (`pakej`) | ⚪ Planned | Not started |

**Audit summary: 7/22 capabilities at 🟢 Alpha, 9/22 at 🟡 Partial, 5/22 at 🔴 Skeleton, 2/22 at ⚪ Planned.**

---

## Quick Start

### Hello World (Bilingual)

Logicodex accepts Malay or English keywords interchangeably:

```
// Malay syntax
fungsi utama(): Tiada {
    cetak_baris("Selamat Dunia!");
}
```

```
// English syntax — equivalent
function main(): Void {
    print_line("Hello World!");
}
```

### Compilation

```bash
# Build the compiler
cargo build --release

# Compile a source file (produces .o object file)
./target/release/logicodex compile src.ms --target x86_64-unknown-linux-gnu

# Link to produce executable
clang output.o -o myprogram
./myprogram
```

### Supported Targets

```bash
# x86_64 Linux (native) — produces working object files
./logicodex compile src.ms --target x86_64-unknown-linux-gnu

# x86_64 freestanding — code complete, not hardware-booted
./logicodex compile src.ms --target x86_64-freestanding-elf

# WASM — LLVM IR works, manual linking required
./logicodex compile src.ms --target wasm32-wasi
```

---

## Current Limitations

1. **No actor runtime.** Actor types parse and semantically check, but there is zero implementation of the thread pool, mailbox scheduler, or message dispatcher. Actors cannot actually execute.

2. **No ARM64 or RISC-V code generation.** Only the LLVM target triple is configured; no instruction selection, no ABI lowering, no object file output.

3. **CI is failing.** Tier A and B tests run, but several are currently red. The project does not have a clean baseline.

4. **No self-hosted compilation.** The compiler is written in Rust and bootstrapped via `cargo`. There is no Logicodex-in-Logicodex path.

5. **HIR is dead code.** The High-Level IR module is fully typed and structured but is bypassed in the production compilation pipeline. It contributes nothing to the working compiler.

6. **WASM requires manual linking.** The compiler emits LLVM IR that is valid for the `wasm32-wasi` target, but producing a final `.wasm` module requires manual `wasm-ld` invocation with correct flags.

7. **Capability hardware gate is a stub.** The compile-time topology verification works, but the runtime path that would interact with hardware security features (TPM, enclaves, etc.) is an empty function body.

8. **FFI coverage is minimal.** Only ~11% of the Raylib API is wrapped. Most functions, constants, and complex types (shaders, audio, models) are not accessible from Logicodex.

---

## Documentation Map

| Document | Purpose |
|---|---|
| [`ROADMAP.md`](ROADMAP.md) | Current development phase, milestone tracking, estimated dates |
| [`ROADMAP_POLICY.md`](ROADMAP_POLICY.md) | Phase-gating policy, promotion criteria, maturity definitions |
| [`SPECIFICATION.md`](SPECIFICATION.md) | Language specification: syntax, semantics, type system, capability model |
| [`CONTRIBUTING.md`](CONTRIBUTING.md) | How to contribute, coding standards, commit conventions |
| [`SECURITY.md`](SECURITY.md) | Security policy, vulnerability reporting, threat model |
| [`CHANGELOG.md`](CHANGELOG.md) | Version history and release notes |

---

## Governance

This project follows **strict phase-gated development**. Capabilities do not progress from Skeleton → Partial → Alpha without explicit review against documented criteria. See [`ROADMAP_POLICY.md`](ROADMAP_POLICY.md) for the promotion rules, review checklist, and current phase boundaries.

**v1.45.0-alpha is a research prototype.** It is suitable for experimentation, language design feedback, and compiler frontend research. It is **not suitable** for production systems, shipping applications, or security-critical workloads.

---

## License

Logicodex is licensed under the **Mozilla Public License 2.0 (MPL-2.0)**.

> This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
> If a copy of the MPL was not distributed with this file, You can obtain one at
> https://mozilla.org/MPL/2.0/.

### Why MPL-2.0?

MPL-2.0 is a **file-level copyleft** license that:

- ✅ **Preserves open semantics** — modifications to Logicodex source files must be shared under MPL-2.0
- ✅ **Allows ecosystem adoption** — you can use Logicodex in proprietary projects via separate files
- ✅ **Protects core evolution** — prevents silent proprietary mutation of compiler core files
- ✅ **Enables commercial growth** — compatible with proprietary tooling and extensions
- ✅ **GPL-compatible** — can be combined with GPL v2+, LGPL 2.1+, AGPL 3.0+ projects

### Historical Licenses

Versions prior to this licensing update were distributed under MIT OR Apache-2.0.
The previous license files (`LICENSE-MIT`, `LICENSE-APACHE`) are retained for
historical reference but no longer apply to current or future distributions.

SPDX-License-Identifier: MPL-2.0

© 2025 Mohamad Supardi Abdul (mymsastudio@gmail.com). All rights reserved.
