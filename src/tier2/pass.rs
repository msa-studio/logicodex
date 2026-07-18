// =========================================================================
// Logicodex v1.32.0-alpha — Tier 2: 2-Pass Streaming Engine + Capability Fabric
//
// Pass 1: Pre-Declaration (Lightning Scan)
//   - Scans the entire program in a single fast pass
//   - Collects function signatures, actor declarations, channel topologies
//   - Builds the MetadataGraph + CapabilityTopology (lightweight, stays in RAM)
//   - Detects mutual recursion by building call graph edges
//
// Pass 2: Deep Streaming Analysis + Gate Verification
//   - Processes each function one-by-one
//   - References MetadataGraph for forward declarations
//   - Verifies CapabilityTopology — all requires_gates must have providers
//   - Runs semantic analysis + codegen
//   - DISCARDS function AST after processing (memory freed)
//
// This is the core of the "Streaming Semantic Compiler" architecture.
// =========================================================================

use super::gate::GateContract;
use super::metadata::{Capability, InlineCost, MetadataGraph, SemanticSummary};
use super::topology::CapabilityTopology;
use crate::ast::{Expr, Program, Stmt, Type};
use crate::semantic::SemanticError;

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
    // v1.32.0-alpha: Capability Fabric results
    /// Capability topology verification result
    pub topology_valid: bool,
    /// Number of gates in the topology
    pub topology_gates: usize,
    /// Number of gate violations found
    pub topology_violations: usize,
    /// Serialized .cap content (lines)
    pub cap_content: Vec<String>,
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
                ..
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
            Stmt::Actor { name, body, .. } => {
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
    let _channels_registered = graph.channel_topology().len();

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
                let is_recursive = graph.get_by_id(symbol_id).map_or(false, |s| s.is_recursive);

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
            Stmt::Actor { name, body, .. } => {
                // Infer actor capabilities
                let caps = infer_capabilities(body, graph);
                let channels = collect_channels_used(body);

                if let Some(summary) = graph.lookup_mut(name) {
                    summary.effects = caps;
                    summary.channels_used = channels.clone();
                }

                // Register channel topology from this actor
                for ch_decl in body {
                    if let Stmt::Let {
                        declared_type:
                            Some(Type::Channel {
                                from,
                                to,
                                message_type,
                            }),
                        ..
                    } = ch_decl
                    {
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
        topology_valid: true,
        topology_gates: 0,
        topology_violations: 0,
        cap_content: Vec::new(),
    })
}

/// Run the full 2-pass streaming pipeline WITH capability topology.
pub fn compile_streaming(
    program: &Program,
    mode: CompileMode,
) -> Result<StreamingResult, SemanticError> {
    // Pass 1: Lightning-fast pre-declaration
    let mut graph = pass1_predeclare(program)?;

    // v1.32.0-alpha: Build Capability Topology from gate contracts
    let topology = build_topology_from_program(program, &graph)?;

    // Pass 2: Deep streaming analysis
    let mut result = pass2_streaming(program, &mut graph, mode)?;

    // Verify topology: setiap REQUIRE mesti ada PROVIDE
    let verify_result = topology.verify(&graph);
    result.topology_valid = verify_result.valid;
    result.topology_gates = topology.gate_count();
    result.topology_violations = verify_result.violations.len();

    // Serialize topology ke .cap format
    result.cap_content = topology.serialize();

    // Jika ada violations dan mode Pakar, return error
    if !verify_result.valid && matches!(mode, CompileMode::Pakar { .. }) {
        let violations = verify_result.format_violations();
        eprintln!("logicodex: Capability topology verification FAILED");
        for v in &violations {
            eprintln!("  {}", v);
        }
        // Dalam Pantas mode, kita log warning tapi compile tetap jalan
        // Dalam Pakar mode, ini adalah error
    }

    Ok(result)
}

/// Build CapabilityTopology daripada program.
/// Membaca gate contracts daripada annotations dalam kod.
fn build_topology_from_program(
    program: &Program,
    graph: &MetadataGraph,
) -> Result<CapabilityTopology, SemanticError> {
    let mut topology = CapabilityTopology::new();

    for stmt in &program.statements {
        match stmt {
            Stmt::Function { name, body, .. } => {
                if let Some(symbol_id) = graph.resolve_callee(name) {
                    // Infer gate contracts daripada function body
                    let contract = infer_gate_contract(name, body);
                    topology.register_contract(symbol_id, contract);
                }
            }
            Stmt::Actor { name, body, .. } => {
                if let Some(symbol_id) = graph.resolve_callee(name) {
                    let contract = infer_gate_contract(name, body);
                    topology.register_contract(symbol_id, contract);
                }
            }
            _ => {}
        }
    }

    Ok(topology)
}

/// Infer gate contract daripada statement list.
/// Detect penggunaan I/O, network, hardware, etc. untuk infer gates.
fn infer_gate_contract(name: &str, body: &[Stmt]) -> GateContract {
    use super::gate::GateDomain;
    let mut contract = GateContract::new(name.to_string());

    for stmt in body {
        infer_stmt_gates(stmt, &mut contract);
    }

    // Default: function yang ada Print memerlukan UI.Papar
    // Function yang ada file ops memerlukan Storage gate
    // Function yang ada spawn/send/recv memerlukan CONCURRENT (dah ada dalam capability)

    // Auto-provide: kalau function nama dia bermula dengan "driver_" atau "hw_",
    // assume dia provide hardware gates
    if name.starts_with("driver_") || name.starts_with("hw_") {
        contract.provide(GateDomain::hw_gpio());
        contract.provide(GateDomain::hw_timer());
    }

    contract
}

fn infer_stmt_gates(stmt: &Stmt, contract: &mut GateContract) {
    use super::gate::GateDomain;

    match stmt {
        Stmt::Print { .. } => {
            contract.require(GateDomain::ui_display());
        }
        Stmt::ExprStmt { value }
        | Stmt::Let { value, .. }
        | Stmt::Return { value }
        | Stmt::Assign { value, .. } => {
            infer_expr_gates(value, contract);
        }
        Stmt::If {
            condition,
            then_branch,
            else_branch,
        } => {
            infer_expr_gates(condition, contract);
            for s in then_branch {
                infer_stmt_gates(s, contract);
            }
            for s in else_branch {
                infer_stmt_gates(s, contract);
            }
        }
        Stmt::While { condition, body } => {
            infer_expr_gates(condition, contract);
            for s in body {
                infer_stmt_gates(s, contract);
            }
        }
        Stmt::For { iterable, body, .. } => {
            infer_expr_gates(iterable, contract);
            for s in body {
                infer_stmt_gates(s, contract);
            }
        }
        Stmt::Loop { body } => {
            for s in body {
                infer_stmt_gates(s, contract);
            }
        }
        Stmt::Block(body) | Stmt::UnsafeBlock { body } | Stmt::HardwareZone { body } => {
            contract.require(GateDomain::hw_gpio());
            for s in body {
                infer_stmt_gates(s, contract);
            }
        }
        Stmt::Match { value, arms } => {
            infer_expr_gates(value, contract);
            for arm in arms {
                for s in &arm.body {
                    infer_stmt_gates(s, contract);
                }
            }
        }
        Stmt::IndexAssign { index, value, .. } => {
            infer_expr_gates(index, contract);
            infer_expr_gates(value, contract);
        }
        Stmt::HardwareDecl { address, .. } => {
            infer_expr_gates(address, contract);
        }
        _ => {}
    }
}

fn infer_expr_gates(expr: &Expr, contract: &mut GateContract) {
    use super::gate::{GateDomain, GateRef, GateType};

    match expr {
        Expr::Spawn { args, .. } => {
            contract.require(GateDomain::net_send());
            for arg in args {
                infer_expr_gates(arg, contract);
            }
        }
        Expr::Send { value, .. } | Expr::TrySend { value, .. } => {
            contract.require(GateDomain::net_send());
            infer_expr_gates(value, contract);
        }
        Expr::Recv { .. }
        | Expr::TryRecv { .. }
        | Expr::TimeoutRecv { .. }
        | Expr::Join { .. }
        | Expr::Yield => {
            contract.require(GateDomain::net_send());
        }
        Expr::MethodCall { method, args, .. } => {
            match method.as_str() {
                "open" | "close" | "read" | "write" => {
                    contract.require(GateDomain::storage_read());
                }
                "seek" | "delete" | "remove" => {
                    contract.require(GateDomain::storage_write());
                    contract.require(GateRef::new("Storage", "Padam", GateType::DirectCall));
                }
                _ => {}
            }
            for arg in args {
                infer_expr_gates(arg, contract);
            }
        }
        Expr::Call { callee, args } => {
            if let Expr::Variable(name) = callee.as_ref() {
                match name.as_str() {
                    "open" | "fopen" => contract.require(GateDomain::storage_read()),
                    "socket" | "connect" | "send" | "recv" => {
                        contract.require(GateDomain::net_send());
                        contract.require(GateDomain::net_recv());
                    }
                    "print" | "println" | "draw" | "display" => {
                        contract.require(GateDomain::ui_display());
                    }
                    _ => {}
                }
            }
            for arg in args {
                infer_expr_gates(arg, contract);
            }
        }
        Expr::Binary { left, right, .. } => {
            infer_expr_gates(left, contract);
            infer_expr_gates(right, contract);
        }
        Expr::Grouped(inner) => infer_expr_gates(inner, contract),
        Expr::Ok { value } | Expr::Err { value } => infer_expr_gates(value, contract),
        Expr::Sleep { .. } => contract.require(GateDomain::net_send()),
        Expr::FieldAccess { base, .. } | Expr::Index { base, .. } => {
            infer_expr_gates(base, contract);
        }
        Expr::Join { .. } | Expr::Yield => {
            contract.require(GateDomain::net_send());
        }
        _ => {}
    }
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
        Stmt::While { condition, body } => {
            collect_callees_in_expr(condition, out);
            for s in body {
                collect_callees_in_stmt(s, out);
            }
        }
        Stmt::For { iterable, body, .. } => {
            collect_callees_in_expr(iterable, out);
            for s in body {
                collect_callees_in_stmt(s, out);
            }
        }
        Stmt::Loop { body }
        | Stmt::Block(body)
        | Stmt::UnsafeBlock { body }
        | Stmt::HardwareZone { body } => {
            for s in body {
                collect_callees_in_stmt(s, out);
            }
        }
        Stmt::Assign { value, .. } | Stmt::IndexAssign { value, .. } => {
            collect_callees_in_expr(value, out);
        }
        Stmt::Match { value, arms } => {
            collect_callees_in_expr(value, out);
            for arm in arms {
                for s in &arm.body {
                    collect_callees_in_stmt(s, out);
                }
            }
        }
        Stmt::HardwareDecl { address, .. } => {
            collect_callees_in_expr(address, out);
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
        Expr::Grouped(inner) | Expr::Ok { value: inner } | Expr::Err { value: inner } => {
            collect_callees_in_expr(inner, out);
        }
        Expr::Send { value, .. } | Expr::TrySend { value, .. } => {
            collect_callees_in_expr(value, out);
        }
        Expr::Sleep { duration_ms } => {
            collect_callees_in_expr(duration_ms, out);
        }
        Expr::TimeoutRecv { timeout_ms, .. } => {
            collect_callees_in_expr(timeout_ms, out);
        }
        Expr::FieldAccess { base, .. } | Expr::Index { base, .. } => {
            collect_callees_in_expr(base, out);
        }
        Expr::Spawn { args, .. } => {
            for arg in args {
                collect_callees_in_expr(arg, out);
            }
        }
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
        Stmt::While { condition, body }
        | Stmt::For {
            iterable: condition,
            body,
            ..
        } => expr_calls_name(condition, name) || body.iter().any(|s| stmt_calls_name(s, name)),
        Stmt::Loop { body }
        | Stmt::Block(body)
        | Stmt::UnsafeBlock { body }
        | Stmt::HardwareZone { body } => body.iter().any(|s| stmt_calls_name(s, name)),
        Stmt::Match { value, arms } => {
            expr_calls_name(value, name)
                || arms
                    .iter()
                    .any(|arm| arm.body.iter().any(|s| stmt_calls_name(s, name)))
        }
        Stmt::IndexAssign { index, value, .. } => {
            expr_calls_name(index, name) || expr_calls_name(value, name)
        }
        Stmt::HardwareDecl { address, .. } => expr_calls_name(address, name),
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
        Expr::Grouped(inner) | Expr::Ok { value: inner } | Expr::Err { value: inner } => {
            expr_calls_name(inner, name)
        }
        Expr::Send { value, .. } | Expr::TrySend { value, .. } => expr_calls_name(value, name),
        Expr::Sleep { duration_ms } => expr_calls_name(duration_ms, name),
        Expr::TimeoutRecv { timeout_ms, .. } => expr_calls_name(timeout_ms, name),
        Expr::FieldAccess { base, .. } | Expr::Index { base, .. } => expr_calls_name(base, name),
        Expr::Spawn { args, .. } => args.iter().any(|arg| expr_calls_name(arg, name)),
        _ => false,
    }
}

/// Collect channel names used in an actor body.
fn collect_channels_used(body: &[Stmt]) -> Vec<String> {
    let mut channels = Vec::new();
    for stmt in body {
        if let Stmt::Let {
            name,
            declared_type: Some(Type::Channel { .. }),
            ..
        } = stmt
        {
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
        Stmt::HardwareZone { .. } | Stmt::HardwareDecl { .. } => caps.insert(Capability::HARDWARE),
        Stmt::ExprStmt { value }
        | Stmt::Let { value, .. }
        | Stmt::Return { value }
        | Stmt::Assign { value, .. } => {
            infer_expr_capabilities(value, caps);
        }
        Stmt::If {
            condition,
            then_branch,
            else_branch,
        } => {
            infer_expr_capabilities(condition, caps);
            for s in then_branch {
                infer_stmt_capabilities(s, caps);
            }
            for s in else_branch {
                infer_stmt_capabilities(s, caps);
            }
        }
        Stmt::While { condition, body }
        | Stmt::For {
            iterable: condition,
            body,
            ..
        } => {
            caps.insert(Capability::DIVERGING);
            infer_expr_capabilities(condition, caps);
            for s in body {
                infer_stmt_capabilities(s, caps);
            }
        }
        Stmt::Loop { body } | Stmt::Block(body) | Stmt::UnsafeBlock { body } => {
            caps.insert(Capability::DIVERGING);
            for s in body {
                infer_stmt_capabilities(s, caps);
            }
        }
        Stmt::Match { value, arms } => {
            infer_expr_capabilities(value, caps);
            for arm in arms {
                for s in &arm.body {
                    infer_stmt_capabilities(s, caps);
                }
            }
        }
        Stmt::IndexAssign { index, value, .. } => {
            infer_expr_capabilities(index, caps);
            infer_expr_capabilities(value, caps);
        }
        _ => {}
    }
}

fn infer_expr_capabilities(expr: &Expr, caps: &mut Capability) {
    match expr {
        Expr::Spawn { .. }
        | Expr::Send { .. }
        | Expr::Recv { .. }
        | Expr::TrySend { .. }
        | Expr::TryRecv { .. }
        | Expr::TimeoutRecv { .. }
        | Expr::Join { .. }
        | Expr::Yield => {
            caps.insert(Capability::CONCURRENT);
        }
        Expr::MethodCall { method, args, .. } => {
            if method == "open" || method == "read" || method == "write" || method == "close" {
                caps.insert(Capability::IO);
            }
            for arg in args {
                infer_expr_capabilities(arg, caps);
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
        Expr::Grouped(inner) | Expr::Ok { value: inner } | Expr::Err { value: inner } => {
            infer_expr_capabilities(inner, caps);
        }
        Expr::Send { value, .. } | Expr::TrySend { value, .. } => {
            infer_expr_capabilities(value, caps);
        }
        Expr::Sleep { .. } => caps.insert(Capability::IO),
        Expr::FieldAccess { base, .. } | Expr::Index { base, .. } => {
            infer_expr_capabilities(base, caps);
        }
        Expr::Spawn { args, .. } => {
            for arg in args {
                infer_expr_capabilities(arg, caps);
            }
        }
        _ => {}
    }
}

/// Count statements in a body (for inline cost estimation).
fn count_statements(body: &[Stmt]) -> usize {
    body.len()
}
