// =========================================================================
// Logicodex Semantic Gate — ACTIVE / CANONICAL MEANING AUTHORITY
//
// Canonical HIR semantic validation used by `check`, `compile`, and the
// defensive codegen boundary. The retired AST Analyzer in `semantic.rs` is
// not authoritative for current compiler behaviour.
//
// Checks: break/continue in loops, unsafe block correctness,
// FFI call safety, return path validity.
// =========================================================================

use crate::ffi::{CallableRegistry, FfiGatekeeper, SafetyContext};
use crate::hir::{
    BinaryOpAst, HirBlock, HirCallableKind, HirCallableSignature, HirCallableValueUse, HirExpr,
    HirExprKind, HirItem, HirModule, HirStmt, LiteralAst, LoweredModule, SymbolTable,
};
use crate::span::{Diagnostic, DiagnosticCode, Severity, Span, Spanned};
use crate::types::{CallableId, TypeId, TypeRegistry};
use std::collections::HashMap;

/// Frozen language-level callable metadata carried by `LoweredModule`.
type CallableSignatures = HashMap<CallableId, HirCallableSignature>;

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
    /// module's ExternFunction items at check start). Their language identity
    /// comes from frozen callable metadata, not the FFI CallableRegistry, so the
    /// capability gate checks them here directly — a Call to one of these names
    /// is a foreign call and must satisfy the CapabilityPolicy.
    pub extern_symbols: std::collections::HashSet<String>,
}

impl SemanticContext {
    /// Plain-HIR validation. A bare `HirModule` carries no frozen callable
    /// metadata, so every call or spawn in it fails closed with
    /// `CallableMetadataIntegrity`. Callable-bearing HIR must be validated
    /// through `check_lowered_module`.
    pub fn check_module(&mut self, module: &HirModule) -> Result<(), Vec<Diagnostic>> {
        self.reset_run_state();
        self.check_module_with_signatures(module, &HashMap::new())
    }

    /// Validate the canonical lowering artifact. Callable semantics are read
    /// only from the frozen table carried by `LoweredModule`; the lowering-time
    /// `SymbolTable` and the FFI `CallableRegistry` never substitute for it.
    pub fn check_lowered_module(&mut self, lowered: &LoweredModule) -> Result<(), Vec<Diagnostic>> {
        self.reset_run_state();
        self.check_callable_metadata_integrity(lowered);
        if !self.diagnostics.is_empty() {
            return Err(self.diagnostics.clone());
        }
        self.check_module_with_signatures(&lowered.module, &lowered.callable_signatures)
    }

    /// Clear all per-run state so a context reused after a failed (or
    /// successful) run starts clean. Configuration (types, callables, policy)
    /// is preserved.
    fn reset_run_state(&mut self) {
        self.diagnostics.clear();
        self.extern_symbols.clear();
        self.loop_depth = 0;
        self.safety_context = SafetyContext::Safe;
        self.current_return_type = None;
    }

    fn check_module_with_signatures(
        &mut self,
        module: &HirModule,
        signatures: &CallableSignatures,
    ) -> Result<(), Vec<Diagnostic>> {
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
        // recognise foreign calls. The name travels with the HIR node; it is
        // never recovered from the lowering-time SymbolTable.
        for item in &module.items {
            if let HirItem::ExternFunction(ext) = &item.node {
                self.extern_symbols.insert(ext.name.clone());
            }
        }
        for item in &module.items {
            self.check_item(item, signatures);
        }

        if self.diagnostics.is_empty() {
            Ok(())
        } else {
            Err(self.diagnostics.clone())
        }
    }

    /// Fail closed on missing, duplicate, stale, inconsistent, or
    /// provenance-invalid frozen callable metadata before any other semantic
    /// check (and therefore before any LLVM construction) runs.
    fn check_callable_metadata_integrity(&mut self, lowered: &LoweredModule) {
        let signatures = &lowered.callable_signatures;

        let mut declarations: HashMap<&str, Vec<HirCallableKind>> = HashMap::new();
        for item in &lowered.module.items {
            let declared = match &item.node {
                HirItem::Function(function) => {
                    Some((function.name.as_str(), HirCallableKind::Function))
                }
                HirItem::Struct(decl) => Some((decl.name.as_str(), HirCallableKind::Constructor)),
                HirItem::ExternFunction(ext) => {
                    Some((ext.name.as_str(), HirCallableKind::UserExtern))
                }
                HirItem::Enum(_) => None,
            };
            if let Some((name, kind)) = declared {
                declarations.entry(name).or_default().push(kind);
            }
        }

        // Table-level integrity: exact identity keys, unique names, coherent
        // shapes, canonical builtins, and no metadata without a declaration.
        let mut by_name: HashMap<&str, CallableId> = HashMap::new();
        let mut keys: Vec<&CallableId> = signatures.keys().collect();
        keys.sort_by_key(|id| id.0);
        for key in keys {
            let signature = &signatures[key];
            if *key != signature.callable {
                self.push_callable_metadata_error(
                    Span::unknown(),
                    format!(
                        "Ralat: metadata callable rosak: kekunci {key:?} tidak sepadan dengan identiti {:?}",
                        signature.callable
                    ),
                    format!(
                        "Error: corrupt callable metadata: key {key:?} does not match identity {:?}",
                        signature.callable
                    ),
                );
            }
            if signature.params.len() != signature.param_enums.len() {
                self.push_callable_metadata_error(
                    Span::unknown(),
                    format!(
                        "Ralat: metadata callable `{}` rosak: {} jenis parameter tetapi {} anotasi enum",
                        signature.name,
                        signature.params.len(),
                        signature.param_enums.len()
                    ),
                    format!(
                        "Error: corrupt callable metadata for `{}`: {} parameter types but {} enum annotations",
                        signature.name,
                        signature.params.len(),
                        signature.param_enums.len()
                    ),
                );
            }
            if let Some(previous) = by_name.insert(signature.name.as_str(), *key) {
                if previous != *key {
                    self.push_callable_metadata_error(
                        Span::unknown(),
                        format!(
                            "Ralat: metadata callable rosak: nama `{}` mempunyai identiti berganda",
                            signature.name
                        ),
                        format!(
                            "Error: corrupt callable metadata: name `{}` has duplicate identities",
                            signature.name
                        ),
                    );
                }
            }
            if signature.kind == HirCallableKind::Builtin {
                let provenance_matches = lowered
                    .builtin_provenance
                    .get(&signature.name)
                    .is_some_and(|callable| callable == key);
                if !provenance_matches || !self.is_canonical_builtin_signature(signature) {
                    self.push_callable_metadata_error(
                        Span::unknown(),
                        format!(
                            "Ralat: provenance atau bentuk builtin `{}` tidak sah",
                            signature.name
                        ),
                        format!(
                            "Error: builtin `{}` has invalid provenance or canonical shape",
                            signature.name
                        ),
                    );
                }
            } else if !declarations
                .get(signature.name.as_str())
                .is_some_and(|kinds| kinds.contains(&signature.kind))
            {
                self.push_callable_metadata_error(
                    Span::unknown(),
                    format!(
                        "Ralat: metadata callable `{}` tiada deklarasi HIR yang sepadan",
                        signature.name
                    ),
                    format!(
                        "Error: callable metadata `{}` has no matching HIR declaration",
                        signature.name
                    ),
                );
            }
        }
        let mut provenance: Vec<(&String, &CallableId)> =
            lowered.builtin_provenance.iter().collect();
        provenance.sort_by_key(|(name, _)| *name);
        for (name, callable) in provenance {
            if !matches!(
                signatures.get(callable),
                Some(signature)
                    if signature.kind == HirCallableKind::Builtin && signature.name == *name
            ) {
                self.push_callable_metadata_error(
                    Span::unknown(),
                    format!("Ralat: provenance builtin `{name}` tidak sah"),
                    format!("Error: builtin provenance `{name}` is invalid"),
                );
            }
        }

        // Declaration-level integrity: every callable declaration is bound to
        // exactly one frozen signature of the same kind, name, and shape.
        // Functions and user externs carry their exact CallableId and are
        // indexed by it; struct declarations carry none, so their constructor
        // is verified through the unique-name table established above. Names
        // that are declared more than once are reported as DuplicateDefinition.
        for item in &lowered.module.items {
            let (name, kind, exact_id) = match &item.node {
                HirItem::Function(function) => (
                    function.name.as_str(),
                    HirCallableKind::Function,
                    Some(function.callable),
                ),
                HirItem::Struct(decl) => (decl.name.as_str(), HirCallableKind::Constructor, None),
                HirItem::ExternFunction(ext) => (
                    ext.name.as_str(),
                    HirCallableKind::UserExtern,
                    Some(ext.callable),
                ),
                HirItem::Enum(_) => continue,
            };
            let indexed = match exact_id {
                Some(callable) => signatures.get(&callable),
                None => by_name
                    .get(name)
                    .and_then(|callable| signatures.get(callable)),
            };
            let signature = indexed.filter(|signature| {
                signature.kind == kind
                    && signature.name == name
                    && exact_id.is_none_or(|id| id == signature.callable)
            });
            let Some(signature) = signature else {
                self.push_callable_metadata_error(
                    item.span,
                    format!(
                        "Ralat: metadata callable hilang atau tidak sepadan untuk deklarasi `{name}`"
                    ),
                    format!(
                        "Error: callable metadata is missing or inconsistent for declaration `{name}`"
                    ),
                );
                continue;
            };
            if declarations.get(name).is_some_and(|kinds| kinds.len() > 1) {
                continue;
            }
            // Frozen enum annotations must equal the declaration-side
            // annotations carried by HIR (read from the AST, independently of
            // the frozen metadata). Parameters/fields are compared before the
            // return; both precede the shape check below.
            match &item.node {
                HirItem::Function(function) => {
                    if signature.param_enums != function.declaration_param_enums {
                        self.push_callable_metadata_error(
                            Span::unknown(),
                            format!(
                                "Ralat: metadata callable untuk fungsi '{name}' mempunyai anotasi enum parameter yang tidak konsisten dengan deklarasi"
                            ),
                            format!(
                                "Error: callable metadata for function '{name}' has parameter enum annotations inconsistent with the declaration"
                            ),
                        );
                    }
                    if signature.return_enum != function.declaration_return_enum {
                        self.push_callable_metadata_error(
                            Span::unknown(),
                            format!(
                                "Ralat: metadata callable untuk fungsi '{name}' mempunyai anotasi enum pulangan yang tidak konsisten dengan deklarasi"
                            ),
                            format!(
                                "Error: callable metadata for function '{name}' has return enum annotation inconsistent with the declaration"
                            ),
                        );
                    }
                }
                HirItem::Struct(decl) => {
                    if signature.param_enums != decl.declaration_field_enums {
                        self.push_callable_metadata_error(
                            Span::unknown(),
                            format!(
                                "Ralat: metadata callable untuk konstruktor '{name}' mempunyai anotasi enum medan yang tidak konsisten dengan deklarasi struct"
                            ),
                            format!(
                                "Error: callable metadata for constructor '{name}' has field enum annotations inconsistent with the struct declaration"
                            ),
                        );
                    }
                    // A constructor returns its struct, never an enum.
                    if signature.return_enum.is_some() {
                        self.push_callable_metadata_error(
                            Span::unknown(),
                            format!(
                                "Ralat: metadata callable untuk konstruktor '{name}' mempunyai anotasi enum pulangan yang tidak konsisten dengan deklarasi struct"
                            ),
                            format!(
                                "Error: callable metadata for constructor '{name}' has an enum return annotation inconsistent with the struct declaration"
                            ),
                        );
                    }
                }
                HirItem::ExternFunction(_) | HirItem::Enum(_) => {}
            }
            let expected_value_use = |return_type| {
                if self.is_unit_type(return_type) {
                    HirCallableValueUse::StatementOnly
                } else {
                    HirCallableValueUse::Value
                }
            };
            let shape_matches = match &item.node {
                HirItem::Function(function) => {
                    signature.params
                        == function
                            .params
                            .iter()
                            .map(|param| param.ty.id)
                            .collect::<Vec<_>>()
                        && signature.return_type == function.return_type.id
                        && !signature.is_variadic
                        && signature.value_use == expected_value_use(function.return_type.id)
                }
                HirItem::Struct(decl) => {
                    signature.params
                        == decl
                            .fields
                            .iter()
                            .map(|field| field.ty.id)
                            .collect::<Vec<_>>()
                        && self.is_canonical_constructor_return(&decl.name, signature.return_type)
                        && !signature.is_variadic
                        && signature.value_use == HirCallableValueUse::Value
                }
                HirItem::ExternFunction(ext) => {
                    signature.params == ext.params
                        && signature.return_type == ext.return_type
                        && signature.is_variadic == ext.is_variadic
                        && signature.value_use == expected_value_use(ext.return_type)
                }
                HirItem::Enum(_) => true,
            };
            if !shape_matches {
                self.push_callable_metadata_error(
                    item.span,
                    format!(
                        "Ralat: bentuk metadata callable `{name}` tidak sepadan dengan deklarasi HIR"
                    ),
                    format!(
                        "Error: callable metadata shape for `{name}` does not match its HIR declaration"
                    ),
                );
            }
            // Struct-by-value FFI signatures have no supported foreign ABI;
            // reject them here rather than failing inside LLVM construction.
            if signature.kind == HirCallableKind::UserExtern {
                let struct_typed = signature
                    .params
                    .iter()
                    .chain(std::iter::once(&signature.return_type))
                    .any(|ty| {
                        matches!(self.types.get(*ty), Some(crate::types::TypeKind::Struct(_)))
                    });
                if struct_typed {
                    self.push_error(
                        DiagnosticCode::FfiBoundaryViolation,
                        item.span,
                        format!(
                            "Ralat: extern `{name}` menggunakan jenis struct mengikut nilai, yang tidak disokong oleh sempadan FFI"
                        ),
                        format!(
                            "Error: extern `{name}` uses a struct type by value, which the FFI boundary does not support"
                        ),
                    );
                }
            }
        }

        // Reference-level integrity: every call and spawn target resolves to
        // frozen metadata, and call result stamps are not stale.
        for item in &lowered.module.items {
            if let HirItem::Function(function) = &item.node {
                self.check_callable_references_in_block(&function.body, signatures);
            }
        }
    }

    /// A struct constructor returns exactly the struct type of its own
    /// declaration: the interned `Struct` kind of the layout registered under
    /// that name (the identity lowering assigned). Unknown or out-of-range
    /// type ids never match.
    fn is_canonical_constructor_return(&self, struct_name: &str, return_type: TypeId) -> bool {
        let Some((layout, _)) = self.types.find_struct_by_name(struct_name) else {
            return false;
        };
        matches!(
            self.types.get(return_type),
            Some(crate::types::TypeKind::Struct(id)) if *id == layout
        )
    }

    fn is_canonical_builtin_signature(&self, signature: &HirCallableSignature) -> bool {
        let ids = self.types.primitive_ids();
        let common = signature.return_type == ids.unit
            && signature.return_enum.is_none()
            && signature.value_use == HirCallableValueUse::StatementOnly
            && signature.param_enums.iter().all(Option::is_none);
        common
            && match signature.name.as_str() {
                "print" => signature.params.is_empty() && signature.is_variadic,
                "logicodex_sleep" => signature.params == [ids.i64_] && !signature.is_variadic,
                "logicodex_yield" => signature.params.is_empty() && !signature.is_variadic,
                _ => false,
            }
    }

    fn check_callable_references_in_block(
        &mut self,
        block: &HirBlock,
        signatures: &CallableSignatures,
    ) {
        for stmt in &block.statements {
            match &stmt.node {
                HirStmt::Let { value, .. } => {
                    if let Some(value) = value {
                        self.check_callable_references_in_expr(value, signatures);
                    }
                }
                HirStmt::Assign { target, value } => {
                    self.check_callable_references_in_expr(target, signatures);
                    self.check_callable_references_in_expr(value, signatures);
                }
                HirStmt::If {
                    condition,
                    then_branch,
                    else_branch,
                    ..
                } => {
                    self.check_callable_references_in_expr(condition, signatures);
                    self.check_callable_references_in_block(then_branch, signatures);
                    if let Some(else_branch) = else_branch {
                        self.check_callable_references_in_block(else_branch, signatures);
                    }
                }
                HirStmt::While { condition, body } => {
                    self.check_callable_references_in_expr(condition, signatures);
                    self.check_callable_references_in_block(body, signatures);
                }
                HirStmt::Loop { body }
                | HirStmt::UnsafeBlock(body)
                | HirStmt::HardwareZone(body) => {
                    self.check_callable_references_in_block(body, signatures);
                }
                HirStmt::Expr(expr) | HirStmt::Return(Some(expr)) => {
                    self.check_callable_references_in_expr(expr, signatures);
                }
                HirStmt::Break { .. }
                | HirStmt::Continue { .. }
                | HirStmt::HardwareDecl { .. }
                | HirStmt::Return(None) => {}
            }
        }
    }

    fn check_callable_references_in_expr(
        &mut self,
        expr: &HirExpr,
        signatures: &CallableSignatures,
    ) {
        match &expr.kind {
            HirExprKind::Call { callee, args } => {
                match signatures.get(callee) {
                    None => self.push_missing_callable_metadata(*callee, expr.span),
                    Some(signature) if expr.ty.id != signature.return_type => {
                        let name = signature.name.clone();
                        self.push_callable_metadata_error(
                            expr.span,
                            format!(
                                "Ralat: cap jenis hasil panggilan `{name}` tidak sepadan dengan metadata beku"
                            ),
                            format!(
                                "Error: call result type stamp for `{name}` does not match frozen metadata"
                            ),
                        );
                    }
                    Some(_) => {}
                }
                for arg in args {
                    self.check_callable_references_in_expr(arg, signatures);
                }
            }
            HirExprKind::Spawn {
                actor_name,
                actor_callable,
                args,
            } => {
                self.check_spawn_target(actor_name, *actor_callable, args, expr.span, signatures);
                for arg in args {
                    self.check_callable_references_in_expr(arg, signatures);
                }
            }
            HirExprKind::Binary { left, right, .. } => {
                self.check_callable_references_in_expr(left, signatures);
                self.check_callable_references_in_expr(right, signatures);
            }
            HirExprKind::Unary { expr: inner, .. }
            | HirExprKind::Cast { expr: inner, .. }
            | HirExprKind::Field { base: inner, .. }
            | HirExprKind::ResultOk { value: inner }
            | HirExprKind::ResultErr { value: inner }
            | HirExprKind::OptionSome { value: inner }
            | HirExprKind::ChannelCreate { capacity: inner }
            | HirExprKind::ChannelRecv { channel: inner }
            | HirExprKind::ChannelTrySend {
                channel_name: _,
                value: inner,
            }
            | HirExprKind::Sleep { duration_ms: inner }
            | HirExprKind::ChannelTimeoutRecv {
                channel_name: _,
                timeout_ms: inner,
            } => self.check_callable_references_in_expr(inner, signatures),
            HirExprKind::Index { base, index } => {
                self.check_callable_references_in_expr(base, signatures);
                self.check_callable_references_in_expr(index, signatures);
            }
            HirExprKind::ArrayLiteral { elements } => {
                for element in elements {
                    self.check_callable_references_in_expr(element, signatures);
                }
            }
            HirExprKind::ChannelSend { channel, value } => {
                self.check_callable_references_in_expr(channel, signatures);
                self.check_callable_references_in_expr(value, signatures);
            }
            HirExprKind::Literal(_)
            | HirExprKind::Local(_)
            | HirExprKind::Global(_)
            // No expression-bearing fields: every field is bound explicitly
            // so a future expression operand fails to compile until covered.
            | HirExprKind::OptionNone
            | HirExprKind::Join { actor_name: _ }
            | HirExprKind::ChannelTryRecv { channel_name: _ }
            | HirExprKind::Yield => {}
        }
    }

    /// A spawn resolves only through the exact actor CallableId it carries,
    /// which must index the frozen Function `__actor_<name>` whose shape
    /// accepts the spawn arguments and returns Unit.
    fn check_spawn_target(
        &mut self,
        actor_name: &str,
        actor_callable: CallableId,
        args: &[HirExpr],
        span: Span,
        signatures: &CallableSignatures,
    ) {
        let actor_symbol = format!("__actor_{actor_name}");
        let Some(signature) = signatures.get(&actor_callable) else {
            self.push_callable_metadata_error(
                span,
                format!(
                    "Ralat: metadata callable actor {actor_callable:?} hilang untuk spawn `{actor_name}`"
                ),
                format!(
                    "Error: frozen actor callable {actor_callable:?} is missing for spawn `{actor_name}`"
                ),
            );
            return;
        };
        let unit = self.types.primitive_ids().unit;
        let shape_matches = signature.callable == actor_callable
            && signature.kind == HirCallableKind::Function
            && signature.name == actor_symbol
            && signature.params.len() == args.len()
            && signature
                .params
                .iter()
                .zip(args)
                .all(|(expected, actual)| *expected == actual.ty.id)
            && signature.return_type == unit
            && !signature.is_variadic
            && signature.value_use == HirCallableValueUse::StatementOnly;
        if !shape_matches {
            self.push_callable_metadata_error(
                span,
                format!(
                    "Ralat: spawn `{actor_name}` tidak sepadan dengan callable actor beku {actor_callable:?}"
                ),
                format!(
                    "Error: spawn `{actor_name}` does not match its frozen actor callable {actor_callable:?}"
                ),
            );
        }
    }

    fn push_missing_callable_metadata(&mut self, callee: CallableId, span: Span) {
        self.push_callable_metadata_error(
            span,
            format!("Ralat: metadata callable hilang untuk rujukan {callee:?}"),
            format!("Error: callable metadata is missing for reference {callee:?}"),
        );
    }

    fn push_callable_metadata_error(&mut self, span: Span, message_ms: String, message_en: String) {
        self.push_error(
            DiagnosticCode::CallableMetadataIntegrity,
            span,
            message_ms,
            message_en,
        );
    }

    fn check_item(&mut self, item: &Spanned<HirItem>, signatures: &CallableSignatures) {
        match &item.node {
            HirItem::Function(function) => {
                let previous_safety = self.safety_context;
                let previous_return_type = self.current_return_type;
                self.safety_context = function.safety;
                self.current_return_type = Some(function.return_type.id);
                self.check_block(&function.body, signatures);
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

    fn check_block(&mut self, block: &HirBlock, signatures: &CallableSignatures) {
        for stmt in &block.statements {
            self.check_statement(stmt, signatures);
        }
    }

    fn check_statement(&mut self, stmt: &Spanned<HirStmt>, signatures: &CallableSignatures) {
        match &stmt.node {
            HirStmt::Let { ty, value, .. } => {
                if let Some(value) = value {
                    self.check_expression(value, signatures);
                    if !self.is_unit_type(ty.id) {
                        self.check_call_result_used_as_value(
                            value, stmt.span, "binding", signatures,
                        );
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
                self.check_expression(target, signatures);
                self.check_expression(value, signatures);
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
                self.check_expression(condition, signatures);
                self.check_condition_type(condition, stmt.span, "if");
                self.check_block(then_branch, signatures);
                if let Some(else_branch) = else_branch {
                    self.check_block(else_branch, signatures);
                }
            }
            HirStmt::While { condition, body } => {
                self.check_expression(condition, signatures);
                self.check_condition_type(condition, stmt.span, "while");
                self.loop_depth += 1;
                self.check_block(body, signatures);
                self.loop_depth -= 1;
            }
            HirStmt::Loop { body } => {
                self.loop_depth += 1;
                self.check_block(body, signatures);
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
                self.check_block(block, signatures);
                self.safety_context = previous;
            }
            HirStmt::HardwareZone(block) => {
                // MMIO zone: inherently unsafe context (volatile hardware access).
                let previous = self.safety_context;
                self.safety_context = SafetyContext::Unsafe;
                self.check_block(block, signatures);
                self.safety_context = previous;
            }
            HirStmt::HardwareDecl { .. } => {
                // Hardware register declaration: no sub-expressions to check.
            }
            HirStmt::Expr(expr) => self.check_expression(expr, signatures),
            HirStmt::Return(expr) => match expr {
                Some(expr) => {
                    self.check_expression(expr, signatures);
                    if let Some(expected) = self.current_return_type {
                        if !self.is_unit_type(expected) {
                            self.check_call_result_used_as_value(
                                expr, stmt.span, "return", signatures,
                            );
                        }
                        let actual = self.expression_effective_type(expr);
                        // A Unit function returning a value owns one
                        // Unit-specific diagnostic; the generic mismatch below
                        // would only duplicate it.
                        let unit_return_value =
                            self.is_unit_type(expected) && !self.is_unit_type(actual);

                        if unit_return_value {
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

                        if !unit_return_value && !self.types_compatible(expected, actual) {
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

    fn check_expression(&mut self, expr: &HirExpr, signatures: &CallableSignatures) {
        match &expr.kind {
            HirExprKind::Literal(_) | HirExprKind::Local(_) | HirExprKind::Global(_) => {}
            HirExprKind::Binary { left, op, right } => {
                self.check_expression(left, signatures);
                self.check_expression(right, signatures);
                self.check_binary_operator(op, left, right, expr.span);
            }
            HirExprKind::Unary { op, expr: operand } => {
                self.check_expression(operand, signatures);
                self.check_unary_operator(op, operand, expr.span);
            }
            HirExprKind::Field {
                base, field_name, ..
            } => {
                self.check_expression(base, signatures);
                self.check_field_access(base, field_name, expr.span);
            }
            HirExprKind::Cast {
                expr: operand,
                target,
            } => {
                self.check_expression(operand, signatures);
                self.check_cast_expression(operand, *target, expr.span);
            }
            HirExprKind::Index { base, index } => {
                self.check_expression(base, signatures);
                self.check_expression(index, signatures);
                self.check_index_expression(base, index, expr.span);
            }
            HirExprKind::ArrayLiteral { elements } => {
                for element in elements {
                    self.check_expression(element, signatures);
                }
                self.check_array_literal_elements(elements, expr.span);
            }
            HirExprKind::Call { callee, args } => {
                for arg in args {
                    self.check_expression(arg, signatures);
                }

                // Callable semantics come only from frozen metadata. A plain
                // HirModule has none, so its calls fail closed here.
                let Some(callable) = signatures.get(callee).cloned() else {
                    self.push_missing_callable_metadata(*callee, expr.span);
                    return;
                };
                // Arity follows the frozen signature: a variadic callable
                // requires at least its fixed prefix and permits trailing
                // arguments; any other callable requires exact arity. Every
                // fixed-prefix argument is type-checked.
                let expected_params = callable.params.clone();
                let name = callable.name.clone();
                let wrong_arity = if callable.is_variadic {
                    args.len() < expected_params.len()
                } else {
                    args.len() != expected_params.len()
                };
                if wrong_arity {
                    let (expected_ms, expected_en) = if callable.is_variadic {
                        (
                            format!("sekurang-kurangnya {}", expected_params.len()),
                            format!("at least {}", expected_params.len()),
                        )
                    } else {
                        (
                            expected_params.len().to_string(),
                            expected_params.len().to_string(),
                        )
                    };
                    self.push_error(
                        DiagnosticCode::TypeMismatch,
                        expr.span,
                        format!(
                            "Ralat: Bilangan argumen fungsi `{name}` tidak sepadan: dijangka {expected_ms}, diterima {}",
                            args.len()
                        ),
                        format!(
                            "Error: Function `{name}` argument count mismatch: expected {expected_en}, got {}",
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
                                signatures,
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

                // Only a frozen UserExtern has foreign-call provenance. Function,
                // Constructor, and Builtin callables are language callables and
                // never become FFI calls, even when their name collides with a
                // CallableRegistry entry.
                if callable.kind == HirCallableKind::UserExtern {
                    self.check_frozen_extern_call(&callable, args, expr.span);
                }
            }
            HirExprKind::Spawn {
                actor_name,
                actor_callable,
                args,
            } => {
                for arg in args {
                    self.check_expression(arg, signatures);
                }
                self.check_spawn_target(actor_name, *actor_callable, args, expr.span, signatures);
            }
            HirExprKind::ResultOk { value }
            | HirExprKind::ResultErr { value }
            | HirExprKind::OptionSome { value }
            | HirExprKind::ChannelTrySend {
                channel_name: _,
                value,
            } => self.check_expression(value, signatures),
            HirExprKind::ChannelCreate { capacity } => self.check_expression(capacity, signatures),
            HirExprKind::ChannelRecv { channel } => self.check_expression(channel, signatures),
            HirExprKind::ChannelSend { channel, value } => {
                self.check_expression(channel, signatures);
                self.check_expression(value, signatures);
            }
            HirExprKind::Sleep { duration_ms } => self.check_expression(duration_ms, signatures),
            HirExprKind::ChannelTimeoutRecv {
                channel_name: _,
                timeout_ms,
            } => self.check_expression(timeout_ms, signatures),
            // No expression-bearing fields (bound explicitly, see above).
            HirExprKind::OptionNone
            | HirExprKind::Join { actor_name: _ }
            | HirExprKind::ChannelTryRecv { channel_name: _ }
            | HirExprKind::Yield => {}
        }
    }

    /// Apply the frozen UserExtern policy to a foreign call. Frozen language
    /// metadata is authoritative for safety (UnsafeRequired), arity, argument
    /// types, and capability. A same-named FFI registry entry is binding
    /// information only: it must agree with the frozen declaration or the call
    /// is rejected, and its own safety/arity/policy never replaces the frozen
    /// policy.
    fn check_frozen_extern_call(
        &mut self,
        callable: &HirCallableSignature,
        args: &[HirExpr],
        span: Span,
    ) {
        let name = callable.name.as_str();
        let frozen = crate::ffi::CallableSignature {
            name: callable.name.clone(),
            params: callable.params.clone(),
            return_type: callable.return_type,
            abi: crate::ffi::CallingConvention::C,
            safety: crate::ffi::CallableSafety::UnsafeRequired,
            is_extern: true,
            is_variadic: callable.is_variadic,
        };
        // The HIR Call.callee is a language CallableId whose id-space is
        // independent of the FFI CallableRegistry, so the registry is consulted
        // by the frozen extern's NAME only, and only to validate the binding.
        if let Some((_, binding)) = self.callables.find_by_name(name) {
            let compatible = binding.is_extern
                && binding.params == frozen.params
                && binding.return_type == frozen.return_type
                && binding.is_variadic == frozen.is_variadic
                && binding.abi == frozen.abi;
            if !compatible {
                self.diagnostics.push(Diagnostic {
                    code: DiagnosticCode::FfiBoundaryViolation,
                    severity: Severity::Error,
                    message_ms: format!(
                        "Ralat: binding FFI '{name}' tidak serasi dengan deklarasi extern beku"
                    ),
                    message_en: format!(
                        "Error: FFI binding '{name}' is incompatible with its frozen extern declaration"
                    ),
                    primary_span: span,
                    notes: Vec::new(),
                });
                return;
            }
        }
        let gate = FfiGatekeeper {
            types: &self.types,
            callables: Some(&self.callables),
            policy: Some(&self.policy),
        };
        if let Err(diagnostic) = gate.validate_call(&frozen, args, self.safety_context, span) {
            self.diagnostics.push(diagnostic);
        }
    }

    /// Call result stamps equal the frozen callable return type: lowering
    /// stamps them from the frozen table, and `check_lowered_module` rejects a
    /// stale stamp before this is consulted.
    fn expression_effective_type(&self, expr: &HirExpr) -> crate::types::TypeId {
        expr.ty.id
    }

    fn check_call_result_used_as_value(
        &mut self,
        expr: &HirExpr,
        span: Span,
        context: &str,
        signatures: &CallableSignatures,
    ) {
        let HirExprKind::Call { callee, .. } = &expr.kind else {
            return;
        };
        let frozen = signatures.get(callee);
        let name = frozen
            .map(|signature| signature.name.clone())
            .unwrap_or_else(|| "<unknown>".to_string());
        let return_type = frozen
            .map(|signature| signature.return_type)
            .unwrap_or(expr.ty.id);

        if self.is_unit_type(return_type) {
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
        } else if self.is_unknown_type(return_type) {
            let metadata_missing = frozen.is_none();

            self.push_error(
                DiagnosticCode::TypeMismatch,
                span,
                if metadata_missing {
                    format!(
                        "Ralat: Jenis hasil panggilan `{name}` tidak dapat diselesaikan untuk digunakan sebagai nilai dalam {context}"
                    )
                } else {
                    format!(
                        "Ralat: Jenis hasil panggilan `{name}` tidak diketahui untuk digunakan sebagai nilai dalam {context}"
                    )
                },
                if metadata_missing {
                    format!(
                        "Error: Call `{name}` result type could not be resolved for value use in {context}"
                    )
                } else {
                    format!(
                        "Error: Call `{name}` result type is unknown for value use in {context}"
                    )
                },
            );
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
                callable: CallableId(8),
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
                declaration_param_enums: Vec::new(),
                declaration_return_enum: None,
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
                callable: CallableId(8),
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
                declaration_param_enums: Vec::new(),
                declaration_return_enum: None,
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
                callable: CallableId(8),
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
                declaration_param_enums: Vec::new(),
                declaration_return_enum: None,
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
                callable: CallableId(8),
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
                declaration_param_enums: Vec::new(),
                declaration_return_enum: None,
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
                callable: CallableId(8),
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
                declaration_param_enums: Vec::new(),
                declaration_return_enum: None,
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
                callable: CallableId(8),
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
                declaration_param_enums: Vec::new(),
                declaration_return_enum: None,
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
        // The language CallableId is independent of the FFI registry identity;
        // the gate routes the frozen callee name to the registry binding.
        let callee = CallableId(7);
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
                callable: CallableId(8),
                name: "test".to_string(),
                symbol: SymbolId(0),
                params: Vec::new(),
                return_type: TypeRef { id: ids.unit },
                body: HirBlock {
                    statements: vec![spanned(HirStmt::Expr(call))],
                },
                safety: SafetyContext::Safe,
                declaration_param_enums: Vec::new(),
                declaration_return_enum: None,
            }))],
        };

        let artifact = ffi_test_artifact(module, callee, ids);
        let diagnostics = ctx
            .check_lowered_module(&artifact)
            .expect_err("unsafe ffi call must fail");
        assert_eq!(diagnostics[0].code, DiagnosticCode::FfiBoundaryViolation);
    }

    #[test]
    fn ffi_call_inside_unsafe_block_succeeds() {
        let mut ctx = base_context();
        let ids = ctx.types.primitive_ids();
        let _ffi_callable = ctx.callables.register(crate::ffi::CallableSignature {
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
                callee: CallableId(7),
                args: vec![expr_i64(&ctx.types, 1)],
            },
            ty: TypeRef { id: ids.i32_ },
            span: Span::unknown(),
        };
        let module = HirModule {
            items: vec![spanned(HirItem::Function(HirFunction {
                callable: CallableId(8),
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
                declaration_param_enums: Vec::new(),
                declaration_return_enum: None,
            }))],
        };

        // A declared user extern must also pass the capability policy.
        ctx.policy.allow_symbol("puts");
        let artifact = ffi_test_artifact(module, CallableId(7), ids);
        let result = ctx.check_lowered_module(&artifact);
        assert!(result.is_ok(), "unexpected diagnostics: {result:?}");
    }

    /// Wrap a single unit `test` function calling extern `puts` in a canonical
    /// lowering artifact with exact frozen metadata for both callables.
    fn ffi_test_artifact(
        mut module: HirModule,
        puts: CallableId,
        ids: crate::types::PrimitiveTypeIds,
    ) -> LoweredModule {
        module.items.insert(
            0,
            spanned(HirItem::ExternFunction(crate::hir::HirExternFunction {
                callable: puts,
                name: "puts".to_string(),
                params: vec![ids.i64_],
                return_type: ids.i32_,
                is_variadic: false,
            })),
        );
        let test = CallableId(8);
        let signature = |callable, name: &str, kind, params: Vec<_>, return_type, value_use| {
            HirCallableSignature {
                callable,
                name: name.to_string(),
                kind,
                param_enums: vec![None; params.len()],
                params,
                return_type,
                return_enum: None,
                is_variadic: false,
                value_use,
            }
        };
        LoweredModule {
            module,
            callable_signatures: HashMap::from([
                (
                    puts,
                    signature(
                        puts,
                        "puts",
                        HirCallableKind::UserExtern,
                        vec![ids.i64_],
                        ids.i32_,
                        HirCallableValueUse::Value,
                    ),
                ),
                (
                    test,
                    signature(
                        test,
                        "test",
                        HirCallableKind::Function,
                        Vec::new(),
                        ids.unit,
                        HirCallableValueUse::StatementOnly,
                    ),
                ),
            ]),
            builtin_provenance: HashMap::new(),
        }
    }

    fn hir_function_returning_i64(
        ctx: &super::SemanticContext,
        expr: crate::hir::HirExpr,
    ) -> crate::hir::HirModule {
        crate::hir::HirModule {
            items: vec![spanned(crate::hir::HirItem::Function(
                crate::hir::HirFunction {
                    callable: CallableId(8),
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
                    declaration_param_enums: Vec::new(),
                    declaration_return_enum: None,
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
// Public API for semantic gatekeeper validation
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

/// Defensive revalidation of the canonical lowering artifact with the same
/// FFI registry and capability policy used by the production semantic check.
/// Codegen calls this immediately before LLVM construction; it does not create
/// a second Meaning Authority.
pub fn validate_lowered_module(
    lowered: &LoweredModule,
    types: TypeRegistry,
    callables: CallableRegistry,
    policy: crate::ffi::CapabilityPolicy,
) -> Result<(), Vec<Diagnostic>> {
    let mut ctx = SemanticContext {
        types,
        symbols: SymbolTable::default(),
        callables,
        diagnostics: Vec::new(),
        loop_depth: 0,
        safety_context: SafetyContext::Safe,
        current_return_type: None,
        policy,
        extern_symbols: std::collections::HashSet::new(),
    };
    ctx.check_lowered_module(lowered)
}

/// Run the semantic gatekeeper and print any diagnostics.
/// Returns true if validation passed.
pub fn validate_module_with_reporting(module: &HirModule, types: TypeRegistry) -> bool {
    match validate_module(module, types) {
        Ok(()) => {
            println!("logicodex: Semantic gatekeeper validation passed");
            true
        }
        Err(diagnostics) => {
            eprintln!(
                "logicodex: Semantic gatekeeper found {} issue(s):",
                diagnostics.len()
            );
            for d in &diagnostics {
                eprintln!("  [{:?}] {}", d.severity, d.message_en);
            }
            false
        }
    }
}
