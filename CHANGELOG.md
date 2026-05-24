# Logicodex Changelog

All notable changes to the Logicodex compiler are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/), and this project adheres to [Semantic Versioning](https://semver.org/) for release versions.

---

## [Merged via PR #12] — 2026-05-24 — Version Gate Integration (v1.30 Pipeline)

### Added — Edition Routing / Version Gate Architecture
- **New CLI flag**: `--pipeline <v1.21|v1.30>` on both `compile` and `check` commands.
  - Default: `v1.21` (stable, backward-compatible).
  - Opt-in: `v1.30` activates experimental parsing for advanced constructs.
- **New `CompilerPipeline` enum** in `src/parser.rs` with `FromStr` implementation for clean CLI parsing.
- **Parser pipeline gating**: `Parser::with_pipeline()` allows per-instance pipeline selection.
  - `v1.21` pipeline: tokens like `struct`, `enum`, `unsafe`, `extern` are trapped with `UnimplementedFeature` error.
  - `v1.30` pipeline: these tokens are parsed into proper AST nodes (`StructDecl`, `EnumDecl`, `UnsafeBlock`, `ExternBlock`).
- **New AST variants** in `src/ast.rs`:
  - `Stmt::StructDecl { name, fields }` — structure type declarations.
  - `Stmt::EnumDecl { name, variants }` — enumeration type declarations.
  - `Stmt::UnsafeBlock { body }` — unsafe code blocks.
  - `Stmt::ExternBlock { abi, functions }` — foreign function interface blocks.
  - `ExternFnDecl` struct for individual extern function signatures.
- **HIR enhancements** in `src/hir.rs`:
  - Added `StmtAst::If` and `HirStmt::If` with condition, then-branch, and optional else-branch.
  - Added `LoweringContext::types` field carrying `TypeRegistry` reference.
  - Replaced hardcoded `TypeId` values with `TypeRegistry::primitive()` lookups.
  - Fixed `AddressOf` to use proper pointer type interning via `TypeKind::Pointer`.
  - Fixed `ExternBlock` lowering bug — now correctly processes all extern functions instead of only the last one.
- **Codegen safety net** in `src/codegen.rs`:
  - `LlvmCompiler::emit_v130_ast_in_v21()` emits `unreachable!()` panic with informative message if v1.30 AST nodes leak into v1.21 codegen.
  - `compile_v130()` entry point for v1.30 HIR-to-object compilation.
  - `CodegenBackend` trait defining the contract for version-gated codegen backends.
- **Semantic gate update** in `src/semantic_gate.rs`:
  - `check_statement()` now handles `HirStmt::If` with proper scope management for both branches.

### Changed
- **Parser `declaration_or_statement()`**: Replaced monolithic v1.21 trap with pipeline-dispatched `match` arms. Cleaner, faster, and enables LLVM jump-table optimization.
- **`named_type_id()`**: Now takes `&TypeRegistry` parameter instead of returning hardcoded `TypeId` values. Eliminates fragile numeric constants.
- **`LoweringContext` construction**: Now requires both `symbols: &mut SymbolTable` and `types: &mut TypeRegistry`.

### Fixed
- **ExternBlock lowering bug** (`src/hir.rs`): Previously only the last extern function in a block was preserved. Now all functions are correctly lowered.
- **AddressOf type bug** (`src/hir.rs`): Previously hardcoded `TypeId(15)` for all pointer types. Now each pointer gets a unique `TypeId` via proper type interning.

### Security / Defense-in-Depth
- **Fail-fast codegen**: v1.21 codegen will panic with a descriptive message (via `unreachable!()`) if it receives v1.30-only AST nodes. This prevents silent corruption and makes pipeline misconfigurations immediately visible.

### Zero Regression Guarantee
- **Default pipeline**: `v1.21` (backward-compatible, no behavior change).
- v1.21 code paths are **untouched**.
- v1.21 does **not** pass through HIR lowering.
- Fail-fast `unreachable!()` safety nets prevent silent pipeline leaks.

### Validation
- All 9 `validate_v121_executable_logic.py` checks pass:
  - AST supports executable v1.21-alpha declarations ✅
  - Lexer exposes canonical v1.21-alpha tokens ✅
  - Parser enforces executable grammar layout ✅
  - Semantic analyzer implements static safety checks ✅
  - Code generator accepts expanded AST ✅
  - CLI wires target and secure flags ✅
  - Dictionary token surface ✅
  - Version-label policy ✅
  - Known regression guards ✅

---

## [1.21.0-alpha] — 2026-05-XX

### Added
- Initial v1.21-alpha compiler core with LLVM backend.
- Malay/English bilingual alias system via `dict/core_map.json`.
- Hardware-zone provenance gates (`ZON_PERKAKASAN` / `hw_unsafe`).
- Reflex-engine example suite covering arithmetic, functions, loops, bitwise operations, hardware-zone provenance, and Boolean conditionals.
- Three-tier error severity classification (Critical / Medium / Low).
- Dormant v1.30.0-alpha subsystem with HIR, layout engine, semantic gate, and codegen contracts.

---

*For older releases, see the Git history.*
