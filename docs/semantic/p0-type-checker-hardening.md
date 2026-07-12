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

## Cast validation

The active HIR semantic gate validates internal `HirExprKind::Cast` nodes before
codegen.

Current P0 behavior:

- Source-level `x as T` syntax is still parser-blocked and intentionally not
  enabled by this step.
- Same-type casts are allowed.
- Integer-to-integer casts are allowed under the current uniform integer codegen
  model.
- Transitional `Option<I64>` / `Result<I64, I64>` scalar ABI casts to `I64` are
  allowed for current match-lowering foundations.
- Struct, array, Unit, function, and pointer casts are rejected unless a future
  explicit capability/provenance rule allows them.
- Unknown source/target types remain tolerated to avoid cascading diagnostics.

Debt note:

- Full source cast syntax and pointer/provenance-aware cast policy are deferred.
- This step only prevents internal HIR cast nodes from silently becoming codegen
  no-ops without semantic validation.

## Struct declaration layout validation

The active HIR semantic gate validates source-defined struct declarations before
codegen.

Current P0 behavior:

- Duplicate field names in the same struct are rejected.
- Struct fields with unknown field types are rejected.
- Direct by-value self-recursive struct fields are rejected when they resolve to
  the same struct layout.
- Unknown recursive references may surface as unknown field-type diagnostics until
  a later named-type predeclaration pass exists.
- Valid struct declarations with known field types continue to pass.
- Enum layout validation is deferred because current source enum syntax and
  payload representation are not yet production-grade.

Debt note:

- Full recursive type analysis, pointer/provenance-aware recursive layouts, and
  enum tagged-union layout remain future work.

## Enum variant lowering validation

The HIR lowering path now rejects unresolved enum variant references instead of
silently lowering them to tag `0`.

Current P0 behavior:

- Known enum variants continue to lower to their numeric tag.
- Unknown `Enum::Variant` references produce a lowering diagnostic.
- The lowered placeholder tag remains `0` only after an emitted diagnostic, so
  `check`/`compile` fail instead of producing a successful binary.
- Full enum payload/layout validation remains deferred until source enum syntax
  and tagged-union representation are production-grade.

## Call result value validation

The active HIR semantic gate now rejects calls that do not produce a usable value
when they appear in value-required positions.

Current P0 behavior:

- A call returning `Unit` cannot be used as an `I64` return value, binding value,
  or non-Unit function argument.
- A call with an unresolved/unknown result type cannot silently satisfy a
  value-required position.
- Unit-returning calls remain valid as expression statements where their result
  is intentionally discarded.
- This prevents void/no-value calls from reaching codegen and becoming fallback
  zero values.

## Explicit return policy

The active semantic gate now requires explicit `return` statements for non-Unit
functions.

Current P0 behavior:

- A non-Unit function must have an explicit guaranteed return path.
- Tail expressions such as `42;` or `Point(1, 2);` do not satisfy return
  obligations yet because they lower to `HirStmt::Expr`.
- Codegen currently discards `HirStmt::Expr` values and may otherwise add an
  implicit fallback `return 0`; the semantic gate prevents that path.
- Tail-expression returns may be reintroduced later only after HIR/codegen
  carries them as real return semantics rather than discarded expression
  statements.

## Return value validation for Unit/unknown returns

The active semantic gate now rejects invalid return value paths before codegen.

Current P0 behavior:

- A `Unit`/unknown-return function cannot silently return a value.
- A return expression with unresolved result type fails semantic validation instead
  of reaching backend type fallback handling.
- Unit functions may still omit an explicit return.
- Strict rejection of all unknown binding annotations is deferred until enum
  annotations and unknown/missing type names can be distinguished cleanly.

## Exact enum qualifier validation

Qualified enum references now require an exact enum-name match.

Current P0 behavior:

- `Status::Ready` resolves only through enum `Status`.
- `MissingEnum::Ready` fails even if another enum has a `Ready` variant.
- Variant-name-only fallback is not allowed for qualified `Enum::Variant`
  references.

## Missing named type vs enum annotation

Named type annotations are now distinguished during HIR lowering.

Current P0 behavior:

- A truly missing named type such as `MissingType` fails HIR lowering.
- A known enum annotation such as `Status` is accepted.
- Primitive named annotations such as `Unit` remain valid.
- Known enum annotations are still lowered through the transitional scalar tag
  ABI (`I64`) until full `TypeKind::Enum` identity is enforced end-to-end.
- This avoids treating missing type names and known enum annotations as the same
  `Unknown` type.

## HIR call result metadata fallback

Call expressions now use their HIR-stamped expression type as the fallback source
of truth when semantic revalidation cannot resolve callable metadata by
`CallableId`.

This keeps compile-time semantic revalidation aligned with HIR lowering without
rehydrating a fresh callable table by name:

- local function calls retain their return type during compile-time gate checks;
- enum-returning functions retain the current transitional scalar ABI return type;
- value-use validation no longer reports `Call <unknown>` for known local calls;
- FFI symbol/capability metadata is not perturbed by a secondary callable
  hydration pass.

## Enum qualifier identity guard

P0 enum ABI remains scalar-tag based (`I64`) while HIR/codegen mature, but
lowering now preserves enough source enum annotation metadata to reject wrong
qualified variants before enum identity is erased by the transitional ABI.

Guarded contexts:

- `let s: Status = Other::Ready;`
- `return Other::Ready;` inside `function f() -> Status`
- `use_status(Other::Ready)` when the parameter annotation is `Status`
- `let s: Status = other();` when `other() -> Other`

This keeps the current scalar ABI stable while preventing unrelated enum types
from becoming silently interchangeable.

## HIR diagnostic classification phase 1

HIR-lowering semantic failures now use specific diagnostic codes instead of
routing all such failures through `ParserUnsupportedFeature`.

Phase 1 classification:

- unknown value/name lookup: `UnknownName`
- unknown function/callable lookup: `UnknownFunction`
- unknown named type annotation: `UnknownType`
- unknown enum variant: `UnknownEnumVariant`
- wrong enum qualifier/enum identity mismatch: `EnumTypeMismatch`

`ParserUnsupportedFeature` remains reserved for genuine unsupported syntax or
capability gaps. Span quality is tracked separately; this phase does not rewrite
span propagation.

## HIR diagnostic span recovery phase 1

HIR-lowering diagnostics now recover source spans for key semantic failures when
their primary span would otherwise be `Span::unknown()`.

Phase 1 covers:

- `UnknownName`
- `UnknownFunction`
- `UnknownType`
- `UnknownEnumVariant`
- `EnumTypeMismatch`

This is a source-text recovery layer at the CLI pipeline boundary. It does not
change parser grammar, AST shape, HIR semantics, ABI, or codegen. Full AST span
propagation remains a later migration once the P0/CPB semantic gates are stable.

## v1.30 diagnostic formatter subset

The v1.30 diagnostic formatter now renders structured diagnostic metadata for
compiler diagnostics:

- `code: <DiagnosticCode>`
- `span: file <file_id>:<start_line>:<start_col>-<end_line>:<end_col>`
- bilingual Malay/English diagnostic message
- optional diagnostic notes

This makes semantic-gate diagnostics such as `TypeMismatch` visible to users and
AI agents as structured output instead of message-only text. The change is
formatter-only: it does not change semantic rules, HIR, parser/AST shape, ABI, or
codegen.
