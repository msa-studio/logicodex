//! CPB-2 PR 1.2 — callable semantic consumers.
//!
//! The semantic gate and LLVM codegen consume one canonical artifact: the
//! frozen callable metadata carried by `LoweredModule`. Missing, duplicate,
//! stale, or provenance-invalid metadata fails closed before LLVM
//! construction, only frozen user externs have FFI provenance, and the
//! retired plain-HIR codegen entry point is rejected.

use std::collections::{BTreeSet, HashSet};
use std::path::{Path, PathBuf};
use std::process::Command;

use logicodex::codegen::{self, CodegenOptions, PLAIN_HIR_CODEGEN_RETIRED};
use logicodex::ffi::{
    CallableRegistry, CallableSafety, CallableSignature, CallingConvention, CapabilityPolicy,
    SafetyContext,
};
use logicodex::hir::{
    HirBlock, HirCallableKind, HirExpr, HirExprKind, HirItem, HirStmt, LiteralAst, LoweredModule,
    LoweringContext, SymbolTable,
};
use logicodex::lexer::{Lexer, Lexicon};
use logicodex::os::target::CompilationTarget;
use logicodex::parser::{CompilerPipeline, Parser};
use logicodex::semantic_gate::{self, SemanticContext};
use logicodex::span::{Diagnostic, DiagnosticCode, FileId, Span};
use logicodex::types::{CallableId, TypeId, TypeRef, TypeRegistry};

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_logicodex")
}

fn dict() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("dict/core_map.json")
}

fn scratch(name: &str) -> PathBuf {
    let dir =
        std::env::temp_dir().join(format!("ldx_cpb2_consumers_{name}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create scratch dir");
    dir
}

fn lower(source: &str) -> (LoweredModule, TypeRegistry) {
    let lexicon = Lexicon::from_path(&dict()).expect("load dictionary");
    let tokens = Lexer::new(source, &lexicon).tokenize().expect("tokenize");
    let program = Parser::new(tokens)
        .with_pipeline(CompilerPipeline::V130)
        .parse()
        .expect("parse");
    let mut symbols = SymbolTable::default();
    let mut types = TypeRegistry::new();
    let lowered = {
        let mut lowering = LoweringContext {
            symbols: &mut symbols,
            types: &mut types,
            diagnostics: Vec::new(),
            current_module: String::new(),
            current_return_enum: None,
        };
        lowering
            .lower_program_with_metadata(program)
            .expect("lower with metadata")
    };
    (lowered, types)
}

fn policy(allowed: &[&str]) -> CapabilityPolicy {
    let mut policy = CapabilityPolicy::with_runtime_builtins();
    for symbol in allowed {
        policy.allow_symbol(*symbol);
    }
    policy
}

fn gate(
    lowered: &LoweredModule,
    types: &TypeRegistry,
    registry: &CallableRegistry,
    allowed: &[&str],
) -> Result<(), Vec<Diagnostic>> {
    let mut ctx = SemanticContext {
        types: types.clone(),
        symbols: SymbolTable::default(),
        callables: registry.clone(),
        diagnostics: Vec::new(),
        loop_depth: 0,
        safety_context: SafetyContext::Safe,
        current_return_type: None,
        policy: policy(allowed),
        extern_symbols: HashSet::new(),
    };
    ctx.check_lowered_module(lowered)
}

fn options(name: &str) -> CodegenOptions {
    CodegenOptions {
        module_name: name.to_string(),
        emit_ir: true,
        secure: false,
        target: CompilationTarget::Native,
    }
}

/// Compile a lowered artifact to an object (and IR) in a fresh directory.
/// Returns the result plus the object and IR paths codegen would write.
fn codegen_lowered(
    name: &str,
    lowered: &LoweredModule,
    types: &TypeRegistry,
    registry: &CallableRegistry,
    allowed: &[&str],
) -> (anyhow::Result<codegen::CodegenArtifact>, PathBuf, PathBuf) {
    let object = scratch(name).join(format!("{name}.o"));
    // Clean-artifact precondition: neither the object nor its IR exists before
    // the production entry point runs (the scratch directory is fresh).
    assert!(
        !object.exists() && !object.with_extension("ll").exists(),
        "artifact paths must be clean before codegen"
    );
    let result = codegen::compile_v130_lowered(
        lowered,
        &object,
        &options(name),
        registry.clone(),
        types.clone(),
        policy(allowed),
    );
    let ir = object.with_extension("ll");
    (result, object, ir)
}

/// One expected diagnostic: exact code, exact English message, exact span.
type Expected<'a> = (DiagnosticCode, &'a str, Span);

/// Source span in file 0 (lines and columns are 1-based as produced upstream).
fn at(start_line: u32, start_col: u32, end_line: u32, end_col: u32) -> Span {
    Span::new(FileId(0), start_line, start_col, end_line, end_col)
}

/// Assert the exact, ordered diagnostic vector: count, code, message_en, and
/// primary span of every diagnostic.
fn assert_exact(diagnostics: &[Diagnostic], expected: &[Expected]) {
    assert!(
        !expected.is_empty(),
        "an expected diagnostic vector is required"
    );
    let actual: Vec<Expected> = diagnostics
        .iter()
        .map(|d| (d.code, d.message_en.as_str(), d.primary_span))
        .collect();
    assert_eq!(actual, expected, "exact diagnostic vector mismatch");
}

/// Assert that the semantic gate rejects `lowered` with exactly `expected`,
/// and that codegen fails with exactly those gate diagnostics and writes no
/// object or IR. Because `compile_v130_lowered` reports this message only from
/// the defensive gate step that precedes `Context::create`, equality proves
/// the rejection happened before LLVM construction.
fn assert_rejected_before_llvm(
    name: &str,
    lowered: &LoweredModule,
    types: &TypeRegistry,
    registry: &CallableRegistry,
    allowed: &[&str],
    expected: &[Expected],
) {
    let diagnostics = assert_codegen_fails_at_gate(name, lowered, types, registry, allowed);
    assert_exact(&diagnostics, expected);
}

/// Directly validate `lowered`, require rejection, and require codegen to fail
/// with exactly those gate diagnostics and to write no object or IR. Returns
/// the gate diagnostics for the caller's exact-vector assertion.
fn assert_codegen_fails_at_gate(
    name: &str,
    lowered: &LoweredModule,
    types: &TypeRegistry,
    registry: &CallableRegistry,
    allowed: &[&str],
) -> Vec<Diagnostic> {
    let diagnostics = semantic_gate::validate_lowered_module(
        lowered,
        types.clone(),
        registry.clone(),
        policy(allowed),
    )
    .expect_err("semantic gate must reject");

    let (result, object, ir) = codegen_lowered(name, lowered, types, registry, allowed);
    let error = result.err().expect("codegen must reject").to_string();
    let expected = format!(
        "semantic gate rejected lowered codegen input:\n{}",
        diagnostics
            .iter()
            .map(|d| format!("[{:?}] {}", d.code, d.message_en))
            .collect::<Vec<_>>()
            .join("\n")
    );
    assert_eq!(error, expected, "codegen must fail at the semantic gate");
    assert!(!object.exists(), "no object may be emitted");
    assert!(!ir.exists(), "no LLVM IR may be emitted");
    diagnostics
}

fn assert_integrity_rejected(
    name: &str,
    lowered: &LoweredModule,
    types: &TypeRegistry,
    expected: &[Expected],
) {
    assert_rejected_before_llvm(
        name,
        lowered,
        types,
        &CallableRegistry::default(),
        &[],
        expected,
    );
}

fn callable_named(lowered: &LoweredModule, name: &str) -> CallableId {
    lowered
        .callable_signatures
        .values()
        .find(|signature| signature.name == name)
        .unwrap_or_else(|| panic!("frozen metadata for `{name}`"))
        .callable
}

fn unsafe_extern(name: &str, params: Vec<TypeId>, return_type: TypeId) -> CallableSignature {
    CallableSignature {
        name: name.to_string(),
        params,
        return_type,
        abi: CallingConvention::C,
        safety: CallableSafety::UnsafeRequired,
        is_extern: true,
        is_variadic: false,
    }
}

/// Compile with the real CLI and run the program; require compilation to
/// succeed and return the process output.
fn build_and_run(name: &str, files: &[(&str, &str)]) -> std::process::Output {
    let dir = scratch(name);
    for (file, source) in files {
        std::fs::write(dir.join(file), source).expect("write fixture");
    }
    let root = dir.join(files[0].0);
    let exe = dir.join(name);
    let compile = Command::new(bin())
        .arg("compile")
        .arg("--dict")
        .arg(dict())
        .arg("-o")
        .arg(&exe)
        .arg(&root)
        .output()
        .expect("spawn logicodex compile");
    assert!(
        compile.status.success(),
        "compile failed for {name}:\n{}",
        String::from_utf8_lossy(&compile.stderr)
    );
    Command::new(&exe).output().expect("run compiled program")
}

/// Run a program with an explicit `main() -> I64` returning 0; require a
/// successful exit status and return stdout.
fn compile_and_run(name: &str, files: &[(&str, &str)]) -> String {
    let run = build_and_run(name, files);
    assert!(
        run.status.success(),
        "program {name} exited with {:?}:\nstdout:\n{}\nstderr:\n{}",
        run.status,
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    String::from_utf8_lossy(&run.stdout).into_owned()
}

/// Run a program whose root has top-level statements (required by `import`).
/// The synthesized `main` wrapper's exit status is not defined by the current
/// lowering (pre-existing, outside this callable-consumer scope), so this
/// requires normal termination by exit code (no signal) rather than status 0.
fn compile_and_run_top_level(name: &str, files: &[(&str, &str)]) -> String {
    let run = build_and_run(name, files);
    assert!(
        run.status.code().is_some(),
        "program {name} terminated abnormally: {:?}",
        run.status
    );
    String::from_utf8_lossy(&run.stdout).into_owned()
}

// ---- exhaustive HIR call/spawn traversal ------------------------------------

/// Visit every call and spawn expression in every HIR function (top-level
/// statements live in the lowered `main` wrapper and actor bodies in their
/// `__actor_*` functions). Matches are exhaustive with no wildcard, so a new
/// HIR statement or expression variant fails to compile here until covered.
fn visit_callable_refs(lowered: &mut LoweredModule, visit: &mut dyn FnMut(&mut HirExpr)) {
    for item in &mut lowered.module.items {
        match &mut item.node {
            HirItem::Function(function) => visit_block(&mut function.body, visit),
            HirItem::Struct(_) | HirItem::Enum(_) | HirItem::ExternFunction(_) => {}
        }
    }
}

fn visit_block(block: &mut HirBlock, visit: &mut dyn FnMut(&mut HirExpr)) {
    for stmt in &mut block.statements {
        match &mut stmt.node {
            HirStmt::Let { value, .. } => {
                if let Some(value) = value {
                    visit_expr(value, visit);
                }
            }
            HirStmt::Assign { target, value } => {
                visit_expr(target, visit);
                visit_expr(value, visit);
            }
            HirStmt::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                visit_expr(condition, visit);
                visit_block(then_branch, visit);
                if let Some(else_branch) = else_branch {
                    visit_block(else_branch, visit);
                }
            }
            HirStmt::While { condition, body } => {
                visit_expr(condition, visit);
                visit_block(body, visit);
            }
            HirStmt::Loop { body } | HirStmt::UnsafeBlock(body) | HirStmt::HardwareZone(body) => {
                visit_block(body, visit)
            }
            HirStmt::Expr(expr) => visit_expr(expr, visit),
            HirStmt::Return(expr) => {
                if let Some(expr) = expr {
                    visit_expr(expr, visit);
                }
            }
            HirStmt::Break { .. } | HirStmt::Continue { .. } | HirStmt::HardwareDecl { .. } => {}
        }
    }
}

fn visit_expr(expr: &mut HirExpr, visit: &mut dyn FnMut(&mut HirExpr)) {
    match &mut expr.kind {
        HirExprKind::Call { args, .. } | HirExprKind::Spawn { args, .. } => {
            for arg in args {
                visit_expr(arg, visit);
            }
        }
        HirExprKind::ArrayLiteral { elements } => {
            for element in elements {
                visit_expr(element, visit);
            }
        }
        HirExprKind::Binary { left, right, .. } => {
            visit_expr(left, visit);
            visit_expr(right, visit);
        }
        HirExprKind::Index { base, index } => {
            visit_expr(base, visit);
            visit_expr(index, visit);
        }
        HirExprKind::ChannelSend { channel, value } => {
            visit_expr(channel, visit);
            visit_expr(value, visit);
        }
        HirExprKind::Unary { expr: inner, .. }
        | HirExprKind::Cast { expr: inner, .. }
        | HirExprKind::Field { base: inner, .. }
        | HirExprKind::ResultOk { value: inner }
        | HirExprKind::ResultErr { value: inner }
        | HirExprKind::OptionSome { value: inner }
        | HirExprKind::ChannelCreate { capacity: inner }
        | HirExprKind::ChannelRecv { channel: inner }
        | HirExprKind::ChannelTrySend { value: inner, .. }
        | HirExprKind::Sleep { duration_ms: inner }
        | HirExprKind::ChannelTimeoutRecv {
            timeout_ms: inner, ..
        } => visit_expr(inner, visit),
        HirExprKind::Literal(_)
        | HirExprKind::Local(_)
        | HirExprKind::Global(_)
        | HirExprKind::OptionNone
        | HirExprKind::Join { .. }
        | HirExprKind::ChannelTryRecv { .. }
        | HirExprKind::Yield => {}
    }
    if matches!(
        expr.kind,
        HirExprKind::Call { .. } | HirExprKind::Spawn { .. }
    ) {
        visit(expr);
    }
}

// ---- fixtures ----------------------------------------------------------------

const ORDINARY: &str = "function add(a: I64, b: I64) -> I64 begin\n    return a + b;\nend\nfunction main() -> I64 begin\n    PAPAR add(2, 3);\n    return 0;\nend\n";

const EXTERNS: &str = "fn main() -> I64 begin\n    UNSAFE begin\n        second(first(1));\n    end\n    return 0;\nend\nextern \"C\" fn first(value: I64) -> I64;\nextern \"C\" fn second(value: I64) -> I64;\n";

/// Shared base for the operand matrices: an ordinary callable `add`, an actor,
/// and a spawn, all in one artifact so every CallableId is consistent.
const OPERAND_BASE: &str = "function add(a: I64, b: I64) -> I64 begin\n    return a + b;\nend\nACTOR pekerja MULA\n    PAPAR 1;\nTAMAT\nSPAWN pekerja();\n";

const ACTOR: &str = "ACTOR pekerja MULA\n    PAPAR 1;\nTAMAT\nSPAWN pekerja();\n";

/// Calls in let, assignment, if/else, while, nested arguments, binary
/// operands, return, top-level statements, an actor body, plus a spawn.
const WALK: &str = "function inc(a: I64) -> I64 begin\n    return a + 1;\nend\nfunction twice(a: I64) -> I64 begin\n    BINA x: I64 = inc(inc(a));\n    if x > inc(0) begin\n        x = inc(x);\n    end else begin\n        x = inc(0);\n    end\n    while x < inc(9) begin\n        x = inc(x);\n    end\n    return inc(x) + inc(1);\nend\nACTOR pekerja MULA\n    PAPAR inc(1);\nTAMAT\nPAPAR twice(1);\nSPAWN pekerja();\n";

// ---- success paths ---------------------------------------------------------

#[test]
fn ordinary_call_compiles_through_frozen_metadata() {
    assert_eq!(
        compile_and_run("ordinary", &[("main.ldx", ORDINARY)]).trim(),
        "5"
    );
}

#[test]
fn constructor_call_routes_by_frozen_constructor_kind() {
    let source = "BENTUK Titik MULA\n    x: I32;\n    y: I32;\nTAMAT\nfunction main() -> I64 begin\n    BINA p: Titik = Titik(3, 4);\n    PAPAR p.x;\n    PAPAR p.y;\n    return 0;\nend\n";
    let (lowered, types) = lower(source);
    let titik = callable_named(&lowered, "Titik");
    assert_eq!(
        lowered.callable_signatures[&titik].kind,
        HirCallableKind::Constructor
    );
    gate(&lowered, &types, &CallableRegistry::default(), &[]).expect("constructor validates");
    let out = compile_and_run("constructor", &[("main.ldx", source)]);
    assert_eq!(out.split_whitespace().collect::<Vec<_>>(), vec!["3", "4"]);
}

#[test]
fn builtin_inventory_and_provenance_are_exact() {
    let (lowered, types) = lower(ORDINARY);
    let builtins: BTreeSet<&str> = lowered
        .callable_signatures
        .values()
        .filter(|signature| signature.kind == HirCallableKind::Builtin)
        .map(|signature| signature.name.as_str())
        .collect();
    assert_eq!(
        builtins,
        BTreeSet::from(["logicodex_sleep", "logicodex_yield", "print"])
    );
    for builtin in &builtins {
        let callable = callable_named(&lowered, builtin);
        assert_eq!(lowered.builtin_provenance.get(*builtin), Some(&callable));
    }
    assert_eq!(lowered.builtin_provenance.len(), builtins.len());
    gate(&lowered, &types, &CallableRegistry::default(), &[]).expect("builtin print validates");
}

#[test]
fn user_extern_metadata_identity_and_declaration_order_are_preserved() {
    let (lowered, types) = lower(EXTERNS);
    let externs: Vec<_> = lowered
        .module
        .items
        .iter()
        .filter_map(|item| match &item.node {
            HirItem::ExternFunction(ext) => Some(ext.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(
        externs.iter().map(|e| e.name.as_str()).collect::<Vec<_>>(),
        vec!["first", "second"]
    );
    for ext in &externs {
        let signature = &lowered.callable_signatures[&ext.callable];
        assert_eq!(signature.kind, HirCallableKind::UserExtern);
        assert_eq!(signature.name, ext.name);
        assert_eq!(signature.params, ext.params);
    }

    let (result, _, _) = codegen_lowered(
        "externs",
        &lowered,
        &types,
        &CallableRegistry::default(),
        &["first", "second"],
    );
    let artifact = result.expect("extern program compiles");
    let ir = std::fs::read_to_string(artifact.ir_path.expect("ir path")).expect("read IR");
    let first = ir
        .find("declare i64 @first(i64)")
        .expect("frozen first decl");
    let second = ir
        .find("declare i64 @second(i64)")
        .expect("frozen second decl");
    assert!(first < second, "user externs keep source order:\n{ir}");
    assert!(ir.contains("call i64 @second(i64 %call_first)"), "{ir}");
}

#[test]
fn spawn_resolves_the_frozen_actor_identity() {
    let (lowered, types) = lower(ACTOR);
    let actor = callable_named(&lowered, "__actor_pekerja");
    assert_eq!(
        lowered.callable_signatures[&actor].kind,
        HirCallableKind::Function
    );
    let (result, _, _) =
        codegen_lowered("actor", &lowered, &types, &CallableRegistry::default(), &[]);
    let ir = std::fs::read_to_string(result.expect("actor compiles").ir_path.unwrap()).unwrap();
    assert!(
        ir.contains("call i64 @logicodex_spawn(ptr @__actor_pekerja)"),
        "{ir}"
    );
}

#[test]
fn imported_callables_compile_through_the_merged_artifact() {
    let out = compile_and_run_top_level(
        "imported",
        &[
            ("main.ldx", "import math;\nPAPAR math.add(2, 3);\n"),
            (
                "math.ldx",
                "public function add(a: I64, b: I64) -> I64 begin\n    return a + b;\nend\n",
            ),
        ],
    );
    assert_eq!(out.trim(), "5");
}

#[test]
fn recursive_calls_resolve_to_their_own_frozen_identity() {
    let source = "function fakt(n: I64) -> I64 begin\n    if n <= 1 begin\n        return 1;\n    end else begin\n        return n * fakt(n - 1);\n    end\nend\nfunction main() -> I64 begin\n    PAPAR fakt(5);\n    return 0;\nend\n";
    assert_eq!(
        compile_and_run("recursive", &[("main.ldx", source)]).trim(),
        "120"
    );
}

#[test]
fn forward_calls_resolve_before_the_callee_body_is_emitted() {
    let source = "function caller(a: I64) -> I64 begin\n    return target(a);\nend\nfunction target(a: I64) -> I64 begin\n    return a + 1;\nend\nfunction main() -> I64 begin\n    PAPAR caller(3);\n    return 0;\nend\n";
    assert_eq!(
        compile_and_run("forward", &[("main.ldx", source)]).trim(),
        "4"
    );
}

// ---- FFI provenance: only frozen user externs are foreign calls --------------

#[test]
fn non_extern_callables_never_become_ffi_calls_by_name_collision() {
    // A Function, a Constructor, and a Builtin whose names collide with
    // UnsafeRequired registry entries of a different arity, all called
    // outside any unsafe block and with no capability allowance.
    let source = "BENTUK Titik MULA\n    x: I64;\nTAMAT\nfunction puts(a: I64) -> I64 begin\n    return a + 1;\nend\nfunction main() -> I64 begin\n    BINA p: Titik = Titik(1);\n    PAPAR puts(p.x);\n    return 0;\nend\n";
    let (lowered, types) = lower(source);
    let ids = types.primitive_ids();
    let mut registry = CallableRegistry::default();
    for name in ["puts", "Titik", "print"] {
        registry.register(unsafe_extern(
            name,
            vec![ids.i64_, ids.i64_, ids.i64_],
            ids.i64_,
        ));
    }
    for (name, kind) in [
        ("puts", HirCallableKind::Function),
        ("Titik", HirCallableKind::Constructor),
        ("print", HirCallableKind::Builtin),
    ] {
        assert_eq!(
            lowered.callable_signatures[&callable_named(&lowered, name)].kind,
            kind
        );
    }

    gate(&lowered, &types, &registry, &[])
        .expect("no FFI safety, arity, or capability policy may apply to language callables");
    let (result, _, _) = codegen_lowered("collision", &lowered, &types, &registry, &[]);
    let ir = std::fs::read_to_string(result.expect("collision program compiles").ir_path.unwrap())
        .unwrap();
    assert!(ir.contains("define i64 @puts(i64"), "{ir}");
    assert!(!ir.contains("declare i64 @puts"), "{ir}");
    assert!(!ir.contains("@Titik"), "{ir}");

    // End-to-end: a user function named like a registered Raylib
    // (UnsafeRequired) function is an ordinary call in the real pipeline.
    let cli = "function InitWindow(a: I64) -> I64 begin\n    return a * 2;\nend\nfunction main() -> I64 begin\n    PAPAR InitWindow(21);\n    return 0;\nend\n";
    assert_eq!(
        compile_and_run("raylib_collision", &[("main.ldx", cli)]).trim(),
        "42"
    );
}

#[test]
fn user_extern_registry_binding_is_still_enforced() {
    // Control for the collision test: the same registry entry is applied to a
    // frozen user extern and an incompatible binding fails before LLVM.
    let (lowered, types) = lower(EXTERNS);
    let ids = types.primitive_ids();
    let mut registry = CallableRegistry::default();
    registry.register(unsafe_extern(
        "first",
        vec![ids.i64_, ids.i64_, ids.i64_],
        ids.i64_,
    ));
    assert_rejected_before_llvm(
        "extern_binding",
        &lowered,
        &types,
        &registry,
        &["first", "second"],
        &[(
            DiagnosticCode::FfiBoundaryViolation,
            "Error: FFI binding 'first' is incompatible with its frozen extern declaration",
            at(3, 16, 3, 24),
        )],
    );
}

// ---- every call and spawn location is gated ---------------------------------

#[test]
fn every_call_and_spawn_location_is_bound_and_gated() {
    let (lowered, types) = lower(WALK);
    gate(&lowered, &types, &CallableRegistry::default(), &[]).expect("walk program validates");

    let mut calls = Vec::new();
    let mut spawns = Vec::new();
    visit_callable_refs(&mut lowered.clone(), &mut |expr| match &expr.kind {
        HirExprKind::Call { callee, .. } => calls.push((*callee, expr.ty.id, expr.span)),
        HirExprKind::Spawn { actor_name, .. } => spawns.push(actor_name.clone()),
        _ => unreachable!(),
    });
    // `twice`: let 2 (nested), if condition 1, then 1, else 1, while
    // condition 1, while body 1, return 2 (binary operands) = 9; actor body:
    // print + inc = 2; top level (main wrapper): print + twice = 2.
    assert_eq!(calls.len(), 13, "walker must reach every call location");
    assert_eq!(spawns, vec!["pekerja".to_string()]);
    for (callee, stamp, _) in &calls {
        let signature = lowered
            .callable_signatures
            .get(callee)
            .unwrap_or_else(|| panic!("call {callee:?} has no frozen metadata"));
        assert_eq!(*stamp, signature.return_type);
    }
    for actor in &spawns {
        let callable = callable_named(&lowered, &format!("__actor_{actor}"));
        assert_eq!(
            lowered.callable_signatures[&callable].kind,
            HirCallableKind::Function
        );
    }

    // Corrupting the result stamp at each call location individually must be
    // rejected, proving the gate reaches every location the walker reaches.
    let stale = types.primitive_ids().bool_;
    for target in 0..calls.len() {
        let mut corrupted = lowered.clone();
        let mut index = 0;
        visit_callable_refs(&mut corrupted, &mut |expr| {
            if matches!(expr.kind, HirExprKind::Call { .. }) {
                if index == target {
                    expr.ty.id = stale;
                }
                index += 1;
            }
        });
        // Exactly one diagnostic, naming this location's callee at its span.
        let (callee, _, span) = calls[target];
        let message = format!(
            "Error: call result type stamp for `{}` does not match frozen metadata",
            lowered.callable_signatures[&callee].name
        );
        assert_integrity_rejected(
            &format!("stamp_{target}"),
            &corrupted,
            &types,
            &[(DiagnosticCode::CallableMetadataIntegrity, &message, span)],
        );
    }
}

// ---- fail-closed paths -----------------------------------------------------

#[test]
fn missing_callee_metadata_is_rejected_before_llvm_construction() {
    let (mut lowered, types) = lower(ORDINARY);
    let add = callable_named(&lowered, "add");
    lowered.callable_signatures.remove(&add);
    assert_integrity_rejected(
        "missing",
        &lowered,
        &types,
        &[
            (
                DiagnosticCode::CallableMetadataIntegrity,
                "Error: callable metadata is missing or inconsistent for declaration `add`",
                at(1, 1, 1, 42),
            ),
            (
                DiagnosticCode::CallableMetadataIntegrity,
                "Error: callable metadata is missing for reference CallableId(3)",
                at(5, 11, 5, 20),
            ),
        ],
    );
}

#[test]
fn spawn_without_frozen_actor_metadata_fails_closed() {
    let (mut lowered, types) = lower(ACTOR);
    let actor = callable_named(&lowered, "__actor_pekerja");
    lowered.callable_signatures.remove(&actor);
    assert_integrity_rejected("spawn_missing", &lowered, &types, &[(DiagnosticCode::CallableMetadataIntegrity, "Error: callable metadata is missing or inconsistent for declaration `__actor_pekerja`", Span::unknown()), (DiagnosticCode::CallableMetadataIntegrity, "Error: frozen actor callable CallableId(3) is missing for spawn `pekerja`", Span::unknown())]);
}

#[test]
fn duplicate_callable_identity_is_rejected() {
    let (mut lowered, types) = lower(ORDINARY);
    let add = callable_named(&lowered, "add");
    let mut alias = lowered.callable_signatures[&add].clone();
    alias.callable = CallableId(9_999);
    lowered.callable_signatures.insert(alias.callable, alias);
    assert_integrity_rejected(
        "duplicate",
        &lowered,
        &types,
        &[(
            DiagnosticCode::CallableMetadataIntegrity,
            "Error: corrupt callable metadata: name `add` has duplicate identities",
            Span::unknown(),
        )],
    );
}

#[test]
fn inconsistent_identity_key_is_rejected() {
    let (mut lowered, types) = lower(ORDINARY);
    let add = callable_named(&lowered, "add");
    lowered.callable_signatures.get_mut(&add).unwrap().callable = CallableId(9_999);
    assert_integrity_rejected("key_mismatch", &lowered, &types, &[(DiagnosticCode::CallableMetadataIntegrity, "Error: corrupt callable metadata: key CallableId(3) does not match identity CallableId(9999)", Span::unknown()), (DiagnosticCode::CallableMetadataIntegrity, "Error: callable metadata is missing or inconsistent for declaration `add`", at(1, 1, 1, 42))]);
}

#[test]
fn stale_signature_shape_is_rejected() {
    let (mut lowered, types) = lower(ORDINARY);
    let add = callable_named(&lowered, "add");
    lowered
        .callable_signatures
        .get_mut(&add)
        .unwrap()
        .return_type = types.primitive_ids().bool_;
    assert_integrity_rejected(
        "stale_shape",
        &lowered,
        &types,
        &[
            (
                DiagnosticCode::CallableMetadataIntegrity,
                "Error: callable metadata shape for `add` does not match its HIR declaration",
                at(1, 1, 1, 42),
            ),
            (
                DiagnosticCode::CallableMetadataIntegrity,
                "Error: call result type stamp for `add` does not match frozen metadata",
                at(5, 11, 5, 20),
            ),
        ],
    );
}

#[test]
fn inconsistent_value_use_policy_is_rejected() {
    let (mut lowered, types) = lower(ORDINARY);
    let add = callable_named(&lowered, "add");
    lowered.callable_signatures.get_mut(&add).unwrap().value_use =
        logicodex::hir::HirCallableValueUse::StatementOnly;
    assert_integrity_rejected(
        "value_use",
        &lowered,
        &types,
        &[(
            DiagnosticCode::CallableMetadataIntegrity,
            "Error: callable metadata shape for `add` does not match its HIR declaration",
            at(1, 1, 1, 42),
        )],
    );
}

#[test]
fn invalid_builtin_provenance_is_rejected() {
    let (mut lowered, types) = lower(ORDINARY);
    lowered.builtin_provenance.remove("print");
    assert_integrity_rejected(
        "provenance_missing",
        &lowered,
        &types,
        &[(
            DiagnosticCode::CallableMetadataIntegrity,
            "Error: builtin `print` has invalid provenance or canonical shape",
            Span::unknown(),
        )],
    );

    let (mut lowered, types) = lower(ORDINARY);
    let print = callable_named(&lowered, "print");
    lowered.callable_signatures.get_mut(&print).unwrap().kind = HirCallableKind::Function;
    assert_integrity_rejected(
        "provenance_kind",
        &lowered,
        &types,
        &[
            (
                DiagnosticCode::CallableMetadataIntegrity,
                "Error: callable metadata `print` has no matching HIR declaration",
                Span::unknown(),
            ),
            (
                DiagnosticCode::CallableMetadataIntegrity,
                "Error: builtin provenance `print` is invalid",
                Span::unknown(),
            ),
        ],
    );
}

#[test]
fn user_extern_identity_mismatch_is_rejected() {
    let (mut lowered, types) = lower(EXTERNS);
    for item in &mut lowered.module.items {
        if let HirItem::ExternFunction(ext) = &mut item.node {
            if ext.name == "first" {
                ext.callable = CallableId(9_999);
            }
        }
    }
    assert_rejected_before_llvm(
        "extern_identity",
        &lowered,
        &types,
        &CallableRegistry::default(),
        &["first", "second"],
        &[(
            DiagnosticCode::CallableMetadataIntegrity,
            "Error: callable metadata is missing or inconsistent for declaration `first`",
            Span::unknown(),
        )],
    );
}

#[test]
fn registry_cannot_substitute_for_missing_frozen_metadata() {
    let (mut lowered, types) = lower(ORDINARY);
    let add = callable_named(&lowered, "add");
    lowered.callable_signatures.remove(&add);
    let ids = types.primitive_ids();
    let mut registry = CallableRegistry::default();
    registry.register(CallableSignature {
        safety: CallableSafety::Safe,
        ..unsafe_extern("add", vec![ids.i64_, ids.i64_], ids.i64_)
    });
    assert_rejected_before_llvm(
        "registry_fallback",
        &lowered,
        &types,
        &registry,
        &["add"],
        &[
            (
                DiagnosticCode::CallableMetadataIntegrity,
                "Error: callable metadata is missing or inconsistent for declaration `add`",
                at(1, 1, 1, 42),
            ),
            (
                DiagnosticCode::CallableMetadataIntegrity,
                "Error: callable metadata is missing for reference CallableId(3)",
                at(5, 11, 5, 20),
            ),
        ],
    );
}

#[test]
fn empty_metadata_map_is_rejected_at_llvm_entry() {
    let (mut lowered, types) = lower(ORDINARY);
    lowered.callable_signatures.clear();
    lowered.builtin_provenance.clear();
    assert_integrity_rejected(
        "empty",
        &lowered,
        &types,
        &[
            (
                DiagnosticCode::CallableMetadataIntegrity,
                "Error: callable metadata is missing or inconsistent for declaration `add`",
                at(1, 1, 1, 42),
            ),
            (
                DiagnosticCode::CallableMetadataIntegrity,
                "Error: callable metadata is missing or inconsistent for declaration `main`",
                at(4, 1, 4, 29),
            ),
            (
                DiagnosticCode::CallableMetadataIntegrity,
                "Error: callable metadata is missing for reference CallableId(0)",
                Span::unknown(),
            ),
            (
                DiagnosticCode::CallableMetadataIntegrity,
                "Error: callable metadata is missing for reference CallableId(3)",
                at(5, 11, 5, 20),
            ),
        ],
    );
}

#[test]
fn plain_hir_codegen_entry_point_fails_closed() {
    let (lowered, types) = lower(ORDINARY);
    let object = scratch("plain").join("plain.o");
    let error = codegen::compile_v130(
        &lowered.module,
        &object,
        &options("plain"),
        CallableRegistry::default(),
        types.clone(),
        Default::default(),
    )
    .err()
    .expect("plain-HIR codegen must be rejected")
    .to_string();
    assert_eq!(error, PLAIN_HIR_CODEGEN_RETIRED);
    assert!(error.contains("compile_v130_lowered"));
    assert!(!object.exists() && !object.with_extension("ll").exists());

    // Plain-HIR semantic validation has no frozen metadata: calls fail closed.
    let diagnostics = semantic_gate::validate_module(&lowered.module, types)
        .expect_err("plain-HIR call must fail closed");
    assert_exact(
        &diagnostics,
        &[
            (
                DiagnosticCode::CallableMetadataIntegrity,
                "Error: callable metadata is missing for reference CallableId(3)",
                at(5, 11, 5, 20),
            ),
            (
                DiagnosticCode::CallableMetadataIntegrity,
                "Error: callable metadata is missing for reference CallableId(0)",
                Span::unknown(),
            ),
        ],
    );
}

fn i64_literal(types: &TypeRegistry, value: i64) -> HirExpr {
    HirExpr {
        kind: HirExprKind::Literal(LiteralAst::Integer(value)),
        ty: TypeRef {
            id: types.primitive_ids().i64_,
        },
        span: Span::unknown(),
    }
}

fn bool_literal(types: &TypeRegistry) -> HirExpr {
    HirExpr {
        kind: HirExprKind::Literal(LiteralAst::Boolean(true)),
        ty: TypeRef {
            id: types.primitive_ids().bool_,
        },
        span: Span::unknown(),
    }
}

fn function_mut<'a>(
    lowered: &'a mut LoweredModule,
    name: &str,
) -> &'a mut logicodex::hir::HirFunction {
    lowered
        .module
        .items
        .iter_mut()
        .find_map(|item| match &mut item.node {
            HirItem::Function(function) if function.name == name => Some(function),
            _ => None,
        })
        .unwrap_or_else(|| panic!("HIR function `{name}`"))
}

fn spawn_callables(lowered: &mut LoweredModule) -> Vec<CallableId> {
    let mut found = Vec::new();
    visit_callable_refs(lowered, &mut |expr| {
        if let HirExprKind::Spawn { actor_callable, .. } = &expr.kind {
            found.push(*actor_callable);
        }
    });
    found
}

fn set_spawn_callable(lowered: &mut LoweredModule, callable: CallableId) {
    visit_callable_refs(lowered, &mut |expr| {
        if let HirExprKind::Spawn { actor_callable, .. } = &mut expr.kind {
            *actor_callable = callable;
        }
    });
}

// ---- exact carried identity ------------------------------------------------

#[test]
fn function_declarations_carry_their_exact_frozen_callable_id() {
    let (lowered, _) = lower(ORDINARY);
    let mut seen = 0;
    for item in &lowered.module.items {
        if let HirItem::Function(function) = &item.node {
            let signature = &lowered.callable_signatures[&function.callable];
            assert_eq!(signature.callable, function.callable);
            assert_eq!(signature.name, function.name);
            assert_eq!(signature.kind, HirCallableKind::Function);
            seen += 1;
        }
    }
    assert_eq!(seen, 2, "add and main");
}

#[test]
fn function_with_mismatched_carried_id_is_rejected_before_llvm() {
    // Point `add` at another real Function id (`main`): name-only equivalence
    // must not rescue it.
    let (mut lowered, types) = lower(ORDINARY);
    let main = callable_named(&lowered, "main");
    function_mut(&mut lowered, "add").callable = main;
    assert_integrity_rejected(
        "fn_id_mismatch",
        &lowered,
        &types,
        &[(
            DiagnosticCode::CallableMetadataIntegrity,
            "Error: callable metadata is missing or inconsistent for declaration `add`",
            at(1, 1, 1, 42),
        )],
    );

    // A carried id with no frozen entry at all.
    let (mut lowered, types) = lower(ORDINARY);
    function_mut(&mut lowered, "add").callable = CallableId(9_999);
    assert_integrity_rejected(
        "fn_id_missing",
        &lowered,
        &types,
        &[(
            DiagnosticCode::CallableMetadataIntegrity,
            "Error: callable metadata is missing or inconsistent for declaration `add`",
            at(1, 1, 1, 42),
        )],
    );
}

#[test]
fn spawn_carries_the_exact_frozen_actor_callable_id() {
    let (mut lowered, types) = lower(ACTOR);
    let actor = callable_named(&lowered, "__actor_pekerja");
    assert_eq!(spawn_callables(&mut lowered), vec![actor]);
    assert_eq!(
        function_mut(&mut lowered, "__actor_pekerja").callable,
        actor
    );
    gate(&lowered, &types, &CallableRegistry::default(), &[]).expect("actor validates");
}

#[test]
fn spawn_with_mismatched_or_missing_actor_id_is_rejected_before_llvm() {
    // Another real Function (the main wrapper) is not the actor.
    let (mut lowered, types) = lower(ACTOR);
    let actor = callable_named(&lowered, "__actor_pekerja");
    let main = callable_named(&lowered, "main");
    set_spawn_callable(&mut lowered, main);
    assert_integrity_rejected(
        "spawn_id_function",
        &lowered,
        &types,
        &[(
            DiagnosticCode::CallableMetadataIntegrity,
            "Error: spawn `pekerja` does not match its frozen actor callable CallableId(4)",
            Span::unknown(),
        )],
    );

    // A builtin id is not the actor either.
    let (mut lowered, types) = lower(ACTOR);
    let print = callable_named(&lowered, "print");
    set_spawn_callable(&mut lowered, print);
    assert_integrity_rejected(
        "spawn_id_builtin",
        &lowered,
        &types,
        &[(
            DiagnosticCode::CallableMetadataIntegrity,
            "Error: spawn `pekerja` does not match its frozen actor callable CallableId(0)",
            Span::unknown(),
        )],
    );

    // A carried id with no frozen entry; the real actor entry is untouched,
    // so name-only resolution would wrongly succeed.
    let (mut lowered, types) = lower(ACTOR);
    set_spawn_callable(&mut lowered, CallableId(9_999));
    assert!(lowered.callable_signatures.contains_key(&actor));
    assert_integrity_rejected(
        "spawn_id_missing",
        &lowered,
        &types,
        &[(
            DiagnosticCode::CallableMetadataIntegrity,
            "Error: frozen actor callable CallableId(9999) is missing for spawn `pekerja`",
            Span::unknown(),
        )],
    );
}

// ---- variadic user externs ---------------------------------------------------

/// EXTERNS with `first` declared variadic (fixed prefix: one I64) in both its
/// HIR declaration and its frozen signature, and the call `first(...)` given
/// `args`.
fn variadic_first(args: impl Fn(&TypeRegistry) -> Vec<HirExpr>) -> (LoweredModule, TypeRegistry) {
    let (mut lowered, types) = lower(EXTERNS);
    let first = callable_named(&lowered, "first");
    lowered
        .callable_signatures
        .get_mut(&first)
        .unwrap()
        .is_variadic = true;
    for item in &mut lowered.module.items {
        if let HirItem::ExternFunction(ext) = &mut item.node {
            if ext.callable == first {
                ext.is_variadic = true;
            }
        }
    }
    let replacement = args(&types);
    visit_callable_refs(&mut lowered, &mut |expr| {
        if let HirExprKind::Call { callee, args } = &mut expr.kind {
            if *callee == first {
                *args = replacement.clone();
            }
        }
    });
    (lowered, types)
}

#[test]
fn variadic_user_extern_accepts_fixed_prefix_and_trailing_args() {
    for (label, args) in [
        ("exact_prefix", 1usize),
        ("one_trailing", 2),
        ("many_trailing", 4),
    ] {
        let (lowered, types) =
            variadic_first(|types| (0..args).map(|i| i64_literal(types, i as i64)).collect());
        gate(
            &lowered,
            &types,
            &CallableRegistry::default(),
            &["first", "second"],
        )
        .unwrap_or_else(|d| panic!("{label}: variadic extern call must validate: {d:?}"));
    }
}

#[test]
fn variadic_user_extern_requires_the_fixed_prefix() {
    let (lowered, types) = variadic_first(|_| Vec::new());
    let diagnostics = assert_codegen_fails_at_gate(
        "variadic_short",
        &lowered,
        &types,
        &CallableRegistry::default(),
        &["first", "second"],
    );
    assert_exact(
        &diagnostics,
        &[
            (
                DiagnosticCode::TypeMismatch,
                "Error: Function `first` argument count mismatch: expected at least 1, got 0",
                at(3, 16, 3, 24),
            ),
            (
                DiagnosticCode::FfiBoundaryViolation,
                "Error: Variadic function 'first' expects at least 1 argument(s)",
                at(3, 16, 3, 24),
            ),
        ],
    );
}

#[test]
fn variadic_user_extern_type_checks_every_fixed_argument() {
    let (lowered, types) = variadic_first(|types| vec![bool_literal(types), i64_literal(types, 2)]);
    let diagnostics = assert_codegen_fails_at_gate(
        "variadic_fixed_type",
        &lowered,
        &types,
        &CallableRegistry::default(),
        &["first", "second"],
    );
    assert_exact(&diagnostics, &[(DiagnosticCode::TypeMismatch, "Error: Function `first` argument 1 type mismatch: expected Primitive(I64), got Primitive(Bool)", Span::unknown()), (DiagnosticCode::FfiBoundaryViolation, "Error: Argument 1 for function 'first' has an incompatible type (expected: Primitive(I64), got: Primitive(Bool))", Span::unknown())]);
}

// ---- frozen UserExtern policy is authoritative over the registry -------------

#[test]
fn registry_binding_cannot_relax_frozen_extern_safety_or_capability() {
    // A hostile, shape-compatible registry binding marked Safe. The frozen
    // extern still requires unsafe, and still needs a capability allowance.
    let outside_unsafe = "fn main() -> I64 begin\n    first(1);\n    return 0;\nend\nextern \"C\" fn first(value: I64) -> I64;\n";
    let (lowered, types) = lower(outside_unsafe);
    let ids = types.primitive_ids();
    let mut registry = CallableRegistry::default();
    registry.register(CallableSignature {
        safety: CallableSafety::Safe,
        ..unsafe_extern("first", vec![ids.i64_], ids.i64_)
    });
    assert_rejected_before_llvm(
        "registry_safe_binding",
        &lowered,
        &types,
        &registry,
        &["first"],
        &[(
            DiagnosticCode::FfiBoundaryViolation,
            "Error: External function call 'first' requires an unsafe block",
            at(2, 5, 2, 13),
        )],
    );

    // Inside unsafe but without an allowance: the registry entry grants none.
    let (lowered, types) = lower(EXTERNS);
    let mut registry = CallableRegistry::default();
    for name in ["first", "second"] {
        registry.register(CallableSignature {
            safety: CallableSafety::Safe,
            ..unsafe_extern(name, vec![ids.i64_], ids.i64_)
        });
    }
    assert_rejected_before_llvm(
        "registry_no_capability",
        &lowered,
        &types,
        &registry,
        &["second"],
        &[(DiagnosticCode::FfiBoundaryViolation, "Error: extern call 'first' denied by FFI capability policy. Declare it in ffi.allow before use.", at(3, 16, 3, 24))],
    );

    // With a compatible binding and a frozen-policy-satisfying call it passes.
    gate(&lowered, &types, &registry, &["first", "second"])
        .expect("compatible binding with satisfied frozen policy validates");
}

#[test]
fn incompatible_registry_variadic_or_return_binding_is_rejected() {
    let (lowered, types) = lower(EXTERNS);
    let ids = types.primitive_ids();
    for (label, binding) in [
        (
            "variadic",
            CallableSignature {
                is_variadic: true,
                ..unsafe_extern("first", vec![ids.i64_], ids.i64_)
            },
        ),
        ("return", unsafe_extern("first", vec![ids.i64_], ids.bool_)),
    ] {
        let mut registry = CallableRegistry::default();
        registry.register(binding);
        assert_rejected_before_llvm(
            &format!("registry_incompatible_{label}"),
            &lowered,
            &types,
            &registry,
            &["first", "second"],
            &[(
                DiagnosticCode::FfiBoundaryViolation,
                "Error: FFI binding 'first' is incompatible with its frozen extern declaration",
                at(3, 16, 3, 24),
            )],
        );
    }
}

// ---- SemanticContext reuse ---------------------------------------------------

#[test]
fn semantic_context_is_reset_between_runs() {
    let (good, types) = lower(EXTERNS);
    let mut bad = good.clone();
    let first = callable_named(&bad, "first");
    bad.callable_signatures.remove(&first);

    let mut ctx = SemanticContext {
        types: types.clone(),
        symbols: SymbolTable::default(),
        callables: CallableRegistry::default(),
        diagnostics: Vec::new(),
        loop_depth: 0,
        safety_context: SafetyContext::Safe,
        current_return_type: None,
        policy: policy(&["first", "second"]),
        extern_symbols: HashSet::new(),
    };
    ctx.check_lowered_module(&bad)
        .expect_err("corrupt artifact fails");
    ctx.check_lowered_module(&good)
        .expect("a reused context must not carry stale diagnostics");

    // Stale per-run state (as left by an aborted run) must not leak in: a
    // leftover Unsafe context would otherwise license an extern call outside
    // any unsafe block, and stale loop depth would license a bare break.
    let outside_unsafe = "fn main() -> I64 begin\n    first(1);\n    return 0;\nend\nextern \"C\" fn first(value: I64) -> I64;\n";
    let (outside, outside_types) = lower(outside_unsafe);
    ctx.types = outside_types;
    ctx.safety_context = SafetyContext::Unsafe;
    ctx.loop_depth = 3;
    ctx.current_return_type = Some(types.primitive_ids().unit);
    ctx.diagnostics.push(Diagnostic {
        code: DiagnosticCode::TypeMismatch,
        severity: logicodex::span::Severity::Error,
        message_ms: "stale".to_string(),
        message_en: "stale".to_string(),
        primary_span: Span::unknown(),
        notes: Vec::new(),
    });
    let diagnostics = ctx
        .check_lowered_module(&outside)
        .expect_err("extern call outside unsafe must fail after reset");
    assert!(diagnostics.iter().all(|d| d.message_en != "stale"));
    assert_exact(
        &diagnostics,
        &[(
            DiagnosticCode::FfiBoundaryViolation,
            "Error: External function call 'first' requires an unsafe block",
            at(2, 5, 2, 13),
        )],
    );

    ctx.safety_context = SafetyContext::Unsafe;
    ctx.loop_depth = 3;
    let bare_break = logicodex::hir::HirModule {
        items: vec![logicodex::span::Spanned {
            node: HirItem::Function(logicodex::hir::HirFunction {
                callable: CallableId(0),
                name: "f".to_string(),
                symbol: logicodex::hir::SymbolId(0),
                params: Vec::new(),
                return_type: TypeRef {
                    id: types.primitive_ids().unit,
                },
                body: HirBlock {
                    statements: vec![logicodex::span::Spanned {
                        node: HirStmt::Break { target_depth: 0 },
                        span: Span::unknown(),
                    }],
                },
                safety: SafetyContext::Safe,
                declaration_param_enums: Vec::new(),
                declaration_return_enum: None,
            }),
            span: Span::unknown(),
        }],
    };
    let diagnostics = ctx.check_module(&bare_break).expect_err("bare break fails");
    assert_exact(
        &diagnostics,
        &[(
            DiagnosticCode::UnsafeBoundaryViolation,
            "Error: Loop control used outside a loop",
            Span::unknown(),
        )],
    );
}

// ---- semantic checks reach nested calls in every expression form ------------

/// Wrap an arity-wrong call `add(1)` in every expression-bearing field of every
/// HIR expression form (the same 21 cases as the integrity matrices, whose
/// field coverage is proven exhaustive by
/// `operand_matrices_cover_every_expression_bearing_field_of_every_variant`)
/// and require the semantic gate's call check to reach it. Some forms also
/// carry their own, form-specific diagnostics; each vector is exact.
#[test]
fn semantic_call_checks_reach_every_child_expression_form() {
    let (base, types) = lower(OPERAND_BASE);
    let add = callable_named(&base, "add");
    let i64_ = types.primitive_ids().i64_;
    let bad_call = || HirExpr {
        kind: HirExprKind::Call {
            callee: add,
            args: vec![i64_literal(&types, 1)],
        },
        ty: TypeRef { id: i64_ },
        span: Span::unknown(),
    };
    for (label, lowered) in every_operand_case(bad_call, &base, &types, add) {
        let diagnostics =
            gate(&lowered, &types, &CallableRegistry::default(), &[]).expect_err(label);
        let arity = (
            DiagnosticCode::TypeMismatch,
            "Error: Function `add` argument count mismatch: expected 2, got 1",
            Span::unknown(),
        );
        let expected: Vec<Expected> = match field_of(label) {
            // The form's own check also rejects an I64 base, after the call
            // check has reached the nested call.
            "Index.base" | "Index.index" => vec![
                arity,
                (
                    DiagnosticCode::TypeMismatch,
                    "Error: Index base must be a local fixed array, got Primitive(I64)",
                    Span::unknown(),
                ),
            ],
            "Field.base" => vec![
                arity,
                (
                    DiagnosticCode::TypeMismatch,
                    "Error: Field access base must be a struct, got Primitive(I64)",
                    Span::unknown(),
                ),
            ],
            _ => vec![arity],
        };
        assert_exact(&diagnostics, &expected);
    }
}

// ---- integrity reaches every expression-bearing operand ----------------------

/// Every HIR expression form with an expression-bearing field, each holding a
/// call `inner` in that field. `Join` and `ChannelTryRecv` carry no expression
/// operand (only a name) and are listed separately as operand-free forms.
fn forms_with_operand(
    inner: impl Fn() -> HirExpr,
    types: &TypeRegistry,
    outer_call: CallableId,
) -> Vec<(&'static str, HirExpr)> {
    let i64_ = types.primitive_ids().i64_;
    let wrap = |kind: HirExprKind| HirExpr {
        kind,
        ty: TypeRef { id: i64_ },
        span: Span::unknown(),
    };
    let lit = || i64_literal(types, 0);
    vec![
        (
            "ResultOk.value",
            wrap(HirExprKind::ResultOk {
                value: Box::new(inner()),
            }),
        ),
        (
            "ResultErr.value",
            wrap(HirExprKind::ResultErr {
                value: Box::new(inner()),
            }),
        ),
        (
            "OptionSome.value",
            wrap(HirExprKind::OptionSome {
                value: Box::new(inner()),
            }),
        ),
        (
            "ChannelCreate.capacity",
            wrap(HirExprKind::ChannelCreate {
                capacity: Box::new(inner()),
            }),
        ),
        (
            "ChannelRecv.channel",
            wrap(HirExprKind::ChannelRecv {
                channel: Box::new(inner()),
            }),
        ),
        (
            "ChannelSend.channel",
            wrap(HirExprKind::ChannelSend {
                channel: Box::new(inner()),
                value: Box::new(lit()),
            }),
        ),
        (
            "ChannelSend.value",
            wrap(HirExprKind::ChannelSend {
                channel: Box::new(lit()),
                value: Box::new(inner()),
            }),
        ),
        (
            "ChannelTrySend.value",
            wrap(HirExprKind::ChannelTrySend {
                channel_name: "c".to_string(),
                value: Box::new(inner()),
            }),
        ),
        (
            "Sleep.duration_ms",
            wrap(HirExprKind::Sleep {
                duration_ms: Box::new(inner()),
            }),
        ),
        (
            "ChannelTimeoutRecv.timeout_ms",
            wrap(HirExprKind::ChannelTimeoutRecv {
                channel_name: "c".to_string(),
                timeout_ms: Box::new(inner()),
            }),
        ),
        (
            "Unary.expr",
            wrap(HirExprKind::Unary {
                op: logicodex::hir::UnaryOpAst::Negate,
                expr: Box::new(inner()),
            }),
        ),
        (
            "Cast.expr",
            wrap(HirExprKind::Cast {
                expr: Box::new(inner()),
                target: TypeRef { id: i64_ },
            }),
        ),
        (
            "ArrayLiteral.elements",
            wrap(HirExprKind::ArrayLiteral {
                elements: vec![lit(), inner()],
            }),
        ),
        (
            "Index.index",
            wrap(HirExprKind::Index {
                base: Box::new(lit()),
                index: Box::new(inner()),
            }),
        ),
        (
            "Index.base",
            wrap(HirExprKind::Index {
                base: Box::new(inner()),
                index: Box::new(lit()),
            }),
        ),
        (
            "Binary.left",
            wrap(HirExprKind::Binary {
                left: Box::new(inner()),
                op: logicodex::hir::BinaryOpAst::Add,
                right: Box::new(lit()),
            }),
        ),
        (
            "Binary.right",
            wrap(HirExprKind::Binary {
                left: Box::new(lit()),
                op: logicodex::hir::BinaryOpAst::Add,
                right: Box::new(inner()),
            }),
        ),
        (
            "Field.base",
            wrap(HirExprKind::Field {
                base: Box::new(inner()),
                field_index: 0,
                field_name: "x".to_string(),
            }),
        ),
        (
            "Call.args",
            wrap(HirExprKind::Call {
                callee: outer_call,
                args: vec![inner(), lit()],
            }),
        ),
    ]
}

/// The operand-free forms, destructured exhaustively (no `..`): adding an
/// expression-bearing field to either fails to compile here.
fn operand_free_forms(types: &TypeRegistry) -> Vec<HirExpr> {
    let unit = types.primitive_ids().unit;
    let forms = vec![
        HirExprKind::Join {
            actor_name: "pekerja".to_string(),
        },
        HirExprKind::ChannelTryRecv {
            channel_name: "c".to_string(),
        },
    ];
    for kind in &forms {
        match kind {
            HirExprKind::Join { actor_name: _ }
            | HirExprKind::ChannelTryRecv { channel_name: _ } => {}
            _ => unreachable!(),
        }
    }
    forms
        .into_iter()
        .map(|kind| HirExpr {
            kind,
            ty: TypeRef { id: unit },
            span: Span::unknown(),
        })
        .collect()
}

fn with_statement(base: &LoweredModule, expr: HirExpr) -> LoweredModule {
    let mut lowered = base.clone();
    function_mut(&mut lowered, "main").body.statements.insert(
        0,
        logicodex::span::Spanned {
            node: HirStmt::Expr(expr),
            span: Span::unknown(),
        },
    );
    lowered
}

/// `Spawn.args` at each position: the ACTOR artifact with its spawn given two
/// arguments (`inner` at `position`, an I64 literal elsewhere) and the frozen
/// actor signature/declaration widened to exactly those argument stamps, so
/// the only defect is whatever `inner` carries.
fn spawn_forms(
    inner: impl Fn() -> HirExpr,
    base: &LoweredModule,
    types: &TypeRegistry,
) -> Vec<(&'static str, LoweredModule)> {
    let actor = callable_named(&base, "__actor_pekerja");
    [("Spawn.args[0]", 0usize), ("Spawn.args[1]", 1usize)]
        .into_iter()
        .map(|(label, position)| {
            let mut lowered = base.clone();
            let mut args = vec![i64_literal(types, 0), i64_literal(types, 0)];
            args[position] = inner();
            let param_types: Vec<TypeId> = args.iter().map(|arg| arg.ty.id).collect();
            visit_callable_refs(&mut lowered, &mut |expr| {
                if let HirExprKind::Spawn {
                    args: spawn_args, ..
                } = &mut expr.kind
                {
                    *spawn_args = args.clone();
                }
            });
            let signature = lowered.callable_signatures.get_mut(&actor).unwrap();
            signature.params = param_types.clone();
            signature.param_enums = vec![None; param_types.len()];
            let actor_fn = function_mut(&mut lowered, "__actor_pekerja");
            actor_fn.declaration_param_enums = vec![None; param_types.len()];
            actor_fn.params = param_types
                .iter()
                .enumerate()
                .map(|(i, ty)| logicodex::hir::HirParam {
                    local: logicodex::hir::LocalId(9_000 + i as u32),
                    ty: TypeRef { id: *ty },
                })
                .collect();
            (label, lowered)
        })
        .collect()
}

/// Every `HirExprKind` variant, numbered. Exhaustive with no wildcard: a new
/// variant fails to compile here until it is counted and covered.
const HIR_EXPR_VARIANTS: usize = 24;
fn variant_index(kind: &HirExprKind) -> usize {
    match kind {
        HirExprKind::Literal(_) => 0,
        HirExprKind::Local(_) => 1,
        HirExprKind::Global(_) => 2,
        HirExprKind::Binary { .. } => 3,
        HirExprKind::Unary { .. } => 4,
        HirExprKind::Call { .. } => 5,
        HirExprKind::Field { .. } => 6,
        HirExprKind::Index { .. } => 7,
        HirExprKind::ArrayLiteral { .. } => 8,
        HirExprKind::Cast { .. } => 9,
        HirExprKind::ResultOk { .. } => 10,
        HirExprKind::ResultErr { .. } => 11,
        HirExprKind::OptionSome { .. } => 12,
        HirExprKind::OptionNone => 13,
        HirExprKind::Spawn { .. } => 14,
        HirExprKind::Join { .. } => 15,
        HirExprKind::ChannelCreate { .. } => 16,
        HirExprKind::ChannelSend { .. } => 17,
        HirExprKind::ChannelRecv { .. } => 18,
        HirExprKind::ChannelTrySend { .. } => 19,
        HirExprKind::ChannelTryRecv { .. } => 20,
        HirExprKind::Yield => 21,
        HirExprKind::Sleep { .. } => 22,
        HirExprKind::ChannelTimeoutRecv { .. } => 23,
    }
}

/// The expression-bearing fields of each variant. Every field is bound by
/// name with no `..`: adding or renaming any field fails to compile here.
fn expression_operands(kind: &HirExprKind) -> Vec<&'static str> {
    match kind {
        HirExprKind::Literal(_) | HirExprKind::Local(_) | HirExprKind::Global(_) => vec![],
        HirExprKind::Binary {
            left: _,
            op: _,
            right: _,
        } => vec!["Binary.left", "Binary.right"],
        HirExprKind::Unary { op: _, expr: _ } => vec!["Unary.expr"],
        HirExprKind::Call { callee: _, args: _ } => vec!["Call.args"],
        HirExprKind::Field {
            base: _,
            field_index: _,
            field_name: _,
        } => vec!["Field.base"],
        HirExprKind::Index { base: _, index: _ } => vec!["Index.base", "Index.index"],
        HirExprKind::ArrayLiteral { elements: _ } => vec!["ArrayLiteral.elements"],
        HirExprKind::Cast { expr: _, target: _ } => vec!["Cast.expr"],
        HirExprKind::ResultOk { value: _ } => vec!["ResultOk.value"],
        HirExprKind::ResultErr { value: _ } => vec!["ResultErr.value"],
        HirExprKind::OptionSome { value: _ } => vec!["OptionSome.value"],
        HirExprKind::OptionNone => vec![],
        HirExprKind::Spawn {
            actor_name: _,
            actor_callable: _,
            args: _,
        } => vec!["Spawn.args"],
        HirExprKind::Join { actor_name: _ } => vec![],
        HirExprKind::ChannelCreate { capacity: _ } => vec!["ChannelCreate.capacity"],
        HirExprKind::ChannelSend {
            channel: _,
            value: _,
        } => {
            vec!["ChannelSend.channel", "ChannelSend.value"]
        }
        HirExprKind::ChannelRecv { channel: _ } => vec!["ChannelRecv.channel"],
        HirExprKind::ChannelTrySend {
            channel_name: _,
            value: _,
        } => vec!["ChannelTrySend.value"],
        HirExprKind::ChannelTryRecv { channel_name: _ } => vec![],
        HirExprKind::Yield => vec![],
        HirExprKind::Sleep { duration_ms: _ } => vec!["Sleep.duration_ms"],
        HirExprKind::ChannelTimeoutRecv {
            channel_name: _,
            timeout_ms: _,
        } => {
            vec!["ChannelTimeoutRecv.timeout_ms"]
        }
    }
}

/// Matrix label → field label (`Spawn.args[1]` → `Spawn.args`).
fn field_of(label: &str) -> &str {
    label.split('[').next().unwrap()
}

#[test]
fn operand_matrices_cover_every_expression_bearing_field_of_every_variant() {
    let (base, types) = lower(OPERAND_BASE);
    let add = callable_named(&base, "add");
    let unit = types.primitive_ids().unit;
    let probe = || i64_literal(&types, 0);
    let leaf = |kind| HirExpr {
        kind,
        ty: TypeRef { id: unit },
        span: Span::unknown(),
    };

    // Samples of every variant: the operand matrix, the spawn forms, the
    // operand-free forms, and the leaves.
    let mut samples: Vec<HirExpr> = forms_with_operand(probe, &types, add)
        .into_iter()
        .map(|(_, expr)| expr)
        .collect();
    let mut spawn_sample = None;
    visit_callable_refs(&mut lower(ACTOR).0, &mut |expr| {
        if matches!(expr.kind, HirExprKind::Spawn { .. }) {
            spawn_sample = Some(expr.clone());
        }
    });
    samples.push(spawn_sample.expect("ACTOR has a spawn"));
    samples.extend(operand_free_forms(&types));
    samples.extend([
        leaf(HirExprKind::Literal(LiteralAst::Integer(0))),
        leaf(HirExprKind::Local(logicodex::hir::LocalId(0))),
        leaf(HirExprKind::Global(logicodex::hir::SymbolId(0))),
        leaf(HirExprKind::OptionNone),
        leaf(HirExprKind::Yield),
    ]);
    let variants: BTreeSet<usize> = samples.iter().map(|e| variant_index(&e.kind)).collect();
    assert_eq!(
        variants,
        (0..HIR_EXPR_VARIANTS).collect(),
        "every variant sampled"
    );

    let required: BTreeSet<&str> = samples
        .iter()
        .flat_map(|e| expression_operands(&e.kind))
        .collect();
    let covered: BTreeSet<String> = forms_with_operand(probe, &types, add)
        .into_iter()
        .map(|(label, _)| field_of(label).to_string())
        .chain(
            spawn_forms(probe, &base, &types)
                .into_iter()
                .map(|(label, _)| field_of(label).to_string()),
        )
        .collect();
    let covered: BTreeSet<&str> = covered.iter().map(String::as_str).collect();
    assert_eq!(
        covered, required,
        "matrix covers exactly every expression-bearing field"
    );
    assert_eq!(required.len(), 20);
}

/// Every operand position used by the integrity and semantic matrices.
fn every_operand_case(
    inner: impl Fn() -> HirExpr + Copy,
    base: &LoweredModule,
    types: &TypeRegistry,
    outer_call: CallableId,
) -> Vec<(&'static str, LoweredModule)> {
    let mut cases: Vec<(&'static str, LoweredModule)> =
        forms_with_operand(inner, types, outer_call)
            .into_iter()
            .map(|(label, expr)| (label, with_statement(base, expr)))
            .collect();
    cases.extend(spawn_forms(inner, base, types));
    assert_eq!(
        cases.len(),
        21,
        "19 operand fields + 2 spawn argument positions"
    );
    cases
}

#[test]
fn missing_callee_inside_every_operand_is_rejected_before_llvm() {
    let (base, types) = lower(OPERAND_BASE);
    let i64_ = types.primitive_ids().i64_;
    let missing = || HirExpr {
        kind: HirExprKind::Call {
            callee: CallableId(9_999),
            args: Vec::new(),
        },
        ty: TypeRef { id: i64_ },
        span: Span::unknown(),
    };
    let add = callable_named(&base, "add");
    for (label, lowered) in every_operand_case(missing, &base, &types, add) {
        assert_rejected_before_llvm(
            &format!("missing_in_{}", label.replace(['.', '[', ']'], "_")),
            &lowered,
            &types,
            &CallableRegistry::default(),
            &[],
            &[(
                DiagnosticCode::CallableMetadataIntegrity,
                "Error: callable metadata is missing for reference CallableId(9999)",
                Span::unknown(),
            )],
        );
    }
}

#[test]
fn stale_call_stamp_inside_every_operand_is_rejected_before_llvm() {
    let (base, types) = lower(OPERAND_BASE);
    let add = callable_named(&base, "add");
    let stale = types.primitive_ids().bool_;
    let stale_call = || HirExpr {
        kind: HirExprKind::Call {
            callee: add,
            args: vec![i64_literal(&types, 1), i64_literal(&types, 2)],
        },
        ty: TypeRef { id: stale },
        span: Span::unknown(),
    };
    for (label, lowered) in every_operand_case(stale_call, &base, &types, add) {
        assert_rejected_before_llvm(
            &format!("stale_in_{}", label.replace(['.', '[', ']'], "_")),
            &lowered,
            &types,
            &CallableRegistry::default(),
            &[],
            &[(
                DiagnosticCode::CallableMetadataIntegrity,
                "Error: call result type stamp for `add` does not match frozen metadata",
                Span::unknown(),
            )],
        );
    }
}

#[test]
fn operand_free_forms_carry_no_callable_reference() {
    let (base, types) = lower(ORDINARY);
    for expr in operand_free_forms(&types) {
        gate(
            &with_statement(&base, expr),
            &types,
            &CallableRegistry::default(),
            &[],
        )
        .expect("Join / ChannelTryRecv contain no callable operand to reject");
    }
}

// ---- constructor return identity ----------------------------------------------

const TWO_STRUCTS: &str = "BENTUK Titik MULA\n    x: I64;\nTAMAT\nBENTUK Garis MULA\n    x: I64;\nTAMAT\nfunction main() -> I64 begin\n    BINA p: Titik = Titik(3);\n    PAPAR p.x;\n    return 0;\nend\n";

#[test]
fn constructor_return_must_be_its_own_struct_type() {
    let (lowered, types) = lower(TWO_STRUCTS);
    gate(&lowered, &types, &CallableRegistry::default(), &[])
        .expect("canonical constructors validate");

    let garis_type = {
        let garis = callable_named(&lowered, "Garis");
        lowered.callable_signatures[&garis].return_type
    };
    for (label, corrupt) in [
        ("primitive", types.primitive_ids().i64_),
        ("other_struct", garis_type),
        ("out_of_range", TypeId(9_999)),
    ] {
        let mut corrupted = lowered.clone();
        let titik = callable_named(&corrupted, "Titik");
        corrupted
            .callable_signatures
            .get_mut(&titik)
            .unwrap()
            .return_type = corrupt;
        assert_rejected_before_llvm(
            &format!("ctor_return_{label}"),
            &corrupted,
            &types,
            &CallableRegistry::default(),
            &[],
            &[
                (
                    DiagnosticCode::CallableMetadataIntegrity,
                    "Error: callable metadata shape for `Titik` does not match its HIR declaration",
                    Span::unknown(),
                ),
                (
                    DiagnosticCode::CallableMetadataIntegrity,
                    "Error: call result type stamp for `Titik` does not match frozen metadata",
                    at(8, 21, 8, 29),
                ),
            ],
        );
    }

    // `Garis` is never called, so no call stamp can expose a corrupt return:
    // only the declaration-level constructor identity check rejects it.
    let titik_type = {
        let titik = callable_named(&lowered, "Titik");
        lowered.callable_signatures[&titik].return_type
    };
    for (label, corrupt) in [
        ("uncalled_primitive", types.primitive_ids().i64_),
        ("uncalled_other_struct", titik_type),
        ("uncalled_out_of_range", TypeId(9_999)),
    ] {
        let mut corrupted = lowered.clone();
        let garis = callable_named(&corrupted, "Garis");
        corrupted
            .callable_signatures
            .get_mut(&garis)
            .unwrap()
            .return_type = corrupt;
        assert_rejected_before_llvm(
            &format!("ctor_return_{label}"),
            &corrupted,
            &types,
            &CallableRegistry::default(),
            &[],
            &[(
                DiagnosticCode::CallableMetadataIntegrity,
                "Error: callable metadata shape for `Garis` does not match its HIR declaration",
                Span::unknown(),
            )],
        );
    }
}

// ---- declaration-side enum annotations vs frozen metadata ---------------------

const ENUM_DECLS: &str = "PILIHAN Status MULA\n    Aktif;\n    Henti;\nTAMAT\nPILIHAN OtherStatus MULA\n    Ya;\n    Tidak;\nTAMAT\nBENTUK Rekod MULA\n    s: Status;\n    n: I64;\nTAMAT\nfunction f(s: Status, n: I64, o: OtherStatus) -> Status begin\n    return s;\nend\nfunction main() -> I64 begin\n    return 0;\nend\n";

fn struct_mut<'a>(
    lowered: &'a mut LoweredModule,
    name: &str,
) -> &'a mut logicodex::hir::HirStructDecl {
    lowered
        .module
        .items
        .iter_mut()
        .find_map(|item| match &mut item.node {
            HirItem::Struct(decl) if decl.name == name => Some(decl),
            _ => None,
        })
        .unwrap_or_else(|| panic!("HIR struct `{name}`"))
}

fn some(name: &str) -> Option<String> {
    Some(name.to_string())
}

#[test]
fn declaration_enum_carriers_come_from_the_ast_not_frozen_metadata() {
    let (lowered, types) = lower(ENUM_DECLS);
    gate(&lowered, &types, &CallableRegistry::default(), &[])
        .expect("clean enum declarations validate");

    let mut probe = lowered.clone();
    let f = function_mut(&mut probe, "f");
    assert_eq!(
        f.declaration_param_enums,
        vec![some("Status"), None, some("OtherStatus")]
    );
    assert_eq!(f.declaration_return_enum, some("Status"));
    // Enum erasure is unchanged: every enum-annotated parameter is still I64.
    let i64_ = types.primitive_ids().i64_;
    assert!(f.params.iter().all(|param| param.ty.id == i64_));
    let rekod = struct_mut(&mut probe, "Rekod");
    assert_eq!(rekod.declaration_field_enums, vec![some("Status"), None]);

    // Mutating the frozen metadata after lowering leaves the declaration
    // carriers untouched: they are not copies of the frozen vectors.
    let before = lowered.module.clone();
    let mut mutated = lowered.clone();
    let f_id = callable_named(&mutated, "f");
    let rekod_id = callable_named(&mutated, "Rekod");
    let sig = mutated.callable_signatures.get_mut(&f_id).unwrap();
    sig.param_enums = vec![None, None, None];
    sig.return_enum = None;
    let sig = mutated.callable_signatures.get_mut(&rekod_id).unwrap();
    sig.param_enums = vec![None, None];
    sig.return_enum = some("Status");
    assert_eq!(
        mutated.module, before,
        "declaration carriers are independent of frozen metadata"
    );
}

const STALE_PARAMS: &str = "Error: callable metadata for function 'f' has parameter enum annotations inconsistent with the declaration";
const STALE_RETURN: &str = "Error: callable metadata for function 'f' has return enum annotation inconsistent with the declaration";
const STALE_FIELDS: &str = "Error: callable metadata for constructor 'Rekod' has field enum annotations inconsistent with the struct declaration";
const STALE_CTOR_RETURN: &str = "Error: callable metadata for constructor 'Rekod' has an enum return annotation inconsistent with the struct declaration";

fn assert_enum_integrity_rejected(
    label: &str,
    lowered: &LoweredModule,
    types: &TypeRegistry,
    message: &str,
) {
    assert_integrity_rejected(
        &format!("enum_{label}"),
        lowered,
        types,
        &[(
            DiagnosticCode::CallableMetadataIntegrity,
            message,
            Span::unknown(),
        )],
    );
}

#[test]
fn stale_function_parameter_enums_are_rejected_before_llvm() {
    let (base, types) = lower(ENUM_DECLS);
    // A: frozen metadata corrupted, declaration intact.
    let mut a = base.clone();
    let f = callable_named(&a, "f");
    a.callable_signatures.get_mut(&f).unwrap().param_enums =
        vec![some("OtherStatus"), None, some("OtherStatus")];
    assert_enum_integrity_rejected("fn_params_a", &a, &types, STALE_PARAMS);
    // B: declaration carrier corrupted, frozen metadata intact.
    let mut b = base.clone();
    function_mut(&mut b, "f").declaration_param_enums =
        vec![some("Status"), some("Status"), some("OtherStatus")];
    assert_enum_integrity_rejected("fn_params_b", &b, &types, STALE_PARAMS);
}

#[test]
fn stale_function_return_enum_is_rejected_before_llvm() {
    let (base, types) = lower(ENUM_DECLS);
    let mut a = base.clone();
    let f = callable_named(&a, "f");
    a.callable_signatures.get_mut(&f).unwrap().return_enum = None;
    assert_enum_integrity_rejected("fn_return_a", &a, &types, STALE_RETURN);
    let mut b = base.clone();
    function_mut(&mut b, "f").declaration_return_enum = some("OtherStatus");
    assert_enum_integrity_rejected("fn_return_b", &b, &types, STALE_RETURN);
}

#[test]
fn stale_constructor_field_enums_are_rejected_before_llvm() {
    let (base, types) = lower(ENUM_DECLS);
    let mut a = base.clone();
    let rekod = callable_named(&a, "Rekod");
    a.callable_signatures.get_mut(&rekod).unwrap().param_enums = vec![None, None];
    assert_enum_integrity_rejected("ctor_fields_a", &a, &types, STALE_FIELDS);
    let mut b = base.clone();
    struct_mut(&mut b, "Rekod").declaration_field_enums = vec![some("Status"), some("OtherStatus")];
    assert_enum_integrity_rejected("ctor_fields_b", &b, &types, STALE_FIELDS);
}

#[test]
fn enum_constructor_return_annotation_is_rejected_before_llvm() {
    // Only direction A exists: the constructor return invariant is the
    // canonical `None`, with no declaration-side carrier to corrupt.
    let (base, types) = lower(ENUM_DECLS);
    let mut a = base.clone();
    let rekod = callable_named(&a, "Rekod");
    a.callable_signatures.get_mut(&rekod).unwrap().return_enum = some("Status");
    assert_enum_integrity_rejected("ctor_return_a", &a, &types, STALE_CTOR_RETURN);
}

#[test]
fn parameter_enum_mismatch_is_reported_before_return_mismatch() {
    let (base, types) = lower(ENUM_DECLS);
    let mut both = base.clone();
    let f = callable_named(&both, "f");
    let sig = both.callable_signatures.get_mut(&f).unwrap();
    sig.param_enums = vec![None, None, None];
    sig.return_enum = None;
    assert_integrity_rejected(
        "enum_fn_both",
        &both,
        &types,
        &[
            (
                DiagnosticCode::CallableMetadataIntegrity,
                STALE_PARAMS,
                Span::unknown(),
            ),
            (
                DiagnosticCode::CallableMetadataIntegrity,
                STALE_RETURN,
                Span::unknown(),
            ),
        ],
    );
}

// ---- struct-bearing callable signatures --------------------------------------

#[test]
fn struct_parameter_return_and_forward_struct_signatures_compile() {
    // Struct declared after every use: functions taking and returning it are
    // declared (phase 2) only after all struct types are registered (phase 1).
    let source = "function main() -> I64 begin\n    PAPAR guna(2);\n    return 0;\nend\nfunction guna(a: I64) -> I64 begin\n    return jumlah(buat(a));\nend\nfunction jumlah(p: Titik) -> I64 begin\n    return p.x + p.y;\nend\nfunction buat(a: I64) -> Titik begin\n    return Titik(a, a + 1);\nend\nBENTUK Titik MULA\n    x: I64;\n    y: I64;\nTAMAT\n";
    let (lowered, types) = lower(source);
    gate(&lowered, &types, &CallableRegistry::default(), &[]).expect("struct signatures validate");
    let (result, _, _) = codegen_lowered(
        "struct_sigs",
        &lowered,
        &types,
        &CallableRegistry::default(),
        &[],
    );
    let ir = std::fs::read_to_string(result.expect("struct signatures compile").ir_path.unwrap())
        .unwrap();
    assert!(ir.contains("define i64 @jumlah("), "{ir}");
    assert!(
        ir.contains("define i64 @buat(ptr"),
        "struct return uses sret:\n{ir}"
    );
    assert_eq!(
        compile_and_run("struct_sigs", &[("main.ldx", source)]).trim(),
        "5"
    );
}

#[test]
fn struct_typed_user_extern_fails_closed_at_semantic_gate() {
    let source = "BENTUK Titik MULA\n    x: I64;\nTAMAT\nfn main() -> I64 begin\n    return 0;\nend\nextern \"C\" fn ambil(p: Titik) -> I64;\nextern \"C\" fn beri(a: I64) -> Titik;\n";
    let (lowered, types) = lower(source);
    assert_rejected_before_llvm(
        "struct_extern",
        &lowered,
        &types,
        &CallableRegistry::default(),
        &["ambil", "beri"],
        &[(DiagnosticCode::FfiBoundaryViolation, "Error: extern `ambil` uses a struct type by value, which the FFI boundary does not support", Span::unknown()), (DiagnosticCode::FfiBoundaryViolation, "Error: extern `beri` uses a struct type by value, which the FFI boundary does not support", Span::unknown())],
    );
}

/// Supplemental only: behaviour is established by the tests above. These
/// source checks guard against reintroducing the retired fallback shapes.
#[test]
fn supplemental_source_guards_for_retired_fallbacks() {
    let codegen_source = include_str!("../src/codegen.rs");
    assert!(!codegen_source.contains("c.find_by_name(n)"));
    assert!(!codegen_source.contains("self.callable_names"));
    assert!(!codegen_source.contains("predeclare_callables warning"));
    assert!(!codegen_source.contains("fn frozen_callable_id"));
    let lowered_entry = codegen_source
        .split("pub fn compile_v130_lowered(")
        .nth(1)
        .expect("lowered entry point");
    let gate_at = lowered_entry
        .find("validate_lowered_module")
        .expect("defensive gate");
    let llvm_at = lowered_entry
        .find("Context::create()")
        .expect("LLVM context");
    assert!(gate_at < llvm_at, "gate must run before LLVM construction");
}
