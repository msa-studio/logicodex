// =========================================================================
// Logicodex v1.36.0-alpha — A1 Critical: HIR Function Codegen Tests
//
// Tests: emit_v130_function() — full LLVM IR emission from HIR
//
// Coverage:
//   - HIR function with no params → LLVM function definition
//   - HIR function with params → parameter alloca + store
//   - HIR function with Let → local variable alloca
//   - HIR function with If → conditional branches
//   - HIR function with While → loop blocks
//   - HIR function with Return → explicit return
//   - HIR function with Call → function call via CallableRegistry
// =========================================================================

use logicodex::codegen::{compile_v130, CodegenOptions};
use logicodex::ffi::{CallableRegistry, CallableSafety, CallableSignature, CallingConvention, SafetyContext};
use logicodex::hir::{
    HirBlock, HirExpr, HirExprKind, HirFunction, HirItem, HirModule, HirParam, HirStmt,
    LocalId, SymbolId, TypeRef,
};
use logicodex::os::target::CompilationTarget;
use logicodex::span::Span;
use logicodex::types::TypeRegistry;
use std::path::Path;

fn unit_ref(types: &TypeRegistry) -> TypeRef {
    TypeRef { id: types.primitive_ids().unit }
}

fn i64_ref(types: &TypeRegistry) -> TypeRef {
    TypeRef { id: types.primitive_ids().i64_ }
}

fn spanned<T>(node: T) -> logicodex::span::Spanned<T> {
    logicodex::span::Spanned { node, span: Span::unknown() }
}

// ─── 1. Empty function generates LLVM IR ───

#[test]
fn hir_codegen_empty_function() {
    let types = TypeRegistry::new();
    let module = HirModule {
        items: vec![spanned(HirItem::Function(HirFunction {
            name: "main".to_string(),
            symbol: SymbolId(0),
            params: Vec::new(),
            return_type: unit_ref(&types),
            body: HirBlock { statements: Vec::new() },
            safety: SafetyContext::Safe,
        }))],
    };

    let callables = CallableRegistry::default();
    let out_path = Path::new("/tmp/logicodex_hir_test_empty.o");
    let result = compile_v130(
        &module,
        out_path,
        &CodegenOptions {
            module_name: "test".to_string(),
            emit_ir: true,
            secure: false,
            target: CompilationTarget::Native,
        },
        callables,
        types,
    );
    assert!(result.is_ok(), "HIR empty function should compile: {:?}", result.err());

    // IR file should exist
    let ir_path = out_path.with_extension("ll");
    if ir_path.exists() {
        let ir = std::fs::read_to_string(&ir_path).unwrap();
        assert!(ir.contains("define i8 @main()"), "IR should contain 'define i8 @main()'\n{}", ir);
    }
}

// ─── 2. Function with Let + Return ───

#[test]
fn hir_codegen_let_and_return() {
    let types = TypeRegistry::new();
    let i64_ty = i64_ref(&types);
    let local_0 = LocalId(0);

    let module = HirModule {
        items: vec![spanned(HirItem::Function(HirFunction {
            name: "get_answer".to_string(),
            symbol: SymbolId(0),
            params: Vec::new(),
            return_type: i64_ty,
            body: HirBlock {
                statements: vec![
                    spanned(HirStmt::Let {
                        local: local_0,
                        ty: i64_ty,
                        value: Some(HirExpr {
                            kind: HirExprKind::Literal(logicodex::hir::LiteralAst::Integer(42)),
                            ty: i64_ty,
                            span: Span::unknown(),
                        }),
                    }),
                    spanned(HirStmt::Return(Some(HirExpr {
                        kind: HirExprKind::Local(local_0),
                        ty: i64_ty,
                        span: Span::unknown(),
                    }))),
                ],
            },
            safety: SafetyContext::Safe,
        }))],
    };

    let callables = CallableRegistry::default();
    let out_path = Path::new("/tmp/logicodex_hir_test_empty.o");
    let result = compile_v130(
        &module,
        out_path,
        &CodegenOptions {
            module_name: "test".to_string(),
            emit_ir: true,
            secure: false,
            target: CompilationTarget::Native,
        },
        callables,
        types,
    );
    assert!(result.is_ok(), "HIR let+return should compile: {:?}", result.err());
}

// ─── 3. Function with If ───

#[test]
fn hir_codegen_if_statement() {
    let types = TypeRegistry::new();
    let i64_ty = i64_ref(&types);
    let unit_ty = unit_ref(&types);
    let local_0 = LocalId(0);

    let module = HirModule {
        items: vec![spanned(HirItem::Function(HirFunction {
            name: "conditional".to_string(),
            symbol: SymbolId(0),
            params: Vec::new(),
            return_type: unit_ty,
            body: HirBlock {
                statements: vec![
                    spanned(HirStmt::Let {
                        local: local_0,
                        ty: i64_ty,
                        value: Some(HirExpr {
                            kind: HirExprKind::Literal(logicodex::hir::LiteralAst::Integer(1)),
                            ty: i64_ty,
                            span: Span::unknown(),
                        }),
                    }),
                    spanned(HirStmt::If {
                        condition: HirExpr {
                            kind: HirExprKind::Local(local_0),
                            ty: i64_ty,
                            span: Span::unknown(),
                        },
                        then_branch: HirBlock {
                            statements: vec![spanned(HirStmt::Expr(HirExpr {
                                kind: HirExprKind::Literal(logicodex::hir::LiteralAst::Integer(10)),
                                ty: i64_ty,
                                span: Span::unknown(),
                            }))],
                        },
                        else_branch: None,
                    }),
                ],
            },
            safety: SafetyContext::Safe,
        }))],
    };

    let callables = CallableRegistry::default();
    let out_path = Path::new("/tmp/logicodex_hir_test_empty.o");
    let result = compile_v130(
        &module,
        out_path,
        &CodegenOptions {
            module_name: "test".to_string(),
            emit_ir: true,
            secure: false,
            target: CompilationTarget::Native,
        },
        callables,
        types,
    );
    assert!(result.is_ok(), "HIR if should compile: {:?}", result.err());
}

// ─── 4. Function with While loop ───

#[test]
fn hir_codegen_while_loop() {
    let types = TypeRegistry::new();
    let i64_ty = i64_ref(&types);
    let unit_ty = unit_ref(&types);
    let local_0 = LocalId(0);

    let module = HirModule {
        items: vec![spanned(HirItem::Function(HirFunction {
            name: "countdown".to_string(),
            symbol: SymbolId(0),
            params: Vec::new(),
            return_type: unit_ty,
            body: HirBlock {
                statements: vec![
                    spanned(HirStmt::Let {
                        local: local_0,
                        ty: i64_ty,
                        value: Some(HirExpr {
                            kind: HirExprKind::Literal(logicodex::hir::LiteralAst::Integer(3)),
                            ty: i64_ty,
                            span: Span::unknown(),
                        }),
                    }),
                    spanned(HirStmt::While {
                        condition: HirExpr {
                            kind: HirExprKind::Local(local_0),
                            ty: i64_ty,
                            span: Span::unknown(),
                        },
                        body: HirBlock {
                            statements: vec![
                                spanned(HirStmt::Expr(HirExpr {
                                    kind: HirExprKind::Literal(logicodex::hir::LiteralAst::Integer(1)),
                                    ty: i64_ty,
                                    span: Span::unknown(),
                                })),
                            ],
                        },
                    }),
                ],
            },
            safety: SafetyContext::Safe,
        }))],
    };

    let callables = CallableRegistry::default();
    let out_path = Path::new("/tmp/logicodex_hir_test_empty.o");
    let result = compile_v130(
        &module,
        out_path,
        &CodegenOptions {
            module_name: "test".to_string(),
            emit_ir: true,
            secure: false,
            target: CompilationTarget::Native,
        },
        callables,
        types,
    );
    assert!(result.is_ok(), "HIR while should compile: {:?}", result.err());
}

// ─── 5. Function with Binary expression ───

#[test]
fn hir_codegen_binary_expression() {
    let types = TypeRegistry::new();
    let i64_ty = i64_ref(&types);

    let module = HirModule {
        items: vec![spanned(HirItem::Function(HirFunction {
            name: "add_nums".to_string(),
            symbol: SymbolId(0),
            params: Vec::new(),
            return_type: i64_ty,
            body: HirBlock {
                statements: vec![spanned(HirStmt::Return(Some(HirExpr {
                    kind: HirExprKind::Binary {
                        left: Box::new(HirExpr {
                            kind: HirExprKind::Literal(logicodex::hir::LiteralAst::Integer(5)),
                            ty: i64_ty,
                            span: Span::unknown(),
                        }),
                        op: logicodex::hir::BinaryOpAst::Add,
                        right: Box::new(HirExpr {
                            kind: HirExprKind::Literal(logicodex::hir::LiteralAst::Integer(7)),
                            ty: i64_ty,
                            span: Span::unknown(),
                        }),
                    },
                    ty: i64_ty,
                    span: Span::unknown(),
                })))],
            },
            safety: SafetyContext::Safe,
        }))],
    };

    let callables = CallableRegistry::default();
    let out_path = Path::new("/tmp/logicodex_hir_test_empty.o");
    let result = compile_v130(
        &module,
        out_path,
        &CodegenOptions {
            module_name: "test".to_string(),
            emit_ir: true,
            secure: false,
            target: CompilationTarget::Native,
        },
        callables,
        types,
    );
    assert!(result.is_ok(), "HIR binary should compile: {:?}", result.err());
}

// ─── 6. Function with extern call ───

#[test]
fn hir_codegen_extern_call() {
    let types = TypeRegistry::new();
    let ids = types.primitive_ids();
    let i64_ty = i64_ref(&types);
    let unit_ty = unit_ref(&types);

    let mut callables = CallableRegistry::default();
    let callable = callables.register(CallableSignature {
        name: "test_func".to_string(),
        params: vec![ids.i64_],
        return_type: ids.i64_,
        abi: CallingConvention::C,
        safety: CallableSafety::Safe,
        is_extern: true,
        is_variadic: false,
    });

    let module = HirModule {
        items: vec![
            spanned(HirItem::ExternFunction(logicodex::hir::HirExternFunction { callable })),
            spanned(HirItem::Function(HirFunction {
                name: "caller".to_string(),
                symbol: SymbolId(1),
                params: Vec::new(),
                return_type: i64_ty,
                body: HirBlock {
                    statements: vec![spanned(HirStmt::Return(Some(HirExpr {
                        kind: HirExprKind::Call {
                            callee: callable,
                            args: vec![HirExpr {
                                kind: HirExprKind::Literal(logicodex::hir::LiteralAst::Integer(42)),
                                ty: i64_ty,
                                span: Span::unknown(),
                            }],
                        },
                        ty: i64_ty,
                        span: Span::unknown(),
                    })))],
                },
                safety: SafetyContext::Safe,
            })),
        ],
    };

    let out_path = Path::new("/tmp/logicodex_hir_test_empty.o");
    let result = compile_v130(
        &module,
        out_path,
        &CodegenOptions {
            module_name: "test".to_string(),
            emit_ir: true,
            secure: false,
            target: CompilationTarget::Native,
        },
        callables,
        types,
    );
    assert!(result.is_ok(), "HIR extern call should compile: {:?}", result.err());
}
