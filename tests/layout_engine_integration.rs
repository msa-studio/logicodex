// =========================================================================
// Logicodex v1.30 — Sprint 2: LayoutEngine Integration Tests
// "Struct Memory Layout — The Foundation of FFI Struct Passing"
//
// Tests that:
//   1. TypeRegistry caches struct layouts and resolves sizes
//   2. LayoutEngine computes correct C ABI layouts
//   3. Raylib struct types (Color, Vector2, Rectangle, Texture2D) are registered
//   4. get_size/get_align return correct values for struct types
// =========================================================================

use logicodex::ffi::raylib::{register_raylib_types, RaylibTypeIds};
use logicodex::layout::{LayoutEngine, StructField, TargetLayout};
use logicodex::types::{StructLayout, TypeKind, TypeRegistry};

// ─── 1. LayoutEngine Basic Functionality ───

#[test]
fn layout_engine_computes_color_correctly() {
    let mut registry = TypeRegistry::new();
    let target = TargetLayout::native();
    let engine = LayoutEngine::new(&registry, target);

    let ids = registry.primitive_ids();
    let layout = engine
        .compute_struct_layout(
            "Color",
            &[
                StructField { name: "r".into(), ty: ids.u8_ },
                StructField { name: "g".into(), ty: ids.u8_ },
                StructField { name: "b".into(), ty: ids.u8_ },
                StructField { name: "a".into(), ty: ids.u8_ },
            ],
            false,
        )
        .expect("Color layout must compute");

    // Color { r: u8, g: u8, b: u8, a: u8 } = 4 bytes
    assert_eq!(layout.total_size_bytes, 4, "Color size must be 4 bytes");
    assert_eq!(layout.fields.len(), 4, "Color must have 4 fields");

    // Each field should be 1 byte, consecutive offsets
    for (i, field) in layout.fields.iter().enumerate() {
        assert_eq!(field.size_bytes, 1, "Color.{} size must be 1", field.name);
        assert_eq!(field.offset_bytes, i, "Color.{} offset must be {}", field.name, i);
    }
}

#[test]
fn layout_engine_computes_vector2_correctly() {
    let mut registry = TypeRegistry::new();
    let target = TargetLayout::native();
    let engine = LayoutEngine::new(&registry, target);

    let ids = registry.primitive_ids();
    let layout = engine
        .compute_struct_layout(
            "Vector2",
            &[
                StructField { name: "x".into(), ty: ids.f32_ },
                StructField { name: "y".into(), ty: ids.f32_ },
            ],
            false,
        )
        .expect("Vector2 layout must compute");

    // Vector2 { x: f32, y: f32 } = 8 bytes, align: 4
    assert_eq!(layout.total_size_bytes, 8, "Vector2 size must be 8 bytes");
    assert_eq!(layout.alignment_bytes, 4, "Vector2 align must be 4");

    assert_eq!(layout.fields[0].offset_bytes, 0, "Vector2.x offset");
    assert_eq!(layout.fields[1].offset_bytes, 4, "Vector2.y offset");
}

#[test]
fn layout_engine_computes_texture2d_correctly() {
    let mut registry = TypeRegistry::new();
    let target = TargetLayout::native();
    let engine = LayoutEngine::new(&registry, target);

    let ids = registry.primitive_ids();
    let layout = engine
        .compute_struct_layout(
            "Texture2D",
            &[
                StructField { name: "id".into(), ty: ids.u32_ },
                StructField { name: "width".into(), ty: ids.i32_ },
                StructField { name: "height".into(), ty: ids.i32_ },
                StructField { name: "mipmaps".into(), ty: ids.i32_ },
                StructField { name: "format".into(), ty: ids.i32_ },
            ],
            false,
        )
        .expect("Texture2D layout must compute");

    // Texture2D { id: u32, w: i32, h: i32, mipmaps: i32, format: i32 } = 20 bytes
    assert_eq!(
        layout.total_size_bytes, 20,
        "Texture2D size must be 20 bytes"
    );
    assert_eq!(layout.alignment_bytes, 4, "Texture2D align must be 4");
    assert_eq!(layout.fields.len(), 5, "Texture2D must have 5 fields");
}

#[test]
fn layout_engine_padded_struct_has_correct_offsets() {
    // struct Padded { a: u8, b: u32 } → size: 8 (pad 3 bytes), align: 4
    let mut registry = TypeRegistry::new();
    let target = TargetLayout::native();
    let engine = LayoutEngine::new(&registry, target);

    let ids = registry.primitive_ids();
    let layout = engine
        .compute_struct_layout(
            "Padded",
            &[
                StructField { name: "a".into(), ty: ids.u8_ },
                StructField { name: "b".into(), ty: ids.u32_ },
            ],
            false,
        )
        .expect("Padded layout must compute");

    // a at offset 0, b at offset 4 (after 3 bytes padding)
    assert_eq!(layout.fields[0].offset_bytes, 0, "Padded.a offset");
    assert_eq!(layout.fields[1].offset_bytes, 4, "Padded.b offset");
    assert_eq!(layout.total_size_bytes, 8, "Padded size with padding");
}

// ─── 2. TypeRegistry Struct Cache ───

#[test]
fn type_registry_caches_struct_layout() {
    let mut registry = TypeRegistry::new();

    // Manually create a layout and register it
    let layout = StructLayout {
        name: "TestPoint".into(),
        fields: vec![],
        total_size_bytes: 8,
        alignment_bytes: 4,
        is_packed: false,
    };

    let id = registry.intern_struct(layout.clone());

    // Should be retrievable
    let cached = registry
        .get_struct_layout(registry.resolve(id).unwrap_struct())
        .expect("Layout must be cached");
    assert_eq!(cached.name, "TestPoint");
    assert_eq!(cached.total_size_bytes, 8);
}

#[test]
fn type_registry_get_size_for_struct() {
    let mut registry = TypeRegistry::new();

    let layout = StructLayout {
        name: "Simple".into(),
        fields: vec![],
        total_size_bytes: 16,
        alignment_bytes: 8,
        is_packed: false,
    };

    let id = registry.intern_struct(layout);

    // get_size and get_align should work for registered struct
    assert_eq!(registry.get_size(id), 16);
    assert_eq!(registry.get_align(id), 8);
}

#[test]
fn type_registry_find_struct_by_name() {
    let mut registry = TypeRegistry::new();

    let layout = StructLayout {
        name: "Findable".into(),
        fields: vec![],
        total_size_bytes: 4,
        alignment_bytes: 4,
        is_packed: false,
    };

    registry.intern_struct(layout);

    let (layout_id, cached) = registry
        .find_struct_by_name("Findable")
        .expect("Must find struct by name");

    assert_eq!(cached.name, "Findable");
    assert_eq!(cached.total_size_bytes, 4);
}

// ─── 3. Raylib Type Registration ───

#[test]
fn raylib_color_type_registered() {
    let mut registry = TypeRegistry::new();
    let (raylib_ids, _) = register_raylib_types(&mut registry);

    // Color should resolve to a Struct kind
    match registry.resolve(raylib_ids.color) {
        TypeKind::Struct(layout_id) => {
            let layout = registry
                .get_struct_layout(layout_id)
                .expect("Color layout must be cached");
            assert_eq!(layout.name, "Color");
            assert_eq!(layout.total_size_bytes, 4, "Color size must be 4");
            assert_eq!(layout.fields.len(), 4);
        }
        other => panic!("Color must be TypeKind::Struct, got {:?}", other),
    }
}

#[test]
fn raylib_vector2_type_registered() {
    let mut registry = TypeRegistry::new();
    let (raylib_ids, _) = register_raylib_types(&mut registry);

    let size = registry.get_size(raylib_ids.vector2);
    let align = registry.get_align(raylib_ids.vector2);

    assert_eq!(size, 8, "Vector2 size must be 8");
    assert_eq!(align, 4, "Vector2 align must be 4");
}

#[test]
fn raylib_texture2d_type_registered() {
    let mut registry = TypeRegistry::new();
    let (raylib_ids, _) = register_raylib_types(&mut registry);

    let size = registry.get_size(raylib_ids.texture2d);
    let align = registry.get_align(raylib_ids.texture2d);

    assert_eq!(size, 20, "Texture2D size must be 20");
    assert_eq!(align, 4, "Texture2D align must be 4");
}

#[test]
fn raylib_rectangle_type_registered() {
    let mut registry = TypeRegistry::new();
    let (raylib_ids, _) = register_raylib_types(&mut registry);

    let size = registry.get_size(raylib_ids.rectangle);
    let align = registry.get_align(raylib_ids.rectangle);

    assert_eq!(size, 16, "Rectangle size must be 16");
    assert_eq!(align, 4, "Rectangle align must be 4");
}

#[test]
fn raylib_all_types_findable_by_name() {
    let mut registry = TypeRegistry::new();
    let _ = register_raylib_types(&mut registry);

    assert!(
        registry.find_struct_by_name("Color").is_some(),
        "Color must be findable"
    );
    assert!(
        registry.find_struct_by_name("Vector2").is_some(),
        "Vector2 must be findable"
    );
    assert!(
        registry.find_struct_by_name("Rectangle").is_some(),
        "Rectangle must be findable"
    );
    assert!(
        registry.find_struct_by_name("Texture2D").is_some(),
        "Texture2D must be findable"
    );
}

// ─── 4. LayoutEngine via TypeRegistry Integration ───

#[test]
fn layout_engine_uses_cached_struct_for_nested() {
    // Register a Point struct, then use it in a Line struct
    let mut registry = TypeRegistry::new();
    let target = TargetLayout::native();

    // Register Point { x: f32, y: f32 } — same as Vector2
    let ids = registry.primitive_ids();
    let engine = LayoutEngine::new(&registry, target);

    let point_layout = engine
        .compute_struct_layout(
            "Point",
            &[
                StructField { name: "x".into(), ty: ids.f32_ },
                StructField { name: "y".into(), ty: ids.f32_ },
            ],
            false,
        )
        .expect("Point layout must compute");
    let point_id = registry.intern_struct(point_layout);

    // Now use Point inside Line { start: Point, end: Point }
    let engine = LayoutEngine::new(&registry, target); // refresh engine
    let line_layout = engine
        .compute_struct_layout(
            "Line",
            &[
                StructField { name: "start".into(), ty: point_id },
                StructField { name: "end".into(), ty: point_id },
            ],
            false,
        )
        .expect("Line layout must compute");

    // Line { Point (8 bytes), Point (8 bytes) } = 16 bytes
    assert_eq!(line_layout.total_size_bytes, 16, "Line size");
    assert_eq!(line_layout.fields[0].offset_bytes, 0, "Line.start offset");
    assert_eq!(line_layout.fields[1].offset_bytes, 8, "Line.end offset");
}
