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

## Governance label contract

The three RFC-related labels represent separate governance facts:

- `architecture-change`
  declares that the pull request changes an architecture-controlled boundary.
  Merely touching an architecture-sensitive file does not require this label.

  The label is required only when the change alters at least one of:

  - Meaning Authority or canonical HIR semantics;
  - the canonical execution path or backend responsibility;
  - subsystem ownership or dependency direction;
  - a capability, security, isolation, or trust boundary;
  - a public ABI, contract, or compatibility obligation;
  - approved roadmap architecture or phase authority.

  Implementing an already-approved roadmap item inside its existing boundary
  is controlled implementation, not architecture drift.

- `size-exception`
  records maintainer review of a pull request above the default size threshold.
  It is a review and cohesion signal only. It neither declares nor approves an
  architecture change.

- `rfc-required`
  declares that a proposed architecture-controlled boundary change is awaiting
  RFC review. It does not mean that the RFC has already been approved.

- `rfc-approved`
  records that the Architect or authorized maintainer has approved the RFC
  and authorized the architecture-changing implementation.

The supported combinations are:

| Pull-request state | Required labels | Governance meaning |
| --- | --- | --- |
| Routine focused or roadmap-aligned implementation | no governance exception label | Normal review |
| Large non-architecture change | `size-exception` | Cohesion justification and maintainer size review |
| Architecture proposal awaiting approval | `architecture-change`, `rfc-required` | Architecture implementation is not yet approved |
| Approved architecture implementation | `architecture-change`, `rfc-approved` | Approved RFC evidence must be linked |
| Large approved architecture implementation | `architecture-change`, `rfc-approved`, `size-exception` | Architecture authorization and size review are independent |

`size-exception` is independent from architecture authorization. It must not
be used as an RFC substitute, and RFC labels must not be required only because
a pull request is large.

`rfc-required` and `rfc-approved` represent different lifecycle states and
must not be used interchangeably. Once an architecture RFC is approved,
`rfc-required` is replaced by `rfc-approved`.

`rfc-approved` must never be applied only to bypass the PR-size gate.

## Size control and architecture control

The default line threshold is an advisory review boundary, not an architecture
veto. A cohesive, roadmap-aligned change may proceed with `size-exception`
when its pull request records:

- the owning roadmap item and bounded scope;
- why further splitting would reduce cohesion or validation quality;
- focused and full-integrity validation evidence;
- confirmation that no architecture-controlled boundary is being changed.

A large pull request that also changes an architecture-controlled boundary
must satisfy both controls independently.

The following do not by themselves constitute architecture drift:

- implementing an already-approved roadmap item;
- touching an architecture-sensitive file;
- adding tests, validators, tooling, examples, or documentation;
- behaviour-preserving internal refactoring;
- exceeding the default pull-request size threshold.

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

Before approval, an architecture proposal carries:

- `architecture-change`
- `rfc-required`

After approval and implementation authorization, it carries:

- `architecture-change`
- `rfc-approved`

The approved RFC and authorization evidence must be linked from the pull
request.

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
