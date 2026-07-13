# CPB Self-Hosting Runway

Status: Active planning document
Scope: Logicodex Community foundation
Depends on: Stdlib Stage 0 contract discipline

## Purpose

This document defines the shortest safe runway from the current Community
compiler foundation toward self-hosting.

Self-hosting must not begin as a large rewrite. It must be reached through
small, contract-backed compiler capability increments.

## Current Baseline

The current stable baseline is:

- HIR is the sole execution path.
- Module/import system exists.
- Stage 0 pure core stdlib contracts are verified.
- Contract metadata and hash evidence exist.
- Legacy core modules are inventoried but not trusted.
- `verify_stdlib_stage0.sh` is the current stdlib gate.

## CPB Meaning

CPB means Community Production Baseline.

For Logicodex, CPB does not mean enterprise assurance. CPB means the Community
compiler has enough stable, tested, contract-backed foundation to support real
programs and eventually compile a compiler subset.

## Self-Hosting Risk

Self-hosting requires more than arithmetic helpers.

Minimum missing foundation:

- prelude baseline
- text/string primitives
- array/slice/list primitives
- option/result/error model
- file/input reading
- path handling
- stable diagnostics
- module/package resolution
- compiler-facing test harness
- compiler subset definition

## Runway Phases

### CPB-0: Contract Discipline

Status: Done.

Proof:

- Stage 0 modules have matching `.ldx` and `.std.toml`.
- Contracts declare exports.
- Run-cases execute.
- Hash evidence is emitted.
- Legacy modules are not accidentally trusted.

### CPB-1: Bootstrap Surface

Goal: define the minimum language and stdlib surface needed by compiler-like
programs.

Deliverables:

- `docs/architecture/compiler-subset.md`
- prelude policy
- text/string module plan
- result/option/error module plan
- file/io boundary plan

No implementation may claim CPB-1 unless the subset is documented and gated.

### CPB-2: Bootstrap Stdlib Slice

Goal: implement only the stdlib modules required for compiler-like programs.

Priority order:

1. `core.prelude`
2. `core.text` or `core.string`
3. `core.option`
4. `core.result`
5. `core.array` or `core.slice`
6. `std.path`
7. `std.file`
8. `std.io`

Each module must follow the Stage 0 contract pattern before it is trusted.

Progress: `core.prelude`, `core.text`, `core.option`, and `core.result` now
ship a Stage 1 contract-backed slice following this pattern. `core.prelude`
is explicit-import only, `core.option` = `Option<I64>`, and `core.result` =
`Result<I64, I64>`. The remaining modules and the generic / wider-payload
surface are still pending.

### CPB-3: Compiler API Freeze Boundary

Goal: freeze the public Rust-side compiler APIs that a future Logicodex compiler
implementation will need to call or mirror.

Minimum boundary:

- lexer input model
- parser entry point
- AST model
- HIR model
- diagnostic model
- module loader model
- codegen handoff model

This is an API boundary, not a self-hosting rewrite.

### CPB-4: Compiler Subset Programs

Goal: write small compiler-shaped programs in Logicodex.

Examples:

- token classifier
- diagnostic formatter
- module path normalizer
- simple AST visitor
- contract metadata reader
- stdlib contract case runner prototype

These programs prove that Logicodex can express compiler work.

### CPB-5: First Self-Hosting Loop

Goal: compile a tiny Logicodex compiler component written in Logicodex using the
Rust compiler host.

First target should be a small component, not the full compiler.

Recommended first component:

- diagnostic formatter
- token classifier
- module path resolver

## Non-Goals

The runway does not include:

- enterprise assurance engine
- LDX-AUD implementation
- AI contract governance implementation
- secret-proof authority gates
- full package manager
- full self-hosted compiler rewrite
- replacing Rust compiler host immediately

## Legacy Rule

Legacy modules must not be repaired ad hoc just to accelerate self-hosting.

If a legacy module is needed by the runway, rebuild or migrate it under the
contract pattern:

- source file
- sidecar contract
- public exports
- run-cases
- focused tests
- verify script coverage
- trust-state update

## Entry Gate for CPB Work

Before any CPB implementation patch:

- branch must be clean or intentionally scoped
- `verify_stdlib_stage0.sh` must pass
- touched module must have a contract plan
- legacy trust state must not be upgraded without proof

## Exit Gate for CPB Foundation

CPB foundation is ready for self-hosting experiments only when:

- compiler subset is documented
- bootstrap stdlib slice is contract-verified
- diagnostics are stable enough for compiler programs
- module/import behavior is regression-tested
- at least three compiler-shaped programs compile and run

### Collections and IO Blocker Status

Block 161 established that CPB Phase 1 Collections and High-level IO are not ready for implementation as normal stdlib modules yet.

Collections are `DeferredBlockedByCompiler`:

- array literal / fixed-array syntax is not supported by the current parser path
- Buffer declaration has no proven initialization path
- slice parameter syntax compiles, but slice construction and round-trip behaviour are not proven

High-level IO is `DeferredBlockedByRuntimeCapability`:

- `PAPAR` is statement-only and not callable
- legacy `core.file` and `core.io_error` do not import cleanly
- `Result<T, IoError>` is not part of the current proven Result foundation

CPB Phase 1 must not fake these APIs. The next valid work is to either design generic compiler/runtime capability support or keep these blockers documented until such support exists.

### Collections compiler foundation update

Status: `CompilerFoundationPartial`.

The CPB Collections blocker is no longer a total compiler blocker. The compiler
now proves the first fixed-local-array foundation:

- fixed array type syntax: `[T; N]`
- array literal syntax: `[a, b, c]`
- index read: `xs[i]`
- index assignment: `xs[i] = v`
- HIR type lowering for fixed arrays
- LLVM local array storage as `[N x i64]`

This does not yet make `core.collections` production-ready. It unlocks the next
stdlib migration step: contract-backed collection helpers can now target a small,
proven fixed-array subset before slices, dynamic buffers, iterators, maps, or
higher-level collection APIs are promoted.

### P1-B3a Fixed-array boundary update

The CPB collections blocker now has a proven compiler-side fixed-array subset,
tracked in `docs/architecture/p1b3-fixed-array-cpb-boundary.md`.

This narrows the blocker but does not complete `core.array` or `core.slice`.
Public stdlib APIs still require the normal `.ldx + .std.toml + tests` contract
path before they can become CPB dependencies.

### P1-B3b Array parameter and return barrier

The CPB runway must not depend on public `core.array` functions yet. Local fixed
arrays are proven, but array function parameters and return values still require
a stable ABI/codegen policy.

See `docs/architecture/p1b3-array-codegen-barrier.md`.

### P1-B4a std.path lexical foundation

The CPB runway may use the narrow `std.path` lexical foundation for
empty/non-empty path checks and simple selection. It must not depend on
filesystem access, normalization, or platform path semantics yet.

See `docs/architecture/std-path-foundation.md`.
