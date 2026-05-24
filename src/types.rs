#![allow(dead_code)]

// =========================================================================
// Logicodex v1.30 architecture simulation: registry-backed type identities.
//
// This module is intentionally dormant. It must not replace the current
// v1.21-alpha string/enum based semantic checks until the staged v1.30 roadmap
// explicitly activates TypeRegistry integration.
// =========================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StructLayoutId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EnumLayoutId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CallableId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeRef {
    pub id: TypeId,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeKind {
    Primitive(PrimitiveType),
    Pointer {
        pointee: TypeId,
        mutability: Mutability,
    },
    Struct(StructLayoutId),
    Enum(EnumLayoutId),
    Array {
        element: TypeId,
        len: usize,
    },
    Function(CallableId),
    Never,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrimitiveType {
    Bool,
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
    String,
    Unit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Mutability {
    Immutable,
    Mutable,
}

#[derive(Debug, Clone)]
pub struct TypeRegistry {
    kinds: Vec<TypeKind>,
    primitive_cache: PrimitiveTypeIds,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PrimitiveTypeIds {
    pub bool_: TypeId,
    pub i8_: TypeId,
    pub i16_: TypeId,
    pub i32_: TypeId,
    pub i64_: TypeId,
    pub u8_: TypeId,
    pub u16_: TypeId,
    pub u32_: TypeId,
    pub u64_: TypeId,
    pub f32_: TypeId,
    pub f64_: TypeId,
    pub string: TypeId,
    pub unit: TypeId,
    pub never: TypeId,
    pub unknown: TypeId,
}

impl Default for TypeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeRegistry {
    pub fn new() -> Self {
        let mut kinds = Vec::new();
        let bool_ = push_kind(&mut kinds, TypeKind::Primitive(PrimitiveType::Bool));
        let i8_ = push_kind(&mut kinds, TypeKind::Primitive(PrimitiveType::I8));
        let i16_ = push_kind(&mut kinds, TypeKind::Primitive(PrimitiveType::I16));
        let i32_ = push_kind(&mut kinds, TypeKind::Primitive(PrimitiveType::I32));
        let i64_ = push_kind(&mut kinds, TypeKind::Primitive(PrimitiveType::I64));
        let u8_ = push_kind(&mut kinds, TypeKind::Primitive(PrimitiveType::U8));
        let u16_ = push_kind(&mut kinds, TypeKind::Primitive(PrimitiveType::U16));
        let u32_ = push_kind(&mut kinds, TypeKind::Primitive(PrimitiveType::U32));
        let u64_ = push_kind(&mut kinds, TypeKind::Primitive(PrimitiveType::U64));
        let f32_ = push_kind(&mut kinds, TypeKind::Primitive(PrimitiveType::F32));
        let f64_ = push_kind(&mut kinds, TypeKind::Primitive(PrimitiveType::F64));
        let string = push_kind(&mut kinds, TypeKind::Primitive(PrimitiveType::String));
        let unit = push_kind(&mut kinds, TypeKind::Primitive(PrimitiveType::Unit));
        let never = push_kind(&mut kinds, TypeKind::Never);
        let unknown = push_kind(&mut kinds, TypeKind::Unknown);

        Self {
            kinds,
            primitive_cache: PrimitiveTypeIds {
                bool_,
                i8_,
                i16_,
                i32_,
                i64_,
                u8_,
                u16_,
                u32_,
                u64_,
                f32_,
                f64_,
                string,
                unit,
                never,
                unknown,
            },
        }
    }

    pub fn intern(&mut self, kind: TypeKind) -> TypeId {
        if let Some((index, _)) = self
            .kinds
            .iter()
            .enumerate()
            .find(|(_, existing)| **existing == kind)
        {
            return TypeId(index as u32);
        }
        push_kind(&mut self.kinds, kind)
    }

    pub fn get(&self, id: TypeId) -> Option<&TypeKind> {
        self.kinds.get(id.0 as usize)
    }

    pub fn is_equivalent(&self, left: TypeId, right: TypeId) -> bool {
        left == right && self.get(left).is_some()
    }

    pub fn primitive_ids(&self) -> PrimitiveTypeIds {
        self.primitive_cache
    }

    pub fn primitive(&self, primitive: PrimitiveType) -> TypeId {
        match primitive {
            PrimitiveType::Bool => self.primitive_cache.bool_,
            PrimitiveType::I8 => self.primitive_cache.i8_,
            PrimitiveType::I16 => self.primitive_cache.i16_,
            PrimitiveType::I32 => self.primitive_cache.i32_,
            PrimitiveType::I64 => self.primitive_cache.i64_,
            PrimitiveType::U8 => self.primitive_cache.u8_,
            PrimitiveType::U16 => self.primitive_cache.u16_,
            PrimitiveType::U32 => self.primitive_cache.u32_,
            PrimitiveType::U64 => self.primitive_cache.u64_,
            PrimitiveType::F32 => self.primitive_cache.f32_,
            PrimitiveType::F64 => self.primitive_cache.f64_,
            PrimitiveType::String => self.primitive_cache.string,
            PrimitiveType::Unit => self.primitive_cache.unit,
        }
    }

    pub fn never(&self) -> TypeId {
        self.primitive_cache.never
    }

    pub fn unknown(&self) -> TypeId {
        self.primitive_cache.unknown
    }

    pub fn len(&self) -> usize {
        self.kinds.len()
    }

    pub fn is_empty(&self) -> bool {
        self.kinds.is_empty()
    }
}

fn push_kind(kinds: &mut Vec<TypeKind>, kind: TypeKind) -> TypeId {
    let id = TypeId(kinds.len() as u32);
    kinds.push(kind);
    id
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primitive_ids_are_deterministic() {
        let registry = TypeRegistry::new();
        let ids = registry.primitive_ids();

        assert_eq!(ids.bool_, TypeId(0));
        assert_eq!(ids.i8_, TypeId(1));
        assert_eq!(ids.i16_, TypeId(2));
        assert_eq!(ids.i32_, TypeId(3));
        assert_eq!(ids.i64_, TypeId(4));
        assert_eq!(ids.u8_, TypeId(5));
        assert_eq!(ids.u16_, TypeId(6));
        assert_eq!(ids.u32_, TypeId(7));
        assert_eq!(ids.u64_, TypeId(8));
        assert_eq!(ids.f32_, TypeId(9));
        assert_eq!(ids.f64_, TypeId(10));
        assert_eq!(ids.string, TypeId(11));
        assert_eq!(ids.unit, TypeId(12));
        assert_eq!(ids.never, TypeId(13));
        assert_eq!(ids.unknown, TypeId(14));
    }

    #[test]
    fn intern_deduplicates_equivalent_type_kinds() {
        let mut registry = TypeRegistry::new();
        let ids = registry.primitive_ids();
        let pointer = TypeKind::Pointer {
            pointee: ids.i64_,
            mutability: Mutability::Mutable,
        };

        let first = registry.intern(pointer.clone());
        let second = registry.intern(pointer);

        assert_eq!(first, second);
        assert!(registry.is_equivalent(first, second));
    }

    #[test]
    fn compound_type_interning_is_stable() {
        let mut registry = TypeRegistry::new();
        let ids = registry.primitive_ids();
        let array = registry.intern(TypeKind::Array {
            element: ids.u8_,
            len: 16,
        });
        let function = registry.intern(TypeKind::Function(CallableId(7)));

        assert_eq!(
            registry.get(array),
            Some(&TypeKind::Array {
                element: ids.u8_,
                len: 16
            })
        );
        assert_eq!(
            registry.get(function),
            Some(&TypeKind::Function(CallableId(7)))
        );
        assert_ne!(array, function);
    }

    #[test]
    fn invalid_type_ids_are_not_equivalent() {
        let registry = TypeRegistry::new();
        assert_eq!(registry.get(TypeId(999)), None);
        assert!(!registry.is_equivalent(TypeId(999), TypeId(999)));
    }
}
