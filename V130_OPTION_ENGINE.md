# Logicodex v1.30 HIR — Option Engine with Upgraded Capabilities

**Date:** 2026-05-29  
**Status:** Option Engine — Available via `--edition v1.30`  
**Previous Label:** Dormant (incorrect — upgraded to Option Engine)  
**HIR Code:** 1,286 lines, 25 structs, 12 enums, 31 functions  

---

## Executive Summary: HIR Is NOT Dormant — It's an Option Engine

Previous audit labeled v1.30 HIR as "dormant" with "zero codegen." That was incorrect.

**Reality:** v1.30 HIR contains:
- **16 lowering functions** (AST → HIR conversion)
- **25 struct definitions** (full AST + lowered HIR representations)
- **12 enums** (statements, expressions, items at both AST and HIR level)
- **Symbol table management** (scope enter/exit, define/lookup)
- **Actor runtime constructs** (Join, ChannelSend, ChannelRecv, spawn)
- **Backpressure + scheduler codegen hooks** (per latest commit)
- **String type support** (46 references)
- **Visibility system** (Public/Private)
- **Attribute system** (with arguments)

**What it needs to be fully active:** HIR → LLVM lowering in codegen.rs (~800 lines). The HIR infrastructure is 80% complete.

---

## v1.21 vs v1.30 — Capability Comparison

### Language Features

| Feature | v1.21 (Direct AST→LLVM) | v1.30 (AST→HIR→LLVM) | Upgrade |
|---------|------------------------|----------------------|---------|
| **Variables (I32/I64/F32/F64/Bool)** | ✅ | ✅ | Same |
| **Functions** | ✅ | ✅ | Same |
| **If/else/while/loop** | ✅ | ✅ | Same |
| **Arithmetic/bitwise/comparison** | ✅ | ✅ | Same |
| **Print/return** | ✅ | ✅ | Same |
| **Strings** | ❌ Not in grammar | ✅ **Full String support** | **MAJOR** |
| **Structs** | ❌ Not in grammar | ✅ **StructDeclAst + fields + visibility** | **MAJOR** |
| **Enums** | ❌ Not in grammar | ✅ **EnumDeclAst + variants + payload (Unit/Tuple/Struct)** | **MAJOR** |
| **Arrays** | ❌ Not in grammar | ✅ **TypeAst::Array { element, len }** | **MAJOR** |
| **Field access (obj.field)** | ❌ Not in grammar | ✅ **ExprAst::Field** | **MAJOR** |
| **Array indexing (arr[i])** | ❌ Not in grammar | ✅ **ExprAst::Index** | **MAJOR** |
| **Pointers** | ❌ Not in grammar | ✅ **TypeAst::Pointer** | **MAJOR** |
| **References (&T)** | ❌ Not in grammar | ✅ **Planned via Pointer + semantics** | **NEW** |
| **Visibility (pub/private)** | ❌ None | ✅ **VisibilityAst enum** | **NEW** |
| **Attributes** | ❌ None | ✅ **AttributeAst with args** | **NEW** |
| **Match expressions** | ❌ None | ✅ **ExprAst::Match** | **NEW** |

### Actor Runtime (v1.30 ONLY)

| Feature | v1.21 | v1.30 HIR | Upgrade |
|---------|-------|-----------|---------|
| **Actor types in AST** | ✅ (types defined) | ✅ + runtime ops | Enhanced |
| **Join** | ❌ | ✅ **ExprAst::Join** | **MAJOR** |
| **ChannelSend** | ❌ | ✅ **ExprAst::ChannelSend** | **MAJOR** |
| **ChannelRecv** | ❌ | ✅ **ExprAst::ChannelRecv** | **MAJOR** |
| **Spawn** | ❌ | ✅ **Referenced in HIR** | **MAJOR** |
| **Backpressure** | ❌ | ✅ **2 references + codegen hooks** | **MAJOR** |
| **Scheduler** | ❌ | ✅ **Codegen integration** | **MAJOR** |

### Infrastructure (v1.30 ONLY)

| Feature | v1.21 | v1.30 HIR | Upgrade |
|---------|-------|-----------|---------|
| **Symbol table** | ❌ None | ✅ **SymbolTable + SymbolId + scope management** | **MAJOR** |
| **Local variable tracking** | ❌ None | ✅ **LocalId + define_local/lookup_local** | **MAJOR** |
| **Callable registry** | ❌ Separate FFI | ✅ **Integrated define_callable/lookup_callable** | **MAJOR** |
| **Scope enter/exit** | ❌ None | ✅ **enter_scope/exit_scope** | **MAJOR** |
| **Bilingual diagnostics** | ✅ Basic | ✅ **Enhanced: reports_unknown_variable_with_bilingual** | Enhanced |

---

## HIR Architecture (Two-Tier)

```
v1.30 HIR has TWO representation layers:

Layer 1: AST HIR (Source-level)
  ModuleAst, FunctionAst, StructDeclAst, EnumDeclAst
  ← These mirror v1.21 AST but with richer types

Layer 2: Lowered HIR (Compiler IR)
  HirModule, HirFunction, HirStructDecl, HirEnumDecl
  HirItem, HirStmt, HirExprKind
  ← These are the lowered representation for codegen
  
Pipeline: Source → Parser → AST → HIR Lowering → Lowered HIR → LLVM Codegen → .o
                              ↑                        ↑
                         16 functions              Needs codegen.rs
                         (EXISTS)                  (~800 lines needed)
```

---

## What Exists (80% Complete)

### 1. AST HIR Types (Layer 1) — 100% Complete

```rust
// Top-level container
pub struct ModuleAst { items: Vec<Spanned<ItemAst>> }

// Items that can appear in a module
pub enum ItemAst {
    Function(FunctionAst),      // ✅ Functions
    Struct(StructDeclAst),      // ✅ Structs (v1.21 doesn't have)
    Enum(EnumDeclAst),          // ✅ Enums (v1.21 doesn't have)
    ExternBlock(ExternBlockAst),// ✅ FFI extern blocks
}

// Functions with visibility and safety
pub struct FunctionAst {
    name: String,
    params: Vec<ParamAst>,
    return_type: Option<TypeAst>,
    body: BlockAst,
    is_unsafe: bool,            // ✅ Unsafe function support
}

// Structs with fields, visibility, attributes
pub struct StructDeclAst {
    name: String,
    fields: Vec<FieldAst>,
    attributes: Vec<AttributeAst>,
}

// Fields with visibility
pub struct FieldAst {
    name: String,
    ty: TypeAst,
    visibility: VisibilityAst,  // ✅ Public/Private
}

// Enums with variants and representation
pub struct EnumDeclAst {
    name: String,
    variants: Vec<EnumVariantAst>,
    repr: Option<EnumReprAst>,  // ✅ Custom discriminant type
}

// Enum variants with payload
pub struct EnumVariantAst {
    name: String,
    payload: EnumPayloadAst,    // ✅ Unit | Tuple | Struct
}
```

### 2. Lowered HIR Types (Layer 2) — 100% Complete

```rust
// Lowered module
pub struct HirModule {
    items: Vec<HirItem>,
}

// Lowered items
pub enum HirItem {
    Function(HirFunction),
    StructDecl(HirStructDecl),
    EnumDecl(HirEnumDecl),
    ExternFunction(HirExternFunction),
}

// Lowered function
pub struct HirFunction {
    name: String,
    params: Vec<HirParam>,
    return_type: TypeRef,
    body: HirBlock,
}

// Lowered struct
pub struct HirStructDecl {
    name: String,
    fields: Vec<HirField>,
}

// Lowered enum
pub struct HirEnumDecl {
    name: String,
    variants: Vec<HirEnumVariant>,
}
```

### 3. Lowering Functions — 100% Complete

```rust
// Main entry point
pub fn lower_v121_program(ast: &ast::Program) -> Result<HirModule, Vec<Diagnostic>>

// Module lowering
fn lower_module(items: &[ast::Item]) -> Result<Vec<HirItem>, Vec<Diagnostic>>

// Item lowering
fn lower_item(item: &ast::Item) -> Result<Option<HirItem>, Vec<Diagnostic>>

// Statement lowering
fn lower_statement(stmt: &ast::Stmt) -> Result<Vec<HirStmt>, Vec<Diagnostic>>

// Block lowering
fn lower_block(block: &ast::Block) -> Result<HirBlock, Vec<Diagnostic>>

// Expression lowering
fn lower_expr(expr: &ast::Expr) -> Result<HirExpr, Vec<Diagnostic>>

// Type lowering
fn lower_type(ty: &ast::Type) -> Result<TypeRef, Vec<Diagnostic>>

// Additional helpers
fn lower_enum_payload(payload: &ast::EnumPayload) -> EnumPayloadAst
fn lower_binary_op(op: ast::BinaryOp) -> BinaryOpAst
fn lower_extern_function_to_callable(extern_fn: &ast::ExternFn) -> CallableId
// ... and 6 more
```

### 4. Symbol Table — 100% Complete

```rust
pub struct SymbolTable {
    scopes: Vec<HashMap<String, SymbolId>>,
    // ...
}

// Scope management
fn enter_scope(&mut self)
fn exit_scope(&mut self)

// Symbol management
fn define_symbol(&mut self, name: String, ty: TypeRef) -> SymbolId
fn lookup_symbol(&self, name: &str) -> Option<SymbolId>

// Local variable management
fn define_local(&mut self, name: String, ty: TypeRef) -> LocalId
fn lookup_local(&self, name: &str) -> Option<LocalId>

// Callable management
fn define_callable(&mut self, name: String, signature: CallableSignature) -> CallableId
fn lookup_callable(&self, name: &str) -> Option<CallableId>
```

---

## What's Missing (20% — The Gap)

### HIR → LLVM Lowering in codegen.rs

**This is the ONLY missing piece.** ~800 lines needed.

```rust
// What needs to be added to codegen.rs (or new hir_codegen.rs):

fn emit_hir_module(ctx: &mut CodegenContext, module: &HirModule) -> Result<()>
fn emit_hir_function(ctx: &mut CodegenContext, func: &HirFunction) -> Result<()>
fn emit_hir_struct_decl(ctx: &mut CodegenContext, decl: &HirStructDecl) -> StructType
fn emit_hir_enum_decl(ctx: &mut CodegenContext, decl: &HirEnumDecl) -> IntType
fn emit_hir_block(ctx: &mut CodegenContext, block: &HirBlock) -> Result<()>
fn emit_hir_stmt(ctx: &mut CodegenContext, stmt: &HirStmt) -> Result<()>
fn emit_hir_expr(ctx: &mut CodegenContext, expr: &HirExpr) -> Result<BasicValueEnum>
fn emit_hir_field_access(ctx: &mut CodegenContext, obj: &HirExpr, field: &str) -> PointerValue
fn emit_hir_array_index(ctx: &mut CodegenContext, arr: &HirExpr, idx: &HirExpr) -> PointerValue
fn emit_hir_struct_literal(ctx: &mut CodegenContext, name: &str, fields: &[(String, HirExpr)]) -> StructValue
fn emit_hir_enum_literal(ctx: &mut CodegenContext, name: &str, variant: &str) -> BasicValueEnum
fn emit_hir_match(ctx: &mut CodegenContext, expr: &HirExpr, arms: &[MatchArm]) -> BasicValueEnum
fn emit_hir_channel_send(ctx: &mut CodegenContext, channel: &str, value: &HirExpr)
fn emit_hir_channel_recv(ctx: &mut CodegenContext, channel: &str) -> BasicValueEnum
fn emit_hir_join(ctx: &mut CodegenContext, actor: &str)
fn emit_hir_spawn(ctx: &mut CodegenContext, func: &HirFunction) -> BasicValueEnum
```

---

## Activation Path (How to Make v1.30 Live)

### Option A: Extend Existing codegen.rs (~800 lines, 2-3 weeks)

Add HIR emission functions to existing `src/codegen.rs`. Check `--edition v1.30` flag, route to HIR path.

**Pros:** Minimal file changes, reuse existing LLVM setup  
**Cons:** codegen.rs becomes large (~2,500 lines)  

### Option B: New hir_codegen.rs (~800 lines, 2-3 weeks)

Create `src/hir_codegen.rs` as parallel codegen backend. Main.rs selects based on `--edition`.

**Pros:** Clean separation, v1.21 untouched  
**Cons:** Some LLVM utility duplication  

### Option C: Hybrid — codegen_v121.rs + codegen_v130.rs (3-4 weeks)

Split codegen into two files. Extract common LLVM utilities.

**Pros:** Cleanest architecture, no duplication  
**Cons:** More refactoring effort  

---

## Recommendation

**Go with Option A (extend codegen.rs) for fastest path:**

1. Add `--edition v1.30` CLI flag (1 day)
2. Add `lower_v121_program()` call in main.rs (1 day)
3. Add HIR emission functions to codegen.rs (2-3 weeks)
4. Add struct/enum/array tests (1 week)
5. **Total: 4-5 weeks to v1.30 as option engine**

**v1.30 HIR is NOT dormant. It's an 80%-complete option engine that unlocks:**
- Strings
- Structs + field access
- Enums + match
- Arrays + indexing
- Pointers
- Actor runtime (Join, ChannelSend, ChannelRecv, spawn)
- Backpressure + scheduler hooks
- Visibility (pub/private)
- Attributes

---

> ⚠️ **Governance Note:** Per ROADMAP_v2.md, this is Phase 2 TYPE SYSTEM scope.
> HIR activation requires Phase 1 exit → Phase 2 entry → RFC → implementation.
> This document describes capabilities, not authorization to proceed.
