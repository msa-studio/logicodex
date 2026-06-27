
# Stdlib Migration Status

Status: Active policy
Scope: Community stdlib migration and trust-state classification

## Trust States

Use these trust states consistently:

```text
ContractVerified
PartialContract
LegacyUnverified
Experimental
OutOfScope
InvalidContract
```

## ContractVerified
- `core.text` — CPB Phase 1 empty/non-empty text predicates, stage 1 contract-backed.

`ContractVerified` means the module follows the Stage 0 contract-backed pattern:

```text
.ldx source
.std.toml contract
declared exports
profile metadata
source hash evidence
contract hash evidence
run-cases
focused Rust e2e import test
verify script coverage
green verification
```

Current ContractVerified Stage 0 modules:

```text
core.math
core.assert
core.bits
core.compare
core.bool
core.range
```

## PartialContract

`PartialContract` means only a declared subset of a surface is contracted.

Use this for FFI and external-library surfaces such as Raylib.

Policy:

```text
Imported symbol must be contracted.
Unimported symbol is OutOfScope.
Newly imported symbol must add contract in the same PR.
```

Do not classify an entire external library as `LegacyUnverified`.

## LegacyUnverified

`LegacyUnverified` means:

```text
works today
not contract-verified
not trusted by default
migration deferred until owning subsystem is touched
```

Use `LegacyUnverified` for older hardcoded/compiler-wired modules or existing
stdlib/runtime-adjacent files that predate the Stage 0 contract discipline.

A LegacyUnverified module should not be treated as part of the trusted Stage 0
surface.

## Experimental

`Experimental` means the module or surface is being explored and is not yet part
of the stable contract-backed stdlib.

Experimental modules must not be promoted into CPB without a contract, tests,
and migration review.

## OutOfScope

`OutOfScope` means the symbol, module, or external capability is intentionally
not covered by the current contract.

For FFI libraries:

```text
unimported symbol = OutOfScope
```

OutOfScope is not a trust claim.

## InvalidContract

`InvalidContract` means the contract exists but fails validation.

Examples:

```text
missing paired .ldx source
invalid TOML
wrong schema version
missing metadata
declared export not public in source
public source export missing from contract
missing run-cases
forbidden import
forbidden feature token
invalid profile metadata
run-case failure
```

InvalidContract must fail verification for Stage 0 modules.

## Rule for New Stdlib Modules

New stdlib modules must not bypass the contract discipline.

A new Stage 0 module must target `ContractVerified`.

A new CPB module must not begin until Stage 0 policy and validation gates are
stable and green.

## Rule for Existing Hardcoded or Legacy Modules

Existing hardcoded, compiler-wired, runtime-adjacent, or pre-contract modules
may remain `LegacyUnverified`.

Do not migrate legacy modules opportunistically unless the owning subsystem is
being touched.

Migration must be explicit and should include:

```text
contract sidecar
exports declaration
run-cases
focused e2e test
verify script coverage
trust-state update
```

## Migration Triggers

A legacy or partial surface should be migrated when:

```text
its owning subsystem is actively modified
it becomes part of CPB
it becomes part of public docs/examples
it is imported by a ContractVerified module
it affects validation, safety, or compatibility
it becomes part of a release promise
```

## Community Boundary

This document is Community validation policy only.

Do not implement Enterprise/Assurance systems here:

```text
LDX-AUD
LDX-AIC
Enterprise Trust Registry
Evidence Engine
Compliance Reports
Authority Reports
Contract Drift Detection
Formal Verification
Advanced Assurance Governance
```

## Legacy Core Inventory Snapshot

Snapshot source: local static inventory plus import/load smoke test.

This inventory is not a trust promotion. It exists to prevent accidental
assumptions that older `lib/core/*.ldx` files are part of the ContractVerified
Stage 0 surface.

### LegacyLoadable

These legacy modules can currently be imported by a minimal smoke program, but
they are not contract-verified and are not trusted by default:

- `core.gate`
- `core.thread`

Policy:

- `LegacyLoadable` means import/load smoke passes.
- `LegacyLoadable` does not mean `ContractVerified`.
- Migration is deferred until the owning subsystem is touched.

### LegacyNotFunctioning

These legacy modules currently fail import/load smoke and should be treated as
not functioning for stdlib trust purposes:

- `core.capability`
- `core.file`
- `core.io_error`
- `core.memori`
- `core.result`
- `core.ring_buffer`
- `core.scheduler`
- `core.shard_manifest`
- `core.sync`

Policy:

- `LegacyNotFunctioning` means the file exists but does not currently load
  through the standard module import path.
- Do not patch these ad hoc as part of Stage 0 policy/gate work.
- Rebuild or migrate them later according to roadmap priority and subsystem
  owner.

### Static Function Surface Notes

Static inventory found no legacy public functions outside the ContractVerified
Stage 0 set. Some legacy files contain private function-like declarations only.

Observed static states:

- `LegacyNoFunctions`
  - `core.capability`
  - `core.gate`
  - `core.result`
  - `core.ring_buffer`
  - `core.shard_manifest`
  - `core.thread`
- `LegacyPrivateOnly`
  - `core.file`
  - `core.io_error`
  - `core.memori`
  - `core.scheduler`
  - `core.sync`

Policy:

- `LegacyNoFunctions` means no public/private function declarations were
  detected.
- `LegacyPrivateOnly` means private function declarations were detected, but no
  public contract surface exists.
- Neither state is a CPB-ready trust claim.

### Rebuild Rule

LegacyNotFunctioning modules should be rebuilt from scratch or migrated under
contract when their stage priority arrives.

Minimum migration requirement:

- new or repaired `.ldx` source
- matching `.std.toml` contract
- declared public exports
- run-cases
- focused e2e import test
- verify script coverage
- trust-state update
- green verification before commit/push
