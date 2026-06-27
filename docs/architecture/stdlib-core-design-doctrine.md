# Stdlib Core Design Doctrine

Status: Active CPB design doctrine
Scope: Logicodex Community stdlib/core
Depends on: Stdlib Stage 0, CPB Self-Hosting Runway, Compiler Subset

## Purpose

This document defines how Logicodex core libraries should be designed before
expanding toward self-hosting, package manager work, and real development tools.

The goal is not only to add functions. The goal is to build a modern,
contract-backed, backward-compatible standard foundation that fits Logicodex
identity.

## Design Identity

Logicodex core libraries must follow these principles:

- semantic truth first
- contract-backed behavior
- explicit compatibility policy
- deterministic diagnostics
- bilingual-friendly surface where safe
- canonical internal meaning
- no accidental trust promotion
- no ad hoc legacy repair
- self-hosting readiness

## Canonical Modern API Rule

Every new trusted stdlib/core module must define a modern canonical API.

Canonical APIs should prefer:

- clear English internal names
- explicit type suffixes where useful
- deterministic behavior
- small composable functions
- contract sidecars
- run-cases
- focused e2e tests
- stable diagnostics for failure cases

One canonical API owns the meaning. Compatibility aliases may exist, but they
must not become the semantic authority.

## Legacy Compatibility Rule

Legacy support is allowed, but it must not corrupt the canonical API.

Allowed:

- safe aliases
- wrapper functions
- migration notes
- compatibility docs
- deprecated-but-working exports
- bilingual examples

Not allowed:

- promoting broken legacy modules as trusted
- silently changing semantics to match old bugs
- bypassing contracts to keep old behavior
- making legacy names the semantic authority
- adding compatibility shims without tests

## Bilingual Surface Rule

Logicodex may support both modern and legacy or bilingual style at user-facing
syntax boundaries.

However, downstream compiler stages and stdlib contracts should operate on a
canonical representation.

Policy: user-facing syntax may be bilingual, but contract authority must be
canonical.

## Backward Compatibility Rule

Once a stdlib/core export is marked ContractVerified, it must not be removed or
renamed without a compatibility decision.

Allowed evolution:

- add new exports
- add aliases
- add stricter validation if behavior is unchanged
- add more run-cases
- increase documentation clarity

Breaking evolution requires:

- contract version review
- migration note
- compatibility period or explicit unstable status
- focused regression tests

## Trust State Rule

Trust state must be explicit.

Important states:

- ContractVerified
- PartialContract
- LegacyLoadable
- LegacyNotFunctioning
- LegacyNoFunctions
- LegacyPrivateOnly
- Experimental
- OutOfScope
- InvalidContract

A module being present in lib/core does not make it trusted.

## Legacy Module Rule

Legacy modules should be treated as source material, not authority.

If a legacy module is useful, migrate it through the modern path:

1. decide canonical module name
2. define canonical exports
3. write .ldx source
4. write .std.toml contract
5. add run-cases
6. add focused e2e tests
7. update trust state
8. run verification

Do not patch legacy files casually just to make them parse.

## Self-Hosting Priority Rule

Core libraries needed by compiler-shaped programs have priority over convenience
libraries.

Priority order:

1. text/string
2. option/result/error
3. array/slice/list
4. path
5. file/io
6. diagnostics
7. package metadata
8. package manager
9. development tools

## Package Manager Hold Rule

Package manager work must wait until the foundational library blockers are
stable enough.

Minimum blockers before package manager MVP:

- text/string
- option/result/error
- path
- file/io
- diagnostics
- module/import regression tests

## Development Tools Hold Rule

Formatter, LSP, test runner, doc generator, and project scaffolder must wait
until package metadata and file/io foundations are stable enough.

Development tools may be planned earlier, but they should not become CPB proof
before their dependencies are contract-backed.

## Module Design Checklist

Before adding a new trusted core module:

- confirm its CPB priority
- choose canonical module name
- decide whether legacy aliases are needed
- define public exports
- define unsupported behavior
- write contract cases first or together with source
- add focused e2e tests
- update verify script coverage
- update trust state docs
- run green verification before commit/push

## Core Text/String Note

Current compiler support for string-like values is partial.

Known baseline:

- string literal printing compiles
- function parameter and return using String compiles
- some local binding syntax may not parse depending on chosen syntax form

Therefore core.text or core.string must start with the smallest reliable surface
and avoid overclaiming.

Minimum first wave should focus on what the current compiler can prove.

## Non-Goals

This doctrine does not implement:

- enterprise assurance
- AI governance
- package manager
- full self-hosting
- dynamic plugin APIs
- unsafe FFI shield
- remote registry
