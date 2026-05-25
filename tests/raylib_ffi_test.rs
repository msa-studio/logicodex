// =========================================================================
// Logicodex v1.30 — Raylib FFI Integration Tests
// Sprint 1: Type System Foundation — FFI Layer
//
// These tests verify:
//   - Raylib C types have correct sizes and alignments for FFI
//   - Color hex packing/unpacking roundtrips correctly
//   - CallableRegistry registration works for all 28 functions
//   - Function signatures have correct parameter counts
// =========================================================================

use logicodex::ffi::raylib::{self, Color, Texture2D, Vector2};
use logicodex::ffi::{CallableRegistry, CallableSafety, CallingConvention};
use logicodex::types::TypeRegistry;

// ─── 1. C Type Layout Tests ───

#[test]
fn color_layout_is_4_bytes_rgba() {
    // Color must be exactly 4 bytes for C ABI compatibility
    assert_eq!(
        std::mem::size_of::<Color>(),
        4,
        "Color size must be 4 bytes (R,G,B,A as u8)"
    );
    assert_eq!(
        std::mem::align_of::<Color>(),
        1,
        "Color alignment must be 1 (packed)"
    );
}

#[test]
fn color_hex_roundtrip_all_channels() {
    // Test that hex encoding/decoding preserves all RGBA channels
    let colors = [
        (Color::from_hex(0xFF0000FF), 255, 0, 0, 255, "Red"),
        (Color::from_hex(0x00FF00FF), 0, 255, 0, 255, "Green"),
        (Color::from_hex(0x0000FFFF), 0, 0, 255, 255, "Blue"),
        (Color::from_hex(0xFFFFFFFF), 255, 255, 255, 255, "White"),
        (Color::from_hex(0x000000FF), 0, 0, 0, 255, "Black"),
        (Color::from_hex(0xFF00FF00), 255, 0, 255, 0, "Magenta transparent"),
        (Color::from_hex(0x80808080), 128, 128, 128, 128, "Gray semi-transparent"),
    ];

    for (color, r, g, b, a, name) in colors {
        assert_eq!(color.r, r, "{}: red channel mismatch", name);
        assert_eq!(color.g, g, "{}: green channel mismatch", name);
        assert_eq!(color.b, b, "{}: blue channel mismatch", name);
        assert_eq!(color.a, a, "{}: alpha channel mismatch", name);

        // Roundtrip
        let hex = color.to_hex();
        let roundtrip = Color::from_hex(hex);
        assert_eq!(color.r, roundtrip.r, "{}: roundtrip red failed", name);
        assert_eq!(color.g, roundtrip.g, "{}: roundtrip green failed", name);
        assert_eq!(color.b, roundtrip.b, "{}: roundtrip blue failed", name);
        assert_eq!(color.a, roundtrip.a, "{}: roundtrip alpha failed", name);
    }
}

#[test]
fn color_predefined_constants_are_correct() {
    assert_eq!(Color::RED.to_hex(), 0xFF0000FF, "RED hex mismatch");
    assert_eq!(Color::GREEN.to_hex(), 0x00FF00FF, "GREEN hex mismatch");
    assert_eq!(Color::BLUE.to_hex(), 0x0000FFFF, "BLUE hex mismatch");
    assert_eq!(Color::WHITE.to_hex(), 0xFFFFFFFF, "WHITE hex mismatch");
    assert_eq!(Color::BLACK.to_hex(), 0x000000FF, "BLACK hex mismatch");
    assert_eq!(Color::YELLOW.to_hex(), 0xFDF900FF, "YELLOW hex mismatch");
    assert_eq!(Color::ORANGE.to_hex(), 0xFFA100FF, "ORANGE hex mismatch");
    assert_eq!(Color::PINK.to_hex(), 0xFF6DC2FF, "PINK hex mismatch");
}

#[test]
fn color_from_rgba_constructor() {
    let color = Color::from_rgba(10, 20, 30, 40);
    assert_eq!(color.r, 10);
    assert_eq!(color.g, 20);
    assert_eq!(color.b, 30);
    assert_eq!(color.a, 40);
}

#[test]
fn vector2_layout_is_8_bytes() {
    // Vector2: x(f32), y(f32) = 8 bytes, align 4
    assert_eq!(
        std::mem::size_of::<Vector2>(),
        8,
        "Vector2 size must be 8 bytes (2×f32)"
    );
    assert_eq!(
        std::mem::align_of::<Vector2>(),
        4,
        "Vector2 alignment must be 4 (f32)"
    );
}

#[test]
fn texture2d_layout_is_20_bytes() {
    // Texture2D: id(u32) + width(i32) + height(i32) + mipmaps(i32) + format(i32) = 20 bytes
    assert_eq!(
        std::mem::size_of::<Texture2D>(),
        20,
        "Texture2D size must be 20 bytes (u32 + 4×i32)"
    );
}

// ─── 2. CallableRegistry Registration Tests ───

#[test]
fn all_core_functions_are_registered() {
    let mut registry = TypeRegistry::new();
    let mut callables = CallableRegistry::default();
    raylib::register_raylib_functions_compat(&mut registry, &mut callables);

    // Windowing functions
    assert!(callables.find_by_name("InitWindow").is_some(), "InitWindow");
    assert!(callables.find_by_name("CloseWindow").is_some(), "CloseWindow");
    assert!(
        callables.find_by_name("WindowShouldClose").is_some(),
        "WindowShouldClose"
    );
    assert!(callables.find_by_name("SetTargetFPS").is_some(), "SetTargetFPS");
    assert!(callables.find_by_name("GetFPS").is_some(), "GetFPS");
    assert!(
        callables.find_by_name("GetFrameTime").is_some(),
        "GetFrameTime"
    );
    assert!(callables.find_by_name("GetTime").is_some(), "GetTime");
    assert!(
        callables.find_by_name("GetScreenWidth").is_some(),
        "GetScreenWidth"
    );
    assert!(
        callables.find_by_name("GetScreenHeight").is_some(),
        "GetScreenHeight"
    );

    // Drawing functions
    assert!(callables.find_by_name("BeginDrawing").is_some(), "BeginDrawing");
    assert!(callables.find_by_name("EndDrawing").is_some(), "EndDrawing");
    assert!(
        callables.find_by_name("ClearBackground").is_some(),
        "ClearBackground"
    );
    assert!(callables.find_by_name("DrawText").is_some(), "DrawText");
    assert!(
        callables.find_by_name("DrawRectangle").is_some(),
        "DrawRectangle"
    );
    assert!(callables.find_by_name("DrawCircle").is_some(), "DrawCircle");
    assert!(callables.find_by_name("DrawLine").is_some(), "DrawLine");
    assert!(
        callables.find_by_name("DrawRectangleLines").is_some(),
        "DrawRectangleLines"
    );
    assert!(callables.find_by_name("DrawPixel").is_some(), "DrawPixel");

    // Texture functions
    assert!(
        callables.find_by_name("LoadTexture").is_some(),
        "LoadTexture"
    );
    assert!(
        callables.find_by_name("DrawTexture").is_some(),
        "DrawTexture"
    );
    assert!(
        callables.find_by_name("UnloadTexture").is_some(),
        "UnloadTexture"
    );

    // Input functions
    assert!(callables.find_by_name("IsKeyDown").is_some(), "IsKeyDown");
    assert!(
        callables.find_by_name("IsKeyPressed").is_some(),
        "IsKeyPressed"
    );
    assert!(
        callables.find_by_name("GetKeyPressed").is_some(),
        "GetKeyPressed"
    );
    assert!(
        callables
            .find_by_name("IsMouseButtonPressed")
            .is_some(),
        "IsMouseButtonPressed"
    );
    assert!(callables.find_by_name("GetMouseX").is_some(), "GetMouseX");
    assert!(callables.find_by_name("GetMouseY").is_some(), "GetMouseY");
}

#[test]
fn registered_functions_have_correct_signatures() {
    let mut registry = TypeRegistry::new();
    let mut callables = CallableRegistry::default();
    raylib::register_raylib_functions_compat(&mut registry, &mut callables);
    let ids = registry.primitive_ids();

    // InitWindow(width: I32, height: I32, title: *const I8) -> Unit
    let (_, sig) = callables.find_by_name("InitWindow").unwrap();
    assert_eq!(sig.params.len(), 3, "InitWindow takes 3 params");
    assert_eq!(sig.params[0], ids.i32_, "InitWindow param 0 is I32");
    assert_eq!(sig.params[1], ids.i32_, "InitWindow param 1 is I32");
    assert_eq!(sig.return_type, ids.unit, "InitWindow returns Unit");

    // WindowShouldClose() -> Bool
    let (_, sig) = callables.find_by_name("WindowShouldClose").unwrap();
    assert!(sig.params.is_empty(), "WindowShouldClose takes no params");
    assert_eq!(sig.return_type, ids.bool_, "WindowShouldClose returns Bool");

    // GetFrameTime() -> F32
    let (_, sig) = callables.find_by_name("GetFrameTime").unwrap();
    assert!(sig.params.is_empty());
    assert_eq!(sig.return_type, ids.f32_, "GetFrameTime returns F32");

    // GetTime() -> F64
    let (_, sig) = callables.find_by_name("GetTime").unwrap();
    assert!(sig.params.is_empty());
    assert_eq!(sig.return_type, ids.f64_, "GetTime returns F64");
}

#[test]
fn all_functions_require_unsafe_block() {
    let mut registry = TypeRegistry::new();
    let mut callables = CallableRegistry::default();
    raylib::register_raylib_functions_compat(&mut registry, &mut callables);

    for (_, signature) in callables.signatures.iter().enumerate() {
        assert_eq!(
            signature.safety, CallableSafety::UnsafeRequired,
            "{}: Raylib functions must require unsafe block",
            signature.name
        );
        assert!(
            signature.is_extern,
            "{}: Raylib functions must be extern",
            signature.name
        );
        assert_eq!(
            signature.abi,
            CallingConvention::C,
            "{}: Raylib functions must use C calling convention",
            signature.name
        );
    }
}

#[test]
fn init_window_params_are_i32_i32_string() {
    let mut registry = TypeRegistry::new();
    let mut callables = CallableRegistry::default();
    raylib::register_raylib_functions_compat(&mut registry, &mut callables);
    let ids = registry.primitive_ids();

    let (_, sig) = callables.find_by_name("InitWindow").unwrap();

    // Parameters should be: width (I32), height (I32), title (pointer)
    assert_eq!(sig.params[0], ids.i32_, "width param must be I32");
    assert_eq!(sig.params[1], ids.i32_, "height param must be I32");

    // Third param should be a pointer to I8 (C string)
    let title_kind = registry.resolve(sig.params[2]);
    match title_kind {
        TypeKind::Pointer {
            pointee,
            mutability: Mutability::Immutable,
        } => {
            assert_eq!(*pointee, ids.i8_, "title must be *const I8");
        }
        other => panic!("title param must be *const I8, got {:?}", other),
    }
}

#[test]
fn drawable_functions_take_color_as_last_param() {
    let mut registry = TypeRegistry::new();
    let mut callables = CallableRegistry::default();
    raylib::register_raylib_functions_compat(&mut registry, &mut callables);
    let ids = registry.primitive_ids();

    // All drawing functions should have I64 (hex color) as last param
    // Sprint 3: Change to proper Color struct type
    let drawing_fns = [
        "ClearBackground",
        "DrawRectangle",
        "DrawCircle",
        "DrawLine",
        "DrawRectangleLines",
        "DrawPixel",
    ];

    for name in drawing_fns {
        let (_, sig) = callables.find_by_name(name).unwrap();
        let last_param = *sig.params.last().unwrap_or(&ids.unit);
        assert_eq!(
            last_param, ids.i64_,
            "{}: last param must be I64 (hex color)",
            name
        );
    }
}

#[test]
fn draw_text_params_are_correct() {
    let mut registry = TypeRegistry::new();
    let mut callables = CallableRegistry::default();
    raylib::register_raylib_functions_compat(&mut registry, &mut callables);
    let ids = registry.primitive_ids();

    // DrawText(text: *const I8, posX: I32, posY: I32, fontSize: I32, color: I64)
    let (_, sig) = callables.find_by_name("DrawText").unwrap();
    assert_eq!(sig.params.len(), 5, "DrawText takes 5 params");
    assert_eq!(sig.params[1], ids.i32_, "posX is I32");
    assert_eq!(sig.params[2], ids.i32_, "posY is I32");
    assert_eq!(sig.params[3], ids.i32_, "fontSize is I32");
    assert_eq!(sig.params[4], ids.i64_, "color is I64 (hex)");
}

// ─── 3. Key Constant Tests ───

#[test]
fn keyboard_key_constants_match_raylib() {
    use logicodex::ffi::raylib::*;

    assert_eq!(KEY_NULL, 0);
    assert_eq!(KEY_A, 65);
    assert_eq!(KEY_Z, 90);
    assert_eq!(KEY_SPACE, 32);
    assert_eq!(KEY_ENTER, 257);
    assert_eq!(KEY_ESCAPE, 256);
    assert_eq!(KEY_BACKSPACE, 259);
    assert_eq!(KEY_LEFT, 263);
    assert_eq!(KEY_RIGHT, 262);
    assert_eq!(KEY_UP, 265);
    assert_eq!(KEY_DOWN, 264);
    assert_eq!(KEY_LEFT_SHIFT, 340);
    assert_eq!(KEY_LEFT_CONTROL, 341);
}

#[test]
fn mouse_button_constants_match_raylib() {
    use logicodex::ffi::raylib::*;

    assert_eq!(MOUSE_BUTTON_LEFT, 0);
    assert_eq!(MOUSE_BUTTON_RIGHT, 1);
    assert_eq!(MOUSE_BUTTON_MIDDLE, 2);
}
