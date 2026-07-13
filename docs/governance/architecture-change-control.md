# Logicodex Architecture Change Control

**Status:** Active
**Effective:** On merge of the Architecture Freeze Exit Decision
**Applies to:** Logicodex `0.46.x-alpha`

## Principle

Modification of an architecture-sensitive file does not by itself constitute
an architectural change.

Bug fixes, tests, diagnostics, lifecycle annotations, compatibility fixes, and
additive non-breaking changes may proceed through normal review when they
preserve active compiler authority and public contracts.

## Architecture-sensitive surfaces

These surfaces require focused review but are not blanket-frozen:

- `src/main.rs`
- `src/ast.rs`
- `src/hir.rs`
- `src/semantic.rs`
- `src/semantic_gate.rs`
- `src/codegen.rs`
- `src/types.rs`

CODEOWNERS remains applicable.

## Changes requiring an RFC

An approved RFC and explicit Architect or maintainer approval are required
when a change:

- replaces or bypasses canonical HIR execution;
- moves Meaning Authority away from `semantic_gate`;
- reactivates the legacy AST `Analyzer` as a canonical path;
- introduces a production backend before its permitted roadmap phase;
- incompatibly changes AST, HIR, ABI, layout, or public compiler contracts;
- changes runtime-profile, capability, ownership, or assurance boundaries;
- performs a broad architecture rewrite instead of a staged migration.

Such pull requests must carry:

- `architecture-change`
- `rfc-approved`

## Locked invariants

Unless explicitly changed through an approved RFC:

1. Canonical HIR remains the sole semantic execution path.
2. `semantic_gate` remains the canonical Meaning Authority.
3. The legacy AST `Analyzer` remains non-canonical.
4. LLVM remains the sole production backend until the permitted backend phase.
5. Architecture migrations must be staged, tested, and auditable.
6. Community compiler work remains separate from future proprietary
   Enterprise assurance enforcement.

## Validation tiers

Logicodex uses two validation tiers.

### Quick integrity

`make quick` is permitted for small changes and incomplete subtasks.

It must cover:

- whitespace and diff validation;
- formatting;
- `cargo check`;
- architecture and governance validation;
- workflow and script syntax;
- focused tests selected from the changed surface.

A specific integration test may be requested with:

`make quick TEST=<integration-test-name>`

Quick integrity supports iteration speed. It does not authorize a push or a
task-complete, architecture-sensitive, release-oriented, or major commit.

### Full integrity

Full integrity is mandatory before every push.

`make integrity` is also mandatory before:

- every task-complete or major commit;
- architecture-sensitive work;
- release-oriented work.

The full gate must execute all currently applicable compiler, integration,
contract, CPB, freestanding, boot, workflow, and governance checks. It must not
depend on a fixed historical test count.

## Required evidence

Routine changes require:

- focused regression tests;
- all relevant local tests and policy gates;
- clean formatting and diff checks;
- normal pull-request review.

Declared architectural changes additionally require:

- an approved RFC;
- compatibility and migration analysis;
- test and assurance obligations;
- explicit Architect or maintainer approval;
- `architecture-change` and `rfc-approved` labels.
