// =========================================================================
// Logicodex v1.30 — Type Checker (CoercionEngine Integration)
// Sprint 1.2: Parser Type Injection
//
// Bridges the semantic analyzer (which uses ast::Type) with the
// TypeRegistry and CoercionEngine (which use TypeId).
//
// Provides stricter type checking than the legacy types_compatible()
// function, with detailed bilingual error messages.
// =========================================================================

use crate::ast::{Expr, Type};
use crate::semantic::coercion::{CoercionEngine, CoercionResult};
use crate::semantic::registry::TypeInspector;
use crate::types::{Mutability, PrimitiveType, TypeId, TypeKind, TypeRegistry};

/// Enhanced type checker that uses CoercionEngine for validation.
/// Provides detailed error messages explaining why a coercion failed.
pub struct TypeChecker<'a> {
    registry: &'a TypeRegistry,
    engine: CoercionEngine<'a>,
    inspector: TypeInspector<'a>,
}

/// Detailed result of a type check, including diagnostic information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeCheckResult {
    /// Types are compatible — no action needed.
    Ok,
    /// Widening conversion (safe, implicit).
    ImplicitWidening { from: String, to: String },
    /// Narrowing conversion — requires explicit cast.
    RequiresExplicitCast { from: String, to: String, suggestion: String },
    /// Incompatible types — no valid conversion.
    Incompatible { from: String, to: String, reason: String },
}

impl TypeChecker<'_> {
    pub fn new(registry: &TypeRegistry) -> TypeChecker<'_> {
        TypeChecker {
            registry,
            engine: CoercionEngine::new(registry),
            inspector: TypeInspector::new(registry),
        }
    }

    /// Convert an ast::Type to a TypeId in the registry.
    fn ast_type_to_id(&self, ast_type: &Type) -> TypeId {
        match ast_type {
            Type::I32 => self.registry.primitive(PrimitiveType::I32),
            Type::I64 => self.registry.primitive(PrimitiveType::I64),
            Type::U16 => self.registry.primitive(PrimitiveType::U16),
            Type::U32 => self.registry.primitive(PrimitiveType::U32),
            Type::F64 => self.registry.primitive(PrimitiveType::F64),
            Type::Bool => self.registry.primitive(PrimitiveType::Bool),
            Type::String => self.registry.primitive(PrimitiveType::String),
            Type::Pointer(inner) => {
                let pointee = self.ast_type_to_id(inner);
                // Look up existing pointer type in registry by scanning kinds
                // (O(n) but n is small for the primitive set)
                let ids = self.registry.primitive_ids();
                let all_ids = [
                    ids.bool_, ids.i8_, ids.i16_, ids.i32_, ids.i64_,
                    ids.u8_, ids.u16_, ids.u32_, ids.u64_,
                    ids.f32_, ids.f64_, ids.string, ids.unit,
                    ids.never, ids.unknown,
                ];
                all_ids
                    .iter()
                    .find(|&&id| {
                        matches!(
                            self.registry.get(id),
                            Some(TypeKind::Pointer {
                                pointee: p,
                                mutability: Mutability::Immutable,
                            }) if *p == pointee
                        )
                    })
                    .copied()
                    .unwrap_or_else(|| {
                        // Pointer type not interned — return synthetic ID.
                        // This is a best-effort for pointer types not yet interned.
                        // The caller should intern the type if precise identity is needed.
                        TypeId(2000 + pointee.0)
                    })
            }
        }
    }

    /// Check if a value of type `actual` can be assigned to a variable
    /// of type `declared`. Returns a detailed result with diagnostic info.
    pub fn check_assignment(&self, declared: &Type, actual: &Type) -> TypeCheckResult {
        let declared_id = self.ast_type_to_id(declared);
        let actual_id = self.ast_type_to_id(actual);
        let coercion = self.engine.can_coerce(actual_id, declared_id);

        match coercion {
            CoercionResult::Identity => TypeCheckResult::Ok,
            CoercionResult::Implicit { .. } => {
                let from_name = self.inspector.type_name(actual_id);
                let to_name = self.inspector.type_name(declared_id);
                TypeCheckResult::ImplicitWidening {
                    from: from_name,
                    to: to_name,
                }
            }
            CoercionResult::RequiresCast { .. } => {
                let from_name = self.inspector.type_name(actual_id);
                let to_name = self.inspector.type_name(declared_id);
                let suggestion = self.suggest_cast(actual, declared);
                TypeCheckResult::RequiresExplicitCast {
                    from: from_name,
                    to: to_name,
                    suggestion,
                }
            }
            CoercionResult::Incompatible => {
                let from_name = self.inspector.type_name(actual_id);
                let to_name = self.inspector.type_name(declared_id);
                let reason = self.explain_incompatibility(actual, declared);
                TypeCheckResult::Incompatible {
                    from: from_name,
                    to: to_name,
                    reason,
                }
            }
        }
    }

    /// Quick check: are these types compatible (no cast needed)?
    pub fn is_compatible(&self, declared: &Type, actual: &Type) -> bool {
        matches!(
            self.check_assignment(declared, actual),
            TypeCheckResult::Ok | TypeCheckResult::ImplicitWidening { .. }
        )
    }

    /// Quick check: is an explicit cast required?
    pub fn needs_cast(&self, declared: &Type, actual: &Type) -> bool {
        matches!(
            self.check_assignment(declared, actual),
            TypeCheckResult::RequiresExplicitCast { .. }
        )
    }

    /// Get the default type for an untyped variable based on its initializer.
    /// Returns None for expressions that cannot be typed without context.
    ///
    /// BINA x = 1      → Some(I64)  (default integer type)
    /// BINA x = 3.14   → Some(F64)  (default float type)
    /// BINA x = benar  → Some(Bool)
    /// BINA x = "hi"   → Some(String)
    /// BINA x = y      → None (need context — y's type unknown without symbol table)
    pub fn infer_default_type(&self, value: &Expr) -> Option<Type> {
        match value {
            Expr::IntegerLiteral(_) | Expr::HexLiteral(_) => Some(Type::I64),
            Expr::FloatLiteral(_) => Some(Type::F64),
            Expr::BooleanLiteral(_) => Some(Type::Bool),
            Expr::StringLiteral(_) => Some(Type::String),
            Expr::Binary { left, op: _, right } => {
                // Infer from operands
                let left_ty = self.infer_default_type(left)?;
                let right_ty = self.infer_default_type(right)?;
                let left_id = self.ast_type_to_id(&left_ty);
                let right_id = self.ast_type_to_id(&right_ty);
                self.engine
                    .common_type(left_id, right_id)
                    .and_then(|id| self.registry.type_id_to_ast(id))
            }
            Expr::Unary { expr, .. } => self.infer_default_type(expr),
            Expr::Call { callee, .. } => {
                // Try to resolve callee as a struct type name → returns that type
                // e.g., Color(255, 0, 0, 255) → Color
                if let Expr::Variable(name) = callee.as_ref() {
                    // Check if name matches a registered struct type
                    if let Some((_, layout)) = self.registry.find_struct_by_name(name) {
                        // Return the type name as ast::Type if possible
                        // For Sprint 2.5, we only support primitive-named structs
                        // Full struct type mapping requires ast::Type extension
                        return None; // Will be refined in Sprint 3
                    }
                }
                None
            }
            // Complex expressions need symbol table context
            _ => None,
        }
    }

    /// Check a function or struct constructor call.
    /// Returns the result type of the call, or an error description.
    pub fn check_call(&self, callee: &Expr, args: &[Expr]) -> Result<Type, String> {
        let callee_name = match callee {
            Expr::Variable(name) => name.as_str(),
            _ => return Err("Complex callees not yet supported (Sprint 3)".into()),
        };

        // Try to resolve as a struct type → struct constructor
        if let Some((_, layout)) = self.registry.find_struct_by_name(callee_name) {
            // Struct constructor: validate arg count matches field count
            if args.len() != layout.fields.len() {
                return Err(format!(
                    "Struct constructor '{}' expects {} arguments, got {}. \
                     / Pembina struktur '{}' memerlukan {} argumen, mendapat {}.",
                    callee_name, layout.fields.len(), args.len(),
                    callee_name, layout.fields.len(), args.len(),
                ));
            }
            // Sprint 3: validate each argument type against field type
            // v1.38: Struct constructors return a packed value (I64 for Color RGBA)
            // This is intentional — structs are value types packed into integer registers
            return Ok(Type::I64);
        }

        // Try CallableRegistry for function calls
        // Sprint 3: integrate with CallableRegistry for full function checking
        Err(format!(
            "Fungsi atau jenis struktur '{}' tidak dikenali. \
             / Function or struct type '{}' is not recognized.",
            callee_name, callee_name
        ))
    }

    /// Generate a bilingual error message for a type mismatch.
    /// Returns (malay_message, english_message).
    pub fn format_error(
        &self,
        name: &str,
        result: &TypeCheckResult,
    ) -> (String, String) {
        match result {
            TypeCheckResult::Ok | TypeCheckResult::ImplicitWidening { .. } => {
                (String::new(), String::new()) // no error
            }
            TypeCheckResult::RequiresExplicitCast { from, to, suggestion } => (
                format!(
                    "Ralat: Pembolehubah '{}' diisytihar sebagai {} tetapi nilai merupakan {}. Penukaran ini memerlukan cast eksplisit: {}",
                    name, to, from, suggestion
                ),
                format!(
                    "Error: Variable '{}' is declared as {} but the value is {}. This conversion requires an explicit cast: {}",
                    name, to, from, suggestion
                ),
            ),
            TypeCheckResult::Incompatible { from, to, reason } => (
                format!(
                    "Ralat: Pembolehubah '{}' diisytihar sebagai {} tetapi nilai merupakan {}. {}",
                    name, to, from, reason
                ),
                format!(
                    "Error: Variable '{}' is declared as {} but the value is {}. {}",
                    name, to, from, reason
                ),
            ),
        }
    }

    // ─── Internal helpers ───

    fn suggest_cast(&self, from: &Type, to: &Type) -> String {
        format!("guna 'sebagai {:?}' / use 'as {:?}'", to, to)
    }

    fn explain_incompatibility(&self, from: &Type, to: &Type) -> String {
        match (from, to) {
            (Type::Bool, Type::Bool) | (Type::Bool, _) | (_, Type::Bool) => {
                "Jenis Boolean tidak boleh ditukar dengan jenis lain / Boolean types cannot be converted to other types".to_string()
            }
            (Type::String, Type::String) | (Type::String, _) | (_, Type::String) => {
                "Jenis String tidak boleh ditukar secara langsung / String types cannot be directly converted".to_string()
            }
            (Type::Pointer(_), _) | (_, Type::Pointer(_)) => {
                "Jenis pointer memerlukan kebenaran provenance yang sah / Pointer types require valid provenance".to_string()
            }
            (Type::F64, Type::I32 | Type::I64 | Type::U16 | Type::U32) => {
                "Penukaran float ke integer memerlukan cast eksplisit / Float to integer conversion requires explicit cast".to_string()
            }
            (Type::I64 | Type::I32, Type::F64) if from != to => {
                format!("Penukaran {:?} ke {:?} memerlukan 'as {:?}' / {:?} to {:?} conversion requires 'as {:?}'", from, to, to, from, to, to)
            }
            _ => "Jenis tidak serasi / Types are incompatible".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Expr;

    // Helper: create a checker for testing.
    // Uses Box::leak to extend lifetime to 'static for test convenience.
    // This is acceptable in tests (small, bounded leak).
    fn make_checker() -> TypeChecker<'static> {
        let registry: &'static TypeRegistry =
            Box::leak(Box::new(TypeRegistry::new()));
        TypeChecker::new(registry)
    }

    #[test]
    fn i32_to_i64_is_widening() {
        let checker = make_checker();
        let result = checker.check_assignment(&Type::I64, &Type::I32);
        assert!(
            matches!(result, TypeCheckResult::ImplicitWidening { .. }),
            "I32 -> I64 should be implicit widening, got {:?}",
            result
        );
    }

    #[test]
    fn i64_to_i32_requires_cast() {
        let checker = make_checker();
        let result = checker.check_assignment(&Type::I32, &Type::I64);
        assert!(
            matches!(result, TypeCheckResult::RequiresExplicitCast { .. }),
            "I64 -> I32 should require cast, got {:?}",
            result
        );
    }

    #[test]
    fn f64_to_i32_requires_cast() {
        let checker = make_checker();
        let result = checker.check_assignment(&Type::I32, &Type::F64);
        assert!(
            matches!(result, TypeCheckResult::RequiresExplicitCast { .. }),
            "F64 -> I32 should require cast"
        );
    }

    #[test]
    fn bool_to_numeric_is_incompatible() {
        let checker = make_checker();
        let result = checker.check_assignment(&Type::I64, &Type::Bool);
        assert!(
            matches!(result, TypeCheckResult::Incompatible { .. }),
            "Bool -> I64 should be incompatible"
        );
    }

    #[test]
    fn infer_integer_literal_is_i64() {
        let checker = make_checker();
        let expr = Expr::IntegerLiteral(42);
        assert_eq!(checker.infer_default_type(&expr), Some(Type::I64));
    }

    #[test]
    fn infer_float_literal_is_f64() {
        let checker = make_checker();
        let expr = Expr::FloatLiteral(3.14);
        assert_eq!(checker.infer_default_type(&expr), Some(Type::F64));
    }

    #[test]
    fn infer_string_literal_is_string() {
        let checker = make_checker();
        let expr = Expr::StringLiteral("hello".to_string());
        assert_eq!(checker.infer_default_type(&expr), Some(Type::String));
    }

    #[test]
    fn infer_boolean_literal_is_bool() {
        let checker = make_checker();
        let expr = Expr::BooleanLiteral(true);
        assert_eq!(checker.infer_default_type(&expr), Some(Type::Bool));
    }

    #[test]
    fn infer_variable_returns_none() {
        // BINA x = y → cannot infer without symbol table
        let checker = make_checker();
        let expr = Expr::Variable("y".to_string());
        assert_eq!(checker.infer_default_type(&expr), None);
    }

    #[test]
    fn error_message_contains_both_languages() {
        let checker = make_checker();
        let result = checker.check_assignment(&Type::I32, &Type::I64);
        let (ms, en) = checker.format_error("x", &result);
        assert!(ms.contains("Ralat"), "Malay error should contain 'Ralat'");
        assert!(en.contains("Error"), "English error should contain 'Error'");
        assert!(ms.contains("x"), "Error should mention variable name");
        assert!(en.contains("x"), "Error should mention variable name");
    }

    #[test]
    fn cast_suggestion_includes_as_syntax() {
        let checker = make_checker();
        let result = checker.check_assignment(&Type::I32, &Type::I64);
        let (_, en) = checker.format_error("x", &result);
        assert!(
            en.contains("as"),
            "Cast suggestion should mention 'as' syntax: {}",
            en
        );
    }
}
