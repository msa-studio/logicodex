use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use logicodex::ffi::{
    CallableRegistry, CallableSafety, CallableSignature, CallingConvention, CapabilityPolicy,
    FfiGatekeeper, SafetyContext,
};
use logicodex::hir::{HirExpr, HirExprKind, LiteralAst};
use logicodex::span::{Diagnostic, DiagnosticCode, Severity, Span};
use logicodex::types::{TypeRef, TypeRegistry};

fn expr_i64(types: &TypeRegistry, span: Span) -> HirExpr {
    HirExpr {
        kind: HirExprKind::Literal(LiteralAst::Integer(1)),
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
fn sig(
    name: &str,
    params: Vec<logicodex::types::TypeId>,
    safety: CallableSafety,
    is_extern: bool,
    variadic: bool,
) -> CallableSignature {
    CallableSignature {
        name: name.to_string(),
        params,
        return_type: logicodex::types::TypeId(0),
        abi: CallingConvention::C,
        safety,
        is_extern,
        is_variadic: variadic,
    }
}
fn direct(
    signature: CallableSignature,
    args: Vec<HirExpr>,
    context: SafetyContext,
    call_span: Span,
) -> Diagnostic {
    let types = TypeRegistry::new();
    let policy = CapabilityPolicy::with_runtime_builtins();
    FfiGatekeeper {
        types: &types,
        callables: None,
        policy: Some(&policy),
    }
    .validate_call(&signature, &args, context, call_span)
    .expect_err("fixture must reject")
}
fn assert_record(d: &Diagnostic, code: DiagnosticCode, span: Span, ms: &str, en: &str) {
    assert_eq!(d.code, code);
    assert_eq!(d.severity, Severity::Error);
    assert_eq!(d.primary_span, span);
    assert!(d.notes.is_empty());
    assert_eq!(d.message_ms, ms);
    assert_eq!(d.message_en, en);
}

#[test]
fn ffi_unsafe_context_span_targets_call() {
    let types = TypeRegistry::new();
    let span = Span::new(logicodex::span::FileId(0), 3, 5, 3, 14);
    let d = direct(
        sig(
            "puts",
            vec![types.primitive_ids().i64_],
            CallableSafety::UnsafeRequired,
            true,
            false,
        ),
        vec![expr_i64(
            &types,
            Span::new(logicodex::span::FileId(0), 3, 10, 3, 11),
        )],
        SafetyContext::Safe,
        span,
    );
    assert_record(
        &d,
        DiagnosticCode::FfiBoundaryViolation,
        span,
        "Ralat: Panggilan fungsi luar 'puts' memerlukan blok unsafe",
        "Error: External function call 'puts' requires an unsafe block",
    );
}
#[test]
fn ffi_fixed_arity_span_targets_call() {
    let types = TypeRegistry::new();
    let span = Span::new(logicodex::span::FileId(0), 4, 5, 4, 18);
    let d = direct(
        sig(
            "fixed",
            vec![types.primitive_ids().i64_, types.primitive_ids().i64_],
            CallableSafety::Safe,
            false,
            false,
        ),
        vec![expr_i64(&types, Span::unknown())],
        SafetyContext::Safe,
        span,
    );
    assert_record(
        &d,
        DiagnosticCode::FfiBoundaryViolation,
        span,
        "Ralat: Fungsi 'fixed' memerlukan 2 argumen tetapi menerima 1",
        "Error: Function 'fixed' expects 2 argument(s) but received 1",
    );
}
#[test]
fn ffi_variadic_minimum_span_targets_call() {
    let types = TypeRegistry::new();
    let span = Span::new(logicodex::span::FileId(0), 5, 5, 5, 16);
    let d = direct(
        sig(
            "variadic",
            vec![types.primitive_ids().i64_],
            CallableSafety::Safe,
            false,
            true,
        ),
        Vec::new(),
        SafetyContext::Safe,
        span,
    );
    assert_record(
        &d,
        DiagnosticCode::FfiBoundaryViolation,
        span,
        "Ralat: Fungsi variadik 'variadic' memerlukan sekurang-kurangnya 1 argumen",
        "Error: Variadic function 'variadic' expects at least 1 argument(s)",
    );
}
#[test]
fn ffi_argument_type_span_targets_bad_argument() {
    let types = TypeRegistry::new();
    let arg = Span::new(logicodex::span::FileId(0), 6, 16, 6, 21);
    let d = direct(
        sig(
            "typed",
            vec![types.primitive_ids().i64_],
            CallableSafety::Safe,
            false,
            false,
        ),
        vec![expr_unit(&types, arg)],
        SafetyContext::Safe,
        Span::new(logicodex::span::FileId(0), 6, 5, 6, 22),
    );
    assert_record(&d,DiagnosticCode::FfiBoundaryViolation,arg,"Ralat: Argumen 1 untuk fungsi 'typed' mempunyai jenis yang tidak sepadan (diperlukan: , diterima: )","Error: Argument 1 for function 'typed' has an incompatible type (expected: , got: )");
}
#[test]
fn ffi_policy_span_targets_call() {
    let types = TypeRegistry::new();
    let span = Span::new(logicodex::span::FileId(0), 7, 5, 7, 14);
    let d = direct(
        sig("blocked", Vec::new(), CallableSafety::Safe, true, false),
        Vec::new(),
        SafetyContext::Unsafe,
        span,
    );
    assert_record(&d,DiagnosticCode::FfiBoundaryViolation,span,"Ralat: panggilan extern 'blocked' ditolak oleh polisi keupayaan FFI. Isytiharkannya dalam ffi.allow sebelum guna.","Error: extern call 'blocked' denied by FFI capability policy. Declare it in ffi.allow before use.");
}
#[test]
fn nested_call_spans_remain_distinct() {
    let outer = Span::new(logicodex::span::FileId(0), 2, 5, 2, 20);
    let inner = Span::new(logicodex::span::FileId(0), 2, 10, 2, 18);
    let arg = Span::new(logicodex::span::FileId(0), 2, 14, 2, 17);
    assert_ne!(outer, inner);
    assert_ne!(inner, arg);
    assert!(outer.start_col < inner.start_col);
    assert!(arg.start_col > inner.start_col);
}
#[test]
fn synthetic_ffi_hir_span_remains_unknown() {
    assert_eq!(
        Span::unknown(),
        Span::new(logicodex::span::FileId(0), 0, 0, 0, 0)
    );
}

fn backend_artifacts(root: &std::path::Path) -> Vec<PathBuf> {
    fn walk(p: &std::path::Path, out: &mut Vec<PathBuf>) {
        if let Ok(rd) = std::fs::read_dir(p) {
            for e in rd.flatten() {
                let q = e.path();
                if q.is_dir() {
                    walk(&q, out)
                } else if matches!(
                    q.extension().and_then(|s| s.to_str()),
                    Some("ll" | "bc" | "o" | "obj" | "exe")
                ) || q.file_name().and_then(|s| s.to_str()) == Some("main")
                {
                    out.push(q)
                }
            }
        }
    }
    let mut out = Vec::new();
    walk(root, &mut out);
    out
}
fn cli_fixture(source: &str, expected_codes: &[&str], expected_spans: &[&str]) {
    static C: AtomicU64 = AtomicU64::new(0);
    let root = std::env::temp_dir().join(format!(
        "logicodex_b2_ffi_{}_{}",
        std::process::id(),
        C.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("tempdir");
    let file = root.join("main.ldx");
    std::fs::write(&file, source).expect("fixture");
    let out = Command::new(env!("CARGO_BIN_EXE_logicodex"))
        .env(
            "LOGICODEX_STD",
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("lib"),
        )
        .arg("check")
        .arg(&file)
        .output()
        .expect("check");
    assert!(!out.status.success());
    let text = format!(
        "{}\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let codes: Vec<&str> = text
        .lines()
        .filter_map(|l| {
            l.strip_prefix("Error: code: ")
                .or_else(|| l.strip_prefix("code: "))
        })
        .collect();
    assert_eq!(codes, expected_codes, "{text}");
    for span in expected_spans {
        assert!(text.contains(span), "{text}")
    }
    assert!(backend_artifacts(&root).is_empty());
    let _ = std::fs::remove_dir_all(root);
}
#[test]
fn ffi_policy_span_targets_call_cli() {
    cli_fixture("fn main() -> I64 begin\n    puts(1);\n    return 0;\nend\nextern \"C\" fn puts(value: I64);\n",&["FfiBoundaryViolation"],&["span: file 0:2:5-2:12"])
}
#[test]
fn ffi_fixed_arity_span_targets_call_cli() {
    cli_fixture("fn main() -> I64 begin\n    puts();\n    return 0;\nend\nextern \"C\" fn puts(value: I64);\n",&["TypeMismatch","FfiBoundaryViolation"],&["span: file 0:2:5-2:11"])
}
#[test]
fn ffi_argument_type_span_targets_bad_argument_cli() {
    cli_fixture("fn main() -> I64 begin\n    add(1, false);\n    return 0;\nend\nextern \"C\" fn add(a: I64, b: I64);\n",&["TypeMismatch","FfiBoundaryViolation"],&["span: file 0:2:12-2:17","span: file 0:2:5-2:18"])
}
