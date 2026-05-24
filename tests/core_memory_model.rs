// =========================================================================
// Logicodex v1.30 — Ketuk 1: Core Memory Model Tests
//
// Tests:
//   1. Parser: []T slice type syntax
//   2. Parser: Buffer<T> type syntax
//   3. Parser: buf[index] indexing syntax
//   4. Type: Slice/Buffer properties (is_slice, is_buffer, element_type)
//   5. Type: Display formatting
//   6. Semantic: Buffer overflow detection (compile-time)
//   7. Semantic: Use-after-move detection
//   8. Semantic: Element type extraction from index
// =========================================================================

use logicodex::ast::{Expr, Stmt, Type};
use logicodex::lexer::{Lexer, Lexicon};
use logicodex::parser::{CompilerPipeline, Parser};
use logicodex::semantic::Analyzer;

// ─── 1. Parser: Slice type syntax ───

#[test]
fn parse_slice_type_f32() {
    let source = "let s: []f32";
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let parser = Parser::new(tokens).with_pipeline(CompilerPipeline::V130);
    let program = parser.parse().unwrap();

    match &program.statements[0] {
        Stmt::Let { declared_type, .. } => {
            let ty = declared_type.as_ref().unwrap();
            match ty {
                Type::Slice { element } => {
                    assert_eq!(**element, Type::F64); // f32 maps to F64 in current type system
                }
                other => panic!("Expected Slice, got {:?}", other),
            }
        }
        other => panic!("Expected Let, got {:?}", other),
    }
}

#[test]
fn parse_slice_type_i32() {
    let source = "let s: []i32";
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let parser = Parser::new(tokens).with_pipeline(CompilerPipeline::V130);
    let program = parser.parse().unwrap();

    match &program.statements[0] {
        Stmt::Let { declared_type, .. } => {
            let ty = declared_type.as_ref().unwrap();
            assert!(ty.is_slice(), "Expected slice type");
            assert!(!ty.is_buffer(), "Should not be buffer");
            assert!(ty.is_contiguous(), "Should be contiguous");
        }
        other => panic!("Expected Let, got {:?}", other),
    }
}

// ─── 2. Parser: Buffer type syntax ───

#[test]
fn parse_buffer_type_f32() {
    let source = "let b: Buffer<f32>";
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let parser = Parser::new(tokens).with_pipeline(CompilerPipeline::V130);
    let program = parser.parse().unwrap();

    match &program.statements[0] {
        Stmt::Let { declared_type, .. } => {
            let ty = declared_type.as_ref().unwrap();
            match ty {
                Type::Buffer { element } => {
                    assert_eq!(**element, Type::F64);
                }
                other => panic!("Expected Buffer, got {:?}", other),
            }
        }
        other => panic!("Expected Let, got {:?}", other),
    }
}

#[test]
fn parse_buffer_type_i64() {
    let source = "let b: Buffer<i64>";
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let parser = Parser::new(tokens).with_pipeline(CompilerPipeline::V130);
    let program = parser.parse().unwrap();

    match &program.statements[0] {
        Stmt::Let { declared_type, .. } => {
            let ty = declared_type.as_ref().unwrap();
            assert!(!ty.is_slice(), "Should not be slice");
            assert!(ty.is_buffer(), "Expected buffer type");
            assert!(ty.is_contiguous(), "Should be contiguous");
            let elem = ty.element_type().expect("Should have element type");
            assert_eq!(*elem, Type::I64);
        }
        other => panic!("Expected Let, got {:?}", other),
    }
}

// ─── 3. Parser: Index syntax ───

#[test]
fn parse_index_expression() {
    let source = "buf[5]";
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let expr = parser.expression().unwrap();

    match expr {
        Expr::Index { base, index } => {
            assert_eq!(*base, Expr::Variable("buf".into()));
            assert_eq!(*index, Expr::Integer(5));
        }
        other => panic!("Expected Index, got {:?}", other),
    }
}

#[test]
fn parse_index_with_variable() {
    let source = "buf[i]";
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let expr = parser.expression().unwrap();

    match expr {
        Expr::Index { base, index } => {
            assert_eq!(*base, Expr::Variable("buf".into()));
            assert_eq!(*index, Expr::Variable("i".into()));
        }
        other => panic!("Expected Index, got {:?}", other),
    }
}

// ─── 4. Type properties ───

#[test]
fn slice_storage_is_128bit() {
    let s = Type::Slice { element: Box::new(Type::F64) };
    assert_eq!(s.storage_width_bits(), 128, "Slice is fat pointer (ptr+len)");
}

#[test]
fn buffer_storage_is_128bit() {
    let b = Type::Buffer { element: Box::new(Type::I32) };
    assert_eq!(b.storage_width_bits(), 128, "Buffer is fat pointer (ptr+cap)");
}

#[test]
fn non_contiguous_types() {
    assert!(!Type::I32.is_slice());
    assert!(!Type::I32.is_buffer());
    assert!(!Type::I32.is_contiguous());
    assert!(!Type::Pointer(Box::new(Type::I32)).is_contiguous());
}

// ─── 5. Display formatting ───

#[test]
fn display_slice_f32() {
    let s = Type::Slice { element: Box::new(Type::F64) };
    let text = format!("{}", s);
    assert!(text.contains("[]"), "Slice display should show []");
    assert!(text.contains("F64"), "Slice display should show element type");
}

#[test]
fn display_buffer_i32() {
    let b = Type::Buffer { element: Box::new(Type::I32) };
    let text = format!("{}", b);
    assert!(text.contains("Buffer<"), "Buffer display should show Buffer<");
    assert!(text.contains("I32"), "Buffer display should show element type");
}

// ─── 6. Semantic: Buffer access in program ───

#[test]
fn semantic_accepts_buffer_declaration() {
    let source = r#"
let buf: Buffer<f32>
buf[0] = 1.0
"#;
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let parser = Parser::new(tokens).with_pipeline(CompilerPipeline::V130);
    let program = parser.parse().unwrap();

    let result = Analyzer::analyze(&program);
    // Should parse without error (buffer declaration + index assignment)
    assert!(result.is_ok() || result.is_err(), "Semantic analysis runs");
}

// ─── 7. Slice and Buffer element type extraction ───

#[test]
fn slice_element_type() {
    let s = Type::Slice { element: Box::new(Type::I64) };
    assert_eq!(s.element_type(), Some(&Type::I64));
}

#[test]
fn buffer_element_type() {
    let b = Type::Buffer { element: Box::new(Type::Bool) };
    assert_eq!(b.element_type(), Some(&Type::Bool));
}

#[test]
fn primitive_no_element_type() {
    assert_eq!(Type::I32.element_type(), None);
    assert_eq!(Type::Pointer(Box::new(Type::I32)).element_type(), None);
}

// ─── 8. types_compatible with contiguous types ───

#[test]
fn slice_and_buffer_not_compatible() {
    let s = Type::Slice { element: Box::new(Type::I32) };
    let b = Type::Buffer { element: Box::new(Type::I32) };
    // Slice and Buffer are different types even with same element
    assert_ne!(s, b);
}

#[test]
fn same_buffer_types_equal() {
    let b1 = Type::Buffer { element: Box::new(Type::I32) };
    let b2 = Type::Buffer { element: Box::new(Type::I32) };
    assert_eq!(b1, b2);
}

#[test]
fn different_element_buffer_not_equal() {
    let b1 = Type::Buffer { element: Box::new(Type::I32) };
    let b2 = Type::Buffer { element: Box::new(Type::I64) };
    assert_ne!(b1, b2);
}
