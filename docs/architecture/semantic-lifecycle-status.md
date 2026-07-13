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
| `semantic/type_checker.rs` | FutureReserved | Implemented and unit-tested, intentionally dormant; canonical semantic authority remains `semantic_gate`. |
| `semantic/coercion.rs` | FutureReserved | Dormant coercion foundation owned by the future type-system work; not part of the canonical execution path. |
| `semantic/registry.rs` | FutureReserved | Dormant type-inspection foundation owned by the future type-system work; not part of the canonical execution path. |

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
- classification resolved: the dormant type-checker subsystem is `FutureReserved`, not `OrphanBug`; activation requires an explicit lifecycle update and integration plan for `semantic_gate`.
- inspect the duplicate semantic validation at the codegen boundary;
- rename stale helper names such as `parse_and_analyze` only in a separate,
  behaviour-preserving change.

## SSM-D2 Future-Reserved Resolution

The SSM-D2 wiring audit found no direct construction or invocation of
`TypeChecker` outside its own dormant subsystem. The only apparent external
method match was a generic method-name collision and not type-checker wiring.

Lifecycle decision:

- `semantic_gate` remains the Active canonical semantic authority.
- `semantic::Analyzer` remains `LegacyReferenceOnly`.
- `semantic/type_checker.rs`, `semantic/coercion.rs`, and
  `semantic/registry.rs` are `FutureReserved`.
- No `OrphanBug` evidence was established.
- Owner: semantic/type-system subsystem.
- Activation condition: an approved phase task must define how selected
  capabilities are integrated into HIR-based `semantic_gate`, provide focused
  parity and regression tests, and update this lifecycle classification.
- Until activation, production source must not directly construct or import
  these FutureReserved components.
