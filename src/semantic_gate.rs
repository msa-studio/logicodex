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
                self.check_missing_return(function.return_type.id, &function.body, item.span);
                self.current_return_type = previous_return_type;
                self.safety_context = previous_safety;
            }
            HirItem::Struct(struct_decl) => {
                self.check_struct_declaration(struct_decl, item.span);
            }
            HirItem::Enum(_) | HirItem::ExternFunction(_) => {}
        }
    }

    fn check_struct_declaration(&mut self, struct_decl: &crate::hir::HirStructDecl, span: Span) {
        let mut seen_fields = std::collections::BTreeSet::new();

        for field in &struct_decl.fields {
            if !seen_fields.insert(field.name.clone()) {
                self.push_error(
                    DiagnosticCode::LayoutError,
                    span,
                    format!(
                        "Ralat: Medan struct `{}` diisytihar lebih daripada sekali dalam `{}`",
                        field.name, struct_decl.name
                    ),
                    format!(
                        "Error: Struct field `{}` is declared more than once in `{}`",
                        field.name, struct_decl.name
                    ),
                );
            }

            if self.is_unknown_type(field.ty.id) {
                self.push_error(
                    DiagnosticCode::LayoutError,
                    span,
                    format!(
                        "Ralat: Medan struct `{}.{}` menggunakan jenis yang tidak diketahui",
                        struct_decl.name, field.name
                    ),
                    format!(
                        "Error: Struct field `{}.{}` uses an unknown type",
                        struct_decl.name, field.name
                    ),
                );
                continue;
            }

            match self.types.resolve(field.ty.id) {
                crate::types::TypeKind::Struct(layout_id) => {
                    if let Some(layout) = self.types.get_struct_layout(*layout_id) {
                        if layout.name == struct_decl.name {
                            self.push_error(
                                DiagnosticCode::LayoutError,
                                span,
                                format!(
                                    "Ralat: Struct `{}` mengandungi medan rekursif by-value `{}`",
                                    struct_decl.name, field.name
                                ),
                                format!(
                                    "Error: Struct `{}` contains recursive by-value field `{}`",
                                    struct_decl.name, field.name
                                ),
                            );
                        }
                    }
                }
                crate::types::TypeKind::Never => {
                    self.push_error(
                        DiagnosticCode::LayoutError,
                        span,
                        format!(
                            "Ralat: Medan struct `{}.{}` menggunakan jenis Never yang tiada layout nilai",
                            struct_decl.name, field.name
                        ),
                        format!(
                            "Error: Struct field `{}.{}` uses Never, which has no value layout",
                            struct_decl.name, field.name
                        ),
                    );
                }
                _ => {}
            }
        }
    }

    fn check_missing_return(
        &mut self,
        return_type: crate::types::TypeId,
        body: &HirBlock,
        span: Span,
    ) {
        if self.is_unit_type(return_type) || self.is_unknown_type(return_type) {
            return;
        }

        if self.block_definitely_returns(body) {
            return;
        }

        self.push_error(
            DiagnosticCode::TypeMismatch,
            span,
            format!(
                "Ralat: Fungsi yang memulangkan {} mesti mempunyai laluan return yang dijamin",
                self.type_label(return_type)
            ),
            format!(
                "Error: Function returning {} must have a guaranteed return path",
                self.type_label(return_type)
            ),
        );
    }

    fn block_definitely_returns(&self, block: &HirBlock) -> bool {
        // P0 fail-fast policy: only explicit return/control-flow returns satisfy
        // a non-Unit function return obligation. Tail expressions are still HIR
        // `Expr` statements and codegen discards their value; treating them as
        // returns would silently reach codegen's implicit return-zero fallback.
        self.block_definitely_returns_with_tail_values(block, false)
    }

    fn block_definitely_returns_with_tail_values(
        &self,
        block: &HirBlock,
        tail_values_allowed: bool,
    ) -> bool {
        let last_index = block.statements.len().saturating_sub(1);
        block.statements.iter().enumerate().any(|(idx, stmt)| {
            self.statement_definitely_returns(&stmt.node, tail_values_allowed && idx == last_index)
        })
    }

    fn statement_definitely_returns(&self, stmt: &HirStmt, tail_value_position: bool) -> bool {
        match stmt {
            HirStmt::Return(_) => true,
            HirStmt::Expr(expr) if tail_value_position => {
                self.tail_expr_returns_current_function(expr)
            }
            HirStmt::If {
                then_branch,
                else_branch: None,
                control_origin: crate::hir::HirControlOrigin::LoweredExhaustiveMatch,
                ..
            } => self.block_definitely_returns_with_tail_values(then_branch, tail_value_position),
            HirStmt::If {
                then_branch,
                else_branch: Some(else_branch),
                ..
            } => {
                self.block_definitely_returns_with_tail_values(then_branch, tail_value_position)
                    && self
                        .block_definitely_returns_with_tail_values(else_branch, tail_value_position)
            }
            HirStmt::UnsafeBlock(block) | HirStmt::HardwareZone(block) => {
                self.block_definitely_returns_with_tail_values(block, tail_value_position)
            }
            // Conservative P0: loops are not treated as guaranteed-return yet.
            // Full control-flow graph / divergence analysis is a later pass.
            _ => false,
        }
    }

    fn tail_expr_returns_current_function(&self, expr: &HirExpr) -> bool {
        let Some(expected) = self.current_return_type else {
            return false;
        };

        if self.is_unit_type(expected) || self.is_unknown_type(expected) {
            return false;
        }

        self.types_compatible(expected, self.expression_effective_type(expr))
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
                    if !self.is_unit_type(ty.id) {
                        self.check_call_result_used_as_value(value, stmt.span, "binding");
                    }
                    let actual = self.expression_effective_type(value);
                    if !self.types_compatible(ty.id, actual) {
                        self.push_error(
                            DiagnosticCode::TypeMismatch,
                            stmt.span,
                            format!(
                                "Ralat: Jenis ikatan tidak sepadan: dijangka {}, diterima {}",
                                self.type_label(ty.id),
                                self.type_label(actual)
                            ),
                            format!(
                                "Error: Binding type mismatch: expected {}, got {}",
                                self.type_label(ty.id),
                                self.type_label(actual)
                            ),
                        );
                    }
                }
            }
            HirStmt::Assign { target, value } => {
                self.check_expression(target);
                self.check_expression(value);
                self.check_assignment_target(target, stmt.span);
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
                ..
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
                        if !self.is_unit_type(expected) {
                            self.check_call_result_used_as_value(expr, stmt.span, "return");
                        }
                        let actual = self.expression_effective_type(expr);

                        if self.is_unit_type(expected) && !self.is_unit_type(actual) {
                            self.push_error(
                                DiagnosticCode::TypeMismatch,
                                stmt.span,
                                format!(
                                    "Ralat: Fungsi Unit tidak boleh memulangkan nilai jenis {}",
                                    self.type_label(actual)
                                ),
                                format!(
                                    "Error: Unit function cannot return a value of type {}",
                                    self.type_label(actual)
                                ),
                            );
                        }

                        if self.is_unknown_type(expected) || self.is_unknown_type(actual) {
                            self.push_error(
                                DiagnosticCode::TypeMismatch,
                                stmt.span,
                                "Ralat: Jenis pulangan tidak diketahui".to_string(),
                                "Error: Return type is unknown".to_string(),
                            );
                        }

                        if !self.types_compatible(expected, actual) {
                            self.push_error(
                                DiagnosticCode::TypeMismatch,
                                stmt.span,
                                format!(
                                    "Ralat: Jenis pulangan tidak sepadan: dijangka {}, diterima {}",
                                    self.type_label(expected),
                                    self.type_label(actual)
                                ),
                                format!(
                                    "Error: Return type mismatch: expected {}, got {}",
                                    self.type_label(expected),
                                    self.type_label(actual)
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
                self.check_binary_operator(op, left, right, expr.span);
            }
            HirExprKind::Unary { op, expr: operand } => {
                self.check_expression(operand);
                self.check_unary_operator(op, operand, expr.span);
            }
            HirExprKind::Field {
                base, field_name, ..
            } => {
                self.check_expression(base);
                self.check_field_access(base, field_name, expr.span);
            }
            HirExprKind::Cast {
                expr: operand,
                target,
            } => {
                self.check_expression(operand);
                self.check_cast_expression(operand, *target, expr.span);
            }
            HirExprKind::Index { base, index } => {
                self.check_expression(base);
                self.check_expression(index);
                self.check_index_expression(base, index, expr.span);
            }
            HirExprKind::ArrayLiteral { elements } => {
                for element in elements {
                    self.check_expression(element);
                }
                self.check_array_literal_elements(elements, expr.span);
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
                            if !self.is_unit_type(expected) {
                                self.check_call_result_used_as_value(
                                    actual,
                                    actual.span,
                                    "argument",
                                );
                            }
                            let actual_ty = self.expression_effective_type(actual);
                            if !self.types_compatible(expected, actual_ty) {
                                self.push_error(
                                    DiagnosticCode::TypeMismatch,
                                    actual.span,
                                    format!(
                                        "Ralat: Jenis argumen fungsi `{name}` tidak sepadan pada argumen {}: dijangka {}, diterima {}",
                                        idx + 1,
                                        self.type_label(expected),
                                        self.type_label(actual_ty)
                                    ),
                                    format!(
                                        "Error: Function `{name}` argument {} type mismatch: expected {}, got {}",
                                        idx + 1,
                                        self.type_label(expected),
                                        self.type_label(actual_ty)
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

    fn expression_effective_type(&self, expr: &HirExpr) -> crate::types::TypeId {
        match &expr.kind {
            HirExprKind::Call { callee, .. } => {
                self.symbols.callable_return(*callee).unwrap_or(expr.ty.id)
            }
            _ => expr.ty.id,
        }
    }

    fn check_call_result_used_as_value(&mut self, expr: &HirExpr, span: Span, context: &str) {
        let HirExprKind::Call { callee, .. } = &expr.kind else {
            return;
        };

        let name = self
            .symbols
            .callable_name(*callee)
            .unwrap_or("<unknown>")
            .to_string();

        match self.symbols.callable_return(*callee) {
            Some(return_type) if self.is_unit_type(return_type) => {
                self.push_error(
                    DiagnosticCode::TypeMismatch,
                    span,
                    format!(
                        "Ralat: Panggilan `{name}` memulangkan Unit dan tidak boleh digunakan sebagai nilai dalam {context}"
                    ),
                    format!(
                        "Error: Call `{name}` returns Unit and cannot be used as a value in {context}"
                    ),
                );
            }
            Some(return_type) if self.is_unknown_type(return_type) => {
                self.push_error(
                    DiagnosticCode::TypeMismatch,
                    span,
                    format!(
                        "Ralat: Jenis hasil panggilan `{name}` tidak dapat diselesaikan untuk digunakan sebagai nilai dalam {context}"
                    ),
                    format!(
                        "Error: Call `{name}` result type could not be resolved for value use in {context}"
                    ),
                );
            }
            None => {
                self.push_error(
                    DiagnosticCode::TypeMismatch,
                    span,
                    format!(
                        "Ralat: Jenis hasil panggilan `{name}` tidak diketahui untuk digunakan sebagai nilai dalam {context}"
                    ),
                    format!(
                        "Error: Call `{name}` result type is unknown for value use in {context}"
                    ),
                );
            }
            _ => {}
        }
    }

    fn check_cast_expression(
        &mut self,
        source: &HirExpr,
        target: crate::types::TypeRef,
        span: Span,
    ) {
        if self.is_unknown_type(source.ty.id) || self.is_unknown_type(target.id) {
            return;
        }

        if self.is_allowed_cast(source.ty.id, target.id) {
            return;
        }

        self.push_error(
            DiagnosticCode::TypeMismatch,
            span,
            format!(
                "Ralat: Cast tidak sah: tidak boleh cast {} kepada {}",
                self.type_label(source.ty.id),
                self.type_label(target.id)
            ),
            format!(
                "Error: Invalid cast: cannot cast {} to {}",
                self.type_label(source.ty.id),
                self.type_label(target.id)
            ),
        );
    }

    fn is_allowed_cast(&self, source: crate::types::TypeId, target: crate::types::TypeId) -> bool {
        self.types.is_equivalent(source, target)
            || (self.is_integer_type(source) && self.is_integer_type(target))
            || self.is_transitional_scalar_abi_cast(source, target)
    }

    fn is_transitional_scalar_abi_cast(
        &self,
        source: crate::types::TypeId,
        target: crate::types::TypeId,
    ) -> bool {
        self.is_option_result_i64_scalar_abi(source) && self.is_i64_type(target)
    }

    fn check_field_access(&mut self, base: &HirExpr, field_name: &str, span: Span) {
        if self.is_unknown_type(base.ty.id) {
            return;
        }

        match self.types.resolve(base.ty.id) {
            crate::types::TypeKind::Struct(layout_id) => {
                if let Some(layout) = self.types.get_struct_layout(*layout_id) {
                    if layout.fields.iter().any(|field| field.name == field_name) {
                        return;
                    }

                    self.push_error(
                        DiagnosticCode::TypeMismatch,
                        span,
                        format!(
                            "Ralat: Medan struct `{}` tidak ditemui pada {}",
                            field_name,
                            self.type_label(base.ty.id)
                        ),
                        format!(
                            "Error: Struct field `{}` was not found on {}",
                            field_name,
                            self.type_label(base.ty.id)
                        ),
                    );
                }
            }
            _ => {
                self.push_error(
                    DiagnosticCode::TypeMismatch,
                    span,
                    format!(
                        "Ralat: Asas akses medan mesti struct, diterima {}",
                        self.type_label(base.ty.id)
                    ),
                    format!(
                        "Error: Field access base must be a struct, got {}",
                        self.type_label(base.ty.id)
                    ),
                );
            }
        }
    }

    fn check_index_expression(&mut self, base: &HirExpr, index: &HirExpr, span: Span) {
        if !self.is_supported_index_base(base) {
            self.push_error(
                DiagnosticCode::TypeMismatch,
                span,
                format!(
                    "Ralat: Asas indeks mesti tatasusunan tetap tempatan, diterima {}",
                    self.type_label(base.ty.id)
                ),
                format!(
                    "Error: Index base must be a local fixed array, got {}",
                    self.type_label(base.ty.id)
                ),
            );
        }

        if !self.is_unknown_type(index.ty.id) && !self.is_integer_type(index.ty.id) {
            self.push_error(
                DiagnosticCode::TypeMismatch,
                span,
                format!(
                    "Ralat: Indeks tatasusunan mesti integer, diterima {}",
                    self.type_label(index.ty.id)
                ),
                format!(
                    "Error: Array index must be an integer, got {}",
                    self.type_label(index.ty.id)
                ),
            );
        }
    }

    fn is_supported_index_base(&self, base: &HirExpr) -> bool {
        if self.is_unknown_type(base.ty.id) {
            return true;
        }

        matches!(base.kind, HirExprKind::Local(_))
            && matches!(
                self.types.resolve(base.ty.id),
                crate::types::TypeKind::Array { .. }
            )
    }

    fn check_array_literal_elements(&mut self, elements: &[HirExpr], span: Span) {
        let Some(expected) = elements
            .iter()
            .map(|element| element.ty.id)
            .find(|id| !self.is_unknown_type(*id))
        else {
            return;
        };

        for (idx, element) in elements.iter().enumerate() {
            if self.is_unknown_type(element.ty.id) {
                continue;
            }

            if !self.types_compatible(expected, element.ty.id) {
                self.push_error(
                    DiagnosticCode::TypeMismatch,
                    span,
                    format!(
                        "Ralat: Jenis elemen literal tatasusunan tidak sepadan pada elemen {}: dijangka {}, diterima {}",
                        idx + 1,
                        self.type_label(expected),
                        self.type_label(element.ty.id)
                    ),
                    format!(
                        "Error: Array literal element {} type mismatch: expected {}, got {}",
                        idx + 1,
                        self.type_label(expected),
                        self.type_label(element.ty.id)
                    ),
                );
            }
        }
    }

    fn check_assignment_target(&mut self, target: &HirExpr, span: Span) {
        if self.is_assignment_target(target) {
            return;
        }

        self.push_error(
            DiagnosticCode::TypeMismatch,
            span,
            format!(
                "Ralat: Sasaran tugasan tidak boleh ditulis: {}",
                self.assignment_target_label(target)
            ),
            format!(
                "Error: Assignment target is not writable: {}",
                self.assignment_target_label(target)
            ),
        );
    }

    fn is_assignment_target(&self, target: &HirExpr) -> bool {
        match &target.kind {
            HirExprKind::Local(_) | HirExprKind::Index { .. } | HirExprKind::Field { .. } => true,
            _ => false,
        }
    }

    fn assignment_target_label(&self, target: &HirExpr) -> &'static str {
        match &target.kind {
            HirExprKind::Literal(_) => "literal",
            HirExprKind::Local(_) => "local",
            HirExprKind::Global(_) => "global",
            HirExprKind::Binary { .. } => "binary expression",
            HirExprKind::Unary { .. } => "unary expression",
            HirExprKind::Call { .. } => "call result",
            HirExprKind::Field { .. } => "field",
            HirExprKind::Index { .. } => "index",
            HirExprKind::ArrayLiteral { .. } => "array literal",
            HirExprKind::Cast { .. } => "cast result",
            HirExprKind::ResultOk { .. } => "Result::Ok",
            HirExprKind::ResultErr { .. } => "Result::Err",
            HirExprKind::OptionSome { .. } => "Option::Some",
            HirExprKind::OptionNone => "Option::None",
            HirExprKind::Spawn { .. } => "spawn result",
            HirExprKind::Join { .. } => "join result",
            HirExprKind::ChannelCreate { .. } => "channel creation",
            HirExprKind::ChannelSend { .. } => "channel send",
            HirExprKind::ChannelRecv { .. } => "channel receive",
            HirExprKind::ChannelTrySend { .. } => "channel try_send",
            HirExprKind::ChannelTryRecv { .. } => "channel try_recv",
            HirExprKind::Yield => "yield",
            HirExprKind::Sleep { .. } => "sleep",
            HirExprKind::ChannelTimeoutRecv { .. } => "channel timeout_recv",
        }
    }

    fn check_binary_operator(
        &mut self,
        op: &BinaryOpAst,
        left: &HirExpr,
        right: &HirExpr,
        span: Span,
    ) {
        if matches!(op, BinaryOpAst::Div | BinaryOpAst::Mod) {
            if let HirExprKind::Literal(LiteralAst::Integer(0)) = &right.kind {
                self.push_error(
                    DiagnosticCode::DivisionByZero,
                    span,
                    "Ralat: Pembahagian dengan sifar".to_string(),
                    "Error: Division by zero".to_string(),
                );
            }
        }

        if self.is_unknown_type(left.ty.id) || self.is_unknown_type(right.ty.id) {
            return;
        }

        match op {
            BinaryOpAst::Add
            | BinaryOpAst::Sub
            | BinaryOpAst::Mul
            | BinaryOpAst::Div
            | BinaryOpAst::Mod
            | BinaryOpAst::BitAnd
            | BinaryOpAst::BitOr
            | BinaryOpAst::BitXor
            | BinaryOpAst::ShiftLeft
            | BinaryOpAst::ShiftRight => {
                if !(self.is_integer_type(left.ty.id) && self.is_integer_type(right.ty.id)) {
                    self.push_binary_operator_error(span, "integer", left.ty.id, right.ty.id);
                }
            }
            BinaryOpAst::Lt | BinaryOpAst::Lte | BinaryOpAst::Gt | BinaryOpAst::Gte => {
                if !(self.is_integer_type(left.ty.id) && self.is_integer_type(right.ty.id)) {
                    self.push_binary_operator_error(
                        span,
                        "integer comparison",
                        left.ty.id,
                        right.ty.id,
                    );
                }
            }
            BinaryOpAst::LogicalAnd | BinaryOpAst::LogicalOr => {
                if !(self.is_bool_type(left.ty.id) && self.is_bool_type(right.ty.id)) {
                    self.push_binary_operator_error(span, "Bool", left.ty.id, right.ty.id);
                }
            }
            BinaryOpAst::Eq | BinaryOpAst::NotEq => {
                if !self.types_compatible(left.ty.id, right.ty.id) {
                    self.push_binary_operator_error(span, "matching", left.ty.id, right.ty.id);
                }
            }
        }
    }

    fn check_unary_operator(&mut self, op: &crate::hir::UnaryOpAst, operand: &HirExpr, span: Span) {
        if self.is_unknown_type(operand.ty.id) {
            return;
        }

        match op {
            crate::hir::UnaryOpAst::Negate => {
                if !self.is_integer_type(operand.ty.id) {
                    self.push_unary_operator_error(span, "integer", operand.ty.id);
                }
            }
            crate::hir::UnaryOpAst::LogicalNot => {
                if !self.is_bool_type(operand.ty.id) {
                    self.push_unary_operator_error(span, "Bool", operand.ty.id);
                }
            }
            crate::hir::UnaryOpAst::AddressOf | crate::hir::UnaryOpAst::Deref => {}
        }
    }

    fn push_binary_operator_error(
        &mut self,
        span: Span,
        expected: &str,
        left: crate::types::TypeId,
        right: crate::types::TypeId,
    ) {
        self.push_error(
            DiagnosticCode::TypeMismatch,
            span,
            format!(
                "Ralat: Jenis operand operator binari tidak sah: dijangka {}, diterima {} dan {}",
                expected,
                self.type_label(left),
                self.type_label(right)
            ),
            format!(
                "Error: Invalid binary operator operand types: expected {}, got {} and {}",
                expected,
                self.type_label(left),
                self.type_label(right)
            ),
        );
    }

    fn push_unary_operator_error(
        &mut self,
        span: Span,
        expected: &str,
        actual: crate::types::TypeId,
    ) {
        self.push_error(
            DiagnosticCode::TypeMismatch,
            span,
            format!(
                "Ralat: Jenis operand operator unari tidak sah: dijangka {}, diterima {}",
                expected,
                self.type_label(actual)
            ),
            format!(
                "Error: Invalid unary operator operand type: expected {}, got {}",
                expected,
                self.type_label(actual)
            ),
        );
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
                        control_origin: crate::hir::HirControlOrigin::Plain,
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
                            control_origin: crate::hir::HirControlOrigin::Plain,
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

    fn hir_function_returning_i64(
        ctx: &super::SemanticContext,
        expr: crate::hir::HirExpr,
    ) -> crate::hir::HirModule {
        crate::hir::HirModule {
            items: vec![spanned(crate::hir::HirItem::Function(
                crate::hir::HirFunction {
                    name: "cast_test".to_string(),
                    symbol: crate::hir::SymbolId(0),
                    params: vec![],
                    return_type: crate::types::TypeRef {
                        id: ctx.types.primitive_ids().i64_,
                    },
                    body: crate::hir::HirBlock {
                        statements: vec![spanned(crate::hir::HirStmt::Return(Some(expr)))],
                    },
                    safety: SafetyContext::Safe,
                },
            ))],
        }
    }

    #[test]
    fn accepts_same_type_hir_cast() {
        let mut ctx = base_context();
        let ids = ctx.types.primitive_ids();
        let i64_ref = crate::types::TypeRef { id: ids.i64_ };

        let cast = crate::hir::HirExpr {
            kind: crate::hir::HirExprKind::Cast {
                expr: Box::new(expr_i64(&ctx.types, 7)),
                target: i64_ref,
            },
            ty: i64_ref,
            span: Span::unknown(),
        };

        let module = hir_function_returning_i64(&ctx, cast);
        ctx.check_module(&module)
            .expect("same-type HIR cast should pass");
    }

    #[test]
    fn rejects_invalid_unit_to_i64_hir_cast() {
        let mut ctx = base_context();
        let ids = ctx.types.primitive_ids();
        let unit_ref = crate::types::TypeRef { id: ids.unit };
        let i64_ref = crate::types::TypeRef { id: ids.i64_ };

        let unit_expr = crate::hir::HirExpr {
            kind: crate::hir::HirExprKind::Literal(crate::hir::LiteralAst::Unit),
            ty: unit_ref,
            span: Span::unknown(),
        };

        let cast = crate::hir::HirExpr {
            kind: crate::hir::HirExprKind::Cast {
                expr: Box::new(unit_expr),
                target: i64_ref,
            },
            ty: i64_ref,
            span: Span::unknown(),
        };

        let module = hir_function_returning_i64(&ctx, cast);
        let diagnostics = ctx
            .check_module(&module)
            .expect_err("Unit -> I64 HIR cast should fail");

        let rendered = diagnostics
            .iter()
            .map(|d| format!("{} / {}", d.message_ms, d.message_en))
            .collect::<Vec<_>>()
            .join("\n");

        assert!(
            rendered.contains("Invalid cast") || rendered.contains("Cast tidak sah"),
            "expected invalid cast diagnostic, got:\n{rendered}"
        );
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
