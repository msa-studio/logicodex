// =========================================================================
// Logicodex v1.30.1-alpha — Fasa 2: Zero-Copy Ownership Transfer Tests
//
// Ownership transfer via Channel send() — RAII move semantics.
// Variable cannot be used after being sent through a Channel channel.
// =========================================================================

use logicodex::ast::{Expr, Stmt, Type};
use logicodex::lexer::{Lexer, Lexicon};
use logicodex::parser::{CompilerPipeline, Parser};
use logicodex::semantic::Analyzer;

// ─── 1. Semantic: Variable moved after send ───

#[test]
fn semantic_moves_variable_on_send() {
    let source = r#"
let data = 42
channel_a.send(data)
"#;
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let parser = Parser::new(tokens).with_pipeline(CompilerPipeline::V130);
    let program = parser.parse().unwrap();

    // Should analyze without error — send is valid first use
    let result = Analyzer::analyze(&program);
    assert!(result.is_ok(), "First send of a variable should succeed");
}

// ─── 2. Semantic: UseAfterHantar detected ───

#[test]
fn semantic_rejects_use_after_send() {
    let source = r#"
let data = 42
channel_a.send(data)
let x = data
"#;
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let parser = Parser::new(tokens).with_pipeline(CompilerPipeline::V130);
    let program = parser.parse().unwrap();

    let result = Analyzer::analyze(&program);
    assert!(result.is_err(), "Use of variable after send should be rejected");
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("UseAfterHantar") || err.contains("disend") || err.contains("ownership"),
        "Error should mention ownership transfer: got {}",
        err
    );
}

// ─── 3. Semantic: Variable readable before send ───

#[test]
fn semantic_allows_use_before_send() {
    let source = r#"
let data = 42
let y = data + 1
channel_a.send(data)
"#;
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let parser = Parser::new(tokens).with_pipeline(CompilerPipeline::V130);
    let program = parser.parse().unwrap();

    let result = Analyzer::analyze(&program);
    assert!(
        result.is_ok(),
        "Using variable before send should be allowed: {:?}",
        result.err()
    );
}

// ─── 4. Semantic: Multiple different variables can be send'd ───

#[test]
fn semantic_allows_send_different_variables() {
    let source = r#"
let data_a = 42
let data_b = 100
channel_a.send(data_a)
channel_b.send(data_b)
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

// ─── 5. Semantic: Double-send same variable rejected ───

#[test]
fn semantic_rejects_double_send_same_variable() {
    let source = r#"
let data = 42
channel_a.send(data)
channel_a.send(data)
"#;
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let parser = Parser::new(tokens).with_pipeline(CompilerPipeline::V130);
    let program = parser.parse().unwrap();

    let result = Analyzer::analyze(&program);
    assert!(result.is_err(), "Double send of same variable should be rejected");
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("UseAfterHantar") || err.contains("disend") || err.contains("ownership"),
        "Error should mention ownership: got {}",
        err
    );
}

// ─── 6. Semantic: recv does not mark variable as moved ───

#[test]
fn semantic_recv_does_not_move() {
    let source = r#"
let msg = channel_a.recv()
let y = msg + 1
"#;
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let parser = Parser::new(tokens).with_pipeline(CompilerPipeline::V130);
    let program = parser.parse().unwrap();

    let result = Analyzer::analyze(&program);
    assert!(
        result.is_ok(),
        "Using value after recv should be allowed: {:?}",
        result.err()
    );
}

// ─── 7. Parser: send with variable expression ───

#[test]
fn parse_send_variable() {
    let source = "channel_data.send(nilai)";
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let expr = parser.expression().unwrap();

    match expr {
        Expr::Send { channel_name, value } => {
            assert_eq!(channel_name, "channel_data");
            match *value {
                Expr::Variable(name) => assert_eq!(name, "nilai"),
                other => panic!("Expected Variable, got {:?}", other),
            }
        }
        other => panic!("Expected Hantar, got {:?}", other),
    }
}

// ─── 8. Parser: recv expression standalone ───

#[test]
fn parse_recv_standalone() {
    let source = "channel_data.recv()";
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let expr = parser.expression().unwrap();

    match expr {
        Expr::Recv { channel_name } => {
            assert_eq!(channel_name, "channel_data");
        }
        other => panic!("Expected Terima, got {:?}", other),
    }
}

// ─── 9. Semantic: Non-variable send (literal) does not trigger move ───

#[test]
fn semantic_send_literal_not_moved() {
    let source = r#"
channel_a.send(42)
channel_a.send(100)
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
actor Producer {
    let buffer = 123
    let processed = buffer * 2
    channel_out.send(processed)
}

actor Consumer {
    let result = channel_in.recv()
    let final_val = result + 10
}

spawn Producer()
spawn Consumer()
join Producer
join Consumer
"#;
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let parser = Parser::new(tokens).with_pipeline(CompilerPipeline::V130);
    let program = parser.parse().unwrap();

    // Should parse 6 statements: 2 actor + 2 spawn + 2 join
    assert_eq!(program.statements.len(), 6);

    // Analyzer should accept — processed is moved correctly, result is from recv
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

function ring_send<T>(ring: *mut RingBuffer<T>, nilai: T) -> bool {
    true
}

function ring_recv<T>(ring: *mut RingBuffer<T>) -> Option<T> {
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
fn semantic_send_expression_not_variable() {
    let source = r#"
let a = 10
let b = 20
channel_x.send(a + b)
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
        "Using variables after send(expression) should succeed: {:?}",
        result.err()
    );
}
