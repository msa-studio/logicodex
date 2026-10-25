# CPB-02E FFI Call and Argument Span Propagation

## Scope

CPB-02E carries source spans for ordinary FFI call expressions and their arguments through the AST, parser, and HIR lowering boundary.

- Policy, unsafe-context, and arity diagnostics use the complete call-expression span.
- Incompatible-argument diagnostics use the offending argument expression span.
- Nested calls retain distinct call and argument spans.
- Synthetic compiler-generated calls retain `Span::unknown()`.

## Authority and compatibility

`semantic_gate` remains the sole Meaning Authority and HIR remains the canonical execution path. LLVM, ABI, ownership, and `LayoutEngine` behavior are unchanged. `semantic.rs` and `src/tier2/pass.rs` receive only `Expr::Call { .. }` pattern compatibility for the new carrier fields. FFI diagnostic codes, bilingual messages, severity, ordering, and fail-closed behavior remain unchanged.

## Evidence contract

The candidate is validated with seven direct FFI span tests, three CLI diagnostic tests, ten callable-closure compatibility tests, repository-wide regression tests, formatting, diff checks, and full integrity. This document records scope and evidence expectations only; it does not introduce runtime behavior.
