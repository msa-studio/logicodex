// =========================================================================
// Logicodex v1.30 — Parser Type Injection Tests
// Sprint 1.2: Typed Variable Declarations
//
// Tests that the parser correctly handles:
//   - Explicit type annotations: BINA x: I32 = 1
//   - Default type inference: BINA x = 1 → I64
//   - Coercion validation via TypeChecker
//   - Bilingual error messages
// =========================================================================

use logicodex::ast::{Expr, Type};
use logicodex::semantic::type_checker::{TypeCheckResult, TypeChecker};
use logicodex::types::TypeRegistry;

// ─── 1. Explicit Type Annotation Tests ───

#[test]
fn explicit_i32_annotation() {
    // BINA x: I32 = 1
    let declared = Type::I32;
    let actual = Type::I32; // literal 1 would be inferred as I64, but let's test exact match
    let registry = TypeRegistry::new();
    let checker = TypeChecker::new(&registry);

    let result = checker.check_assignment(&declared, &actual);
    assert!(
        matches!(result, TypeCheckResult::Ok),
        "I32 = I32 should be exact match"
    );
}

#[test]
fn explicit_i32_with_i64_value_requires_cast() {
    // BINA x: I32 = 100000
    // The literal 100000 would be I64 (default), assigning to I32 needs cast
    let declared = Type::I32;
    let actual = Type::I64;
    let registry = TypeRegistry::new();
    let checker = TypeChecker::new(&registry);

    let result = checker.check_assignment(&declared, &actual);
    assert!(
        matches!(result, TypeCheckResult::RequiresExplicitCast { .. }),
        "I64 -> I32 should require cast"
    );
}

#[test]
fn explicit_i64_annotation_accepts_i32_value() {
    // BINA x: I64 = 1
    // I32 value assigned to I64 variable — widening (implicit)
    let declared = Type::I64;
    let actual = Type::I32;
    let registry = TypeRegistry::new();
    let checker = TypeChecker::new(&registry);

    let result = checker.check_assignment(&declared, &actual);
    assert!(
        matches!(result, TypeCheckResult::ImplicitWidening { .. }),
        "I32 -> I64 should be implicit widening"
    );
}

#[test]
fn explicit_f64_annotation_accepts_integer() {
    // BINA x: F64 = 42
    // Integer literal assigned to F64 — implicit conversion
    let declared = Type::F64;
    let actual = Type::I64;
    let registry = TypeRegistry::new();
    let checker = TypeChecker::new(&registry);

    let result = checker.check_assignment(&declared, &actual);
    assert!(
        result.is_compatible(),
        "I64 -> F64 should be compatible (widening)"
    );
}

#[test]
fn explicit_i32_with_f64_value_is_incompatible() {
    // BINA x: I32 = 3.14
    // Float assigned to integer — incompatible without cast
    let declared = Type::I32;
    let actual = Type::F64;
    let registry = TypeRegistry::new();
    let checker = TypeChecker::new(&registry);

    let result = checker.check_assignment(&declared, &actual);
    assert!(
        matches!(result, TypeCheckResult::RequiresExplicitCast { .. }),
        "F64 -> I32 should require explicit cast"
    );
}

// ─── 2. Default Type Inference Tests ───

#[test]
fn infer_integer_literal_default_is_i64() {
    // BINA x = 42  →  x: I64
    let registry = TypeRegistry::new();
    let checker = TypeChecker::new(&registry);
    let expr = Expr::IntegerLiteral(42);

    assert_eq!(
        checker.infer_default_type(&expr),
        Some(Type::I64),
        "Untyped integer literal should default to I64"
    );
}

#[test]
fn infer_float_literal_default_is_f64() {
    // BINA x = 3.14  →  x: F64
    let registry = TypeRegistry::new();
    let checker = TypeChecker::new(&registry);
    let expr = Expr::FloatLiteral(3.14);

    assert_eq!(
        checker.infer_default_type(&expr),
        Some(Type::F64),
        "Untyped float literal should default to F64"
    );
}

#[test]
fn infer_string_literal_default_is_string() {
    // BINA x = "hello"  →  x: String
    let registry = TypeRegistry::new();
    let checker = TypeChecker::new(&registry);
    let expr = Expr::StringLiteral("hello".to_string());

    assert_eq!(
        checker.infer_default_type(&expr),
        Some(Type::String),
        "Untyped string literal should default to String"
    );
}

#[test]
fn infer_boolean_literal_default_is_bool() {
    // BINA x = benar  →  x: Bool
    let registry = TypeRegistry::new();
    let checker = TypeChecker::new(&registry);
    let expr = Expr::BooleanLiteral(true);

    assert_eq!(
        checker.infer_default_type(&expr),
        Some(Type::Bool),
        "Untyped boolean literal should default to Bool"
    );
}

#[test]
fn infer_binary_op_with_integers_is_i64() {
    // BINA x = 1 + 2  →  x: I64
    let registry = TypeRegistry::new();
    let checker = TypeChecker::new(&registry);
    let expr = Expr::Binary {
        left: Box::new(Expr::IntegerLiteral(1)),
        op: logicodex::ast::BinaryOp::Add,
        right: Box::new(Expr::IntegerLiteral(2)),
    };

    assert_eq!(
        checker.infer_default_type(&expr),
        Some(Type::I64),
        "Binary op on integers should default to I64"
    );
}

#[test]
fn infer_binary_op_with_float_and_int_is_f64() {
    // BINA x = 1.5 + 2  →  x: F64 (common type)
    let registry = TypeRegistry::new();
    let checker = TypeChecker::new(&registry);
    let expr = Expr::Binary {
        left: Box::new(Expr::FloatLiteral(1.5)),
        op: logicodex::ast::BinaryOp::Add,
        right: Box::new(Expr::IntegerLiteral(2)),
    };

    assert_eq!(
        checker.infer_default_type(&expr),
        Some(Type::F64),
        "Binary op with float should default to F64"
    );
}

// ─── 3. Type Compatibility Tests ───

#[test]
fn same_types_are_compatible() {
    let registry = TypeRegistry::new();
    let checker = TypeChecker::new(&registry);

    assert!(checker.is_compatible(&Type::I32, &Type::I32));
    assert!(checker.is_compatible(&Type::I64, &Type::I64));
    assert!(checker.is_compatible(&Type::F64, &Type::F64));
    assert!(checker.is_compatible(&Type::Bool, &Type::Bool));
    assert!(checker.is_compatible(&Type::String, &Type::String));
}

#[test]
fn widening_is_compatible() {
    let registry = TypeRegistry::new();
    let checker = TypeChecker::new(&registry);

    // I32 → I64 (widening)
    assert!(checker.is_compatible(&Type::I64, &Type::I32));
    // I32 → F64 (int to float)
    assert!(checker.is_compatible(&Type::F64, &Type::I32));
}

#[test]
fn narrowing_needs_cast() {
    let registry = TypeRegistry::new();
    let checker = TypeChecker::new(&registry);

    // I64 → I32 (narrowing)
    assert!(checker.needs_cast(&Type::I32, &Type::I64));
    // F64 → I32 (float to int)
    assert!(checker.needs_cast(&Type::I32, &Type::F64));
}

#[test]
fn bool_to_numeric_is_incompatible() {
    let registry = TypeRegistry::new();
    let checker = TypeChecker::new(&registry);

    assert!(!checker.is_compatible(&Type::I64, &Type::Bool));
    assert!(!checker.is_compatible(&Type::F64, &Type::Bool));
    assert!(!checker.is_compatible(&Type::Bool, &Type::I64));
}

#[test]
fn string_to_numeric_is_incompatible() {
    let registry = TypeRegistry::new();
    let checker = TypeChecker::new(&registry);

    assert!(!checker.is_compatible(&Type::I64, &Type::String));
    assert!(!checker.is_compatible(&Type::String, &Type::I64));
}

// ─── 4. Bilingual Error Message Tests ───

#[test]
fn narrowing_error_is_bilingual() {
    let registry = TypeRegistry::new();
    let checker = TypeChecker::new(&registry);

    let result = checker.check_assignment(&Type::I32, &Type::I64);
    let (ms, en) = checker.format_error("skor", &result);

    assert!(
        ms.contains("Ralat"),
        "Malay error should contain 'Ralat': {}",
        ms
    );
    assert!(
        en.contains("Error"),
        "English error should contain 'Error': {}",
        en
    );
    assert!(ms.contains("skor"), "Error should mention variable name");
    assert!(en.contains("skor"), "Error should mention variable name");
}

#[test]
fn incompatible_error_is_bilingual() {
    let registry = TypeRegistry::new();
    let checker = TypeChecker::new(&registry);

    let result = checker.check_assignment(&Type::I64, &Type::Bool);
    let (ms, en) = checker.format_error("aktif", &result);

    assert!(
        matches!(result, TypeCheckResult::Incompatible { .. }),
        "Bool -> I64 should be incompatible"
    );
    assert!(ms.contains("Ralat"), "Malay error missing: {}", ms);
    assert!(en.contains("Error"), "English error missing: {}", en);
    assert!(
        ms.contains("aktif"),
        "Error should mention variable 'aktif'"
    );
}

#[test]
fn infer_unbound_variable_returns_none() {
    // BINA x = y → cannot infer y's type without symbol table
    let registry = TypeRegistry::new();
    let checker = TypeChecker::new(&registry);
    let expr = Expr::Variable("y".to_string());
    assert_eq!(checker.infer_default_type(&expr), None);
}

#[test]
fn infer_call_returns_none() {
    // BINA x = foo() → cannot infer without function signature
    let registry = TypeRegistry::new();
    let checker = TypeChecker::new(&registry);
    let expr = Expr::Call("foo".to_string(), vec![]);
    assert_eq!(checker.infer_default_type(&expr), None);
}

#[test]
fn cast_suggestion_includes_as_syntax() {
    let registry = TypeRegistry::new();
    let checker = TypeChecker::new(&registry);

    let result = checker.check_assignment(&Type::I32, &Type::I64);
    let (_, en) = checker.format_error("x", &result);

    assert!(
        en.contains("as"),
        "Cast suggestion should mention 'as' syntax: {}",
        en
    );
}

// ─── 5. AST Bridge Tests ───

#[test]
fn ast_type_to_id_roundtrip() {
    let registry = TypeRegistry::new();

    let ast_types = [Type::I32, Type::I64, Type::U16, Type::U32, Type::F64, Type::Bool, Type::String];

    for ast_ty in &ast_types {
        let id = registry.ast_type_to_id(ast_ty);
        let roundtrip = registry.type_id_to_ast(id);
        assert_eq!(
            roundtrip.as_ref(),
            Some(ast_ty),
            "Roundtrip failed for {:?}",
            ast_ty
        );
    }
}

#[test]
fn ast_types_compatible_uses_coercion_engine() {
    let registry = TypeRegistry::new();

    // I32 -> I64: widening, should be implicit
    let result = registry.ast_types_compatible(&Type::I64, &Type::I32);
    assert!(
        result.is_allowed(),
        "I32 -> I64 should be allowed via CoercionEngine"
    );

    // I64 -> I32: narrowing, should require cast
    let result = registry.ast_types_compatible(&Type::I32, &Type::I64);
    assert!(
        result.needs_cast(),
        "I64 -> I32 should require cast via CoercionEngine"
    );
}
