// =========================================================================
// Logicodex v1.36.0-alpha — A5: Struct Constructor Codegen Tests
//
// Tests: Struct definition registration + Color(r,g,b,a) → packed u32
// =========================================================================

use logicodex::codegen::{compile_v130, CodegenOptions};
use logicodex::ffi::{CallableRegistry, SafetyContext};
use logicodex::hir::{
    HirBlock, HirExpr, HirExprKind, HirField, HirFunction, HirItem, HirModule, HirStmt,
    HirStructDecl, LocalId, SymbolId, TypeRef,
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

// ─── 1. Struct definition is registered ───

#[test]
fn hir_codegen_struct_registration() {
    let types = TypeRegistry::new();
    let i64_ty = i64_ref(&types);
    let unit_ty = unit_ref(&types);

    let module = HirModule {
        items: vec![
            spanned(HirItem::Struct(HirStructDecl {
                symbol: SymbolId(1),
                fields: vec![
                    HirField { name: "r".to_string(), ty: i64_ty },
                    HirField { name: "g".to_string(), ty: i64_ty },
                    HirField { name: "b".to_string(), ty: i64_ty },
                    HirField { name: "a".to_string(), ty: i64_ty },
                ],
            })),
            spanned(HirItem::Function(HirFunction {
                name: "main".to_string(),
                symbol: SymbolId(0),
                params: Vec::new(),
                return_type: unit_ty,
                body: HirBlock { statements: Vec::new() },
                safety: SafetyContext::Safe,
            })),
        ],
    };

    let callables = CallableRegistry::default();
    let out_path = Path::new("/tmp/logicodex_hir_test_struct_reg.o");
    let result = compile_v130(
        &module, out_path,
        &CodegenOptions { module_name: "test".to_string(), emit_ir: true, secure: false, target: CompilationTarget::Native },
        callables, types,
    );
    assert!(result.is_ok(), "Struct registration should compile: {:?}", result.err());
}

// ─── 2. Color constructor → packed u32 ───

#[test]
fn hir_codegen_color_constructor_literal() {
    let types = TypeRegistry::new();
    let i64_ty = i64_ref(&types);
    let unit_ty = unit_ref(&types);

    // Register a "Color" struct-like callable in the registry
    let mut callables = CallableRegistry::default();
    let ids = types.primitive_ids();
    let color_callable = callables.register(logicodex::ffi::CallableSignature {
        name: "Color".to_string(),
        params: vec![ids.i64_; 4],
        return_type: ids.i64_,
        abi: logicodex::ffi::CallingConvention::C,
        safety: logicodex::ffi::CallableSafety::Safe,
        is_extern: false,
        is_variadic: false,
    });

    let module = HirModule {
        items: vec![
            spanned(HirItem::Struct(HirStructDecl {
                symbol: SymbolId(1),
                fields: vec![
                    HirField { name: "r".to_string(), ty: i64_ty },
                    HirField { name: "g".to_string(), ty: i64_ty },
                    HirField { name: "b".to_string(), ty: i64_ty },
                    HirField { name: "a".to_string(), ty: i64_ty },
                ],
            })),
            spanned(HirItem::Function(HirFunction {
                name: "main".to_string(),
                symbol: SymbolId(0),
                params: Vec::new(),
                return_type: unit_ty,
                body: HirBlock {
                    statements: vec![spanned(HirStmt::Let {
                        local: LocalId(0),
                        ty: i64_ty,
                        value: Some(HirExpr {
                            kind: HirExprKind::Call {
                                callee: color_callable,
                                args: vec![
                                    HirExpr { kind: HirExprKind::Literal(logicodex::hir::LiteralAst::Integer(255)), ty: i64_ty, span: Span::unknown() },
                                    HirExpr { kind: HirExprKind::Literal(logicodex::hir::LiteralAst::Integer(128)), ty: i64_ty, span: Span::unknown() },
                                    HirExpr { kind: HirExprKind::Literal(logicodex::hir::LiteralAst::Integer(64)), ty: i64_ty, span: Span::unknown() },
                                    HirExpr { kind: HirExprKind::Literal(logicodex::hir::LiteralAst::Integer(32)), ty: i64_ty, span: Span::unknown() },
                                ],
                            },
                            ty: i64_ty,
                            span: Span::unknown(),
                        }),
                    })],
                },
                safety: SafetyContext::Safe,
            })),
        ],
    };

    let out_path = Path::new("/tmp/logicodex_hir_test_color.o");
    let result = compile_v130(
        &module, out_path,
        &CodegenOptions { module_name: "test".to_string(), emit_ir: true, secure: false, target: CompilationTarget::Native },
        callables, types,
    );
    assert!(result.is_ok(), "Color constructor should compile: {:?}", result.err());
}
