# CPB-02D-A Diagnostic Boundary Follow-up

## Scope and authority

This cycle adds test/documentation coverage only. The canonical authority remains `semantic_gate`; HIR remains the execution path; LLVM remains the production backend; `LayoutEngine` remains target-sensitive layout authority. `semantic.rs::Analyzer` is not imported or called.

- Worktree: `/home/msastudio/.hermes/cache/scratch/logicodex-cpb-2d-diagnostic-boundaries`
- Branch: `feat/cpb-2-diagnostic-boundaries`
- Base: `e2f56b6539a562138057cdd1668d66ac329a9b4d`
- Graft: deterministic target graph `2806 nodes`, `2842 edges`, `190 cards`; `graph check: OK`; deep/LLM layer not built.
- Graft evidence: `/home/msastudio/awang/n8n-cpb/reports/cpb-02d-diagnostic-boundaries-20261025/research-live.log`

## Baseline evidence

The counts below are quoted from these source logs, not regenerated manually:

- `/home/msastudio/awang/n8n-cpb/reports/cpb-02d-diagnostic-boundaries-20261025/type-hardening-baseline.log`: `test result: ok. 75 passed; 0 failed`
- `/home/msastudio/awang/n8n-cpb/reports/cpb-02d-diagnostic-boundaries-20261025/rejects-break-baseline.log`: `semantic_gate::tests::rejects_break_outside_loop ... ok`
- `/home/msastudio/awang/n8n-cpb/reports/cpb-02d-diagnostic-boundaries-20261025/callable-closure-baseline.log`: `test result: ok. 10 passed; 0 failed`
- `/home/msastudio/awang/n8n-cpb/reports/cpb-02d-diagnostic-boundaries-20261025/authority-baseline.log`: `test result: ok. 3 passed; 0 failed`
- FFI baseline: `cargo test ffi::` matched zero tests; this cycle supplies the missing direct matrix.

## Direct matrix records

The ten tests in `tests/cpb2_diagnostic_boundary_matrix.rs` assert one exact record, `Severity::Error`, exact bilingual messages, empty notes, and the expected current span. They cover:

- `UnsafeBoundaryViolation`: break and continue outside loops.
- `DuplicateDefinition`: repeated `duplicate_target` HIR function.
- `FfiBoundaryViolation`: unsafe context, fixed arity, variadic minimum arity, argument type, and capability policy.
- CLI rejection: duplicate function and break outside loop.

At CPB-02D-A, the public direct FFI type-name path rendered empty type labels for the `typed` mismatch fixture:

```text
Ralat: Argumen 1 untuk fungsi 'typed' mempunyai jenis yang tidak sepadan (diperlukan: , diterima: )
Error: Argument 1 for function 'typed' has an incompatible type (expected: , got: )
```

CPB-02D-A locked this as observed behavior and did not repair production message formatting.

**Resolved (CPB-2 FFI type label).** `TypeRegistry::type_name` now renders the `{:?}` label of a registered `TypeKind`, matching `semantic_gate` type labels, and returns `None` for an unregistered ID. The fixture now renders:

```text
Ralat: Argumen 1 untuk fungsi 'typed' mempunyai jenis yang tidak sepadan (diperlukan: Primitive(I64), diterima: Primitive(Unit))
Error: Argument 1 for function 'typed' has an incompatible type (expected: Primitive(I64), got: Primitive(Unit))
```

Code, severity, span, and record count are unchanged. CLI `check` output for user-declared externs is unchanged: those calls take the capability-only branch in `semantic_gate`, and their argument-type mismatches are reported as `TypeMismatch`, so the CLI does not reach this FFI message.

- Evidence: `/home/msastudio/.hermes/cache/scratch/claude-logicodex/evidence-update1/`

## Current CLI behavior

Captured evidence source: `/home/msastudio/awang/n8n-cpb/reports/cpb-02d-diagnostic-boundaries-20261025/cli-fixtures-observed.txt`

Duplicate fixture:

```text
Error: code: DuplicateDefinition
span: file 0:0:0-0:0
Ralat: Fungsi 'duplicate_target' ditakrif lebih daripada sekali / Error: Function 'duplicate_target' is defined more than once
```

Break fixture:

```text
Error: code: UnsafeBoundaryViolation
span: file 0:0:0-0:0
Ralat: Kawalan gelung digunakan di luar gelung / Error: Loop control used outside a loop
```

Both checks exit nonzero and produce no `.ll`, `.bc`, `.o`, `.obj`, `.exe`, or `main` backend artifact.

The `0:0:0-0:0` span is an explicit current limitation. Source-backed span propagation is not implemented here.

## CPB-02D-B handoff

A future production cycle must separately investigate source-span propagation for duplicate definitions, unsafe control flow, and FFI diagnostics. It must review any change to HIR span propagation or `recover_semantic_diagnostic_spans`; no such production change is included in CPB-02D-A.
