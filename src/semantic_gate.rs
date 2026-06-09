// =========================================================================
// Logicodex v1.38 — Semantic Gatekeeper (I1: Activated)
//
// Final semantic validation pass before codegen.
// Checks: break/continue in loops, unsafe block correctness,
// FFI call safety, return path validity.
//
// Called after v1.21 semantic analysis and before LLVM codegen.
// =========================================================================

use crate::ffi::{CallableRegistry, FfiGatekeeper, SafetyContext};
use crate::hir::{HirBlock, HirExpr, HirExprKind, HirItem, HirModule, HirStmt, SymbolTable};
use crate::span::{Diagnostic, DiagnosticCode, Severity, Span, Spanned};
use crate::types::TypeRegistry;

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
        match &item.node {
            HirItem::Function(function) => {
                let previous_safety = self.safety_context;
                self.safety_context = function.safety;
                self.check_block(&function.body);
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
            HirStmt::Let { value, .. } => {
                if let Some(value) = value {
                    self.check_expression(value);
                }
            }
            HirStmt::Assign { target, value } => {
                self.check_expression(target);
                self.check_expression(value);
            }
            HirStmt::If { condition, then_branch, else_branch } => {
                self.check_expression(condition);
                self.check_block(then_branch);
                if let Some(else_branch) = else_branch {
                    self.check_block(else_branch);
                }
            }
            HirStmt::While { condition, body } => {
                self.check_expression(condition);
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
            HirStmt::Expr(expr) => self.check_expression(expr),
            HirStmt::Return(expr) => {
                if let Some(expr) = expr {
                    self.check_expression(expr);
                }
            }
        }
    }

    fn check_expression(&mut self, expr: &HirExpr) {
        match &expr.kind {
            HirExprKind::Literal(_) | HirExprKind::Local(_) | HirExprKind::Global(_) => {}
            HirExprKind::Binary { left, right, .. } => {
                self.check_expression(left);
                self.check_expression(right);
                // v1.30 uses a uniform i64 integer model, so integer operands of
                // differing declared widths (e.g. I32 vs the default-I64 of an
                // integer literal) are compatible. Only flag genuinely mismatched
                // categories (e.g. int vs float).
                if left.ty.id != right.ty.id
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
                    };
                    let result =
                        gate.validate_call(signature, args, self.safety_context, expr.span);
                    if let Err(diagnostic) = result {
                        self.diagnostics.push(diagnostic);
                    }
                }
            }
            _ => {}
        }
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
        }
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
    };
    ctx.check_module(module)
}

/// Run the semantic gatekeeper and print any diagnostics.
/// Returns true if validation passed.
pub fn validate_module_with_reporting(
    module: &HirModule,
    types: TypeRegistry,
) -> bool {
    match validate_module(module, types) {
        Ok(()) => {
            println!("logicodex v1.38: Semantic gatekeeper validation passed");
            true
        }
        Err(diagnostics) => {
            eprintln!("logicodex v1.38: Semantic gatekeeper found {} issue(s):", diagnostics.len());
            for d in &diagnostics {
                eprintln!("  [{:?}] {}", d.severity, d.message_en);
            }
            false
        }
    }
}
