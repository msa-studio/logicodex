use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use logicodex::codegen::{self, CodegenOptions};
use logicodex::ffi::{CallableRegistry, CapabilityPolicy, SafetyContext};
use logicodex::hir::{
    HirBlock, HirExpr, HirExprKind, HirFunction, HirItem, HirStmt, LoweredModule, LoweringContext,
    SymbolTable,
};
use logicodex::lexer::{Lexer, Lexicon};
use logicodex::os::target::CompilationTarget;
use logicodex::parser::{CompilerPipeline, Parser};
use logicodex::semantic_gate::{self, SemanticContext};
use logicodex::span::{Diagnostic, DiagnosticCode, FileId, Span};
use logicodex::types::{CallableId, TypeRegistry};

static NEXT_ROOT: AtomicU64 = AtomicU64::new(0);

struct FixtureRoot(PathBuf);
impl Drop for FixtureRoot {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Record {
    code: String,
    severity: String,
    span: String,
    ms: String,
    en: String,
}

fn artifacts(root: &Path) -> Vec<PathBuf> {
    fn walk(path: &Path, out: &mut Vec<PathBuf>) {
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() {
                    walk(&p, out);
                } else if matches!(
                    p.extension().and_then(|x| x.to_str()),
                    Some("ll" | "bc" | "o" | "obj" | "exe")
                ) || p.file_name().and_then(|x| x.to_str()) == Some("main")
                {
                    out.push(p);
                }
            }
        }
    }
    let mut out = Vec::new();
    walk(root, &mut out);
    out
}

fn fixture(source: &str) -> (FixtureRoot, String) {
    let root = std::env::temp_dir().join(format!(
        "logicodex_cpb2b_{}_{}",
        std::process::id(),
        NEXT_ROOT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir(&root).expect("fresh fixture root");
    let file = root.join("main.ldx");
    std::fs::write(&file, source).expect("fixture source");
    (FixtureRoot(root), file.to_string_lossy().into_owned())
}

fn parse_records(text: &str) -> Vec<Record> {
    let lines: Vec<&str> = text.lines().collect();
    let mut records = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let (severity, code) = if let Some(code) = lines[i].strip_prefix("Error: code: ") {
            ("Error", code)
        } else if let Some(code) = lines[i].strip_prefix("code: ") {
            ("Error", code)
        } else {
            i += 1;
            continue;
        };
        assert!(i + 2 < lines.len(), "truncated diagnostic: {text}");
        let span = lines[i + 1]
            .strip_prefix("span: ")
            .expect("diagnostic span")
            .to_string();
        let message = lines[i + 2];
        let (ms, en) = message
            .split_once(" / Error: ")
            .expect("bilingual diagnostic");
        records.push(Record {
            code: code.to_string(),
            severity: severity.to_string(),
            span,
            ms: ms.to_string(),
            en: format!("Error: {en}"),
        });
        i += 3;
    }
    records
}

fn check(source: &str, expected: &[Record], must_fail: bool) {
    let (root_guard, file) = fixture(source);
    let root = &root_guard.0;
    let output = Command::new(env!("CARGO_BIN_EXE_logicodex"))
        .current_dir(root)
        .env(
            "LOGICODEX_STD",
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("lib"),
        )
        .arg("check")
        .arg("--dict")
        .arg(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("dict/core_map.json"))
        .arg(&file)
        .output()
        .expect("logicodex check");
    assert_eq!(
        output.status.success(),
        !must_fail,
        "unexpected exit: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(parse_records(&text), expected, "raw diagnostics:\n{text}");
    assert!(
        artifacts(root).is_empty(),
        "rejected fixture produced backend artifacts: {:?}",
        artifacts(root)
    );
}

fn mismatch(code: &str, span: &str, ms: &str, en: &str) -> Record {
    Record {
        code: code.into(),
        severity: "Error".into(),
        span: span.into(),
        ms: ms.into(),
        en: en.into(),
    }
}

// ---- lowered-artifact harness ------------------------------------------------

fn dict() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("dict/core_map.json")
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "ldx_cpb2_assignment_return_{name}_{}",
        std::process::id()
    ));
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

/// Direct semantic validation of the lowered artifact with the production
/// runtime-builtin policy and an empty FFI registry.
fn gate(lowered: &LoweredModule, types: &TypeRegistry) -> Result<(), Vec<Diagnostic>> {
    let mut ctx = SemanticContext {
        types: types.clone(),
        symbols: SymbolTable::default(),
        callables: CallableRegistry::default(),
        diagnostics: Vec::new(),
        loop_depth: 0,
        safety_context: SafetyContext::Safe,
        current_return_type: None,
        policy: CapabilityPolicy::with_runtime_builtins(),
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
) -> (anyhow::Result<codegen::CodegenArtifact>, PathBuf, PathBuf) {
    let object = scratch(name).join(format!("{name}.o"));
    assert!(
        !object.exists() && !object.with_extension("ll").exists(),
        "artifact paths must be clean before codegen"
    );
    let result = codegen::compile_v130_lowered(
        lowered,
        &object,
        &options(name),
        CallableRegistry::default(),
        types.clone(),
        CapabilityPolicy::with_runtime_builtins(),
    );
    let ir = object.with_extension("ll");
    (result, object, ir)
}

/// Directly validate `lowered`, require rejection, and require codegen to fail
/// with exactly those gate diagnostics and to write no object or IR. Because
/// `compile_v130_lowered` reports this message only from the defensive gate
/// step that precedes LLVM construction, equality proves pre-LLVM rejection.
fn assert_codegen_fails_at_gate(
    name: &str,
    lowered: &LoweredModule,
    types: &TypeRegistry,
) -> Vec<Diagnostic> {
    let diagnostics = semantic_gate::validate_lowered_module(
        lowered,
        types.clone(),
        CallableRegistry::default(),
        CapabilityPolicy::with_runtime_builtins(),
    )
    .expect_err("semantic gate must reject");

    let (result, object, ir) = codegen_lowered(name, lowered, types);
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

/// Require semantic acceptance and a successful object emission.
fn assert_accepted(name: &str, lowered: &LoweredModule, types: &TypeRegistry) {
    let result = gate(lowered, types);
    assert!(result.is_ok(), "unexpected diagnostics: {result:?}");
    let (result, object, _ir) = codegen_lowered(name, lowered, types);
    result.expect("codegen must accept");
    assert!(object.exists(), "accepted artifact must emit an object");
}

/// One expected diagnostic: exact code, Malay message, English message, span.
type Expected<'a> = (DiagnosticCode, &'a str, &'a str, Span);

fn at(start_line: u32, start_col: u32, end_line: u32, end_col: u32) -> Span {
    Span::new(FileId(0), start_line, start_col, end_line, end_col)
}

/// Assert the exact, ordered diagnostic vector.
fn assert_exact(diagnostics: &[Diagnostic], expected: &[Expected]) {
    assert!(
        !expected.is_empty(),
        "an expected diagnostic vector is required"
    );
    let actual: Vec<Expected> = diagnostics
        .iter()
        .map(|d| {
            (
                d.code,
                d.message_ms.as_str(),
                d.message_en.as_str(),
                d.primary_span,
            )
        })
        .collect();
    assert_eq!(actual, expected, "exact diagnostic vector mismatch");
}

fn function_mut<'a>(lowered: &'a mut LoweredModule, name: &str) -> &'a mut HirFunction {
    lowered
        .module
        .items
        .iter_mut()
        .find_map(|item| match &mut item.node {
            HirItem::Function(function) if function.name == name => Some(function),
            _ => None,
        })
        .unwrap_or_else(|| panic!("lowered function `{name}`"))
}

fn body_mut<'a>(lowered: &'a mut LoweredModule, name: &str) -> &'a mut HirBlock {
    &mut function_mut(lowered, name).body
}

fn callable_named(lowered: &LoweredModule, name: &str) -> CallableId {
    lowered
        .callable_signatures
        .values()
        .find(|signature| signature.name == name)
        .unwrap_or_else(|| panic!("frozen metadata for `{name}`"))
        .callable
}

fn then_branch(stmt: &mut HirStmt) -> &mut HirBlock {
    match stmt {
        HirStmt::If { then_branch, .. } => then_branch,
        other => panic!("expected if statement, got {other:?}"),
    }
}

fn loop_body(stmt: &mut HirStmt) -> &mut HirBlock {
    match stmt {
        HirStmt::While { body, .. } | HirStmt::Loop { body } => body,
        other => panic!("expected loop statement, got {other:?}"),
    }
}

fn return_value(stmt: &mut HirStmt) -> &mut HirExpr {
    match stmt {
        HirStmt::Return(Some(expr)) => expr,
        other => panic!("expected value return, got {other:?}"),
    }
}

const RETURN_MISMATCH_MS: &str =
    "Ralat: Jenis pulangan tidak sepadan: dijangka Primitive(I64), diterima Primitive(Bool)";
const RETURN_MISMATCH_EN: &str =
    "Error: Return type mismatch: expected Primitive(I64), got Primitive(Bool)";
const ASSIGN_MISMATCH_MS: &str =
    "Ralat: Jenis tugasan tidak sepadan: dijangka Primitive(I64), diterima Primitive(Bool)";
const ASSIGN_MISMATCH_EN: &str =
    "Error: Assignment type mismatch: expected Primitive(I64), got Primitive(Bool)";

#[test]
fn assignment_mismatch_preserves_exact_record() {
    check(
        "fn main() -> I64 begin\n    let x: I64 = 1;\n    x = true;\n    return 0;\nend\n",
        &[mismatch(
            "TypeMismatch",
            "file 0:3:5-3:14",
            "Ralat: Jenis tugasan tidak sepadan: dijangka Primitive(I64), diterima Primitive(Bool)",
            "Error: Assignment type mismatch: expected Primitive(I64), got Primitive(Bool)",
        )],
        true,
    );
}

#[test]
fn assignment_match_is_accepted() {
    check(
        "fn main() -> I64 begin\n    let x: I64 = 1;\n    x = 2;\n    return x;\nend\n",
        &[],
        false,
    );
}

#[test]
fn binding_call_result_mismatch_preserves_exact_record() {
    check("fn flag() -> Bool begin\n    return true;\nend\nfn main() -> I64 begin\n    let x: I64 = flag();\n    return 0;\nend\n", &[mismatch("TypeMismatch", "file 0:5:5-5:25", "Ralat: Jenis ikatan tidak sepadan: dijangka Primitive(I64), diterima Primitive(Bool)", "Error: Binding type mismatch: expected Primitive(I64), got Primitive(Bool)")], true);
}

#[test]
fn return_mismatch_preserves_exact_record() {
    check("fn bad() -> I64 begin\n    return true;\nend\nfn main() -> I64 begin\n    return 0;\nend\n", &[mismatch("TypeMismatch", "file 0:2:5-2:17", "Ralat: Jenis pulangan tidak sepadan: dijangka Primitive(I64), diterima Primitive(Bool)", "Error: Return type mismatch: expected Primitive(I64), got Primitive(Bool)")], true);
}

#[test]
fn return_match_is_accepted() {
    check("fn flag() -> Bool begin\n    return true;\nend\nfn main() -> I64 begin\n    return 0;\nend\n", &[], false);
}

#[test]
fn unit_return_value_preserves_exact_record() {
    check(
        "fn bad() -> Unit begin\n    return 1;\nend\nfn main() -> I64 begin\n    return 0;\nend\n",
        &[mismatch(
            "TypeMismatch",
            "file 0:2:5-2:14",
            "Ralat: Fungsi Unit tidak boleh memulangkan nilai jenis Primitive(I64)",
            "Error: Unit function cannot return a value of type Primitive(I64)",
        )],
        true,
    );
}

#[test]
fn call_result_used_as_value_preserves_exact_record() {
    check("fn noop() -> Unit begin\nend\nfn main() -> I64 begin\n    return noop();\nend\n", &[mismatch("TypeMismatch", "file 0:4:5-4:19", "Ralat: Panggilan `noop` memulangkan Unit dan tidak boleh digunakan sebagai nilai dalam return", "Error: Call `noop` returns Unit and cannot be used as a value in return"), mismatch("TypeMismatch", "file 0:4:5-4:19", "Ralat: Jenis pulangan tidak sepadan: dijangka Primitive(I64), diterima Primitive(Unit)", "Error: Return type mismatch: expected Primitive(I64), got Primitive(Unit)")], true);
}

#[test]
fn rejected_fixture_emits_no_backend_artifacts() {
    check(
        "fn main() -> I64 begin\n    let x: I64 = 1;\n    x = true;\n    return 0;\nend\n",
        &[mismatch(
            "TypeMismatch",
            "file 0:3:5-3:14",
            "Ralat: Jenis tugasan tidak sepadan: dijangka Primitive(I64), diterima Primitive(Bool)",
            "Error: Assignment type mismatch: expected Primitive(I64), got Primitive(Bool)",
        )],
        true,
    );
}

#[test]
fn assignment_target_error_is_exact_and_pre_llvm() {
    let (mut lowered, types) =
        lower("fn main() -> I64 begin\n    let x: I64 = 1;\n    x = 2;\n    return 0;\nend\n");
    let span = at(3, 5, 3, 11);
    let assign = &mut body_mut(&mut lowered, "main").statements[1];
    assign.span = span;
    match &mut assign.node {
        HirStmt::Assign { target, value } => {
            assert!(matches!(target.kind, HirExprKind::Local(_)));
            // The parser cannot spell a literal target; inject it into HIR.
            *target = value.clone();
            assert!(matches!(target.kind, HirExprKind::Literal(_)));
        }
        other => panic!("expected assignment, got {other:?}"),
    }

    let diagnostics = assert_codegen_fails_at_gate("assignment_target", &lowered, &types);
    assert_exact(
        &diagnostics,
        &[(
            DiagnosticCode::TypeMismatch,
            "Ralat: Sasaran tugasan tidak boleh ditulis: literal",
            "Error: Assignment target is not writable: literal",
            span,
        )],
    );
}

#[test]
fn assignment_type_mismatch_is_exact_and_pre_llvm() {
    let (mut lowered, types) =
        lower("fn main() -> I64 begin\n    let x: I64 = 1;\n    x = true;\n    return 0;\nend\n");
    let span = at(3, 5, 3, 14);
    body_mut(&mut lowered, "main").statements[1].span = span;

    let diagnostics = assert_codegen_fails_at_gate("assignment_type", &lowered, &types);
    assert_exact(
        &diagnostics,
        &[(
            DiagnosticCode::TypeMismatch,
            ASSIGN_MISMATCH_MS,
            ASSIGN_MISMATCH_EN,
            span,
        )],
    );
}

#[test]
fn nested_assignment_type_mismatch_is_exact_and_pre_llvm() {
    let (mut lowered, types) = lower(
        "fn main() -> I64 begin\n    let x: I64 = 1;\n    if true begin\n        while false begin\n            x = true;\n        end\n    end\n    return 0;\nend\n",
    );
    let span = at(5, 13, 5, 22);
    let body = body_mut(&mut lowered, "main");
    let inner = &mut loop_body(&mut then_branch(&mut body.statements[1].node).statements[0].node)
        .statements[0];
    assert!(matches!(inner.node, HirStmt::Assign { .. }));
    inner.span = span;

    let diagnostics = assert_codegen_fails_at_gate("nested_assignment", &lowered, &types);
    assert_exact(
        &diagnostics,
        &[(
            DiagnosticCode::TypeMismatch,
            ASSIGN_MISMATCH_MS,
            ASSIGN_MISMATCH_EN,
            span,
        )],
    );
}

#[test]
fn direct_return_mismatch_is_exact_and_pre_llvm() {
    let (mut lowered, types) = lower(
        "fn bad() -> I64 begin\n    return true;\nend\nfn main() -> I64 begin\n    return 0;\nend\n",
    );
    let span = at(2, 5, 2, 17);
    body_mut(&mut lowered, "bad").statements[0].span = span;

    let diagnostics = assert_codegen_fails_at_gate("direct_return", &lowered, &types);
    assert_exact(
        &diagnostics,
        &[(
            DiagnosticCode::TypeMismatch,
            RETURN_MISMATCH_MS,
            RETURN_MISMATCH_EN,
            span,
        )],
    );
}

#[test]
fn nested_return_mismatch_is_exact_and_pre_llvm() {
    let (mut lowered, types) = lower(
        "fn bad() -> I64 begin\n    if true begin\n        return true;\n    end\n    return 0;\nend\nfn main() -> I64 begin\n    return 0;\nend\n",
    );
    let span = at(3, 9, 3, 21);
    let body = body_mut(&mut lowered, "bad");
    let inner = &mut then_branch(&mut body.statements[0].node).statements[0];
    assert!(matches!(inner.node, HirStmt::Return(Some(_))));
    inner.span = span;

    let diagnostics = assert_codegen_fails_at_gate("nested_return", &lowered, &types);
    assert_exact(
        &diagnostics,
        &[(
            DiagnosticCode::TypeMismatch,
            RETURN_MISMATCH_MS,
            RETURN_MISMATCH_EN,
            span,
        )],
    );
}

#[test]
fn unit_return_value_emits_one_exact_diagnostic_and_is_pre_llvm() {
    let (mut lowered, types) = lower(
        "fn bad() -> Unit begin\n    return 1;\nend\nfn main() -> I64 begin\n    return 0;\nend\n",
    );
    let span = at(2, 5, 2, 14);
    body_mut(&mut lowered, "bad").statements[0].span = span;

    let diagnostics = assert_codegen_fails_at_gate("unit_return_value", &lowered, &types);
    assert_exact(
        &diagnostics,
        &[(
            DiagnosticCode::TypeMismatch,
            "Ralat: Fungsi Unit tidak boleh memulangkan nilai jenis Primitive(I64)",
            "Error: Unit function cannot return a value of type Primitive(I64)",
            span,
        )],
    );
    assert!(
        diagnostics
            .iter()
            .all(|d| !d.message_en.starts_with("Error: Return type mismatch")),
        "generic return mismatch must not duplicate the Unit diagnostic: {diagnostics:?}"
    );
}

#[test]
fn missing_return_path_is_exact() {
    let (mut lowered, types) = lower(
        "fn bad() -> I64 begin\n    let x: I64 = 1;\nend\nfn main() -> I64 begin\n    return 0;\nend\n",
    );
    let span = at(1, 1, 3, 4);
    let item = lowered
        .module
        .items
        .iter_mut()
        .find(|item| matches!(&item.node, HirItem::Function(f) if f.name == "bad"))
        .expect("lowered function `bad`");
    item.span = span;

    let diagnostics = gate(&lowered, &types).expect_err("missing return must fail");
    assert_exact(
        &diagnostics,
        &[(
            DiagnosticCode::TypeMismatch,
            "Ralat: Fungsi yang memulangkan Primitive(I64) mesti mempunyai laluan return yang dijamin",
            "Error: Function returning Primitive(I64) must have a guaranteed return path",
            span,
        )],
    );
}

#[test]
fn returned_callable_result_mismatch_is_exact_and_pre_llvm() {
    let (mut lowered, types) = lower(
        "fn flag() -> Bool begin\n    return true;\nend\nfn main() -> I64 begin\n    return flag();\nend\n",
    );
    let flag = callable_named(&lowered, "flag");
    let frozen_return = lowered.callable_signatures[&flag].return_type;
    assert_eq!(frozen_return, types.primitive_ids().bool_);

    let span = at(5, 5, 5, 19);
    let stmt = &mut body_mut(&mut lowered, "main").statements[0];
    stmt.span = span;
    let call = return_value(&mut stmt.node);
    match &call.kind {
        HirExprKind::Call { callee, .. } => assert_eq!(*callee, flag),
        other => panic!("expected call, got {other:?}"),
    }
    // The call result stamp is the frozen PR #104 return type.
    assert_eq!(call.ty.id, frozen_return);

    let diagnostics = assert_codegen_fails_at_gate("returned_callable", &lowered, &types);
    assert_exact(
        &diagnostics,
        &[(
            DiagnosticCode::TypeMismatch,
            RETURN_MISMATCH_MS,
            RETURN_MISMATCH_EN,
            span,
        )],
    );
}

#[test]
fn function_return_context_is_restored() {
    let (mut lowered, types) = lower(
        "fn first() -> Bool begin\n    return 1;\nend\nfn second() -> I64 begin\n    return true;\nend\nfn main() -> I64 begin\n    return 0;\nend\n",
    );
    let first = at(2, 5, 2, 14);
    let second = at(5, 5, 5, 17);
    body_mut(&mut lowered, "first").statements[0].span = first;
    body_mut(&mut lowered, "second").statements[0].span = second;

    let diagnostics = assert_codegen_fails_at_gate("return_context", &lowered, &types);
    assert_exact(
        &diagnostics,
        &[
            (
                DiagnosticCode::TypeMismatch,
                "Ralat: Jenis pulangan tidak sepadan: dijangka Primitive(Bool), diterima Primitive(I64)",
                "Error: Return type mismatch: expected Primitive(Bool), got Primitive(I64)",
                first,
            ),
            (
                DiagnosticCode::TypeMismatch,
                RETURN_MISMATCH_MS,
                RETURN_MISMATCH_EN,
                second,
            ),
        ],
    );
}

#[test]
fn unknown_return_type_fails_closed() {
    // The parser cannot represent an Unknown-typed returned value; stamp the
    // registry's in-range Unknown type directly into HIR.
    let (mut lowered, types) = lower(
        "fn bad() -> I64 begin\n    return 1;\nend\nfn main() -> I64 begin\n    return 0;\nend\n",
    );
    let span = at(2, 5, 2, 14);
    let stmt = &mut body_mut(&mut lowered, "bad").statements[0];
    stmt.span = span;
    return_value(&mut stmt.node).ty.id = types.unknown();

    let diagnostics = assert_codegen_fails_at_gate("unknown_return_type", &lowered, &types);
    assert_exact(
        &diagnostics,
        &[(
            DiagnosticCode::TypeMismatch,
            "Ralat: Jenis pulangan tidak diketahui",
            "Error: Return type is unknown",
            span,
        )],
    );

    // A Unit function returning an Unknown-typed value keeps both the
    // Unit-specific and the unknown-type diagnostics.
    let (mut lowered, types) = lower(
        "fn bad() -> Unit begin\n    return 1;\nend\nfn main() -> I64 begin\n    return 0;\nend\n",
    );
    let stmt = &mut body_mut(&mut lowered, "bad").statements[0];
    stmt.span = span;
    return_value(&mut stmt.node).ty.id = types.unknown();

    let diagnostics = assert_codegen_fails_at_gate("unknown_unit_return", &lowered, &types);
    assert_exact(
        &diagnostics,
        &[
            (
                DiagnosticCode::TypeMismatch,
                "Ralat: Fungsi Unit tidak boleh memulangkan nilai jenis Unknown",
                "Error: Unit function cannot return a value of type Unknown",
                span,
            ),
            (
                DiagnosticCode::TypeMismatch,
                "Ralat: Jenis pulangan tidak diketahui",
                "Error: Return type is unknown",
                span,
            ),
        ],
    );
}

#[test]
fn compatible_assignment_and_return_are_accepted() {
    let (lowered, types) =
        lower("fn main() -> I64 begin\n    let x: I64 = 1;\n    x = 2;\n    return x;\nend\n");
    assert_accepted("compatible_assignment", &lowered, &types);
}

#[test]
fn compatible_nested_return_and_callable_result_are_accepted() {
    let (lowered, types) = lower(
        "fn one() -> I64 begin\n    return 1;\nend\nfn main() -> I64 begin\n    if true begin\n        return one();\n    end\n    return 0;\nend\n",
    );
    let one = callable_named(&lowered, "one");
    assert_eq!(
        lowered.callable_signatures[&one].return_type,
        types.primitive_ids().i64_
    );
    assert_accepted("compatible_nested", &lowered, &types);
}

#[test]
fn unit_function_without_value_return_is_accepted() {
    let (lowered, types) = lower(
        "fn noop() -> Unit begin\n    let x: I64 = 1;\nend\nfn main() -> I64 begin\n    noop();\n    return 0;\nend\n",
    );
    assert_accepted("unit_without_value", &lowered, &types);
}
