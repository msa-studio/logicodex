// =========================================================================
// Logicodex v1.31.0-alpha — Tier 2: 2-Pass Streaming Engine Tests
//
// Tests the MetadataGraph, SemanticSummary, Capability tracking,
// and the 2-Pass Engine (pre-declare + streaming analysis).
// =========================================================================

use logicodex::ast::{Expr, Stmt, Type};
use logicodex::lexer::{Lexer, Lexicon};
use logicodex::parser::{CompilerPipeline, Parser};
use logicodex::tier2::{
    Capability, CompileMode, InlineCost, MetadataGraph, SemanticSummary,
    compile_streaming, pass1_predeclare, pass2_streaming,
};

// ─── 1. Capability basics ───

#[test]
fn capability_pure_default() {
    let c = Capability::default();
    assert!(c.contains(Capability::PURE));
    assert!(!c.has_io());
    assert!(!c.has_unsafe());
    assert!(!c.has_concurrent());
}

#[test]
fn capability_merge_io() {
    let mut c = Capability::default();
    c.insert(Capability::IO);
    assert!(c.has_io());
    assert!(c.contains(Capability::PURE)); // PURE still set
}

#[test]
fn capability_merge_concurrent() {
    let mut c = Capability::default();
    c.insert(Capability::CONCURRENT);
    assert!(c.has_concurrent());
}

#[test]
fn capability_multiple() {
    let mut c = Capability::default();
    c.insert(Capability::IO);
    c.insert(Capability::CONCURRENT);
    c.insert(Capability::UNSAFE);
    assert!(c.has_io());
    assert!(c.has_concurrent());
    assert!(c.has_unsafe());
}

// ─── 2. InlineCost estimation ───

#[test]
fn inline_cost_trivial() {
    assert_eq!(InlineCost::from_statement_count(1, false), InlineCost::Trivial);
}

#[test]
fn inline_cost_small() {
    assert_eq!(InlineCost::from_statement_count(3, false), InlineCost::Small);
}

#[test]
fn inline_cost_medium() {
    assert_eq!(InlineCost::from_statement_count(7, false), InlineCost::Medium);
}

#[test]
fn inline_cost_large() {
    assert_eq!(InlineCost::from_statement_count(15, false), InlineCost::Large);
}

#[test]
fn inline_cost_recursive() {
    assert_eq!(InlineCost::from_statement_count(3, true), InlineCost::Recursive);
}

// ─── 3. SemanticSummary creation ───

#[test]
fn semantic_summary_function() {
    let s = SemanticSummary::new_function(
        42,
        "add".to_string(),
        vec![Type::I32, Type::I32],
        Some(Type::I32),
    );
    assert_eq!(s.symbol_id, 42);
    assert_eq!(s.name, "add");
    assert_eq!(s.params.len(), 2);
    assert_eq!(s.ret_type, Some(Type::I32));
    assert!(!s.is_actor);
    assert!(!s.is_recursive);
}

#[test]
fn semantic_summary_actor() {
    let s = SemanticSummary::new_actor(
        7,
        "Worker".to_string(),
        vec!["ch1".to_string(), "ch2".to_string()],
    );
    assert_eq!(s.symbol_id, 7);
    assert_eq!(s.name, "Worker");
    assert!(s.is_actor);
    assert!(s.has_concurrent());
    assert_eq!(s.channels_used.len(), 2);
}

#[test]
fn semantic_summary_size_small() {
    let s = SemanticSummary::new_function(
        1,
        "foo".to_string(),
        vec![Type::I64],
        Some(Type::I64),
    );
    let size = s.size_bytes();
    assert!(size < 500, "SemanticSummary should be < 500 bytes, got {}", size);
}

// ─── 4. MetadataGraph: register and lookup ───

#[test]
fn metadata_graph_register_lookup() {
    let mut g = MetadataGraph::new();
    let s = SemanticSummary::new_function(0, "main".to_string(), vec![], Some(Type::I64));
    g.register(s);

    assert!(g.has_symbol("main"));
    assert!(!g.has_symbol("unknown"));
    assert_eq!(g.symbol_count(), 1);

    let looked_up = g.lookup("main").unwrap();
    assert_eq!(looked_up.name, "main");
}

#[test]
fn metadata_graph_actor_registration() {
    let mut g = MetadataGraph::new();
    g.register_actor("Producer".to_string());
    g.register_actor("Consumer".to_string());

    assert!(g.is_actor("Producer"));
    assert!(g.is_actor("Consumer"));
    assert!(!g.is_actor("main"));
    assert_eq!(g.actor_names().len(), 2);
}

#[test]
fn metadata_graph_channel_topology() {
    let mut g = MetadataGraph::new();
    g.register_channel("Producer".to_string(), "Consumer".to_string(), "i32".to_string());

    assert_eq!(g.channel_topology().len(), 1);
    let (from, to, msg) = &g.channel_topology()[0];
    assert_eq!(from, "Producer");
    assert_eq!(to, "Consumer");
    assert_eq!(msg, "i32");
}

#[test]
fn metadata_graph_call_dependencies() {
    let mut g = MetadataGraph::new();
    let a = SemanticSummary::new_function(0, "A".to_string(), vec![], None);
    let b = SemanticSummary::new_function(1, "B".to_string(), vec![], None);
    g.register(a);
    g.register(b);

    g.add_call(0, 1); // A calls B

    assert!(g.resolve_callee("B").is_some());
    assert!(g.resolve_callee("A").is_some());
}

// ─── 5. MetadataGraph: mutual recursion detection ───

#[test]
fn metadata_graph_detects_mutual_recursion() {
    let mut g = MetadataGraph::new();
    let a = SemanticSummary::new_function(0, "even".to_string(), vec![Type::I32], Some(Type::Bool));
    let b = SemanticSummary::new_function(1, "odd".to_string(), vec![Type::I32], Some(Type::Bool));
    g.register(a);
    g.register(b);

    g.add_call(0, 1); // even calls odd
    g.add_call(1, 0); // odd calls even

    assert!(g.is_mutually_recursive(0, 1));
    assert!(g.is_mutually_recursive(1, 0));
}

#[test]
fn metadata_graph_no_false_mutual_recursion() {
    let mut g = MetadataGraph::new();
    let a = SemanticSummary::new_function(0, "A".to_string(), vec![], None);
    let b = SemanticSummary::new_function(1, "B".to_string(), vec![], None);
    g.register(a);
    g.register(b);

    g.add_call(0, 1); // A calls B (but B does NOT call A)

    assert!(!g.is_mutually_recursive(0, 1));
}

// ─── 6. Pass 1: Pre-declaration on simple program ───

#[test]
fn pass1_predeclare_simple() {
    let source = r#"
function add(a: i32, b: i32) -> i32 {
    return a + b
}

function main() {
    let result = add(1, 2)
    print(result)
}
"#;
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let parser = Parser::new(tokens);
    let program = parser.parse().unwrap();

    let graph = pass1_predeclare(&program).unwrap();

    assert_eq!(graph.symbol_count(), 2);
    assert!(graph.has_symbol("add"));
    assert!(graph.has_symbol("main"));
    assert!(!graph.is_actor("add"));
}

// ─── 7. Pass 1: Pre-declaration with actors ───

#[test]
fn pass1_predeclare_with_actors() {
    let source = r#"
actor Producer {
    let ch: Channel<Producer, Consumer, i32>
}

actor Consumer {
    let ch: Channel<Producer, Consumer, i32>
}
"#;
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let parser = Parser::new(tokens).with_pipeline(CompilerPipeline::V130);
    let program = parser.parse().unwrap();

    let graph = pass1_predeclare(&program).unwrap();

    assert_eq!(graph.symbol_count(), 2);
    assert!(graph.is_actor("Producer"));
    assert!(graph.is_actor("Consumer"));
    assert!(graph.has_symbol("Producer"));
    assert!(graph.has_symbol("Consumer"));
}

// ─── 8. Pass 1: Detects self-recursion ───

#[test]
fn pass1_detects_self_recursion() {
    let source = r#"
function factorial(n: i32) -> i32 {
    if (n <= 1) {
        return 1
    }
    return n * factorial(n - 1)
}
"#;
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let parser = Parser::new(tokens);
    let program = parser.parse().unwrap();

    let graph = pass1_predeclare(&program).unwrap();

    let summary = graph.lookup("factorial").unwrap();
    assert!(summary.is_recursive, "factorial should be detected as recursive");
    assert_eq!(format!("{:?}", summary.inline_cost), "Recursive");
}

// ─── 9. Full 2-pass: simple functions ───

#[test]
fn full_2pass_simple_functions() {
    let source = r#"
function add(a: i32, b: i32) -> i32 {
    return a + b
}

function main() {
    let x = add(1, 2)
    print(x)
}
"#;
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let parser = Parser::new(tokens);
    let program = parser.parse().unwrap();

    let result = compile_streaming(&program, CompileMode::Pantas).unwrap();

    assert_eq!(result.functions_compiled, 2);
    assert_eq!(result.actors_registered, 0);
    // Metadata should be much smaller than estimated AST
    assert!(
        result.metadata_memory_bytes < result.estimated_ast_memory_bytes,
        "Metadata ({}B) should be smaller than AST estimate ({}B)",
        result.metadata_memory_bytes,
        result.estimated_ast_memory_bytes
    );
}

// ─── 10. Full 2-pass: actor with channels ───

#[test]
fn full_2pass_actor_channels() {
    let source = r#"
actor Producer {
    let out: Channel<Producer, Consumer, i32>
    out.send(42)
}

actor Consumer {
    let in_ch: Channel<Producer, Consumer, i32>
    let msg = in_ch.recv()
}
"#;
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let parser = Parser::new(tokens).with_pipeline(CompilerPipeline::V130);
    let program = parser.parse().unwrap();

    let result = compile_streaming(&program, CompileMode::Pantas).unwrap();

    assert_eq!(result.actors_registered, 2);
    // Actors should have CONCURRENT capability
}

// ─── 11. Full 2-pass: Pakar mode ───

#[test]
fn full_2pass_pakar_mode() {
    let source = r#"
function greet() {
    print("hello")
}
"#;
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let parser = Parser::new(tokens);
    let program = parser.parse().unwrap();

    let result = compile_streaming(&program, CompileMode::Pakar { max_ram_mb: 256 }).unwrap();

    assert_eq!(result.functions_compiled, 1);
    assert_eq!(result.mode, CompileMode::Pakar { max_ram_mb: 256 });
}

// ─── 12. MetadataGraph memory is small ───

#[test]
fn metadata_graph_memory_efficient() {
    let mut g = MetadataGraph::new();
    for i in 0..100 {
        let s = SemanticSummary::new_function(
            i,
            format!("func_{}", i),
            vec![Type::I32, Type::I32],
            Some(Type::I64),
        );
        g.register(s);
    }

    let mem = g.total_memory_bytes();
    // 100 functions should fit in < 20KB
    assert!(
        mem < 20_000,
        "100-function MetadataGraph should be < 20KB, got {}B",
        mem
    );
}
