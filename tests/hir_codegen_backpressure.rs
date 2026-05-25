// =========================================================================
// Logicodex v1.36.0-alpha — A4: Backpressure + Scheduler Codegen Tests
//
// Tests: HIR ChannelTrySend, ChannelTryRecv, Yield, Sleep,
//        ChannelTimeoutRecv → LLVM IR
// =========================================================================

use logicodex::codegen::{compile_v130, CodegenOptions};
use logicodex::ffi::{CallableRegistry, SafetyContext};
use logicodex::hir::{
    HirBlock, HirExpr, HirExprKind, HirFunction, HirItem, HirModule, HirStmt,
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

// ─── 1. ChannelTrySend codegen ───

#[test]
fn hir_codegen_channel_try_send() {
    let types = TypeRegistry::new();
    let unit_ty = unit_ref(&types);
    let i64_ty = i64_ref(&types);

    let module = HirModule {
        items: vec![spanned(HirItem::Function(HirFunction {
            name: "test_try_send".to_string(),
            symbol: SymbolId(0),
            params: Vec::new(),
            return_type: unit_ty,
            body: HirBlock {
                statements: vec![spanned(HirStmt::Expr(HirExpr {
                    kind: HirExprKind::ChannelTrySend {
                        channel_name: "bp_ch".to_string(),
                        value: Box::new(HirExpr {
                            kind: HirExprKind::Literal(logicodex::hir::LiteralAst::Integer(100)),
                            ty: i64_ty,
                            span: Span::unknown(),
                        }),
                    },
                    ty: i64_ty, // returns bool as i64
                    span: Span::unknown(),
                }))],
            },
            safety: SafetyContext::Safe,
        }))],
    };

    let callables = CallableRegistry::default();
    let out_path = Path::new("/tmp/logicodex_hir_test_trysend.o");
    let result = compile_v130(
        &module, out_path,
        &CodegenOptions { module_name: "test".to_string(), emit_ir: true, secure: false, target: CompilationTarget::Native },
        callables, types,
    );
    assert!(result.is_ok(), "HIR try_send should compile: {:?}", result.err());
}

// ─── 2. ChannelTryRecv codegen ───

#[test]
fn hir_codegen_channel_try_recv() {
    let types = TypeRegistry::new();
    let unit_ty = unit_ref(&types);
    let i64_ty = i64_ref(&types);

    let module = HirModule {
        items: vec![spanned(HirItem::Function(HirFunction {
            name: "test_try_recv".to_string(),
            symbol: SymbolId(0),
            params: Vec::new(),
            return_type: unit_ty,
            body: HirBlock {
                statements: vec![spanned(HirStmt::Expr(HirExpr {
                    kind: HirExprKind::ChannelTryRecv {
                        channel_name: "bp_ch".to_string(),
                    },
                    ty: i64_ty,
                    span: Span::unknown(),
                }))],
            },
            safety: SafetyContext::Safe,
        }))],
    };

    let callables = CallableRegistry::default();
    let out_path = Path::new("/tmp/logicodex_hir_test_tryrecv.o");
    let result = compile_v130(
        &module, out_path,
        &CodegenOptions { module_name: "test".to_string(), emit_ir: true, secure: false, target: CompilationTarget::Native },
        callables, types,
    );
    assert!(result.is_ok(), "HIR try_recv should compile: {:?}", result.err());
}

// ─── 3. Yield codegen ───

#[test]
fn hir_codegen_yield() {
    let types = TypeRegistry::new();
    let unit_ty = unit_ref(&types);

    let module = HirModule {
        items: vec![spanned(HirItem::Function(HirFunction {
            name: "test_yield".to_string(),
            symbol: SymbolId(0),
            params: Vec::new(),
            return_type: unit_ty,
            body: HirBlock {
                statements: vec![spanned(HirStmt::Expr(HirExpr {
                    kind: HirExprKind::Yield,
                    ty: unit_ty,
                    span: Span::unknown(),
                }))],
            },
            safety: SafetyContext::Safe,
        }))],
    };

    let callables = CallableRegistry::default();
    let out_path = Path::new("/tmp/logicodex_hir_test_yield.o");
    let result = compile_v130(
        &module, out_path,
        &CodegenOptions { module_name: "test".to_string(), emit_ir: true, secure: false, target: CompilationTarget::Native },
        callables, types,
    );
    assert!(result.is_ok(), "HIR yield should compile: {:?}", result.err());
}

// ─── 4. Sleep codegen ───

#[test]
fn hir_codegen_sleep() {
    let types = TypeRegistry::new();
    let unit_ty = unit_ref(&types);
    let i64_ty = i64_ref(&types);

    let module = HirModule {
        items: vec![spanned(HirItem::Function(HirFunction {
            name: "test_sleep".to_string(),
            symbol: SymbolId(0),
            params: Vec::new(),
            return_type: unit_ty,
            body: HirBlock {
                statements: vec![spanned(HirStmt::Expr(HirExpr {
                    kind: HirExprKind::Sleep {
                        duration_ms: Box::new(HirExpr {
                            kind: HirExprKind::Literal(logicodex::hir::LiteralAst::Integer(1000)),
                            ty: i64_ty,
                            span: Span::unknown(),
                        }),
                    },
                    ty: unit_ty,
                    span: Span::unknown(),
                }))],
            },
            safety: SafetyContext::Safe,
        }))],
    };

    let callables = CallableRegistry::default();
    let out_path = Path::new("/tmp/logicodex_hir_test_sleep.o");
    let result = compile_v130(
        &module, out_path,
        &CodegenOptions { module_name: "test".to_string(), emit_ir: true, secure: false, target: CompilationTarget::Native },
        callables, types,
    );
    assert!(result.is_ok(), "HIR sleep should compile: {:?}", result.err());
}

// ─── 5. TimeoutRecv codegen ───

#[test]
fn hir_codegen_timeout_recv() {
    let types = TypeRegistry::new();
    let unit_ty = unit_ref(&types);
    let i64_ty = i64_ref(&types);

    let module = HirModule {
        items: vec![spanned(HirItem::Function(HirFunction {
            name: "test_timeout_recv".to_string(),
            symbol: SymbolId(0),
            params: Vec::new(),
            return_type: unit_ty,
            body: HirBlock {
                statements: vec![spanned(HirStmt::Expr(HirExpr {
                    kind: HirExprKind::ChannelTimeoutRecv {
                        channel_name: "bp_ch".to_string(),
                        timeout_ms: Box::new(HirExpr {
                            kind: HirExprKind::Literal(logicodex::hir::LiteralAst::Integer(5000)),
                            ty: i64_ty,
                            span: Span::unknown(),
                        }),
                    },
                    ty: i64_ty,
                    span: Span::unknown(),
                }))],
            },
            safety: SafetyContext::Safe,
        }))],
    };

    let callables = CallableRegistry::default();
    let out_path = Path::new("/tmp/logicodex_hir_test_torecv.o");
    let result = compile_v130(
        &module, out_path,
        &CodegenOptions { module_name: "test".to_string(), emit_ir: true, secure: false, target: CompilationTarget::Native },
        callables, types,
    );
    assert!(result.is_ok(), "HIR timeout_recv should compile: {:?}", result.err());
}

// ─── 6. Full backpressure workflow ───

#[test]
fn hir_codegen_full_backpressure_workflow() {
    let types = TypeRegistry::new();
    let unit_ty = unit_ref(&types);
    let i64_ty = i64_ref(&types);
    let local_0 = LocalId(0);
    let local_1 = LocalId(1);

    let module = HirModule {
        items: vec![spanned(HirItem::Function(HirFunction {
            name: "backpressure_demo".to_string(),
            symbol: SymbolId(0),
            params: Vec::new(),
            return_type: unit_ty,
            body: HirBlock {
                statements: vec![
                    // let ok = ch.try_send(42)
                    spanned(HirStmt::Let {
                        local: local_0,
                        ty: i64_ty,
                        value: Some(HirExpr {
                            kind: HirExprKind::ChannelTrySend {
                                channel_name: "ch".to_string(),
                                value: Box::new(HirExpr {
                                    kind: HirExprKind::Literal(logicodex::hir::LiteralAst::Integer(42)),
                                    ty: i64_ty,
                                    span: Span::unknown(),
                                }),
                            },
                            ty: i64_ty,
                            span: Span::unknown(),
                        }),
                    }),
                    // yield
                    spanned(HirStmt::Expr(HirExpr {
                        kind: HirExprKind::Yield,
                        ty: unit_ty,
                        span: Span::unknown(),
                    })),
                    // sleep(10)
                    spanned(HirStmt::Expr(HirExpr {
                        kind: HirExprKind::Sleep {
                            duration_ms: Box::new(HirExpr {
                                kind: HirExprKind::Literal(logicodex::hir::LiteralAst::Integer(10)),
                                ty: i64_ty,
                                span: Span::unknown(),
                            }),
                        },
                        ty: unit_ty,
                        span: Span::unknown(),
                    })),
                    // let msg = ch.try_recv()
                    spanned(HirStmt::Let {
                        local: local_1,
                        ty: i64_ty,
                        value: Some(HirExpr {
                            kind: HirExprKind::ChannelTryRecv {
                                channel_name: "ch".to_string(),
                            },
                            ty: i64_ty,
                            span: Span::unknown(),
                        }),
                    }),
                    // ch.timeout_recv(5000)
                    spanned(HirStmt::Expr(HirExpr {
                        kind: HirExprKind::ChannelTimeoutRecv {
                            channel_name: "ch".to_string(),
                            timeout_ms: Box::new(HirExpr {
                                kind: HirExprKind::Literal(logicodex::hir::LiteralAst::Integer(5000)),
                                ty: i64_ty,
                                span: Span::unknown(),
                            }),
                        },
                        ty: i64_ty,
                        span: Span::unknown(),
                    })),
                ],
            },
            safety: SafetyContext::Safe,
        }))],
    };

    let callables = CallableRegistry::default();
    let out_path = Path::new("/tmp/logicodex_hir_test_bp_workflow.o");
    let result = compile_v130(
        &module, out_path,
        &CodegenOptions { module_name: "test".to_string(), emit_ir: true, secure: false, target: CompilationTarget::Native },
        callables, types,
    );
    assert!(result.is_ok(), "HIR backpressure workflow should compile: {:?}", result.err());
}
