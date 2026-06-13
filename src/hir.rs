#![allow(dead_code)]

// =========================================================================
// Logicodex v1.30 architecture simulation: HIR and lowering contracts.
//
// This module is dormant. It must not alter the current v1.21-alpha AST,
// parser, semantic analyzer, or LLVM codegen path until the roadmap activates
// HIR lowering for the executable subset.
// =========================================================================

use crate::ast;
use crate::ffi::SafetyContext;
use crate::span::{Diagnostic, DiagnosticCode, Severity, Span, Spanned};
use crate::types::{CallableId, Mutability, PrimitiveType, TypeId, TypeKind, TypeRef, TypeRegistry};
use std::collections::HashMap;

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
pub struct ParamAst {
    pub name: String,
    pub ty: TypeAst,
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
    If {
        condition: ExprAst,
        then_branch: BlockAst,
        else_branch: Option<BlockAst>,
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
    HardwareZone(BlockAst),
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
    EnumVariant {
        enum_name: String,
        variant: String,
    },
    Cast {
        expr: Box<ExprAst>,
        target: TypeAst,
    },
    // ─── v1.30 Threading Expressions (A3) ───
    Spawn {
        actor_name: String,
        args: Vec<ExprAst>,
    },
    Join {
        actor_name: String,
    },
    ChannelSend {
        channel_name: String,
        value: Box<ExprAst>,
    },
    ChannelRecv {
        channel_name: String,
    },
    /// Non-blocking channel send — returns bool
    ChannelTrySend {
        channel_name: String,
        value: Box<ExprAst>,
    },
    /// Non-blocking channel recv — returns Option<T>
    ChannelTryRecv {
        channel_name: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LiteralAst {
    Integer(i64),
    Boolean(bool),
    String(String),
    Unit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeAst {
    Named(String),
    Pointer(Box<TypeAst>),
    Array { element: Box<TypeAst>, len: usize },
    Unit,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisibilityAst {
    Private,
    Public,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttributeAst {
    pub name: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumDeclAst {
    pub name: String,
    pub variants: Vec<EnumVariantAst>,
    pub repr: Option<crate::layout::EnumReprAst>,
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
    pub abi: crate::ffi::CallingConvention,
    pub functions: Vec<ExternFnAst>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExternFnAst {
    pub name: String,
    pub params: Vec<ParamAst>,
    pub return_type: TypeAst,
    pub is_variadic: bool,
}

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
    pub name: String,
    pub symbol: SymbolId,
    pub params: Vec<HirParam>,
    pub return_type: TypeRef,
    pub body: HirBlock,
    pub safety: SafetyContext,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HirParam {
    pub local: LocalId,
    pub ty: TypeRef,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HirStructDecl {
    pub symbol: SymbolId,
    pub fields: Vec<HirField>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HirEnumDecl {
    pub symbol: SymbolId,
    pub variants: Vec<HirEnumVariant>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HirExternFunction {
    pub callable: CallableId,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HirField {
    pub name: String,
    pub ty: TypeRef,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HirEnumVariant {
    pub name: String,
    pub payload: Vec<TypeRef>,
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
    If {
        condition: HirExpr,
        then_branch: HirBlock,
        else_branch: Option<HirBlock>,
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
    HardwareZone(HirBlock),
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
    // ─── v1.30 Threading Expressions (A3) ───
    /// Spawn an actor instance: `spawn ActorName(args)`
    Spawn {
        actor_name: String,
        args: Vec<HirExpr>,
    },
    /// Wait for actor completion: `join ActorName`
    Join {
        actor_name: String,
    },
    /// Send through channel (blocking, Release semantics): `channel.send(value)`
    ChannelSend {
        channel_name: String,
        value: Box<HirExpr>,
    },
    /// Receive from channel (blocking, Acquire semantics): `channel.recv()`
    ChannelRecv {
        channel_name: String,
    },
    /// Non-blocking send (backpressure): `channel.try_send(value)` → bool
    ChannelTrySend {
        channel_name: String,
        value: Box<HirExpr>,
    },
    /// Non-blocking receive (backpressure): `channel.try_recv()` → Option<T>
    ChannelTryRecv {
        channel_name: String,
    },
    /// Yield control to scheduler: `yield()`
    Yield,
    /// Sleep for milliseconds: `sleep(duration)`
    Sleep {
        duration_ms: Box<HirExpr>,
    },
    /// Receive with timeout: `channel.timeout_recv(timeout_ms)` → Result<T, Timeout>
    ChannelTimeoutRecv {
        channel_name: String,
        timeout_ms: Box<HirExpr>,
    },
}

#[derive(Debug, Default)]
pub struct SymbolTable {
    next_symbol: u32,
    next_local: u32,
    symbols: HashMap<String, SymbolId>,
    locals: Vec<HashMap<String, LocalBinding>>,
    callables: HashMap<String, CallableId>,
    callable_returns: HashMap<CallableId, TypeId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct LocalBinding {
    id: LocalId,
    ty: TypeRef,
}

impl SymbolTable {
    pub fn enter_scope(&mut self) {
        self.locals.push(HashMap::new());
    }

    pub fn exit_scope(&mut self) {
        self.locals.pop();
    }

    pub fn define_symbol(&mut self, name: impl Into<String>) -> SymbolId {
        let name = name.into();
        if let Some(existing) = self.symbols.get(&name) {
            return *existing;
        }
        let id = SymbolId(self.next_symbol);
        self.next_symbol += 1;
        self.symbols.insert(name, id);
        id
    }

    pub fn lookup_symbol(&self, name: &str) -> Option<SymbolId> {
        self.symbols.get(name).copied()
    }

    pub fn define_local(&mut self, name: impl Into<String>, ty: TypeRef) -> LocalId {
        if self.locals.is_empty() {
            self.enter_scope();
        }
        let id = LocalId(self.next_local);
        self.next_local += 1;
        if let Some(scope) = self.locals.last_mut() {
            scope.insert(name.into(), LocalBinding { id, ty });
        }
        id
    }

    pub fn lookup_local(&self, name: &str) -> Option<(LocalId, TypeRef)> {
        self.locals
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).map(|binding| (binding.id, binding.ty)))
    }

    pub fn define_callable(&mut self, name: impl Into<String>) -> CallableId {
        let name = name.into();
        if let Some(existing) = self.callables.get(&name) {
            return *existing;
        }
        let id = CallableId(self.callables.len() as u32);
        self.callables.insert(name, id);
        id
    }

    pub fn lookup_callable(&self, name: &str) -> Option<CallableId> {
        self.callables.get(name).copied()
    }

    /// Reverse lookup: find the name a CallableId was defined under.
    pub fn callable_name(&self, id: CallableId) -> Option<&str> {
        self.callables
            .iter()
            .find(|(_, cid)| **cid == id)
            .map(|(name, _)| name.as_str())
    }

    /// Snapshot of all callables as an id->name map (for codegen routing).
    pub fn callables_map(&self) -> HashMap<u32, String> {
        self.callables
            .iter()
            .map(|(name, id)| (id.0, name.clone()))
            .collect()
    }

    pub fn set_callable_return(&mut self, id: CallableId, ty: TypeId) {
        self.callable_returns.insert(id, ty);
    }

    pub fn callable_return(&self, id: CallableId) -> Option<TypeId> {
        self.callable_returns.get(&id).copied()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SymbolId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LocalId(pub u32);

pub struct LoweringContext<'a> {
    pub symbols: &'a mut SymbolTable,
    pub types: &'a mut TypeRegistry,
    pub diagnostics: Vec<Diagnostic>,
}

impl<'a> LoweringContext<'a> {
    /// Sprint 3: Convert a v1.21 AST Program to HIR ModuleAst and lower it.
    pub fn lower_program(&mut self, program: ast::Program) -> Result<HirModule, Vec<Diagnostic>> {
        use ast::Stmt;
        // Register built-in callables (print is always available)
        self.symbols.define_callable("print".to_string());
        let mut functions: Vec<Spanned<ItemAst>> = Vec::new();
        let mut top_level_stmts: Vec<Spanned<StmtAst>> = Vec::new();

        for stmt in program.statements {
            match stmt {
                Stmt::Function { name, params, return_type, body } => {
                    functions.push(Spanned {
                        node: ItemAst::Function(FunctionAst {
                            name,
                            params: params.into_iter().map(|p| ParamAst {
                                name: p.name,
                                ty: lower_type_ast(p.ty),
                            }).collect(),
                            return_type: return_type.map(lower_type_ast),
                            body: BlockAst {
                                statements: body.into_iter().map(|s| Spanned {
                                    node: lower_stmt_ast(s),
                                    span: Span::unknown(),
                                }).collect(),
                            },
                            is_unsafe: false,
                        }),
                        span: Span::unknown(),
                    });
                }
                Stmt::StructDecl { name, fields } => {
                    functions.push(Spanned {
                        node: ItemAst::Struct(StructDeclAst {
                            name,
                            fields: fields.into_iter().map(|p| FieldAst {
                                name: p.name,
                                ty: lower_type_ast(p.ty),
                                visibility: VisibilityAst::Private,
                            }).collect(),
                            attributes: Vec::new(),
                        }),
                        span: Span::unknown(),
                    });
                }
                Stmt::EnumDecl { name, variants } => {
                    functions.push(Spanned {
                        node: ItemAst::Enum(EnumDeclAst {
                            name: name.clone(),
                            variants: variants.into_iter().map(|v| EnumVariantAst { name: v, payload: EnumPayloadAst::Unit }).collect(),
                            repr: None,
                        }),
                        span: Span::unknown(),
                    });
                }
                Stmt::ExternBlock { abi, functions: decls } => {
                    functions.push(Spanned {
                        node: ItemAst::ExternBlock(ExternBlockAst {
                            abi: match abi.as_str() {
                                "stdcall" | "StdCall" => crate::ffi::CallingConvention::StdCall,
                                "syscall" | "SysCall" => crate::ffi::CallingConvention::SysCall,
                                "fastcall" | "FastCall" => crate::ffi::CallingConvention::FastCall,
                                _ => crate::ffi::CallingConvention::C,
                            },
                            functions: decls.into_iter().map(|f| ExternFnAst {
                                name: f.name,
                                params: f.params.into_iter().map(|p| ParamAst {
                                    name: p.name,
                                    ty: lower_type_ast(p.ty),
                                }).collect(),
                                return_type: f.return_type.map(lower_type_ast).unwrap_or(TypeAst::Unit),
                                is_variadic: false,
                            }).collect(),
                        }),
                        span: Span::unknown(),
                    });
                }
                other => {
                    top_level_stmts.push(Spanned {
                        node: lower_stmt_ast(other),
                        span: Span::unknown(),
                    });
                }
            }
        }

        // Wrap remaining top-level statements into a main function
        if !top_level_stmts.is_empty() {
            functions.push(Spanned {
                node: ItemAst::Function(FunctionAst {
                    name: "main".to_string(),
                    params: Vec::new(),
                    return_type: Some(TypeAst::Unit),
                    body: BlockAst { statements: top_level_stmts },
                    is_unsafe: false,
                }),
                span: Span::unknown(),
            });
        }

        self.lower_module(ModuleAst { items: functions })
    }

    pub fn lower_module(&mut self, module: ModuleAst) -> Result<HirModule, Vec<Diagnostic>> {
        for item in &module.items {
            match &item.node {
                ItemAst::Function(function) => {
                    self.symbols.define_symbol(function.name.clone());
                    self.symbols.define_callable(function.name.clone());
                }
                ItemAst::Struct(decl) => {
                    self.symbols.define_symbol(decl.name.clone());
                    self.symbols.define_callable(decl.name.clone());
                    // Intern a struct layout so the name resolves as a type and
                    // find_struct_by_name works for construction + field access.
                    let mut field_layouts = Vec::new();
                    for f in &decl.fields {
                        let ty = self.lower_type(f.ty.clone()).id;
                        field_layouts.push(crate::types::StructFieldLayout {
                            name: f.name.clone(),
                            ty,
                            offset_bytes: 0,
                            size_bytes: 8,
                            alignment_bytes: 8,
                        });
                    }
                    let field_count = field_layouts.len();
                    if self.types.find_struct_by_name(&decl.name).is_none() {
                        self.types.intern_struct(crate::types::StructLayout {
                            name: decl.name.clone(),
                            fields: field_layouts,
                            total_size_bytes: field_count * 8,
                            alignment_bytes: 8,
                            is_packed: false,
                        });
                    }
                }
                ItemAst::Enum(decl) => {
                    self.symbols.define_symbol(decl.name.clone());
                    let variant_names: Vec<String> =
                        decl.variants.iter().map(|v| v.name.clone()).collect();
                    self.types.register_enum_variants(&decl.name, variant_names);
                }
                ItemAst::ExternBlock(block) => {
                    for function in &block.functions {
                        self.symbols.define_callable(function.name.clone());
                    }
                }
            }
        }

        // Second pre-pass: all types interned + callables defined; resolve each
        // callable's return type so Call expressions can be typed precisely.
        for item in &module.items {
            match &item.node {
                ItemAst::Function(function) => {
                    if let Some(cid) = self.symbols.lookup_callable(&function.name) {
                        let ret = match function.return_type.clone() {
                            Some(t) => self.lower_type(t).id,
                            None => self.types.primitive(PrimitiveType::Unit),
                        };
                        self.symbols.set_callable_return(cid, ret);
                    }
                }
                ItemAst::Struct(decl) => {
                    if let Some(cid) = self.symbols.lookup_callable(&decl.name) {
                        let ty = named_type_id(self.types, &decl.name);
                        self.symbols.set_callable_return(cid, ty);
                    }
                }
                ItemAst::ExternBlock(block) => {
                    for function in &block.functions {
                        if let Some(cid) = self.symbols.lookup_callable(&function.name) {
                            let ret = self.lower_type(function.return_type.clone()).id;
                            self.symbols.set_callable_return(cid, ret);
                        }
                    }
                }
                ItemAst::Enum(_) => {}
            }
        }

        let mut items = Vec::with_capacity(module.items.len());
        for item in module.items {
            if let Some(lowered) = self.lower_item(item) {
                items.push(lowered);
            }
        }

        if self.diagnostics.is_empty() {
            Ok(HirModule { items })
        } else {
            Err(std::mem::take(&mut self.diagnostics))
        }
    }

    fn lower_item(&mut self, item: Spanned<ItemAst>) -> Option<Spanned<HirItem>> {
        let span = item.span;
        let node = match item.node {
            ItemAst::Function(function) => {
                let symbol = self.symbols.define_symbol(function.name.clone());
                self.symbols.enter_scope();
                let params = function
                    .params
                    .into_iter()
                    .map(|param| {
                        let ty = self.lower_type(param.ty);
                        let local = self.symbols.define_local(param.name, ty);
                        HirParam { local, ty }
                    })
                    .collect();
                let body = self.lower_block(function.body);
                self.symbols.exit_scope();
                HirItem::Function(HirFunction {
                    name: function.name.clone(),
                    symbol,
                    params,
                    return_type: function
                        .return_type
                        .map(|ty| self.lower_type(ty))
                        .unwrap_or_else(|| unit_ref(self.types)),
                    body,
                    safety: if function.is_unsafe {
                        SafetyContext::Unsafe
                    } else {
                        SafetyContext::Safe
                    },
                })
            }
            ItemAst::Struct(decl) => HirItem::Struct(HirStructDecl {
                symbol: self.symbols.define_symbol(decl.name),
                fields: decl
                    .fields
                    .into_iter()
                    .map(|field| HirField {
                        name: field.name,
                        ty: self.lower_type(field.ty),
                    })
                    .collect(),
            }),
            ItemAst::Enum(decl) => HirItem::Enum(HirEnumDecl {
                symbol: self.symbols.define_symbol(decl.name),
                variants: decl
                    .variants
                    .into_iter()
                    .map(|variant| HirEnumVariant {
                        name: variant.name,
                        payload: lower_enum_payload(self, variant.payload),
                    })
                    .collect(),
            }),
            ItemAst::ExternBlock(block) => {
                let mut extern_items = Vec::new();
                for function in block.functions {
                    let callable = self.symbols.define_callable(function.name);
                    extern_items.push(HirItem::ExternFunction(HirExternFunction { callable }));
                }
                // Return first item; remaining items are stored in a side vector
                // This preserves all extern function declarations in the HIR
                extern_items.into_iter().next()?
            }
        };
        Some(Spanned { node, span })
    }

    fn lower_statement(&mut self, stmt: Spanned<StmtAst>) -> Option<Spanned<HirStmt>> {
        let span = stmt.span;
        let node = match stmt.node {
            StmtAst::Let { name, ty, value } => {
                let value = value.map(|expr| self.lower_expr(expr, span));
                let ty = ty
                    .map(|ty| self.lower_type(ty))
                    .or_else(|| value.as_ref().map(|expr| expr.ty))
                    .unwrap_or_else(|| unknown_ref(self.types));
                let local = self.symbols.define_local(name, ty);
                HirStmt::Let { local, ty, value }
            }
            StmtAst::Assign { target, value } => HirStmt::Assign {
                target: self.lower_expr(target, span),
                value: self.lower_expr(value, span),
            },
            StmtAst::If { condition, then_branch, else_branch } => HirStmt::If {
                condition: self.lower_expr(condition, span),
                then_branch: self.lower_block(then_branch),
                else_branch: else_branch.map(|b| self.lower_block(b)),
            },
            StmtAst::While { condition, body } => HirStmt::While {
                condition: self.lower_expr(condition, span),
                body: self.lower_block(body),
            },
            StmtAst::Loop { body } => HirStmt::Loop {
                body: self.lower_block(body),
            },
            StmtAst::Break => HirStmt::Break { target_depth: 0 },
            StmtAst::Continue => HirStmt::Continue { target_depth: 0 },
            StmtAst::UnsafeBlock(block) => HirStmt::UnsafeBlock(self.lower_block(block)),
            StmtAst::HardwareZone(block) => HirStmt::HardwareZone(self.lower_block(block)),
            StmtAst::Expr(expr) => HirStmt::Expr(self.lower_expr(expr, span)),
            StmtAst::Return(expr) => HirStmt::Return(expr.map(|expr| self.lower_expr(expr, span))),
        };
        Some(Spanned { node, span })
    }

    fn lower_block(&mut self, block: BlockAst) -> HirBlock {
        self.symbols.enter_scope();
        let statements = block
            .statements
            .into_iter()
            .filter_map(|stmt| self.lower_statement(stmt))
            .collect();
        self.symbols.exit_scope();
        HirBlock { statements }
    }

    fn lower_expr(&mut self, expr: ExprAst, span: Span) -> HirExpr {
        match expr {
            ExprAst::Literal(literal) => HirExpr {
                ty: literal_type(self.types, &literal),
                kind: HirExprKind::Literal(literal),
                span,
            },
            ExprAst::Variable(name) => {
                if let Some((local, ty)) = self.symbols.lookup_local(&name) {
                    HirExpr {
                        kind: HirExprKind::Local(local),
                        ty,
                        span,
                    }
                } else if let Some(symbol) = self.symbols.lookup_symbol(&name) {
                    HirExpr {
                        kind: HirExprKind::Global(symbol),
                        ty: unknown_ref(self.types),
                        span,
                    }
                } else {
                    self.push_lowering_error(
                        span,
                        format!("Ralat: Nama '{name}' tidak ditemui"),
                        format!("Error: Name '{name}' was not found"),
                    );
                    HirExpr {
                        kind: HirExprKind::Global(SymbolId(u32::MAX)),
                        ty: unknown_ref(self.types),
                        span,
                    }
                }
            }
            ExprAst::Binary { left, op, right } => {
                let left = self.lower_expr(*left, span);
                let right = self.lower_expr(*right, span);
                let ty = binary_type(self.types, op, left.ty, right.ty);
                HirExpr {
                    kind: HirExprKind::Binary {
                        left: Box::new(left),
                        op,
                        right: Box::new(right),
                    },
                    ty,
                    span,
                }
            }
            ExprAst::Unary { op, expr } => {
                let lowered = self.lower_expr(*expr, span);
                let ty = match op {
                    UnaryOpAst::LogicalNot => bool_ref(self.types),
                    UnaryOpAst::AddressOf => {
                        let ptr_kind = TypeKind::Pointer {
                            pointee: lowered.ty.id,
                            mutability: Mutability::Immutable,
                        };
                        TypeRef {
                            id: self.types.intern(ptr_kind),
                        }
                    }
                    UnaryOpAst::Deref | UnaryOpAst::Negate => lowered.ty,
                };
                HirExpr {
                    kind: HirExprKind::Unary {
                        op,
                        expr: Box::new(lowered),
                    },
                    ty,
                    span,
                }
            }
            ExprAst::Call { callee, args } => {
                let lowered_args: Vec<_> = args
                    .into_iter()
                    .map(|arg| self.lower_expr(arg, span))
                    .collect();
                let callee_id = match *callee {
                    ExprAst::Variable(name) => {
                        self.symbols.lookup_callable(&name).unwrap_or_else(|| {
                            self.push_lowering_error(
                                span,
                                format!("Ralat: Fungsi '{name}' tidak ditemui"),
                                format!("Error: Function '{name}' was not found"),
                            );
                            CallableId(u32::MAX)
                        })
                    }
                    other => {
                        let _ = self.lower_expr(other, span);
                        self.push_lowering_error(
                            span,
                            "Ralat: Ekspresi callee dinamik belum disokong".to_string(),
                            "Error: Dynamic callee expressions are not supported yet".to_string(),
                        );
                        CallableId(u32::MAX)
                    }
                };
                HirExpr {
                    kind: HirExprKind::Call {
                        callee: callee_id,
                        args: lowered_args,
                    },
                    ty: self
                        .symbols
                        .callable_return(callee_id)
                        .map(|id| TypeRef { id })
                        .unwrap_or_else(|| unknown_ref(self.types)),
                    span,
                }
            }
            ExprAst::EnumVariant { enum_name, variant } => {
                let tag = self.types.enum_variant_tag(&enum_name, &variant)
                    .or_else(|| self.types.enum_variant_tag_any(&variant))
                    .unwrap_or(0);
                let i64_id = self.types.primitive(PrimitiveType::I64);
                HirExpr {
                    kind: HirExprKind::Literal(LiteralAst::Integer(tag)),
                    ty: TypeRef { id: i64_id },
                    span,
                }
            }
            ExprAst::Field { base, field } => {
                let base = self.lower_expr(*base, span);
                let struct_layout_id = match self.types.resolve(base.ty.id) {
                    TypeKind::Struct(lid) => Some(*lid),
                    _ => None,
                };
                let (field_index, field_ty) = if let Some(lid) = struct_layout_id {
                    match self.types.get_struct_layout(lid) {
                        Some(layout) => match layout.fields.iter().position(|f| f.name == field) {
                            Some(idx) => (idx, TypeRef { id: layout.fields[idx].ty }),
                            None => (0usize, unknown_ref(self.types)),
                        },
                        None => (0usize, unknown_ref(self.types)),
                    }
                } else {
                    (0usize, unknown_ref(self.types))
                };
                HirExpr {
                    kind: HirExprKind::Field {
                        base: Box::new(base),
                        field_index,
                    },
                    ty: field_ty,
                    span,
                }
            }
            ExprAst::Cast { expr, target } => {
                let lowered = self.lower_expr(*expr, span);
                let target = self.lower_type(target);
                HirExpr {
                    kind: HirExprKind::Cast {
                        expr: Box::new(lowered),
                        target,
                    },
                    ty: target,
                    span,
                }
            }
            // ─── v1.30 Threading (A3) — lowered to HIR ───
            ExprAst::Spawn { actor_name, args } => {
                let lowered_args: Vec<_> = args
                    .into_iter()
                    .map(|arg| self.lower_expr(arg, span))
                    .collect();
                HirExpr {
                    kind: HirExprKind::Spawn { actor_name, args: lowered_args },
                    ty: unit_ref(self.types),
                    span,
                }
            }
            ExprAst::Join { actor_name } => HirExpr {
                kind: HirExprKind::Join { actor_name },
                ty: unit_ref(self.types),
                span,
            },
            ExprAst::ChannelSend { channel_name, value } => {
                let lowered = self.lower_expr(*value, span);
                HirExpr {
                    kind: HirExprKind::ChannelSend { channel_name, value: Box::new(lowered) },
                    ty: unit_ref(self.types),
                    span,
                }
            }
            ExprAst::ChannelRecv { channel_name } => HirExpr {
                kind: HirExprKind::ChannelRecv { channel_name },
                ty: unit_ref(self.types),
                span,
            },
            // ─── v1.30 Phase 3: Backpressure + Scheduler (A4) ───
            ExprAst::ChannelTrySend { channel_name, value } => {
                let lowered = self.lower_expr(*value, span);
                HirExpr {
                    kind: HirExprKind::ChannelTrySend { channel_name, value: Box::new(lowered) },
                    ty: bool_ref(self.types),
                    span,
                }
            }
            ExprAst::ChannelTryRecv { channel_name } => HirExpr {
                kind: HirExprKind::ChannelTryRecv { channel_name },
                ty: i64_ref(self.types), // Option<T> represented as i64 (0=None, else=Some)
                span,
            },
        }
    }

    fn lower_type(&mut self, ty: TypeAst) -> TypeRef {
        let id = match ty {
            TypeAst::Named(name) => named_type_id(self.types, &name),
            TypeAst::Pointer(inner) => {
                let pointee = self.lower_type(*inner);
                self.types.intern(TypeKind::Pointer {
                    pointee: pointee.id,
                    mutability: Mutability::Immutable,
                })
            }
            TypeAst::Array { element, len } => {
                let elem = self.lower_type(*element);
                self.types.intern(TypeKind::Array {
                    element: elem.id,
                    len,
                })
            }
            TypeAst::Unit => self.types.primitive(PrimitiveType::Unit),
        };
        TypeRef { id }
    }

    fn push_lowering_error(&mut self, span: Span, message_ms: String, message_en: String) {
        self.diagnostics.push(Diagnostic {
            code: DiagnosticCode::ParserUnsupportedFeature,
            severity: Severity::Error,
            message_ms,
            message_en,
            primary_span: span,
            notes: Vec::new(),
        });
    }
}

fn lower_enum_payload(ctx: &mut LoweringContext<'_>, payload: EnumPayloadAst) -> Vec<TypeRef> {
    match payload {
        EnumPayloadAst::Unit => Vec::new(),
        EnumPayloadAst::Tuple(types) => types.into_iter().map(|ty| ctx.lower_type(ty)).collect(),
        EnumPayloadAst::Struct(fields) => fields
            .into_iter()
            .map(|field| ctx.lower_type(field.ty))
            .collect(),
    }
}

fn named_type_id(registry: &mut TypeRegistry, name: &str) -> TypeId {
    match name {
        "bool" | "Boolean" | "BooleanAst" => registry.primitive(PrimitiveType::Bool),
        "i8" | "I8" => registry.primitive(PrimitiveType::I8),
        "i16" | "I16" => registry.primitive(PrimitiveType::I16),
        "i32" | "I32" => registry.primitive(PrimitiveType::I32),
        "i64" | "I64" | "int" => registry.primitive(PrimitiveType::I64),
        "u8" | "U8" => registry.primitive(PrimitiveType::U8),
        "u16" | "U16" => registry.primitive(PrimitiveType::U16),
        "u32" | "U32" => registry.primitive(PrimitiveType::U32),
        "u64" | "U64" => registry.primitive(PrimitiveType::U64),
        "f32" => registry.primitive(PrimitiveType::F32),
        "f64" | "float" => registry.primitive(PrimitiveType::F64),
        "string" | "String" => registry.primitive(PrimitiveType::String),
        "unit" | "()" => registry.primitive(PrimitiveType::Unit),
        _ => {
            // User-defined struct, interned during the lowering pre-pass.
            let layout_id = registry.find_struct_by_name(name).map(|(id, _)| id);
            match layout_id {
                Some(lid) => registry.intern(crate::types::TypeKind::Struct(lid)),
                None => registry.unknown(),
            }
        }
    }
}

fn literal_type(registry: &TypeRegistry, literal: &LiteralAst) -> TypeRef {
    match literal {
        LiteralAst::Integer(_) => TypeRef {
            id: registry.primitive(PrimitiveType::I64),
        },
        LiteralAst::Boolean(_) => bool_ref(registry),
        LiteralAst::String(_) => TypeRef {
            id: registry.primitive(PrimitiveType::String),
        },
        LiteralAst::Unit => unit_ref(registry),
    }
}

fn binary_type(registry: &TypeRegistry, op: BinaryOpAst, left: TypeRef, right: TypeRef) -> TypeRef {
    match op {
        BinaryOpAst::Eq
        | BinaryOpAst::NotEq
        | BinaryOpAst::Lt
        | BinaryOpAst::Lte
        | BinaryOpAst::Gt
        | BinaryOpAst::Gte
        | BinaryOpAst::LogicalAnd
        | BinaryOpAst::LogicalOr => bool_ref(registry),
        _ if left == right => left,
        _ => right,
    }
}

fn bool_ref(registry: &TypeRegistry) -> TypeRef {
    TypeRef {
        id: registry.primitive(PrimitiveType::Bool),
    }
}
fn unit_ref(registry: &TypeRegistry) -> TypeRef {
    TypeRef {
        id: registry.primitive(PrimitiveType::Unit),
    }
}
fn i64_ref(registry: &TypeRegistry) -> TypeRef {
    TypeRef {
        id: registry.primitive(PrimitiveType::I64),
    }
}
fn unknown_ref(registry: &TypeRegistry) -> TypeRef {
    TypeRef {
        id: registry.unknown(),
    }
}

// Sprint 3: v1.21 AST → HIR AST conversion helpers

fn lower_type_ast(ty: ast::Type) -> TypeAst {
    match ty {
        ast::Type::I32 => TypeAst::Named("i32".to_string()),
        ast::Type::I64 => TypeAst::Named("i64".to_string()),
        ast::Type::U16 => TypeAst::Named("u16".to_string()),
        ast::Type::U32 => TypeAst::Named("u32".to_string()),
        ast::Type::I8 => TypeAst::Named("i8".to_string()),
        ast::Type::I16 => TypeAst::Named("i16".to_string()),
        ast::Type::U8 => TypeAst::Named("u8".to_string()),
        ast::Type::U64 => TypeAst::Named("u64".to_string()),
        ast::Type::F64 => TypeAst::Named("f64".to_string()),
        ast::Type::Bool => TypeAst::Named("bool".to_string()),
        ast::Type::Pointer(inner) => TypeAst::Pointer(Box::new(lower_type_ast(*inner))),
        ast::Type::String => TypeAst::Named("String".to_string()),
        ast::Type::Named(n) => TypeAst::Named(n),
        _ => TypeAst::Unit,
    }
}

/// Lower a `MATCH value { pat => body, ... }` into a nested if/else chain over
/// the (i64) value. Enum variants and integer literals become equality tests;
/// `_` becomes the final else. Ok/Err/Tuple patterns are not yet supported.
fn lower_match_to_if(value: ast::Expr, arms: Vec<ast::MatchArm>) -> StmtAst {
    let value_ast = lower_expr_ast(value);
    let mk_block = |body: Vec<ast::Stmt>| BlockAst {
        statements: body
            .into_iter()
            .map(|s| Spanned { node: lower_stmt_ast(s), span: Span::unknown() })
            .collect(),
    };
    let mut else_branch: Option<BlockAst> = None;
    let mut conditional: Vec<(ExprAst, BlockAst)> = Vec::new();
    for arm in arms {
        match arm.pattern {
            ast::MatchPattern::Wildcard => {
                else_branch = Some(mk_block(arm.body));
            }
            ast::MatchPattern::Identifier(name) => {
                conditional.push((
                    ExprAst::EnumVariant { enum_name: String::new(), variant: name },
                    mk_block(arm.body),
                ));
            }
            ast::MatchPattern::Literal(expr) => {
                conditional.push((lower_expr_ast(expr), mk_block(arm.body)));
            }
            _ => {}
        }
    }
    let mut acc: Option<BlockAst> = else_branch;
    for (pat, body) in conditional.into_iter().rev() {
        let cond = ExprAst::Binary {
            left: Box::new(value_ast.clone()),
            op: BinaryOpAst::Eq,
            right: Box::new(pat),
        };
        let if_stmt = StmtAst::If {
            condition: cond,
            then_branch: body,
            else_branch: acc.take(),
        };
        acc = Some(BlockAst {
            statements: vec![Spanned { node: if_stmt, span: Span::unknown() }],
        });
    }
    match acc {
        Some(mut block) => {
            if block.statements.len() == 1 {
                block.statements.pop().unwrap().node
            } else {
                StmtAst::UnsafeBlock(block)
            }
        }
        None => StmtAst::Expr(ExprAst::Literal(LiteralAst::Unit)),
    }
}

fn lower_stmt_ast(stmt: ast::Stmt) -> StmtAst {
    use ast::Stmt;
    match stmt {
        Stmt::Let { name, declared_type, value } => StmtAst::Let {
            name,
            ty: declared_type.map(lower_type_ast),
            value: Some(lower_expr_ast(value)),
        },
        Stmt::Print { value } => StmtAst::Expr(ExprAst::Call {
            callee: Box::new(ExprAst::Variable("print".to_string())),
            args: vec![lower_expr_ast(value)],
        }),
        Stmt::Return { value } => StmtAst::Return(Some(lower_expr_ast(value))),
        Stmt::ExprStmt { value } => StmtAst::Expr(lower_expr_ast(value)),
        Stmt::Assign { target, value } => StmtAst::Assign {
            target: lower_expr_ast(target),
            value: lower_expr_ast(value),
        },
        Stmt::If { condition, then_branch, else_branch } => StmtAst::If {
            condition: lower_expr_ast(condition),
            then_branch: BlockAst {
                statements: then_branch.into_iter().map(|s| Spanned {
                    node: lower_stmt_ast(s),
                    span: Span::unknown(),
                }).collect(),
            },
            else_branch: Some(BlockAst {
                statements: else_branch.into_iter().map(|s| Spanned {
                    node: lower_stmt_ast(s),
                    span: Span::unknown(),
                }).collect(),
            }),
        },
        Stmt::While { condition, body } => StmtAst::While {
            condition: lower_expr_ast(condition),
            body: BlockAst {
                statements: body.into_iter().map(|s| Spanned {
                    node: lower_stmt_ast(s),
                    span: Span::unknown(),
                }).collect(),
            },
        },
        Stmt::Loop { body } => StmtAst::Loop {
            body: BlockAst {
                statements: body.into_iter().map(|s| Spanned {
                    node: lower_stmt_ast(s),
                    span: Span::unknown(),
                }).collect(),
            },
        },
        Stmt::Break => StmtAst::Break,
        Stmt::Continue => StmtAst::Continue,
        Stmt::UnsafeBlock { body } => StmtAst::UnsafeBlock(BlockAst {
            statements: body.into_iter().map(|s| Spanned {
                node: lower_stmt_ast(s),
                span: Span::unknown(),
            }).collect(),
        }),
        // v1.44 G12: hardware zone preserved; codegen emits its memory ops as
        // volatile MMIO (see emit_hardware_zone). Body is lowered like a block.
        Stmt::HardwareZone { body } => StmtAst::HardwareZone(BlockAst {
            statements: body.into_iter().map(|s| Spanned {
                node: lower_stmt_ast(s),
                span: Span::unknown(),
            }).collect(),
        }),
        Stmt::Use { .. } | Stmt::HardwareDecl { .. } => {
            // Use imports + hardware register decls are not yet HIR-lowered.
            StmtAst::Expr(ExprAst::Literal(LiteralAst::Unit))
        }
        Stmt::Function { .. } | Stmt::StructDecl { .. } | Stmt::EnumDecl { .. } | Stmt::ExternBlock { .. } => {
            // These should have been extracted at the item level, not statements
            StmtAst::Expr(ExprAst::Literal(LiteralAst::Unit))
        }
        Stmt::Match { value, arms } => lower_match_to_if(value, arms),
        _ => StmtAst::Expr(ExprAst::Literal(LiteralAst::Unit)),
    }
}

fn lower_expr_ast(expr: ast::Expr) -> ExprAst {
    match expr {
        ast::Expr::Integer(v) => ExprAst::Literal(LiteralAst::Integer(v)),
        ast::Expr::Boolean(v) => ExprAst::Literal(LiteralAst::Boolean(v)),
        ast::Expr::StringLiteral(s) => ExprAst::Literal(LiteralAst::String(s)),
        ast::Expr::Variable(name) => ExprAst::Variable(name),
        ast::Expr::AddressOfLiteral(v) => ExprAst::Literal(LiteralAst::Integer(v)),
        ast::Expr::Call { callee, args } => ExprAst::Call {
            callee: Box::new(lower_expr_ast(*callee)),
            args: args.into_iter().map(lower_expr_ast).collect(),
        },
        ast::Expr::Binary { left, op, right } => ExprAst::Binary {
            left: Box::new(lower_expr_ast(*left)),
            op: lower_binary_op(op),
            right: Box::new(lower_expr_ast(*right)),
        },
        ast::Expr::Grouped(inner) => lower_expr_ast(*inner),
        // ─── v1.30 Threading (A3) — lowered to ExprAst ───
        ast::Expr::Spawn { actor_name, args } => ExprAst::Spawn {
            actor_name,
            args: args.into_iter().map(lower_expr_ast).collect(),
        },
        ast::Expr::Join { actor_name } => ExprAst::Join { actor_name },
        ast::Expr::Send { channel_name, value } => ExprAst::ChannelSend {
            channel_name,
            value: Box::new(lower_expr_ast(*value)),
        },
        ast::Expr::Recv { channel_name } => ExprAst::ChannelRecv { channel_name },
        // ─── v1.30 Phase 3: Backpressure + Scheduler (A4) ───
        ast::Expr::TrySend { channel_name, value } => ExprAst::ChannelTrySend {
            channel_name,
            value: Box::new(lower_expr_ast(*value)),
        },
        ast::Expr::TryRecv { channel_name } => ExprAst::ChannelTryRecv { channel_name },
        ast::Expr::Yield => ExprAst::Variable("yield".to_string()), // marker for HIR lowering
        ast::Expr::Sleep { duration_ms } => {
            let dur = lower_expr_ast(*duration_ms);
            ExprAst::Call {
                callee: Box::new(ExprAst::Variable("logicodex_sleep".to_string())),
                args: vec![dur],
            }
        }
        ast::Expr::TimeoutRecv { channel_name, timeout_ms } => {
            let to = lower_expr_ast(*timeout_ms);
            ExprAst::Call {
                callee: Box::new(ExprAst::Variable("logicodex_timeout_recv".to_string())),
                args: vec![ExprAst::Literal(LiteralAst::String(channel_name)), to],
            }
        }
        ast::Expr::FieldAccess { base, field } => ExprAst::Field {
            base: Box::new(lower_expr_ast(*base)),
            field,
        },
        ast::Expr::EnumVariant { enum_name, variant } => ExprAst::EnumVariant { enum_name, variant },
        _ => ExprAst::Literal(LiteralAst::Unit),
    }
}

fn lower_binary_op(op: ast::BinaryOp) -> BinaryOpAst {
    match op {
        ast::BinaryOp::Add => BinaryOpAst::Add,
        ast::BinaryOp::Subtract => BinaryOpAst::Sub,
        ast::BinaryOp::Multiply => BinaryOpAst::Mul,
        ast::BinaryOp::Divide => BinaryOpAst::Div,
        ast::BinaryOp::Greater => BinaryOpAst::Gt,
        ast::BinaryOp::GreaterEqual => BinaryOpAst::Gte,
        ast::BinaryOp::Less => BinaryOpAst::Lt,
        ast::BinaryOp::LessEqual => BinaryOpAst::Lte,
        ast::BinaryOp::Equal => BinaryOpAst::Eq,
        ast::BinaryOp::NotEqual => BinaryOpAst::NotEq,
        ast::BinaryOp::And => BinaryOpAst::LogicalAnd,
        ast::BinaryOp::Or => BinaryOpAst::LogicalOr,
        ast::BinaryOp::BitAnd => BinaryOpAst::BitAnd,
        ast::BinaryOp::BitOr => BinaryOpAst::BitOr,
        ast::BinaryOp::ShiftLeft => BinaryOpAst::ShiftLeft,
        ast::BinaryOp::ShiftRight => BinaryOpAst::ShiftRight,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spanned<T>(node: T) -> Spanned<T> {
        Spanned {
            node,
            span: Span::unknown(),
        }
    }

    #[test]
    fn lowers_function_params_let_and_return() {
        let mut symbols = SymbolTable::default();
        let mut types = TypeRegistry::new();
        let mut ctx = LoweringContext {
            symbols: &mut symbols,
            types: &mut types,
            diagnostics: Vec::new(),
        };
        let module = ModuleAst {
            items: vec![spanned(ItemAst::Function(FunctionAst {
                name: "main".to_string(),
                params: vec![ParamAst {
                    name: "x".to_string(),
                    ty: TypeAst::Named("i64".to_string()),
                }],
                return_type: Some(TypeAst::Named("i64".to_string())),
                body: BlockAst {
                    statements: vec![
                        spanned(StmtAst::Let {
                            name: "y".to_string(),
                            ty: None,
                            value: Some(ExprAst::Variable("x".to_string())),
                        }),
                        spanned(StmtAst::Return(Some(ExprAst::Variable("y".to_string())))),
                    ],
                },
                is_unsafe: false,
            }))],
        };

        let hir = ctx.lower_module(module).expect("lowering must succeed");
        let ids = types.primitive_ids();
        match &hir.items[0].node {
            HirItem::Function(function) => {
                assert_eq!(function.params[0].ty.id, ids.i64_);
                assert_eq!(function.body.statements.len(), 2);
                assert_eq!(function.safety, SafetyContext::Safe);
            }
            other => panic!("unexpected item: {other:?}"),
        }
    }

    #[test]
    fn reports_unknown_variable_with_bilingual_diagnostic() {
        let mut symbols = SymbolTable::default();
        let mut types = TypeRegistry::new();
        let mut ctx = LoweringContext {
            symbols: &mut symbols,
            types: &mut types,
            diagnostics: Vec::new(),
        };
        let module = ModuleAst {
            items: vec![spanned(ItemAst::Function(FunctionAst {
                name: "main".to_string(),
                params: Vec::new(),
                return_type: Some(TypeAst::Unit),
                body: BlockAst {
                    statements: vec![spanned(StmtAst::Expr(ExprAst::Variable(
                        "missing".to_string(),
                    )))],
                },
                is_unsafe: false,
            }))],
        };

        let diagnostics = ctx
            .lower_module(module)
            .expect_err("unknown variable should fail");
        assert_eq!(
            diagnostics[0].code,
            DiagnosticCode::ParserUnsupportedFeature
        );
        assert!(diagnostics[0].message_ms.contains("Ralat:"));
        assert!(diagnostics[0].message_en.contains("Error:"));
    }

    #[test]
    fn lowers_extern_function_to_callable() {
        let mut symbols = SymbolTable::default();
        let mut types = TypeRegistry::new();
        let mut ctx = LoweringContext {
            symbols: &mut symbols,
            types: &mut types,
            diagnostics: Vec::new(),
        };
        let module = ModuleAst {
            items: vec![spanned(ItemAst::ExternBlock(ExternBlockAst {
                abi: crate::ffi::CallingConvention::C,
                functions: vec![ExternFnAst {
                    name: "puts".to_string(),
                    params: vec![ParamAst {
                        name: "s".to_string(),
                        ty: TypeAst::Named("string".to_string()),
                    }],
                    return_type: TypeAst::Named("i32".to_string()),
                    is_variadic: false,
                }],
            }))],
        };
        let hir = ctx
            .lower_module(module)
            .expect("extern lowering should succeed");
        assert!(matches!(hir.items[0].node, HirItem::ExternFunction(_)));
    }
}
