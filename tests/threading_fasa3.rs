// =========================================================================
// Logicodex v1.30.1-alpha — Phase 3: Backpressure + Scheduler Tests
//
// try_send, try_recv, yield, sleep, timeout_recv
// =========================================================================

use logicodex::ast::{Expr, Stmt, Type};
use logicodex::lexer::{Lexer, Lexicon};
use logicodex::parser::{CompilerPipeline, Parser};
use logicodex::semantic::Analyzer;

// ─── 1. Parser: try_send expression ───

#[test]
fn parse_try_send() {
    let source = "channel_a.try_send(42)";
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let expr = parser.expression().unwrap();

    match expr {
        Expr::TrySend { channel_name, value } => {
            assert_eq!(channel_name, "channel_a");
            assert_eq!(*value, Expr::Integer(42));
        }
        other => panic!("Expected TrySend, got {:?}", other),
    }
}

// ─── 2. Parser: try_recv expression ───

#[test]
fn parse_try_recv() {
    let source = "channel_a.try_recv()";
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let expr = parser.expression().unwrap();

    match expr {
        Expr::TryRecv { channel_name } => {
            assert_eq!(channel_name, "channel_a");
        }
        other => panic!("Expected TryRecv, got {:?}", other),
    }
}

// ─── 3. Parser: yield expression ───

#[test]
fn parse_yield() {
    let source = "yield()";
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let expr = parser.expression().unwrap();

    match expr {
        Expr::Yield => {},
        other => panic!("Expected Yield, got {:?}", other),
    }
}

// ─── 4. Parser: sleep expression ───

#[test]
fn parse_sleep() {
    let source = "sleep(1000)";
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let expr = parser.expression().unwrap();

    match expr {
        Expr::Sleep { duration_ms } => {
            assert_eq!(*duration_ms, Expr::Integer(1000));
        }
        other => panic!("Expected Sleep, got {:?}", other),
    }
}

// ─── 5. Parser: timeout_recv expression ───

#[test]
fn parse_timeout_recv() {
    let source = "channel_a.timeout_recv(5000)";
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let expr = parser.expression().unwrap();

    match expr {
        Expr::TimeoutRecv { channel_name, timeout_ms } => {
            assert_eq!(channel_name, "channel_a");
            assert_eq!(*timeout_ms, Expr::Integer(5000));
        }
        other => panic!("Expected TimeoutRecv, got {:?}", other),
    }
}

// ─── 6. Lexer: Phase 3 tokens ───

#[test]
fn lexer_phase3_tokens() {
    let source = "try_send try_recv yield sleep timeout_recv";
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    // These are identifiers, not keywords (method call syntax)
    // The lexer recognizes them as identifiers unless they match TokenKind aliases
    assert_eq!(tokens[0].lexeme, "try_send");
    assert_eq!(tokens[1].lexeme, "try_recv");
    assert_eq!(tokens[2].lexeme, "yield");
    assert_eq!(tokens[3].lexeme, "sleep");
    assert_eq!(tokens[4].lexeme, "timeout_recv");
}

// ─── 7. Semantic: try_send with valid channel ───

#[test]
fn semantic_try_send_valid() {
    let source = r#"
actor Producer {
    let ch: Channel<Producer, Consumer, i32>
}
actor Consumer {
    let ch: Channel<Producer, Consumer, i32>
}
actor Producer {
    let ch: Channel<Producer, Consumer, i32>
    ch.try_send(42)
}
"#;
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let parser = Parser::new(tokens).with_pipeline(CompilerPipeline::V130);
    let program = parser.parse().unwrap();

    let result = Analyzer::analyze(&program);
    assert!(result.is_ok(), "try_send on valid channel should pass: {:?}", result.err());
}

// ─── 8. Semantic: try_recv with valid channel ───

#[test]
fn semantic_try_recv_valid() {
    let source = r#"
actor Producer {
    let ch: Channel<Producer, Consumer, i32>
}
actor Consumer {
    let ch: Channel<Producer, Consumer, i32>
    let msg = ch.try_recv()
}
"#;
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let parser = Parser::new(tokens).with_pipeline(CompilerPipeline::V130);
    let program = parser.parse().unwrap();

    let result = Analyzer::analyze(&program);
    assert!(result.is_ok(), "try_recv on valid channel should pass: {:?}", result.err());
}

// ─── 9. Semantic: yield in actor body ───

#[test]
fn semantic_yield_in_actor() {
    let source = r#"
actor Worker {
    yield()
}
"#;
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let parser = Parser::new(tokens).with_pipeline(CompilerPipeline::V130);
    let program = parser.parse().unwrap();

    let result = Analyzer::analyze(&program);
    assert!(result.is_ok(), "yield in actor should pass: {:?}", result.err());
}

// ─── 10. Semantic: sleep with numeric duration ───

#[test]
fn semantic_sleep_numeric() {
    let source = r#"
actor Worker {
    sleep(1000)
}
"#;
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let parser = Parser::new(tokens).with_pipeline(CompilerPipeline::V130);
    let program = parser.parse().unwrap();

    let result = Analyzer::analyze(&program);
    assert!(result.is_ok(), "sleep with numeric duration should pass: {:?}", result.err());
}

// ─── 11. Semantic: timeout_recv with valid channel ───

#[test]
fn semantic_timeout_recv_valid() {
    let source = r#"
actor Producer {
    let ch: Channel<Producer, Consumer, i32>
}
actor Consumer {
    let ch: Channel<Producer, Consumer, i32>
    let msg = ch.timeout_recv(5000)
}
"#;
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let parser = Parser::new(tokens).with_pipeline(CompilerPipeline::V130);
    let program = parser.parse().unwrap();

    let result = Analyzer::analyze(&program);
    assert!(result.is_ok(), "timeout_recv on valid channel should pass: {:?}", result.err());
}

// ─── 12. Full backpressure + scheduler scenario ───

#[test]
fn full_backpressure_scheduler_scenario() {
    let source = r#"
actor Producer {
    let out: Channel<Producer, Consumer, i32>
    let mut count = 0
    while (count < 10) {
        let ok = out.try_send(count)
        if (ok.is_ok()) {
            count = count + 1
        } else {
            yield()
        }
    }
}

actor Consumer {
    let in: Channel<Producer, Consumer, i32>
    let mut total = 0
    while (total < 45) {
        let msg = in.try_recv()
        if (msg.is_some()) {
            total = total + msg.unwrap()
        } else {
            yield()
        }
    }
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

    assert_eq!(program.statements.len(), 6); // 2 actors + 2 spawn + 2 join

    let result = Analyzer::analyze(&program);
    assert!(result.is_ok(), "Full backpressure scenario should pass: {:?}", result.err());
}

// ─── 13. Scheduler library parses ───

#[test]
fn scheduler_library_parses() {
    let source = r#"
struct Scheduler {
    actor_count: i32,
    current_actor: i32,
    active_mask: i32,
    yields_since_switch: i32
}

function sched_new(max_actors: i32) -> Scheduler {
    Scheduler {
        actor_count: 0,
        current_actor: 0,
        active_mask: 0xFFFFFFFF,
        yields_since_switch: 0
    }
}
"#;
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let parser = Parser::new(tokens).with_pipeline(CompilerPipeline::V130);
    let program = parser.parse().unwrap();

    assert_eq!(program.statements.len(), 2);
    match &program.statements[0] {
        Stmt::Struct { name, .. } => assert_eq!(name, "Scheduler"),
        other => panic!("Expected Struct, got {:?}", other),
    }
}

// ─── 14. sleep with variable duration ───

#[test]
fn parse_sleep_variable() {
    let source = r#"
actor Worker {
    let delay = 500
    sleep(delay)
}
"#;
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let parser = Parser::new(tokens).with_pipeline(CompilerPipeline::V130);
    let program = parser.parse().unwrap();

    let result = Analyzer::analyze(&program);
    assert!(result.is_ok(), "sleep with variable duration should pass: {:?}", result.err());
}
