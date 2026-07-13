# Logicodex Core Stdlib Trust State

Status owner: Community compiler / CPB roadmap  
Scope: `lib/core`, selected `stdlib/` legacy files, and compiler-proven foundations.  
Purpose: prevent legacy source material from being mistaken for trusted stdlib.

## Trust levels

| Trust level | Meaning |
|---|---|
| `ContractVerified` | `.ldx` module has `.std.toml` contract metadata and passes the stdlib contract verifier. |
| `CompilerFoundationPartial` | Compiler feature exists and is tested, but stdlib API is not production-ready yet. |
| `LegacySourceOnly` | Legacy module may contain useful ideas, but is not CPB-trusted and must not be used as an authority. |
| `DocumentationOnly` | File documents old ABI/runtime behavior and must not be treated as a callable stdlib module. |
| `DeferredBlockedByCompiler` | Valid roadmap target, but compiler foundation is not ready. |
| `DeferredBlockedByRuntimeCapability` | Valid roadmap target, but runtime/capability/error model is not ready. |

## ContractVerified — Stage 0 scalar/core helpers

These modules are trusted as small, pure, contract-backed foundation modules:

- `core.assert`
- `core.math`
- `core.bits`
- `core.bool`
- `core.compare`
- `core.range`

## ContractVerified — CPB Phase 1 foundation helpers

These modules are trusted, but their scope is intentionally narrow:

- `core.prelude`
  - explicit-import scalar bootstrap helpers
  - not a magic auto-prelude
  - temporary duplication with `core.bool` is acceptable until inter-module re-export/delegation is trusted
- `core.text`
  - empty/non-empty text predicates and emptiness selection only
  - not full `core.string`
  - arbitrary non-empty String equality, length, concat, substring, UTF-8 traversal, and String ABI remain deferred
- `core.option`
  - `Option<I64>` helpers only
  - generic `Option<T>`, nested options, map/and_then, and heap-backed payloads remain deferred
- `core.result`
  - `Result<I64, I64>` helpers only
  - generic `Result<T, E>` and `Result<T, IoError>` remain deferred

## CompilerFoundationPartial

### Collections fixed local arrays

The compiler has proven the first fixed-array subset:

- fixed array type syntax: `[T; N]`
- array literal syntax: `[a, b, c]`
- index read: `xs[i]`
- index assignment: `xs[i] = v`
- HIR lowering for fixed arrays
- LLVM local array storage as `[N x i64]`

This does not make `core.array`, `core.slice`, `Vec`, `List`, heap collections,
or dynamic buffers production-ready.

Next required compiler proofs:

- array or slice parameter passing
- array/slice ABI policy
- bounds-check policy
- array/slice type checking
- non-local array semantics

## LegacySourceOnly

Legacy modules are source material, not authority. They must be migrated through
the modern contract-backed path before being used as CPB dependencies.

Legacy examples include:

- `core.file`
- `core.io_error`
- `core.memori`
- `core.ring_buffer`
- `core.scheduler`
- `core.shard_manifest`
- `core.sync`
- `core.thread`
- `core.capability`
- `core.gate`

If any legacy module contains aspirational generic `Result<T, E>`, generic
buffers, or high-level IO ideas, those ideas must be reintroduced only through
contract-backed `.ldx` + `.std.toml` + tests + verifier evidence.

## DocumentationOnly / legacy ABI notes

Files such as old `stdlib/io.ldx` style ABI notes must not be treated as trusted
stdlib APIs unless migrated to the contract-backed pattern.

## Current CPB blockers

### `core.array` / `core.slice`

Not blocked primarily by heap allocation or generics. The first blockers are:

- array/slice passing ABI
- bounds policy
- type checking
- non-local array semantics
- contract-backed fixed-array helper surface

### `core.string`

Blocked by representation and memory policy:

- String ABI representation
- ownership and lifetime model
- arbitrary equality
- length
- concat/runtime helper policy
- substring/slice policy
- UTF-8 vs byte-string policy
- FFI `char*` boundary

### `std.file` / `std.io`

Blocked by runtime/capability/error model:

- callable IO functions
- path handling
- file read/write
- stdin/stdout/stderr abstraction
- `Result<T, IoError>` direction
- runtime profile and capability policy

### C import / FFI

Blocked by ABI completeness:

- stable FFI declaration syntax
- C primitive mapping
- pointer types
- `char*`, `void*`, `size_t`
- struct layout / alignment / padding
- static/dynamic linker integration
- manifest allowlist and unsafe marker
- ownership/null/error policy

### P1-B3a fixed-array boundary

The compiler-proven fixed-array subset is tracked as a CPB boundary, not as a
stable public stdlib module. `core.array` and `core.slice` remain unclaimed until
they are implemented through the contract-backed `.ldx + .std.toml + tests`
path.

See `docs/architecture/p1b3-fixed-array-cpb-boundary.md`.

### P1-B3b array ABI blocker

`core.array` and `core.slice` remain unclaimed because array parameters and
array return values are still codegen barriers. The proven subset is local
fixed-array use only.

See `docs/architecture/p1b3-array-codegen-barrier.md`.
