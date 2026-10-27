# CPB-02B Assignment and Return Compatibility

## Contract

This stage closes regression coverage for the existing semantic compatibility behavior:

- assignment and declared-binding values must match their expected types;
- explicit return expressions must match the enclosing function return type;
- Unit functions cannot return values;
- Unit call results cannot be consumed where a value is required;
- matching assignment and return cases remain accepted.

The contract preserves exact `TypeMismatch` diagnostic identity, bilingual messages, `Error` severity, source spans, and record ordering. Rejected checks must fail closed without backend artifacts.

## Authority boundaries

No production source changes are made by CPB-02B compatibility closure. `semantic_gate` remains the sole Meaning Authority, HIR remains the canonical execution path, LLVM remains the production backend, `LayoutEngine` remains the target-sensitive layout authority, and `semantic.rs::Analyzer` remains legacy/reference-only. Unsupported behavior is not activated and no ABI, ownership, backend, or dormant behavior changes are introduced.

## Validation

The dedicated matrix contains eight exact tests and invokes the compiled `logicodex check` binary through the live parser → AST → HIR → `semantic_gate` path. Existing type-checker, P0 hardening, result/option, all-target, formatting, diff, and full-integrity gates remain part of the stage validation.
