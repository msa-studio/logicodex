# P1-B4a std.path Lexical Foundation

Status: Active CPB foundation slice

## Decision

`std.path` starts as a pure lexical foundation module.

P1-B4a intentionally provides only String-to-I64 helpers that are already safe
under the current compiler and contract harness.

## Implemented API

The implemented CPB slice is:

- `is_empty_path_i64(path: String) -> I64`
- `not_empty_path_i64(path: String) -> I64`
- `same_emptiness_path_i64(left: String, right: String) -> I64`
- `select_by_empty_path_i64(path: String, when_empty: I64, when_nonempty: I64) -> I64`

These functions are wrappers around the proven `core.text` emptiness foundation.

## Non-claims

P1-B4a does not claim:

- filesystem access
- path existence checks
- current working directory access
- path normalization
- separator inspection
- basename / dirname / extension extraction
- absolute / relative path semantics
- platform-specific path rules
- security boundary checks for file IO

Those belong to later `std.path` phases after stronger string operations and
file/IO policy are available.

## Contract policy

`std.path` is contract-backed through:

- `lib/std/path.ldx`
- `lib/std/path.std.toml`
- `tests/stdlib_std_path.rs`
- `tools/verify_stdlib_contracts.py`

The module is pure and requires no capabilities.
