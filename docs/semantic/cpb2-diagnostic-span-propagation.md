# CPB-02D-B1 Source-Backed Diagnostic Spans

## RFC and scope

- RFC: `/home/msastudio/awang/external-review/requests/cpb-02d-span-propagation-rfc-draft-20261025.md`
- Status: `APPROVED_LOCAL` by Ady; external/community approval and merge/release authorization are not claimed.
- B1 scope: function declaration, break, and continue spans only.
- FFI expression/argument spans are deferred to B2.

## Ownership path

Lexer token positions are captured by parser-owned AST spans, copied through `src/hir.rs` lowering into existing HIR `Spanned` wrappers, then consumed by the existing `semantic_gate` diagnostic paths. No generic `main.rs` recovery was added. `semantic.rs::Analyzer` remains unwired.

Affected compatibility files:

- `src/ast.rs`
- `src/parser.rs`
- `src/hir.rs`
- `src/semantic.rs` (pattern-match compatibility only)
- `tests/cpb2_callable_metadata_boundary.rs` (synthetic `Span::unknown()` constructors)
- `tests/cpb2_diagnostic_boundary_matrix.rs` (expected propagated ranges)
- `tests/cpb2_diagnostic_span_propagation.rs`

## Span convention

- File ID: `FileId(0)` for the single temporary source fixture.
- Lines/columns: 1-based.
- End: exclusive.
- Duplicate declaration: `span: file 0:4:1-4:41`.
- Break statement: `span: file 0:2:5-2:11`.
- Continue statement: `span: file 0:2:5-2:14`.

## Diagnostic records preserved

Duplicate:

```text
Error: code: DuplicateDefinition
Ralat: Fungsi 'duplicate_target' ditakrif lebih daripada sekali / Error: Function 'duplicate_target' is defined more than once
```

Break/continue:

```text
Error: code: UnsafeBoundaryViolation
Ralat: Kawalan gelung digunakan di luar gelung / Error: Loop control used outside a loop
```

The B1 tests assert exact code, severity, bilingual messages, record count, file identity, ranges, and no backend artifacts. Synthetic/compiler-generated constructs continue to use `Span::unknown()` explicitly.

## Validation evidence

- Span propagation tests: 5 passed.
- CPB-02D-A boundary regression: 10 passed.
- `cargo fmt --check`: pass.
- `git diff --check`: pass.
- Full integrity: required before implementation review.

## B2 handoff

FFI call and offending-argument source spans remain deferred. They require a separate parser/AST expression-carrier design and fresh plan review; B1 does not alter FFI production paths, ABI, backend, LayoutEngine, or diagnostic codes.
