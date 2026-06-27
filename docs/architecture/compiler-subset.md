# Logicodex Compiler Subset

Status: Active CPB-1 planning document
Scope: Community self-hosting runway
Depends on: CPB Self-Hosting Runway

## Purpose

This document defines the minimum Logicodex language subset required to write
compiler-shaped programs on the path to self-hosting.

The compiler subset is not the full language. It is the stable subset that must
be reliable enough for compiler components.

## Goal

Enable small compiler components to be written in Logicodex before attempting a
full self-hosted compiler.

Recommended first components:

- token classifier
- module path normalizer
- diagnostic formatter
- simple contract case runner
- simple AST or HIR visitor prototype

## Non-Goal

This document does not start full self-hosting.

It also does not require:

- package manager
- network runtime
- actor runtime
- enterprise assurance
- AI governance
- dynamic plugins
- full standard library

## Required Language Surface

The compiler subset requires these language features:

- module imports
- function declarations
- explicit parameter and return types
- local variable bindings
- integer types
- boolean-like integer predicates
- string/text literals
- if/else
- loops
- return statements
- qualified module calls
- structs
- enums
- simple pattern matching or equivalent branching
- deterministic diagnostics output

## Required Type Surface

Minimum required types:

- `I64`
- `U64`
- `Bool` or boolean-compatible predicate result
- `Text` or string pointer-compatible representation
- fixed-size arrays
- simple structs
- simple enums
- result-like success/failure representation

## Phase 1 Library Blocker Priority

Phase 1 must remove foundational blockers before package manager or full
development tooling work begins.

The priority is:

1. `core.text` or `core.string`
2. `core.option`
3. `core.result`
4. `core.array` or `core.slice`
5. `std.path`
6. `std.file`
7. `std.io`
8. diagnostic formatter subset
9. package metadata reader
10. package manager MVP
11. real development tools

### P1-B1: Text/String

Text and string helpers are the first blocker because compiler-like programs
need stable text processing for tokens, paths, diagnostics, contract metadata,
and source messages.

Minimum trusted surface:

- length
- empty check
- equality
- prefix/suffix check
- simple byte or character access if supported by current language semantics

### P1-B2: Option/Result/Error

Option and result are required before file/io and package tooling.

They are needed for:

- fallible parsing
- missing file handling
- diagnostic construction
- package metadata validation
- contract verification result flow

Legacy `core.result` is not a CPB proof surface. It must be rebuilt or migrated
under contract when this blocker is implemented.

### P1-B3: Array/Slice/List

Compiler-like programs need collection primitives for:

- token lists
- diagnostic lists
- import lists
- contract case lists
- path segment lists

Start with the simplest representation that current Logicodex can compile
reliably. Avoid advanced generics until the compiler subset supports them.

### P1-B4: Path

Path helpers must come before file/io and package manager work.

Minimum trusted surface:

- normalize module path
- join path segments
- split module path
- detect empty or invalid segment
- convert dotted module path to filesystem path

### P1-B5: File/IO

File and IO are required for package manager and real development tools, but
they should not be first.

They depend on:

- text/string
- result/error
- path
- diagnostics

Minimum trusted surface:

- read text file
- write text file
- exists check
- deterministic error reporting

### P1-B6: Diagnostic Formatter Subset

Diagnostics must become stable enough for compiler-shaped programs.

Minimum trusted surface:

- stable code
- Malay message
- English message
- position rendering
- deterministic ordering

### P1-B7: Package Metadata Reader

Before a package manager, Logicodex needs a small metadata reader.

Minimum trusted surface:

- package name
- version
- source files
- dependency names
- contract paths
- profile flags

### P1-B8: Package Manager MVP

Only after the above blockers are stable should `ldx-pkg` begin.

First package manager MVP:

- init
- build
- test
- local dependency resolution
- lockfile skeleton

No remote registry is required for the first MVP.

### P1-B9: Real Development Tools

Real development tools come after package metadata and file/io are stable.

Priority:

- formatter
- test runner
- doc generator
- LSP diagnostics
- project scaffolder

## Package Manager Hold Rule

Package manager work must not begin until these foundations are contract-backed
or explicitly accepted as experimental:

- text/string
- option/result
- path
- file/io
- diagnostics
- module/import regression tests

## Real Development Tools Gate

Formatter, LSP, test runner, and project scaffolding must not be treated as CPB
proof until the package metadata reader and file/io foundation are stable.

## Required Stdlib Surface

The minimum stdlib surface for compiler-like programs is:

1. `core.prelude`
2. `core.text` or `core.string`
3. `core.option`
4. `core.result`
5. `core.array` or `core.slice`
6. `std.path`
7. `std.file`
8. `std.io`

Each trusted module must follow the contract pattern:

- `.ldx` source
- `.std.toml` sidecar
- declared public exports
- non-empty run-cases
- focused e2e tests
- verify script coverage
- trust-state update

## Bootstrap Rule

The compiler subset must prefer simple, explicit features over advanced
language features.

Allowed early:

- simple functions
- simple structs
- simple enums
- fixed integer operations
- deterministic string/path helpers
- explicit error values

Deferred:

- generics
- traits
- macros
- reflection
- dynamic dispatch
- async runtime
- actor runtime
- package registry
- unsafe FFI
- theorem proving
- enterprise policy engine

## Runtime Rule

Compiler subset programs should initially run under the Rust-hosted Logicodex
compiler/runtime.

The first self-hosting loop compiles small Logicodex compiler components using
the existing Rust compiler host. It does not replace the host immediately.

## Diagnostics Rule

Compiler subset programs must produce deterministic diagnostics.

Minimum diagnostic requirements:

- stable error code
- Malay message
- English message
- source position when available
- deterministic ordering
- no hidden runtime-only failure for expected compiler errors

## Module Rule

Compiler subset programs must use the normal module/import path.

No direct filesystem hacks, generated imports, or ad hoc path bypasses may be
used to prove compiler subset readiness.

## Contract Rule

Any stdlib function used by compiler subset examples must be either:

- `ContractVerified`, or
- explicitly marked experimental in the example and excluded from CPB proof

Legacy modules cannot be used as CPB proof unless migrated under contract.

## First Proof Programs

The first proof programs should be small:

1. token classifier
2. module path normalizer
3. diagnostic formatter

A full parser or compiler rewrite is explicitly deferred until these pass.

## Exit Criteria

CPB-1 compiler subset is ready when:

- this document is committed
- CPB runway verify passes
- required subset terms are gated
- first three proof programs are defined
- no legacy unverified module is required for the proof plan
