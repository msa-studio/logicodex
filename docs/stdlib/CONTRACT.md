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

Normal user compilation should not validate .std.toml by default. A normal
compile resolves and compiles .ldx modules. Contract validation is a dev/CI or
explicit verification activity.

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
fields should not be silently ignored in strict validation mode.---

## 6. Validation Modes

The contract harness should support these validation modes:

    quick:
      metadata + static validation

    standard:
      metadata + static + compile + run

    full:
      standard + negative cases

    stress:
      reserved for future stress/property cases

Stage 0 default:

    standard

stress is reserved and must not run by default.

---

## 7. Test Oracle

The generic stdlib contract harness must not use core.assert as the authority
for pass/fail.

Reason: Stage 0 core.assert.eq_i64 and core.assert.is_true return 1 or 0; they
do not abort. If a generated program calls core.assert.eq_i64(...) and then
returns 0, the executable can pass even when the assertion returned 0.

Therefore the Stage 0 generic oracle is stdout comparison:

    generate one temporary program per module
    PAPAR each case expression
    run executable
    compare stdout line-by-line with expected values

Example generated program:

    import core.math;

    function main() -> I64 begin
        PAPAR core.math.abs_i64(-5);
        PAPAR core.math.pow_i64(2, 10);
        return 0;
    end

Expected stdout:

    5
    1024

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

The harness must eventually provide:

    compile timeout per module
    run timeout per module
    child process kill on timeout
    stdout/stderr size limits
    clear diagnostics on timeout

If no timeout crate is available, use standard Rust primitives:

    Command::spawn
    child.try_wait()
    Instant timeout loop
    child.kill() on timeout---

## 9. Export Validation

Every function listed in:

    [exports]
    functions = ["name"]

must exist as a public function in the matching .ldx source.

Private functions do not count.

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

Everything else under lib/core, lib/std, or lib/framework must be treated as
draft, archived proposal, reserved, or non-stdlib infrastructure until it has a
contract.

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
    lockfile---

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

## 15. Definition of Done for Stage 0 Contract Harness

Stage 0 contract harness is complete when:

    docs/stdlib/CONTRACT.md exists
    lib/core/math.std.toml exists
    lib/core/assert.std.toml exists
    src/stdlib_contract.rs or equivalent utility exists
    tests/stdlib_contract.rs exists
    contract discovery covers core/std/framework
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
