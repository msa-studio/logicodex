// =========================================================================
// Logicodex v1.36.0-alpha — A3: Threading Expressions Codegen Tests
//
// Tests: HIR Spawn, Join, ChannelSend, ChannelRecv → LLVM IR
//
// Coverage:
//   - Spawn actor → logicodex_spawn() call
//   - Join actor → logicodex_join() call
//   - ChannelSend → logicodex_channel_send() call
//   - ChannelRecv → logicodex_channel_recv() call
//   - Threading within function body with locals
// =========================================================================

use logicodex::codegen::{compile_v130, CodegenOptions};
use logicodex::ffi::{CallableRegistry, SafetyContext};
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

// ─── 1. Spawn actor codegen ───

#[test]
fn hir_codegen_spawn() {
    let types = TypeRegistry::new();
    let unit_ty = unit_ref(&types);

    let module = HirModule {
        items: vec![spanned(HirItem::Function(HirFunction {
            name: "main".to_string(),
            symbol: SymbolId(0),
            params: Vec::new(),
            return_type: unit_ty,
            body: HirBlock {
                statements: vec![spanned(HirStmt::Expr(HirExpr {
                    kind: HirExprKind::Spawn {
                        actor_name: "Worker".to_string(),
                        args: vec![],
                    },
                    ty: unit_ty,
                    span: Span::unknown(),
                }))],
            },
            safety: SafetyContext::Safe,
        }))],
    };

    let callables = CallableRegistry::default();
    let out_path = Path::new("/tmp/logicodex_hir_test_spawn.o");
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
    assert!(result.is_ok(), "HIR spawn should compile: {:?}", result.err());

    let ir_path = out_path.with_extension("ll");
    if ir_path.exists() {
        let ir = std::fs::read_to_string(&ir_path).unwrap();
        assert!(ir.contains("logicodex_spawn"), "IR should declare logicodex_spawn\n{}", ir);
        assert!(ir.contains("Worker"), "IR should reference actor name 'Worker'\n{}", ir);
    }
}

// ─── 2. Join actor codegen ───

#[test]
fn hir_codegen_join() {
    let types = TypeRegistry::new();
    let unit_ty = unit_ref(&types);

    let module = HirModule {
        items: vec![spanned(HirItem::Function(HirFunction {
            name: "main".to_string(),
            symbol: SymbolId(0),
            params: Vec::new(),
            return_type: unit_ty,
            body: HirBlock {
                statements: vec![spanned(HirStmt::Expr(HirExpr {
                    kind: HirExprKind::Join {
                        actor_name: "Worker".to_string(),
                    },
                    ty: unit_ty,
                    span: Span::unknown(),
                }))],
            },
            safety: SafetyContext::Safe,
        }))],
    };

    let callables = CallableRegistry::default();
    let out_path = Path::new("/tmp/logicodex_hir_test_join.o");
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
    assert!(result.is_ok(), "HIR join should compile: {:?}", result.err());

    let ir_path = out_path.with_extension("ll");
    if ir_path.exists() {
        let ir = std::fs::read_to_string(&ir_path).unwrap();
        assert!(ir.contains("logicodex_join"), "IR should declare logicodex_join\n{}", ir);
    }
}

// ─── 3. Channel send codegen ───

#[test]
fn hir_codegen_channel_send() {
    let types = TypeRegistry::new();
    let unit_ty = unit_ref(&types);
    let i64_ty = i64_ref(&types);
    let local_0 = LocalId(0);

    let module = HirModule {
        items: vec![spanned(HirItem::Function(HirFunction {
            name: "main".to_string(),
            symbol: SymbolId(0),
            params: Vec::new(),
            return_type: unit_ty,
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
                    spanned(HirStmt::Expr(HirExpr {
                        kind: HirExprKind::ChannelSend {
                            channel_name: "data_ch".to_string(),
                            value: Box::new(HirExpr {
                                kind: HirExprKind::Local(local_0),
                                ty: i64_ty,
                                span: Span::unknown(),
                            }),
                        },
                        ty: unit_ty,
                        span: Span::unknown(),
                    })),
                ],
            },
            safety: SafetyContext::Safe,
        }))],
    };

    let callables = CallableRegistry::default();
    let out_path = Path::new("/tmp/logicodex_hir_test_send.o");
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
    assert!(result.is_ok(), "HIR channel send should compile: {:?}", result.err());

    let ir_path = out_path.with_extension("ll");
    if ir_path.exists() {
        let ir = std::fs::read_to_string(&ir_path).unwrap();
        assert!(ir.contains("logicodex_channel_send"), "IR should declare logicodex_channel_send\n{}", ir);
        assert!(ir.contains("data_ch"), "IR should reference channel name 'data_ch'\n{}", ir);
    }
}

// ─── 4. Channel recv codegen ───

#[test]
fn hir_codegen_channel_recv() {
    let types = TypeRegistry::new();
    let unit_ty = unit_ref(&types);

    let module = HirModule {
        items: vec![spanned(HirItem::Function(HirFunction {
            name: "main".to_string(),
            symbol: SymbolId(0),
            params: Vec::new(),
            return_type: unit_ty,
            body: HirBlock {
                statements: vec![spanned(HirStmt::Expr(HirExpr {
                    kind: HirExprKind::ChannelRecv {
                        channel_name: "data_ch".to_string(),
                    },
                    ty: unit_ty,
                    span: Span::unknown(),
                }))],
            },
            safety: SafetyContext::Safe,
        }))],
    };

    let callables = CallableRegistry::default();
    let out_path = Path::new("/tmp/logicodex_hir_test_recv.o");
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
    assert!(result.is_ok(), "HIR channel recv should compile: {:?}", result.err());

    let ir_path = out_path.with_extension("ll");
    if ir_path.exists() {
        let ir = std::fs::read_to_string(&ir_path).unwrap();
        assert!(ir.contains("logicodex_channel_recv"), "IR should declare logicodex_channel_recv\n{}", ir);
    }
}

// ─── 5. Full actor workflow: spawn + send + recv + join ───

#[test]
fn hir_codegen_full_actor_workflow() {
    let types = TypeRegistry::new();
    let unit_ty = unit_ref(&types);
    let i64_ty = i64_ref(&types);
    let local_0 = LocalId(0);

    let module = HirModule {
        items: vec![spanned(HirItem::Function(HirFunction {
            name: "orchestrate".to_string(),
            symbol: SymbolId(0),
            params: Vec::new(),
            return_type: unit_ty,
            body: HirBlock {
                statements: vec![
                    // spawn Worker
                    spanned(HirStmt::Expr(HirExpr {
                        kind: HirExprKind::Spawn {
                            actor_name: "Worker".to_string(),
                            args: vec![],
                        },
                        ty: unit_ty,
                        span: Span::unknown(),
                    })),
                    // let msg = 42
                    spanned(HirStmt::Let {
                        local: local_0,
                        ty: i64_ty,
                        value: Some(HirExpr {
                            kind: HirExprKind::Literal(logicodex::hir::LiteralAst::Integer(42)),
                            ty: i64_ty,
                            span: Span::unknown(),
                        }),
                    }),
                    // send msg through "ch"
                    spanned(HirStmt::Expr(HirExpr {
                        kind: HirExprKind::ChannelSend {
                            channel_name: "ch".to_string(),
                            value: Box::new(HirExpr {
                                kind: HirExprKind::Local(local_0),
                                ty: i64_ty,
                                span: Span::unknown(),
                            }),
                        },
                        ty: unit_ty,
                        span: Span::unknown(),
                    })),
                    // recv from "ch"
                    spanned(HirStmt::Expr(HirExpr {
                        kind: HirExprKind::ChannelRecv {
                            channel_name: "ch".to_string(),
                        },
                        ty: unit_ty,
                        span: Span::unknown(),
                    })),
                    // join Worker
                    spanned(HirStmt::Expr(HirExpr {
                        kind: HirExprKind::Join {
                            actor_name: "Worker".to_string(),
                        },
                        ty: unit_ty,
                        span: Span::unknown(),
                    })),
                ],
            },
            safety: SafetyContext::Safe,
        }))],
    };

    let callables = CallableRegistry::default();
    let out_path = Path::new("/tmp/logicodex_hir_test_workflow.o");
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
    assert!(result.is_ok(), "HIR full workflow should compile: {:?}", result.err());

    let ir_path = out_path.with_extension("ll");
    if ir_path.exists() {
        let ir = std::fs::read_to_string(&ir_path).unwrap();
        assert!(ir.contains("logicodex_spawn"), "IR should contain spawn\n{}", ir);
        assert!(ir.contains("logicodex_channel_send"), "IR should contain send\n{}", ir);
        assert!(ir.contains("logicodex_channel_recv"), "IR should contain recv\n{}", ir);
        assert!(ir.contains("logicodex_join"), "IR should contain join\n{}", ir);
    }
}
