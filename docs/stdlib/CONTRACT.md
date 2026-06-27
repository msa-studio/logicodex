# Logicodex Standard Library Contract Specification

Status: Stage 0 architecture contract for feature/stdlib-core.

This document defines how official Logicodex library modules enter the compiler
ecosystem from this point forward.

A Logicodex library module is a contract-compliant static module, not a runtime
plugin.

The compiler must not hardcode individual modules such as core.math,
core.assert, std.io, or framework.http. The compiler may know official library
namespace categories, but each module itself must be declared and proven by
contract metadata.

Official rule:

    New module = .ldx implementation + .std.toml contract, no compiler change.
    New language/runtime capability = compiler/runtime change.

---

## 1. Identity

The stdlib contract exists to keep Logicodex aligned with its identity as a
Meaning Authority.

Every official library module must declare:

    what meaning it introduces
    what layer it belongs to
    what profile/runtime it assumes
    what imports it is allowed to use
    what exports it provides
    what capabilities it requires
    what is explicitly unsupported
    what acceptance cases prove it

The contract is not runtime metadata. It is used by documentation, tests, CI,
future tooling, and future diagnostics.

**Important:** The contract validator (`tools/verify_stdlib_contracts.py`) is a dev/CI
validation tool only. Normal user compilation does not validate `.std.toml` by default.
A normal compile resolves and compiles `.ldx` modules. Contract validation is a dev/CI
or explicit verification activity.

---

## 2. Official Library Layers

Logicodex reserves three official library layers:

    core.*        pure Logicodex foundation
    std.*         OS/libc/profile-dependent official standard library
    framework.*   higher-level official framework layer

### 2.1 core.*

core.* modules must be pure Logicodex and bare-compatible.

Required properties:

    pure = true
    extern = false
    profile = "core"
    no OS dependency
    no libc dependency
    no malloc/free dependency
    no file/network/syscall dependency
    no std.* import
    no framework.* import

Stage 0 active core.* modules:

    core.math
    core.assert

### 2.2 std.*

std.* modules are official system-facing libraries. They may depend on OS,
libc, extern C, or runtime profile support in future stages.

Stage 0 has no active std.* modules.

Future std.* modules must declare:

    profile requirement
    capability requirement
    extern/FFI boundary if used
    allowed imports
    acceptance cases

core.* must not depend on std.*.

### 2.3 framework.*

framework.* is reserved for future official higher-level framework modules.

Stage 0 has no active framework.* modules.

For now, framework.* is documented as reserved/future only. Do not implement
framework modules in Stage 0.

Future framework.* modules must declare:

    runtime/profile assumptions
    capabilities
    dependencies on core/std/framework
    acceptance cases
    diagnostic expectations
---

## 3. Namespace Resolution Contract

The compiler may recognize these official namespace prefixes:

    core
    std
    framework

The compiler must not hardcode individual modules.

Resolution target:

    import core.math;        -> <library-root>/core/math.ldx
    import std.io;           -> <library-root>/std/io.ldx
    import framework.http;   -> <library-root>/framework/http.ldx
    import app.models;       -> filesystem-relative app/models.ldx

The library root order is:

    1. LOGICODEX_STD
    2. <compiler-dir>/lib
    3. ./lib

The current implementation may still use the historical name std_root. The next
refactor should rename this concept to library_root because it covers core.*,
std.*, and framework.*.

---

## 4. Module Contract File

Every official library module must eventually have:

    lib/<layer>/<module>.ldx
    lib/<layer>/<module>.std.toml

Example:

    lib/core/math.ldx
    lib/core/math.std.toml

Stage 0 contract files to add:

    lib/core/math.std.toml
    lib/core/assert.std.toml

---

## 5. Metadata Schema

Minimum schema:

    [contract]
    version = 0

    [module]
    name = "core.math"
    layer = "core"
    stage = 0
    profile = "core"
    pure = true
    extern = false

    [validation]
    metadata = true
    static = true
    compile = true
    run = true
    stress = false

    [limits]
    compile_timeout_ms = 10000
    run_timeout_ms = 2000
    stdout_limit_bytes = 65536
    max_cases = 50

    [exports]
    functions = [
      "abs_i64"
    ]

    [constraints]
    allowed_imports = ["core.*"]
    forbidden_imports = ["std.*", "framework.*"]
    forbidden_features = ["extern", "malloc", "free", "file", "network", "syscall"]

    [capabilities]
    requires = []

    [[cases]]
    name = "abs negative"
    expr = "core.math.abs_i64(-5)"
    expect_i64 = 5

The schema is intentionally small for Stage 0. It may grow later, but unknown
fields should not be silently ignored in strict validation mode.

### 5.1 Versioning and Integrity Policy

Stage 0 currently uses `contract.version = 0`. This means:

    version 0 = Stage 0 contract schema
    no compatibility promise beyond the current Stage 0 validator
    no source hash binding yet

Before the stdlib moves from Stage 0 toward a stable public contract surface,
contract metadata should grow in a controlled way rather than through ad hoc
fields. The expected next evolution is:

    schema_version       version of the contract metadata schema
    api_version          version of the module's public API contract
    compatibility        compatibility class, for example experimental/stable/deprecated
    source_hash          deterministic hash of the paired `.ldx` source file
    contract_hash        deterministic hash of the normalized `.std.toml` contract

The validator should eventually use these fields to detect accidental drift
between source and contract, distinguish schema changes from API changes, and
support compatibility checks. Stage 0 does not enforce these fields yet; this
section records the intended policy so future validators and CI can implement it
without changing the meaning of existing `version = 0` contracts.

Current Stage 0 tooling can emit advisory SHA-256 evidence for each contract
sidecar and its paired `.ldx` source file:

    python3 tools/verify_stdlib_contracts.py --emit-hashes

This output is for review and drift investigation only. It is not yet a schema
field, compatibility promise, or security guarantee.

---

## 6. Validation Modes

The contract harness (`tools/verify_stdlib_contracts.py`) supports three validation modes:

    1. Plain Validation (Default):
       Validates metadata schema, layer rules, export completeness, and static constraints.
       Command: python3 tools/verify_stdlib_contracts.py

    2. Hash Evidence:
       Performs plain validation and prints advisory SHA-256 hashes for each
       `.std.toml` sidecar and paired `.ldx` source file.
       Command: python3 tools/verify_stdlib_contracts.py --emit-hashes

    3. Run Cases Validation:
       Performs plain validation PLUS compiles and runs each `[[cases]]` expression
       through Logicodex, comparing the bounded stdout against `expect_i64`.
       Command: python3 tools/verify_stdlib_contracts.py --run-cases

In CI, plain validation runs during the `check` job, while full `--run-cases`
validation runs during the `test` job against the compiled release binary. Hash
evidence is available for local review and drift investigation.

---

## 7. Test Oracle

The generic stdlib contract harness must not use core.assert as the authority
for pass/fail.

Reason: Stage 0 core.assert.eq_i64 and core.assert.is_true return 1 or 0; they
do not abort. If a generated program calls core.assert.eq_i64(...) and then
returns 0, the executable can pass even when the assertion returned 0.

Therefore the Stage 0 generic oracle is bounded stdout comparison:

    generate one temporary program per case
    PAPAR the case expression
    run executable
    compare stdout tokens with expected_i64

Example generated program for `expr = "core.math.abs_i64(-5)"`:

    import core.math;
    PAPAR core.math.abs_i64(-5);

Expected stdout:

    5

core.assert may still be tested as a normal module through PAPAR
core.assert.eq_i64(...), but it must not be the harness oracle.

---

## 8. Anti-Hang Requirements

Generated compile/run tests must be bounded.

Default limits:

    compile_timeout_ms = 10000
    run_timeout_ms = 2000
    stdout_limit_bytes = 65536
    max_cases = 50

The harness provides:

    compile timeout per case
    run timeout per case
    child process kill on timeout (via subprocess.run timeout)
    stdout/stderr size limits
    clear diagnostics on timeout

---

## 9. Export Validation

Every function listed in:

    [exports]
    functions = ["name"]

must exist as a public function in the matching .ldx source.

Private functions do not count.

The verifier enforces a bidirectional check:
1. All declared exports must be public in the source.
2. All public functions in the source must be declared as exports.

Failure example:

    stdlib contract export missing: core.math.abs_i64

---

## 10. Layer Validation Rules

### core.*

Reject if source contains or declares:

    extern
    import std.
    import framework.
    malloc
    free
    file
    network
    syscall
    actor/service runtime dependency

### std.*

Future std.* modules must declare profile and capability requirements when
OS/libc/extern behavior exists.

### framework.*

Future framework.* modules must declare runtime/profile assumptions. Stage 0
only reserves the namespace.

---

## 11. Stage 0 Active Modules

Only these modules are official active stdlib modules in Stage 0:

    core.math
    core.assert

Other files under `lib/core/`, such as `result.ldx`, `file.ldx`, `io_error.ldx`, `memori.ldx`, `ring_buffer.ldx`, `scheduler.ldx`, `sync.ldx`, `thread.ldx`, `gate.ldx`, `capability.ldx`, and `shard_manifest.ldx`, are design/future/reference modules unless they have an active `.std.toml` sidecar and green oracle cases. They must not be presented as stable active stdlib APIs yet.

---

## 12. Not Built in Stage 0

Deferred, not rejected:

    core.bits
    core.rand
    core.mem
    std.io
    std.fs
    std.time
    std.str
    std.mem
    std.mathf
    std.c
    std.net
    std.json
    framework.http
    framework.actor
    framework.service
    framework.game
    framework.db
    extern-in-modules
    cross-module struct/enum/type/const exports
    dynamic runtime plugins
    package registry
    remote fetch
    lockfile

---

## 13. Diagnostic Direction

Contract validation should later integrate with the Logicodex Diagnostic
Intelligence Pipeline, LDX-DIP.

Future diagnostics should include:

    error code
    stage
    module
    contract path
    metadata key
    source span if available
    rule violated
    safe fix suggestion
    machine-readable fields

Example:

    LDX-DIP-CONT-001
    Contract violation: core module imported std module
    Module: core.math
    Contract: lib/core/math.std.toml
    Rule: core modules cannot import std.*
    Fix: remove std.* dependency or move the module to std.*

Diagnostics are structured evidence, not just plain text messages.

---

## 14. No Silent Fallback

Unsupported behavior must not silently become success.

Forbidden:

    unknown contract field silently ignored in strict mode
    missing export treated as valid
    unsupported import treated as filesystem fallback
    failed case ignored
    timeout ignored
    dummy zero returned as success

Required:

    contract error
    diagnostic error
    semantic gate failure
    test failure

---

## 15. Authoring Flow for Future Modules

To add a new official stdlib module:

1. **Implement in Logicodex:** Create the `.ldx` file (e.g., `lib/core/math.ldx`).
2. **Add Contract Sidecar:** Create the matching `.std.toml` file (e.g., `lib/core/math.std.toml`).
3. **Declare Exports:** Ensure `[exports]` perfectly matches the `public function` list in the `.ldx` file.
4. **Add Contract Cases:** Add `[[cases]]` covering the behavioral oracle of the module.
5. **Verify Locally:**
   ```bash
   # Run metadata and static checks
   python3 tools/verify_stdlib_contracts.py
   
   # Build the compiler
   cargo build
   
   # Run behavioral cases using the compiled binary
   python3 tools/verify_stdlib_contracts.py --run-cases --bin target/debug/logicodex
   ```
6. **Add e2e Tests:** If needed, add specific Rust e2e tests in `tests/` for edge cases not covered by the contract harness.

---

## 16. Contract Validator Command Line

The contract validator is located at `tools/verify_stdlib_contracts.py`.

Usage:

    python3 tools/verify_stdlib_contracts.py [contracts...] [--emit-hashes] [--run-cases] [--bin BIN]

Arguments:
- `contracts`: Optional list of contract files to verify. Defaults to `lib/**/*.std.toml`.
- `--emit-hashes`: Print advisory SHA-256 evidence for each contract and paired source file.
- `--run-cases`: Compile and run each `[[cases]]` expression through Logicodex and compare bounded stdout.
- `--bin`: Path to a prebuilt logicodex binary. Defaults to `target/debug/logicodex`, `target/release/logicodex`, then `cargo run`.

---

## 17. Definition of Done for Stage 0 Contract Harness

Stage 0 contract harness is complete when:

    docs/stdlib/CONTRACT.md exists
    lib/core/math.std.toml exists
    lib/core/assert.std.toml exists
    tools/verify_stdlib_contracts.py exists
    CI runs metadata validation
    CI runs --run-cases against a built Logicodex binary
    stdout oracle is used
    compile/run is bounded
    exports are validated
    core layer rules are validated
    manual stdlib tests remain green
    normal compile does not read .std.toml by default

Final status target:

    Stdlib Contract Harness Stage 0 complete.
    Official active modules: core.math, core.assert.
    Reserved layers: std.*, framework.*.
    No per-module compiler hardcoding.
    Future stdlib work follows .ldx + .std.toml contract.
