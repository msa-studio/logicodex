#![allow(dead_code)]

// =============================================================================================================================================================================
// Callable signatures and FFI gates.
//
// Active module: provides the CallableRegistry / CallableSignature that codegen
// and the Raylib FFI (src/ffi/) use to resolve and type-check foreign calls.
// =============================================================================================================================================================================

pub mod math;
pub mod raylib;
pub mod raylib_sys;

// Re-export Raylib helpers for external use

use crate::hir::HirExpr;
use crate::span::{Diagnostic, DiagnosticCode, Severity, Span};
use crate::types::{CallableId, TypeId, TypeRegistry};
use std::collections::HashSet;

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

/// FFI capability policy (zero-trust, default deny).
///
/// Every `extern "C"` symbol a program calls must be explicitly permitted before
/// it can pass the gate. This is the security door that must exist BEFORE `lod`
/// opens the external C ecosystem: `lod` will later populate `allowed_symbols`
/// from `logicodex.toml [ffi.allow]`, but the deny-by-default contract lives
/// here so it is enforced regardless of how the policy is sourced.
///
/// Classification (Chief Architect, locked):
/// - `builtin_symbols`: the compiler-emitted runtime ABI (`logicodex_*`) only.
///   These are Logicodex's own OS-primitive shims, not third-party C, so they
///   are always allowed.
/// - `allowed_symbols`: explicit user opt-in (`ffi.allow = ["sqlite3_open", ...]`).
/// - `allowed_libraries`: reserved for library-level opt-in (`ffi.allow_lib`),
///   wired later; symbol-level is enough for the MVP.
/// - External C libraries (Raylib, sqlite, openssl, ...) are NOT builtin; they
///   require an explicit `allowed_symbols` entry.
#[derive(Debug, Clone)]
pub struct CapabilityPolicy {
    pub builtin_symbols: HashSet<String>,
    pub allowed_symbols: HashSet<String>,
    pub allowed_libraries: HashSet<String>,
    pub default_deny: bool,
}

impl Default for CapabilityPolicy {
    fn default() -> Self {
        Self::with_runtime_builtins()
    }
}

impl CapabilityPolicy {
    /// The compiler-emitted runtime ABI symbols. These are Logicodex's own
    /// shims over OS primitives (threads, channels, print), emitted by codegen
    /// and backed by the audited runtime_actor.c — never third-party C.
    pub const RUNTIME_BUILTINS: &'static [&'static str] = &[
        "logicodex_spawn",
        "logicodex_spawn_ctx",
        "logicodex_join",
        "logicodex_channel_create",
        "logicodex_channel_send",
        "logicodex_channel_recv",
        "logicodex_channel_try_send",
        "logicodex_channel_try_recv",
        "logicodex_print_i64",
        "logicodex_yield",
        "logicodex_sleep",
    ];

    /// A default-deny policy pre-seeded with only the runtime builtins. No
    /// external C library (Raylib included) is allowed until explicitly added.
    pub fn with_runtime_builtins() -> Self {
        CapabilityPolicy {
            builtin_symbols: Self::RUNTIME_BUILTINS
                .iter()
                .map(|s| s.to_string())
                .collect(),
            allowed_symbols: HashSet::new(),
            allowed_libraries: HashSet::new(),
            default_deny: true,
        }
    }

    /// Add an explicitly allowed symbol (the `ffi.allow` opt-in path). `lod`
    /// will call this from `logicodex.toml` later; tests/manual use call it now.
    pub fn allow_symbol(&mut self, symbol: impl Into<String>) {
        self.allowed_symbols.insert(symbol.into());
    }

    /// Check order: (1) runtime builtin -> allow; (2) symbol explicitly allowed
    /// -> allow; (3) library explicitly allowed (reserved) -> allow; (4) else
    /// deny (when default_deny). Symbol-level is authoritative for the MVP.
    pub fn is_symbol_allowed(&self, symbol: &str) -> bool {
        if self.builtin_symbols.contains(symbol) {
            return true;
        }
        if self.allowed_symbols.contains(symbol) {
            return true;
        }
        // Library-level opt-in is reserved; no symbol->library mapping yet.
        if !self.allowed_libraries.is_empty() {
            // Placeholder for the future ffi.allow_lib path. Until lod wires the
            // symbol->library table, an allow_lib entry does not auto-allow
            // individual symbols, so this stays conservative (no accidental
            // allow). Intentionally a no-op match for now.
        }
        !self.default_deny
    }
}

pub struct FfiGatekeeper<'a> {
    pub types: &'a TypeRegistry,
    pub callables: Option<&'a CallableRegistry>,
    /// FFI capability policy. `None` = capability checking disabled (legacy
    /// callers / contexts that only want type-safety). `Some(policy)` enforces
    /// default-deny capability gating on extern symbols.
    pub policy: Option<&'a CapabilityPolicy>,
}

impl<'a> FfiGatekeeper<'a> {
    /// Validate an FFI call with coercion support.
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
        // Check unsafe context
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

        // Check argument count
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

        // Type checking with coercion
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

        // Capability gate (zero-trust, default deny). This is an ADDITIONAL
        // layer on top of the unsafe/arity/type checks above: a call can be
        // type-correct and inside an unsafe block yet still be denied because
        // the program never opted in to that extern symbol. Only extern symbols
        // are gated; native Logicodex functions are not foreign and are skipped.
        if signature.is_extern {
            if let Some(policy) = self.policy {
                if !policy.is_symbol_allowed(&signature.name) {
                    return Err(ffi_error(
                        call_span,
                        format!(
                            "Ralat: panggilan extern '{}' ditolak oleh polisi keupayaan FFI. \
                             Isytiharkannya dalam ffi.allow sebelum guna.",
                            signature.name
                        ),
                        format!(
                            "Error: extern call '{}' denied by FFI capability policy. \
                             Declare it in ffi.allow before use.",
                            signature.name
                        ),
                    ));
                }
            }
        }
        Ok(())
    }

    /// Check if `actual` type can be coerced to `expected` type.
    /// Implements the widening coercion matrix for numeric types.
    fn is_compatible_with_coercion(
        &self,
        actual: crate::types::TypeId,
        expected: crate::types::TypeId,
    ) -> bool {
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

        // Widening coercion matrix
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

#[cfg(test)]
mod capability_tests {
    use super::CapabilityPolicy;

    #[test]
    fn external_symbol_denied_by_default() {
        // Zero-trust: an unknown external symbol is denied unless opted in.
        let policy = CapabilityPolicy::with_runtime_builtins();
        assert!(!policy.is_symbol_allowed("sqlite3_open"));
        assert!(!policy.is_symbol_allowed("InitWindow")); // Raylib not auto-allowed
    }

    #[test]
    fn explicitly_allowed_symbol_passes() {
        // The ffi.allow opt-in path: once added, the symbol is permitted.
        let mut policy = CapabilityPolicy::with_runtime_builtins();
        policy.allow_symbol("sqlite3_open");
        assert!(policy.is_symbol_allowed("sqlite3_open"));
        // Other unlisted symbols stay denied.
        assert!(!policy.is_symbol_allowed("sqlite3_exec"));
    }

    #[test]
    fn runtime_builtins_always_allowed() {
        // Compiler-emitted runtime ABI (logicodex_*) is always allowed; it is
        // Logicodex's own shim, not third-party C.
        let policy = CapabilityPolicy::with_runtime_builtins();
        assert!(policy.is_symbol_allowed("logicodex_spawn"));
        assert!(policy.is_symbol_allowed("logicodex_channel_send"));
        assert!(policy.is_symbol_allowed("logicodex_join"));
    }

    #[test]
    fn raylib_requires_explicit_allow() {
        // Chief Architect decision: Raylib is external C, not a builtin. It is
        // denied until explicitly allowed (then it passes like any opt-in).
        let mut policy = CapabilityPolicy::with_runtime_builtins();
        assert!(!policy.is_symbol_allowed("InitWindow"));
        policy.allow_symbol("InitWindow");
        assert!(policy.is_symbol_allowed("InitWindow"));
    }
}
