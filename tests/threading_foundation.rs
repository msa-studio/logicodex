// =========================================================================
// Logicodex v1.30.1-alpha — Threading Foundation Tests
// Kotak & Pintu — Topology Validation
// =========================================================================

use logicodex::ast::{Expr, Stmt, Type};
use logicodex::lexer::{Lexer, Lexicon};
use logicodex::parser::{CompilerPipeline, Parser};

// ─── 1. Parser: Kotak declaration ───

#[test]
fn parse_kotak_declaration() {
    let source = r#"
kotak SensorSuhu {
    let x = 1
}
"#;
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let parser = Parser::new(tokens).with_pipeline(CompilerPipeline::V130);
    let program = parser.parse().unwrap();

    assert_eq!(program.statements.len(), 1);
    match &program.statements[0] {
        Stmt::Kotak { name, body } => {
            assert_eq!(name, "SensorSuhu");
            assert_eq!(body.len(), 1);
        }
        other => panic!("Expected Kotak, got {:?}", other),
    }
}

#[test]
fn parse_kotak_with_pintu() {
    let source = r#"
kotak SensorSuhu {
    let pintu_data: Pintu<SensorSuhu, Enjin, DataSuhu>
}
"#;
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let parser = Parser::new(tokens).with_pipeline(CompilerPipeline::V130);
    let program = parser.parse().unwrap();

    match &program.statements[0] {
        Stmt::Kotak { name, body } => {
            assert_eq!(name, "SensorSuhu");
            match &body[0] {
                Stmt::Let { declared_type, .. } => {
                    let ty = declared_type.as_ref().unwrap();
                    match ty {
                        Type::Pintu { from, to, message_type } => {
                            assert_eq!(from, "SensorSuhu");
                            assert_eq!(to, "Enjin");
                            assert_eq!(message_type, "DataSuhu");
                        }
                        other => panic!("Expected Pintu, got {:?}", other),
                    }
                }
                other => panic!("Expected Let, got {:?}", other),
            }
        }
        other => panic!("Expected Kotak, got {:?}", other),
    }
}

// ─── 2. Parser: lahirkan expression ───

#[test]
fn parse_lahirkan() {
    let source = "lahirkan SensorSuhu()";
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let expr = parser.expression().unwrap();

    match expr {
        Expr::Spawn { kotak_name, args } => {
            assert_eq!(kotak_name, "SensorSuhu");
            assert!(args.is_empty());
        }
        other => panic!("Expected Spawn, got {:?}", other),
    }
}

// ─── 3. Parser: tunggu expression ───

#[test]
fn parse_tunggu() {
    let source = "tunggu SensorSuhu";
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let expr = parser.expression().unwrap();

    match expr {
        Expr::Tunggu { kotak_name } => {
            assert_eq!(kotak_name, "SensorSuhu");
        }
        other => panic!("Expected Tunggu, got {:?}", other),
    }
}

// ─── 4. Parser: hantar expression ───

#[test]
fn parse_hantar() {
    let source = "pintu_data.hantar(42)";
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let expr = parser.expression().unwrap();

    match expr {
        Expr::Hantar { pintu_name, value } => {
            assert_eq!(pintu_name, "pintu_data");
            assert_eq!(*value, Expr::Integer(42));
        }
        other => panic!("Expected Hantar, got {:?}", other),
    }
}

// ─── 5. Parser: terima expression ───

#[test]
fn parse_terima() {
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

// ─── 6. Type: Pintu properties ───

#[test]
fn pintu_type_properties() {
    let p = Type::Pintu {
        from: "SensorSuhu".to_string(),
        to: "Enjin".to_string(),
        message_type: "DataSuhu".to_string(),
    };
    assert!(p.is_pintu());
    assert!(!p.is_opaque());
    assert!(!p.is_result());
}

#[test]
fn pintu_capability() {
    let p = Type::Pintu {
        from: "A".to_string(),
        to: "B".to_string(),
        message_type: "Msg".to_string(),
    };
    assert_eq!(p.pintu_capability(), Some(("A", "B", "Msg")));
}

#[test]
fn non_pintu_no_capability() {
    assert_eq!(Type::I32.pintu_capability(), None);
    assert!(!Type::I32.is_pintu());
}

// ─── 7. Lexer tokens ───

#[test]
fn lexer_threading_tokens() {
    let source = "kotak pintu lahirkan hantar terima tunggu";
    let tokens = Lexer::new(source, &Lexicon::from_str("{}").unwrap()).tokenize().unwrap();
    assert_eq!(tokens[0].lexeme, "kotak");
    assert_eq!(tokens[1].lexeme, "pintu");
    assert_eq!(tokens[2].lexeme, "lahirkan");
    assert_eq!(tokens[3].lexeme, "hantar");
    assert_eq!(tokens[4].lexeme, "terima");
    assert_eq!(tokens[5].lexeme, "tunggu");
}

// ─── 8. Display formatting ───

#[test]
fn display_pintu() {
    let p = Type::Pintu {
        from: "A".to_string(),
        to: "B".to_string(),
        message_type: "Msg".to_string(),
    };
    let text = format!("{}", p);
    assert!(text.contains("Pintu<"));
    assert!(text.contains("A"));
    assert!(text.contains("B"));
    assert!(text.contains("Msg"));
}

// ─── 9. Full topology example parse ───

#[test]
fn full_topology_parse() {
    let source = r#"
kotak SensorSuhu {
    let pintu_data: Pintu<SensorSuhu, Enjin, DataSuhu>
}

kotak Enjin {
    let pintu_data: Pintu<SensorSuhu, Enjin, DataSuhu>
}

lahirkan SensorSuhu()
lahirkan Enjin()
tunggu SensorSuhu
tunggu Enjin
"#;
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let parser = Parser::new(tokens).with_pipeline(CompilerPipeline::V130);
    let program = parser.parse().unwrap();

    // 2 Kotak + 4 expr stmts
    assert_eq!(program.statements.len(), 6);

    // Verify Kotak declarations
    match &program.statements[0] {
        Stmt::Kotak { name, .. } => assert_eq!(name, "SensorSuhu"),
        _ => panic!("Expected Kotak"),
    }
    match &program.statements[1] {
        Stmt::Kotak { name, .. } => assert_eq!(name, "Enjin"),
        _ => panic!("Expected Kotak"),
    }

    // Verify lahirkan calls
    match &program.statements[2] {
        Stmt::ExprStmt { value: Expr::Spawn { kotak_name, .. } } => {
            assert_eq!(kotak_name, "SensorSuhu");
        }
        _ => panic!("Expected lahirkan SensorSuhu"),
    }
    match &program.statements[3] {
        Stmt::ExprStmt { value: Expr::Spawn { kotak_name, .. } } => {
            assert_eq!(kotak_name, "Enjin");
        }
        _ => panic!("Expected lahirkan Enjin"),
    }

    // Verify tunggu
    match &program.statements[4] {
        Stmt::ExprStmt { value: Expr::Tunggu { kotak_name } } => {
            assert_eq!(kotak_name, "SensorSuhu");
        }
        _ => panic!("Expected tunggu SensorSuhu"),
    }
}

// ─── 10. Semantic: Duplicate Kotak rejected ───

#[test]
fn semantic_rejects_duplicate_kotak() {
    let source = r#"
kotak SensorSuhu { let x = 1 }
kotak SensorSuhu { let y = 2 }
"#;
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let parser = Parser::new(tokens).with_pipeline(CompilerPipeline::V130);
    let program = parser.parse().unwrap();

    let result = logicodex::semantic::Analyzer::analyze(&program);
    assert!(result.is_err(), "Duplicate Kotak should be rejected");
}

// ─── 11. Semantic: lahirkan non-existent Kotak rejected ───

#[test]
fn semantic_rejects_spawn_nonexistent_kotak() {
    let source = "lahirkan SensorSuhu()";
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let parser = Parser::new(tokens).with_pipeline(CompilerPipeline::V130);
    let program = parser.parse().unwrap();

    let result = logicodex::semantic::Analyzer::analyze(&program);
    assert!(result.is_err(), "Spawn of non-existent Kotak should be rejected");
}

// ─── 12. Pintu type display ───

#[test]
fn pintu_display_full() {
    let p = Type::Pintu {
        from: "Pengesan".to_string(),
        to: "Pemproses".to_string(),
        message_type: "Isyarat".to_string(),
    };
    assert_eq!(format!("{}", p), "Pintu<Pengesan, Pemproses, Isyarat>");
}
