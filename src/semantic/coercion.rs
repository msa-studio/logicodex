// =========================================================================
// Logicodex v1.30 — Coercion Engine
// Sprint 1: Type System Foundation
//
// The CoercionEngine determines if and how types can be converted between
// each other. It enforces Logicodex's safety-first philosophy:
//   - Widening conversions are implicit (safe)
//   - Narrowing conversions require explicit casts (unsafe)
//   - Pointer conversions are restricted
// =========================================================================

use crate::types::{PrimitiveType, TypeId, TypeKind, TypeRegistry};

/// Result of a coercion check between two types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoercionResult {
    /// Types are identical — no conversion needed.
    Identity,
    /// Safe implicit conversion (widening, no data loss).
    Implicit { target: TypeId },
    /// Explicit cast required (narrowing, potential data loss).
    RequiresCast { target: TypeId },
    /// No valid conversion exists between these types.
    Incompatible,
}

impl CoercionResult {
    /// Check if the coercion is allowed (Identity or Implicit).
    pub fn is_allowed(&self) -> bool {
        matches!(self, CoercionResult::Identity | CoercionResult::Implicit { .. })
    }

    /// Check if an explicit cast is required.
    pub fn needs_cast(&self) -> bool {
        matches!(self, CoercionResult::RequiresCast { .. })
    }

    /// Get the target type of the coercion, if any.
    pub fn target_type(&self) -> Option<TypeId> {
        match self {
            CoercionResult::Implicit { target } | CoercionResult::RequiresCast { target } => {
                Some(*target)
            }
            _ => None,
        }
    }
}

/// Determines type conversion rules for Logicodex.
///
/// The coercion ladder (from highest to lowest):
/// ```text
///     F64 (widest)
///     I64, U64
///     F32
///     I32, U32
///     I16, U16
///     I8, U8
///     Bool
///     Unit (narrowest)
/// ```
pub struct CoercionEngine<'a> {
    registry: &'a TypeRegistry,
}

impl<'a> CoercionEngine<'a> {
    pub fn new(registry: &'a TypeRegistry) -> Self {
        Self { registry }
    }

    /// Determine if `from` can be coerced to `to`.
    /// This is the primary entry point for type checking.
    pub fn can_coerce(&self, from: TypeId, to: TypeId) -> CoercionResult {
        // Identity: same type
        if from == to {
            return CoercionResult::Identity;
        }

        let from_kind = self.registry.resolve(from);
        let to_kind = self.registry.resolve(to);

        match (from_kind, to_kind) {
            // ─── Primitive coercions ───
            (TypeKind::Primitive(from_p), TypeKind::Primitive(to_p)) => {
                self.coerce_primitives(*from_p, *to_p, to)
            }

            // ─── Pointer coercions ───
            // Same pointee type: identity-equivalent
            (
                TypeKind::Pointer {
                    pointee: p1,
                    mutability: m1,
                },
                TypeKind::Pointer {
                    pointee: p2,
                    mutability: m2,
                },
            ) => {
                if p1 == p2 {
                    // Mutable → Immutable is safe (covariant)
                    // Immutable → Mutable is unsafe (requires cast)
                    match (m1, m2) {
                        (_, crate::types::Mutability::Immutable) => {
                            CoercionResult::Implicit { target: to }
                        }
                        (crate::types::Mutability::Immutable, crate::types::Mutability::Mutable) => {
                            CoercionResult::RequiresCast { target: to }
                        }
                        _ => CoercionResult::Identity, // same mutability, same pointee
                    }
                } else {
                    // Different pointee types — generally incompatible
                    // Exception: *const T can coerce to *const Void (opaque)
                    if p2.0 == self.registry.primitive(PrimitiveType::Unit).0 {
                        CoercionResult::RequiresCast { target: to }
                    } else {
                        CoercionResult::Incompatible
                    }
                }
            }

            // ─── String to C-string pointer (FFI) ───
            // String → *const I8 (for Raylib FFI)
            (
                TypeKind::Primitive(PrimitiveType::String),
                TypeKind::Pointer {
                    pointee,
                    mutability: crate::types::Mutability::Immutable,
                },
            ) => {
                if self.registry.resolve(*pointee)
                    == &TypeKind::Primitive(PrimitiveType::I8)
                {
                    CoercionResult::Implicit { target: to }
                } else {
                    CoercionResult::Incompatible
                }
            }

            // ─── Array to pointer decay (C-style) ───
            // [T; N] → *T  (for FFI compatibility)
            (
                TypeKind::Array { element, .. },
                TypeKind::Pointer { pointee, .. },
            ) => {
                if element == pointee {
                    CoercionResult::Implicit { target: to }
                } else {
                    CoercionResult::Incompatible
                }
            }

            // ─── All other combinations are incompatible ───
            _ => CoercionResult::Incompatible,
        }
    }

    /// Check if a function argument of type `arg_type` can be passed
    /// to a parameter of type `param_type`.
    /// This is used for function call type checking.
    pub fn can_pass_argument(&self, arg_type: TypeId, param_type: TypeId) -> CoercionResult {
        // For function arguments, we use the same coercion rules
        // but with stricter defaults (no auto-widening for I64 → I32)
        self.can_coerce(arg_type, param_type)
    }

    /// Get the common type of two types (for type inference in binary ops).
    /// Returns None if the types are incompatible.
    pub fn common_type(&self, left: TypeId, right: TypeId) -> Option<TypeId> {
        if left == right {
            return Some(left);
        }

        let left_kind = self.registry.resolve(left);
        let right_kind = self.registry.resolve(right);

        // Both numeric: return the wider type
        if let (
            TypeKind::Primitive(left_p),
            TypeKind::Primitive(right_p),
        ) = (left_kind, right_kind)
        {
            return self.wider_primitive(*left_p, *right_p);
        }

        // Pointers to same type: use left's mutability
        if let (
            TypeKind::Pointer {
                pointee: p1,
                mutability: _m1,
            },
            TypeKind::Pointer {
                pointee: p2,
                mutability: _,
            },
        ) = (left_kind, right_kind)
        {
            if p1 == p2 {
                return Some(left); // preserve left's mutability
            }
        }

        None
    }

    // ─── Internal helpers ───

    fn coerce_primitives(
        &self,
        from: PrimitiveType,
        to: PrimitiveType,
        to_id: TypeId,
    ) -> CoercionResult {
        use PrimitiveType::*;

        // Same primitive: should have been caught by identity check
        if from == to {
            return CoercionResult::Identity;
        }

        match (from, to) {
            // ─── Widening (implicit, safe) ───
            // Integer widening
            (I8, I16 | I32 | I64) => CoercionResult::Implicit { target: to_id },
            (I16, I32 | I64) => CoercionResult::Implicit { target: to_id },
            (I32, I64) => CoercionResult::Implicit { target: to_id },
            (U8, U16 | U32 | U64 | I16 | I32 | I64) => {
                CoercionResult::Implicit { target: to_id }
            }
            (U16, U32 | U64 | I32 | I64) => CoercionResult::Implicit { target: to_id },
            (U32, U64 | I64) => CoercionResult::Implicit { target: to_id },

            // Integer to float (widening)
            (I32, F32 | F64) => CoercionResult::Implicit { target: to_id },
            (I64, F64) => CoercionResult::Implicit { target: to_id },
            (U32, F32 | F64) => CoercionResult::Implicit { target: to_id },
            (U64, F64) => CoercionResult::Implicit { target: to_id },

            // Float widening
            (F32, F64) => CoercionResult::Implicit { target: to_id },

            // ─── Narrowing (requires explicit cast) ───
            (I64, I32 | I16 | I8) => CoercionResult::RequiresCast { target: to_id },
            (I32, I16 | I8) => CoercionResult::RequiresCast { target: to_id },
            (I16, I8) => CoercionResult::RequiresCast { target: to_id },
            (U64, U32 | U16 | U8 | I32 | I16 | I8) => {
                CoercionResult::RequiresCast { target: to_id }
            }
            (U32, U16 | U8 | I16 | I8) => CoercionResult::RequiresCast { target: to_id },
            (U16, U8 | I8) => CoercionResult::RequiresCast { target: to_id },

            // Float narrowing
            (F64, F32) => CoercionResult::RequiresCast { target: to_id },

            // Float to integer (always requires cast — data loss)
            (F32, I32 | I64 | U32 | U64) => CoercionResult::RequiresCast { target: to_id },
            (F64, I32 | I64 | U32 | U64) => CoercionResult::RequiresCast { target: to_id },

            // Bool conversions (requires cast — semantically distinct)
            (Bool, I32 | I64 | U32 | U64) => CoercionResult::RequiresCast { target: to_id },
            (I32 | I64 | U32 | U64, Bool) => CoercionResult::RequiresCast { target: to_id },

            // Unit conversions (incompatible)
            (Unit, _) | (_, Unit) => CoercionResult::Incompatible,

            // String conversions (incompatible)
            (String, _) | (_, String) => CoercionResult::Incompatible,

            // Cross-signed conversions (potential overflow)
            (I64, U64) | (U64, I64) => CoercionResult::RequiresCast { target: to_id },
            (I32, U32) | (U32, I32) => CoercionResult::RequiresCast { target: to_id },

            // Default: incompatible
            _ => CoercionResult::Incompatible,
        }
    }

    /// Return the wider of two primitive types.
    fn wider_primitive(&self, a: PrimitiveType, b: PrimitiveType) -> Option<TypeId> {
        use PrimitiveType::*;
        let ids = self.registry.primitive_ids();

        // Floats are wider than integers of same or smaller size
        match (a, b) {
            (F64, _) | (_, F64) => Some(ids.f64_),
            (F32, I32 | U32 | I16 | U16 | I8 | U8 | Bool)
            | (I32 | U32 | I16 | U16 | I8 | U8 | Bool, F32) => Some(ids.f32_),
            (I64, _) | (_, I64) => Some(ids.i64_),
            (U64, _) | (_, U64) => Some(ids.u64_),
            (I32, _) | (_, I32) => Some(ids.i32_),
            (U32, _) | (_, U32) => Some(ids.u32_),
            (I16, _) | (_, I16) => Some(ids.i16_),
            (U16, _) | (_, U16) => Some(ids.u16_),
            (I8, _) | (_, I8) => Some(ids.i8_),
            (U8, _) | (_, U8) => Some(ids.u8_),
            (Bool, Bool) => Some(ids.bool_),
            _ => None, // incompatible categories (e.g., String + I32)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> (&'static TypeRegistry, CoercionEngine<'static>) {
        let registry: &'static TypeRegistry =
            Box::leak(Box::new(TypeRegistry::new()));
        let engine = CoercionEngine::new(registry);
        (registry, engine)
    }

    #[test]
    fn identity_coercion() {
        let (reg, engine) = setup();
        let ids = reg.primitive_ids();
        assert_eq!(engine.can_coerce(ids.i32_, ids.i32_), CoercionResult::Identity);
    }

    #[test]
    fn integer_widening_is_implicit() {
        let (reg, engine) = setup();
        let ids = reg.primitive_ids();
        assert!(
            engine.can_coerce(ids.i32_, ids.i64_).is_allowed(),
            "I32 -> I64 should be implicit"
        );
        assert!(
            engine.can_coerce(ids.u32_, ids.u64_).is_allowed(),
            "U32 -> U64 should be implicit"
        );
        assert!(
            engine.can_coerce(ids.i8_, ids.i32_).is_allowed(),
            "I8 -> I32 should be implicit"
        );
    }

    #[test]
    fn integer_narrowing_requires_cast() {
        let (reg, engine) = setup();
        let ids = reg.primitive_ids();
        assert!(
            engine.can_coerce(ids.i64_, ids.i32_).needs_cast(),
            "I64 -> I32 should require cast"
        );
        assert!(
            engine.can_coerce(ids.i32_, ids.i8_).needs_cast(),
            "I32 -> I8 should require cast"
        );
    }

    #[test]
    fn int_to_float_widening() {
        let (reg, engine) = setup();
        let ids = reg.primitive_ids();
        assert!(
            engine.can_coerce(ids.i32_, ids.f64_).is_allowed(),
            "I32 -> F64 should be implicit"
        );
        assert!(
            engine.can_coerce(ids.i64_, ids.f64_).is_allowed(),
            "I64 -> F64 should be implicit"
        );
    }

    #[test]
    fn float_to_int_is_narrowing() {
        let (reg, engine) = setup();
        let ids = reg.primitive_ids();
        assert!(
            engine.can_coerce(ids.f64_, ids.i32_).needs_cast(),
            "F64 -> I32 should require cast"
        );
        assert!(
            engine.can_coerce(ids.f32_, ids.i64_).needs_cast(),
            "F32 -> I64 should require cast"
        );
    }

    #[test]
    fn string_to_c_string_ptr() {
        let (reg, engine) = setup();
        let ids = reg.primitive_ids();
        let c_string = reg.const_char_ptr();

        // String → *const I8 (for FFI)
        let result = engine.can_coerce(ids.string, c_string);
        assert!(result.is_allowed(), "String -> *const I8 should be implicit for FFI");
    }

    #[test]
    fn common_type_for_binops() {
        let (reg, engine) = setup();
        let ids = reg.primitive_ids();

        assert_eq!(engine.common_type(ids.i32_, ids.i64_), Some(ids.i64_));
        assert_eq!(engine.common_type(ids.f32_, ids.f64_), Some(ids.f64_));
        assert_eq!(engine.common_type(ids.i32_, ids.f64_), Some(ids.f64_));
        assert_eq!(engine.common_type(ids.i32_, ids.bool_), None); // incompatible
    }

    #[test]
    fn unit_is_incompatible() {
        let (reg, engine) = setup();
        let ids = reg.primitive_ids();
        assert_eq!(
            engine.can_coerce(ids.unit, ids.i32_),
            CoercionResult::Incompatible
        );
        assert_eq!(
            engine.can_coerce(ids.i32_, ids.unit),
            CoercionResult::Incompatible
        );
    }
}
