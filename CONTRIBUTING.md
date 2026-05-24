# Contributing to Logicodex

Thank you for contributing to **Logicodex**, the current logicodex v 1.21 alpha compiler for the Logicodex programming language. This repository is currently under a feature-freeze discipline for compiler-integrity restoration. Contributions should therefore prioritize correctness, repeatable validation, diagnostic clarity, and physical examples before proposing new language features.

## Development Environment

Logicodex is implemented in Rust and uses LLVM through the `inkwell` crate. The repository metadata pins the public CI channel to **Rust 1.75.0** and the LLVM feature family to **LLVM 15**. Local development should match that environment as closely as practical.

```bash
rustup toolchain install 1.75.0
rustup component add rustfmt --toolchain 1.75.0
sudo apt-get update
sudo apt-get install -y llvm-15 llvm-15-dev clang-15 lld-15
cargo +1.75.0 check
cargo +1.75.0 test
```

If your system uses a non-standard LLVM installation path, set `LLVM_SYS_150_PREFIX` before running Cargo.

```bash
export LLVM_SYS_150_PREFIX=/usr/lib/llvm-15
cargo check
cargo test
```

## Repository Integrity Rules

Compiler changes must preserve the active v1.21 architecture unless a maintainer explicitly approves a staged migration. Do not replace working core files with broad rewrites when a smaller, auditable change is sufficient. In particular, `src/main.rs` must remain the CLI driver that connects the lexer, parser, semantic analyzer, and LLVM code generator, while `src/ast.rs` must remain the shared AST contract consumed by parser, semantic analysis, and backend code generation.

| Area | Requirement | Validation |
|---|---|---|
| Formatting | Rust code must be rustfmt-compliant. | `cargo fmt --all -- --check` |
| Compiler health | The crate must type-check with the pinned dependencies. | `cargo check --locked` |
| Regression coverage | Unit and integration tests must pass. | `cargo test --locked` |
| Examples | New examples must use syntax accepted by the current parser. | `cargo run -- check examples/name.ldx` |
| Safety-sensitive syntax | Hardware and raw-address examples must remain explicitly gated. | Prefer `--target freestanding --object-only` validation |

## Adding or Updating Dictionary Tokens

The canonical token vocabulary lives in `dict/core_map.json`. Each token entry should preserve the three-surface model used by the current compiler: a stable token identifier, one expert canonical spelling, and a primary Malay alias where applicable. English pseudocode aliases may be added only when they do not conflict with existing lexemes.

When adding a token, update the dictionary, lexer normalization, parser handling, grammar documentation, and tests together. A dictionary-only addition is not sufficient if the lexer or parser cannot consume the token. Keep token names stable and avoid reusing an existing lexeme for a different semantic meaning.

## Bilingual Diagnostic Policy

Logicodex diagnostics must be clear to both Malay-first and English-first users. New compiler errors should include a Malay message and an English message when they are surfaced as user-facing diagnostics. The expected convention is:

> `Mesej Melayu / English message`

For example, a semantic error should explain both the local-language problem statement and its English equivalent. Documentation prose may remain English-only, but compiler diagnostics and contributor-facing diagnostic examples must preserve this bilingual standard.

## Pull Request Checklist

Before requesting review, run the full local validation set and include the command output summary in the pull request description.

```bash
cargo fmt --all -- --check
cargo check --locked
cargo test --locked
python3.11 scripts/check_bilingual_error_annotations.py
python3.11 scripts/validate_v121_executable_logic.py
```

A pull request should explain the integrity problem being fixed, list the files changed, describe any new examples or tests, and state whether any language behavior changed. Feature additions should be separated from integrity hotfixes so reviewers can verify repository restoration independently.
