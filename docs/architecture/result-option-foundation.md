# Result / Option Foundation

Status: minimum foundation PROVEN. The compiler now supports the typed
`Result<I64, I64>` and `Option<I64>` slice end to end, and `core.option` /
`core.result` ship as Stage 1 contract-backed stdlib modules. Generic
`Result<T, E>` / `Option<T>`, helper combinators, and the richer diagnostic
intelligence layer remain deferred (see "Out of Scope" and the LDX-DIP note
below).

This document defines the compiler and stdlib foundation required before
continuing deeper CPB Phase 1 stdlib expansion.

## Current Status (Proven)

The following is implemented and proven by live (non-ignored) e2e tests in
`tests/compiler_result_option_foundation.rs` and the stdlib acceptance tests in
`tests/stdlib_core_result_option.rs`:

- enum declarations lower to deterministic `i64` variant tags.
- `Result<I64, I64>` can be returned from functions and constructed via
  `Ok(I64)` / `Err(I64)`.
- `Option<I64>` can be returned from functions and constructed via `Some(I64)`
  / `None`.
- `match` destructures `Ok(v)` / `Err(e)` and `Some(v)` / `None`, binding the
  payload.
- LLVM module verification passes and compiled programs run with the expected
  stdout.

### Transitional scalar encoding (compiler-foundation ABI)

The proven slice uses a single `i64` with a low-bit discriminant, NOT a final
tagged-union layout. This is a documented compiler-foundation step, not final
`Result<T, E>` / `Option<T>` semantics:

- `Ok(v)` and `Some(v)` lower to `(v << 1) | 1` (low bit tag = 1).
- `Err(e)` lowers to `e << 1` (low bit tag = 0).
- `None` lowers to `0`.
- `match` lowers to an if-chain (`lower_match_to_if`): tag = `value & 1`,
  payload = `value >> 1`, and the bound payload is typed `i64`.

Honest limitations of this encoding:

- The payload is effectively 63-bit because of the shift; a payload that uses
  the top bit will not round-trip. Wider payloads wait for the real tagged
  layout.
- `match` is currently lowered to an if-chain, so there is no exhaustiveness
  checking yet.
- The semantic type identity (`TypeKind::Result { ok, err }` /
  `TypeKind::Option { some }`) is preserved in the type registry so future
  diagnostics and the real layout can recover the original meaning even though
  the ABI is currently scalar.

## Why This Exists

Logicodex has early parser/AST support for pieces of:

- enum declarations
- enum variants
- `Result<T, E>`
- `Ok(...)`
- `Err(...)`
- `match`

However, the current pipeline is not yet contract-proven end to end across:

- parser
- semantic analysis
- HIR/lowering
- type layout
- codegen
- LLVM verification
- runtime behaviour

Therefore `core.result` must not be migrated as a fake status-code bridge under
the name `core.result`.

## Original Blockers (Resolved)

These were the probe results that originally blocked the foundation. They are
kept as a record; each is now resolved by the proven slice described above:

- `Ok(5)` and `Err(9)` compiled only as smoke expressions. RESOLVED — they now
  lower as typed constructors.
- Returning `Result<I64, I64>` previously failed LLVM module verification.
  RESOLVED — verification passes.
- `match` with `return` inside `Ok(...)` / `Err(...)` arms previously failed
  parse. RESOLVED — match destructuring lowers via `lower_match_to_if`.
- enum declaration syntax needed canonical confirmation. RESOLVED — enum
  variants resolve to deterministic `i64` tags.
- enum variant layout/tag output was not contract-proven. RESOLVED — proven by
  `enum_variants_have_deterministic_i64_tags`.
- the legacy `lib/core/result.ldx` generic sketch was not suitable for
  ContractVerified status. RESOLVED — replaced by the Stage 1 `Result<I64, I64>`
  contract-backed module.

## Architectural Decision

Logicodex will use a dual-layer long-term model:

### Low-level status/error layer

A future module such as `core.error` or `core.status` may represent low-level
status codes for syscall, C ABI, FFI, runtime, file descriptors, hardware, and
loader boundaries.

This layer is not a replacement for typed `Result<T, E>`.

### High-level typed data layer

`core.option` and `core.result` must represent typed Option/Result semantics
once compiler support is proven.

They must not be faked using status-code helpers.

## Branch Rule

This branch must prioritize compiler foundation before adding more CPB stdlib
modules.

`core.option` and `core.result` are now Stage 1 ContractVerified for the
`Option<I64>` / `Result<I64, I64>` slice, because the matching compiler
behaviour is proven by tests. Any expansion beyond this slice (generic
payloads, combinators, error payload types) must again be proven by compiler
support, contracts, and tests before it is added as ContractVerified surface.

A future `core.error` or `core.status` module must not be faked using
status-code helpers wearing the `core.result` name.

## Minimum Foundation Target

The first stable target is intentionally small:

- enum declaration parses in canonical expert syntax.
- enum variants resolve to deterministic tags.
- `Result<I64, I64>` can be returned from functions.
- `Ok(I64)` and `Err(I64)` construct typed Result values.
- `match` can destructure `Ok(v)` and `Err(e)`.
- `Option<I64>` can be returned from functions.
- `Some(I64)` and `None` construct typed Option values.
- `match` can destructure `Some(v)` and `None`.
- LLVM verifier passes.
- compiled programs run with expected output.

## Out of Scope for First Foundation Pass

The first pass does not need to support:

- arbitrary generic payloads for all types
- nested `Result<Option<T>, E>`
- `String` payloads
- heap allocation
- slices or arrays as payloads
- `map`, `and_then`, `expect`, or panic-based helpers
- full exhaustiveness diagnostics

Those can be added after the I64 payload foundation is proven.

## Test Strategy

The branch carries executable compiler tests in:

- `tests/compiler_result_option_foundation.rs`

These tests defined the intended behaviour and were unignored one by one as the
compiler foundation became real. They are now live (non-ignored) and green for
the proven slice; they are the visible contract for enum tags,
`Result<I64, I64>`, `Option<I64>`, and match destructuring.

Ignored tests were never a way to hide failure. They were a visible backlog of
contract targets for this branch, now discharged for the I64 slice.

Current compiler-foundation tests validate compile success, stderr cleanliness,
and stdout behaviour. Process exit-code normalization is tracked separately
because generated executables can currently emit correct output while returning
a nonzero process status.

Result<I64, I64> return payload foundation is allowed as an intermediate
compiler step: `Ok(x)` and `Err(x)` may lower to the `i64` payload while full
tagged layout and match destructuring remain pending. This must not be presented
as complete `Result<T, E>` semantics.

### Debuggability and smart-compiler constraints

Result and Option foundation work must preserve semantic truth for the future
Logicodex Diagnostic Intelligence Pipeline (LDX-DIP). Compiler changes must not
only make tests pass; they must keep enough structure for causal diagnostics,
AI-queryable debugging, and future safe self-treatment.

The following constraints apply to this branch:

- `Ok`, `Err`, `Some`, and `None` must not become anonymous values once match
  destructuring is introduced.
- Unsupported match patterns must not be silently ignored during lowering.
- Match lowering must preserve which branch was selected and which payload was
  bound, even if the first implementation uses a temporary scalar encoding.
- Transitional encodings must be documented as ABI/compiler-foundation steps,
  not as final language semantics.
- Any future optimization may erase representation details only after diagnostics
  and semantic metadata have had a chance to observe the original meaning.

This keeps Community compiler work aligned with the long-term smart compiler and
Enterprise Assurance direction without implementing Enterprise-only enforcement
inside the public compiler.

## Definition of Done

This branch's minimum foundation target is met. The following are green:

- [x] compiler e2e tests for enum tags
- [x] compiler e2e tests for `Result<I64, I64>`
- [x] compiler e2e tests for `Option<I64>`
- [x] match destructuring tests for Result and Option
- [x] LLVM module verification
- [x] CPB baseline regression (`verify_cpb_self_hosting_runway.sh`)
- [x] `core.option` ContractVerified (Stage 1)
- [x] `core.result` ContractVerified (Stage 1)

Process exit-code normalization remains tracked separately: the foundation
tests assert stdout behaviour, not process exit status.

## Integrity Rule

A green test must represent real behaviour.

Do not weaken tests, rename failures away, or replace typed Result/Option with
status-code helpers just to make the branch green.
