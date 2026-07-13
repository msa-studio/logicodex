# Semantic Lifecycle and Meaning Authority

Status: Active SSM lifecycle record

## Canonical authority

`src/semantic_gate.rs` is the canonical HIR semantic validation authority.

It is invoked by:

- the `check` path;
- the `compile` path;
- the subsystem self-check;
- the defensive codegen boundary.

The codegen invocation is defense in depth. It does not establish a second
semantic authority.

## Lifecycle classification

| Component | Status | Current ownership |
|---|---|---|
| `semantic_gate.rs` | Active | Canonical HIR semantic validation |
| `semantic.rs::Analyzer` | LegacyReferenceOnly | Historical AST checks and migration reference |
| `semantic.rs::SemanticError` | Active | Error dependency used by `tier2::pass` |
| `semantic/type_checker.rs` | OrphanCandidate | Implemented and unit-tested, but not wired into the canonical pipeline |
| `semantic/coercion.rs` | FutureReserved / OrphanCandidate | Supporting component for the dormant type checker |
| `semantic/registry.rs` | FutureReserved / OrphanCandidate | Supporting component for the dormant type checker |

## Rules

1. New semantic correctness work must target HIR lowering, `semantic_gate.rs`,
   or another explicitly approved canonical HIR pass.
2. The AST `Analyzer` must not be reactivated silently.
3. No uncalled semantic component may be deleted automatically.
4. Future activation of the dormant type checker requires an RFC or an approved
   roadmap task defining its relationship with the canonical HIR gate.
5. `SemanticError` ownership may be extracted later, but moving it is not part
   of this classification-only change.
6. Unit-test coverage alone does not prove that a component is active in the
   production compiler pipeline.

## Debt follow-up

- inspect whether `SemanticError` should move to a neutral diagnostics or Tier 2
  ownership module;
- determine whether the dormant type checker is FutureReserved or an OrphanBug;
- inspect the duplicate semantic validation at the codegen boundary;
- rename stale helper names such as `parse_and_analyze` only in a separate,
  behaviour-preserving change.
