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

use crate::ast::Type;
use crate::semantic::coercion::{CoercionEngine, CoercionResult};
use crate::semantic::registry::TypeInspector;
use crate::types::TypeRegistry;

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

    /// Check if a value of type `actual` can be assigned to a variable
    /// of type `declared`. Returns a detailed result with diagnostic info.
    pub fn check_assignment(&self, declared: &Type, actual: &Type) -> TypeCheckResult {
        let declared_id = self.registry.ast_type_to_id(declared);
        let actual_id = self.registry.ast_type_to_id(actual);
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
    /// BINA x = 1      → I64 (default integer type)
    /// BINA x = 3.14   → F64 (default float type)
    /// BINA x = benar  → Bool
    /// BINA x = "hi"   → String
    pub fn infer_default_type(&self, value: &crate::ast::Expr) -> Type {
        use crate::ast::Expr;
        match value {
            Expr::IntegerLiteral(_) | Expr::HexLiteral(_) => Type::I64,
            Expr::FloatLiteral(_) => Type::F64,
            Expr::BooleanLiteral(_) => Type::Bool,
            Expr::StringLiteral(_) => Type::String,
            Expr::Binary { left, op, right } => {
                // Infer from operands
                let left_ty = self.infer_default_type(left);
                let right_ty = self.infer_default_type(right);
                // Use common type if available, else I64
                let left_id = self.registry.ast_type_to_id(&left_ty);
                let right_id = self.registry.ast_type_to_id(&right_ty);
                self.engine
                    .common_type(left_id, right_id)
                    .and_then(|id| self.registry.type_id_to_ast(id))
                    .unwrap_or(Type::I64)
            }
            Expr::Unary { expr, .. } => self.infer_default_type(expr),
            _ => Type::I64, // fallback for complex expressions
        }
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
        format!("guna 'sebagai {}' / use 'as {:?}'", to, to)
    }

    fn explain_incompatibility(&self, from: &Type, to: &Type) -> String {
        use crate::ast::Type;
        match (from, to) {
            (Type::Bool, _) | (_, Type::Bool) => {
                "Jenis Boolean tidak boleh ditukar dengan jenis numerik / Boolean types cannot be converted to numeric types".to_string()
            }
            (Type::String, _) | (_, Type::String) => {
                "Jenis String tidak boleh ditukar secara langsung / String types cannot be directly converted".to_string()
            }
            (Type::Pointer(_), _) | (_, Type::Pointer(_)) => {
                "Jenis pointer memerlukan kebenaran provenance yang sah / Pointer types require valid provenance".to_string()
            }
            _ => "Jenis tidak serasi / Types are incompatible".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Expr;

    #[test]
    fn i32_to_i64_is_widening() {
        let registry = TypeRegistry::new();
        let checker = TypeChecker::new(&registry);

        let result = checker.check_assignment(&Type::I64, &Type::I32);
        assert!(
            matches!(result, TypeCheckResult::ImplicitWidening { .. }),
            "I32 -> I64 should be implicit widening, got {:?}",
            result
        );
    }

    #[test]
    fn i64_to_i32_requires_cast() {
        let registry = TypeRegistry::new();
        let checker = TypeChecker::new(&registry);

        let result = checker.check_assignment(&Type::I32, &Type::I64);
        assert!(
            matches!(result, TypeCheckResult::RequiresExplicitCast { .. }),
            "I64 -> I32 should require cast, got {:?}",
            result
        );
    }

    #[test]
    fn f64_to_i32_requires_cast() {
        let registry = TypeRegistry::new();
        let checker = TypeChecker::new(&registry);

        let result = checker.check_assignment(&Type::I32, &Type::F64);
        assert!(
            matches!(result, TypeCheckResult::RequiresExplicitCast { .. }),
            "F64 -> I32 should require cast"
        );
    }

    #[test]
    fn bool_to_numeric_is_incompatible() {
        let registry = TypeRegistry::new();
        let checker = TypeChecker::new(&registry);

        let result = checker.check_assignment(&Type::I64, &Type::Bool);
        assert!(
            matches!(result, TypeCheckResult::Incompatible { .. }),
            "Bool -> I64 should be incompatible"
        );
    }

    #[test]
    fn infer_integer_literal_is_i64() {
        let registry = TypeRegistry::new();
        let checker = TypeChecker::new(&registry);

        let expr = Expr::IntegerLiteral(42);
        assert_eq!(checker.infer_default_type(&expr), Type::I64);
    }

    #[test]
    fn infer_float_literal_is_f64() {
        let registry = TypeRegistry::new();
        let checker = TypeChecker::new(&registry);

        let expr = Expr::FloatLiteral(3.14);
        assert_eq!(checker.infer_default_type(&expr), Type::F64);
    }

    #[test]
    fn infer_string_literal_is_string() {
        let registry = TypeRegistry::new();
        let checker = TypeChecker::new(&registry);

        let expr = Expr::StringLiteral("hello".to_string());
        assert_eq!(checker.infer_default_type(&expr), Type::String);
    }

    #[test]
    fn infer_boolean_literal_is_bool() {
        let registry = TypeRegistry::new();
        let checker = TypeChecker::new(&registry);

        let expr = Expr::BooleanLiteral(true);
        assert_eq!(checker.infer_default_type(&expr), Type::Bool);
    }

    #[test]
    fn error_message_contains_both_languages() {
        let registry = TypeRegistry::new();
        let checker = TypeChecker::new(&registry);

        let result = checker.check_assignment(&Type::I32, &Type::I64);
        let (ms, en) = checker.format_error("x", &result);

        assert!(ms.contains("Ralat"), "Malay error should contain 'Ralat'");
        assert!(en.contains("Error"), "English error should contain 'Error'");
        assert!(
            ms.contains("x"),
            "Error message should mention variable name"
        );
        assert!(
            en.contains("x"),
            "Error message should mention variable name"
        );
    }
}
