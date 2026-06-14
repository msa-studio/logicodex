// =========================================================================
// Enhanced TypeRegistry operations — type-system foundation.
//
// This module provides high-level type inspection and validation operations
// built on top of the core TypeRegistry. It is the interface used by the
// semantic analyzer, coercion engine, and FFI layer.
// =========================================================================

use crate::types::{CAbiInfo, PrimitiveType, TypeId, TypeKind, TypeRegistry};

/// High-level type inspection and validation.
///
/// TypeInspector provides ergonomic methods for querying type properties
/// without directly pattern-matching on TypeKind. It is the primary
/// interface for the semantic analysis phase.
pub struct TypeInspector<'a> {
    registry: &'a TypeRegistry,
}

impl<'a> TypeInspector<'a> {
    pub fn new(registry: &'a TypeRegistry) -> Self {
        Self { registry }
    }

    /// Check if a type is a primitive integer (signed or unsigned).
    pub fn is_integer(&self, id: TypeId) -> bool {
        matches!(
            self.registry.resolve(id),
            TypeKind::Primitive(
                PrimitiveType::I8
                    | PrimitiveType::I16
                    | PrimitiveType::I32
                    | PrimitiveType::I64
                    | PrimitiveType::U8
                    | PrimitiveType::U16
                    | PrimitiveType::U32
                    | PrimitiveType::U64
            )
        )
    }

    /// Check if a type is a signed integer.
    pub fn is_signed_integer(&self, id: TypeId) -> bool {
        matches!(
            self.registry.resolve(id),
            TypeKind::Primitive(
                PrimitiveType::I8 | PrimitiveType::I16 | PrimitiveType::I32 | PrimitiveType::I64
            )
        )
    }

    /// Check if a type is an unsigned integer.
    pub fn is_unsigned_integer(&self, id: TypeId) -> bool {
        matches!(
            self.registry.resolve(id),
            TypeKind::Primitive(
                PrimitiveType::U8 | PrimitiveType::U16 | PrimitiveType::U32 | PrimitiveType::U64
            )
        )
    }

    /// Check if a type is a floating-point type.
    pub fn is_float(&self, id: TypeId) -> bool {
        matches!(
            self.registry.resolve(id),
            TypeKind::Primitive(PrimitiveType::F32 | PrimitiveType::F64)
        )
    }

    /// Check if a type is numeric (integer or float).
    pub fn is_numeric(&self, id: TypeId) -> bool {
        self.is_integer(id) || self.is_float(id)
    }

    /// Check if a type is a pointer (any mutability).
    pub fn is_pointer(&self, id: TypeId) -> bool {
        matches!(self.registry.resolve(id), TypeKind::Pointer { .. })
    }

    /// Check if a type is the boolean type.
    pub fn is_bool(&self, id: TypeId) -> bool {
        matches!(
            self.registry.resolve(id),
            TypeKind::Primitive(PrimitiveType::Bool)
        )
    }

    /// Check if a type is the unit/void type.
    pub fn is_unit(&self, id: TypeId) -> bool {
        matches!(
            self.registry.resolve(id),
            TypeKind::Primitive(PrimitiveType::Unit)
        )
    }

    /// Check if a type is the string type.
    pub fn is_string(&self, id: TypeId) -> bool {
        matches!(
            self.registry.resolve(id),
            TypeKind::Primitive(PrimitiveType::String)
        )
    }

    /// Get the pointee type of a pointer, if it is one.
    pub fn pointee_type(&self, id: TypeId) -> Option<TypeId> {
        match self.registry.resolve(id) {
            TypeKind::Pointer { pointee, .. } => Some(*pointee),
            _ => None,
        }
    }

    /// Get the mutability of a pointer type.
    pub fn pointer_mutability(&self, id: TypeId) -> Option<crate::types::Mutability> {
        match self.registry.resolve(id) {
            TypeKind::Pointer { mutability, .. } => Some(*mutability),
            _ => None,
        }
    }

    /// Check if a type is valid for use (not Never or Unknown).
    pub fn is_valid(&self, id: TypeId) -> bool {
        !matches!(
            self.registry.resolve(id),
            TypeKind::Never | TypeKind::Unknown
        )
    }

    /// Get the byte width classification for diagnostics.
    /// Returns "1-byte", "2-byte", "4-byte", "8-byte", or "unknown".
    pub fn byte_width_name(&self, id: TypeId) -> &'static str {
        match self.registry.get_size(id) {
            0 => "zero-sized",
            1 => "1-byte",
            2 => "2-byte",
            4 => "4-byte",
            8 => "8-byte",
            _ => "variable-sized",
        }
    }

    /// Format a type for diagnostic messages.
    /// Returns a human-readable name like "I64", "Pointer<I32>", etc.
    pub fn type_name(&self, id: TypeId) -> String {
        match self.registry.resolve(id) {
            TypeKind::Primitive(p) => format!("{:?}", p),
            TypeKind::Pointer {
                pointee,
                mutability,
            } => {
                let pointee_name = self.type_name(*pointee);
                match mutability {
                    crate::types::Mutability::Mutable => {
                        format!("*mut {}", pointee_name)
                    }
                    crate::types::Mutability::Immutable => {
                        format!("*const {}", pointee_name)
                    }
                }
            }
            TypeKind::Struct(_) => "struct".to_string(),
            TypeKind::Enum(_) => "enum".to_string(),
            TypeKind::Array { element, len } => {
                format!("[{}; {}]", self.type_name(*element), len)
            }
            TypeKind::Function(_) => "fn".to_string(),
            TypeKind::Never => "!".to_string(),
            TypeKind::Unknown => "?".to_string(),
        }
    }

    /// Validate that a type can be used in an FFI call.
    /// Returns the C ABI info if valid, or an error message.
    pub fn validate_ffi_type(&self, id: TypeId) -> Result<CAbiInfo, String> {
        match self.registry.resolve(id) {
            TypeKind::Primitive(_) | TypeKind::Pointer { .. } | TypeKind::Function(_) => {
                Ok(self.registry.c_abi_info(id))
            }
            TypeKind::Array { .. } => {
                // Arrays can be passed as pointers in C
                Ok(self.registry.c_abi_info(id))
            }
            TypeKind::Struct(_) => {
                Err("Struct types in FFI require explicit layout (Sprint 3)".to_string())
            }
            TypeKind::Enum(_) => {
                Err("Enum types in FFI require explicit representation (Sprint 3)".to_string())
            }
            TypeKind::Never => Err("Never type cannot be used in FFI".to_string()),
            TypeKind::Unknown => Err("Unknown type cannot be used in FFI".to_string()),
        }
    }

    /// Check if a widening conversion from `from` to `to` is lossless.
    /// I32 → I64 is lossless, I64 → I32 is not.
    pub fn is_lossless_conversion(&self, from: TypeId, to: TypeId) -> bool {
        if from == to {
            return true;
        }
        match (self.registry.resolve(from), self.registry.resolve(to)) {
            // Integer widening
            (TypeKind::Primitive(PrimitiveType::I32), TypeKind::Primitive(PrimitiveType::I64)) => {
                true
            }
            (TypeKind::Primitive(PrimitiveType::U32), TypeKind::Primitive(PrimitiveType::U64)) => {
                true
            }
            (TypeKind::Primitive(PrimitiveType::I32), TypeKind::Primitive(PrimitiveType::F64)) => {
                true
            }
            (TypeKind::Primitive(PrimitiveType::I64), TypeKind::Primitive(PrimitiveType::F64)) => {
                true
            }
            (TypeKind::Primitive(PrimitiveType::F32), TypeKind::Primitive(PrimitiveType::F64)) => {
                true
            }
            // Same-width promotions (precision-preserving)
            (TypeKind::Primitive(PrimitiveType::I32), TypeKind::Primitive(PrimitiveType::F32)) => {
                true
            } // exactly representable up to 2^24
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inspector_correctly_classifies_primitives() {
        let registry = TypeRegistry::new();
        let ids = registry.primitive_ids();
        let inspector = TypeInspector::new(&registry);

        assert!(inspector.is_integer(ids.i32_));
        assert!(inspector.is_integer(ids.i64_));
        assert!(inspector.is_integer(ids.u32_));
        assert!(inspector.is_signed_integer(ids.i32_));
        assert!(!inspector.is_signed_integer(ids.u32_));
        assert!(inspector.is_unsigned_integer(ids.u32_));
        assert!(inspector.is_float(ids.f32_));
        assert!(inspector.is_float(ids.f64_));
        assert!(inspector.is_numeric(ids.i32_));
        assert!(inspector.is_numeric(ids.f64_));
        assert!(inspector.is_bool(ids.bool_));
        assert!(inspector.is_string(ids.string));
        assert!(inspector.is_unit(ids.unit));
        assert!(!inspector.is_integer(ids.bool_));
        assert!(!inspector.is_numeric(ids.bool_));
    }

    #[test]
    fn inspector_pointer_detection() {
        let mut registry = TypeRegistry::new();
        let ids = registry.primitive_ids();
        let ptr = registry.intern(TypeKind::Pointer {
            pointee: ids.i32_,
            mutability: crate::types::Mutability::Immutable,
        });
        let inspector = TypeInspector::new(&registry);

        assert!(inspector.is_pointer(ptr));
        assert!(!inspector.is_pointer(ids.i32_));
        assert_eq!(inspector.pointee_type(ptr), Some(ids.i32_));
        assert_eq!(inspector.pointee_type(ids.i32_), None);
    }

    #[test]
    fn inspector_type_names() {
        let registry = TypeRegistry::new();
        let ids = registry.primitive_ids();
        let inspector = TypeInspector::new(&registry);

        assert_eq!(inspector.type_name(ids.i32_), "I32");
        assert_eq!(inspector.type_name(ids.i64_), "I64");
        assert_eq!(inspector.type_name(ids.f64_), "F64");
        assert_eq!(inspector.type_name(ids.bool_), "Bool");
        assert_eq!(inspector.type_name(ids.unit), "Unit");
        assert_eq!(inspector.byte_width_name(ids.i32_), "4-byte");
        assert_eq!(inspector.byte_width_name(ids.i64_), "8-byte");
        assert_eq!(inspector.byte_width_name(ids.unit), "zero-sized");
    }

    #[test]
    fn inspector_ffi_validation() {
        let registry = TypeRegistry::new();
        let ids = registry.primitive_ids();
        let inspector = TypeInspector::new(&registry);

        // Primitives are valid for FFI
        assert!(inspector.validate_ffi_type(ids.i32_).is_ok());
        assert!(inspector.validate_ffi_type(ids.f64_).is_ok());

        // Unit/void is valid for FFI (return type)
        assert!(inspector.validate_ffi_type(ids.unit).is_ok());
    }

    #[test]
    fn inspector_lossless_conversions() {
        let registry = TypeRegistry::new();
        let ids = registry.primitive_ids();
        let inspector = TypeInspector::new(&registry);

        // Widening is lossless
        assert!(inspector.is_lossless_conversion(ids.i32_, ids.i64_));
        assert!(inspector.is_lossless_conversion(ids.f32_, ids.f64_));

        // Narrowing is not
        assert!(!inspector.is_lossless_conversion(ids.i64_, ids.i32_));
        assert!(!inspector.is_lossless_conversion(ids.f64_, ids.f32_));

        // Same type is lossless (identity)
        assert!(inspector.is_lossless_conversion(ids.i32_, ids.i32_));
    }
}
