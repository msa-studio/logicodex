# CPB-2 Callable Signature Ownership Classification

Status: Candidate classification for PR 1.1
Owner milestone: CPB-2 Callable and Function Type Closure
Baseline: `origin/main` at `baa284cde6e8ccad8a486a221c57c956e65be5ed`

## Decision

The PR 1.1 ownership work is classified as **additive roadmap-aligned implementation**, not an architecture-controlled boundary change, provided the implementation stays within the constraints below.

No RFC label is required for the constrained additive design. `src/hir.rs` remains architecture-sensitive and requires focused review and full integrity before push.

## Live-state evidence

- `src/hir.rs` owns both `HirModule` and the lowering-time `SymbolTable`; the proposed artifact remains inside the existing HIR/lowering subsystem.
- `HirModule` is a public struct currently containing only `items`, and repository consumers construct it with struct literals.
- `SymbolTable` currently stores callable names, parameter types, return types, enum annotations, and visibility across parallel maps.
- `LoweringContext` populates those maps during callable predeclaration and uses them to stamp direct `HirExprKind::Call` nodes.
- Production `compile` and `check` move the populated `SymbolTable` into `SemanticContext`, but subsystem self-check and the public defensive `validate_module` helper construct empty symbol tables.
- The FFI `CallableRegistry` is a separate registry and ID space. Semantic FFI lookup is name-based specifically to prevent numeric-ID aliasing.
- `TypeKind::Function` is not required by the direct-call representation and remains dormant.

## Governance analysis

The constrained design preserves all architecture-controlled boundaries:

- canonical execution remains source -> parser -> AST -> HIR -> `semantic_gate` -> LLVM;
- `semantic_gate` remains sole Meaning Authority;
- HIR/lowering retains ownership of language callable identity and metadata;
- no dependency direction changes between compiler subsystems;
- FFI signatures, capability policy, and IDs remain separate;
- no public function-value, function-pointer ABI, parser grammar, layout, or backend contract is introduced;
- the work implements the already-approved CPB-2 callable-closure milestone;
- `HirModule` shape and existing public lowering APIs are preserved.

Under `docs/governance/architecture-change-control.md`, this is an additive, non-breaking, roadmap-aligned implementation within an existing subsystem.

## Required additive implementation shape

PR 1.1 may:

1. add a complete language-level `HirCallableSignature` representation in `src/hir.rs`;
2. add a new HIR/lowering artifact that contains the unchanged `HirModule` plus frozen language-callable signatures/names needed after lowering;
3. keep `SymbolTable` as the lowering-time builder while freezing one complete signature table at the lowering boundary;
4. add new lowering entry points that return the richer artifact while preserving existing public entry points as behavior-compatible delegates;
5. repair multi-function extern lowering so every declaration survives;
6. define complete current-policy metadata for active builtins;
7. add deterministic internal identity/table-integrity detection and focused tests without introducing source-language duplicate/conflict acceptance policy in lowering;
8. retain `CallableId` as direct-call identity and keep `TypeKind::Function` dormant.

PR 1.2 may later migrate production and defensive semantic contexts to the additive artifact after PR 1.1 is integrated and owner-approved.

## Explicit stop conditions

Stop and request architecture/RFC disposition before implementation if evidence requires any of the following:

- adding a required field to public `HirModule` or incompatibly changing its construction contract;
- changing the return type or removing an existing public lowering API without a compatibility path;
- moving Meaning Authority or callable acceptance rules out of `semantic_gate`;
- merging language signatures or `CallableId` identity with FFI `CallableRegistry` identity;
- activating first-class function values, dynamic calls, `TypeKind::Function`, or function-pointer ABI semantics;
- changing parser grammar, AST contract, target layout authority, production backend responsibility, or capability/trust boundaries;
- introducing a second canonical lowering or semantic path instead of a delegating compatibility API;
- introducing or relocating source-language duplicate/conflicting callable acceptance policy into lowering; PR 1.1 may detect internal identity/table corruption only, while source acceptance remains in `semantic_gate`;
- allocating, renumbering, remapping, or independently merging `CallableId` values while freezing signatures; the artifact must preserve the exact shared whole-program/multi-module ID assignments created by the existing `SymbolTable`;
- requiring a broad rewrite rather than staged migration.

Any such finding changes the classification to architecture-controlled and requires explicit owner authorization plus the applicable RFC state before code proceeds.

## Validation obligations

- strict RED -> GREEN evidence in the same mergeable PR;
- focused callable-closure tests;
- compatibility tests for unchanged `HirModule` and existing lowering entry points;
- language/FFI ID separation tests;
- quick integrity during iteration;
- full integrity before push or task-complete;
- focused architecture review against this classification;
- normal owner checkpoint before integration.

## Classification result

**Proceed to PR 1.1 RED tests under normal review, constrained by this record.**

This classification does not authorize PR 1.2 context migration, Sprint 2 semantics, Sprint 3 diagnostic provenance, or any stop-condition change.