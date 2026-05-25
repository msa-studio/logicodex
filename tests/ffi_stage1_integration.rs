// =========================================================================
// Logicodex v1.30 — Stage 1 Integration Test
// "Window Opens, Circle Draws, Color Works"
//
// This test validates the complete user story:
//   1. Window management (InitWindow, CloseWindow, WindowShouldClose)
//   2. Frame timing (SetTargetFPS, GetFrameTime)
//   3. 2D drawing with Color (ClearBackground, DrawCircle, DrawText)
//   4. Keyboard input (IsKeyDown with KEY_ constants)
//   5. Math utility (clamp for boundary checking)
//
// Note: This test validates the FFI structure and registration only.
// Actual window opening requires a running Raylib library at link time.
// =========================================================================

use logicodex::ffi::math::clamp;
use logicodex::ffi::raylib_sys::Color;
use logicodex::ffi::{CallableRegistry, CallableSafety, CallingConvention};
use logicodex::types::TypeRegistry;

// ─── 1. Color as u32 Packed RGBA ───

#[test]
fn color_packed_rgba_roundtrip() {
    // Color(255, 0, 0, 255) → 0xFF0000FF
    let red = Color::from_rgba(255, 0, 0, 255);
    assert_eq!(red.to_hex(), 0xFF0000FF);

    // Color(0, 255, 0, 255) → 0x00FF00FF
    let green = Color::from_rgba(0, 255, 0, 255);
    assert_eq!(green.to_hex(), 0x00FF00FF);

    // Color(0, 0, 255, 255) → 0x0000FFFF
    let blue = Color::from_rgba(0, 0, 255, 255);
    assert_eq!(blue.to_hex(), 0x0000FFFF);

    // Color(240, 240, 240, 255) → light gray (background)
    let bg = Color::from_rgba(240, 240, 240, 255);
    assert_eq!(bg.to_hex(), 0xF0F0F0FF);
}

#[test]
fn color_size_is_4_bytes_for_u32_passing() {
    // Color must be 4 bytes so it can be passed as u32
    assert_eq!(std::mem::size_of::<Color>(), 4);
    assert_eq!(std::mem::size_of::<Color>(), std::mem::size_of::<u32>());
}

// ─── 2. Math Utilities ───

#[test]
fn clamp_keeps_value_in_range() {
    assert_eq!(clamp(5, 0, 10), 5, "in-range value stays");
    assert_eq!(clamp(-3, 0, 10), 0, "below min clamps to min");
    assert_eq!(clamp(15, 0, 10), 10, "above max clamps to max");
}

#[test]
fn clamp_for_circle_boundary() {
    // Bola di posisi (400, 300), radius 30, window 800x600
    // x = clamp(x, radius, width - radius)
    let x = 400i64;
    let radius = 30i64;
    let width = 800i64;

    // Normal position
    assert_eq!(clamp(x, radius, width - radius), 400);

    // Too far left
    assert_eq!(clamp(10, radius, width - radius), 30);

    // Too far right
    assert_eq!(clamp(790, radius, width - radius), 770);
}

#[test]
fn lerp_basic_interpolation() {
    use logicodex::ffi::math::lerp;

    // lerp(0, 10, 0.0) = 0
    assert!((lerp(0.0, 10.0, 0.0) - 0.0).abs() < 1e-6);
    // lerp(0, 10, 0.5) = 5
    assert!((lerp(0.0, 10.0, 0.5) - 5.0).abs() < 1e-6);
    // lerp(0, 10, 1.0) = 10
    assert!((lerp(0.0, 10.0, 1.0) - 10.0).abs() < 1e-6);
}

// ─── 3. FFI Function Registration ───

#[test]
fn stage1_functions_are_registered() {
    let mut registry = TypeRegistry::new();
    let mut callables = CallableRegistry::default();
    logicodex::ffi::raylib::register_raylib_functions_compat(&mut registry, &mut callables);

    // Core windowing
    assert!(
        callables.find_by_name("InitWindow").is_some(),
        "InitWindow must be registered"
    );
    assert!(
        callables.find_by_name("CloseWindow").is_some(),
        "CloseWindow must be registered"
    );
    assert!(
        callables.find_by_name("WindowShouldClose").is_some(),
        "WindowShouldClose must be registered"
    );
    assert!(
        callables.find_by_name("SetTargetFPS").is_some(),
        "SetTargetFPS must be registered"
    );
    assert!(
        callables.find_by_name("GetFrameTime").is_some(),
        "GetFrameTime must be registered"
    );

    // Drawing with Color (now registered as u32)
    assert!(
        callables.find_by_name("ClearBackground").is_some(),
        "ClearBackground must be registered"
    );
    assert!(
        callables.find_by_name("DrawCircle").is_some(),
        "DrawCircle must be registered"
    );
    assert!(
        callables.find_by_name("DrawText").is_some(),
        "DrawText must be registered"
    );

    // Input
    assert!(
        callables.find_by_name("IsKeyDown").is_some(),
        "IsKeyDown must be registered"
    );
    assert!(
        callables.find_by_name("IsKeyPressed").is_some(),
        "IsKeyPressed must be registered"
    );
}

#[test]
fn color_functions_take_u32_not_i64() {
    let mut registry = TypeRegistry::new();
    let mut callables = CallableRegistry::default();
    logicodex::ffi::raylib::register_raylib_functions_compat(&mut registry, &mut callables);

    let ids = registry.primitive_ids();

    // ClearBackground should take u32 (Color as packed RGBA)
    let (_, sig) = callables.find_by_name("ClearBackground").unwrap();
    let last_param = *sig.params.last().unwrap();
    assert_eq!(
        last_param, ids.u32_,
        "ClearBackground color param must be U32 (packed RGBA), not I64"
    );

    // DrawCircle should take u32 for color
    let (_, sig) = callables.find_by_name("DrawCircle").unwrap();
    let last_param = *sig.params.last().unwrap();
    assert_eq!(
        last_param, ids.u32_,
        "DrawCircle color param must be U32 (packed RGBA), not I64"
    );

    // DrawText should take u32 for color
    let (_, sig) = callables.find_by_name("DrawText").unwrap();
    let last_param = *sig.params.last().unwrap();
    assert_eq!(
        last_param, ids.u32_,
        "DrawText color param must be U32 (packed RGBA), not I64"
    );
}

#[test]
fn all_stage1_functions_require_unsafe() {
    let mut registry = TypeRegistry::new();
    let mut callables = CallableRegistry::default();
    logicodex::ffi::raylib::register_raylib_functions_compat(&mut registry, &mut callables);

    let stage1_functions = [
        "InitWindow",
        "CloseWindow",
        "WindowShouldClose",
        "SetTargetFPS",
        "GetFrameTime",
        "ClearBackground",
        "DrawCircle",
        "DrawText",
        "DrawRectangle",
        "DrawLine",
        "IsKeyDown",
        "IsKeyPressed",
        "GetKeyPressed",
    ];

    for name in stage1_functions {
        let (_, sig) = callables
            .find_by_name(name)
            .unwrap_or_else(|| panic!("{} not registered", name));
        assert_eq!(
            sig.safety, CallableSafety::UnsafeRequired,
            "{} must require unsafe block",
            name
        );
        assert!(
            sig.is_extern,
            "{} must be extern",
            name
        );
        assert_eq!(
            sig.abi, CallingConvention::C,
            "{} must use C ABI",
            name
        );
    }
}

// ─── 4. User Story Simulation ───

#[test]
fn user_story_types_check_out() {
    // Simulate: BINA x = 400 → I64 (default integer)
    let registry = TypeRegistry::new();
    let checker = logicodex::semantic::type_checker::TypeChecker::new(&registry);

    // InitWindow(800, 600, "Hello!") — args should be (I32, I32, *const I8)
    let (_, sig) = {
        let mut callables = CallableRegistry::default();
        logicodex::ffi::raylib::register_raylib_functions_compat(&mut registry.clone(), &mut callables);
        callables.find_by_name("InitWindow").unwrap()
    };
    assert_eq!(sig.params.len(), 3, "InitWindow takes 3 params");
    assert_eq!(sig.params[0], registry.c_int(), "width is I32");
    assert_eq!(sig.params[1], registry.c_int(), "height is I32");

    // SetTargetFPS(60) — arg should be I32
    let (_, sig) = {
        let mut callables = CallableRegistry::default();
        logicodex::ffi::raylib::register_raylib_functions_compat(&mut registry.clone(), &mut callables);
        callables.find_by_name("SetTargetFPS").unwrap()
    };
    assert_eq!(sig.params, &[registry.c_int()]);

    // IsKeyDown(KEY_LEFT) → Bool
    let (_, sig) = {
        let mut callables = CallableRegistry::default();
        logicodex::ffi::raylib::register_raylib_functions_compat(&mut registry.clone(), &mut callables);
        callables.find_by_name("IsKeyDown").unwrap()
    };
    assert_eq!(sig.params, &[registry.c_int()], "key code is I32");
    assert_eq!(sig.return_type, registry.primitive(logicodex::types::PrimitiveType::Bool));
}

// ─── 5. Math Module is Available ───

#[test]
fn math_module_exports_clamp() {
    // clamp should be callable from Logicodex FFI
    let result = clamp(400i64, 30i64, 770i64);
    assert_eq!(result, 400);
}

#[test]
fn math_module_exports_lerp() {
    use logicodex::ffi::math::lerp;
    let result = lerp(0.0, 100.0, 0.5);
    assert!((result - 50.0).abs() < 1e-6);
}
