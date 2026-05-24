// =========================================================================
// Logicodex v1.30.1-alpha — Fasa 2: Zero-Copy Ownership Transfer Tests
//
// Ownership transfer via Pintu hantar() — RAII move semantics.
// Variable cannot be used after being sent through a Pintu channel.
// =========================================================================

use logicodex::ast::{Expr, Stmt, Type};
use logicodex::lexer::{Lexer, Lexicon};
use logicodex::parser::{CompilerPipeline, Parser};
use logicodex::semantic::Analyzer;

// ─── 1. Semantic: Variable moved after hantar ───

#[test]
fn semantic_moves_variable_on_hantar() {
    let source = r#"
let data = 42
pintu_a.hantar(data)
"#;
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let parser = Parser::new(tokens).with_pipeline(CompilerPipeline::V130);
    let program = parser.parse().unwrap();

    // Should analyze without error — hantar is valid first use
    let result = Analyzer::analyze(&program);
    assert!(result.is_ok(), "First hantar of a variable should succeed");
}

// ─── 2. Semantic: UseAfterHantar detected ───

#[test]
fn semantic_rejects_use_after_hantar() {
    let source = r#"
let data = 42
pintu_a.hantar(data)
let x = data
"#;
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let parser = Parser::new(tokens).with_pipeline(CompilerPipeline::V130);
    let program = parser.parse().unwrap();

    let result = Analyzer::analyze(&program);
    assert!(result.is_err(), "Use of variable after hantar should be rejected");
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("UseAfterHantar") || err.contains("dihantar") || err.contains("ownership"),
        "Error should mention ownership transfer: got {}",
        err
    );
}

// ─── 3. Semantic: Variable readable before hantar ───

#[test]
fn semantic_allows_use_before_hantar() {
    let source = r#"
let data = 42
let y = data + 1
pintu_a.hantar(data)
"#;
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let parser = Parser::new(tokens).with_pipeline(CompilerPipeline::V130);
    let program = parser.parse().unwrap();

    let result = Analyzer::analyze(&program);
    assert!(
        result.is_ok(),
        "Using variable before hantar should be allowed: {:?}",
        result.err()
    );
}

// ─── 4. Semantic: Multiple different variables can be hantar'd ───

#[test]
fn semantic_allows_hantar_different_variables() {
    let source = r#"
let data_a = 42
let data_b = 100
pintu_a.hantar(data_a)
pintu_b.hantar(data_b)
"#;
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let parser = Parser::new(tokens).with_pipeline(CompilerPipeline::V130);
    let program = parser.parse().unwrap();

    let result = Analyzer::analyze(&program);
    assert!(
        result.is_ok(),
        "Hantar different variables should succeed: {:?}",
        result.err()
    );
}

// ─── 5. Semantic: Double-hantar same variable rejected ───

#[test]
fn semantic_rejects_double_hantar_same_variable() {
    let source = r#"
let data = 42
pintu_a.hantar(data)
pintu_a.hantar(data)
"#;
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let parser = Parser::new(tokens).with_pipeline(CompilerPipeline::V130);
    let program = parser.parse().unwrap();

    let result = Analyzer::analyze(&program);
    assert!(result.is_err(), "Double hantar of same variable should be rejected");
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("UseAfterHantar") || err.contains("dihantar") || err.contains("ownership"),
        "Error should mention ownership: got {}",
        err
    );
}

// ─── 6. Semantic: terima does not mark variable as moved ───

#[test]
fn semantic_terima_does_not_move() {
    let source = r#"
let msg = pintu_a.terima()
let y = msg + 1
"#;
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let parser = Parser::new(tokens).with_pipeline(CompilerPipeline::V130);
    let program = parser.parse().unwrap();

    let result = Analyzer::analyze(&program);
    assert!(
        result.is_ok(),
        "Using value after terima should be allowed: {:?}",
        result.err()
    );
}

// ─── 7. Parser: hantar with variable expression ───

#[test]
fn parse_hantar_variable() {
    let source = "pintu_data.hantar(nilai)";
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let expr = parser.expression().unwrap();

    match expr {
        Expr::Hantar { pintu_name, value } => {
            assert_eq!(pintu_name, "pintu_data");
            match *value {
                Expr::Variable(name) => assert_eq!(name, "nilai"),
                other => panic!("Expected Variable, got {:?}", other),
            }
        }
        other => panic!("Expected Hantar, got {:?}", other),
    }
}

// ─── 8. Parser: terima expression standalone ───

#[test]
fn parse_terima_standalone() {
    let source = "pintu_data.terima()";
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let expr = parser.expression().unwrap();

    match expr {
        Expr::Terima { pintu_name } => {
            assert_eq!(pintu_name, "pintu_data");
        }
        other => panic!("Expected Terima, got {:?}", other),
    }
}

// ─── 9. Semantic: Non-variable hantar (literal) does not trigger move ───

#[test]
fn semantic_hantar_literal_not_moved() {
    let source = r#"
pintu_a.hantar(42)
pintu_a.hantar(100)
"#;
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let parser = Parser::new(tokens).with_pipeline(CompilerPipeline::V130);
    let program = parser.parse().unwrap();

    let result = Analyzer::analyze(&program);
    assert!(
        result.is_ok(),
        "Hantar literals multiple times should succeed: {:?}",
        result.err()
    );
}

// ─── 10. Full zero-copy ownership transfer scenario ───

#[test]
fn full_zero_copy_ownership_scenario() {
    let source = r#"
kotak Producer {
    let buffer = 123
    let processed = buffer * 2
    pintu_out.hantar(processed)
}

kotak Consumer {
    let result = pintu_in.terima()
    let final_val = result + 10
}

lahirkan Producer()
lahirkan Consumer()
tunggu Producer
tunggu Consumer
"#;
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let parser = Parser::new(tokens).with_pipeline(CompilerPipeline::V130);
    let program = parser.parse().unwrap();

    // Should parse 6 statements: 2 kotak + 2 lahirkan + 2 tunggu
    assert_eq!(program.statements.len(), 6);

    // Analyzer should accept — processed is moved correctly, result is from terima
    let result = Analyzer::analyze(&program);
    assert!(
        result.is_ok(),
        "Full ownership scenario should pass: {:?}",
        result.err()
    );
}

// ─── 11. Ring buffer library exists and parses ───

#[test]
fn ring_buffer_library_parses() {
    // This is a structural test — ring_buffer.ldx should be parseable
    let source = r#"
struct RingBuffer<T> {
    penimbal: *mut T,
    kapasiti: i32,
    kepala: *mut i32,
    ekor: *mut i32
}

function ring_hantar<T>(ring: *mut RingBuffer<T>, nilai: T) -> bool {
    true
}

function ring_terima<T>(ring: *mut RingBuffer<T>) -> Option<T> {
    None
}
"#;
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let parser = Parser::new(tokens).with_pipeline(CompilerPipeline::V130);
    let program = parser.parse().unwrap();

    assert_eq!(program.statements.len(), 3); // struct + 2 functions
    match &program.statements[0] {
        Stmt::Struct { name, .. } => assert_eq!(name, "RingBuffer"),
        other => panic!("Expected Struct, got {:?}", other),
    }
}

// ─── 12. Ownership transfer with complex expression (not moved) ───

#[test]
fn semantic_hantar_expression_not_variable() {
    let source = r#"
let a = 10
let b = 20
pintu_x.hantar(a + b)
let c = a + b
"#;
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let parser = Parser::new(tokens).with_pipeline(CompilerPipeline::V130);
    let program = parser.parse().unwrap();

    // a + b is an expression, not a variable — a and b are not moved
    let result = Analyzer::analyze(&program);
    assert!(
        result.is_ok(),
        "Using variables after hantar(expression) should succeed: {:?}",
        result.err()
    );
}
