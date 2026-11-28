use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use logicodex::ast::{Expr, Param, Program, Stmt, Type};
use logicodex::hir::{LoweringContext, SymbolTable};
use logicodex::span::{FileId, Span};
use logicodex::types::TypeRegistry;

fn check_fixture(source: &str) -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let root = std::env::temp_dir().join(format!(
        "logicodex_span_propagation_{}_{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("tempdir");
    let source_path = root.join("main.ldx");
    std::fs::write(&source_path, source).expect("fixture");
    let output = Command::new(env!("CARGO_BIN_EXE_logicodex"))
        .env(
            "LOGICODEX_STD",
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("lib"),
        )
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("spawn check");
    assert!(!output.status.success());
    let rendered = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let _ = std::fs::remove_dir_all(&root);
    rendered
}

fn assert_record(rendered: &str, code: &str, message: &str, span: &str) {
    assert_eq!(
        rendered
            .lines()
            .filter(|line| line.starts_with("Error: code: "))
            .count(),
        1
    );
    assert!(
        rendered.contains(&format!("Error: code: {code}")),
        "{rendered}"
    );
    assert!(rendered.contains(message), "{rendered}");
    assert!(rendered.contains(span), "{rendered}");
}

#[test]
fn duplicate_function_cli_reports_second_declaration_span() {
    let rendered = check_fixture(
        "function duplicate_target() -> I64 begin\n    return 1;\nend\nfunction duplicate_target() -> I64 begin\n    return 2;\nend\n",
    );
    assert_record(
        &rendered,
        "DuplicateDefinition",
        "Ralat: Fungsi 'duplicate_target' ditakrif lebih daripada sekali / Error: Function 'duplicate_target' is defined more than once",
        "span: file 0:4:1-4:41",
    );
}

#[test]
fn break_cli_reports_break_statement_span() {
    let rendered = check_fixture("function main() -> Unit begin\n    break;\nend\n");
    assert_record(
        &rendered,
        "UnsafeBoundaryViolation",
        "Ralat: Kawalan gelung digunakan di luar gelung / Error: Loop control used outside a loop",
        "span: file 0:2:5-2:11",
    );
}

#[test]
fn continue_cli_reports_continue_statement_span() {
    let rendered = check_fixture("function main() -> Unit begin\n    continue;\nend\n");
    assert_record(
        &rendered,
        "UnsafeBoundaryViolation",
        "Ralat: Kawalan gelung digunakan di luar gelung / Error: Loop control used outside a loop",
        "span: file 0:2:5-2:14",
    );
}

#[test]
fn synthetic_hir_control_flow_preserves_unknown_span() {
    let mut symbols = SymbolTable::default();
    let mut types = TypeRegistry::new();
    let mut lowering = LoweringContext {
        symbols: &mut symbols,
        types: &mut types,
        diagnostics: Vec::new(),
        current_module: String::new(),
        current_return_enum: None,
    };
    let program = Program::new(vec![Stmt::Function {
        name: "synthetic".to_string(),
        params: vec![Param {
            name: "value".to_string(),
            ty: Type::I64,
        }],
        return_type: None,
        body: vec![Stmt::Break {
            span: Span::unknown(),
        }],
        span: Span::unknown(),
        is_public: false,
    }]);
    let lowered = lowering
        .lower_program_with_metadata(program)
        .expect("synthetic lowering");
    let item = &lowered.module.items[0];
    let logicodex::hir::HirItem::Function(function) = &item.node else {
        panic!("expected function item");
    };
    assert_eq!(item.span, Span::unknown());
    assert_eq!(function.body.statements[0].span, Span::unknown());
    assert_eq!(Span::unknown(), Span::new(FileId(0), 0, 0, 0, 0));
}

#[test]
fn existing_boundary_diagnostics_keep_code_and_messages() {
    let duplicate = check_fixture(
        "function duplicate_target() -> I64 begin\n    return 1;\nend\nfunction duplicate_target() -> I64 begin\n    return 2;\nend\n",
    );
    assert!(duplicate.contains("code: DuplicateDefinition"));
    let break_output = check_fixture("function main() -> Unit begin\n    break;\nend\n");
    assert!(break_output.contains("code: UnsafeBoundaryViolation"));
}

const LET_MISMATCH: &str = "Ralat: Jenis ikatan tidak sepadan: dijangka Primitive(I64), diterima Primitive(Bool) / Error: Binding type mismatch: expected Primitive(I64), got Primitive(Bool)";
const ASSIGN_MISMATCH: &str = "Ralat: Jenis tugasan tidak sepadan: dijangka Primitive(I64), diterima Primitive(Bool) / Error: Assignment type mismatch: expected Primitive(I64), got Primitive(Bool)";
const RETURN_MISMATCH: &str = "Ralat: Jenis pulangan tidak sepadan: dijangka Primitive(I64), diterima Primitive(Bool) / Error: Return type mismatch: expected Primitive(I64), got Primitive(Bool)";

#[test]
fn let_mismatch_reports_let_statement_span() {
    let rendered =
        check_fixture("function f() -> I64 begin\n    let x: I64 = true;\n    return 1;\nend\n");
    assert_record(
        &rendered,
        "TypeMismatch",
        LET_MISMATCH,
        "span: file 0:2:5-2:23",
    );
    let rendered = check_fixture(
        "function f() -> I64 begin\n    let a: I64 = 1;\n    let x: I64 = true;\n    return a;\nend\n",
    );
    assert_record(
        &rendered,
        "TypeMismatch",
        LET_MISMATCH,
        "span: file 0:3:5-3:23",
    );
}

#[test]
fn assign_mismatch_reports_assign_statement_span() {
    let rendered = check_fixture(
        "function f() -> I64 begin\n    let x: I64 = 1;\n    x = true;\n    return 1;\nend\n",
    );
    assert_record(
        &rendered,
        "TypeMismatch",
        ASSIGN_MISMATCH,
        "span: file 0:3:5-3:14",
    );
}

#[test]
fn return_mismatch_reports_return_statement_span() {
    let rendered = check_fixture("function f() -> I64 begin\n    return true;\nend\n");
    assert_record(
        &rendered,
        "TypeMismatch",
        RETURN_MISMATCH,
        "span: file 0:2:5-2:17",
    );
}

#[test]
fn missing_return_reports_function_declaration_span() {
    let rendered = check_fixture("function foo() -> I64 begin\n    let x = 1;\nend\n");
    assert_record(
        &rendered,
        "TypeMismatch",
        "Ralat: Fungsi yang memulangkan Primitive(I64) mesti mempunyai laluan return yang dijamin / Error: Function returning Primitive(I64) must have a guaranteed return path",
        "span: file 0:1:1-1:28",
    );
}

#[test]
fn known_statement_span_is_not_overridden_by_recovery() {
    let rendered = check_fixture(
        "function a() -> Bool begin\n    return true;\nend\nfunction f() -> I64 begin\n    return true;\nend\n",
    );
    assert_record(
        &rendered,
        "TypeMismatch",
        RETURN_MISMATCH,
        "span: file 0:5:5-5:17",
    );
}

#[test]
fn synthetic_ast_statements_lower_with_unknown_span() {
    let mut symbols = SymbolTable::default();
    let mut types = TypeRegistry::new();
    let mut lowering = LoweringContext {
        symbols: &mut symbols,
        types: &mut types,
        diagnostics: Vec::new(),
        current_module: String::new(),
        current_return_enum: None,
    };
    let program = Program::new(vec![Stmt::Function {
        name: "synthetic".to_string(),
        params: Vec::new(),
        return_type: Some(Type::I64),
        body: vec![
            Stmt::Let {
                name: "x".to_string(),
                declared_type: Some(Type::I64),
                value: Expr::Integer(1),
                span: Span::unknown(),
            },
            Stmt::Assign {
                target: Expr::Variable("x".to_string()),
                value: Expr::Integer(2),
                span: Span::unknown(),
            },
            Stmt::Return {
                value: Expr::Variable("x".to_string()),
                span: Span::unknown(),
            },
        ],
        span: Span::unknown(),
        is_public: false,
    }]);
    let lowered = lowering
        .lower_program_with_metadata(program)
        .expect("synthetic lowering");
    let logicodex::hir::HirItem::Function(function) = &lowered.module.items[0].node else {
        panic!("expected function item");
    };
    assert_eq!(function.body.statements.len(), 3);
    for statement in &function.body.statements {
        assert_eq!(statement.span, Span::unknown());
    }
}
