# Logicodex Stdlib Stage 0

Status: Active policy
Scope: Community stdlib-core only
Phase: Stdlib Contract Framework -> Stage 0 Pure Core Library Foundation

## Purpose

Stage 0 exists to prove the stdlib contract discipline before production-facing
Community Production Baseline (CPB) modules are added.

Stage 0 is not full CPB. It is the repeatable foundation for adding small,
pure, contract-backed core helpers using native Logicodex `.ldx` source and a
`.std.toml` contract sidecar.

Do not start CPB Phase 1 modules until the Stage 0 gates are documented,
repeatable, and green.

## Current Stage 0 Modules

The current ContractVerified Stage 0 modules are:

```text
core.math      16 exports
core.assert     4 exports
core.bits       7 exports
core.compare    5 exports
core.bool       6 exports
core.range      6 exports
```

Current Stage 0 helper surface:

```text
6 Stage 0 core modules
44 exported helper functions
```

## Definition of Done

A Stage 0 module is complete only when it has:

1. Native Logicodex `.ldx` source.
2. Matching `.std.toml` contract sidecar.
3. Declared exports in `[exports].functions`.
4. Required profile metadata in `[module]`.
5. Source hash evidence from the paired `.ldx` file.
6. Contract hash evidence from the `.std.toml` file.
7. `[[cases]]` run-cases or validation examples.
8. Focused Rust e2e import test.
9. Coverage in `scripts/dev/verify_stdlib_stage0.sh`.
10. Green verification before commit and push.

## Required File Pattern

Every ContractVerified Stage 0 module must follow this pattern:

```text
lib/core/<name>.ldx
lib/core/<name>.std.toml
tests/stdlib_core_<name>.rs
```

The contract sidecar is the authority for the module's exported public helper
surface. Public functions in the `.ldx` source must match the declared exports
in the contract.

## Validation Pattern

The required Stage 0 validation pattern is:

```text
.ldx source
+ .std.toml contract
+ metadata validation
+ source/contract hash evidence
+ run-cases validation
+ focused Rust e2e tests
+ standard verify script
+ commit/push only after green
```

## Stage 0 Module Rules

Stage 0 modules must be pure/helper-oriented.

For `core.*` Stage 0 modules:

```text
module.stage = 0
module.layer = "core"
module.profile = "core"
module.pure = true
module.extern = false
capabilities.requires = []
```

Stage 0 modules must not require OS services, dynamic allocation, files,
networking, syscalls, runtime profiles, actor runtime, FFI, or Enterprise/
Assurance features.

Forbidden imports for `core.*` Stage 0 modules include:

```text
std.*
framework.*
```

Forbidden feature tokens include:

```text
extern
malloc
free
file
network
syscall
```

## CPB Hold Point

Do not add these modules yet:

```text
prelude.ldx
string.ldx
collections.ldx
high-level io.ldx
```

CPB Phase 1 begins only after:

```text
docs/stdlib_stage0.md exists
docs/stdlib_contract_versioning.md exists
docs/architecture/stdlib-migration-status.md exists
verify_stdlib_stage0.sh checks Stage 0 modules
verify_stdlib_contracts.py validates contract metadata and hash evidence
current Stage 0 modules remain green
```

## Non-Goals

Stage 0 does not implement:

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

Stage 0 policy is Community validation only.
