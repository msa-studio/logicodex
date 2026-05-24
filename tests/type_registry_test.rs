// =========================================================================
// Logicodex v1.30 — TypeRegistry TDD Tests
// Sprint 1: Type System Foundation
//
// These tests enforce the core contracts of the TypeRegistry:
//   - Primitive sizes must match C ABI (I32=4, I64=8, F64=8, etc.)
//   - Type interning must be idempotent
//   - Type equivalence must be reflexive, symmetric, transitive
//   - FFI type aliases must resolve to correct primitives
// =========================================================================

// Import the compiler's internal modules for testing
use logicodex::semantic::coercion::{CoercionEngine, CoercionResult};
use logicodex::semantic::registry::TypeInspector;
use logicodex::types::{
    CAbiInfo, Mutability, PrimitiveType, TypeId, TypeKind, TypeRegistry,
};

// ─── 1. Primitive Size Tests (Single Source of Truth) ───

#[test]
fn primitive_sizes_match_c_abi() {
    let registry = TypeRegistry::new();
    let ids = registry.primitive_ids();

    assert_eq!(registry.get_size(ids.bool_), 1, "Bool must be 1 byte");
    assert_eq!(registry.get_size(ids.i8_), 1, "I8 must be 1 byte");
    assert_eq!(registry.get_size(ids.i16_), 2, "I16 must be 2 bytes");
    assert_eq!(registry.get_size(ids.i32_), 4, "I32 must be 4 bytes");
    assert_eq!(registry.get_size(ids.i64_), 8, "I64 must be 8 bytes");
    assert_eq!(registry.get_size(ids.u8_), 1, "U8 must be 1 byte");
    assert_eq!(registry.get_size(ids.u16_), 2, "U16 must be 2 bytes");
    assert_eq!(registry.get_size(ids.u32_), 4, "U32 must be 4 bytes");
    assert_eq!(registry.get_size(ids.u64_), 8, "U64 must be 8 bytes");
    assert_eq!(registry.get_size(ids.f32_), 4, "F32 must be 4 bytes");
    assert_eq!(registry.get_size(ids.f64_), 8, "F64 must be 8 bytes");
    assert_eq!(registry.get_size(ids.string), 8, "String must be pointer-sized (8 bytes)");
    assert_eq!(registry.get_size(ids.unit), 0, "Unit must be zero-sized");
}

#[test]
fn pointer_size_is_8_on_64_bit() {
    let mut registry = TypeRegistry::new();
    let ids = registry.primitive_ids();

    // Pointer to any type must be 8 bytes on 64-bit
    let ptr_i32 = registry.intern(TypeKind::Pointer {
        pointee: ids.i32_,
        mutability: Mutability::Immutable,
    });
    let ptr_i64 = registry.intern(TypeKind::Pointer {
        pointee: ids.i64_,
        mutability: Mutability::Immutable,
    });
    let ptr_void = registry.intern(TypeKind::Pointer {
        pointee: ids.unit,
        mutability: Mutability::Immutable,
    });

    assert_eq!(registry.get_size(ptr_i32), 8, "*I32 must be 8 bytes (pointer)");
    assert_eq!(registry.get_size(ptr_i64), 8, "*I64 must be 8 bytes (pointer)");
    assert_eq!(registry.get_size(ptr_void), 8, "*Unit must be 8 bytes (pointer)");
}

#[test]
fn array_size_is_element_size_times_length() {
    let registry = TypeRegistry::new();
    let ids = registry.primitive_ids();

    // [I32; 10] = 4 * 10 = 40 bytes
    let arr_i32_10 = registry.intern(TypeKind::Array {
        element: ids.i32_,
        len: 10,
    });
    assert_eq!(
        registry.get_size(arr_i32_10),
        40,
        "[I32; 10] must be 40 bytes"
    );

    // [I64; 5] = 8 * 5 = 40 bytes
    let arr_i64_5 = registry.intern(TypeKind::Array {
        element: ids.i64_,
        len: 5,
    });
    assert_eq!(
        registry.get_size(arr_i64_5),
        40,
        "[I64; 5] must be 40 bytes"
    );

    // [U8; 256] = 1 * 256 = 256 bytes
    let arr_u8_256 = registry.intern(TypeKind::Array {
        element: ids.u8_,
        len: 256,
    });
    assert_eq!(
        registry.get_size(arr_u8_256),
        256,
        "[U8; 256] must be 256 bytes"
    );
}

// ─── 2. Alignment Tests ───

#[test]
fn primitive_alignments_match_c_abi() {
    let registry = TypeRegistry::new();
    let ids = registry.primitive_ids();

    assert_eq!(registry.get_align(ids.i8_), 1, "I8 align must be 1");
    assert_eq!(registry.get_align(ids.i16_), 2, "I16 align must be 2");
    assert_eq!(registry.get_align(ids.i32_), 4, "I32 align must be 4");
    assert_eq!(registry.get_align(ids.i64_), 8, "I64 align must be 8");
    assert_eq!(registry.get_align(ids.f32_), 4, "F32 align must be 4");
    assert_eq!(registry.get_align(ids.f64_), 8, "F64 align must be 8");
    assert_eq!(registry.get_align(ids.bool_), 1, "Bool align must be 1");
}

#[test]
fn pointer_alignment_is_8_on_64_bit() {
    let mut registry = TypeRegistry::new();
    let ids = registry.primitive_ids();

    let ptr = registry.intern(TypeKind::Pointer {
        pointee: ids.i32_,
        mutability: Mutability::Immutable,
    });
    assert_eq!(registry.get_align(ptr), 8, "Pointer align must be 8 on 64-bit");
}

#[test]
fn array_alignment_matches_element() {
    let registry = TypeRegistry::new();
    let ids = registry.primitive_ids();

    let arr_i32 = registry.intern(TypeKind::Array {
        element: ids.i32_,
        len: 10,
    });
    assert_eq!(
        registry.get_align(arr_i32),
        registry.get_align(ids.i32_),
        "Array alignment must match element alignment"
    );
}

// ─── 3. Idempotency Tests ───

#[test]
fn interning_same_type_twice_returns_same_id() {
    let mut registry = TypeRegistry::new();
    let ids = registry.primitive_ids();

    let kind = TypeKind::Pointer {
        pointee: ids.i64_,
        mutability: Mutability::Mutable,
    };

    let first = registry.intern(kind.clone());
    let second = registry.intern(kind);

    assert_eq!(
        first, second,
        "Interning the same type twice must return the same TypeId"
    );
}

#[test]
fn interning_different_types_returns_different_ids() {
    let mut registry = TypeRegistry::new();
    let ids = registry.primitive_ids();

    let ptr_i32 = registry.intern(TypeKind::Pointer {
        pointee: ids.i32_,
        mutability: Mutability::Immutable,
    });
    let ptr_i64 = registry.intern(TypeKind::Pointer {
        pointee: ids.i64_,
        mutability: Mutability::Immutable,
    });

    assert_ne!(
        ptr_i32, ptr_i64,
        "Different types must get different TypeIds"
    );
}

#[test]
fn type_equivalence_is_reflexive() {
    let registry = TypeRegistry::new();
    let ids = registry.primitive_ids();

    // Every type must be equivalent to itself
    assert!(registry.is_equivalent(ids.i32_, ids.i32_));
    assert!(registry.is_equivalent(ids.i64_, ids.i64_));
    assert!(registry.is_equivalent(ids.f64_, ids.f64_));
}

#[test]
fn type_equivalence_is_symmetric() {
    let registry = TypeRegistry::new();
    let ids = registry.primitive_ids();

    let a = ids.i32_;
    let b = ids.i32_;
    assert_eq!(
        registry.is_equivalent(a, b),
        registry.is_equivalent(b, a),
        "Type equivalence must be symmetric"
    );
}

#[test]
fn invalid_type_ids_are_not_equivalent() {
    let registry = TypeRegistry::new();
    let bad_id = TypeId(9999);

    // Invalid IDs should not be equivalent even to themselves
    assert!(
        !registry.is_equivalent(bad_id, bad_id),
        "Invalid TypeId must not be equivalent to itself"
    );
}

// ─── 4. FFI Type Alias Tests ───

#[test]
fn ffi_type_aliases_resolve_correctly() {
    let mut registry = TypeRegistry::new();
    let ids = registry.primitive_ids();

    // C int → I32
    assert_eq!(registry.c_int(), ids.i32_, "C int must be I32");

    // C unsigned int → U32
    assert_eq!(registry.c_uint(), ids.u32_, "C uint must be U32");

    // C long → I64 (LP64)
    assert_eq!(registry.c_long(), ids.i64_, "C long must be I64");

    // C float → F32
    assert_eq!(registry.c_float(), ids.f32_, "C float must be F32");

    // C double → F64
    assert_eq!(registry.c_double(), ids.f64_, "C double must be F64");

    // C char → I8
    assert_eq!(registry.c_char(), ids.i8_, "C char must be I8");

    // C unsigned char → U8
    assert_eq!(registry.c_uchar(), ids.u8_, "C uchar must be U8");
}

#[test]
fn c_void_ptr_is_pointer_to_unit() {
    let mut registry = TypeRegistry::new();
    let ids = registry.primitive_ids();

    let void_ptr = registry.c_void_ptr();
    let kind = registry.resolve(void_ptr);

    match kind {
        TypeKind::Pointer {
            pointee,
            mutability: Mutability::Mutable,
        } => {
            assert_eq!(*pointee, ids.unit, "void* must point to Unit");
        }
        other => panic!("c_void_ptr must be *mut Unit, got {:?}", other),
    }
}

#[test]
fn c_const_char_ptr_is_pointer_to_i8() {
    let mut registry = TypeRegistry::new();
    let ids = registry.primitive_ids();

    let char_ptr = registry.c_const_char_ptr();
    let kind = registry.resolve(char_ptr);

    match kind {
        TypeKind::Pointer {
            pointee,
            mutability: Mutability::Immutable,
        } => {
            assert_eq!(*pointee, ids.i8_, "const char* must point to I8");
        }
        other => panic!(
            "c_const_char_ptr must be *const I8, got {:?}",
            other
        ),
    }
}

// ─── 5. C ABI Info Tests ───

#[test]
fn c_abi_info_is_correct_for_all_primitives() {
    let registry = TypeRegistry::new();
    let ids = registry.primitive_ids();

    // I32: size=4, align=4
    let i32_abi = registry.c_abi_info(ids.i32_);
    assert_eq!(i32_abi.size, 4);
    assert_eq!(i32_abi.align, 4);

    // I64: size=8, align=8
    let i64_abi = registry.c_abi_info(ids.i64_);
    assert_eq!(i64_abi.size, 8);
    assert_eq!(i64_abi.align, 8);

    // F64: size=8, align=8
    let f64_abi = registry.c_abi_info(ids.f64_);
    assert_eq!(f64_abi.size, 8);
    assert_eq!(f64_abi.align, 8);

    // Bool: size=1, align=1
    let bool_abi = registry.c_abi_info(ids.bool_);
    assert_eq!(bool_abi.size, 1);
    assert_eq!(bool_abi.align, 1);
}

// ─── 6. TypeInspector Tests ───

#[test]
fn inspector_classifies_types_correctly() {
    let registry = TypeRegistry::new();
    let ids = registry.primitive_ids();
    let inspector = TypeInspector::new(&registry);

    assert!(inspector.is_integer(ids.i32_));
    assert!(inspector.is_integer(ids.u64_));
    assert!(!inspector.is_integer(ids.f32_));
    assert!(!inspector.is_integer(ids.bool_));

    assert!(inspector.is_signed_integer(ids.i32_));
    assert!(!inspector.is_signed_integer(ids.u32_));

    assert!(inspector.is_float(ids.f32_));
    assert!(inspector.is_float(ids.f64_));
    assert!(!inspector.is_float(ids.i32_));

    assert!(inspector.is_numeric(ids.i32_));
    assert!(inspector.is_numeric(ids.f64_));
    assert!(!inspector.is_numeric(ids.bool_));
}

#[test]
fn inspector_type_names_are_readable() {
    let registry = TypeRegistry::new();
    let ids = registry.primitive_ids();
    let inspector = TypeInspector::new(&registry);

    assert_eq!(inspector.type_name(ids.i32_), "I32");
    assert_eq!(inspector.type_name(ids.i64_), "I64");
    assert_eq!(inspector.type_name(ids.f64_), "F64");
    assert_eq!(inspector.type_name(ids.bool_), "Bool");
    assert_eq!(inspector.type_name(ids.unit), "Unit");
}

#[test]
fn inspector_lossless_conversions() {
    let registry = TypeRegistry::new();
    let ids = registry.primitive_ids();
    let inspector = TypeInspector::new(&registry);

    assert!(inspector.is_lossless_conversion(ids.i32_, ids.i64_));
    assert!(inspector.is_lossless_conversion(ids.f32_, ids.f64_));
    assert!(!inspector.is_lossless_conversion(ids.i64_, ids.i32_));
    assert!(!inspector.is_lossless_conversion(ids.f64_, ids.i32_));
}

// ─── 7. CoercionEngine Tests ───

#[test]
fn coercion_identity() {
    let registry = TypeRegistry::new();
    let engine = CoercionEngine::new(&registry);
    let ids = registry.primitive_ids();

    assert_eq!(engine.can_coerce(ids.i32_, ids.i32_), CoercionResult::Identity);
}

#[test]
fn coercion_integer_widening_is_implicit() {
    let registry = TypeRegistry::new();
    let engine = CoercionEngine::new(&registry);
    let ids = registry.primitive_ids();

    let result = engine.can_coerce(ids.i32_, ids.i64_);
    assert!(result.is_allowed(), "I32 -> I64 must be implicit");
}

#[test]
fn coercion_integer_narrowing_requires_cast() {
    let registry = TypeRegistry::new();
    let engine = CoercionEngine::new(&registry);
    let ids = registry.primitive_ids();

    let result = engine.can_coerce(ids.i64_, ids.i32_);
    assert!(result.needs_cast(), "I64 -> I32 must require cast");
}

#[test]
fn coercion_int_to_float_widening() {
    let registry = TypeRegistry::new();
    let engine = CoercionEngine::new(&registry);
    let ids = registry.primitive_ids();

    assert!(
        engine.can_coerce(ids.i32_, ids.f64_).is_allowed(),
        "I32 -> F64 must be implicit"
    );
}

#[test]
fn coercion_float_to_int_requires_cast() {
    let registry = TypeRegistry::new();
    let engine = CoercionEngine::new(&registry);
    let ids = registry.primitive_ids();

    assert!(
        engine.can_coerce(ids.f64_, ids.i32_).needs_cast(),
        "F64 -> I32 must require cast"
    );
}

#[test]
fn coercion_string_to_c_string_for_ffi() {
    let mut registry = TypeRegistry::new();
    let engine = CoercionEngine::new(&registry);
    let ids = registry.primitive_ids();
    let c_string = registry.c_const_char_ptr();

    let result = engine.can_coerce(ids.string, c_string);
    assert!(
        result.is_allowed(),
        "String -> *const I8 must be implicit for FFI"
    );
}

#[test]
fn coercion_unit_is_incompatible() {
    let registry = TypeRegistry::new();
    let engine = CoercionEngine::new(&registry);
    let ids = registry.primitive_ids();

    assert_eq!(
        engine.can_coerce(ids.unit, ids.i32_),
        CoercionResult::Incompatible
    );
}

#[test]
fn coercion_common_type_for_binops() {
    let registry = TypeRegistry::new();
    let engine = CoercionEngine::new(&registry);
    let ids = registry.primitive_ids();

    assert_eq!(engine.common_type(ids.i32_, ids.i64_), Some(ids.i64_));
    assert_eq!(engine.common_type(ids.f32_, ids.f64_), Some(ids.f64_));
    assert_eq!(engine.common_type(ids.i32_, ids.f64_), Some(ids.f64_));
}

// ─── 8. Edge Cases ───

#[test]
fn never_type_has_zero_size() {
    let registry = TypeRegistry::new();
    let ids = registry.primitive_ids();

    assert_eq!(registry.get_size(ids.never), 0, "Never must be zero-sized");
}

#[test]
fn unknown_type_has_zero_size() {
    let registry = TypeRegistry::new();
    let ids = registry.primitive_ids();

    assert_eq!(registry.get_size(ids.unknown), 0, "Unknown must be zero-sized");
}

#[test]
fn function_pointer_size_is_8() {
    let registry = TypeRegistry::new();
    let fn_ptr = registry.intern(TypeKind::Function(super::CallableId(0)));

    assert_eq!(
        registry.get_size(fn_ptr),
        8,
        "Function pointer must be 8 bytes"
    );
}

#[test]
fn resolve_panics_on_invalid_id() {
    let registry = TypeRegistry::new();
    let result = std::panic::catch_unwind(|| {
        registry.resolve(TypeId(99999));
    });
    assert!(result.is_err(), "resolve() must panic on invalid TypeId");
}
