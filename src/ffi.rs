#![allow(dead_code)]

// =============================================================================================================================================================================
// Logicodex v1.30 architecture simulation: callable signatures and FFI gates.
//
// This module is dormant. Extern and unsafe execution remains parser-trapped in
// the current v1.21-alpha split-implementation boundary.
// =============================================================================================================================================================================

pub mod raylib;
pub mod raylib_sys;
pub mod math;

// v1.42: Re-export Raylib helpers for external use


use crate::hir::HirExpr;
use crate::span::{Diagnostic, DiagnosticCode, Severity, Span};
use crate::types::{CallableId, TypeId, TypeRegistry};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallableSignature {
    pub name: String,
    pub params: Vec<TypeId>,
    pub return_type: TypeId,
    pub abi: CallingConvention,
    pub safety: CallableSafety,
    pub is_extern: bool,
    pub is_variadic: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CallingConvention {
    C,
    StdCall,
    SysCall,
    FastCall,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CallableSafety {
    Safe,
    UnsafeRequired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SafetyContext {
    Safe,
    Unsafe,
}

#[derive(Debug, Default, Clone)]
pub struct CallableRegistry {
    pub signatures: Vec<CallableSignature>,
}

impl CallableRegistry {
    pub fn register(&mut self, signature: CallableSignature) -> CallableId {
        let id = CallableId(self.signatures.len() as u32);
        self.signatures.push(signature);
        id
    }

    pub fn get(&self, callable: CallableId) -> Option<&CallableSignature> {
        self.signatures.get(callable.0 as usize)
    }

    pub fn find_by_name(&self, name: &str) -> Option<(CallableId, &CallableSignature)> {
        self.signatures
            .iter()
            .enumerate()
            .find(|(_, signature)| signature.name == name)
            .map(|(index, signature)| (CallableId(index as u32), signature))
    }

    /// Look up a callable by name, returning an owned copy of the signature.
    pub fn lookup_callable(&self, name: &str) -> Option<(CallableId, CallableSignature)> {
        self.signatures
            .iter()
            .enumerate()
            .find(|(_, signature)| signature.name == name)
            .map(|(index, signature)| (CallableId(index as u32), signature.clone()))
    }
}

pub struct FfiGatekeeper<'a> {
    pub types: &'a TypeRegistry,
    pub callables: Option<&'a CallableRegistry>,
}

impl<'a> FfiGatekeeper<'a> {
    /// v1.42 P8: Validate an FFI call with coercion support.
    //
    /// Coercion rules (widening allowed, narrowing rejected):
    /// - I32 ← I64 (widening): allowed
    /// - I32 ← F64 (int-to-float): allowed
    /// - I64 ← I32 (narrowing): requires explicit cast — error
    /// - I64 ← F64 (widening): allowed
    /// - Struct types: exact match only (no struct coercion)
    /// - Bool: exact match only
    pub fn validate_call(
        &self,
        signature: &CallableSignature,
        args: &[HirExpr],
        context: SafetyContext,
        call_span: Span,
    ) -> Result<(), Diagnostic> {
        // v1.42 P8: Check unsafe context
        if (signature.is_extern || signature.safety == CallableSafety::UnsafeRequired)
            && context != SafetyContext::Unsafe
        {
            return Err(ffi_error(
                call_span,
                format!(
                    "Ralat: Panggilan fungsi luar '{}' memerlukan blok unsafe",
                    signature.name
                ),
                format!(
                    "Error: External function call '{}' requires an unsafe block",
                    signature.name
                ),
            ));
        }

        // v1.42 P8: Check argument count
        if !signature.is_variadic && args.len() != signature.params.len() {
            return Err(ffi_error(
                call_span,
                format!(
                    "Ralat: Fungsi '{}' memerlukan {} argumen tetapi menerima {}",
                    signature.name,
                    signature.params.len(),
                    args.len()
                ),
                format!(
                    "Error: Function '{}' expects {} argument(s) but received {}",
                    signature.name,
                    signature.params.len(),
                    args.len()
                ),
            ));
        }

        if signature.is_variadic && args.len() < signature.params.len() {
            return Err(ffi_error(
                call_span,
                format!(
                    "Ralat: Fungsi variadik '{}' memerlukan sekurang-kurangnya {} argumen",
                    signature.name,
                    signature.params.len()
                ),
                format!(
                    "Error: Variadic function '{}' expects at least {} argument(s)",
                    signature.name,
                    signature.params.len()
                ),
            ));
        }

        // v1.42 P8: Type checking with coercion
        for (index, expected) in signature.params.iter().enumerate() {
            let Some(actual) = args.get(index) else {
                break;
            };
            if !self.is_compatible_with_coercion(actual.ty.id, *expected) {
                return Err(ffi_error(
                    actual.span,
                    format!(
                        "Ralat: Argumen {} untuk fungsi '{}' mempunyai jenis yang tidak sepadan (diperlukan: {}, diterima: {})",
                        index + 1,
                        signature.name,
                        self.types.type_name(*expected).unwrap_or_default(),
                        self.types.type_name(actual.ty.id).unwrap_or_default(),
                    ),
                    format!(
                        "Error: Argument {} for function '{}' has an incompatible type (expected: {}, got: {})",
                        index + 1,
                        signature.name,
                        self.types.type_name(*expected).unwrap_or_default(),
                        self.types.type_name(actual.ty.id).unwrap_or_default(),
                    ),
                ));
            }
        }

        Ok(())
    }

    /// v1.42 P8: Check if `actual` type can be coerced to `expected` type.
    /// Implements the widening coercion matrix for numeric types.
    fn is_compatible_with_coercion(&self, actual: crate::types::TypeId, expected: crate::types::TypeId) -> bool {
        // Exact match always OK
        if self.types.is_equivalent(actual, expected) {
            return true;
        }

        // Get the primitive kinds
        let actual_kind = self.types.resolve(actual);
        let expected_kind = self.types.resolve(expected);

        use crate::types::PrimitiveType;
        let actual_prim = match actual_kind {
            crate::types::TypeKind::Primitive(p) => p,
            _ => return false, // Struct, pointer, etc: exact match only
        };
        let expected_prim = match expected_kind {
            crate::types::TypeKind::Primitive(p) => p,
            _ => return false,
        };

        // v1.42 P8: Widening coercion matrix
        match (actual_prim, expected_prim) {
            // Legacy IMON exact check (kind only, no widening)
            (PrimitiveType::I32, PrimitiveType::I32) => true,
            (PrimitiveType::I64, PrimitiveType::I64) => true,
            (PrimitiveType::F32, PrimitiveType::F32) => true,
            (PrimitiveType::F64, PrimitiveType::F64) => true,

            // P8 widening allowed
            (PrimitiveType::I32, PrimitiveType::I64) => true,
            (PrimitiveType::I32, PrimitiveType::F64) => true,
            (PrimitiveType::I64, PrimitiveType::F64) => true,

            _ => false,
        }
    }
}

/// Helper: create an FFI diagnostic with dual messages.
fn ffi_error(span: Span, mus_msg: String, eng_msg: String) -> Diagnostic {
    Diagnostic {
        code: DiagnosticCode::FfiBoundaryViolation,
        severity: Severity::Error,
        message_ms: mus_msg,
        message_en: eng_msg,
        primary_span: span,
        notes: Vec::new(),
    }
}
