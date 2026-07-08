# P0 Type Checker Hardening

Status: active P0 branch.

## P0-TC-A local function call validation

This branch hardens local function calls in `src/semantic.rs`.

Implemented:

- undefined local function calls are rejected
- local function argument count is validated
- local function argument types are validated
- local function call expressions now return the declared function return type
  instead of the previous default `I64`

Regression tests:

- `local_function_call_argument_count_mismatch_fails`
- `local_function_call_argument_type_mismatch_fails`
- `local_function_call_return_type_flows_to_declared_assignment`
- `local_function_call_with_matching_args_still_passes`

Deferred:

- extern/FFI callable registry typing
- builtin call policy cleanup
- generic/unknown call expression handling outside simple local function names
- richer structured diagnostic codes for semantic errors

## Return type validation

The active HIR semantic gate validates explicit `return` statements against the
current function return type before LLVM codegen.

Current P0 behavior:

- `return expr;` must be compatible with the enclosing function return type.
- At HIR level, bare `Return(None)` is accepted only for `Unit`/unknown
  return contexts.
- Non-`Unit` HIR functions with bare `Return(None)` produce a type mismatch
  diagnostic. Source-level bare `return;` may be rejected earlier by the parser.
- Transitional scalar ABI compatibility for current `Option<I64>` and
  `Result<I64, I64>` foundations remains preserved.

## Assignment type validation

The active HIR semantic gate validates assignment statements before codegen.

Current P0 behavior:

- `target = value;` requires the lowered target type to be compatible with the
  lowered value type.
- Compatibility reuses the same rule set as let-binding, call arguments, and
  return expressions:
  - exact/equivalent types,
  - unknown-type tolerance,
  - current uniform integer compatibility,
  - transitional scalar ABI compatibility for current `Option<I64>` and
    `Result<I64, I64>` foundations.

## Condition type validation

The active HIR semantic gate validates control-flow condition expressions before
codegen.

Current P0 behavior:

- `if` conditions must lower to `Bool` or `Unknown`.
- `while` conditions must lower to `Bool` or `Unknown`.
- `Unknown` remains tolerated so name-resolution/type-inference gaps do not
  cascade into misleading secondary errors.
- Integer truthiness helpers such as `core.bool.truthy_i64(...)` should be used
  explicitly when an integer value is intended to become a boolean condition.

## Operator operand validation

The active HIR semantic gate validates operator operand categories before
codegen.

Current P0 behavior:

- Arithmetic, bitwise, modulo, divide, and shift operators require integer
  operands.
- Ordering comparisons require integer operands.
- Logical `&&` / `||` require `Bool` operands.
- Equality / inequality require compatible operand types.
- Unary negation requires an integer operand.
- Unary logical-not requires a `Bool` operand.
- `Unknown` remains tolerated to avoid cascading diagnostics from unresolved
  names or intentionally incomplete inference.

## Assignment target validation

The active HIR semantic gate validates assignment targets before codegen.

Current P0 behavior:

- Writable assignment targets are limited to HIR places that codegen can store
  into: local variables, index targets, and field targets.
- Non-place expressions such as literals, call results, binary expressions,
  unary expressions, casts, constructors, channel operations, and temporaries are
  rejected before codegen.
- This prevents unsupported assignment targets from silently becoming no-op
  stores in the backend.

## Index and array literal validation

The active HIR semantic gate validates fixed-array indexing before codegen.

Current P0 behavior:

- Index expressions require a local fixed-array base.
- Index expressions require an integer index expression.
- Array literal elements must have mutually compatible element types.
- Index assignment value compatibility remains covered by assignment type
  validation because the HIR index expression carries the array element type.
- `Unknown` remains tolerated to avoid cascading diagnostics from unresolved
  names or incomplete upstream inference.

## Missing return validation

The active HIR semantic gate validates that non-`Unit` functions have a
guaranteed return path.

Current P0 behavior:

- Functions returning non-`Unit` / non-`Unknown` types must definitely return.
- `Unit` functions may omit `return`.
- A block definitely returns if a reachable statement definitely returns.
- The final tail expression of a non-`Unit` function can satisfy the return
  path when its type is compatible with the function return type.
- An `if` definitely returns only when both `then` and `else` branches
  definitely return.
- `unsafe` and hardware-zone blocks forward their inner return behavior.
- Loops are not treated as guaranteed-return yet; full CFG/divergence analysis
  is deferred beyond this conservative P0 check.

### Accepted transitional debt: match-lowering metadata

P0 missing-return validation uses explicit HIR control-origin metadata to
distinguish ordinary `if` statements from `if` chains produced by exhaustive
`match` lowering.

Accepted debt:

- `HirStmt::If` carries `HirControlOrigin::LoweredExhaustiveMatch` for
  exhaustive lowered matches.
- This preserves semantic truth without guessing from nested-if shape.
- It avoids Option/Result-specific special cases in the semantic gate.
- It avoids weakening the missing-return checker.

Repayment trigger:

- Replace this transitional metadata with a native HIR `Match` node or a proper
  CFG/exhaustiveness pass once match semantics are promoted beyond the current
  Result/Option foundation.

## Field access validation

The active HIR semantic gate validates field access before codegen.

Current P0 behavior:

- Field access requires a known struct base.
- Accessing a field on a non-struct value fails in semantic validation.
- Accessing an unknown field on a known struct fails in semantic validation.
- Unknown base types remain tolerated to avoid cascading diagnostics.
- HIR preserves the original field name so diagnostics can identify the failed
  field instead of reporting only a lowered field index.
