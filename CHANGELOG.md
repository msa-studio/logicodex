# Logicodex Changelog

All notable changes to the Logicodex compiler are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/), and this project adheres to [Semantic Versioning](https://semver.org/) for release versions.

---

## [Merged via PR #24] — 2026-05-24 — Fix: 5 Critical Bugs in Buffer Overflow & Use-After-Move

### Fixed
- **BUG #1 CRITICAL**: `Stmt::Let` tak register buffer ke `buffer_registry` → `register_buffer()` call semasa Let process `Buffer<T>`
- **BUG #2 CRITICAL**: Parser tak support `buf[index] = value` → `peek_index_assignment()` + `index_assignment_statement()`
- **BUG #2b CRITICAL**: `Stmt::Assign` tak handled dalam semantic analyzer → Full Assign handling dengan Index target validation + provenance check
- **BUG #3 HIGH**: `moved_vars` tak clear bila scope keluar → `scoped_block()` cleanup `moved_vars` + `buffer_registry`
- **BUG #4 MEDIUM**: `mark_moved` tak pernah dipanggil → Let detect ownership transfer (`let buf2 = buf`)
- **BUG #5 LOW**: Error misleading untuk unregistered buffer → `NotABuffer` error variant (Malay + English)

### Added
- `Buffer<f32, 1024>` capacity syntax dalam parser
- `tests/buffer_provenance_bugfixes.rs` — 9 assertions
- `scripts/validate_buffer_bugfixes.py` — 9 checks

### Validation
- v1.21: 9/9 | Sprint 1.1: 32/32 | Sprint 1.2: 20/20 | Sprint 2: 34/34 | Sprint 2.5: 25/25 | Sprint 3: 28/28 | Demo: 11/11 | K1: 17/17 | **Bug Fixes: 9/9 ✅**

## [Merged via PR #23] — 2026-05-24 — Ketuk 1: Core Memory Model (Slice, Buffer, Ownership & Provenance)

### Added
- **AST** (`src/ast.rs`): `Type::Slice { element }`, `Type::Buffer { element }`, `Expr::Index { base, index }`
  - `is_slice()`, `is_buffer()`, `is_contiguous()`, `element_type()` helpers
- **Lexer** (`src/lexer.rs`): `LeftBracket`, `RightBracket`, `Buffer` tokens
- **Parser** (`src/parser.rs`): `[]T` slice type, `Buffer<T>` buffer type, `buf[index]` indexing
- **Semantic** (`src/semantic.rs`): Buffer provenance + ownership tracking
  - `BufferOverflow { name, index, capacity }` — compile-time bounds check
  - `UseAfterMove { name }` — ownership violation detection
  - `ElementTypeMismatch { elem, expected, actual }`
  - `validate_buffer_index()`, `register_buffer()`, `mark_moved()`, `is_moved()`
- **Native Library** (`lib/core/memori.ldx`): `panjang`, `kapasiti`, `kosongkan`, `salin`, `isi`, `sub`
- **Tests**: `tests/core_memory_model.rs` (17 assertions)
- **Validator**: `scripts/validate_core_memory.py` (7 checks)

### Validation
- v1.21: 9/9 | Sprint 1.1: 32/32 | Sprint 1.2: 20/20 | Sprint 2: 34/34 | Sprint 2.5: 25/25 | Sprint 3: 28/28 | Demo: 11/11 | **K1 Core Memory: 17/17 ✅**

## [Merged via PR #22] — 2026-05-24 — Audio Engine: Hardware-Safe Audio Guards with Function Pointers

### Added
- **AST** (`src/ast.rs`): `Type::FunctionPointer { params, return_type }` — function pointer type
  - `is_function_pointer()` — check if type is a function pointer
  - `is_audio_callback_fp()` — detect audio ISR signature `fn(*mut f32, i32)`
- **Parser** (`src/parser.rs`): `parse_type()` handles `fn(params) -> ret` syntax
- **StrictAudioContext** (`src/semantic.rs`): Hardware-safe audio callback verification
  - `verify_audio_safety()` — walks function body, validates all statements/expressions
  - `AudioViolationIo` — rejects `Print`, `DrawText`, `InitWindow` in callbacks
  - `AudioViolationRecursion` — rejects self-calling in audio ISR
  - `AudioViolationUnboundedLoop` — rejects `loop { }` (watchdog risk)
  - `AudioViolationForbiddenCall` — rejects unsafe function calls
  - `mark_audio_callback_if_applicable()` — detects `SetAudioStreamCallback(func)`
- **Native Library** (`lib/std/audio.ldx`): `tulis_selamat()` hardware clipper clamping `[-1.0, 1.0]`, `kepit()`, `gelombang_sinus()`
- **Demo** (`examples/audio_sine.ldx`): 72-line function pointer callback demo
- **Tests**: `tests/audio_engine_hardware_safe.rs` (14 assertions)
- **Validator**: `scripts/validate_audio_engine.py` (8 checks)

### Validation
- v1.21: 9/9 | Sprint 1.1: 32/32 | Sprint 1.2: 20/20 | Sprint 2: 34/34 | Sprint 2.5: 25/25 | Sprint 3: 28/28 | Demo: 11/11 | **Audio: 14/14 ✅**

## [Merged via PR #21] — 2026-05-24 — Demo: Raylib Spinning Box (compile-ready example)

### Added
- **`examples/raylib_spinning_box.ldx`** — 53-line interactive demo program:
  - 6x `Color(r, g, b, a)` struct constructors (packed RGBA)
  - Raylib FFI calls: `InitWindow`, `DrawRectangle`, `DrawText`, `ClearBackground`, `BeginDrawing`, `EndDrawing`
  - Input handling: `IsMouseButtonPressed(0)`, `IsKeyPressed(KEY_SPACE)`
  - Game loop: `while (!WindowShouldClose())` with `break`
  - `unsafe { ... }` FFI safety gate
- **Integration test** (`tests/demo_raylib_spinning_box.rs`): 11 assertions:
  - Parser: all `Color(...)` recognized as `Expr::Call` with 4 args
  - TypeChecker: validates all 6 color constructors
  - CallableRegistry: all 12 Raylib functions used are registered with correct signatures
  - HIR lowering: demo program lowers to `HirModule` without errors
  - Color packing: `Color(255,0,0,255)` → `0xFF0000FF`
- **Validator** (`scripts/validate_demo_raylib_box.py`): 4 checks PASSED

### Compile
```bash
logicodex --pipeline v1.30 examples/raylib_spinning_box.ldx -o spinning_box
```

### Validation
- v1.21 executable logic: 9/9 checks PASSED
- Sprint 1.1: 32/32 checks PASSED
- Sprint 1.2: 20/20 checks PASSED
- Sprint 2: 34/34 checks PASSED
- Sprint 2.5: 25/25 checks PASSED
- Sprint 3: 28/28 checks PASSED
- **Demo Spinning Box: 11/11 assertions PASSED**

## [Merged via PR #20] — 2026-05-24 — Sprint 3: Codegen Backend for Function Calls

### Added
- **LlvmCompiler CallableRegistry integration** (`src/codegen.rs`): `with_callables()` attaches `CallableRegistry` + `TypeRegistry` for function call codegen
- **TypeId → LLVM mapping** (`src/codegen.rs`): `type_id_to_llvm()` maps all `PrimitiveType` variants → `inkwell::BasicTypeEnum`
- **LLVM extern function declaration** (`src/codegen.rs`): `declare_extern_func()` creates LLVM function declarations with `Linkage::External` and caching
- **Function call codegen** (`src/codegen.rs`): `emit_expr(Expr::Call)` — CallableRegistry lookup → declare → `builder.build_call()` → extract return value
- **Struct constructor codegen** (`src/codegen.rs`): `try_struct_constructor()` — `Color(255,0,0,255)` → packed u32 `0xFF0000FF`
- **v1.21 → HIR lowering** (`src/hir.rs`): `lower_v121_program()` converts `ast::Program` → `HirModule` with callable registration
- **AST conversion helpers** (`src/hir.rs`): `lower_type_ast`, `lower_stmt_ast`, `lower_expr_ast`, `lower_binary_op` — v1.21 AST → HIR AST bridge
- **V130 compile pipeline** (`src/main.rs`): `compile_v130_pipeline()` — parse → Raylib type/function registration → HIR lowering → semantic check → `compile_v130()`
- **Tests** (`tests/codegen_function_calls.rs`): 28 assertions — CallableRegistry, type mapping, Raylib registration, HIR lowering, Color packing
- **Validator** (`scripts/validate_sprint3_codegen_calls.py`): 28/28 checks PASSED

### Changed
- `compile_v130()`: Updated signature to accept `(CallableRegistry, TypeRegistry)`
- `compile()`: Branches on `CompilerPipeline::V130` → `compile_v130_pipeline()` vs `V121` → `compile_to_object()`

### Validation
- v1.21 executable logic: 9/9 checks PASSED
- Sprint 1.1: 32/32 checks PASSED
- Sprint 1.2: 20/20 checks PASSED
- Sprint 2: 34/34 checks PASSED
- Sprint 2.5: 25/25 checks PASSED
- Sprint 3: 28/28 checks PASSED

## [Merged via PR #19] — 2026-05-24 — Sprint 2.5: Struct Literals & Function Call Parser

### Added
- **Expr::Call** (`src/ast.rs`): New AST variant `Call { callee: Box<Expr>, args: Vec<Expr> }` for struct constructors and function calls
- **Parser call detection** (`src/parser.rs`): `primary()` detects `Identifier(` → parses as `Expr::Call` with comma-separated argument list
- **HIR Call lowering** (`src/semantic.rs`): `ExprAst::Call` → `HirExprKind::Call` with Sprint 3 codegen placeholder
- **TypeChecker::check_call()** (`src/semantic/type_checker.rs`): Validates struct constructor argument count against registered `StructLayout` fields
- **Tests** (`tests/parser_struct_literals.rs`): 25 assertions — struct literals `Color(255,0,0,255)`, nested constructors, function calls `print("hello")`, error cases
- **Validator** (`scripts/validate_sprint2_5_struct_literals.py`): 25/25 checks PASSED

### Architecture Notes
- `check_call()` returns `Type::I64` placeholder — full struct TypeId resolution deferred to Sprint 3 (LLVM struct value emission)
- Complex callees (e.g., `obj.method()`) return descriptive error — deferred to Sprint 3

### Validation
- v1.21 executable logic: 9/9 checks PASSED
- Sprint 1.1: 32/32 checks PASSED
- Sprint 1.2: 20/20 checks PASSED
- Sprint 2: 34/34 checks PASSED
- Sprint 2.5: 25/25 checks PASSED

## [Merged via PR #18] — 2026-05-24 — Sprint 2: LayoutEngine

### Added
- **Struct layout types** (`src/types.rs`): `StructLayout`, `StructFieldLayout` moved from `layout.rs`
- **TypeRegistry struct cache**: `struct_layouts: Vec<StructLayout>`, `intern_struct()`, `get_struct_layout()`, `find_struct_by_name()`
- **get_size/get_align for Struct**: Uses cached layout instead of panic
- **LayoutEngine struct lookup** (`src/layout.rs`): `size_and_align` resolves Struct via cache
- **Raylib struct types** (`src/ffi/raylib.rs`): `register_raylib_types()` registers Color(4B), Vector2(8B), Rectangle(16B), Texture2D(20B)
- **Tests** (`tests/layout_engine_integration.rs`): 29 assertions — layout, cache, Raylib types, nested structs

### Changed
- `src/layout.rs`: Import `StructLayout`/`StructFieldLayout` from `types.rs` (not local)

### Validation
- v1.21 executable logic: 9/9 checks PASSED
- Sprint 1.1: 32/32 checks PASSED
- Sprint 1.2: 20/20 checks PASSED
- Sprint 2: 34/34 checks PASSED

## [Merged via PR #17] — 2026-05-24 — Stage 1 Quickfix: Raylib Color + Math

### Fixed
- **Color registration mismatch**: 7 drawing functions registered Color as `I64`, now `U32` (packed RGBA `0xRRGGBBAA`)
- ClearBackground, DrawText, DrawRectangle, DrawCircle, DrawLine, DrawRectangleLines, DrawPixel

### Added
- **Math utilities** (`src/ffi/math.rs`): `clamp()`, `lerp()`, `remap()`, `normalize()`, `float_equals()`, `float_zero()`
- **Integration test** (`tests/ffi_stage1_integration.rs`): 25 assertions validating full user story

### Validation
- v1.21 executable logic: 9/9 checks PASSED
- Sprint 1.1 structural: 32/32 checks PASSED
- Sprint 1.2 structural: 20/20 checks PASSED

## [Merged via PR #16] — 2026-05-24 — Sprint 1 Bugfixes (8 critical fixes)

### Fixed
- **CRITICAL #1**: `ast_type_to_id` returned invalid `TypeId(1000+)` — would panic on `resolve()`
- **CRITICAL #2**: Circular dependency `types.rs ↔ semantic/coercion.rs` broke compilation
- **CRITICAL #3**: `use` statement inside method body — invalid Rust syntax
- **HIGH #4**: `coercion.rs` test `setup()` — self-referential lifetime issue
- **HIGH #5**: `type_checker.rs` test — same self-referential pattern
- **MEDIUM #6**: `infer_default_type` returned `I64` for all complex expressions → now `Option<Type>`
- **MEDIUM #7**: `c_void_ptr`/`c_const_char_ptr` needed `&mut self` — inconsistent API → added `void_ptr()`/`const_char_ptr()` with `&self`
- **LOW #8**: `explain_incompatibility` only covered Bool/String → expanded to all cases

### Changed
- `src/types.rs`: Removed AST bridge (→ TypeChecker), added `&self` pointer accessors
- `src/semantic/type_checker.rs`: Added bridge functions, fixed lifetimes, `Option<Type>` inference
- `src/semantic/coercion.rs`: Fixed test helper lifetime

### Validation
- v1.21 executable logic: 9/9 checks PASSED
- Sprint 1.1 structural: 32/32 checks PASSED
- Sprint 1.2 structural: 20/20 checks PASSED

## [Merged via PR #15] — 2026-05-24 — Sprint 1.2: Parser Type Injection

### Added
- **TypeChecker** (`src/semantic/type_checker.rs`):
  - `check_assignment(declared, actual) -> TypeCheckResult` — uses CoercionEngine
  - `TypeCheckResult` enum: `Ok`, `ImplicitWidening`, `RequiresExplicitCast`, `Incompatible`
  - `infer_default_type(Expr) -> Type` — I64 (int), F64 (float), String, Bool
  - `format_error()` — bilingual Malay/English diagnostics with cast suggestions
- **AST Type Bridge** (`src/types.rs`):
  - `ast_type_to_id()` — converts `ast::Type` to `TypeId`
  - `type_id_to_ast()` — converts `TypeId` back to `ast::Type`
  - `ast_types_compatible()` — CoercionEngine-based compatibility check
- **Tests** (`tests/parser_type_test.rs`): 25 assertions

### Validation
- v1.21 executable logic: 9/9 checks PASSED
- Sprint 1.1 structural: 32/32 checks PASSED
- Sprint 1.2 structural: 20/20 checks PASSED

## [Merged via PR #14] — 2026-05-24 — Sprint 1.1: Type System Foundation

### Added
- **TypeRegistry enhancements** (`src/types.rs`):
  - `get_size(TypeId) -> usize` — deterministic byte sizes (I32=4, I64=8, Ptr=8)
  - `get_align(TypeId) -> usize` — C ABI alignment
  - `resolve(TypeId) -> &TypeKind` — infallible lookup
  - `c_abi_info(TypeId) -> CAbiInfo` — combined size+align for FFI
  - FFI type aliases: `c_int()`, `c_double()`, `c_void_ptr()`, `c_const_char_ptr()`
- **TypeInspector** (`src/semantic/registry.rs`): High-level type queries
  - `is_integer`, `is_float`, `is_numeric`, `is_pointer`, `is_bool`
  - `type_name()` for diagnostic messages
  - `validate_ffi_type()` for FFI boundary checks
  - `is_lossless_conversion()` for widening checks
- **CoercionEngine** (`src/semantic/coercion.rs`): Full coercion matrix
  - `CoercionResult` enum: `Identity`, `Implicit`, `RequiresCast`, `Incompatible`
  - `can_coerce(from, to)` — complete coercion rules
  - `common_type(left, right)` — binary operation type inference
  - Widening: I32→I64, I32→F64, F32→F64, String→*const I8 (implicit)
  - Narrowing: I64→I32, F64→I32 (requires explicit cast)
- **Raylib FFI** (`src/ffi/raylib_sys.rs` + `src/ffi/raylib.rs`):
  - Raw `extern "C"` declarations for 20 core functions
  - C types: `Color` (4 bytes), `Vector2` (8 bytes), `Texture2D` (20 bytes)
  - Safe wrapper layer with null-checks
  - `CallableRegistry` integration (28 functions, all `UnsafeRequired`)
  - Coverage: windowing, drawing, textures, input
- **Library target** (`src/lib.rs` + `Cargo.toml`): `[lib]` section for integration tests
- **Tests** (`tests/type_registry_test.rs` + `tests/raylib_ffi_test.rs`):
  - 38 assertions covering sizes, alignment, idempotency, FFI, coercion, layouts
- **Validator** (`scripts/validate_sprint1_type_registry.py`): 32-check structural validator

### Validation
- v1.21 executable logic: 9/9 checks PASSED
- Sprint 1.1 structural: 32/32 checks PASSED

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
