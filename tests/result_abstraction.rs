// =========================================================================
// Logicodex v1.30 — Ketuk 2: Result<T, E> Abstraction Tests
// =========================================================================

use logicodex::ast::{Expr, MatchPattern, Stmt, Type};
use logicodex::lexer::{Lexer, Lexicon};
use logicodex::parser::{CompilerPipeline, Parser};

// ─── 1. Parser: Result<T, E> type ───

#[test]
fn parse_result_type_i32_i64() {
    let source = "let r: Result<i32, i64>";
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let parser = Parser::new(tokens).with_pipeline(CompilerPipeline::V130);
    let program = parser.parse().unwrap();

    match &program.statements[0] {
        Stmt::Let { declared_type, .. } => {
            let ty = declared_type.as_ref().unwrap();
            match ty {
                Type::Result { ok, err } => {
                    assert_eq!(**ok, Type::I32);
                    assert_eq!(**err, Type::I64);
                }
                other => panic!("Expected Result, got {:?}", other),
            }
        }
        other => panic!("Expected Let, got {:?}", other),
    }
}

#[test]
fn parse_result_type_buffer_error() {
    let source = "let r: Result<Buffer<f32>, IoError>";
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let parser = Parser::new(tokens).with_pipeline(CompilerPipeline::V130);
    let program = parser.parse().unwrap();

    match &program.statements[0] {
        Stmt::Let { declared_type, .. } => {
            let ty = declared_type.as_ref().unwrap();
            assert!(ty.is_result());
        }
        other => panic!("Expected Let, got {:?}", other),
    }
}

// ─── 2. Parser: Ok(value) expression ───

#[test]
fn parse_ok_constructor() {
    let source = "Ok(42)";
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let expr = parser.expression().unwrap();

    match expr {
        Expr::Ok { value } => assert_eq!(*value, Expr::Integer(42)),
        other => panic!("Expected Ok, got {:?}", other),
    }
}

#[test]
fn parse_ok_with_variable() {
    let source = "Ok(data)";
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let expr = parser.expression().unwrap();

    match expr {
        Expr::Ok { value } => assert_eq!(*value, Expr::Variable("data".into())),
        other => panic!("Expected Ok, got {:?}", other),
    }
}

// ─── 3. Parser: Err(value) expression ───

#[test]
fn parse_err_constructor() {
    let source = "Err(error)";
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let expr = parser.expression().unwrap();

    match expr {
        Expr::Err { value } => assert_eq!(*value, Expr::Variable("error".into())),
        other => panic!("Expected Err, got {:?}", other),
    }
}

// ─── 4. Parser: match statement ───

#[test]
fn parse_match_ok_err() {
    let source = r#"
match result {
    Ok(v) => v,
    Err(e) => 0
}
"#;
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let parser = Parser::new(tokens).with_pipeline(CompilerPipeline::V130);
    let program = parser.parse().unwrap();

    assert_eq!(program.statements.len(), 1);
    match &program.statements[0] {
        Stmt::Match { value, arms } => {
            assert_eq!(*value, Expr::Variable("result".into()));
            assert_eq!(arms.len(), 2);
            // First arm: Ok(v) => v
            match &arms[0].pattern {
                MatchPattern::Ok { binding } => assert_eq!(binding, "v"),
                _ => panic!("Expected Ok pattern"),
            }
            // Second arm: Err(e) => 0
            match &arms[1].pattern {
                MatchPattern::Err { binding } => assert_eq!(binding, "e"),
                _ => panic!("Expected Err pattern"),
            }
        }
        other => panic!("Expected Match, got {:?}", other),
    }
}

#[test]
fn parse_match_wildcard() {
    let source = r#"
match result {
    Ok(v) => v,
    _ => 0
}
"#;
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let parser = Parser::new(tokens).with_pipeline(CompilerPipeline::V130);
    let program = parser.parse().unwrap();

    match &program.statements[0] {
        Stmt::Match { arms, .. } => {
            assert_eq!(arms.len(), 2);
            match &arms[1].pattern {
                MatchPattern::Wildcard => {}
                _ => panic!("Expected Wildcard pattern"),
            }
        }
        other => panic!("Expected Match, got {:?}", other),
    }
}

// ─── 5. Type: Result properties ───

#[test]
fn result_is_result() {
    let r = Type::Result { ok: Box::new(Type::I32), err: Box::new(Type::I64) };
    assert!(r.is_result());
    assert!(!r.is_slice());
    assert!(!r.is_buffer());
}

#[test]
fn result_ok_type() {
    let r = Type::Result { ok: Box::new(Type::I32), err: Box::new(Type::I64) };
    assert_eq!(r.ok_type(), Some(&Type::I32));
}

#[test]
fn result_err_type() {
    let r = Type::Result { ok: Box::new(Type::I32), err: Box::new(Type::I64) };
    assert_eq!(r.err_type(), Some(&Type::I64));
}

#[test]
fn primitive_not_result() {
    assert!(!Type::I32.is_result());
    assert_eq!(Type::I32.ok_type(), None);
    assert_eq!(Type::I32.err_type(), None);
}

// ─── 6. Display formatting ───

#[test]
fn display_result_i32_i64() {
    let r = Type::Result { ok: Box::new(Type::I32), err: Box::new(Type::I64) };
    let text = format!("{}", r);
    assert!(text.contains("Result<"));
    assert!(text.contains("I32"));
    assert!(text.contains("I64"));
}

// ─── 7. Lexer tokens ───

#[test]
fn lexer_result_tokens() {
    let source = "Result Ok Err match => _";
    let tokens = Lexer::new(source, &Lexicon::from_str("{}").unwrap()).tokenize().unwrap();
    assert_eq!(tokens[0].lexeme, "Result");
    assert_eq!(tokens[1].lexeme, "Ok");
    assert_eq!(tokens[2].lexeme, "Err");
    assert_eq!(tokens[3].lexeme, "match");
    assert_eq!(tokens[4].lexeme, "=>");
    assert_eq!(tokens[5].lexeme, "_");
}

// ─── 8. Semantic: match on non-Result rejected ───

#[test]
fn semantic_rejects_match_on_non_result() {
    let source = r#"
match 42 {
    Ok(v) => v,
    Err(e) => 0
}
"#;
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let parser = Parser::new(tokens).with_pipeline(CompilerPipeline::V130);
    let program = parser.parse().unwrap();

    let result = logicodex::semantic::Analyzer::analyze(&program);
    assert!(result.is_err(), "Match on non-Result should be rejected");
}

// ─── 9. Full IO function example ───

#[test]
fn parse_io_function_signature() {
    let source = r#"
function read_file(path: []u8) -> Result<[]u8, IoError> {
    Ok(path)
}
"#;
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let parser = Parser::new(tokens).with_pipeline(CompilerPipeline::V130);
    let program = parser.parse().unwrap();

    match &program.statements[0] {
        Stmt::Function { name, return_type, .. } => {
            assert_eq!(name, "read_file");
            assert!(return_type.as_ref().unwrap().is_result());
        }
        other => panic!("Expected Function, got {:?}", other),
    }
}
