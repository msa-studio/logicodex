# Version Reference Classification

Status: Active architecture and governance policy.

This document classifies version references by purpose so that historical
evidence is preserved without allowing old releases to become current
compiler authority.

## CurrentAuthority

Active code, workflows and normative guidance must use the canonical version
authority or version-agnostic wording. Hardcoded release labels must not act
as current authority.

## HistoricalProvenance

A historical version may remain when it explains when a meaningful capability,
design decision or compatibility boundary was introduced. The surrounding text
must explicitly identify it as historical provenance.

## CompatibilityFixture

Compatibility selectors, parser fixtures, regression inputs and negative-test
claims may retain older version strings when the value is part of the tested
contract. Their fixture or compatibility role must remain clear.

## ArchivedZone

The following paths are historical and non-authoritative as a whole:

- `docs/archive/**`
- `scripts/_archive/**`
- `scripts/validators/_archive_pre_hir/**`
- `spec/v*/**`

Version references within these paths are retained as provenance unless an
artifact is promoted back into an active surface. Current behavior remains
governed by the active compiler and canonical validation.

## DisposableDecoration

A version appearing only as a source-file banner, milestone prefix, workflow
label or decorative comment carries no authority or compatibility meaning.
Remove the version and retain only the useful technical explanation.

## Review rule

Never perform a global version replacement. Classify each active reference as
authority, provenance, compatibility fixture or disposable decoration before
changing it.

## Classified preserved active surfaces

The following active files intentionally retain historical version references:

- `.github/AUDIT_TEMPLATE.md`: historical audit criteria and evidence prompts.
- `.github/ROADMAP_POLICY.md`: dated policy revision history.
- `.github/SECURITY.md`: dated security evidence snapshots.
- `SPECIFICATION.md`: explicitly labelled historical milestone tables.
- `docs/architecture/hir-decision.md`: architecture decision provenance.
- `docs/diagnostics/fail-fast-inventory.md`: captured audit excerpts.
- `docs/runtime/LXDGE_EXTRACTION.md`: extraction and fork provenance.
- `docs/wiki/README.md`: index of historical and archived documents.

These references must not be interpreted as current compiler authority.

## Versioned technical identities

The following version values are technical identities and are not release
authority:

- parser aliases and editions retained for compatibility;
- negative and regression test fixtures;
- dependency and GitHub Action versions;
- serialized metadata or schema versions;
- graph and artifact format versions;
- dictionary dataset provenance such as `dict/core_map.json::__version`.

Changing these values requires explicit compatibility or schema review.
