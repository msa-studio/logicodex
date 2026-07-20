use logicodex::hir::{
    BlockAst, ExternBlockAst, ExternFnAst, FieldAst, FunctionAst, HirCallableKind,
    HirCallableValueUse, HirItem, ItemAst, LoweringContext, ModuleAst, ParamAst, StmtAst,
    StructDeclAst, SymbolTable, TypeAst, VisibilityAst,
};
use logicodex::span::{Span, Spanned};
use logicodex::types::{TypeKind, TypeRegistry};

fn spanned<T>(node: T) -> Spanned<T> {
    Spanned {
        node,
        span: Span::unknown(),
    }
}

#[test]
fn lowered_artifact_freezes_local_signature_without_reindexing_callable() {
    let mut symbols = SymbolTable::default();
    let mut types = TypeRegistry::new();
    let i64_type = types.primitive_ids().i64_;
    let module = ModuleAst {
        items: vec![spanned(ItemAst::Function(FunctionAst {
            name: "identity".to_string(),
            params: vec![ParamAst {
                name: "value".to_string(),
                ty: TypeAst::Named("i64".to_string()),
            }],
            return_type: Some(TypeAst::Named("i64".to_string())),
            body: BlockAst {
                statements: vec![spanned(StmtAst::Return(Some(
                    logicodex::hir::ExprAst::Variable("value".to_string()),
                )))],
            },
            is_unsafe: false,
            is_public: false,
        }))],
    };

    let mut lowering = LoweringContext {
        symbols: &mut symbols,
        types: &mut types,
        diagnostics: Vec::new(),
        current_module: String::new(),
        current_return_enum: None,
    };
    let artifact = lowering
        .lower_module_with_metadata(module)
        .expect("lowering should produce a complete artifact");
    drop(lowering);

    let callable = symbols
        .lookup_callable("identity")
        .expect("lowering must preserve the original callable identity");
    let signature = artifact
        .callable_signatures
        .get(&callable)
        .expect("the lowered artifact must freeze the local signature");

    assert_eq!(artifact.module.items.len(), 1);
    assert_eq!(signature.callable, callable);
    assert_eq!(signature.name, "identity");
    assert_eq!(signature.params, vec![i64_type]);
    assert_eq!(signature.return_type, i64_type);
}

#[test]
fn lower_module_preserves_every_declaration_in_one_extern_block() {
    let mut symbols = SymbolTable::default();
    let mut types = TypeRegistry::new();
    let module = ModuleAst {
        items: vec![spanned(ItemAst::ExternBlock(ExternBlockAst {
            abi: logicodex::ffi::CallingConvention::C,
            functions: vec![
                ExternFnAst {
                    name: "first_external".to_string(),
                    params: Vec::new(),
                    return_type: TypeAst::Unit,
                    is_variadic: false,
                },
                ExternFnAst {
                    name: "second_external".to_string(),
                    params: Vec::new(),
                    return_type: TypeAst::Unit,
                    is_variadic: false,
                },
            ],
        }))],
    };

    let mut lowering = LoweringContext {
        symbols: &mut symbols,
        types: &mut types,
        diagnostics: Vec::new(),
        current_module: String::new(),
        current_return_enum: None,
    };
    let hir = lowering
        .lower_module(module)
        .expect("extern block lowering should succeed");

    let names: Vec<&str> = hir
        .items
        .iter()
        .filter_map(|item| match &item.node {
            HirItem::ExternFunction(function) => Some(function.name.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(names, vec!["first_external", "second_external"]);
}

#[test]
fn lower_program_artifact_records_complete_builtin_policies() {
    let mut symbols = SymbolTable::default();
    let mut types = TypeRegistry::new();
    let ids = types.primitive_ids();
    let mut lowering = LoweringContext {
        symbols: &mut symbols,
        types: &mut types,
        diagnostics: Vec::new(),
        current_module: String::new(),
        current_return_enum: None,
    };
    let artifact = lowering
        .lower_program_with_metadata(logicodex::ast::Program::new(Vec::new()))
        .expect("builtin metadata lowering should succeed");
    drop(lowering);

    let expectations = [
        ("print", Vec::new(), true),
        ("logicodex_sleep", vec![ids.i64_], false),
        ("logicodex_yield", Vec::new(), false),
    ];
    for (name, params, is_variadic) in expectations {
        let callable = symbols
            .lookup_callable(name)
            .unwrap_or_else(|| panic!("missing builtin callable {name}"));
        let signature = artifact
            .callable_signatures
            .get(&callable)
            .unwrap_or_else(|| panic!("missing builtin signature {name}"));
        assert_eq!(signature.callable, callable);
        assert_eq!(signature.name, name);
        assert_eq!(signature.kind, HirCallableKind::Builtin);
        assert_eq!(signature.params, params);
        assert_eq!(signature.return_type, ids.unit);
        assert_eq!(signature.is_variadic, is_variadic);
        assert_eq!(signature.value_use, HirCallableValueUse::StatementOnly);
    }
}

#[test]
fn variadic_builtin_metadata_does_not_activate_legacy_exact_arity() {
    let mut symbols = SymbolTable::default();
    let mut types = TypeRegistry::new();
    let mut lowering = LoweringContext {
        symbols: &mut symbols,
        types: &mut types,
        diagnostics: Vec::new(),
        current_module: String::new(),
        current_return_enum: None,
    };
    let artifact = lowering
        .lower_program_with_metadata(logicodex::ast::Program::new(Vec::new()))
        .expect("builtin metadata lowering should succeed");
    drop(lowering);

    let print = symbols.lookup_callable("print").expect("print callable");
    let signature = artifact
        .callable_signatures
        .get(&print)
        .expect("print signature must be frozen");
    assert!(signature.is_variadic);
    assert!(signature.params.is_empty());
    assert_eq!(
        symbols.callable_params(print),
        None,
        "PR 1.1 must not expose variadic builtin metadata as legacy exact arity"
    );
}

#[test]
fn lowered_artifact_records_constructor_and_user_extern_policies() {
    let mut symbols = SymbolTable::default();
    let mut types = TypeRegistry::new();
    let ids = types.primitive_ids();
    let module = ModuleAst {
        items: vec![
            spanned(ItemAst::Struct(StructDeclAst {
                name: "Pair".to_string(),
                fields: vec![
                    FieldAst {
                        name: "number".to_string(),
                        ty: TypeAst::Named("i64".to_string()),
                        visibility: VisibilityAst::Private,
                    },
                    FieldAst {
                        name: "flag".to_string(),
                        ty: TypeAst::Named("bool".to_string()),
                        visibility: VisibilityAst::Private,
                    },
                ],
                attributes: Vec::new(),
            })),
            spanned(ItemAst::ExternBlock(ExternBlockAst {
                abi: logicodex::ffi::CallingConvention::C,
                functions: vec![ExternFnAst {
                    name: "external_sum".to_string(),
                    params: vec![ParamAst {
                        name: "value".to_string(),
                        ty: TypeAst::Named("i64".to_string()),
                    }],
                    return_type: TypeAst::Named("i64".to_string()),
                    is_variadic: true,
                }],
            })),
        ],
    };

    let mut lowering = LoweringContext {
        symbols: &mut symbols,
        types: &mut types,
        diagnostics: Vec::new(),
        current_module: String::new(),
        current_return_enum: None,
    };
    let artifact = lowering
        .lower_module_with_metadata(module)
        .expect("constructor and extern metadata lowering should succeed");
    drop(lowering);

    let pair_layout = types
        .find_struct_by_name("Pair")
        .expect("Pair layout must be interned")
        .0;
    let pair_callable = symbols.lookup_callable("Pair").expect("Pair callable");
    let pair = artifact
        .callable_signatures
        .get(&pair_callable)
        .expect("constructor signature must be frozen");
    assert_eq!(pair.kind, HirCallableKind::Constructor);
    assert_eq!(pair.params, vec![ids.i64_, ids.bool_]);
    assert_eq!(pair.param_enums, vec![None, None]);
    assert_eq!(
        types.get(pair.return_type),
        Some(&TypeKind::Struct(pair_layout))
    );
    assert_eq!(pair.return_enum, None);
    assert!(!pair.is_variadic);
    assert_eq!(pair.value_use, HirCallableValueUse::Value);

    let extern_callable = symbols
        .lookup_callable("external_sum")
        .expect("extern callable");
    let external = artifact
        .callable_signatures
        .get(&extern_callable)
        .expect("user extern signature must be frozen");
    assert_eq!(external.kind, HirCallableKind::UserExtern);
    assert_eq!(external.params, vec![ids.i64_]);
    assert_eq!(external.param_enums, vec![None]);
    assert_eq!(external.return_type, ids.i64_);
    assert_eq!(external.return_enum, None);
    assert!(external.is_variadic);
    assert_eq!(external.value_use, HirCallableValueUse::Value);
}

#[test]
fn imported_module_artifact_preserves_mangled_signature_identity() {
    let mut symbols = SymbolTable::default();
    let mut types = TypeRegistry::new();
    let ids = types.primitive_ids();
    let module = ModuleAst {
        items: vec![spanned(ItemAst::Function(FunctionAst {
            name: "double".to_string(),
            params: vec![ParamAst {
                name: "value".to_string(),
                ty: TypeAst::Named("i64".to_string()),
            }],
            return_type: Some(TypeAst::Named("i64".to_string())),
            body: BlockAst {
                statements: vec![spanned(StmtAst::Return(Some(
                    logicodex::hir::ExprAst::Variable("value".to_string()),
                )))],
            },
            is_unsafe: false,
            is_public: true,
        }))],
    };

    let mut lowering = LoweringContext {
        symbols: &mut symbols,
        types: &mut types,
        diagnostics: Vec::new(),
        current_module: String::new(),
        current_return_enum: None,
    };
    let artifact = lowering
        .lower_module_as_with_metadata("math", module)
        .expect("imported module metadata lowering should succeed");
    drop(lowering);

    let mangled = logicodex::module_loader::mangle_symbol("math", "double");
    let callable = symbols
        .lookup_callable(&mangled)
        .expect("mangled imported callable must keep its lowering identity");
    let signature = artifact
        .callable_signatures
        .get(&callable)
        .expect("mangled imported signature must be frozen");
    assert_eq!(signature.callable, callable);
    assert_eq!(signature.name, mangled);
    assert_eq!(signature.kind, HirCallableKind::Function);
    assert_eq!(signature.params, vec![ids.i64_]);
    assert_eq!(signature.return_type, ids.i64_);
    assert!(matches!(
        &artifact.module.items[0].node,
        HirItem::Function(function) if function.name == signature.name
    ));
}

#[test]
fn lowered_artifact_rejects_incomplete_internal_callable_metadata() {
    let mut symbols = SymbolTable::default();
    symbols.define_callable("orphan_internal_callable");
    let mut types = TypeRegistry::new();
    let mut lowering = LoweringContext {
        symbols: &mut symbols,
        types: &mut types,
        diagnostics: Vec::new(),
        current_module: String::new(),
        current_return_enum: None,
    };

    let result = lowering.lower_module_with_metadata(ModuleAst { items: Vec::new() });
    assert!(
        result.is_err(),
        "incomplete internal callable metadata must fail closed instead of being omitted"
    );
    let diagnostics = result.expect_err("incomplete metadata must produce a diagnostic");
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message_en
            .contains("incomplete internal callable metadata")
    }));
}

#[test]
fn forward_and_recursive_calls_use_the_frozen_callable_identities() {
    fn call(name: &str) -> logicodex::hir::ExprAst {
        logicodex::hir::ExprAst::Call {
            callee: Box::new(logicodex::hir::ExprAst::Variable(name.to_string())),
            args: Vec::new(),
        }
    }

    let function = |name: &str, target: &str| {
        spanned(ItemAst::Function(FunctionAst {
            name: name.to_string(),
            params: Vec::new(),
            return_type: Some(TypeAst::Named("i64".to_string())),
            body: BlockAst {
                statements: vec![spanned(StmtAst::Return(Some(call(target))))],
            },
            is_unsafe: false,
            is_public: false,
        }))
    };
    let module = ModuleAst {
        items: vec![
            function("forward_caller", "recursive_target"),
            function("recursive_target", "recursive_target"),
        ],
    };
    let mut symbols = SymbolTable::default();
    let mut types = TypeRegistry::new();
    let mut lowering = LoweringContext {
        symbols: &mut symbols,
        types: &mut types,
        diagnostics: Vec::new(),
        current_module: String::new(),
        current_return_enum: None,
    };
    let artifact = lowering
        .lower_module_with_metadata(module)
        .expect("forward and recursive calls should lower from the predeclared table");
    drop(lowering);

    let caller = symbols
        .lookup_callable("forward_caller")
        .expect("caller id");
    let target = symbols
        .lookup_callable("recursive_target")
        .expect("target id");
    assert!(artifact.callable_signatures.contains_key(&caller));
    assert!(artifact.callable_signatures.contains_key(&target));

    let lowered_call = |index: usize| match &artifact.module.items[index].node {
        HirItem::Function(function) => match &function.body.statements[0].node {
            logicodex::hir::HirStmt::Return(Some(expr)) => match expr.kind {
                logicodex::hir::HirExprKind::Call { callee, .. } => callee,
                ref other => panic!("expected lowered call, got {other:?}"),
            },
            other => panic!("expected return statement, got {other:?}"),
        },
        other => panic!("expected function item, got {other:?}"),
    };
    assert_eq!(lowered_call(0), target);
    assert_eq!(lowered_call(1), target);
}

#[test]
fn legacy_lowering_entry_points_still_return_plain_hir_modules() {
    let mut symbols = SymbolTable::default();
    let mut types = TypeRegistry::new();
    let mut lowering = LoweringContext {
        symbols: &mut symbols,
        types: &mut types,
        diagnostics: Vec::new(),
        current_module: String::new(),
        current_return_enum: None,
    };
    let module: logicodex::hir::HirModule = lowering
        .lower_module(ModuleAst { items: Vec::new() })
        .expect("legacy lower_module must remain compatible");
    assert!(module.items.is_empty());

    let mut program_symbols = SymbolTable::default();
    let mut program_types = TypeRegistry::new();
    let mut program_lowering = LoweringContext {
        symbols: &mut program_symbols,
        types: &mut program_types,
        diagnostics: Vec::new(),
        current_module: String::new(),
        current_return_enum: None,
    };
    let program_module: logicodex::hir::HirModule = program_lowering
        .lower_program(logicodex::ast::Program::new(Vec::new()))
        .expect("legacy lower_program must remain compatible");
    assert!(program_module.items.is_empty());
}

#[test]
fn language_artifact_and_ffi_registry_keep_separate_identity_spaces() {
    let mut types = TypeRegistry::new();
    let ids = types.primitive_ids();
    let mut ffi_registry = logicodex::ffi::CallableRegistry::default();
    let ffi_id = ffi_registry.register(logicodex::ffi::CallableSignature {
        name: "ffi_only".to_string(),
        params: Vec::new(),
        return_type: ids.i64_,
        abi: logicodex::ffi::CallingConvention::C,
        safety: logicodex::ffi::CallableSafety::Safe,
        is_extern: true,
        is_variadic: false,
    });

    let mut symbols = SymbolTable::default();
    let module = ModuleAst {
        items: vec![spanned(ItemAst::Function(FunctionAst {
            name: "language_only".to_string(),
            params: Vec::new(),
            return_type: Some(TypeAst::Named("i64".to_string())),
            body: BlockAst {
                statements: vec![spanned(StmtAst::Return(Some(
                    logicodex::hir::ExprAst::Literal(logicodex::hir::LiteralAst::Integer(1)),
                )))],
            },
            is_unsafe: false,
            is_public: false,
        }))],
    };
    let mut lowering = LoweringContext {
        symbols: &mut symbols,
        types: &mut types,
        diagnostics: Vec::new(),
        current_module: String::new(),
        current_return_enum: None,
    };
    let artifact = lowering
        .lower_module_with_metadata(module)
        .expect("language callable lowering should succeed");
    drop(lowering);
    let language_id = symbols
        .lookup_callable("language_only")
        .expect("language callable id");

    assert_eq!(
        ffi_id, language_id,
        "independent allocators may overlap numerically"
    );
    assert_eq!(ffi_registry.get(ffi_id).unwrap().name, "ffi_only");
    assert_eq!(
        artifact.callable_signatures.get(&language_id).unwrap().name,
        "language_only"
    );
    assert!(ffi_registry.find_by_name("language_only").is_none());
    assert!(artifact
        .callable_signatures
        .values()
        .all(|signature| signature.name != "ffi_only"));
}
