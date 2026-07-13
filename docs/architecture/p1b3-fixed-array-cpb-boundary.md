# P1-B3a Fixed-Array CPB Boundary

Status: Active CPB boundary record

## Decision

P1-B3 must proceed through the proven fixed-array subset before any public
`core.array`, `core.slice`, `Vec`, `List`, heap collection, iterator, or dynamic
buffer API is claimed CPB-ready.

The current compiler-proven subset is:

- fixed array type syntax: `[T; N]`
- fixed array literals: `[a, b, c]`
- local fixed-array binding
- local fixed-array index read: `xs[i]`
- local fixed-array index assignment: `xs[i] = value`
- semantic rejection for array literal length mismatch
- semantic rejection for array literal element type mismatch
- semantic rejection for non-integer array indexes

This is a compiler foundation subset, not a stable stdlib API.

## Non-claims

P1-B3a does not claim:

- `core.array` is implemented
- `core.slice` is implemented
- generic `Array<T>`
- generic `Slice<T>`
- heap-backed `Vec` / `List`
- dynamic collection allocation
- iterator APIs
- array or slice ABI stability
- slice construction / call / round-trip semantics

`[]I64` slice parameter syntax may parse and pass semantic validation, but that
alone is not a CPB slice API claim.

## Contract policy

A public CPB stdlib module requires the normal contract-backed pattern:

- `.ldx` implementation
- `.std.toml` contract sidecar
- oracle cases
- integration tests
- CPB docs update

Until that exists, `core.array` and `core.slice` must not be presented as stable
active stdlib APIs.

## Regression guard

The boundary is guarded by:

- `tests/compiler_collections_foundation.rs`
- `tests/cpb_fixed_array_boundary.rs`

These tests prove the fixed-array subset and prevent silent fallback, missing
type validation, or codegen/LLVM leakage for representative invalid collection
programs.

## Next valid work

The next valid expansion is one of:

1. deepen fixed-array compiler tests and diagnostics;
2. design a minimal contract-backed `core.array` API around the proven fixed
   array subset only;
3. defer `core.slice` until construction, call, and round-trip behaviour are
   proven end to end.

## P1-B3b array boundary update

P1-B3b confirms that local fixed arrays are not enough to introduce a public
`core.array` API. Array parameters and array return values currently pass
semantic `check` but fail during `compile`, so array ABI policy remains a CPB
blocker.

See `docs/architecture/p1b3-array-codegen-barrier.md`.
