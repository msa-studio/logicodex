# Semantic Gate Diagnostic Audit — CPB-02C

## Executive Summary

This audit documents **representative, demonstrated diagnostic coverage** for the Logicodex semantic gate. It verifies selected diagnostic failures, concrete source spans, and the semantic-before-codegen barrier for the scenarios listed below. It does **not** claim complete coverage of every diagnostic code: UnsafeBoundaryViolation, FfiBoundaryViolation, and DuplicateDefinition span accuracy remain explicit follow-up gaps.

## Diagnostic Code Enumeration

The following 13 diagnostic codes are defined in `src/span.rs:L74-89` and are the complete enum set. Some covered semantic scenarios, including call-arity mismatch and missing-return validation, are emitted using the existing `TypeMismatch` code rather than separate enum variants. The table therefore distinguishes diagnostic codes from rejection scenarios:

| Code | Responsibility | Function | Test Evidence |
|------|---|---|---|
| `ParserUnsupportedFeature` | Parser rejection | semantic_gate (pre-lowering) | parser tests |
| `UnknownName` | Name resolution failure | check_expression | semantic_missing_return_has_nonzero_span |
| `UnknownFunction` | Function lookup failure | check_expression (call) | unknown_function_uses_specific_diagnostic_code |
| `UnknownType` | Type lookup failure | check_statement, check_expression | unknown_type_uses_specific_diagnostic_code |
| `UnknownEnumVariant` | Enum variant lookup | check_expression (enum construct) | unknown_enum_variant_uses_specific_diagnostic_code |
| `EnumTypeMismatch` | Enum tag/payload mismatch | check_expression | enum variant type mismatch tests |
| `TypeMismatch` | Type incompatibility in assignment/return/call | types_compatible, check_statement, check_expression | assignment_type_mismatch_fails, function_return_expression_type_mismatch_fails |
| `UnsafeBoundaryViolation` | Memory safety violation | check_expression (unsafe context) | unsafe boundary tests |
| `FfiBoundaryViolation` | FFI type boundary violation | check_expression (ffi calls) | ffi tests |
| `LayoutError` | Invalid memory layout | check_lowered_module (layout validation) | layout tests |
| `DuplicateDefinition` | Redefined symbol | lower_module (HIR lowering) | duplicate def tests |
| `DivisionByZero` | Constant division by zero | check_expression (constant folding) | division_by_zero tests |
| `CallableMetadataIntegrity` | Builtin callable metadata mismatch | lower_module_program_with_metadata (PR-02A-META) | cpb2_callable_metadata_boundary tests |

## Span Recovery Evidence

Span recovery infrastructure in `src/main.rs:L600-619` explicitly recovers and formats spans for all Type/Division diagnostics:

- `recover_semantic_diagnostic_spans()` replaces `{:?}` diagnostic output with formatted `span: file X:Y:Z-U:V:W` lines
- Test `semantic_missing_return_has_nonzero_span` asserts output contains `span: file` and rejects zero spans
- All semantic gate rejection paths that emit diagnostics go through `push_error()` which records spans

## Codegen Rejection Barrier

In `src/codegen.rs:L369-412`, the `compile_v130_lowered` function enforces a hard barrier:

```rust
match check_lowered_module(&lowered, &diagnostics) {
    Ok(()) => { /* proceed to codegen */ },
    Err(_) => {
        diagnostics.emit_all();
        anyhow!("Semantic validation failed")
    }
}
```

On failure, `emit_all()` is called, then `anyhow!()` returns early. No subsequent `compile_v130_validated()` call occurs. LLVM codegen functions are never reached.

Test evidence: `p1b3_array_codegen_barrier` demonstrates that semantic checks pass before hitting codegen barriers.

## End-to-End Diagnostic Trace

Example: Type mismatch in assignment

1. **Source:** `let x: I64 = "hello";`
2. **Parser:** Produces AST with Literal("hello") and Type::I64
3. **HIR Lowering:** Converts to lowered_expr(Literal("hello")) with inferred Type::String
4. **Semantic Gate:** `check_statement()` calls `types_compatible(String, I64)` → false
5. **push_error():** Appends DiagnosticCode::TypeMismatch with span (line/col of "hello")
6. **Span Recovery:** Formats output `span: file path.ldx:1:14-1:21`
7. **Exit:** Diagnostic emitted, no codegen reached

## Test Coverage Summary

### Demonstrated Coverage (this candidate)
- Type checker suite: 75 tests covering UnknownName, UnknownFunction, UnknownType, UnknownEnumVariant, TypeMismatch with span verification
- P0 hardening: 1 test with a code + span matrix for 15+ error conditions
- Option/Result: 7 tests verifying ABI stability across semantic checks
- New diagnostic E2E: 7 tests covering TypeMismatch scenarios, call-arity rejection, unknown-name/function rejection, missing-return rejection, and valid success paths
- New span recovery: 4 tests that parse a concrete `span: file <file>:<line>:<column>-<line>:<column>` record and reject all-zero coordinates

### Explicit Gaps (not claimed as covered by this candidate)
- UnsafeBoundaryViolation systematic coverage (2-3 tests)
- FfiBoundaryViolation systematic coverage (2-3 tests)
- DuplicateDefinition span accuracy (1 test)

This candidate is a diagnostic audit and representative semantic-gate coverage update, not a claim that every DiagnosticCode has a dedicated regression fixture.

## Semantic Authority Preservation

This audit confirms:
- Semantic gate is the sole Meaning Authority (canonical)
- HIR is the execution path (no alternative checkers)
- Authority unchanged from PR-02A-META
- Diagnostic codes are static and fully enumerated
- No redesign or invention of semantic rules

## Recommendations

1. **Immediate (PR-02D):** Add 4-6 tests for UnsafeBoundaryViolation and FfiBoundaryViolation coverage
2. **Future:** Consolidate diagnostic audit into automated regression suite per release

---

Audit completed: 2026-10-25 03:28 UTC
Status: READY FOR IMPLEMENTATION REVIEW
