# Logicodex v1.21-alpha Executable Implementation Design

This design keeps the repository on **version 1.21-alpha** and implements executable compiler-core support for Issue #01 and Issue #02 rather than introducing a new version label.

| Area | Implementation Decision |
|---|---|
| Lexer invariants | Extend `TokenKind` and `dict/core_map.json` so the **expert canonical shorthand** surface and the Malay/English pseudocode alias surfaces normalize into the parser's canonical token kinds for `hw`, `addr`, `use`, `fn`, `return`, `:`, `,`, `->`, `PTR`, integer-width types, and braces/then aliases. |
| Parser grammar | Preserve existing Malay/English pseudocode alias programs and expert canonical Phase 1 programs while adding AST nodes for `HardwareDecl`, `UseDecl`, `FunctionDef`, `Return`, explicit type annotations, and `AddressOfLiteral`. The recursive-descent parser will keep precedence-separated expression parsing and add typed declarations without parser-side lexeme rewriting. |
| Semantic gatekeeper | Introduce `ProvenanceClass`, `SemanticOptions`, and typed symbol metadata. Hardware declarations require `PTR<T>` and classify `addr <literal>` as `Hardware` in freestanding mode or `Wild` in hosted mode. Static division by zero remains rejected. Reuse/double definition of scoped resources continues to be blocked through duplicate binding checks, and object-scope boundaries are enforced through scoped symbol tables. |
| CLI metadata | Keep `1.21-alpha` version strings. Parse target before semantic analysis and pass `SemanticOptions { target, secure }` into the analyzer, then pass the same target/secure metadata into codegen. |
| Code generation severity | Prototype Tier 2 dynamic division-by-zero guards around non-constant divisors. Hosted codegen declares and calls `exit(1)`; freestanding codegen emits a fail-stop loop through `llvm.trap` plus an infinite block. Tier 1 helper paths are used for critical fail-stop and freestanding unsafe hardware access. Tier 3 warnings should remain diagnostic-first unless a measured runtime check is explicitly implemented. |
| Validation | Install/use Rust tooling if available, run `cargo fmt`, `cargo check`, and `cargo build`. If system LLVM constraints prevent full linking, record the exact failure and still run structural validation. |
| Packaging | Regenerate `logicodex-v1.21-alpha.zip`, `logicodex-v1.21-alpha.tar.gz`, and SHA256 files only; do not bump version. |
