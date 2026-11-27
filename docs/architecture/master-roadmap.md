# Logicodex Master Roadmap

Status: Active long-term direction
Owner: Mohamad Supardi Abdul (Chief Architect, MSA Studio)
Recorded: 2026-11-27
Baseline: `origin/main` at `9a16258` (merge of PR #105)
Scope: Logicodex Community compiler, standard library, tooling and runtime profiles

## 1. Purpose

This is the single long-term development reference for Logicodex. It unifies
four planning sources that previously ran on separate axes:

- the CPB runway (`cpb-self-hosting-runway.md`);
- the P0–P2 blocker inventory (`cpb-next-roadmap-blockers.md`);
- the Language Implementation Model (LIM-1 to LIM-5);
- the historical phase baseline (`ROADMAP_v2.md`, Phases 1–5).

Those sources keep their detail, but they no longer define direction or order.
When they disagree with this document on stage placement, this document wins.

This document is **not** an execution sequence. The only active execution
sequence lives in [`current-authority.md`](current-authority.md). This roadmap
says where Logicodex is going and in which stage each capability belongs; it
does not authorize any agent or contributor to start future-stage work.

## 2. Authority hierarchy

| Document | Authority |
|---|---|
| `current-authority.md` | The only active execution sequence and current invariants |
| `master-roadmap.md` (this file) | Owner-approved long-term direction and stage placement |
| Milestone RFCs | Architecture and implementation authorization for one stage or sub-milestone |
| `cpb-self-hosting-runway.md` | Detailed CPB capability requirements |
| `cpb-next-roadmap-blockers.md` | P0–P2 blocker inventory |
| `ROADMAP_v2.md` | Historical phase baseline, provenance only |
| `Cargo.toml` | Release version authority |

Rules:

1. Placement in this roadmap is not authorization. Each stage starts only when
   `current-authority.md` names it and its RFC is approved.
2. A stage may not be declared closed from a local branch, an unmerged pull
   request, or a demo alone (see section 8).
3. Enterprise / Assurance work is outside this roadmap and remains in its
   separate private repository. No Community stage depends on it.

## 3. Current position

Snapshot at the baseline commit. `current-authority.md` remains the source of
truth for status; this section only orients readers.

- **Toolchain:** LLVM 23.1.6 and Rust 1.100.0 across CI, release workflow,
  Makefile and contributor docs (PRs #102, #103). Version-reference hygiene
  fails closed on any other LLVM major or Rust release in toolchain context.
- **Stage A, CPB-2S item 1** — Callable and Function Type Closure:
  implementation merged (callable signature freeze; callable semantic consumer
  migration, PR #104).
- **Stage A, CPB-2S item 2** — Assignment and Return Compatibility:
  implementation merged (PR #105).
- **Stage A, CPB-2S item 3** — Diagnostic End-to-End: next in sequence.
- **Formal closure:** items 1 and 2 are not yet recorded as closed in
  `current-authority.md`; Stage A closes only under section 8.
- **Evidence at baseline:** `cargo test --locked` 609 passed, 0 failed,
  3 ignored; quick integrity gate passes.
- Superseded and closed unmerged: PR #100 (R2 research matrix) and PR #101
  (content delivered through #102).

## 4. Invariants

These hold at every stage and cannot be traded for schedule:

- **One language, one meaning.** `semantic_gate` is the sole Meaning Authority.
  No interpreter, VM or backend may define semantics of its own.
- **Canonical HIR.** Every execution implementation conforms to the same HIR
  contracts.
- **Freestanding-first.** No program is forced to link a runtime. Runtime
  features are opt-in profiles.
- **Bilingual identity.** Every new user-facing syntax form has a Malay and an
  English spelling in `dict/core_map.json`, and every new diagnostic is
  emitted in both languages.
- **Fail closed.** Unsupported constructs produce structured diagnostics,
  never a silent fallback value or a plausible-looking binary.
- **Default-deny FFI.** External symbols require explicit capability policy.
- **Native-first.** Use Logicodex to build Logicodex where practical; the Rust
  host remains for infrastructure not yet expressible natively.
- **Evidence-driven.** A capability is complete only with execution and
  qualification evidence.
- **Anti-bloat.** No abstraction, framework or dependency without a present
  need.
- **LLVM is the sole production backend** until Stage H is separately
  authorized.

## 5. Identifier mapping

### 5.1 CPB-2 disambiguation

Two documents previously used "CPB-2" for different scopes. From this document
onward:

| Identifier | Scope | Defined in |
|---|---|---|
| **CPB-2S** — Semantic Closure | Callable and function type closure; assignment and return compatibility; diagnostic end-to-end | `current-authority.md` sequence |
| **CPB-2L** — Library Closure | Bootstrap stdlib slice: text, option, result, array/slice, path, file, io | `cpb-self-hosting-runway.md` |

CPB-2 is closed only when both CPB-2S and CPB-2L are closed. Renaming the
items inside the `current-authority.md` sequence requires the normal authority
update process; until then, its three items are CPB-2S.

### 5.2 Legacy identifiers to stages

| Legacy identifier | Stage |
|---|---|
| SSM-D4, CPB-2S, P0 (active pipeline scope) | A |
| CPB-2L, CPB-3, P1.1–P1.4, numeric foundations | B |
| P2.1–P2.6 (FFI-C1 to FFI-C4, `ldx-pkg` MVP), CPB-4 | C |
| CPB-5, LIM-1, LIM-2, ROADMAP_v2 Phase 2 (formatter, LSP) | D |
| ROADMAP_v2 Phase 3 (runtime), networking, WASM | E |
| ROADMAP_v2 Phase 4 (platforms) | F |
| LIM-5, ROADMAP_v2 Phase 5 (production) | G |
| LIM-3 (LDB bytecode / VM), LIM-4 (native backend) | H |

## 6. Stages

| Stage | Name | Maturity | Version convention |
|---|---|---|---|
| A | Semantic Foundation | Alpha 1 | `0.x.y-alpha.n` |
| B | Usable Core | Alpha 2 | `0.x.y-alpha.n` |
| C | C Ecosystem Integration | **Beta** | `0.x.y-beta.n` |
| D | Language Maturity & Bootstrap | Self-Hosting Preview | `0.x.y-preview.n` |
| E | Runtime & Web | Pre-stable | `0.x.y` (minor bump per capability) |
| F | Platform Qualification | Pre-stable | `0.x.y` (minor bump per capability) |
| G | Production Community Compiler | RC, then Stable | `1.0.0-rc.n`, then `1.0.0` |
| H | Backend Independence | Post-stable | `1.x`; `2.0.0` only for breaking API change |

Each stage below lists goal, scope, entry gate, exit proof and non-goals. The
exit proof is the functional demonstration; closure also requires the full
formula in section 8.

### Stage A — Semantic Foundation *(current)*

- **Goal:** the production compiler accepts valid programs and rejects invalid
  ones without silent fallback.
- **Scope:** SSM-D4 integration; CPB-2S (the three `current-authority.md`
  items); P0 limited to the active pipeline — callable, assignment/return,
  diagnostics, and removal of unsafe fallback-to-zero in lowering and codegen.
- **Entry gate:** already active. Items 1 and 2 have merged implementations;
  item 3 remains (see section 3).
- **Exit proof:** a negative/positive conformance suite over the active
  pipeline in which every unsupported construct fails with a coded bilingual
  diagnostic and no fallback path remains.
- **Non-goals:** activating dormant `FutureReserved` type-checker surfaces;
  broad type-checker cleanup; new language features.

### Stage B — Usable Core

- **Goal:** real command-line programs can be written in Logicodex.
- **Scope:**
  - CPB-2L: `core.array`/slice Stage 1, `core.text` Stage 2 (equality, length,
    concat, slicing, UTF-8 policy), `std.path`, `std.file`, `std.io` with an
    `IoError` model.
  - Numeric foundations: `F32`/`F64` literals and arithmetic, C-width integer
    types, typed integer literals, explicit cast syntax, numeric conversion
    rules.
  - **Minimal generics:** monomorphized generic enums and functions only, so
    `Option<T>` and `Result<T, E>` replace I64-only foundations. No traits.
  - **Memory model RFC** covering Stages B–D, implemented here only for string
    ownership, allocation policy and array bounds. The model is Logicodex's
    own design, not an import of another language's borrow checker.
  - Text output from `PAPAR`; dictionary embedded in the binary so the
    compiler runs from any directory.
  - Native test framework, first slice.
  - CPB-3 compiler API freeze at the end of the stage.
  - P0 scope: numeric, string and array/slice type checking.
- **Entry gate:** Stage A closed.
- **Exit proof:** a native `.ldx` CLI program that reads a file, processes text
  and prints correct text output.
- **Non-goals:** pointers, FFI beyond current behavior, traits, heap
  collections, runtime profiles.
- **Parallel work:** after Stage A, track-preparation work that does not alter
  production architecture (RFCs, design records, test fixtures) may start with
  specific owner approval. Full parallel implementation authority begins only
  after Stage B.

### Stage C — C Ecosystem Integration → **Beta**

- **Goal:** Logicodex applications can use mature C libraries.
- **Platform scope:** Linux x86_64 (System V ABI) only. Other ABIs belong to
  Stage F.
- **Scope:**
  - FFI gates in order: **FFI-C1** libm (primitive numeric ABI) → **FFI-C2**
    custom C library (pointers, `char*`, null diagnostics) → **FFI-C3** sqlite3
    (opaque pointers, resource ownership, error strings) → **FFI-C4** Raylib
    (struct layout, by-value struct ABI, minimal draw loop).
  - Pointer types, nullability, function pointers / callbacks.
  - Memory model: pointer lifetime, borrowed versus owned FFI resources.
  - **`lod bind` MVP:** generates reviewable `.ldx` binding source from C
    headers with explicit target triple, include paths, type mapping, layout
    and calling convention. Unsupported declarations are rejected with a
    diagnostic, never approximated.
  - `ldx-pkg` local MVP (local metadata, path dependencies, build/check,
    contract verification; no network registry).
  - **Minimum C export:** plain functions with scalar and pointer parameters
    callable from C.
  - CPB-4 compiler-shaped programs.
  - P0 scope: pointer, FFI and struct/enum layout validation.
- **Entry gate:** Stage B closed; FFI RFC approved.
- **Exit proof:** a desktop Logicodex application using SQLite and Raylib,
  built through package tooling and generated bindings.
- **Non-goals:** universal C importer; C++ interop; static/shared library
  output; Windows or macOS ABIs; network package registry.

### Stage D — Language Maturity & Bootstrap → Self-Hosting Preview

Sub-milestones close independently, in this order unless an RFC reorders them:

- **D1** CPB-5 first self-hosting loop. Not blocked by traits.
- **D2** Full generic types and functions.
- **D3** Generic collections (`Vec<T>`, `Map<K, V>`), allocators and the
  complete memory contracts.
- **D4** Traits / interface abstraction.
- **D5** HIR interpreter (LIM-2) as reference evaluator: compiler subset
  execution, differential testing against LLVM output, compile-time evaluation
  under approved contracts. A smaller reference subset (LIM-2a: numeric
  evaluation, control flow, pure functions) may be built during B or C, but it
  is optional there and never a Beta gate.
- **D6** `ldx-fmt` and LSP foundation with bilingual diagnostics.
- Across D: LIM-1 — formal semantics, numeric semantics, ABI and data layout
  contracts, undefined-behavior boundaries. P0 scope: generics, traits and
  ownership diagnostics.
- **Entry gate:** Stage C closed.
- **Exit proof:** a native compiler component that uses generic collections,
  with interpreter and LLVM outputs matching on its conformance suite.
- **Non-goals:** full self-hosting; alternative production backends.

### Stage E — Runtime & Web

- **Goal:** concurrent services and web targets, without making runtime
  mandatory.
- **Scope:**
  - Optional concurrent runtime profile: actors, M:N scheduler, channels,
    supervision, deterministic execution profile, runtime capability
    enforcement.
  - Networking (TCP, UDP, HTTP) and JSON serialization.
  - **WASM/WASI** profile: CLI and compute modules, tested in Wasmtime.
  - **WASM/Browser** profile: Raylib through Emscripten, including JavaScript
    glue, canvas and event-loop adaptation, with application logic kept in
    shared `.ldx` source.
  - Static and shared library generation; extended C export.
- **Entry gate:** Stage D closed (D1–D4 at minimum).
- **Exit proof:** a Raylib application running on desktop and in the browser
  from shared source, talking to an HTTP backend written in Logicodex.
- **Non-goals:** runtime linked into freestanding or hosted programs that do
  not request it.

### Stage F — Platform Qualification

- **Goal:** the same application and kernel qualify across operating systems
  and CPU architectures.
- **Scope:** explicit target-triple support matrix with support tiers; ABI
  lowering for Win64 and AAPCS64; Linux, Windows and macOS hosted targets;
  x86-64, ARM64 and RISC-V freestanding targets; debug information and
  debugger support; multi-architecture CI.
- **Entry gate:** Stage E closed.
- **Exit proof:** one common application passes on three operating systems,
  and one common freestanding kernel boots on three CPU architectures in QEMU.
- **Non-goals:** platforms outside the published support matrix.

### Stage G — Production Community Compiler → Stable 1.0

- **Goal:** a production-qualified, self-hosting Community release.
- **Scope:** LIM-5 full self-hosting with a three-stage bootstrap; package
  registry; production LSP; complete language, stdlib and tooling
  documentation; public API and compatibility policy; security review;
  release qualification suite.
- **Entry gate:** Stage F closed; feature scope frozen before `1.0.0-rc.1`.
- **Exit proof:** reproducible self-hosting bootstrap (stage 2 matches stage 1
  for equivalent input, target and deterministic build environment) and a
  passing production qualification suite.
- **Non-goals:** alternative backends; identical binaries across different
  host operating systems.

### Stage H — Backend Independence *(post-stable, separately authorized)*

- **Scope:** LIM-3 Logicodex bytecode (LDB), verifier and VM with sandboxed,
  deterministic execution; LIM-4 independent native backend. LLVM remains the
  optimized production backend.
- **Entry gate:** Stage G closed and a dedicated architecture RFC approved.
- **Exit proof:** a common conformance suite producing matching observable
  behavior through LLVM, the VM and the native backend.
- **Non-goals:** removing LLVM. Stage H does not block Logicodex being
  production-grade.

## 7. Cross-stage tracks

| Track | A | B | C | D | E | F | G |
|---|---|---|---|---|---|---|---|
| P0 diagnostics / type checking | active pipeline | numeric, string, array | pointer, FFI, layout | generics, traits, ownership | runtime | platform | qualification |
| Memory model | — | RFC; strings, allocation, bounds | pointer lifetime, FFI ownership | full model, allocators | runtime ownership | — | stability |
| Bilingual syntax + diagnostics | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Native test framework | — | first slice | library testing | generics-aware | — | — | stable |
| Interop direction | — | — | call C; minimum export | — | libraries out | — | stable ABI policy |
| Performance and security regression | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Trust-state and documentation hygiene | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |

A capability is promoted only with complete diagnostics for its own scope.
Diagnostics are not built for features that do not exist yet.

## 8. Milestone closure

A stage or sub-milestone closes only when all of the following are present on
the target branch:

1. **Functional demonstration** — a real program written in `.ldx`, not a
   hardcoded or special-cased path.
2. **Conformance evidence** — positive, negative and boundary tests.
3. **Regression evidence** — every earlier capability still passes.
4. **Architecture integrity** — no ownership drift and no bypass of
   `semantic_gate`.
5. **Bilingual evidence** — Malay and English syntax and diagnostics for every
   new surface.
6. **Reproducibility** — the evidence regenerates from a clean checkout.
7. **Release qualification** — platform and artifact verification for the
   stage's declared scope.
8. **Owner authorization** — evidence reviewed and closure approved by the
   owner, recorded in `CHANGELOG.md` and `current-authority.md`.

## 9. Target capability by stage

✓ means the capability is a qualified deliverable once the stage closes. It is
not a statement of current implementation status.

| Capability | A | B | C | D | E | F | G | H |
|---|---|---|---|---|---|---|---|---|
| Fail-closed semantic foundation | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Practical CLI, file and text processing | — | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Float and C-width numerics | — | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `Option<T>` / `Result<T, E>` | — | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Calling C libraries (Linux x86_64) | — | — | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Desktop graphics (Raylib) and SQLite apps | — | — | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Generated C bindings, local packages | — | — | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Generic collections and traits | — | — | — | ✓ | ✓ | ✓ | ✓ | ✓ |
| First self-hosting loop | — | — | — | ✓ | ✓ | ✓ | ✓ | ✓ |
| Formatter and LSP | — | — | — | ✓ | ✓ | ✓ | ✓ | ✓ |
| Concurrent runtime, networking | — | — | — | — | ✓ | ✓ | ✓ | ✓ |
| WASI and browser applications | — | — | — | — | ✓ | ✓ | ✓ | ✓ |
| Library output for other languages | — | — | — | — | ✓ | ✓ | ✓ | ✓ |
| Windows, macOS, ARM64, RISC-V | — | — | — | — | — | ✓ | ✓ | ✓ |
| Full self-hosting, package registry | — | — | — | — | — | — | ✓ | ✓ |
| Independent VM and native backend | — | — | — | — | — | — | — | ✓ |

## 10. Recorded decisions

1. Eight stages, A–H, each closed by section 8.
2. C interoperability and Raylib desktop define Beta (Stage C).
3. CPB-2 is split into CPB-2S (Stage A) and CPB-2L (Stage B).
4. Minimal monomorphized generics for enums and functions enter at Stage B;
   full generics and traits at D2–D4.
5. The memory model is designed once, in a Stage B RFC, and implemented in
   forward-compatible slices across B–D.
6. The HIR interpreter is a reference evaluator. Its reduced subset is
   optional before Stage D and never gates Beta.
7. Beta platform scope is Linux x86_64 System V only.
8. Bilingual syntax and diagnostics are a closure gate at every stage.
9. Runtime is an opt-in profile; WASM is split into WASI and Browser profiles.
10. Stable Community is `1.0.0`. VM and native backend come after it and do
    not require `2.0.0` unless the public API breaks.

## 11. Change control

- Changes to stage placement, stage order or recorded decisions require owner
  approval and an update to this file, `CHANGELOG.md`, and any affected
  subordinate record in the same change.
- Each stage needs its own RFC before implementation, stating entry gate, exit
  gate, dependencies, non-goals and authorization boundary.
- Detailed implementation plans belong in stage RFCs, not here.
