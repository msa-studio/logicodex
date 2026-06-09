#![allow(dead_code)]

// =========================================================================
// Logicodex v1.30 architecture simulation: backend codegen contract.
//
// This module is dormant. The active v1.21-alpha LLVM backend remains in
// codegen.rs and must not consume these contracts until HIR, layout, and FFI
// validation are activated by staged implementation.
// =========================================================================

use crate::ffi::{CallableRegistry, CallableSignature};
use crate::hir::{HirFunction, HirModule};
use crate::layout::LayoutRegistry;
use crate::types::StructLayout;
use crate::types::TypeRegistry;

pub struct CodegenInput<'a> {
    pub hir: &'a HirModule,
    pub types: &'a TypeRegistry,
    pub layouts: &'a LayoutRegistry,
    pub callables: &'a CallableRegistry,
}

pub trait CodegenBackend {
    type Error;

    fn emit_module(&mut self, input: CodegenInput<'_>) -> Result<(), Self::Error>;

    fn emit_function(&mut self, function: &HirFunction) -> Result<(), Self::Error>;

    fn emit_struct_type(&mut self, layout: &StructLayout) -> Result<(), Self::Error>;

    fn emit_extern_function(&mut self, signature: &CallableSignature) -> Result<(), Self::Error>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ffi::{CallableRegistry, CallableSafety, CallingConvention};
    use crate::hir::{HirBlock, HirFunction, HirItem, HirModule, SymbolId};
    use crate::layout::LayoutRegistry;
    use crate::span::{Span, Spanned};
    use crate::types::{TypeRef, TypeRegistry};

    #[derive(Default)]
    struct MockBackend {
        modules: usize,
        functions: usize,
        structs: usize,
        externs: usize,
    }

    impl CodegenBackend for MockBackend {
        type Error = String;

        fn emit_module(&mut self, input: CodegenInput<'_>) -> Result<(), Self::Error> {
            self.modules += 1;
            for item in &input.hir.items {
                if let HirItem::Function(function) = &item.node {
                    self.emit_function(function)?;
                }
            }
            for layout in &input.layouts.structs {
                self.emit_struct_type(layout)?;
            }
            for signature in &input.callables.signatures {
                self.emit_extern_function(signature)?;
            }
            let _ = input.types.primitive_ids();
            Ok(())
        }

        fn emit_function(&mut self, _function: &HirFunction) -> Result<(), Self::Error> {
            self.functions += 1;
            Ok(())
        }

        fn emit_struct_type(&mut self, _layout: &StructLayout) -> Result<(), Self::Error> {
            self.structs += 1;
            Ok(())
        }

        fn emit_extern_function(
            &mut self,
            _signature: &CallableSignature,
        ) -> Result<(), Self::Error> {
            self.externs += 1;
            Ok(())
        }
    }

    #[test]
    fn backend_contract_dispatches_core_inputs() {
        let types = TypeRegistry::new();
        let ids = types.primitive_ids();
        let hir = HirModule {
            items: vec![Spanned {
                node: HirItem::Function(HirFunction {
                    name: "main".to_string(),
                    symbol: SymbolId(0),
                    params: Vec::new(),
                    return_type: TypeRef { id: ids.unit },
                    body: HirBlock {
                        statements: Vec::new(),
                    },
                    safety: crate::ffi::SafetyContext::Safe,
                }),
                span: Span::unknown(),
            }],
        };
        let layouts = LayoutRegistry::default();
        let mut callables = CallableRegistry::default();
        callables.register(CallableSignature {
            name: "puts".to_string(),
            params: vec![ids.string],
            return_type: ids.i32_,
            abi: CallingConvention::C,
            safety: CallableSafety::UnsafeRequired,
            is_extern: true,
            is_variadic: false,
        });
        let mut backend = MockBackend::default();

        backend
            .emit_module(CodegenInput {
                hir: &hir,
                types: &types,
                layouts: &layouts,
                callables: &callables,
            })
            .expect("mock backend should emit");

        assert_eq!(backend.modules, 1);
        assert_eq!(backend.functions, 1);
        assert_eq!(backend.structs, 0);
        assert_eq!(backend.externs, 1);
    }
}
