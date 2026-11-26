# Contributing to Logicodex

Thank you for your interest in **Logicodex** (engine `v0.46.0-alpha`), a research-grade compiler for the Logicodex programming language. The project is maintained by a single developer (Mohamad Supardi Abdul) and is under a feature-freeze discipline focused on compiler-integrity restoration. Contributions should prioritize correctness, repeatable validation (`cargo test` must stay green), diagnostic clarity, and verified examples (every `examples/*.ldx` must pass `check`) over new language features. To propose becoming a co-maintainer, open an issue describing sustained prior contributions.

## Development Environment

Logicodex is implemented in Rust and uses LLVM through the `inkwell` crate. `rust-toolchain.toml` pins **Rust 1.100.0**, and `Cargo.toml` selects the inkwell `llvm23-1` feature, so CI and releases build against **LLVM 23.1.6** on Ubuntu 24.04 (apt.llvm.org). Local development should match that environment as closely as practical.

```bash
rustup toolchain install 1.100.0 --component rustfmt,clippy
sudo install -d -m 0755 /etc/apt/keyrings
curl -fsSL https://apt.llvm.org/llvm-snapshot.gpg.key \
  | sudo gpg --dearmor -o /etc/apt/keyrings/llvm-snapshot.gpg
echo 'deb [signed-by=/etc/apt/keyrings/llvm-snapshot.gpg] https://apt.llvm.org/noble/ llvm-toolchain-noble-23 main' \
  | sudo tee /etc/apt/sources.list.d/llvm23.list
sudo apt-get update
sudo apt-get install -y llvm-23-dev libclang-23-dev libpolly-23-dev libzstd-dev pkg-config
cargo check --locked
cargo test --locked
```

`llvm-sys` locates LLVM through `LLVM_SYS_231_PREFIX`. Set it, together with the linker search path, before running Cargo.

```bash
export LLVM_SYS_231_PREFIX=/usr/lib/llvm-23
export RUSTFLAGS="-L/usr/lib/llvm-23/lib"
cargo check
cargo test
```

On macOS, `brew install llvm` provides the LLVM 23 major line; set `LLVM_SYS_231_PREFIX="$(brew --prefix llvm)"`. Homebrew does not guarantee the exact 23.1.6 patch release used on Linux.

## Repository Integrity Rules

Compiler changes must preserve the active architecture unless a maintainer explicitly approves a staged migration. The execution path is **HIR-based**: source → lexer → parser → AST → HIR lowering → semantic gate → LLVM codegen (the legacy v1.21 AST-codegen path was retired; see `docs/architecture/hir-decision.md`). Do not replace working core files with broad rewrites when a smaller, auditable change is sufficient. In particular, `src/main.rs` must remain the CLI driver, `src/ast.rs` the shared AST contract, and `src/hir.rs` the sole lowering path into codegen.

| Area | Requirement | Validation |
|---|---|---|
| Formatting | Rust code must be rustfmt-compliant. | `cargo fmt --all -- --check` |
| Compiler health | The crate must type-check with the pinned dependencies. | `cargo check` |
| Regression coverage | Unit and integration tests must pass. | `cargo test` |
| Examples | New examples must use syntax accepted by the current parser. | `cargo run -- check examples/name.ldx` |
| Example phase-gate | Every shipped example must pass `check` (enforced by the `shipped_examples_pass_semantic_check` test). | `for f in examples/*.ldx; do cargo run -- check "$f"; done` |
| Safety-sensitive syntax | Hardware and raw-address examples must remain explicitly gated. | Prefer `--target freestanding --object-only` validation when exercising backend object generation. |

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
cargo check
cargo test
for file in examples/*.ldx; do
  cargo run -- check "$file"
done
```

A pull request should explain the integrity problem being fixed, list the files changed, describe any new examples or tests, and state whether any language behavior changed. If the change touches parser, semantic analysis, CLI behavior, or documentation examples, include the example phase-gate result (`check` on every `examples/*.ldx`) in the PR body. Feature additions should be separated from integrity hotfixes so reviewers can verify repository restoration independently.
