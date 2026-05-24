// =========================================================================
// Logicodex v1.31.0-alpha — Tier 2: 2-Pass Streaming Engine
//
// Pass 1: Pre-Declaration (Lightning Scan)
//   - Scans the entire program in a single fast pass
//   - Collects function signatures, actor declarations, channel topologies
//   - Builds the MetadataGraph (lightweight, stays in RAM)
//   - Detects mutual recursion by building call graph edges
//
// Pass 2: Deep Streaming Analysis
//   - Processes each function one-by-one
//   - References MetadataGraph for forward declarations
//   - Runs semantic analysis + codegen
//   - DISCARDS function AST after processing (memory freed)
//
// This is the core of the "Streaming Semantic Compiler" architecture.
// =========================================================================

use crate::ast::{Expr, Program, Stmt, Type};
use crate::semantic::SemanticError;
use super::metadata::{Capability, InlineCost, MetadataGraph, SemanticSummary};

/// Result of running both passes.
#[derive(Debug)]
pub struct StreamingResult {
    /// Number of functions successfully compiled
    pub functions_compiled: usize,
    /// Number of actors registered
    pub actors_registered: usize,
    /// Number of channels registered
    pub channels_registered: usize,
    /// Total metadata graph memory (bytes)
    pub metadata_memory_bytes: usize,
    /// Estimated AST memory that would have been kept (bytes)
    pub estimated_ast_memory_bytes: usize,
    /// Compilation mode used
    pub mode: CompileMode,
}

/// Compilation mode — controls streaming behavior and memory limits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompileMode {
    /// Fast mode: aggressive streaming, minimal LTO, immediate memory discard
    /// Suitable for responsive development.
    Pantas,
    /// Expert mode: full optimization within memory budget
    /// Adaptive window shrinks semantic cache when RAM is nearly full.
    Pakar { max_ram_mb: usize },
}

impl Default for CompileMode {
    fn default() -> Self {
        CompileMode::Pantas
    }
}

impl CompileMode {
    pub fn from_cli_args(pantas: bool, max_ram_mb: Option<usize>) -> Self {
        if pantas || max_ram_mb.is_none() {
            CompileMode::Pantas
        } else {
            CompileMode::Pakar {
                max_ram_mb: max_ram_mb.unwrap_or(512),
            }
        }
    }

    /// Should we drop function AST immediately after codegen?
    pub fn aggressive_discard(&self) -> bool {
        matches!(self, CompileMode::Pantas)
    }

    /// Maximum RAM budget in bytes (None = unlimited).
    pub fn max_ram_bytes(&self) -> Option<usize> {
        match self {
            CompileMode::Pantas => None,
            CompileMode::Pakar { max_ram_mb } => Some(max_ram_mb * 1024 * 1024),
        }
    }
}

// ─── Pass 1: Pre-Declaration ───

/// Lightning-fast scan to collect all top-level declarations.
/// This stays in RAM throughout compilation — it's the "index".
pub fn pass1_predeclare(program: &Program) -> Result<MetadataGraph, SemanticError> {
    let mut graph = MetadataGraph::new();

    for stmt in &program.statements {
        match stmt {
            Stmt::Function {
                name,
                params,
                return_type,
                body,
            } => {
                let id = graph.alloc_id();
                let param_types: Vec<Type> = params.iter().map(|p| p.ty.clone()).collect();
                let summary = SemanticSummary::new_function(
                    id,
                    name.clone(),
                    param_types,
                    return_type.clone(),
                );
                graph.register(summary);

                // Detect direct recursion: does this function call itself?
                let is_recursive = detect_self_call(name, body);
                if is_recursive {
                    if let Some(s) = graph.lookup_mut(name) {
                        s.is_recursive = true;
                        s.inline_cost = InlineCost::Recursive;
                    }
                }
            }
            Stmt::Actor { name, body } => {
                let id = graph.alloc_id();
                // Collect channels used by this actor
                let channels = collect_channels_used(body);
                let summary = SemanticSummary::new_actor(id, name.clone(), channels);
                graph.register(summary);
                graph.register_actor(name.clone());
            }
            _ => {} // Other statements are not top-level declarations
        }
    }

    // Second sub-pass: build call graph edges and detect mutual recursion
    // We need all symbol IDs allocated before we can resolve callees
    for stmt in &program.statements {
        if let Stmt::Function { name, body, .. } = stmt {
            if let Some(caller_id) = graph.resolve_callee(name) {
                let callees = collect_callee_names(body);
                for callee_name in callees {
                    if let Some(callee_id) = graph.resolve_callee(&callee_name) {
                        graph.add_call(caller_id, callee_id);
                        // Check for mutual recursion
                        if graph.is_mutually_recursive(caller_id, callee_id) {
                            if let Some(s) = graph.lookup_mut(name) {
                                s.is_recursive = true;
                                s.inline_cost = InlineCost::Recursive;
                            }
                            if let Some(s) = graph.lookup_mut(&callee_name) {
                                s.is_recursive = true;
                                s.inline_cost = InlineCost::Recursive;
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(graph)
}

// ─── Pass 2: Deep Streaming ───

/// Analyzes and "compiles" each function individually.
/// In full implementation, this would emit LLVM IR and drop the function AST.
/// For v1.31.0-alpha, we run semantic analysis and mark the function as done.
pub fn pass2_streaming(
    program: &Program,
    graph: &mut MetadataGraph,
    mode: CompileMode,
) -> Result<StreamingResult, SemanticError> {
    let mut functions_compiled = 0;
    let mut actors_registered = graph.actor_names().len();
    let mut channels_registered = graph.channel_topology().len();

    // Estimate AST memory (rough: ~500 bytes per statement)
    let estimated_ast_bytes = program.statements.len() * 500;

    for stmt in &program.statements {
        match stmt {
            Stmt::Function { name, body, .. } => {
                // Resolve this function in the metadata graph
                let symbol_id = graph
                    .resolve_callee(name)
                    .ok_or_else(|| SemanticError::UndefinedVariable(name.clone()))?;

                // Infer capabilities from function body
                let caps = infer_capabilities(body, graph);
                let stmt_count = count_statements(body);
                let is_recursive = graph
                    .get_by_id(symbol_id)
                    .map_or(false, |s| s.is_recursive);

                if let Some(summary) = graph.lookup_mut(name) {
                    summary.effects = caps;
                    summary.inline_cost =
                        InlineCost::from_statement_count(stmt_count, is_recursive);
                }

                functions_compiled += 1;

                // In aggressive discard mode, we would drop the function body here
                // after LLVM IR emission. For now, we just mark it as compiled.
                if mode.aggressive_discard() {
                    // Placeholder: discard_function_body(name);
                }
            }
            Stmt::Actor { name, body } => {
                // Infer actor capabilities
                let caps = infer_capabilities(body, graph);
                let channels = collect_channels_used(body);

                if let Some(summary) = graph.lookup_mut(name) {
                    summary.effects = caps;
                    summary.channels_used = channels.clone();
                }

                // Register channel topology from this actor
                for ch_decl in body {
                    if let Stmt::Let { declared_type: Some(Type::Channel { from, to, message_type }), .. } = ch_decl {
                        graph.register_channel(from.clone(), to.clone(), message_type.clone());
                    }
                }

                actors_registered += 1;
            }
            _ => {} // Non-declaration statements processed in semantic phase
        }
    }

    let metadata_memory = graph.total_memory_bytes();

    Ok(StreamingResult {
        functions_compiled,
        actors_registered,
        channels_registered: graph.channel_topology().len(),
        metadata_memory_bytes: metadata_memory,
        estimated_ast_memory_bytes: estimated_ast_bytes,
        mode,
    })
}

/// Run the full 2-pass streaming pipeline.
pub fn compile_streaming(
    program: &Program,
    mode: CompileMode,
) -> Result<StreamingResult, SemanticError> {
    // Pass 1: Lightning-fast pre-declaration
    let mut graph = pass1_predeclare(program)?;

    // Pass 2: Deep streaming analysis
    pass2_streaming(program, &mut graph, mode)
}

// ─── Helper Functions ───

/// Detect if a function calls itself (direct recursion).
fn detect_self_call(func_name: &str, body: &[Stmt]) -> bool {
    for stmt in body {
        if stmt_calls_name(stmt, func_name) {
            return true;
        }
    }
    false
}

/// Collect all function names called within a statement list.
fn collect_callee_names(body: &[Stmt]) -> Vec<String> {
    let mut names = Vec::new();
    for stmt in body {
        collect_callees_in_stmt(stmt, &mut names);
    }
    names
}

fn collect_callees_in_stmt(stmt: &Stmt, out: &mut Vec<String>) {
    match stmt {
        Stmt::ExprStmt { value } | Stmt::Print { value } => {
            collect_callees_in_expr(value, out);
        }
        Stmt::Let { value, .. } => {
            collect_callees_in_expr(value, out);
        }
        Stmt::Return { value } => {
            collect_callees_in_expr(value, out);
        }
        Stmt::If {
            condition,
            then_branch,
            else_branch,
        } => {
            collect_callees_in_expr(condition, out);
            for s in then_branch {
                collect_callees_in_stmt(s, out);
            }
            for s in else_branch {
                collect_callees_in_stmt(s, out);
            }
        }
        Stmt::While { condition, body } | Stmt::Loop { body } => {
            collect_callees_in_expr(condition, out);
            for s in body {
                collect_callees_in_stmt(s, out);
            }
        }
        Stmt::Assign { value, .. } => {
            collect_callees_in_expr(value, out);
        }
        _ => {}
    }
}

fn collect_callees_in_expr(expr: &Expr, out: &mut Vec<String>) {
    match expr {
        Expr::Call { callee, args } => {
            if let Expr::Variable(name) = callee.as_ref() {
                out.push(name.clone());
            }
            for arg in args {
                collect_callees_in_expr(arg, out);
            }
        }
        Expr::Binary { left, right, .. } => {
            collect_callees_in_expr(left, out);
            collect_callees_in_expr(right, out);
        }
        Expr::Grouped(inner) => collect_callees_in_expr(inner, out),
        Expr::Ok { value } | Expr::Err { value } => collect_callees_in_expr(value, out),
        Expr::Send { value, .. } => collect_callees_in_expr(value, out),
        Expr::Sleep { duration_ms } => collect_callees_in_expr(duration_ms, out),
        Expr::TimeoutRecv { timeout_ms, .. } => collect_callees_in_expr(timeout_ms, out),
        _ => {}
    }
}

/// Check if a statement contains a call to a specific function name.
fn stmt_calls_name(stmt: &Stmt, name: &str) -> bool {
    match stmt {
        Stmt::ExprStmt { value } | Stmt::Print { value } => expr_calls_name(value, name),
        Stmt::Let { value, .. } | Stmt::Return { value } | Stmt::Assign { value, .. } => {
            expr_calls_name(value, name)
        }
        Stmt::If {
            condition,
            then_branch,
            else_branch,
        } => {
            expr_calls_name(condition, name)
                || then_branch.iter().any(|s| stmt_calls_name(s, name))
                || else_branch.iter().any(|s| stmt_calls_name(s, name))
        }
        Stmt::While { condition, body } | Stmt::Loop { body } => {
            expr_calls_name(condition, name) || body.iter().any(|s| stmt_calls_name(s, name))
        }
        _ => false,
    }
}

fn expr_calls_name(expr: &Expr, name: &str) -> bool {
    match expr {
        Expr::Call { callee, .. } => {
            if let Expr::Variable(callee_name) = callee.as_ref() {
                callee_name == name
            } else {
                false
            }
        }
        Expr::Binary { left, right, .. } => {
            expr_calls_name(left, name) || expr_calls_name(right, name)
        }
        Expr::Grouped(inner) => expr_calls_name(inner, name),
        Expr::Ok { value } | Expr::Err { value } => expr_calls_name(value, name),
        Expr::Send { value, .. } => expr_calls_name(value, name),
        Expr::Sleep { duration_ms } => expr_calls_name(duration_ms, name),
        Expr::TimeoutRecv { timeout_ms, .. } => expr_calls_name(timeout_ms, name),
        _ => false,
    }
}

/// Collect channel names used in an actor body.
fn collect_channels_used(body: &[Stmt]) -> Vec<String> {
    let mut channels = Vec::new();
    for stmt in body {
        if let Stmt::Let { name, declared_type: Some(Type::Channel { .. }), .. } = stmt {
            channels.push(name.clone());
        }
    }
    channels
}

/// Infer capabilities from a statement list.
fn infer_capabilities(body: &[Stmt], _graph: &MetadataGraph) -> Capability {
    let mut caps = Capability::default();
    for stmt in body {
        infer_stmt_capabilities(stmt, &mut caps);
    }
    caps
}

fn infer_stmt_capabilities(stmt: &Stmt, caps: &mut Capability) {
    match stmt {
        Stmt::Print { .. } => caps.insert(Capability::IO),
        Stmt::HardwareZone { .. } => caps.insert(Capability::HARDWARE),
        Stmt::ExprStmt { value } | Stmt::Let { value, .. } | Stmt::Return { value } | Stmt::Assign { value, .. } => {
            infer_expr_capabilities(value, caps);
        }
        Stmt::If { condition, then_branch, else_branch } => {
            infer_expr_capabilities(condition, caps);
            for s in then_branch { infer_stmt_capabilities(s, caps); }
            for s in else_branch { infer_stmt_capabilities(s, caps); }
        }
        Stmt::While { condition, body } | Stmt::Loop { body } => {
            caps.insert(Capability::DIVERGING);
            infer_expr_capabilities(condition, caps);
            for s in body { infer_stmt_capabilities(s, caps); }
        }
        _ => {}
    }
}

fn infer_expr_capabilities(expr: &Expr, caps: &mut Capability) {
    match expr {
        Expr::Spawn { .. } | Expr::Send { .. } | Expr::Recv { .. }
        | Expr::TrySend { .. } | Expr::TryRecv { .. } | Expr::TimeoutRecv { .. } => {
            caps.insert(Capability::CONCURRENT);
        }
        Expr::MethodCall { method, .. } => {
            if method == "open" || method == "read" || method == "write" || method == "close" {
                caps.insert(Capability::IO);
            }
        }
        Expr::Call { callee, args } => {
            if let Expr::Variable(name) = callee.as_ref() {
                if name == "open" || name == "read" || name == "write" || name == "print" {
                    caps.insert(Capability::IO);
                }
                if name == "unsafe" {
                    caps.insert(Capability::UNSAFE);
                }
            }
            for arg in args {
                infer_expr_capabilities(arg, caps);
            }
        }
        Expr::Binary { left, right, .. } => {
            infer_expr_capabilities(left, caps);
            infer_expr_capabilities(right, caps);
        }
        Expr::Grouped(inner) => infer_expr_capabilities(inner, caps),
        Expr::Ok { value } | Expr::Err { value } => infer_expr_capabilities(value, caps),
        Expr::Send { value, .. } => infer_expr_capabilities(value, caps),
        Expr::Sleep { .. } => caps.insert(Capability::IO),
        _ => {}
    }
}

/// Count statements in a body (for inline cost estimation).
fn count_statements(body: &[Stmt]) -> usize {
    body.len()
}
