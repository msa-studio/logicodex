# Logicodex
+------------------------------------------------------------------+
|  v0.46.0-alpha — Research-grade systems language prototype       |
|  Not for production. See current-authority.md before planning.   |
+------------------------------------------------------------------+

**A deterministic systems programming language with a zero-runtime core,
compile-time capability checking, and bilingual (Malay/English) syntax.**

> Note on versioning: the engine crate is versioned `0.46.0-alpha` (Cargo);
> project-milestone docs use a separate `v1.4x` axis. They are different axes,
> not a contradiction.

---

## Identity: Zero Runtime Core, Optional Runtime Profiles

Logicodex is **zero-runtime by default, runtime-capable by profile**. The core
language lowers to a portable semantic IR and emits native code with no mandatory
runtime linked in. Features that genuinely need a scheduler/state/sandbox (actors,
channels, sandboxed I/O, runtime capability enforcement) are **opt-in profiles**,
not part of the core. See `docs/architecture/runtime-doctrine.md`.

---

## What's actually working (verified)

The compiler runs a single pipeline: source → lexer → parser → AST → **HIR
lowering (the sole execution path)** → semantic gate → LLVM codegen → native
object/executable.

| Capability | Status | Detail |
|---|---|---|
| Bilingual lexer (MS/EN) | Working | Malay + English keywords |
| Parser | Working | Expressions, declarations, functions, types |
| HIR lowering | Working | The **sole** execution path (historical provenance: v1.21 AST codegen retired) |
| Semantic analysis | Working | Name/scope/type checks, FFI-unsafe gate |
| Type checker | Working | `I32/I64/F32/F64/Bool` + full fixed-width ints `I8/I16/I32/I64/U8/U16/U32/U64` (wrap at every boundary) |
| LLVM codegen → native | Working | Emits `.ll`/`.o` and links a native executable on x86_64 Linux |
| Variables, arithmetic, booleans | Working | See `examples/01a`–`01c` |
| Control flow | Working | if/else (`JIKA…MULA…TAMAT MELAINKAN`), while (`SELAGI`), loop (`ULANG`) + break/continue |
| Functions & recursion | Working | `FUNGSI … MULA … PULANG …; TAMAT` |
| Structs & enums | Working | `BENTUK` construct/field-access, `PILIHAN` variants |
| Compile-time capability check | Working | `check` validates each `Service requires` gate against the standard vocabulary |

All of the above are exercised by the test suite and by the check-gated programs
in `examples/`.

## Not yet built (honest)

These are **runtime-profile** work per the doctrine — labelled, not pretended:

- Actor runtime (thread pool / mailbox scheduler) — types parse, no execution
- Channels / async / sandboxed I/O
- Runtime capability **enforcement** (compile-time validation works; runtime gate does not)
- Capability provider-topology (`docs/architecture/capability-topology.md`)
- WASM linking — emits a `wasm32` object, but final `.wasm` needs `wasm-ld` (not bundled)
- Freestanding x86_64 **boots in QEMU** (multiboot1 -> long-mode -> serial -> clean exit); runtime in shared crate `logicodex-os`. All 4 freestanding gaps closed: IDT-256+32 handlers, panic->UART, SSE2, MMIO codegen complete (`ZON_PERKAKASAN` + `HW reg: ty = ALAMAT n;` -> volatile load/store to inttoptr(address)). **End-to-end `.ldx` -> bootable kernel works:** a `.ldx` program compiles to a freestanding object, links with the `logicodex-os` runtime, and boots in QEMU running its own code (CI-guarded via `make boot-e2e`; tested with structs, recursion, functions, if/else, nested loops). Deferred: full crt0 (zero-BSS/.data, trigger-based)
- Network reactor / sharded runtime (`src/net` is not compiled)
- Raylib FFI is not wired into the HIR path
- Float literals (`3.14`) do not yet parse (`.` is field-access); `^` (xor) has no token

## Testing

`cargo test` compiles and runs green: **229 passing, 0 failing,
3 ignored** (g1/g10/g14 string-test crt0, deferred — see #3; the live kernel uses a multiboot _start). Two drift-resistant phase
gates guard behaviour via the real binary:

- `tests/e2e_pipeline.rs` — compiles/checks fixtures through the CLI
- `shipped_examples_pass_semantic_check` — every `examples/*.ldx` must pass `check`

---

## Maturity matrix

Tiers: **FULL** (working, tested) · **PARTIAL** (works for some cases, gaps) · **SKELETON** (types/triple only).

| Capability | Tier |
|---|---|
| Bilingual lexer, parser, AST | FULL |
| Type checking + fixed-width ints (I8–U64) | FULL |
| HIR lowering (sole execution path) | FULL |
| Compiler pipeline → native exe | FULL |
| Bilingual diagnostics | FULL |
| Benchmark framework, validator tiering | FULL |
| Compile-time capability vocabulary check | PARTIAL (runtime + provider-topology pending) |
| Actor model | PARTIAL (types + semantics; no runtime) |
| Sharded runtime / network reactor | PARTIAL (`src/net` not compiled) |
| WASM backend | PARTIAL (emits object; no linker) |
| Freestanding x86_64 | PARTIAL (boots in QEMU; **end-to-end `.ldx`->kernel proven + CI-guarded**; runtime in `logicodex-os`; 4 gaps closed incl. full MMIO; crt0 tests g1/g10/g14 deferred) |
| Raylib FFI | PARTIAL (not wired to HIR) |
| CI/CD | ACTIVE (full suite and freeze-exit stability evidence accepted) |
| Deterministic execution | SKELETON |
| Freestanding aarch64 / riscv64 | SKELETON |
| Self-hosting, package manager, LSP | SKELETON |

Current authority and work sequence:
`docs/architecture/current-authority.md`. Long-horizon phase detail and
historical issue links remain in `ROADMAP_v2.md`.

---

## Quick start

```bash
# Build the compiler (the v1_30 feature is required)
cargo build

# Semantic-check a program
./target/debug/logicodex check examples/01a_variables.ldx

# Compile to native and run
./target/debug/logicodex compile --emit-ir examples/00_sanity.ldx
./examples/00_sanity        # prints 42
```

### Hello (current syntax)
PAPAR 42;
A slightly larger, verified program (Malay canonical):
FUNGSI tambah(a: I32, b: I32) -> I32 MULA
PULANG a + b;
TAMAT
BINA hasil: I32 = tambah(3, 4);
PAPAR hasil;

Statements end with `;`. Blocks use `MULA … TAMAT` (or `{ … }`). See `examples/`
for one verified program per language feature — these are kept correct by the
phase-gate test.

### Targets

`compile --target <native|host|freestanding|wasm>` (default `native`).
`native` produces a runnable executable; `freestanding` emits an object that the
`logicodex-os` runtime links into a bootable x86_64 kernel (see `make boot-e2e`);
`wasm` emits an object needing external linking. Targets sit under the runtime
profiles described in the doctrine.

```bash
# Compile a .ldx program to a freestanding object, link it into the kernel,
# and boot it in QEMU (asserts the program's own serial output):
make boot-e2e        # examples/freestanding/showcase.ldx -> 10 20 55 17 36
make boot-evidence   # examples/freestanding/minimal.ldx  -> clean boot
```

---

## Documentation map

| Document | Purpose |
|---|---|
| [`docs/architecture/current-authority.md`](docs/architecture/current-authority.md) | Single entry point for current invariants, debt disposition, and work sequence |
| [`docs/DOCUMENTATION_POLICY.md`](docs/DOCUMENTATION_POLICY.md) | Doc tiers + the documentation phase gate |
| [`docs/architecture/runtime-doctrine.md`](docs/architecture/runtime-doctrine.md) | Zero-runtime core + optional profiles |
| [`docs/architecture/hir-decision.md`](docs/architecture/hir-decision.md) | Why HIR is the sole execution path |
| [`docs/architecture/cpb-self-hosting-runway.md`](docs/architecture/cpb-self-hosting-runway.md) | CPB runway toward self-hosting |
| [`docs/architecture/compiler-subset.md`](docs/architecture/compiler-subset.md) | Minimum compiler subset for self-hosting runway |
| [`docs/architecture/stdlib-core-design-doctrine.md`](docs/architecture/stdlib-core-design-doctrine.md) | Modern, legacy-aware stdlib/core design doctrine |
| [`docs/architecture/capability-topology.md`](docs/architecture/capability-topology.md) | What capability checking does / defers |
| [`ROADMAP_v2.md`](ROADMAP_v2.md) | Long-horizon phases and maturity history; not current work-sequence authority |
| [`CHANGELOG.md`](CHANGELOG.md) | Version history |
| [`examples/`](examples/) | One verified program per feature |

> Other documents (`SPECIFICATION.md`, the `docs/guide` book, the
> `docs/white-paper` book, grammar/syntax analyses) are mid-repair and may still
> reflect older syntax. They are being brought current under the documentation
> phase gate; treat `examples/` and this README as the syntax source of truth
> until then.

---

## Governance

Phase-gated development: capabilities do not advance without review against
documented criteria. **v0.46.0-alpha is a research prototype** — suitable for
language-design feedback and compiler research, **not** for production or
security-critical workloads.

---

## License

Logicodex is licensed under the **Mozilla Public License 2.0 (MPL-2.0)**.

> This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
> If a copy of the MPL was not distributed with this file, You can obtain one at
> https://mozilla.org/MPL/2.0/.

Versions prior to the licensing update were distributed under MIT OR Apache-2.0;
the previous license files are retained for historical reference but no longer
apply to current or future distributions.

SPDX-License-Identifier: MPL-2.0

© 2025 Mohamad Supardi Abdul (mymsastudio@gmail.com). All rights reserved.

## Current CPB foundation status

Logicodex is moving from aspirational stdlib APIs to evidence-backed,
contract-verified foundations.

Trusted contract-backed core modules currently include:

- `core.assert`
- `core.math`
- `core.bits`
- `core.bool`
- `core.compare`
- `core.range`
- `core.prelude`
- `core.text`
- `core.option`
- `core.result`

Important scope limits:

- `core.prelude` is explicit-import only, not a magic auto-prelude.
- `core.text` proves empty/non-empty text helpers, not full string manipulation.
- `core.option` is currently `Option<I64>` focused.
- `core.result` is currently `Result<I64, I64>` focused.
- Collections are `CompilerFoundationPartial`: fixed local arrays are proven,
  but `core.array`, `core.slice`, `Vec`, `List`, and heap collections are not
  production-ready.
- High-level IO remains deferred until callable IO, path handling, `IoError`,
  and runtime capability/profile policy are designed.

See:

- `docs/stdlib/core-trust-state.md`
- `docs/architecture/cpb-next-roadmap-blockers.md`
