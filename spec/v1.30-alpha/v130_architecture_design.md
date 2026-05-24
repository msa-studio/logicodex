# Logicodex v1.30 Systems-Grade Architecture Baseline

**Document status:** documentation-first engineering blueprint and design-freeze candidate.

**Repository baseline:** current logicodex v 1.21 alpha.

**Implementation policy:** this document defines future skeleton structures and compiler contracts only; it does not claim that the v1.30 runtime, type registry, layout engine, ownership model, or FFI backend are implemented today.

## 1. Architecture Philosophy and Compiler Boundary

Logicodex v1.30 should evolve from the current alias-to-canonical frontend into a systems-grade compiler architecture without weakening the current compiler integrity boundary. The guiding philosophy is **human-readable frontend syntax with industrial-grade backend semantics**. Malay-first beginner aliases, English pseudocode aliases, and expert canonical shorthand should continue to normalize deterministically before parsing, while deeper systems features must be represented by explicit AST, HIR, semantic, layout, and backend contracts.

> No hidden magic is allowed at machine-code generation time. Memory safety boundaries, ABI decisions, struct layout, enum representation, and unsafe or FFI authorization must be resolved before LLVM IR emission.

The intended high-level pipeline is shown below. The current v1.21-alpha compiler already has lexer, parser, AST, semantic, and codegen layers for the executable subset. The v1.30 blueprint introduces a documented **HIR lowering** and **registry-backed type system** between the syntactic AST and backend-oriented IR generation.

```text
[ .ldx Source File ]
        │
        ▼
[ Lexer Canonicalization Map ]
        │
        ▼
[ Parser: Canonical AST + Span ]
        │
        ▼
[ HIR Lowering + Early Normalization ]
        │
        ▼
[ Semantic Gatekeeper + TypeRegistry + LayoutEngine + FFI Policy ]
        │
        ▼
[ LLVM IR Codegen ] ──► [ Target Object / Executable ]
```

| Layer | v1.30 responsibility | Integrity rule |
|---|---|---|
| Lexer | Normalize all surface aliases into canonical token identities. | Alias choice must not change semantics. |
| Parser | Build a span-rich canonical AST. | Unsupported constructs must fail with bilingual diagnostics, not partial lowering. |
| HIR lowering | Remove syntactic noise and prepare normalized semantic input. | HIR must preserve source spans for diagnostics. |
| Semantic gatekeeper | Resolve types, unsafe boundaries, layout, ABI, and ownership-relevant facts. | Invalid memory or FFI behavior must stop before codegen. |
| Codegen | Emit LLVM IR only for validated semantic objects. | Backend must not invent missing layout, ABI, or safety decisions. |

## 2. Source Span Skeleton

Every AST and HIR node that can produce a user-facing diagnostic should carry a source span. The span object is deliberately small, copyable, and independent from the text storage system. File metadata can be stored elsewhere in a source map or compilation session.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FileId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    pub file_id: FileId,
    pub start_line: u32,
    pub start_col: u32,
    pub end_line: u32,
    pub end_col: u32,
}

impl Span {
    pub const fn new(
        file_id: FileId,
        start_line: u32,
        start_col: u32,
        end_line: u32,
        end_col: u32,
    ) -> Self {
        Self {
            file_id,
            start_line,
            start_col,
            end_line,
            end_col,
        }
    }

    pub const fn unknown() -> Self {
        Self {
            file_id: FileId(0),
            start_line: 0,
            start_col: 0,
            end_line: 0,
            end_col: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spanned<T> {
    pub node: T,
    pub span: Span,
}
```

The diagnostic policy remains consistent with the current repository language rule. Documentation prose stays **English**, while emitted compiler diagnostics use a **Malay / English** format.

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub code: DiagnosticCode,
    pub severity: Severity,
    pub message_ms: String,
    pub message_en: String,
    pub primary_span: Span,
    pub notes: Vec<DiagnosticNote>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiagnosticCode {
    ParserUnsupportedFeature,
    TypeMismatch,
    UnsafeBoundaryViolation,
    FfiBoundaryViolation,
    LayoutError,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticNote {
    pub span: Option<Span>,
    pub message_ms: String,
    pub message_en: String,
}
```

## 3. AST Skeleton for Systems Features

The AST should remain close to source syntax but must avoid preserving cosmetic alias distinctions. By the time parsing occurs, `selagi` and `while`, or `bentuk` and `struct`, should already be the same canonical token kind.

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct ModuleAst {
    pub items: Vec<Spanned<ItemAst>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ItemAst {
    Function(FunctionAst),
    Struct(StructDeclAst),
    Enum(EnumDeclAst),
    ExternBlock(ExternBlockAst),
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionAst {
    pub name: String,
    pub params: Vec<ParamAst>,
    pub return_type: Option<TypeAst>,
    pub body: BlockAst,
    pub is_unsafe: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockAst {
    pub statements: Vec<Spanned<StmtAst>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StmtAst {
    Let {
        name: String,
        ty: Option<TypeAst>,
        value: Option<ExprAst>,
    },
    Assign {
        target: ExprAst,
        value: ExprAst,
    },
    While {
        condition: ExprAst,
        body: BlockAst,
    },
    Loop {
        body: BlockAst,
    },
    Break,
    Continue,
    UnsafeBlock(BlockAst),
    Expr(ExprAst),
    Return(Option<ExprAst>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExprAst {
    Literal(LiteralAst),
    Variable(String),
    Binary {
        left: Box<ExprAst>,
        op: BinaryOpAst,
        right: Box<ExprAst>,
    },
    Unary {
        op: UnaryOpAst,
        expr: Box<ExprAst>,
    },
    Call {
        callee: Box<ExprAst>,
        args: Vec<ExprAst>,
    },
    Field {
        base: Box<ExprAst>,
        field: String,
    },
    Cast {
        expr: Box<ExprAst>,
        target: TypeAst,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOpAst {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    NotEq,
    Lt,
    Lte,
    Gt,
    Gte,
    LogicalAnd,
    LogicalOr,
    BitAnd,
    BitOr,
    BitXor,
    ShiftLeft,
    ShiftRight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOpAst {
    Negate,
    LogicalNot,
    AddressOf,
    Deref,
}
```

The declaration skeletons below document the future shape of user-defined types and FFI-facing items. They are not intended to bypass the current parser trap for complex declarations in v1.21-alpha.

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct StructDeclAst {
    pub name: String,
    pub fields: Vec<FieldAst>,
    pub attributes: Vec<AttributeAst>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FieldAst {
    pub name: String,
    pub ty: TypeAst,
    pub visibility: VisibilityAst,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumDeclAst {
    pub name: String,
    pub variants: Vec<EnumVariantAst>,
    pub repr: Option<EnumReprAst>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumVariantAst {
    pub name: String,
    pub payload: EnumPayloadAst,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EnumPayloadAst {
    Unit,
    Tuple(Vec<TypeAst>),
    Struct(Vec<FieldAst>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExternBlockAst {
    pub abi: CallingConventionAst,
    pub functions: Vec<ExternFnAst>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExternFnAst {
    pub name: String,
    pub params: Vec<ParamAst>,
    pub return_type: TypeAst,
    pub is_variadic: bool,
}
```

## 4. HIR Lowering Skeleton

The HIR is a normalized semantic input layer. It should remove purely syntactic distinctions, attach resolved symbols where possible, and keep spans for all nodes that can fail semantic validation.

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct HirModule {
    pub items: Vec<Spanned<HirItem>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HirItem {
    Function(HirFunction),
    Struct(HirStructDecl),
    Enum(HirEnumDecl),
    ExternFunction(HirExternFunction),
}

#[derive(Debug, Clone, PartialEq)]
pub struct HirFunction {
    pub symbol: SymbolId,
    pub params: Vec<HirParam>,
    pub return_type: TypeRef,
    pub body: HirBlock,
    pub safety: SafetyContext,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HirBlock {
    pub statements: Vec<Spanned<HirStmt>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HirStmt {
    Let {
        local: LocalId,
        ty: TypeRef,
        value: Option<HirExpr>,
    },
    Assign {
        target: HirExpr,
        value: HirExpr,
    },
    While {
        condition: HirExpr,
        body: HirBlock,
    },
    Loop {
        body: HirBlock,
    },
    Break {
        target_depth: u32,
    },
    Continue {
        target_depth: u32,
    },
    UnsafeBlock(HirBlock),
    Expr(HirExpr),
    Return(Option<HirExpr>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct HirExpr {
    pub kind: HirExprKind,
    pub ty: TypeRef,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HirExprKind {
    Literal(LiteralAst),
    Local(LocalId),
    Global(SymbolId),
    Binary {
        left: Box<HirExpr>,
        op: BinaryOpAst,
        right: Box<HirExpr>,
    },
    Unary {
        op: UnaryOpAst,
        expr: Box<HirExpr>,
    },
    Call {
        callee: CallableId,
        args: Vec<HirExpr>,
    },
    Field {
        base: Box<HirExpr>,
        field_index: usize,
    },
    Cast {
        expr: Box<HirExpr>,
        target: TypeRef,
    },
}
```

The lowering pass is intentionally separated from semantic validation. Its job is to normalize and bind shallow identities, while the semantic gatekeeper remains responsible for type equivalence, memory layout, ABI enforcement, and unsafe authorization.

```rust
pub struct LoweringContext<'a> {
    pub symbols: &'a mut SymbolTable,
    pub diagnostics: Vec<Diagnostic>,
}

impl<'a> LoweringContext<'a> {
    pub fn lower_module(&mut self, module: ModuleAst) -> Result<HirModule, Vec<Diagnostic>> {
        todo!("lower AST into HIR while preserving spans")
    }

    fn lower_statement(&mut self, stmt: Spanned<StmtAst>) -> Option<Spanned<HirStmt>> {
        todo!("lower one AST statement into one normalized HIR statement")
    }
}
```

## 5. TypeRegistry and TypeId Skeleton

String-based type comparison is not a sufficient long-term base for user-defined structs, enums, pointers, arrays, or FFI signatures. The v1.30 baseline should use interned type identities. The semantic layer then compares `TypeId` values rather than repeatedly comparing strings.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StructLayoutId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EnumLayoutId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeRef {
    pub id: TypeId,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeKind {
    Primitive(PrimitiveType),
    Pointer {
        pointee: TypeId,
        mutability: Mutability,
    },
    Struct(StructLayoutId),
    Enum(EnumLayoutId),
    Array {
        element: TypeId,
        len: usize,
    },
    Function(CallableId),
    Never,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrimitiveType {
    Bool,
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
    String,
    Unit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Mutability {
    Immutable,
    Mutable,
}

#[derive(Debug, Default)]
pub struct TypeRegistry {
    kinds: Vec<TypeKind>,
    primitive_cache: PrimitiveTypeIds,
}

#[derive(Debug, Clone, Copy)]
pub struct PrimitiveTypeIds {
    pub bool_: TypeId,
    pub i64_: TypeId,
    pub f64_: TypeId,
    pub string: TypeId,
    pub unit: TypeId,
    pub never: TypeId,
    pub unknown: TypeId,
}

impl TypeRegistry {
    pub fn new() -> Self {
        todo!("initialize primitive TypeId values in a deterministic order")
    }

    pub fn intern(&mut self, kind: TypeKind) -> TypeId {
        todo!("return an existing TypeId for an equivalent TypeKind or insert a new one")
    }

    pub fn kind(&self, id: TypeId) -> Option<&TypeKind> {
        self.kinds.get(id.0 as usize)
    }

    pub fn is_equivalent(&self, left: TypeId, right: TypeId) -> bool {
        left == right
    }
}
```

## 6. StructLayout and LayoutEngine Skeleton

The future struct implementation must calculate size, alignment, field offsets, and packed-layout behavior before codegen. LLVM IR generation should consume validated layout metadata rather than recomputing field layout heuristically.

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructLayout {
    pub name: String,
    pub fields: Vec<StructFieldLayout>,
    pub total_size_bytes: usize,
    pub alignment_bytes: usize,
    pub is_packed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructFieldLayout {
    pub name: String,
    pub ty: TypeId,
    pub offset_bytes: usize,
    pub size_bytes: usize,
    pub alignment_bytes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayoutRequest {
    pub name: String,
    pub fields: Vec<LayoutFieldRequest>,
    pub attributes: Vec<LayoutAttribute>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayoutFieldRequest {
    pub name: String,
    pub ty: TypeId,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutAttribute {
    Packed,
    ReprC,
}

pub struct LayoutEngine<'a> {
    pub types: &'a TypeRegistry,
    pub target: TargetLayout,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetLayout {
    pub pointer_size_bytes: usize,
    pub pointer_alignment_bytes: usize,
    pub integer_alignment_bytes: usize,
}

impl<'a> LayoutEngine<'a> {
    pub fn compute_struct_layout(
        &self,
        request: LayoutRequest,
    ) -> Result<StructLayout, Diagnostic> {
        todo!("compute natural alignment, optional packed layout, and field offsets")
    }

    fn align_to(value: usize, alignment: usize) -> usize {
        if alignment == 0 {
            value
        } else {
            (value + alignment - 1) & !(alignment - 1)
        }
    }
}
```

| Rule | Expected behavior | Diagnostic boundary |
|---|---|---|
| Natural alignment | Each field is placed at the next address compatible with its alignment. | Invalid or unknown field type stops layout. |
| Packed layout | Padding is removed and alignment becomes one byte unless target policy rejects it. | Packed access may require unsafe or target-specific validation later. |
| Repr C layout | Field order is preserved for ABI-facing structures. | Unsupported target ABI stops before codegen. |
| LLVM field access | Codegen uses validated field indices and layout metadata. | Backend does not perform semantic layout inference. |

## 7. Enum Representation Skeleton

Enums require an explicit representation model before they can become executable. A future implementation should distinguish simple C-like enums, tagged unions, and payload-bearing variants.

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumLayout {
    pub name: String,
    pub tag_type: TypeId,
    pub variants: Vec<EnumVariantLayout>,
    pub total_size_bytes: usize,
    pub alignment_bytes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumVariantLayout {
    pub name: String,
    pub tag_value: u64,
    pub payload: EnumPayloadLayout,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnumPayloadLayout {
    Unit,
    Tuple(Vec<TypeId>),
    Struct(Vec<StructFieldLayout>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnumReprAst {
    Default,
    U8,
    U16,
    U32,
    U64,
    I32,
    I64,
}
```

The semantic analyzer should reject enum operations until the tag representation, payload layout, constructor syntax, pattern matching or equivalent branching, and backend lowering are all defined by tests.

## 8. CallableSignature, ABI, and FFI Gatekeeping Skeleton

External calls are inherently boundary-crossing operations. The v1.30 baseline should represent every callable through a stable signature object, then require semantic authorization before codegen emits calls to external symbols.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CallableId(pub u32);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallableSignature {
    pub name: String,
    pub params: Vec<TypeId>,
    pub return_type: TypeId,
    pub abi: CallingConvention,
    pub safety: CallableSafety,
    pub is_extern: bool,
    pub is_variadic: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CallingConvention {
    C,
    StdCall,
    SysCall,
    FastCall,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CallableSafety {
    Safe,
    UnsafeRequired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SafetyContext {
    Safe,
    Unsafe,
}

pub struct FfiGatekeeper<'a> {
    pub types: &'a TypeRegistry,
}

impl<'a> FfiGatekeeper<'a> {
    pub fn validate_call(
        &self,
        signature: &CallableSignature,
        args: &[HirExpr],
        context: SafetyContext,
        call_span: Span,
    ) -> Result<(), Diagnostic> {
        if signature.is_extern && context != SafetyContext::Unsafe {
            return Err(Diagnostic {
                code: DiagnosticCode::FfiBoundaryViolation,
                severity: Severity::Error,
                message_ms: format!(
                    "Ralat: Panggilan fungsi luar '{}' memerlukan blok unsafe",
                    signature.name
                ),
                message_en: format!(
                    "Error: External function call '{}' requires an unsafe block",
                    signature.name
                ),
                primary_span: call_span,
                notes: Vec::new(),
            });
        }

        todo!("validate argument count, variadic policy, ABI-compatible types, and explicit casts")
    }
}
```

The emitted diagnostic text should be rendered as one bilingual message at the frontend boundary, for example:

```text
Ralat: Panggilan fungsi luar 'InitWindow' memerlukan blok unsafe / Error: External function call 'InitWindow' requires an unsafe block
--> main.ldx:14:5
```

## 9. Semantic Gatekeeper Skeleton

The semantic gatekeeper coordinates type checking, loop legality, unsafe authorization, layout, and callable validation. It should be the last authority before backend lowering.

```rust
pub struct SemanticContext {
    pub types: TypeRegistry,
    pub symbols: SymbolTable,
    pub callables: CallableRegistry,
    pub diagnostics: Vec<Diagnostic>,
    pub loop_depth: u32,
    pub safety_context: SafetyContext,
}

impl SemanticContext {
    pub fn check_module(&mut self, module: &HirModule) -> Result<(), Vec<Diagnostic>> {
        for item in &module.items {
            self.check_item(item);
        }

        if self.diagnostics.is_empty() {
            Ok(())
        } else {
            Err(self.diagnostics.clone())
        }
    }

    fn check_item(&mut self, item: &Spanned<HirItem>) {
        todo!("dispatch function, struct, enum, and extern semantic checks")
    }

    fn check_statement(&mut self, stmt: &Spanned<HirStmt>) {
        match &stmt.node {
            HirStmt::Break { .. } | HirStmt::Continue { .. } if self.loop_depth == 0 => {
                self.diagnostics.push(Diagnostic {
                    code: DiagnosticCode::UnsafeBoundaryViolation,
                    severity: Severity::Error,
                    message_ms: "Ralat: Kawalan gelung digunakan di luar gelung".to_string(),
                    message_en: "Error: Loop control used outside a loop".to_string(),
                    primary_span: stmt.span,
                    notes: Vec::new(),
                });
            }
            _ => todo!("check the remaining HIR statement forms"),
        }
    }
}
```

## 10. Codegen Contract Skeleton

The backend must receive only semantically valid HIR and layout metadata. It should not accept raw AST declarations for complex systems features.

```rust
pub struct CodegenInput<'a> {
    pub hir: &'a HirModule,
    pub types: &'a TypeRegistry,
    pub layouts: &'a LayoutRegistry,
    pub callables: &'a CallableRegistry,
}

pub struct LayoutRegistry {
    pub structs: Vec<StructLayout>,
    pub enums: Vec<EnumLayout>,
}

pub struct CallableRegistry {
    pub signatures: Vec<CallableSignature>,
}

pub struct SymbolTable;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SymbolId(pub u32);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LocalId(pub u32);

pub trait CodegenBackend {
    type Error;

    fn emit_module(&mut self, input: CodegenInput<'_>) -> Result<(), Self::Error>;
    fn emit_function(&mut self, function: &HirFunction) -> Result<(), Self::Error>;
    fn emit_struct_type(&mut self, layout: &StructLayout) -> Result<(), Self::Error>;
    fn emit_extern_function(&mut self, signature: &CallableSignature) -> Result<(), Self::Error>;
}
```

## 11. Phased Implementation Roadmap

The roadmap should preserve the current v1.21-alpha split implementation and avoid mixing too many semantic layers in one commit. The following sequence keeps compiler integrity measurable.

| Phase | Scope | Acceptance signal |
|---|---|---|
| v1.30-A | Add dormant skeleton modules and documentation for `Span`, `TypeId`, `TypeRegistry`, `StructLayout`, `CallableSignature`, and HIR contracts. | Rust code compiles with no behavioral change; documentation states the boundary clearly. |
| v1.30-B | Attach spans to AST nodes and parse errors. | Parser diagnostics identify source ranges and remain bilingual. |
| v1.30-C | Introduce HIR lowering for the current executable subset. | Existing v1.21 executable examples compile through AST-to-HIR-to-codegen without regression. |
| v1.30-D | Activate TypeRegistry for primitives and function signatures. | Type checking uses `TypeId`; regression tests cover primitive equivalence and mismatch diagnostics. |
| v1.30-E | Implement StructLayout for declaration-only structs. | Unit tests verify size, alignment, and field offsets without enabling unsafe field access prematurely. |
| v1.30-F | Add extern signatures and unsafe block validation. | External calls outside unsafe blocks fail semantically with bilingual diagnostics. |
| v1.30-G | Connect validated layout and FFI metadata to LLVM codegen. | Minimal native example links against an explicitly configured external symbol. |

## 12. Non-Goals for the First v1.30 Skeleton Commit

The first skeleton commit should not implement a full borrow checker, enum pattern matching, implicit FFI coercions, object lifetime tracking, packed memory dereferencing, or target-specific ABI lowering. These are design-dependent systems features and should be activated only after the registry, span, HIR, and layout foundations are tested.

## 13. Maintainer Checklist

| Check | Required result |
|---|---|
| Documentation language | Prose remains English. Malay words appear only as language aliases, examples, or bilingual diagnostic text. |
| Diagnostic language | User-facing compiler diagnostics are rendered as Malay / English pairs. |
| Compiler behavior | Skeleton documentation does not claim implementation that does not exist. |
| Parser safety | Unsupported complex declarations fail explicitly until AST, HIR, semantic, and backend layers are ready. |
| Backend safety | Codegen consumes semantic facts; it never invents layout, ABI, or unsafe authorization decisions. |

This document can be used as the canonical starting point for pair-programming v1.30 architecture work. It intentionally records **Rust skeleton structures** in documentation first so future implementation commits can be reviewed against a stable baseline rather than improvised directly inside the compiler runtime.
