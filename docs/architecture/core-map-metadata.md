# Core Map Metadata Contract

## Status

This document is active architecture guidance for the Logicodex dictionary resources.

It defines the current interpretation of metadata carried by the canonical core map and its bundled editor snapshot. It does not define compiler language semantics or the active compiler release.

## Resources

The canonical dictionary resource is:

- `dict/core_map.json`

The VS Code extension carries a separate bundled snapshot at:

- `extensions/vscode-logicodex/resources/core_map.json`

The bundled snapshot supports editor tooling. It is not automatically authoritative over the canonical dictionary and must not be force-synchronized without explicit review.

## `__version` meaning

The `__version` field currently present in `dict/core_map.json` is dictionary dataset provenance.

Its value records a historical dictionary transformation or dataset state. It is not:

- the active Logicodex compiler version;
- the compiler compatibility authority;
- a language specification version;
- a guarantee that every dictionary entry reflects that historical release.

The active compiler version must be obtained from the repository's canonical version authority, not from `dict/core_map.json::__version`.

The existing `__version` value is preserved until a separately approved metadata migration defines a replacement field contract and compatibility policy.

## Schema metadata

A future `schema_version` field, if introduced, must describe the shape and interpretation of the JSON document only.

A schema version must not be treated as the compiler version or as proof that the canonical and bundled maps contain identical vocabulary.

## Synchronization policy

The canonical map and the bundled VS Code map may differ intentionally.

Before synchronizing either map, review at least:

- token identities;
- canonical spelling and casing;
- aliases;
- `expert` values;
- `primary_ms` values;
- status fields;
- descriptions;
- fields present in only one map.

A broad copy, global replacement, or automatic whole-file synchronization is prohibited unless a dedicated compatibility review proves that every differing field has the same ownership and meaning.

## Version-reference hygiene

Historical release references inside descriptions may be retained when they are clearly labeled as provenance.

Active or normative wording must not present an older release as the current compiler authority.

Where historical provenance and current behavior are both relevant, documentation should state that current behavior is governed by the active compiler and canonical validation.

## Change control

Changes to core-map metadata require:

1. consumer and repository-reference review;
2. explicit classification of the field as schema, dataset, compatibility, or historical provenance metadata;
3. validation that compiler-version authority remains separate;
4. review of canonical-map and bundled-snapshot drift;
5. focused and full-integrity validation appropriate to the change.

Renaming or removing `__version`, introducing `schema_version`, or synchronizing metadata into the VS Code snapshot requires a separate reviewed change.
