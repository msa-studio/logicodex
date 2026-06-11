# Runtime Doctrine: Zero Runtime Core, Optional Runtime Profiles

**Status:** Adopted (architecture doctrine)
**Scope:** Defines how runtime is layered in Logicodex. Supersedes the implicit
"zero runtime mediation" framing by making it one tier of a tiered model.

## Doctrine

> **Zero Runtime Core. Optional Runtime Profiles. Portable Semantic IR.**

Logicodex is **zero-runtime by default, runtime-capable by profile.** The core
language carries no mandatory runtime: source lowers to a canonical IR that can be
emitted to native/object/WASM with nothing linked in. Runtime exists only as
**opt-in target profiles** for features that genuinely require a scheduler, state,
or a sandbox.

This preserves Logicodex's primary identity as a *meaning authority / portable
intermediate language* while allowing real runtime where it is unavoidable.

## Why tiered (not a binary choice)

Treating "zero-runtime vs runtime" as one global switch is wrong for Logicodex.
Zero-runtime is the right identity for a systems/IR language; but WASM sandboxing,
actors, channels, and plugin hosting genuinely need runtime support. The mature
answer is to keep the **core** runtime-free and attach runtime as a **profile**
on top of a backend, chosen per build.

## The dividing rule

- **If a feature can compile away -> it belongs to the zero-runtime Core.**
- **If a feature needs a scheduler, persistent state, or a sandbox -> it belongs
  to a Runtime Profile.**

### Core (zero-runtime) — compiles away

- Types, fixed-width integers, structs, enums
- Functions, calls, recursion
- Deterministic control flow (if / while / loop / break / continue)
- Pattern matching (lowered to branches)
- The FFI boundary (declaration + linkage; calling C is link-time, not runtime)
- **Static capability validation** (compile-time gate-vocabulary checks)

Everything implemented in the engine so far sits on this side of the line — the
core has grown without acquiring a mandatory runtime, which is exactly the
intended trajectory.

### Runtime (profile) — needs scheduler / state / sandbox

- Actors and message channels
- Async
- Sandboxed I/O
- GC / reference counting (if ever introduced)
- Dynamic modules / hot reload
- Telemetry, plugin host
- **Runtime capability enforcement** (a gate that traps unauthorized access)

## Architecture
Logicodex Surface Syntax
|
Canonical AST
|
Canonical HIR        <- portable semantic IR (the meaning authority)
|
Backend Profiles
|- native-zero      (native/object, no runtime linked)
|- freestanding     (bare-metal targets, no host runtime)
|- wasm             (portable sandboxed execution)
|- lxdge-runtime    (managed host runtime)
|- actor-runtime    (scheduler + channels)

The core stops at HIR. A **profile** is selected per build and decides which
backend target is used *and* which runtime, if any, is linked.

## Profiles vs the existing `--target` flag

`--target` already selects an LLVM target (`native`, `wasm`,
`freestanding-*`). A **profile is the higher-level selector**: it pins a target
*and* a runtime-linkage policy. Profiles are not a replacement for `--target`;
they sit above it.

| Profile          | Target              | Runtime linked            |
|------------------|---------------------|---------------------------|
| `native-zero`    | native/object       | none (pure core)          |
| `freestanding`   | freestanding-*      | none (bare metal)         |
| `wasm-min`       | wasm                | minimal (sandbox shim)    |
| `actor-runtime`  | native              | scheduler + channels      |
| `lxdge-managed`  | native/wasm         | managed host runtime      |

The default profile is `native-zero` — i.e. the core, runtime-free.

## How this resolves the capability tension (Issue #07)

The earlier friction ("zero-runtime mediation" vs the P3-D3 runtime gate) dissolves
under tiering:

- **Static capability validation = Core.** Already active: `check` validates each
  `Service` `requires` against the standard gate vocabulary at compile time.
- **Runtime capability enforcement = Runtime Profile.** A trapping gate is a
  property of a runtime profile (e.g. `lxdge-managed`), not of the core. It is not
  a contradiction of zero-runtime; it is simply out of the core tier.

## Status and honesty

- The **Core** is real and working (native/object emission, all the features
  listed above).
- The **profile layer is doctrine, not yet implemented.** `--target wasm` and the
  freestanding targets exist in the pipeline, but the profile selector, runtime
  shims, and managed/actor runtimes are not built. This document fixes the
  *intended* shape; it does not claim the profiles exist.

## Consequences for the roadmap

- Runtime work (actor runtime, WASM runtime #10, capability runtime #07 P3-D3) is
  reframed as **profile** work, not core work — and stays optional.
- The core must never acquire a mandatory runtime dependency. Any feature that
  cannot compile away must be introduced behind a profile.
- A future change may add a `--profile` selector; until then, `--target` plus the
  implicit `native-zero` default is the operative model.
