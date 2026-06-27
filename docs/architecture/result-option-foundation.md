# Result / Option Foundation

Status: active foundation branch.

This document defines the compiler and stdlib foundation required before
continuing deeper CPB Phase 1 stdlib expansion.

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

## Current Blockers

Observed probe results:

- `Ok(5)` and `Err(9)` compile as smoke expressions.
- Returning `Result<I64, I64>` currently fails LLVM module verification.
- `match` with `return` inside `Ok(...)` / `Err(...)` arms currently fails parse.
- enum declaration syntax requires canonical confirmation.
- enum variant layout/tag output is not yet contract-proven.
- legacy `lib/core/result.ldx` exists but is not suitable for ContractVerified status.

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

Do not add `core.error`, `core.option`, or `core.result` as ContractVerified
stdlib modules until the relevant compiler behaviour is proven by tests.

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

The branch starts with ignored executable compiler tests in:

- `tests/compiler_result_option_foundation.rs`

These tests define the intended behaviour and must be unignored one by one as
the compiler foundation becomes real.

Ignored tests are not a way to hide failure. They are a visible backlog of
contract targets for this branch.

Current compiler-foundation tests validate compile success, stderr cleanliness,
and stdout behaviour. Process exit-code normalization is tracked separately
because generated executables can currently emit correct output while returning
a nonzero process status.

Result<I64, I64> return payload foundation is allowed as an intermediate
compiler step: `Ok(x)` and `Err(x)` may lower to the `i64` payload while full
tagged layout and match destructuring remain pending. This must not be presented
as complete `Result<T, E>` semantics.

## Definition of Done

This branch is complete when the following are green:

- compiler e2e tests for enum tags
- compiler e2e tests for `Result<I64, I64>`
- compiler e2e tests for `Option<I64>`
- match destructuring tests for Result and Option
- LLVM module verification
- CPB baseline regression
- `core.option` ContractVerified
- `core.result` ContractVerified

## Integrity Rule

A green test must represent real behaviour.

Do not weaken tests, rename failures away, or replace typed Result/Option with
status-code helpers just to make the branch green.
