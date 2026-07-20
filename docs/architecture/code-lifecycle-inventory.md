# Code Lifecycle Inventory

Status: Active SSM-D2 lifecycle record

## Purpose

This document is the canonical cross-compiler inventory for inactive,
suppressed, legacy, reserved, and potentially orphaned code discovered during
Semantic Stabilization Milestone Stage D2.

SSM-D2 is classification work, not cleanup. It does not authorize deletion,
automatic rewiring, broad warning removal, or architecture changes.

The subsystem-specific semantic classifications remain authoritative in
[`semantic-lifecycle-status.md`](semantic-lifecycle-status.md).

## Lifecycle statuses

The permitted lifecycle statuses are:

- `Active`
- `FutureReserved`
- `Experimental`
- `LegacyReferenceOnly`
- `Deprecated`
- `OrphanCandidate`
- `OrphanBug`

A warning suppression is not itself a lifecycle status. Active code may
temporarily retain a compatibility suppression, but the underlying artifact
must still be classified explicitly.

## Audit baseline

The original SSM-D2 audit established 15 explicit suppressions. Following the
evidence-backed SSM-D4 closure below, the current validated baseline is:

- 13 explicit dead-code, unused-variable, or related suppression attributes;
- 7 crate-level suppression attributes;
- 6 item-level suppression attributes;
- 87 warnings visible under warning-enabled `cargo check --all-targets`;
- no established `OrphanBug`;
- no broad source-removal authorization.

The 87-warning baseline is technical debt for staged follow-up. It is not a
mandate for broad cleanup during SSM-D2.

## Suppression and lifecycle inventory

| Suppressed artifact | Status | Canonical evidence | Owner and intended action | Activation or review condition |
|---|---|---|---|---|
| `src/codegen_contract.rs`: crate-level `dead_code` | FutureReserved | No direct external module reference. Introduced as a dormant backend-contract simulation with an internal mock test. The similarly named trait in `codegen.rs` is a separate production definition. | Backend architecture / LMCBA. Preserve and keep outside the production path. | Activation requires CPB completion, permitted backend phase, architecture phase gate, and RFC approval. |
| `src/contract_metadata.rs`: crate-level `dead_code` | Active | Used by `module_loader.rs` for library-layer recognition and contract metadata hints. | Contract metadata subsystem. Retain; narrowing the broad suppression requires a dedicated behaviour-preserving cleanup. | Reassess only with focused metadata tests and compatibility validation. |
| `src/ffi.rs`: crate-level `dead_code` | Active | Provides `CallableRegistry`, `CallableSignature`, FFI resolution, and capability-gate inputs used by codegen and semantic validation. | FFI subsystem. Retain. | Narrow suppression only after a focused unused-surface audit. |
| `src/hir.rs`: crate-level `dead_code` | Active | Canonical HIR types and AST-to-HIR lowering used by the single semantic and codegen engine. | HIR subsystem. Retain. | Narrow suppression only after HIR surface compatibility review. |
| `src/layout.rs`: crate-level `dead_code` | Active | Layout engine is used by the HIR backend and associated tests. | Type-layout subsystem. Retain. | Narrow suppression only with layout regression coverage. |
| `src/span.rs`: crate-level `dead_code` | Active | Span and diagnostic types have extensive production use across main, HIR, FFI, layout, semantic gate, and codegen-contract foundations. | Diagnostics/span subsystem. Retain. | Narrow suppression incrementally after diagnostic API ownership is stabilized. |
| `src/span.rs`: crate-level `unused_variables` | Active | Same active diagnostics module; the suppression hides incomplete or compatibility-oriented diagnostic surfaces. | Diagnostics/span subsystem. Retain temporarily. | Review during structured diagnostics stabilization, not SSM-D2 cleanup. |
| `src/main.rs`: `#[allow(dead_code)] mod semantic` | Active | The module contains mixed lifecycle content: active `SemanticError` dependencies and a non-authoritative legacy AST analyzer. Detailed ownership is defined by the semantic lifecycle record. | Compiler entry-point and semantic subsystems. Retain the declaration; do not treat the entire module as legacy. | Reassess after active diagnostic ownership is separated from legacy analyzer code. |
| `src/semantic.rs::Analyzer::analyze` | LegacyReferenceOnly | No production caller. The HIR `semantic_gate` is the canonical semantic authority. | Semantic subsystem. Preserve as historical and migration reference only. | Reactivation requires an approved roadmap task or RFC and parity evidence against `semantic_gate`. |
| `src/types.rs::LegacyType` | FutureReserved | No production caller. Documentation identifies it as translation-front-end mapping infrastructure. | Foreign-type and frontend-adapter subsystem. Preserve. | Activate only with an approved C, COBOL, Fortran, or other frontend-adapter phase. |
| `src/types.rs::impl LegacyType` | FutureReserved | No production caller; methods map legacy source-language types to canonical native primitives. | Foreign-type and frontend-adapter subsystem. Preserve. | Same activation condition as `LegacyType`. |
| `src/types.rs::PrimitiveType::int_bits` | Active | Called by `codegen.rs` when selecting integer extension behaviour. | Type and codegen subsystem. Retain. | Suppression may be narrowed separately if warning behaviour permits. |
| `src/types.rs::PrimitiveType::is_unsigned_int` | Active | Called by `codegen.rs` to select zero-extension versus sign-extension behaviour. | Type and codegen subsystem. Retain. | Suppression may be narrowed separately if warning behaviour permits. |

## Semantic subsystem reference

The dormant semantic type-checker subsystem has already been classified in
`semantic-lifecycle-status.md`:

- `semantic/type_checker.rs` — `FutureReserved`
- `semantic/coercion.rs` — `FutureReserved`
- `semantic/registry.rs` — `FutureReserved`
- `semantic.rs::Analyzer` — `LegacyReferenceOnly`
- `semantic_gate.rs` — `Active`

No direct production wiring to the dormant type-checker subsystem was found.

## SSM-D2 decisions

1. `codegen_contract.rs` is `FutureReserved`, not `OrphanBug`.
2. `storage_width_bits` and `is_signed_int` remained `OrphanCandidate` pending
   ownership and wiring review; SSM-D4 resolves both below.
3. Active modules with broad suppression remain Active; suppression presence
   does not downgrade their lifecycle status.
4. Unit tests alone do not establish production activation.
5. No uncalled function, type, module, or trait may be deleted automatically.
6. No `OrphanBug` evidence has been established by this audit.
7. Source removal is not authorized by SSM-D2.

## SSM-D4 Orphan / Legacy Closure

SSM-D4 performed the ownership and wiring review that SSM-D2 deliberately
deferred. This is a narrow, evidence-backed closure, not broad dead-code
cleanup.

- `src/ast.rs::Type::storage_width_bits`: `Delete` — canonical layout ownership belongs to `LayoutEngine`.
- `src/types.rs::PrimitiveType::is_signed_int`: `Delete` — integer extension behavior is already closed by `int_bits` plus `is_unsigned_int`.

The first helper duplicated target-sensitive layout policy at the AST layer,
including a catch-all 64-bit fallback. Wiring it would have created a second
layout authority beside `LayoutEngine`. The second helper was redundant within
the codegen integer path: `int_bits()` first restricts the domain to integer
primitives, then `is_unsigned_int()` selects zero extension; the remaining
integer primitives are exactly the signed extension cases.

Although `ast` and `types` are public library modules, neither helper is a
documented or contracted API. Logicodex `0.46.0-alpha` explicitly remains
pre-1.0 with an unstable API, while semver-guaranteed public API and deprecation
policy are deferred to roadmap P5-D5. The removals therefore close unowned
alpha surface without changing compiler behaviour; they still require normal
review before landing.

`src/semantic.rs::Analyzer::analyze` remains `LegacyReferenceOnly`. It is
retained as historical and migration reference, is not wired into production,
and may not change lifecycle status without approved parity evidence against
the canonical HIR `semantic_gate`.

The lifecycle validator fails closed if either deleted helper is reintroduced,
if either resolution record disappears, or if the legacy analyzer status
drifts. These exact decisions do not prohibit future legitimate
`OrphanCandidate` classifications.

## Follow-up boundary

Later stages may:

- add a lifecycle validator in SSM-D3;
- review any newly discovered OrphanCandidate wiring and ownership;
- narrow broad suppression attributes incrementally;
- separate active diagnostic ownership from legacy semantic code;
- reduce the warning baseline through focused, behaviour-preserving changes.

These follow-ups must remain staged and must not become a broad cleanup rewrite.
