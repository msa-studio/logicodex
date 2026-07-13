# P1-B5a std.file / std.io Boundary Record

Status: Active CPB blocker record

## Decision

Do not introduce public `std.file` or `std.io` CPB APIs yet.

`PAPAR` currently works as compiler/runtime builtin output. That is useful for
tests and examples, but it is not a callable `std.io` module API.

## Current facts

The current proven surface is:

- `PAPAR <expr>;` compiles and prints integer-like values through the runtime
  print builtin.
- String parameters can compile in ordinary functions.
- `std.path` exists only as a pure lexical path foundation.

The current blocked surface is:

- `import std.file;`
- `import std.io;`
- `std.io.print_i64(...)`
- file open/read/write/close APIs
- stdin/stdout/stderr abstractions as public stdlib APIs
- `FileHandle` as a CPB-trusted value type
- `Result<T, IoError>` or richer IO error modelling

## Why this is blocked

File and IO APIs cross multiple architecture boundaries:

1. capability gates (`Storage.*`, `UI.Papar`, future IO gates);
2. runtime profile policy (`bare`, `std`, `safe`, `actor`, `service`);
3. hosted vs freestanding target behaviour;
4. filesystem authority and sandboxing;
5. error model and handle lifetime policy;
6. FFI/syscall/host ABI boundary.

A fake `.ldx` wrapper without these decisions would create a misleading CPB API.

## Required before implementation

Before `std.file` or callable `std.io` can be contract-backed, Logicodex needs:

- explicit IO capability policy;
- profile compatibility rules;
- handle representation and lifetime rules;
- error/status representation;
- target behaviour for hosted, freestanding, and future WASI/profile builds;
- tests that distinguish builtin `PAPAR` from public `std.io`;
- contract metadata that can honestly represent IO capabilities.

## Regression guard

The current boundary is guarded by `tests/p1b5_file_io_boundary.rs`.

When real file/IO support is implemented, replace the negative boundary tests
with positive contract-backed tests and only then add:

- `lib/std/file.ldx`
- `lib/std/file.std.toml`
- `lib/std/io.ldx`
- `lib/std/io.std.toml`
