# CPB Next Roadmap Blockers

This document records the next blockers after the stdlib-core foundation merge.
It is not a replacement roadmap. It refines the existing sequence:

1. Compiler Foundation
2. Stdlib Contract Framework
3. Community Production Baseline
4. Backend Independence
5. Self-Hosting Readiness
6. Assurance Edition

Enterprise/Assurance implementation remains out of scope for the community
pre-beta line.

## Current status

Already present:

- `core.prelude` foundation
- `core.text` emptiness helpers
- `core.option` / `core.result` I64 foundations
- contract verifier and `.std.toml` sidecars
- fixed local array compiler foundation
- CPB docs and compiler subset docs

Still not production-grade:

- full diagnostics
- complete type checker
- full string semantics
- array/slice stdlib
- high-level IO
- C ABI / pointer / layout model
- package metadata and package manager

## P0 — before production-grade community baseline

### P0.1 Diagnostics hardening

Required:

- stable diagnostic code registry
- precise source spans
- causal notes
- hints/suggestions
- no silent success for unsupported code paths
- audit and remove unsafe fallback-to-zero behavior in codegen/semantic lowering

Unsupported expressions or unimplemented features must become structured compile
errors, not default `0` values.

### P0.2 Type checker hardening

Required:

- function argument type checking
- return type validation
- array/slice typing
- pointer/FFI typing
- struct/enum layout validation
- clearer diagnostics for unsupported generic or heap-backed patterns

### P0.3 Trust and documentation hygiene

Required:

- maintain `docs/stdlib/core-trust-state.md`
- keep README, CHANGELOG, roadmap, and trust state aligned
- clearly mark legacy modules as `LegacySourceOnly`
- do not treat aspirational legacy APIs as CPB dependencies

## P1 — CPB usable foundation

### P1.1 `core.array` Stage 1

Start from the fixed local array subset.

Required before broad stdlib promotion:

- array or slice parameter passing
- bounds policy
- fixed-array helper contracts
- type checking for array/slice use
- tests proving helper calls through normal import path

Do not start heap `Vec`/`List` as the first collection target.

### P1.2 `core.text` / `core.string` Stage 2

Required:

- arbitrary String equality
- length
- concat policy
- substring/slice policy
- UTF-8 or byte-string policy
- String ABI and ownership rules

### P1.3 `std.path`

Required before high-level file IO and package tooling:

- join
- normalize
- basename/dirname
- extension
- absolute/relative policy

### P1.4 `std.file` and `std.io`

Required:

- callable IO functions
- file exists/read/write
- stdin/stdout/stderr abstractions
- `IoError` model
- `Result<T, IoError>` direction
- runtime capability/profile policy

## P2 — ecosystem and C import

### P2.1 FFI-C1: libm end-to-end

First C import target:

- primitive numeric C ABI only
- `sin`, `cos`, `sqrt`, `pow`
- manifest allowlist
- linker proof
- compile + run tests

Do not start with Raylib.

### P2.2 Package metadata reader

Wait until string/path/file/io/diagnostics foundations are stable enough.

### P2.3 `ldx-pkg` MVP

Required:

- local package metadata
- dependency path resolution
- build/check command
- contract verification integration
- no network registry initially unless explicitly scoped

### P2.4 FFI-C2 tiny custom C library

Required:

- simple exported C functions
- `char*` input
- pointer/null diagnostics

### P2.5 FFI-C3 sqlite3 minimal

Required:

- opaque pointer `sqlite3*`
- open/close
- exec simple query
- error string
- null and ownership handling

### P2.6 FFI-C4 Raylib minimal

Required:

- struct ABI layout
- platform linker handling
- `Color` / `Vector2`
- minimal draw loop

## Package manager hold rule

Do not prioritize `ldx-pkg` before these are stable enough:

- string
- path
- file/io
- diagnostics
- package metadata reader

## Near-term sprint order

Recommended order:

1. main-readiness docs and CHANGELOG
2. merge stdlib-core foundation to main
3. trust-state and roadmap blocker hygiene
4. diagnostics hardening
5. type checker hardening
6. `core.array` Stage 1
7. `core.text/string` Stage 2
8. `std.path`
9. `std.file/std.io`
10. FFI-C1 libm end-to-end
11. package metadata reader
12. `ldx-pkg` MVP
13. sqlite3
14. Raylib

### P1.4a `std.file` / `std.io` boundary record

`std.file` and callable `std.io` remain blocked by capability/runtime-profile
policy. The first CPB-safe step is the boundary record and negative regression
guard, not public file/IO APIs.

See `docs/architecture/std-file-io-boundary.md`.
