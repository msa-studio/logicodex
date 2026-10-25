use logicodex::ast::{Expr, Param, Program, Stmt, Type};
use logicodex::hir::{HirCallableKind, LoweringContext, SymbolTable};
use logicodex::span::{Diagnostic, Span};
use logicodex::types::{CallableId, TypeRegistry};

fn lower_program_compatibility(
    program: Program,
) -> Result<logicodex::hir::HirModule, Vec<Diagnostic>> {
    let mut symbols = SymbolTable::default();
    let mut types = TypeRegistry::new();
    let mut context = LoweringContext {
        symbols: &mut symbols,
        types: &mut types,
        diagnostics: Vec::new(),
        current_module: String::new(),
        current_return_enum: None,
    };
    context.lower_program(program)
}

fn lower_program_additive(
    program: Program,
) -> Result<logicodex::hir::LoweredModule, Vec<Diagnostic>> {
    let mut symbols = SymbolTable::default();
    let mut types = TypeRegistry::new();
    let mut context = LoweringContext {
        symbols: &mut symbols,
        types: &mut types,
        diagnostics: Vec::new(),
        current_module: String::new(),
        current_return_enum: None,
    };
    context.lower_program_with_metadata(program)
}

fn lower_module_compatibility(
    module_name: &str,
    program: Program,
) -> Result<logicodex::hir::HirModule, Vec<Diagnostic>> {
    let mut symbols = SymbolTable::default();
    let mut types = TypeRegistry::new();
    let mut context = LoweringContext {
        symbols: &mut symbols,
        types: &mut types,
        diagnostics: Vec::new(),
        current_module: String::new(),
        current_return_enum: None,
    };
    context.lower_module_program(module_name, program)
}

fn lower_module_additive(
    module_name: &str,
    program: Program,
) -> Result<logicodex::hir::LoweredModule, Vec<Diagnostic>> {
    let mut symbols = SymbolTable::default();
    let mut types = TypeRegistry::new();
    let mut context = LoweringContext {
        symbols: &mut symbols,
        types: &mut types,
        diagnostics: Vec::new(),
        current_module: String::new(),
        current_return_enum: None,
    };
    context.lower_module_program_with_metadata(module_name, program)
}

fn main_program() -> Program {
    Program::new(vec![Stmt::Function {
        name: "main".to_string(),
        params: vec![Param {
            name: "value".to_string(),
            ty: Type::I64,
        }],
        return_type: Some(Type::I64),
        body: vec![Stmt::Return {
            value: Expr::Variable("value".to_string()),
        }],
        span: Span::unknown(),
        is_public: false,
    }])
}

fn module_program() -> Program {
    Program::new(vec![Stmt::Function {
        name: "add_one".to_string(),
        params: vec![Param {
            name: "value".to_string(),
            ty: Type::I64,
        }],
        return_type: Some(Type::I64),
        body: vec![Stmt::Return {
            value: Expr::Variable("value".to_string()),
        }],
        span: Span::unknown(),
        is_public: true,
    }])
}

fn invalid_program() -> Program {
    Program::new(vec![Stmt::Function {
        name: "main".to_string(),
        params: Vec::new(),
        return_type: None,
        body: vec![Stmt::ExprStmt {
            value: Expr::Variable("missing".to_string()),
        }],
        span: Span::unknown(),
        is_public: false,
    }])
}

#[test]
fn lower_program_compatibility_preserves_exact_hir_module() {
    let compatibility =
        lower_program_compatibility(main_program()).expect("compatibility lowering");
    let additive = lower_program_additive(main_program()).expect("additive lowering");

    assert_eq!(compatibility, additive.module);
}

#[test]
fn lower_module_program_compatibility_preserves_exact_hir_module() {
    let compatibility =
        lower_module_compatibility("math", module_program()).expect("compatibility lowering");
    let additive = lower_module_additive("math", module_program()).expect("additive lowering");

    assert_eq!(compatibility, additive.module);
}

#[test]
fn additive_artifact_preserves_callable_ids_and_names() {
    let program_artifact =
        lower_program_additive(Program::new(Vec::new())).expect("additive lowering");
    let module_artifact =
        lower_module_additive("math", Program::new(Vec::new())).expect("module additive lowering");

    for artifact in [program_artifact, module_artifact] {
        assert_eq!(artifact.builtin_provenance.len(), 3);
        for (expected_id, name) in [(0, "print"), (1, "logicodex_sleep"), (2, "logicodex_yield")] {
            let callable = artifact
                .builtin_provenance
                .get(name)
                .copied()
                .expect("canonical builtin provenance");
            assert_eq!(callable, CallableId(expected_id));
            let signature = artifact
                .callable_signatures
                .get(&callable)
                .expect("builtin provenance must point to a signature");
            assert_eq!(signature.name, name);
            assert_eq!(signature.kind, HirCallableKind::Builtin);
            assert_eq!(signature.callable, callable);
        }
    }
}

#[test]
fn module_source_lowering_error_codes_messages_and_spans_are_equivalent() {
    let compatibility =
        lower_module_compatibility("math", invalid_program()).expect_err("must fail");
    let additive = lower_module_additive("math", invalid_program()).expect_err("must fail");

    assert_eq!(compatibility, additive);
}

#[test]
fn source_lowering_error_codes_messages_and_spans_are_equivalent() {
    let compatibility = lower_program_compatibility(invalid_program()).expect_err("must fail");
    let additive = lower_program_additive(invalid_program()).expect_err("must fail");

    assert_eq!(compatibility, additive);
}
