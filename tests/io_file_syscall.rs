// =========================================================================
// Logicodex v1.30 — Ketuk 3 + 4: File Handle ABI + Syscall Backend Tests
// =========================================================================

use logicodex::ast::{Expr, Stmt, Type};
use logicodex::lexer::{Lexer, Lexicon};
use logicodex::parser::{CompilerPipeline, Parser};

// ─── K3: Parser: FileHandle opaque type ───

#[test]
fn parse_filehandle_type() {
    let source = "let h: FileHandle";
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let parser = Parser::new(tokens).with_pipeline(CompilerPipeline::V130);
    let program = parser.parse().unwrap();

    match &program.statements[0] {
        Stmt::Let { declared_type, .. } => {
            let ty = declared_type.as_ref().unwrap();
            assert!(ty.is_opaque());
            assert_eq!(ty.opaque_name(), Some("FileHandle"));
        }
        other => panic!("Expected Let, got {:?}", other),
    }
}

// ─── K3: Parser: Method call h.read(1024) ───

#[test]
fn parse_method_call_read() {
    let source = "h.read(1024)";
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let expr = parser.expression().unwrap();

    match expr {
        Expr::MethodCall { object, method, args } => {
            assert_eq!(object, "h");
            assert_eq!(method, "read");
            assert_eq!(args.len(), 1);
            assert_eq!(args[0], Expr::Integer(1024));
        }
        other => panic!("Expected MethodCall, got {:?}", other),
    }
}

#[test]
fn parse_method_call_close() {
    let source = "h.close()";
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let expr = parser.expression().unwrap();

    match expr {
        Expr::MethodCall { object, method, args } => {
            assert_eq!(object, "h");
            assert_eq!(method, "close");
            assert!(args.is_empty());
        }
        other => panic!("Expected MethodCall, got {:?}", other),
    }
}

#[test]
fn parse_method_call_write() {
    let source = "h.write(data)";
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let expr = parser.expression().unwrap();

    match expr {
        Expr::MethodCall { object, method, args } => {
            assert_eq!(object, "h");
            assert_eq!(method, "write");
            assert_eq!(args.len(), 1);
            assert_eq!(args[0], Expr::Variable("data".into()));
        }
        other => panic!("Expected MethodCall, got {:?}", other),
    }
}

// ─── K3: Type properties ───

#[test]
fn opaque_type_properties() {
    let fh = Type::Opaque { name: "FileHandle".to_string() };
    assert!(fh.is_opaque());
    assert!(!fh.is_result());
    assert!(!fh.is_buffer());
    assert_eq!(fh.opaque_name(), Some("FileHandle"));
}

#[test]
fn opaque_display() {
    let fh = Type::Opaque { name: "FileHandle".to_string() };
    assert_eq!(format!("{}", fh), "FileHandle");
}

// ─── K3: Lexer tokens ───

#[test]
fn lexer_file_handle_tokens() {
    let source = "FileHandle open close read write seek";
    let tokens = Lexer::new(source, &Lexicon::from_str("{}").unwrap()).tokenize().unwrap();
    assert_eq!(tokens[0].lexeme, "FileHandle");
    assert_eq!(tokens[1].lexeme, "open");
    assert_eq!(tokens[2].lexeme, "close");
    assert_eq!(tokens[3].lexeme, "read");
    assert_eq!(tokens[4].lexeme, "write");
    assert_eq!(tokens[5].lexeme, "seek");
}

#[test]
fn lexer_dot_token() {
    let source = "h.read";
    let tokens = Lexer::new(source, &Lexicon::from_str("{}").unwrap()).tokenize().unwrap();
    assert_eq!(tokens[0].lexeme, "h");
    assert_eq!(tokens[1].lexeme, ".");
    assert_eq!(tokens[2].lexeme, "read");
}

// ─── K4: Syscall module exists ───

#[test]
fn syscall_linux_constants() {
    // Verify syscall numbers are correct
    assert_eq!(logicodex::os::syscall::linux::SYS_READ, 0);
    assert_eq!(logicodex::os::syscall::linux::SYS_WRITE, 1);
    assert_eq!(logicodex::os::syscall::linux::SYS_OPEN, 2);
    assert_eq!(logicodex::os::syscall::linux::SYS_CLOSE, 3);
    assert_eq!(logicodex::os::syscall::linux::SYS_LSEEK, 8);
}

#[test]
fn file_op_enum_exists() {
    use logicodex::os::syscall::FileOp;
    let _ = FileOp::Read;
    let _ = FileOp::Write;
    let _ = FileOp::Close;
    let _ = FileOp::Seek;
}

// ─── Full IO lifecycle parse ───

#[test]
fn full_io_lifecycle_parse() {
    let source = r#"
let h: FileHandle
h.read(1024)
h.write(data)
h.close()
"#;
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let parser = Parser::new(tokens).with_pipeline(CompilerPipeline::V130);
    let program = parser.parse().unwrap();

    assert_eq!(program.statements.len(), 3);

    // h.read(1024)
    match &program.statements[0] {
        Stmt::ExprStmt { value } => match value {
            Expr::MethodCall { object, method, .. } => {
                assert_eq!(object, "h");
                assert_eq!(method, "read");
            }
            other => panic!("Expected MethodCall, got {:?}", other),
        },
        other => panic!("Expected ExprStmt, got {:?}", other),
    }

    // h.close()
    match &program.statements[2] {
        Stmt::ExprStmt { value } => match value {
            Expr::MethodCall { object, method, .. } => {
                assert_eq!(object, "h");
                assert_eq!(method, "close");
            }
            other => panic!("Expected MethodCall, got {:?}", other),
        },
        other => panic!("Expected ExprStmt, got {:?}", other),
    }
}
