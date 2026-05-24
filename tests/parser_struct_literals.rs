// =========================================================================
// Logicodex v1.30 — Sprint 2.5: Struct Literal / Function Call Tests
//
// Tests that:
//   1. Parser recognizes Name(arg1, arg2, ...) as Expr::Call
//   2. TypeChecker validates struct constructors via TypeRegistry
//   3. Function calls work as statements and expressions
// =========================================================================

use logicodex::ast::Expr;
use logicodex::ffi::raylib::register_raylib_types;
use logicodex::lexer::{Lexer, Lexicon};
use logicodex::parser::{CompilerPipeline, Parser};
use logicodex::semantic::type_checker::TypeChecker;
use logicodex::types::TypeRegistry;

// ─── 1. Parser: Basic Function/Struct Literal Parsing ───

#[test]
fn parse_simple_call_no_args() {
    let source = "foo()";
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let expr = parser.expression().unwrap();

    match expr {
        Expr::Call { callee, args } => {
            assert_eq!(callee.as_ref(), &Expr::Variable("foo".into()));
            assert!(args.is_empty());
        }
        other => panic!("Expected Expr::Call, got {:?}", other),
    }
}

#[test]
fn parse_call_with_args() {
    let source = "Color(255, 0, 0, 255)";
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let expr = parser.expression().unwrap();

    match expr {
        Expr::Call { callee, args } => {
            assert_eq!(callee.as_ref(), &Expr::Variable("Color".into()));
            assert_eq!(args.len(), 4);
            // Check each argument
            match &args[0] {
                Expr::Integer(255) => {}
                other => panic!("Expected Integer(255), got {:?}", other),
            }
            match &args[1] {
                Expr::Integer(0) => {}
                other => panic!("Expected Integer(0), got {:?}", other),
            }
            match &args[2] {
                Expr::Integer(0) => {}
                other => panic!("Expected Integer(0), got {:?}", other),
            }
            match &args[3] {
                Expr::Integer(255) => {}
                other => panic!("Expected Integer(255), got {:?}", other),
            }
        }
        other => panic!("Expected Expr::Call, got {:?}", other),
    }
}

#[test]
fn parse_nested_call_as_arg() {
    let source = "foo(bar(1, 2), 3)";
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let expr = parser.expression().unwrap();

    match expr {
        Expr::Call { callee, args } => {
            assert_eq!(callee.as_ref(), &Expr::Variable("foo".into()));
            assert_eq!(args.len(), 2);
            // First arg is a nested call
            match &args[0] {
                Expr::Call { callee: inner_callee, args: inner_args } => {
                    assert_eq!(inner_callee.as_ref(), &Expr::Variable("bar".into()));
                    assert_eq!(inner_args.len(), 2);
                }
                other => panic!("Expected nested Call, got {:?}", other),
            }
            // Second arg is a simple integer
            match &args[1] {
                Expr::Integer(3) => {}
                other => panic!("Expected Integer(3), got {:?}", other),
            }
        }
        other => panic!("Expected Expr::Call, got {:?}", other),
    }
}

#[test]
fn parse_call_with_float_args() {
    let source = "Vector2(400.0, 300.0)";
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let expr = parser.expression().unwrap();

    match expr {
        Expr::Call { callee, args } => {
            assert_eq!(callee.as_ref(), &Expr::Variable("Vector2".into()));
            assert_eq!(args.len(), 2);
        }
        other => panic!("Expected Expr::Call, got {:?}", other),
    }
}

#[test]
fn parse_call_with_mixed_args() {
    let source = "InitWindow(800, 600, \"Hello\")";
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let expr = parser.expression().unwrap();

    match expr {
        Expr::Call { callee, args } => {
            assert_eq!(callee.as_ref(), &Expr::Variable("InitWindow".into()));
            assert_eq!(args.len(), 3);
            assert!(matches!(args[0], Expr::Integer(800)));
            assert!(matches!(args[1], Expr::Integer(600)));
            assert!(matches!(args[2], Expr::StringLiteral(ref s) if s == "Hello\""));
        }
        other => panic!("Expected Expr::Call, got {:?}", other),
    }
}

#[test]
fn parse_variable_not_call() {
    // Ensure that a bare identifier is still a Variable, not a Call
    let source = "x";
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let expr = parser.expression().unwrap();

    assert_eq!(expr, Expr::Variable("x".into()));
}

// ─── 2. TypeChecker: Struct Constructor Validation ───

#[test]
fn check_struct_constructor_color() {
    let mut registry = TypeRegistry::new();
    let _raylib_ids = register_raylib_types(&mut registry);
    let checker = TypeChecker::new(&registry);

    // Color(255, 0, 0, 255) — 4 args for 4 fields
    let result = checker.check_call(
        &Expr::Variable("Color".into()),
        &[
            Expr::Integer(255),
            Expr::Integer(0),
            Expr::Integer(0),
            Expr::Integer(255),
        ],
    );
    assert!(result.is_ok(), "Color(255, 0, 0, 255) should be valid: {:?}", result);
}

#[test]
fn check_struct_constructor_wrong_arg_count() {
    let mut registry = TypeRegistry::new();
    let _raylib_ids = register_raylib_types(&mut registry);
    let checker = TypeChecker::new(&registry);

    // Color(255, 0) — only 2 args, but Color has 4 fields
    let result = checker.check_call(
        &Expr::Variable("Color".into()),
        &[Expr::Integer(255), Expr::Integer(0)],
    );
    assert!(result.is_err(), "Color(255, 0) with 2 args should fail");
    let err = result.unwrap_err();
    assert!(err.contains("expects 4 arguments"), "Error should mention arg count: {}", err);
}

#[test]
fn check_struct_constructor_vector2() {
    let mut registry = TypeRegistry::new();
    let _raylib_ids = register_raylib_types(&mut registry);
    let checker = TypeChecker::new(&registry);

    // Vector2(400.0, 300.0) — 2 args for 2 fields
    let result = checker.check_call(
        &Expr::Variable("Vector2".into()),
        &[Expr::Integer(400), Expr::Integer(300)],
    );
    assert!(result.is_ok(), "Vector2(400, 300) should be valid: {:?}", result);
}

#[test]
fn check_unknown_function_or_struct() {
    let registry = TypeRegistry::new();
    let checker = TypeChecker::new(&registry);

    // FooBar(1, 2) — not a known struct or function
    let result = checker.check_call(
        &Expr::Variable("FooBar".into()),
        &[Expr::Integer(1), Expr::Integer(2)],
    );
    assert!(result.is_err(), "Unknown type should fail");
}

// ─── 3. TypeChecker: Raylib Types Registered ───

#[test]
fn color_is_findable_in_typechecker() {
    let mut registry = TypeRegistry::new();
    let _raylib_ids = register_raylib_types(&mut registry);

    assert!(
        registry.find_struct_by_name("Color").is_some(),
        "Color must be findable after register_raylib_types"
    );

    let (_, layout) = registry.find_struct_by_name("Color").unwrap();
    assert_eq!(layout.fields.len(), 4, "Color must have 4 fields");
    assert_eq!(layout.total_size_bytes, 4, "Color must be 4 bytes");
}

#[test]
fn texture2d_is_findable_in_typechecker() {
    let mut registry = TypeRegistry::new();
    let _raylib_ids = register_raylib_types(&mut registry);

    assert!(
        registry.find_struct_by_name("Texture2D").is_some(),
        "Texture2D must be findable"
    );

    let (_, layout) = registry.find_struct_by_name("Texture2D").unwrap();
    assert_eq!(layout.fields.len(), 5, "Texture2D must have 5 fields");
    assert_eq!(layout.total_size_bytes, 20, "Texture2D must be 20 bytes");
}
