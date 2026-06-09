#![allow(dead_code)]

// =========================================================================
// Logicodex v1.30 architecture simulation: struct and enum layout contracts.
//
// This module is dormant and must not perform backend layout inference for the
// current v1.21-alpha executable subset.
// =========================================================================

use crate::span::{Diagnostic, DiagnosticCode, Severity, Span};
use crate::types::{
    PrimitiveType, StructFieldLayout, StructLayout, TypeId, TypeKind, TypeRegistry,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayoutRequest {
    pub name: String,
    pub fields: Vec<LayoutFieldRequest>,
    pub attributes: Vec<LayoutAttribute>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayoutFieldRequest {
    pub name: String,
    pub ty: TypeId,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutAttribute {
    Packed,
    ReprC,
}

pub struct LayoutEngine<'a> {
    pub types: &'a TypeRegistry,
    pub target: TargetLayout,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Endian {
    Little,
    Big,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetLayout {
    pub pointer_size_bytes: usize,
    pub pointer_alignment_bytes: usize,
    pub integer_alignment_bytes: usize,
}

impl Default for TargetLayout {
    fn default() -> Self {
        Self {
            pointer_size_bytes: 8,
            pointer_alignment_bytes: 8,
            integer_alignment_bytes: 8,
        }
    }
}

impl TargetLayout {
    /// Native target layout (64-bit little-endian)
    pub fn native() -> Self {
        Self {
            pointer_size_bytes: 8,
            pointer_alignment_bytes: 8,
            integer_alignment_bytes: 8,
        }
    }

    /// Create with custom pointer size
    pub fn with_pointer_size(size: usize) -> Self {
        Self {
            pointer_size_bytes: size,
            pointer_alignment_bytes: size,
            integer_alignment_bytes: size,
        }
    }
}

impl<'a> LayoutEngine<'a> {
    /// Create a new LayoutEngine
    pub fn new(types: &'a crate::types::TypeRegistry, _target: &TargetLayout) -> Self {
        Self {
            types,
            target: TargetLayout::native(),
        }
    }

    /// Create with native target (placeholder)
    pub fn native(types: &'a crate::types::TypeRegistry) -> Self {
        Self {
            types,
            target: TargetLayout::native(),
        }
    }

    pub fn compute_struct_layout(
        &self,
        request: LayoutRequest,
    ) -> Result<StructLayout, Diagnostic> {
        let is_packed = request.attributes.contains(&LayoutAttribute::Packed);
        let mut offset = 0usize;
        let mut max_alignment = 1usize;
        let mut fields = Vec::with_capacity(request.fields.len());

        for field in request.fields {
            let (size_bytes, natural_alignment) = self.size_and_align(field.ty, field.span)?;
            let field_alignment = if is_packed { 1 } else { natural_alignment };
            offset = Self::align_to(offset, field_alignment);
            fields.push(StructFieldLayout {
                name: field.name,
                ty: field.ty,
                offset_bytes: offset,
                size_bytes,
                alignment_bytes: field_alignment,
            });
            offset += size_bytes;
            max_alignment = max_alignment.max(field_alignment);
        }

        Ok(StructLayout {
            name: request.name,
            fields,
            total_size_bytes: Self::align_to(offset, max_alignment),
            alignment_bytes: max_alignment,
            is_packed,
        })
    }

    fn size_and_align(&self, ty: TypeId, span: Span) -> Result<(usize, usize), Diagnostic> {
        match self.types.get(ty) {
            Some(TypeKind::Primitive(primitive)) => Ok(self.primitive_size_and_align(*primitive)),
            Some(TypeKind::Pointer { .. }) | Some(TypeKind::Function(_)) => Ok((
                self.target.pointer_size_bytes,
                self.target.pointer_alignment_bytes,
            )),
            Some(TypeKind::Array { element, len }) => {
                let (element_size, element_align) = self.size_and_align(*element, span)?;
                Ok((element_size.saturating_mul(*len), element_align))
            }
            Some(TypeKind::Struct(layout_id)) => {
                // Lookup cached layout from TypeRegistry
                match self.types.get_struct_layout(*layout_id) {
                    Some(layout) => {
                        Ok((layout.total_size_bytes, layout.alignment_bytes))
                    }
                    None => Err(layout_error(
                        span,
                        format!(
                            "Ralat: StructLayoutId({}) tidak ditemui dalam cache",
                            layout_id.0
                        ),
                        format!(
                            "Error: StructLayoutId({}) not found in layout cache. \
                             Register struct with intern_struct() first.",
                            layout_id.0
                        ),
                    )),
                }
            }
            Some(TypeKind::Enum(_layout_id)) => {
                // Enum: tag-only (u32) layout for now.
                Ok((4, self.target.integer_alignment_bytes.min(4)))
            }
            Some(TypeKind::Never) | Some(TypeKind::Unknown) | None => Err(layout_error(
                span,
                format!("Ralat: Jenis '{ty:?}' belum mempunyai layout memori yang sah"),
                format!("Error: Type '{ty:?}' does not have a valid memory layout yet"),
            )),
        }
    }

    fn primitive_size_and_align(&self, primitive: PrimitiveType) -> (usize, usize) {
        match primitive {
            PrimitiveType::Bool | PrimitiveType::I8 | PrimitiveType::U8 => (1, 1),
            PrimitiveType::I16 | PrimitiveType::U16 => (2, 2),
            PrimitiveType::I32 | PrimitiveType::U32 | PrimitiveType::F32 => (4, 4),
            PrimitiveType::I64 | PrimitiveType::U64 | PrimitiveType::F64 => {
                (8, self.target.integer_alignment_bytes.min(8))
            }
            PrimitiveType::String => (
                self.target.pointer_size_bytes,
                self.target.pointer_alignment_bytes,
            ),
            PrimitiveType::Unit => (0, 1),
        }
    }

    fn align_to(value: usize, alignment: usize) -> usize {
        if alignment <= 1 {
            value
        } else {
            let remainder = value % alignment;
            if remainder == 0 {
                value
            } else {
                value + alignment - remainder
            }
        }
    }
}

fn layout_error(span: Span, message_ms: String, message_en: String) -> Diagnostic {
    Diagnostic {
        code: DiagnosticCode::LayoutError,
        severity: Severity::Error,
        message_ms,
        message_en,
        primary_span: span,
        notes: Vec::new(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumLayout {
    pub name: String,
    pub tag_type: TypeId,
    pub variants: Vec<EnumVariantLayout>,
    pub total_size_bytes: usize,
    pub alignment_bytes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumVariantLayout {
    pub name: String,
    pub tag_value: u64,
    pub payload: EnumPayloadLayout,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnumPayloadLayout {
    Unit,
    Tuple(Vec<TypeId>),
    Struct(Vec<StructFieldLayout>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnumReprAst {
    Default,
    U8,
    U16,
    U32,
    U64,
    I32,
    I64,
}

#[derive(Debug, Default)]
pub struct LayoutRegistry {
    pub structs: Vec<StructLayout>,
    pub enums: Vec<EnumLayout>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::span::Span;

    #[test]
    fn computes_natural_struct_layout() {
        let types = TypeRegistry::new();
        let ids = types.primitive_ids();
        let engine = LayoutEngine {
            types: &types,
            target: TargetLayout::default(),
        };

        let layout = engine
            .compute_struct_layout(LayoutRequest {
                name: "Pair".to_string(),
                fields: vec![
                    LayoutFieldRequest {
                        name: "x".to_string(),
                        ty: ids.i64_,
                        span: Span::unknown(),
                    },
                    LayoutFieldRequest {
                        name: "y".to_string(),
                        ty: ids.i64_,
                        span: Span::unknown(),
                    },
                ],
                attributes: Vec::new(),
                span: Span::unknown(),
            })
            .expect("layout should be valid");

        assert_eq!(layout.total_size_bytes, 16);
        assert_eq!(layout.alignment_bytes, 8);
        assert_eq!(layout.fields[0].offset_bytes, 0);
        assert_eq!(layout.fields[1].offset_bytes, 8);
    }

    #[test]
    fn computes_packed_struct_layout() {
        let types = TypeRegistry::new();
        let ids = types.primitive_ids();
        let engine = LayoutEngine {
            types: &types,
            target: TargetLayout::default(),
        };

        let layout = engine
            .compute_struct_layout(LayoutRequest {
                name: "Packed".to_string(),
                fields: vec![
                    LayoutFieldRequest {
                        name: "a".to_string(),
                        ty: ids.u8_,
                        span: Span::unknown(),
                    },
                    LayoutFieldRequest {
                        name: "b".to_string(),
                        ty: ids.i64_,
                        span: Span::unknown(),
                    },
                ],
                attributes: vec![LayoutAttribute::Packed],
                span: Span::unknown(),
            })
            .expect("packed layout should be valid");

        assert_eq!(layout.total_size_bytes, 9);
        assert_eq!(layout.alignment_bytes, 1);
        assert_eq!(layout.fields[1].offset_bytes, 1);
    }

    #[test]
    fn rejects_unknown_field_type() {
        let types = TypeRegistry::new();
        let engine = LayoutEngine {
            types: &types,
            target: TargetLayout::default(),
        };
        let diagnostic = engine
            .compute_struct_layout(LayoutRequest {
                name: "Bad".to_string(),
                fields: vec![LayoutFieldRequest {
                    name: "x".to_string(),
                    ty: types.unknown(),
                    span: Span::unknown(),
                }],
                attributes: Vec::new(),
                span: Span::unknown(),
            })
            .expect_err("unknown layout must fail");

        assert_eq!(diagnostic.code, DiagnosticCode::LayoutError);
        assert!(diagnostic.message_ms.contains("Ralat:"));
        assert!(diagnostic.message_en.contains("Error:"));
    }
}
