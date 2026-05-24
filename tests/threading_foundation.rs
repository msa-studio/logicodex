// =========================================================================
// Logicodex v1.30.1-alpha — Threading Foundation Tests
// Actor & Channel — Topology Validation
// =========================================================================

use logicodex::ast::{Expr, Stmt, Type};
use logicodex::lexer::{Lexer, Lexicon};
use logicodex::parser::{CompilerPipeline, Parser};

// ─── 1. Parser: Actor declaration ───

#[test]
fn parse_actor_declaration() {
    let source = r#"
actor SensorSuhu {
    let x = 1
}
"#;
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let parser = Parser::new(tokens).with_pipeline(CompilerPipeline::V130);
    let program = parser.parse().unwrap();

    assert_eq!(program.statements.len(), 1);
    match &program.statements[0] {
        Stmt::Actor { name, body } => {
            assert_eq!(name, "SensorSuhu");
            assert_eq!(body.len(), 1);
        }
        other => panic!("Expected Actor, got {:?}", other),
    }
}

#[test]
fn parse_actor_with_channel() {
    let source = r#"
actor SensorSuhu {
    let channel_data: Channel<SensorSuhu, Enjin, DataSuhu>
}
"#;
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let parser = Parser::new(tokens).with_pipeline(CompilerPipeline::V130);
    let program = parser.parse().unwrap();

    match &program.statements[0] {
        Stmt::Actor { name, body } => {
            assert_eq!(name, "SensorSuhu");
            match &body[0] {
                Stmt::Let { declared_type, .. } => {
                    let ty = declared_type.as_ref().unwrap();
                    match ty {
                        Type::Channel { from, to, message_type } => {
                            assert_eq!(from, "SensorSuhu");
                            assert_eq!(to, "Enjin");
                            assert_eq!(message_type, "DataSuhu");
                        }
                        other => panic!("Expected Channel, got {:?}", other),
                    }
                }
                other => panic!("Expected Let, got {:?}", other),
            }
        }
        other => panic!("Expected Actor, got {:?}", other),
    }
}

// ─── 2. Parser: spawn expression ───

#[test]
fn parse_spawn() {
    let source = "spawn SensorSuhu()";
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let expr = parser.expression().unwrap();

    match expr {
        Expr::Spawn { actor_name, args } => {
            assert_eq!(actor_name, "SensorSuhu");
            assert!(args.is_empty());
        }
        other => panic!("Expected Spawn, got {:?}", other),
    }
}

// ─── 3. Parser: join expression ───

#[test]
fn parse_join() {
    let source = "join SensorSuhu";
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let expr = parser.expression().unwrap();

    match expr {
        Expr::Join { actor_name } => {
            assert_eq!(actor_name, "SensorSuhu");
        }
        other => panic!("Expected Tunggu, got {:?}", other),
    }
}

// ─── 4. Parser: send expression ───

#[test]
fn parse_send() {
    let source = "channel_data.send(42)";
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let expr = parser.expression().unwrap();

    match expr {
        Expr::Send { channel_name, value } => {
            assert_eq!(channel_name, "channel_data");
            assert_eq!(*value, Expr::Integer(42));
        }
        other => panic!("Expected Hantar, got {:?}", other),
    }
}

// ─── 5. Parser: recv expression ───

#[test]
fn parse_recv() {
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

// ─── 6. Type: Channel properties ───

#[test]
fn channel_type_properties() {
    let p = Type::Channel {
        from: "SensorSuhu".to_string(),
        to: "Enjin".to_string(),
        message_type: "DataSuhu".to_string(),
    };
    assert!(p.is_channel());
    assert!(!p.is_opaque());
    assert!(!p.is_result());
}

#[test]
fn channel_capability() {
    let p = Type::Channel {
        from: "A".to_string(),
        to: "B".to_string(),
        message_type: "Msg".to_string(),
    };
    assert_eq!(p.channel_capability(), Some(("A", "B", "Msg")));
}

#[test]
fn non_channel_no_capability() {
    assert_eq!(Type::I32.channel_capability(), None);
    assert!(!Type::I32.is_channel());
}

// ─── 7. Lexer tokens ───

#[test]
fn lexer_threading_tokens() {
    let source = "actor channel spawn send recv join";
    let tokens = Lexer::new(source, &Lexicon::from_str("{}").unwrap()).tokenize().unwrap();
    assert_eq!(tokens[0].lexeme, "actor");
    assert_eq!(tokens[1].lexeme, "channel");
    assert_eq!(tokens[2].lexeme, "spawn");
    assert_eq!(tokens[3].lexeme, "send");
    assert_eq!(tokens[4].lexeme, "recv");
    assert_eq!(tokens[5].lexeme, "join");
}

// ─── 8. Display formatting ───

#[test]
fn display_channel() {
    let p = Type::Channel {
        from: "A".to_string(),
        to: "B".to_string(),
        message_type: "Msg".to_string(),
    };
    let text = format!("{}", p);
    assert!(text.contains("Channel<"));
    assert!(text.contains("A"));
    assert!(text.contains("B"));
    assert!(text.contains("Msg"));
}

// ─── 9. Full topology example parse ───

#[test]
fn full_topology_parse() {
    let source = r#"
actor SensorSuhu {
    let channel_data: Channel<SensorSuhu, Enjin, DataSuhu>
}

actor Enjin {
    let channel_data: Channel<SensorSuhu, Enjin, DataSuhu>
}

spawn SensorSuhu()
spawn Enjin()
join SensorSuhu
join Enjin
"#;
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let parser = Parser::new(tokens).with_pipeline(CompilerPipeline::V130);
    let program = parser.parse().unwrap();

    // 2 Actor + 4 expr stmts
    assert_eq!(program.statements.len(), 6);

    // Verify Actor declarations
    match &program.statements[0] {
        Stmt::Actor { name, .. } => assert_eq!(name, "SensorSuhu"),
        _ => panic!("Expected Actor"),
    }
    match &program.statements[1] {
        Stmt::Actor { name, .. } => assert_eq!(name, "Enjin"),
        _ => panic!("Expected Actor"),
    }

    // Verify spawn calls
    match &program.statements[2] {
        Stmt::ExprStmt { value: Expr::Spawn { actor_name, .. } } => {
            assert_eq!(actor_name, "SensorSuhu");
        }
        _ => panic!("Expected spawn SensorSuhu"),
    }
    match &program.statements[3] {
        Stmt::ExprStmt { value: Expr::Spawn { actor_name, .. } } => {
            assert_eq!(actor_name, "Enjin");
        }
        _ => panic!("Expected spawn Enjin"),
    }

    // Verify join
    match &program.statements[4] {
        Stmt::ExprStmt { value: Expr::Join { actor_name } } => {
            assert_eq!(actor_name, "SensorSuhu");
        }
        _ => panic!("Expected join SensorSuhu"),
    }
}

// ─── 10. Semantic: Duplicate Actor rejected ───

#[test]
fn semantic_rejects_duplicate_actor() {
    let source = r#"
actor SensorSuhu { let x = 1 }
actor SensorSuhu { let y = 2 }
"#;
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let parser = Parser::new(tokens).with_pipeline(CompilerPipeline::V130);
    let program = parser.parse().unwrap();

    let result = logicodex::semantic::Analyzer::analyze(&program);
    assert!(result.is_err(), "Duplicate Actor should be rejected");
}

// ─── 11. Semantic: spawn non-existent Actor rejected ───

#[test]
fn semantic_rejects_spawn_nonexistent_actor() {
    let source = "spawn SensorSuhu()";
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let parser = Parser::new(tokens).with_pipeline(CompilerPipeline::V130);
    let program = parser.parse().unwrap();

    let result = logicodex::semantic::Analyzer::analyze(&program);
    assert!(result.is_err(), "Spawn of non-existent Actor should be rejected");
}

// ─── 12. Channel type display ───

#[test]
fn channel_display_full() {
    let p = Type::Channel {
        from: "Pengesan".to_string(),
        to: "Pemproses".to_string(),
        message_type: "Isyarat".to_string(),
    };
    assert_eq!(format!("{}", p), "Channel<Pengesan, Pemproses, Isyarat>");
}
