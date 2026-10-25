use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use logicodex::ffi::{
    CallableRegistry, CallableSafety, CallableSignature, CallingConvention, CapabilityPolicy,
    FfiGatekeeper, SafetyContext,
};
use logicodex::hir::{
    HirBlock, HirExpr, HirExprKind, HirFunction, HirItem, HirModule, HirStmt, LiteralAst, SymbolId,
    SymbolTable,
};
use logicodex::semantic_gate::SemanticContext;
use logicodex::span::{DiagnosticCode, Severity, Span, Spanned};
use logicodex::types::{TypeId, TypeRef, TypeRegistry};

fn spanned<T>(node: T) -> Spanned<T> {
    Spanned {
        node,
        span: Span::unknown(),
    }
}
fn direct_context() -> SemanticContext {
    SemanticContext {
        types: TypeRegistry::new(),
        symbols: SymbolTable::default(),
        callables: CallableRegistry::default(),
        diagnostics: Vec::new(),
        loop_depth: 0,
        safety_context: SafetyContext::Safe,
        current_return_type: None,
        policy: CapabilityPolicy::with_runtime_builtins(),
        extern_symbols: HashSet::new(),
    }
}

fn expr_i64(types: &TypeRegistry, value: i64, span: Span) -> HirExpr {
    HirExpr {
        kind: HirExprKind::Literal(LiteralAst::Integer(value)),
        ty: TypeRef {
            id: types.primitive_ids().i64_,
        },
        span,
    }
}

fn expr_unit(types: &TypeRegistry, span: Span) -> HirExpr {
    HirExpr {
        kind: HirExprKind::Literal(LiteralAst::Boolean(false)),
        ty: TypeRef {
            id: types.primitive_ids().unit,
        },
        span,
    }
}

fn signature(
    name: &str,
    params: Vec<TypeId>,
    safety: CallableSafety,
    is_extern: bool,
    is_variadic: bool,
) -> CallableSignature {
    CallableSignature {
        name: name.to_string(),
        params,
        return_type: TypeId(0),
        abi: CallingConvention::C,
        safety,
        is_extern,
        is_variadic,
    }
}

fn assert_record(
    diagnostic: &logicodex::span::Diagnostic,
    code: DiagnosticCode,
    ms: &str,
    en: &str,
    span: Span,
) {
    assert_eq!(diagnostic.code, code);
    assert_eq!(diagnostic.severity, Severity::Error);
    assert_eq!(diagnostic.message_ms, ms);
    assert_eq!(diagnostic.message_en, en);
    assert!(diagnostic.notes.is_empty());
    assert_eq!(diagnostic.primary_span, span);
}

#[test]
fn unsafe_break_direct_record_is_exact() {
    let mut ctx = direct_context();
    let module = HirModule {
        items: vec![spanned(HirItem::Function(HirFunction {
            name: "test".to_string(),
            symbol: SymbolId(0),
            params: Vec::new(),
            return_type: TypeRef {
                id: ctx.types.primitive_ids().unit,
            },
            body: HirBlock {
                statements: vec![spanned(HirStmt::Break { target_depth: 0 })],
            },
            safety: SafetyContext::Safe,
        }))],
    };
    let diagnostics = ctx.check_module(&module).expect_err("break must fail");
    assert_eq!(diagnostics.len(), 1);
    assert_record(
        &diagnostics[0],
        DiagnosticCode::UnsafeBoundaryViolation,
        "Ralat: Kawalan gelung digunakan di luar gelung",
        "Error: Loop control used outside a loop",
        Span::unknown(),
    );
}

#[test]
fn unsafe_continue_direct_record_is_exact() {
    let mut ctx = direct_context();
    let module = HirModule {
        items: vec![spanned(HirItem::Function(HirFunction {
            name: "test".to_string(),
            symbol: SymbolId(0),
            params: Vec::new(),
            return_type: TypeRef {
                id: ctx.types.primitive_ids().unit,
            },
            body: HirBlock {
                statements: vec![spanned(HirStmt::Continue { target_depth: 0 })],
            },
            safety: SafetyContext::Safe,
        }))],
    };
    let diagnostics = ctx.check_module(&module).expect_err("continue must fail");
    assert_eq!(diagnostics.len(), 1);
    assert_record(
        &diagnostics[0],
        DiagnosticCode::UnsafeBoundaryViolation,
        "Ralat: Kawalan gelung digunakan di luar gelung",
        "Error: Loop control used outside a loop",
        Span::unknown(),
    );
}

#[test]
fn duplicate_function_direct_record_is_exact() {
    let mut ctx = direct_context();
    let function = |symbol| {
        HirItem::Function(HirFunction {
            name: "duplicate_target".to_string(),
            symbol,
            params: Vec::new(),
            return_type: TypeRef {
                id: ctx.types.primitive_ids().i64_,
            },
            body: HirBlock {
                statements: vec![spanned(HirStmt::Return(Some(expr_i64(
                    &ctx.types,
                    1,
                    Span::unknown(),
                ))))],
            },
            safety: SafetyContext::Safe,
        })
    };
    let module = HirModule {
        items: vec![
            spanned(function(SymbolId(0))),
            spanned(function(SymbolId(1))),
        ],
    };
    let diagnostics = ctx.check_module(&module).expect_err("duplicate must fail");
    assert_eq!(diagnostics.len(), 1);
    assert_record(
        &diagnostics[0],
        DiagnosticCode::DuplicateDefinition,
        "Ralat: Fungsi 'duplicate_target' ditakrif lebih daripada sekali",
        "Error: Function 'duplicate_target' is defined more than once",
        Span::unknown(),
    );
}

fn validate(
    types: &TypeRegistry,
    signature: CallableSignature,
    args: Vec<HirExpr>,
    context: SafetyContext,
) -> logicodex::span::Diagnostic {
    let policy = CapabilityPolicy::with_runtime_builtins();
    let gate = FfiGatekeeper {
        types: &types,
        callables: None,
        policy: Some(&policy),
    };
    gate.validate_call(&signature, &args, context, Span::unknown())
        .expect_err("fixture must fail")
}

#[test]
fn ffi_unsafe_context_direct_record_is_exact() {
    let types = TypeRegistry::new();
    let ids = types.primitive_ids();
    let diagnostic = validate(
        &types,
        signature(
            "puts",
            vec![ids.i64_],
            CallableSafety::UnsafeRequired,
            true,
            false,
        ),
        vec![expr_i64(&types, 1, Span::unknown())],
        SafetyContext::Safe,
    );
    assert_record(
        &diagnostic,
        DiagnosticCode::FfiBoundaryViolation,
        "Ralat: Panggilan fungsi luar 'puts' memerlukan blok unsafe",
        "Error: External function call 'puts' requires an unsafe block",
        Span::unknown(),
    );
}

#[test]
fn ffi_fixed_arity_direct_record_is_exact() {
    let types = TypeRegistry::new();
    let ids = types.primitive_ids();
    let diagnostic = validate(
        &types,
        signature(
            "fixed_arity",
            vec![ids.i64_, ids.i64_],
            CallableSafety::Safe,
            false,
            false,
        ),
        vec![expr_i64(&types, 1, Span::unknown())],
        SafetyContext::Safe,
    );
    assert_record(
        &diagnostic,
        DiagnosticCode::FfiBoundaryViolation,
        "Ralat: Fungsi 'fixed_arity' memerlukan 2 argumen tetapi menerima 1",
        "Error: Function 'fixed_arity' expects 2 argument(s) but received 1",
        Span::unknown(),
    );
}

#[test]
fn ffi_variadic_minimum_direct_record_is_exact() {
    let types = TypeRegistry::new();
    let ids = types.primitive_ids();
    let diagnostic = validate(
        &types,
        signature(
            "variadic",
            vec![ids.i64_],
            CallableSafety::Safe,
            false,
            true,
        ),
        Vec::new(),
        SafetyContext::Safe,
    );
    assert_record(
        &diagnostic,
        DiagnosticCode::FfiBoundaryViolation,
        "Ralat: Fungsi variadik 'variadic' memerlukan sekurang-kurangnya 1 argumen",
        "Error: Variadic function 'variadic' expects at least 1 argument(s)",
        Span::unknown(),
    );
}

#[test]
fn ffi_argument_type_direct_record_is_exact() {
    let types = TypeRegistry::new();
    let ids = types.primitive_ids();
    let diagnostic = validate(
        &types,
        signature("typed", vec![ids.i64_], CallableSafety::Safe, false, false),
        vec![expr_unit(&types, Span::unknown())],
        SafetyContext::Safe,
    );
    assert_record(&diagnostic, DiagnosticCode::FfiBoundaryViolation, "Ralat: Argumen 1 untuk fungsi 'typed' mempunyai jenis yang tidak sepadan (diperlukan: , diterima: )", "Error: Argument 1 for function 'typed' has an incompatible type (expected: , got: )", Span::unknown());
}

#[test]
fn ffi_capability_policy_direct_record_is_exact() {
    let types = TypeRegistry::new();
    let ids = types.primitive_ids();
    let diagnostic = validate(
        &types,
        signature("blocked", Vec::new(), CallableSafety::Safe, true, false),
        Vec::new(),
        SafetyContext::Unsafe,
    );
    assert_record(&diagnostic, DiagnosticCode::FfiBoundaryViolation, "Ralat: panggilan extern 'blocked' ditolak oleh polisi keupayaan FFI. Isytiharkannya dalam ffi.allow sebelum guna.", "Error: extern call 'blocked' denied by FFI capability policy. Declare it in ffi.allow before use.", Span::unknown());
    let _ = ids;
}

fn backend_artifacts(root: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(dir).expect("read artifact directory") {
            let path = entry.expect("read artifact entry").path();
            if path.is_dir() {
                stack.push(path);
            } else if matches!(
                path.extension().and_then(|s| s.to_str()),
                Some("ll" | "bc" | "o" | "obj" | "exe")
            ) || path.file_name().and_then(|s| s.to_str()) == Some("main")
            {
                found.push(path);
            }
        }
    }
    found
}

fn cli_fixture(source: &str, expected_code: &str, expected_message: &str) {
    static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);
    let temp = std::env::temp_dir().join(format!(
        "logicodex_cpb2d_cli_{}_{}",
        std::process::id(),
        TEMP_COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&temp);
    std::fs::create_dir_all(&temp).expect("tempdir");
    let root = temp.join("main.ldx");
    std::fs::write(&root, source).expect("fixture");
    let output = Command::new(env!("CARGO_BIN_EXE_logicodex"))
        .env(
            "LOGICODEX_STD",
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("lib"),
        )
        .arg("check")
        .arg(&root)
        .output()
        .expect("spawn check");
    assert!(!output.status.success());
    let rendered = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        rendered
            .lines()
            .filter(|line| line.starts_with("Error: code: "))
            .count(),
        1
    );
    assert!(rendered.contains(&format!("Error: code: {expected_code}")));
    assert!(rendered.contains(expected_message));
    assert!(rendered.contains("span: file 0:0:0-0:0"));
    assert!(backend_artifacts(&temp).is_empty());
    let _ = std::fs::remove_dir_all(&temp);
}

#[test]
fn duplicate_function_cli_rejects_before_codegen() {
    cli_fixture("function duplicate_target() -> I64 begin\n    return 1;\nend\nfunction duplicate_target() -> I64 begin\n    return 2;\nend\n", "DuplicateDefinition", "Ralat: Fungsi 'duplicate_target' ditakrif lebih daripada sekali / Error: Function 'duplicate_target' is defined more than once");
}

#[test]
fn break_outside_loop_cli_rejects_before_codegen() {
    cli_fixture(
        "function main() -> Unit begin\n    break;\nend\n",
        "UnsafeBoundaryViolation",
        "Ralat: Kawalan gelung digunakan di luar gelung / Error: Loop control used outside a loop",
    );
}
