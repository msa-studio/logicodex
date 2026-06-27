
# Logicodex Stdlib Contract Versioning

Status: Active policy
Scope: Community stdlib contract sidecars
Current schema version: `0`

## Contract Schema Version

Stage 0 contracts must use:

```toml
[contract]
version = 0
```

`contract.version = 0` means the sidecar uses the initial Community validation
schema for pure/helper-oriented stdlib modules.

The current schema is intentionally small and strict. Unknown keys are rejected
by the verifier so accidental contract drift is detected early.

## Required Tables

A Stage 0 `.std.toml` contract must contain exactly these top-level sections:

```text
[contract]
[module]
[validation]
[limits]
[exports]
[constraints]
[capabilities]
[[cases]]
```

## Required Metadata Fields

The `[module]` table must define:

```text
name
layer
stage
profile
pure
extern
```

For `core.*` Stage 0 modules:

```text
layer = "core"
stage = 0
profile = "core"
pure = true
extern = false
```

The `[validation]` table must define:

```text
metadata
static
compile
run
stress
```

The `[limits]` table must define:

```text
compile_timeout_ms
run_timeout_ms
stdout_limit_bytes
max_cases
```

The `[exports]` table must define:

```text
functions
```

The `[constraints]` table must define:

```text
allowed_imports
forbidden_imports
forbidden_features
```

The `[capabilities]` table must define:

```text
requires
```

## Source Hash Policy

Hash evidence must be generated from the paired `.ldx` source file.

For a contract:

```text
lib/core/<name>.std.toml
```

The paired source is:

```text
lib/core/<name>.ldx
```

The verifier must be able to emit current SHA-256 source hash evidence for the
paired `.ldx` file.

Source hash evidence must be regenerated whenever the `.ldx` source changes.

## Contract Hash Policy

Hash evidence must be generated from the `.std.toml` sidecar file itself.

Contract hash evidence must be regenerated whenever contract metadata, exports,
constraints, limits, capabilities, or run-cases change.

Current Stage 0 uses emitted hash evidence as Community validation output. If a
future stored-hash manifest is introduced, the verifier must compare stored
hashes against the current `.ldx` and `.std.toml` files and fail on mismatch.

## Compatibility Rules

A change is compatible when it does not remove or change the meaning of an
existing exported helper.

Compatible examples:

```text
adding a new run-case
adding a new helper export with tests
tightening documentation without changing behaviour
adding validation coverage
```

A change may break compatibility when it:

```text
removes an exported function
renames an exported function
changes function parameters
changes return semantics
changes truthiness conventions
changes range/bounds semantics
changes required capabilities
changes purity or extern status
widens the module beyond its declared profile
```

Breaking changes require an explicit compatibility note and must not be hidden
inside routine contract refresh work.

## Module Contract Evolution

Stage 0 contracts remain at schema version `0` until the contract format itself
changes.

Adding new functions to a module does not automatically require a schema version
change. It requires updated exports, run-cases, focused e2e tests, hash evidence,
and green verification.

A future schema version may be introduced only when the meaning or required
shape of `.std.toml` changes.
