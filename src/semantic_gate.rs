// =========================================================================
// Logicodex v1.38 — Semantic Gatekeeper (I1: Activated)
//
// Final semantic validation pass before codegen.
// Checks: break/continue in loops, unsafe block correctness,
// FFI call safety, return path validity.
//
// Called after semantic analysis and before LLVM codegen.
// =========================================================================

use crate::ffi::{CallableRegistry, FfiGatekeeper, SafetyContext};
use crate::hir::{
    BinaryOpAst, HirBlock, HirExpr, HirExprKind, HirItem, HirModule, HirStmt, LiteralAst,
    SymbolTable,
};
use crate::span::{Diagnostic, DiagnosticCode, Severity, Span, Spanned};
use crate::types::TypeRegistry;

pub struct SemanticContext {
    pub types: TypeRegistry,
    pub symbols: SymbolTable,
    pub callables: CallableRegistry,
    pub diagnostics: Vec<Diagnostic>,
    pub loop_depth: u32,
    pub safety_context: SafetyContext,
    /// Expected return type for the function currently being checked.
    pub current_return_type: Option<crate::types::TypeId>,
    /// FFI capability policy (zero-trust, default deny). Seeded with the runtime
    /// builtins; lod (or manual ffi.allow) adds external symbols. The semantic
    /// gate passes this to the FfiGatekeeper so denied externs fail at check.
    pub policy: crate::ffi::CapabilityPolicy,
    /// Names of user-declared `extern "C"` functions (collected from the HIR
    /// module's ExternFunction items at check start). These live in the
    /// SymbolTable, not the FFI CallableRegistry, so the capability gate checks
    /// them here directly — a Call to one of these names is a foreign call and
    /// must satisfy the CapabilityPolicy.
    pub extern_symbols: std::collections::HashSet<String>,
}

impl SemanticContext {
    pub fn check_module(&mut self, module: &HirModule) -> Result<(), Vec<Diagnostic>> {
        // Duplicate function definitions (ported from the legacy analyzer).
        let mut seen_fns = std::collections::HashSet::new();
        for item in &module.items {
            if let HirItem::Function(f) = &item.node {
                if !seen_fns.insert(f.name.clone()) {
                    self.push_error(
                        DiagnosticCode::DuplicateDefinition,
                        item.span,
                        format!("Ralat: Fungsi '{}' ditakrif lebih daripada sekali", f.name),
                        format!("Error: Function '{}' is defined more than once", f.name),
                    );
                }
            }
        }
        // Collect user-declared extern symbol names so the capability gate can
        // recognise foreign calls (these are in the SymbolTable, not the FFI
        // CallableRegistry which only holds compiler-registered FFI like Raylib).
        for item in &module.items {
            if let HirItem::ExternFunction(ext) = &item.node {
                if let Some(name) = self.symbols.callable_name(ext.callable) {
                    self.extern_symbols.insert(name.to_string());
                }
            }
        }
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
        match &item.node {
            HirItem::Function(function) => {
                let previous_safety = self.safety_context;
                let previous_return_type = self.current_return_type;
                self.safety_context = function.safety;
                self.current_return_type = Some(function.return_type.id);
                self.check_block(&function.body);
                self.current_return_type = previous_return_type;
                self.safety_context = previous_safety;
            }
            HirItem::Struct(_) | HirItem::Enum(_) | HirItem::ExternFunction(_) => {}
        }
    }

    fn check_block(&mut self, block: &HirBlock) {
        for stmt in &block.statements {
            self.check_statement(stmt);
        }
    }

    fn check_statement(&mut self, stmt: &Spanned<HirStmt>) {
        match &stmt.node {
            HirStmt::Let { ty, value, .. } => {
                if let Some(value) = value {
                    self.check_expression(value);
                    if !self.types_compatible(ty.id, value.ty.id) {
                        self.push_error(
                            DiagnosticCode::TypeMismatch,
                            stmt.span,
                            format!(
                                "Ralat: Jenis ikatan tidak sepadan: dijangka {}, diterima {}",
                                self.type_label(ty.id),
                                self.type_label(value.ty.id)
                            ),
                            format!(
                                "Error: Binding type mismatch: expected {}, got {}",
                                self.type_label(ty.id),
                                self.type_label(value.ty.id)
                            ),
                        );
                    }
                }
            }
            HirStmt::Assign { target, value } => {
                self.check_expression(target);
                self.check_expression(value);
                if !self.types_compatible(target.ty.id, value.ty.id) {
                    self.push_error(
                        DiagnosticCode::TypeMismatch,
                        stmt.span,
                        format!(
                            "Ralat: Jenis tugasan tidak sepadan: dijangka {}, diterima {}",
                            self.type_label(target.ty.id),
                            self.type_label(value.ty.id)
                        ),
                        format!(
                            "Error: Assignment type mismatch: expected {}, got {}",
                            self.type_label(target.ty.id),
                            self.type_label(value.ty.id)
                        ),
                    );
                }
            }
            HirStmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.check_expression(condition);
                self.check_condition_type(condition, stmt.span, "if");
                self.check_block(then_branch);
                if let Some(else_branch) = else_branch {
                    self.check_block(else_branch);
                }
            }
            HirStmt::While { condition, body } => {
                self.check_expression(condition);
                self.check_condition_type(condition, stmt.span, "while");
                self.loop_depth += 1;
                self.check_block(body);
                self.loop_depth -= 1;
            }
            HirStmt::Loop { body } => {
                self.loop_depth += 1;
                self.check_block(body);
                self.loop_depth -= 1;
            }
            HirStmt::Break { .. } | HirStmt::Continue { .. } if self.loop_depth == 0 => {
                self.push_error(
                    DiagnosticCode::UnsafeBoundaryViolation,
                    stmt.span,
                    "Ralat: Kawalan gelung digunakan di luar gelung".to_string(),
                    "Error: Loop control used outside a loop".to_string(),
                );
            }
            HirStmt::Break { .. } | HirStmt::Continue { .. } => {}
            HirStmt::UnsafeBlock(block) => {
                let previous = self.safety_context;
                self.safety_context = SafetyContext::Unsafe;
                self.check_block(block);
                self.safety_context = previous;
            }
            HirStmt::HardwareZone(block) => {
                // MMIO zone: inherently unsafe context (volatile hardware access).
                let previous = self.safety_context;
                self.safety_context = SafetyContext::Unsafe;
                self.check_block(block);
                self.safety_context = previous;
            }
            HirStmt::HardwareDecl { .. } => {
                // Hardware register declaration: no sub-expressions to check.
            }
            HirStmt::Expr(expr) => self.check_expression(expr),
            HirStmt::Return(expr) => match expr {
                Some(expr) => {
                    self.check_expression(expr);
                    if let Some(expected) = self.current_return_type {
                        if !self.types_compatible(expected, expr.ty.id) {
                            self.push_error(
                                DiagnosticCode::TypeMismatch,
                                stmt.span,
                                format!(
                                    "Ralat: Jenis pulangan tidak sepadan: dijangka {}, diterima {}",
                                    self.type_label(expected),
                                    self.type_label(expr.ty.id)
                                ),
                                format!(
                                    "Error: Return type mismatch: expected {}, got {}",
                                    self.type_label(expected),
                                    self.type_label(expr.ty.id)
                                ),
                            );
                        }
                    }
                }
                None => {
                    if let Some(expected) = self.current_return_type {
                        if !self.is_unit_type(expected) && !self.is_unknown_type(expected) {
                            self.push_error(
                                DiagnosticCode::TypeMismatch,
                                stmt.span,
                                format!(
                                    "Ralat: Pulangan memerlukan nilai jenis {}",
                                    self.type_label(expected)
                                ),
                                format!(
                                    "Error: Return requires a value of type {}",
                                    self.type_label(expected)
                                ),
                            );
                        }
                    }
                }
            },
        }
    }

    fn check_expression(&mut self, expr: &HirExpr) {
        match &expr.kind {
            HirExprKind::Literal(_) | HirExprKind::Local(_) | HirExprKind::Global(_) => {}
            HirExprKind::Binary { left, op, right } => {
                self.check_expression(left);
                self.check_expression(right);
                // Division by a literal zero (ported from the legacy analyzer).
                if matches!(op, BinaryOpAst::Div) {
                    if let HirExprKind::Literal(LiteralAst::Integer(0)) = &right.kind {
                        self.push_error(
                            DiagnosticCode::DivisionByZero,
                            expr.span,
                            "Ralat: Pembahagian dengan sifar".to_string(),
                            "Error: Division by zero".to_string(),
                        );
                    }
                }
                // The compiler uses a uniform i64 integer model, so integer operands of
                // differing declared widths (e.g. I32 vs the default-I64 of an
                // integer literal) are compatible. Only flag genuinely mismatched
                // categories (e.g. int vs float).
                let either_unknown =
                    self.is_unknown_type(left.ty.id) || self.is_unknown_type(right.ty.id);
                if left.ty.id != right.ty.id
                    && !either_unknown
                    && !(self.is_integer_type(left.ty.id) && self.is_integer_type(right.ty.id))
                {
                    self.push_error(
                        DiagnosticCode::TypeMismatch,
                        expr.span,
                        "Ralat: Jenis operand binari tidak sepadan".to_string(),
                        "Error: Binary operand types do not match".to_string(),
                    );
                }
            }
            HirExprKind::Unary { expr, .. }
            | HirExprKind::Field { base: expr, .. }
            | HirExprKind::Cast { expr, .. } => {
                self.check_expression(expr);
            }
            HirExprKind::Call { callee, args } => {
                for arg in args {
                    self.check_expression(arg);
                }

                let expected_params = self.symbols.callable_params(*callee).map(|p| p.to_vec());
                if let Some(expected_params) = expected_params {
                    let name = self
                        .symbols
                        .callable_name(*callee)
                        .unwrap_or("<unknown>")
                        .to_string();

                    if expected_params.len() != args.len() {
                        self.push_error(
                            DiagnosticCode::TypeMismatch,
                            expr.span,
                            format!(
                                "Ralat: Bilangan argumen fungsi `{name}` tidak sepadan: dijangka {}, diterima {}",
                                expected_params.len(),
                                args.len()
                            ),
                            format!(
                                "Error: Function `{name}` argument count mismatch: expected {}, got {}",
                                expected_params.len(),
                                args.len()
                            ),
                        );
                    } else {
                        for (idx, (expected, actual)) in
                            expected_params.iter().copied().zip(args.iter()).enumerate()
                        {
                            if !self.types_compatible(expected, actual.ty.id) {
                                self.push_error(
                                    DiagnosticCode::TypeMismatch,
                                    actual.span,
                                    format!(
                                        "Ralat: Jenis argumen fungsi `{name}` tidak sepadan pada argumen {}: dijangka {}, diterima {}",
                                        idx + 1,
                                        self.type_label(expected),
                                        self.type_label(actual.ty.id)
                                    ),
                                    format!(
                                        "Error: Function `{name}` argument {} type mismatch: expected {}, got {}",
                                        idx + 1,
                                        self.type_label(expected),
                                        self.type_label(actual.ty.id)
                                    ),
                                );
                            }
                        }
                    }
                }

                // Resolve the callee by NAME against the FFI registry. The HIR
                // Call.callee is a SymbolTable id (builtins/user fns) whose id-space
                // is independent of the FFI CallableRegistry, so a raw .get(id) can
                // alias an unrelated extern (e.g. print -> InitWindow). Name-based
                // lookup only validates genuine FFI functions; builtins resolve to
                // None here and are correctly skipped.
                let ffi_sig = self
                    .symbols
                    .callable_name(*callee)
                    .and_then(|name| self.callables.find_by_name(name));
                if let Some((_, signature)) = ffi_sig {
                    let gate = FfiGatekeeper {
                        types: &self.types,
                        callables: Some(&self.callables),
                        policy: Some(&self.policy),
                    };
                    let result =
                        gate.validate_call(signature, args, self.safety_context, expr.span);
                    if let Err(diagnostic) = result {
                        self.diagnostics.push(diagnostic);
                    }
                } else if let Some(name) = self.symbols.callable_name(*callee) {
                    // User-declared extern (in the SymbolTable, not the FFI
                    // registry). It is a foreign call, so it must pass the
                    // capability gate: default-deny unless explicitly allowed.
                    // (unsafe/arity/type for these are handled elsewhere; here we
                    // enforce ONLY the capability layer.)
                    if self.extern_symbols.contains(name) && !self.policy.is_symbol_allowed(name) {
                        self.diagnostics.push(crate::span::Diagnostic {
                            code: DiagnosticCode::FfiBoundaryViolation,
                            severity: Severity::Error,
                            message_ms: format!(
                                "Ralat: panggilan extern '{name}' ditolak oleh polisi \
                                 keupayaan FFI. Isytiharkannya dalam ffi.allow sebelum guna."
                            ),
                            message_en: format!(
                                "Error: extern call '{name}' denied by FFI capability \
                                 policy. Declare it in ffi.allow before use."
                            ),
                            primary_span: expr.span,
                            notes: Vec::new(),
                        });
                    }
                }
            }
            _ => {}
        }
    }

    fn check_condition_type(&mut self, condition: &HirExpr, span: Span, context: &str) {
        if self.is_condition_type(condition.ty.id) {
            return;
        }

        let label = self.type_label(condition.ty.id);
        self.push_error(
            DiagnosticCode::TypeMismatch,
            span,
            format!(
                "Ralat: Jenis syarat {} tidak sepadan: dijangka Bool, diterima {}",
                context, label
            ),
            format!(
                "Error: {} condition type mismatch: expected Bool, got {}",
                context, label
            ),
        );
    }

    fn is_condition_type(&self, id: crate::types::TypeId) -> bool {
        self.is_bool_type(id) || self.is_unknown_type(id)
    }

    fn is_bool_type(&self, id: crate::types::TypeId) -> bool {
        matches!(
            self.types.resolve(id),
            crate::types::TypeKind::Primitive(crate::types::PrimitiveType::Bool)
        )
    }

    /// True if a TypeId resolves to any integer primitive (I8..U64). Used to
    /// treat integer operands of differing widths as compatible under the
    /// uniform-i64 codegen model.
    fn is_integer_type(&self, id: crate::types::TypeId) -> bool {
        use crate::types::{PrimitiveType, TypeKind};
        matches!(
            self.types.resolve(id),
            TypeKind::Primitive(
                PrimitiveType::I8
                    | PrimitiveType::I16
                    | PrimitiveType::I32
                    | PrimitiveType::I64
                    | PrimitiveType::U8
                    | PrimitiveType::U16
                    | PrimitiveType::U32
                    | PrimitiveType::U64
            )
        )
    }

    /// True if a TypeId is the not-yet-inferred Unknown type. Call results carry
    /// Unknown today (return-type inference is not wired), so treat it as a
    /// wildcard rather than a hard binary-operand mismatch.
    fn is_unknown_type(&self, id: crate::types::TypeId) -> bool {
        matches!(self.types.resolve(id), crate::types::TypeKind::Unknown)
    }

    fn types_compatible(
        &self,
        expected: crate::types::TypeId,
        actual: crate::types::TypeId,
    ) -> bool {
        self.types.is_equivalent(expected, actual)
            || self.is_unknown_type(expected)
            || self.is_unknown_type(actual)
            || (self.is_integer_type(expected) && self.is_integer_type(actual))
            || self.is_transitional_scalar_abi_compatible(expected, actual)
    }

    fn is_transitional_scalar_abi_compatible(
        &self,
        expected: crate::types::TypeId,
        actual: crate::types::TypeId,
    ) -> bool {
        (self.is_option_result_i64_scalar_abi(expected) && self.is_i64_type(actual))
            || (self.is_option_result_i64_scalar_abi(actual) && self.is_i64_type(expected))
    }

    fn is_option_result_i64_scalar_abi(&self, id: crate::types::TypeId) -> bool {
        match self.types.resolve(id) {
            crate::types::TypeKind::Option { some } => self.is_i64_type(*some),
            crate::types::TypeKind::Result { ok, err } => {
                self.is_i64_type(*ok) && self.is_i64_type(*err)
            }
            _ => false,
        }
    }

    fn is_i64_type(&self, id: crate::types::TypeId) -> bool {
        matches!(
            self.types.resolve(id),
            crate::types::TypeKind::Primitive(crate::types::PrimitiveType::I64)
        )
    }

    fn is_unit_type(&self, id: crate::types::TypeId) -> bool {
        matches!(
            self.types.resolve(id),
            crate::types::TypeKind::Primitive(crate::types::PrimitiveType::Unit)
        )
    }

    fn type_label(&self, id: crate::types::TypeId) -> String {
        format!("{:?}", self.types.resolve(id))
    }

    fn push_error(
        &mut self,
        code: DiagnosticCode,
        span: Span,
        message_ms: String,
        message_en: String,
    ) {
        self.diagnostics.push(Diagnostic {
            code,
            severity: Severity::Error,
            message_ms,
            message_en,
            primary_span: span,
            notes: Vec::new(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ffi::{CallableSafety, CallingConvention};
    use crate::hir::{HirExprKind, HirFunction, LiteralAst, SymbolId};
    use crate::types::TypeRef;

    fn spanned<T>(node: T) -> Spanned<T> {
        Spanned {
            node,
            span: Span::unknown(),
        }
    }

    fn expr_i64(types: &TypeRegistry, value: i64) -> HirExpr {
        HirExpr {
            kind: HirExprKind::Literal(LiteralAst::Integer(value)),
            ty: TypeRef {
                id: types.primitive_ids().i64_,
            },
            span: Span::unknown(),
        }
    }

    fn base_context() -> SemanticContext {
        SemanticContext {
            types: TypeRegistry::new(),
            symbols: SymbolTable::default(),
            callables: CallableRegistry::default(),
            diagnostics: Vec::new(),
            loop_depth: 0,
            safety_context: SafetyContext::Safe,
            current_return_type: None,
            policy: crate::ffi::CapabilityPolicy::with_runtime_builtins(),
            extern_symbols: std::collections::HashSet::new(),
        }
    }

    fn expr_bool(types: &TypeRegistry, value: bool) -> HirExpr {
        HirExpr {
            kind: HirExprKind::Literal(LiteralAst::Boolean(value)),
            ty: TypeRef {
                id: types.primitive_ids().bool_,
            },
            span: Span::unknown(),
        }
    }

    #[test]
    fn rejects_i64_if_condition_in_hir() {
        let mut ctx = base_context();
        let module = HirModule {
            items: vec![spanned(HirItem::Function(HirFunction {
                name: "bad_if".to_string(),
                symbol: SymbolId(0),
                params: vec![],
                return_type: TypeRef {
                    id: ctx.types.primitive_ids().unit,
                },
                body: HirBlock {
                    statements: vec![spanned(HirStmt::If {
                        condition: expr_i64(&ctx.types, 1),
                        then_branch: HirBlock { statements: vec![] },
                        else_branch: None,
                    })],
                },
                safety: SafetyContext::Safe,
            }))],
        };

        let diagnostics = ctx
            .check_module(&module)
            .expect_err("i64 if condition should fail");

        let rendered = diagnostics
            .iter()
            .map(|d| format!("{} / {}", d.message_ms, d.message_en))
            .collect::<Vec<_>>()
            .join("\n");

        assert!(
            rendered.contains("if condition type mismatch") || rendered.contains("Jenis syarat if"),
            "expected if condition type diagnostic, got:\n{}",
            rendered
        );
    }

    #[test]
    fn rejects_i64_while_condition_in_hir() {
        let mut ctx = base_context();
        let module = HirModule {
            items: vec![spanned(HirItem::Function(HirFunction {
                name: "bad_while".to_string(),
                symbol: SymbolId(0),
                params: vec![],
                return_type: TypeRef {
                    id: ctx.types.primitive_ids().unit,
                },
                body: HirBlock {
                    statements: vec![spanned(HirStmt::While {
                        condition: expr_i64(&ctx.types, 1),
                        body: HirBlock { statements: vec![] },
                    })],
                },
                safety: SafetyContext::Safe,
            }))],
        };

        let diagnostics = ctx
            .check_module(&module)
            .expect_err("i64 while condition should fail");

        let rendered = diagnostics
            .iter()
            .map(|d| format!("{} / {}", d.message_ms, d.message_en))
            .collect::<Vec<_>>()
            .join("\n");

        assert!(
            rendered.contains("while condition type mismatch")
                || rendered.contains("Jenis syarat while"),
            "expected while condition type diagnostic, got:\n{}",
            rendered
        );
    }

    #[test]
    fn accepts_bool_if_and_while_conditions_in_hir() {
        let mut ctx = base_context();
        let module = HirModule {
            items: vec![spanned(HirItem::Function(HirFunction {
                name: "good_conditions".to_string(),
                symbol: SymbolId(0),
                params: vec![],
                return_type: TypeRef {
                    id: ctx.types.primitive_ids().unit,
                },
                body: HirBlock {
                    statements: vec![
                        spanned(HirStmt::If {
                            condition: expr_bool(&ctx.types, true),
                            then_branch: HirBlock { statements: vec![] },
                            else_branch: None,
                        }),
                        spanned(HirStmt::While {
                            condition: expr_bool(&ctx.types, false),
                            body: HirBlock { statements: vec![] },
                        }),
                    ],
                },
                safety: SafetyContext::Safe,
            }))],
        };

        ctx.check_module(&module)
            .expect("bool if/while conditions should pass");
    }

    #[test]
    fn rejects_bare_return_in_non_unit_hir_function() {
        let mut ctx = base_context();
        let module = HirModule {
            items: vec![spanned(HirItem::Function(HirFunction {
                name: "bad".to_string(),
                symbol: SymbolId(0),
                params: vec![],
                return_type: TypeRef {
                    id: ctx.types.primitive_ids().i64_,
                },
                body: HirBlock {
                    statements: vec![spanned(HirStmt::Return(None))],
                },
                safety: SafetyContext::Safe,
            }))],
        };

        let diagnostics = ctx
            .check_module(&module)
            .expect_err("bare return in non-Unit function should fail");

        let rendered = diagnostics
            .iter()
            .map(|d| format!("{} / {}", d.message_ms, d.message_en))
            .collect::<Vec<_>>()
            .join("\n");

        assert!(
            rendered.contains("Return requires a value")
                || rendered.contains("Pulangan memerlukan nilai"),
            "expected missing return value diagnostic, got:\n{rendered}"
        );
    }

    #[test]
    fn rejects_break_outside_loop() {
        let mut ctx = base_context();
        let module = HirModule {
            items: vec![spanned(HirItem::Function(HirFunction {
                name: "test".to_string(),
                symbol: SymbolId(0),
                params: Vec::new(),
                return_type: TypeRef {
                    id: ctx.types.primitive_ids().unit,
                },
                body: HirBlock {
                    statements: vec![spanned(HirStmt::Break { target_depth: 0 })],
                },
                safety: SafetyContext::Safe,
            }))],
        };

        let diagnostics = ctx
            .check_module(&module)
            .expect_err("break outside loop must fail");
        assert_eq!(diagnostics[0].code, DiagnosticCode::UnsafeBoundaryViolation);
    }

    #[test]
    fn allows_break_inside_loop() {
        let mut ctx = base_context();
        let module = HirModule {
            items: vec![spanned(HirItem::Function(HirFunction {
                name: "test".to_string(),
                symbol: SymbolId(0),
                params: Vec::new(),
                return_type: TypeRef {
                    id: ctx.types.primitive_ids().unit,
                },
                body: HirBlock {
                    statements: vec![spanned(HirStmt::Loop {
                        body: HirBlock {
                            statements: vec![spanned(HirStmt::Break { target_depth: 0 })],
                        },
                    })],
                },
                safety: SafetyContext::Safe,
            }))],
        };

        assert!(ctx.check_module(&module).is_ok());
    }

    #[test]
    fn ffi_call_requires_unsafe_block() {
        let mut ctx = base_context();
        let ids = ctx.types.primitive_ids();
        let _callable = ctx.callables.register(crate::ffi::CallableSignature {
            name: "puts".to_string(),
            params: vec![ids.i64_],
            return_type: ids.i32_,
            abi: CallingConvention::C,
            safety: CallableSafety::UnsafeRequired,
            is_extern: true,
            is_variadic: false,
        });
        // The semantic gate resolves FFI callees by NAME via the symbol table,
        // so the call must reference a symbol id whose name matches the registry.
        let callee = ctx.symbols.define_callable("puts");
        let call = HirExpr {
            kind: HirExprKind::Call {
                callee,
                args: vec![expr_i64(&ctx.types, 1)],
            },
            ty: TypeRef { id: ids.i32_ },
            span: Span::unknown(),
        };
        let module = HirModule {
            items: vec![spanned(HirItem::Function(HirFunction {
                name: "test".to_string(),
                symbol: SymbolId(0),
                params: Vec::new(),
                return_type: TypeRef { id: ids.unit },
                body: HirBlock {
                    statements: vec![spanned(HirStmt::Expr(call))],
                },
                safety: SafetyContext::Safe,
            }))],
        };

        let diagnostics = ctx
            .check_module(&module)
            .expect_err("unsafe ffi call must fail");
        assert_eq!(diagnostics[0].code, DiagnosticCode::FfiBoundaryViolation);
    }

    #[test]
    fn ffi_call_inside_unsafe_block_succeeds() {
        let mut ctx = base_context();
        let ids = ctx.types.primitive_ids();
        let callable = ctx.callables.register(crate::ffi::CallableSignature {
            name: "puts".to_string(),
            params: vec![ids.i64_],
            return_type: ids.i32_,
            abi: CallingConvention::C,
            safety: CallableSafety::UnsafeRequired,
            is_extern: true,
            is_variadic: false,
        });
        let call = HirExpr {
            kind: HirExprKind::Call {
                callee: callable,
                args: vec![expr_i64(&ctx.types, 1)],
            },
            ty: TypeRef { id: ids.i32_ },
            span: Span::unknown(),
        };
        let module = HirModule {
            items: vec![spanned(HirItem::Function(HirFunction {
                name: "test".to_string(),
                symbol: SymbolId(0),
                params: Vec::new(),
                return_type: TypeRef { id: ids.unit },
                body: HirBlock {
                    statements: vec![spanned(HirStmt::UnsafeBlock(HirBlock {
                        statements: vec![spanned(HirStmt::Expr(call))],
                    }))],
                },
                safety: SafetyContext::Safe,
            }))],
        };

        assert!(ctx.check_module(&module).is_ok());
    }
}

// =========================================================================
// v1.38 I1: Public API for semantic gatekeeper validation
// =========================================================================

/// Run the semantic gatekeeper as a final validation pass before codegen.
/// Returns Ok(()) if no issues found, or a list of diagnostics otherwise.
pub fn validate_module(module: &HirModule, types: TypeRegistry) -> Result<(), Vec<Diagnostic>> {
    let callables = CallableRegistry::default();
    let mut ctx = SemanticContext {
        types,
        symbols: SymbolTable::default(),
        callables,
        diagnostics: Vec::new(),
        loop_depth: 0,
        safety_context: SafetyContext::Safe,
        current_return_type: None,
        policy: crate::ffi::CapabilityPolicy::with_runtime_builtins(),
        extern_symbols: std::collections::HashSet::new(),
    };
    ctx.check_module(module)
}

/// Run the semantic gatekeeper and print any diagnostics.
/// Returns true if validation passed.
pub fn validate_module_with_reporting(module: &HirModule, types: TypeRegistry) -> bool {
    match validate_module(module, types) {
        Ok(()) => {
            println!("logicodex v1.38: Semantic gatekeeper validation passed");
            true
        }
        Err(diagnostics) => {
            eprintln!(
                "logicodex v1.38: Semantic gatekeeper found {} issue(s):",
                diagnostics.len()
            );
            for d in &diagnostics {
                eprintln!("  [{:?}] {}", d.severity, d.message_en);
            }
            false
        }
    }
}
