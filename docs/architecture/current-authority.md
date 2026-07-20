# Logicodex Current Authority

Status: Active

## Purpose

This is the single entry point for current Logicodex authority. Agents and
contributors must read this file before using roadmap, lifecycle, governance,
or archived documents. Detailed records remain in their owning documents, but
they do not define a competing work sequence.

## Authority order

When sources disagree, use this order:

1. Live source and executable tests define implemented behavior.
2. This file defines current invariants, debt disposition, and work sequence.
3. `Cargo.toml` defines release version authority; approved governance policy,
   RFCs, and maintainer decisions define change authority.
4. Subordinate architecture and lifecycle records provide scoped evidence.
5. `ROADMAP_v2.md` and other planning documents provide long-horizon context,
   not an alternative current sequence.
6. Archived or versioned historical zones preserve provenance only.

A lower item may not override a higher item. If live behavior conflicts with
this record, stop, report the drift, and reconcile both in one scoped change.

## Change-size and architecture boundary

- Any pull request above 500 changed lines carries `size-exception`.
- Documentation updates have no upper line cap. A documentation change above
  500 lines remains allowed with `size-exception` when its scope is cohesive.
- Line count alone does not make a change architectural.
- Architecture-altering work is outside standing authority. It requires
  explicit owner approval and the applicable RFC/governance process.

## Locked current facts

- The canonical compiler path is source -> lexer/parser -> AST -> HIR ->
  `semantic_gate` -> LLVM.
- `semantic_gate` is the sole Meaning Authority. Defensive invocation at the
  codegen boundary does not create a second authority.
- `LayoutEngine` owns target-sensitive layout. AST helpers must not create a
  second layout authority.
- `semantic.rs::Analyzer` is `LegacyReferenceOnly` and has no production role.
- LLVM is the sole production backend until an approved roadmap phase or RFC
  changes that boundary.
- Community compiler semantics remain separate from future Enterprise assurance
  policy, which may consume but must not redefine meaning.
- Current version claims must resolve from `Cargo.toml`; historical milestone
  labels are provenance, not release authority.
- The active governance label is `phase-1` for the `v0.46.x` stabilization / CPB
  line; the owner-locked sequence below refines work inside that phase.

## SSM-D4 transition gate

SSM-D4 is integrated only on a target branch that contains all of the following:

- active version-reference hygiene and exact historical-zone classifications;
- evidence-reviewed closure of the known lifecycle orphan candidates;
- the legacy AST analyzer locked as `LegacyReferenceOnly`;
- passing version, lifecycle, authority-document, quick, and full-integrity
  validators.

Do not claim SSM-D4 integrated-complete from a local stack or an unmerged pull
request. Once the gate is present on the target branch, CPB work follows the
single sequence below.

## Active owner-locked sequence

1. `CPB-2 Callable and Function Type Closure`
2. `CPB-2 Assignment and Return Compatibility`
3. `CPB-2 Diagnostic End-to-End`

Do not replace this sequence with audit-backlog priority, old sprint order,
issue numbering, or broad type-checker/diagnostics cleanup. A sequence change
requires explicit maintainer approval and an update to this file.

## Residual debt disposition

| Debt | Disposition | Blocking point |
|---|---|---|
| Warning baseline and broad suppression attributes | Reduce through focused, behavior-preserving changes; never broad cleanup | Non-blocking for callable/function closure unless a touched surface adds drift |
| `SemanticError` ownership | Decide in diagnostics-owned work | Before or within `CPB-2 Diagnostic End-to-End` |
| Defensive semantic validation at codegen | Retain as defense in depth until parity evidence supports change | Review after callable and assignment/return semantics stabilize |
| Stale helper naming such as `parse_and_analyze` | Separate behavior-preserving cleanup | Non-blocking |
| `FutureReserved` type-checker, coercion, registry, legacy-type, and backend-contract surfaces | Preserve dormant; activation requires approved task/RFC and integration evidence | Not cleanup debt and not implicitly active |
| Newly discovered orphan candidates | Classify by callers, ownership, and tests before wire/archive/delete decisions | Must be resolved only when discovered and scoped |

Residual debt does not authorize a broad rewrite. A CPB task owns only debt that
blocks its stated semantic contract or is directly touched by its implementation.

## Subordinate evidence records

| Record | Scope |
|---|---|
| [`code-lifecycle-inventory.md`](code-lifecycle-inventory.md) | Suppressions, lifecycle status, orphan and legacy evidence |
| [`semantic-lifecycle-status.md`](semantic-lifecycle-status.md) | Meaning Authority and semantic component lifecycle |
| [`version-reference-classification.md`](version-reference-classification.md) | Current, historical, fixture, and archived version references |
| [`cpb-self-hosting-runway.md`](cpb-self-hosting-runway.md) | Long-run CPB capability runway and entry/exit gates |
| [`cpb-next-roadmap-blockers.md`](cpb-next-roadmap-blockers.md) | Blocker inventory only; it does not own execution order |
| [`CONTRACT_EXTENSION_ARCHITECTURE.md`](CONTRACT_EXTENSION_ARCHITECTURE.md) | Contract-extension pattern; its foundation order is historical only |
| [`../governance/architecture-change-control.md`](../governance/architecture-change-control.md) | RFC boundaries, locked invariants, and validation tiers |
| [`../../ROADMAP_v2.md`](../../ROADMAP_v2.md) | Long-horizon phase baseline and maturity history |

Archived documents and `.zone-status.md` historical boundaries are never current
compiler authority.

## Update rule

Only this file may contain the heading `Active owner-locked sequence`. Detailed
records may link here but must not copy or reorder that sequence. Every change to
current authority must update affected evidence records and changelog in the same
batch, pass the repository documentation/governance and integrity gates, and
receive normal review. Architecture boundary changes still require the RFC
process defined by active governance.
