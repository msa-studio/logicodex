# P1-B3b Array Parameter and Return Codegen Barrier

Status: Active CPB blocker record

## Decision

Do not introduce a public `core.array` or `core.slice` CPB API yet.

The compiler currently proves local fixed-array use, but array values are not
yet safe across function ABI boundaries.

## Proven today

The compiler-proven subset remains:

- local fixed-array binding
- fixed-array literal as a typed local initializer
- local fixed-array index read
- local fixed-array index assignment
- semantic rejection for invalid fixed-array programs

## Current barrier

The following currently pass semantic `check` but fail during `compile`:

- `[I64; N]` parameter passed to a function
- `[I64; N]` parameter indexed inside a callee
- `[I64; N]` returned from a function

Observed barriers include:

- LLVM verifier failure for parameter type mismatch
- unsupported codegen path for array literal return values

This means `core.array` cannot yet expose functions that accept or return
fixed arrays as stable CPB API.

## Required before public `core.array`

Before `core.array` can become a CPB dependency, the compiler must define and
prove at least one stable array boundary policy:

1. by-value fixed-array ABI;
2. by-reference fixed-array ABI;
3. explicit pointer/length representation;
4. compiler intrinsic lowering with documented restrictions.

Whichever policy is chosen must be proven through:

- semantic checks
- HIR lowering
- codegen
- integration tests
- contract harness cases
- CPB documentation

## Regression guard

The current blocker is guarded by `tests/p1b3_array_codegen_barrier.rs`.

That test is intentionally a blocker marker. When array ABI support is
implemented, replace the barrier test with positive array boundary tests and
only then introduce contract-backed `core.array`.
