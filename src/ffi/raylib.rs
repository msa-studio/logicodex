// =========================================================================
// Logicodex v1.30 — Raylib Safe Wrapper Layer
// Sprint 1: Type System Foundation — FFI Layer
//
// This module provides safe Rust wrappers around the raw Raylib FFI
// declarations in raylib_sys.rs. It also registers functions with the
// Logicodex CallableRegistry so they can be called from .ldx code.
//
// Safety: All functions require an UnsafeBlock context in Logicodex.
// =========================================================================

use super::raylib_sys;
use super::{
    CallableId, CallableRegistry, CallableSafety, CallableSignature, CallingConvention,
};
use crate::layout::{LayoutEngine, StructField};
use crate::types::{PrimitiveType, StructLayout, TypeRegistry};

/// Type IDs for Raylib struct types, populated by `register_raylib_types()`.
#[derive(Debug, Clone, Copy)]
pub struct RaylibTypeIds {
    pub color: crate::types::TypeId,
    pub vector2: crate::types::TypeId,
    pub rectangle: crate::types::TypeId,
    pub texture2d: crate::types::TypeId,
}

/// Register Raylib C struct types (Color, Vector2, Rectangle, Texture2D)
/// with the TypeRegistry. Computes layouts via LayoutEngine.
///
/// Must be called *before* `register_raylib_functions()` so that
/// function signatures can reference struct types.
/// v1.42: Register Raylib C struct types (Color, Vector2, Rectangle, Texture2D)
/// with the TypeRegistry. Computes layouts via LayoutEngine.
///
/// Must be called *before* `register_raylib_functions()` so that
/// function signatures can reference struct types.
///
/// Returns both the TypeRegistry struct type IDs (for codegen) and
/// a mapping for struct constructor detection.
pub fn register_raylib_types(registry: &mut TypeRegistry) -> (RaylibTypeIds, RaylibStructIds) {
    let ids = registry.primitive_ids();

    let target = crate::layout::TargetLayout::native();
    let engine = LayoutEngine::new(registry, target);

    // ─── Color { r: u8, g: u8, b: u8, a: u8 } — 4 bytes, align 1 ───
    let color_layout = engine
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
    let color = registry.intern_struct(color_layout);

    // ─── Vector2 { x: f32, y: f32 } — 8 bytes, align 4 ───
    let vector2_layout = engine
        .compute_struct_layout(
            "Vector2",
            &[
                StructField { name: "x".into(), ty: ids.f32_ },
                StructField { name: "y".into(), ty: ids.f32_ },
            ],
            false,
        )
        .expect("Vector2 layout must compute");
    let vector2 = registry.intern_struct(vector2_layout);

    // ─── Rectangle { x: f32, y: f32, w: f32, h: f32 } — 16 bytes, align 4 ───
    let rectangle_layout = engine
        .compute_struct_layout(
            "Rectangle",
            &[
                StructField { name: "x".into(), ty: ids.f32_ },
                StructField { name: "y".into(), ty: ids.f32_ },
                StructField { name: "width".into(), ty: ids.f32_ },
                StructField { name: "height".into(), ty: ids.f32_ },
            ],
            false,
        )
        .expect("Rectangle layout must compute");
    let rectangle = registry.intern_struct(rectangle_layout);

    // ─── Texture2D { id: u32, width: i32, height: i32, mipmaps: i32, format: i32 } — 20 bytes ───
    let texture2d_layout = engine
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
    let texture2d = registry.intern_struct(texture2d_layout);

    let type_ids = RaylibTypeIds {
        color,
        vector2,
        rectangle,
        texture2d,
    };
    let struct_ids = RaylibStructIds {
        color,
        vector2,
        rectangle,
        texture2d,
    };
    (type_ids, struct_ids)
}

pub use raylib_sys::{
    Color, Image, Rectangle, Texture2D, Vector2, Vector3, KEY_A, KEY_B, KEY_BACKSPACE,
    KEY_C, KEY_D, KEY_DELETE, KEY_DOWN, KEY_E, KEY_EIGHT, KEY_ENTER, KEY_ESCAPE, KEY_F,
    KEY_FIVE, KEY_FOUR, KEY_G, KEY_H, KEY_I, KEY_INSERT, KEY_J, KEY_K, KEY_L, KEY_LEFT,
    KEY_LEFT_ALT, KEY_LEFT_CONTROL, KEY_LEFT_SHIFT, KEY_LEFT_SUPER, KEY_M, KEY_MINUS,
    KEY_N, KEY_NINE, KEY_O, KEY_ONE, KEY_P, KEY_PERIOD, KEY_Q, KEY_R, KEY_RIGHT,
    KEY_RIGHT_ALT, KEY_RIGHT_CONTROL, KEY_RIGHT_SHIFT, KEY_RIGHT_SUPER, KEY_S, KEY_SEMICOLON,
    KEY_SEVEN, KEY_SIX, KEY_SLASH, KEY_SPACE, KEY_T, KEY_TAB, KEY_THREE, KEY_TWO, KEY_U,
    KEY_UP, KEY_V, KEY_W, KEY_X, KEY_Y, KEY_Z, KEY_ZERO, MOUSE_BUTTON_LEFT,
    MOUSE_BUTTON_MIDDLE, MOUSE_BUTTON_RIGHT,
};

/// Type IDs for all Raylib constructible struct types.
/// Populated by `register_raylib_types()`.
#[derive(Debug, Clone, Copy)]
pub struct RaylibStructIds {
    pub color: crate::types::TypeId,
    pub vector2: crate::types::TypeId,
    pub rectangle: crate::types::TypeId,
    pub texture2d: crate::types::TypeId,
}

// ─── Safe Wrapper Functions ───
// Each wrapper validates inputs, translates types, then calls the raw FFI.

/// Initialize a window with the given dimensions and title.
/// # Safety
/// Must be called before any other window/drawing function.
/// Only one window can be open at a time.
pub unsafe fn init_window(width: i32, height: i32, title: &str) {
    let c_title = std::ffi::CString::new(title).expect("title contains null byte");
    raylib_sys::InitWindow(width, height, c_title.as_ptr());
}

/// Close the window and release resources.
/// # Safety
/// Must be called to properly cleanup. No window functions after this.
pub unsafe fn close_window() {
    raylib_sys::CloseWindow();
}

/// Check if the window should close (user pressed close button or Escape).
pub unsafe fn window_should_close() -> bool {
    raylib_sys::WindowShouldClose()
}

/// Set the target frame rate.
pub unsafe fn set_target_fps(fps: i32) {
    raylib_sys::SetTargetFPS(fps);
}

/// Get current FPS.
pub unsafe fn get_fps() -> i32 {
    raylib_sys::GetFPS()
}

/// Get frame time in seconds (delta time).
pub unsafe fn get_frame_time() -> f32 {
    raylib_sys::GetFrameTime()
}

/// Get elapsed time since InitWindow().
pub unsafe fn get_time() -> f64 {
    raylib_sys::GetTime()
}

/// Get current screen width.
pub unsafe fn get_screen_width() -> i32 {
    raylib_sys::GetScreenWidth()
}

/// Get current screen height.
pub unsafe fn get_screen_height() -> i32 {
    raylib_sys::GetScreenHeight()
}

// ─── Drawing ───

/// Begin drawing to the screen (clear + start frame).
pub unsafe fn begin_drawing() {
    raylib_sys::BeginDrawing();
}

/// End drawing and swap buffers (present frame).
pub unsafe fn end_drawing() {
    raylib_sys::EndDrawing();
}

/// Clear the screen with a color.
pub unsafe fn clear_background(color: Color) {
    raylib_sys::ClearBackground(color);
}

/// Draw text at the specified position.
pub unsafe fn draw_text(text: &str, pos_x: i32, pos_y: i32, font_size: i32, color: Color) {
    let c_text = std::ffi::CString::new(text).expect("text contains null byte");
    raylib_sys::DrawText(c_text.as_ptr(), pos_x, pos_y, font_size, color);
}

/// Draw a filled rectangle.
pub unsafe fn draw_rectangle(x: i32, y: i32, width: i32, height: i32, color: Color) {
    raylib_sys::DrawRectangle(x, y, width, height, color);
}

/// Draw a filled circle.
pub unsafe fn draw_circle(center_x: i32, center_y: i32, radius: f32, color: Color) {
    raylib_sys::DrawCircle(center_x, center_y, radius, color);
}

/// Draw a line between two points.
pub unsafe fn draw_line(x1: i32, y1: i32, x2: i32, y2: i32, color: Color) {
    raylib_sys::DrawLine(x1, y1, x2, y2, color);
}

/// Draw a rectangle outline.
pub unsafe fn draw_rectangle_lines(x: i32, y: i32, width: i32, height: i32, color: Color) {
    raylib_sys::DrawRectangleLines(x, y, width, height, color);
}

/// Draw a single pixel.
pub unsafe fn draw_pixel(x: i32, y: i32, color: Color) {
    raylib_sys::DrawPixel(x, y, color);
}

// ─── Textures ───

/// Load a texture from a file.
/// # Safety
/// File must exist and be a valid image format supported by Raylib.
pub unsafe fn load_texture(file_name: &str) -> Texture2D {
    let c_name = std::ffi::CString::new(file_name).expect("filename contains null byte");
    raylib_sys::LoadTexture(c_name.as_ptr())
}

/// Draw a texture at the specified position.
pub unsafe fn draw_texture(texture: Texture2D, x: i32, y: i32, tint: Color) {
    raylib_sys::DrawTexture(texture, x, y, tint);
}

/// Unload a texture and free GPU memory.
pub unsafe fn unload_texture(texture: Texture2D) {
    raylib_sys::UnloadTexture(texture);
}

// ─── Input ───

/// Check if a key is currently held down.
pub unsafe fn is_key_down(key: i32) -> bool {
    raylib_sys::IsKeyDown(key)
}

/// Check if a key was pressed this frame (single trigger).
pub unsafe fn is_key_pressed(key: i32) -> bool {
    raylib_sys::IsKeyPressed(key)
}

/// Get the last key pressed (queue-based).
pub unsafe fn get_key_pressed() -> i32 {
    raylib_sys::GetKeyPressed()
}

/// Check if a mouse button was pressed this frame.
pub unsafe fn is_mouse_button_pressed(button: i32) -> bool {
    raylib_sys::IsMouseButtonPressed(button)
}

/// Get mouse X position.
pub unsafe fn get_mouse_x() -> i32 {
    raylib_sys::GetMouseX()
}

/// Get mouse Y position.
pub unsafe fn get_mouse_y() -> i32 {
    raylib_sys::GetMouseY()
}

/// Get mouse position as a Vector2.
pub unsafe fn get_mouse_position() -> Vector2 {
    raylib_sys::GetMousePosition()
}

// ─── Struct Constructor Registry ───

/// Return true if the given name is a Raylib struct constructor
/// that can be called as `StructName(x, y, ...)` from .ldx code.
pub fn is_struct_constructor(name: &str) -> bool {
    matches!(name, "Color" | "Vector2" | "Rectangle")
}

/// Get the parameter count for a struct constructor.
/// Returns None if not a known constructor.
pub fn struct_constructor_arity(name: &str) -> Option<usize> {
    match name {
        "Color" => Some(4),      // r, g, b, a
        "Vector2" => Some(2),    // x, y
        "Rectangle" => Some(4),  // x, y, width, height
        _ => None,
    }
}

// ─── CallableRegistry Integration ───

/// v1.42: Register all Raylib core functions with the Logicodex CallableRegistry.
/// This allows .ldx code to call Raylib functions through the FFI layer.
///
/// Changes from v1.30:
/// - Drawing functions now take Color struct type (not packed u32)
/// - Texture2D functions use Texture2D struct type (not i64 handle)
/// - Math utilities (clamp, lerp, remap) registered as safe functions
///
/// Each Raylib function is registered with:
/// - UnsafeRequired safety (must be inside UnsafeBlock)
/// - C calling convention
/// - Correct parameter and return types from TypeRegistry
pub fn register_raylib_functions(
    registry: &mut TypeRegistry,
    callables: &mut CallableRegistry,
    struct_ids: &RaylibTypeIds,
) {
    let ids = registry.primitive_ids();

    // Helper to create C string pointer type
    let c_string = registry.intern(crate::types::TypeKind::Pointer {
        pointee: ids.i8_,
        mutability: crate::types::Mutability::Immutable,
    });

    // Helper macro to register a function
    macro_rules! register_fn {
        ($name:expr, $params:expr, $ret:expr) => {
            callables.register(CallableSignature {
                name: $name.to_string(),
                params: $params.to_vec(),
                return_type: $ret,
                abi: CallingConvention::C,
                safety: CallableSafety::UnsafeRequired,
                is_extern: true,
                is_variadic: false,
            })
        };
    }

    // ─── Windowing (9 functions) ───
    register_fn!("InitWindow", &[ids.i32_, ids.i32_, c_string], ids.unit);
    register_fn!("CloseWindow", &[], ids.unit);
    register_fn!("WindowShouldClose", &[], ids.bool_);
    register_fn!("SetTargetFPS", &[ids.i32_], ids.unit);
    register_fn!("GetFPS", &[], ids.i32_);
    register_fn!("GetFrameTime", &[], ids.f32_);
    register_fn!("GetTime", &[], ids.f64_);
    register_fn!("GetScreenWidth", &[], ids.i32_);
    register_fn!("GetScreenHeight", &[], ids.i32_);

    // ─── Drawing (9 functions) ───
    // v1.42: Color passed as struct type (struct-by-value), not packed u32
    register_fn!("BeginDrawing", &[], ids.unit);
    register_fn!("EndDrawing", &[], ids.unit);
    register_fn!("ClearBackground", &[struct_ids.color], ids.unit);
    register_fn!("DrawText", &[c_string, ids.i32_, ids.i32_, ids.i32_, struct_ids.color], ids.unit);
    register_fn!("DrawRectangle", &[ids.i32_, ids.i32_, ids.i32_, ids.i32_, struct_ids.color], ids.unit);
    register_fn!("DrawCircle", &[ids.i32_, ids.i32_, ids.f32_, struct_ids.color], ids.unit);
    register_fn!("DrawLine", &[ids.i32_, ids.i32_, ids.i32_, ids.i32_, struct_ids.color], ids.unit);
    register_fn!("DrawRectangleLines", &[ids.i32_, ids.i32_, ids.i32_, ids.i32_, struct_ids.color], ids.unit);
    register_fn!("DrawPixel", &[ids.i32_, ids.i32_, struct_ids.color], ids.unit);

    // ─── Textures (3 functions) ───
    // v1.42: Texture2D passed as struct type
    register_fn!("LoadTexture", &[c_string], struct_ids.texture2d);
    register_fn!("DrawTexture", &[struct_ids.texture2d, ids.i32_, ids.i32_, struct_ids.color], ids.unit);
    register_fn!("UnloadTexture", &[struct_ids.texture2d], ids.unit);

    // ─── Input (6 functions) ───
    register_fn!("IsKeyDown", &[ids.i32_], ids.bool_);
    register_fn!("IsKeyPressed", &[ids.i32_], ids.bool_);
    register_fn!("GetKeyPressed", &[], ids.i32_);
    register_fn!("IsMouseButtonPressed", &[ids.i32_], ids.bool_);
    register_fn!("GetMouseX", &[], ids.i32_);
    register_fn!("GetMouseY", &[], ids.i32_);

    // ─── Math Utilities (v1.42 P4: safe functions, no unsafe required) ───
    register_math_functions(registry, callables);
}

/// Backward-compatible wrapper for tests that don't have struct IDs.
/// Looks up struct types from TypeRegistry by name.
pub fn register_raylib_functions_compat(registry: &mut TypeRegistry, callables: &mut CallableRegistry) {
    let (_, struct_ids) = register_raylib_types(registry);
    register_raylib_functions(registry, callables, &struct_ids);
}

/// v1.42: Register math utility functions as safe (no unsafe required).
/// These are pure Rust functions that mirror raymath.h operations.
fn register_math_functions(registry: &mut TypeRegistry, callables: &mut CallableRegistry) {
    let ids = registry.primitive_ids();

    macro_rules! register_fn {
        ($name:expr, $params:expr, $ret:expr, $safety:expr) => {
            callables.register(CallableSignature {
                name: $name.to_string(),
                params: $params.to_vec(),
                return_type: $ret,
                abi: CallingConvention::C,
                safety: $safety,
                is_extern: false, // pure Rust, not FFI
                is_variadic: false,
            })
        };
    }

    // clamp(v: F32, min: F32, max: F32) → F32
    register_fn!("clamp", &[ids.f32_, ids.f32_, ids.f32_], ids.f32_, CallableSafety::Safe);

    // lerp(a: F32, b: F32, t: F32) → F32
    register_fn!("lerp", &[ids.f32_, ids.f32_, ids.f32_], ids.f32_, CallableSafety::Safe);

    // remap(value: F32, low1: F32, high1: F32, low2: F32, high2: F32) → F32
    register_fn!(
        "remap",
        &[ids.f32_, ids.f32_, ids.f32_, ids.f32_, ids.f32_],
        ids.f32_,
        CallableSafety::Safe
    );

    // normalize(value: F32, low: F32, high: F32) → F32
    register_fn!("normalize", &[ids.f32_, ids.f32_, ids.f32_], ids.f32_, CallableSafety::Safe);
}

/// v1.42 P4: Math utility implementations exposed as extern-C shims
/// so they can be called from LLVM-generated code via the CallableRegistry.
pub mod math_shims {
    use super::*;

    /// Shim: clamp(v: f32, min: f32, max: f32) → f32
    #[no_mangle]
    pub extern "C" fn logicodex_clamp_f32(v: f32, min: f32, max: f32) -> f32 {
        super::super::math::clamp(v, min, max)
    }

    /// Shim: lerp(a: f32, b: f32, t: f32) → f32
    #[no_mangle]
    pub extern "C" fn logicodex_lerp_f32(a: f32, b: f32, t: f32) -> f32 {
        super::super::math::lerp(a, b, t)
    }

    /// Shim: remap(value: f32, low1: f32, high1: f32, low2: f32, high2: f32) → f32
    #[no_mangle]
    pub extern "C" fn logicodex_remap_f32(value: f32, low1: f32, high1: f32, low2: f32, high2: f32) -> f32 {
        super::super::math::remap(value, low1, high1, low2, high2)
    }

    /// Shim: normalize(value: f32, low: f32, high: f32) → f32
    #[no_mangle]
    pub extern "C" fn logicodex_normalize_f32(value: f32, low: f32, high: f32) -> f32 {
        super::super::math::normalize(value, low, high)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ffi::{CallableRegistry, CallableSafety};
    use crate::types::TypeRegistry;

    #[test]
    fn raylib_functions_are_registered() {
        let mut registry = TypeRegistry::new();
        let mut callables = CallableRegistry::default();
        register_raylib_functions(&mut registry, &mut callables);

        // Check key functions are registered
        assert!(callables.find_by_name("InitWindow").is_some());
        assert!(callables.find_by_name("CloseWindow").is_some());
        assert!(callables.find_by_name("WindowShouldClose").is_some());
        assert!(callables.find_by_name("BeginDrawing").is_some());
        assert!(callables.find_by_name("EndDrawing").is_some());
        assert!(callables.find_by_name("DrawText").is_some());
        assert!(callables.find_by_name("IsKeyDown").is_some());
        assert!(callables.find_by_name("LoadTexture").is_some());
    }

    #[test]
    fn raylib_functions_require_unsafe() {
        let mut registry = TypeRegistry::new();
        let mut callables = CallableRegistry::default();
        register_raylib_functions(&mut registry, &mut callables);

        let (_, signature) = callables.find_by_name("InitWindow").unwrap();
        assert_eq!(signature.safety, CallableSafety::UnsafeRequired);
        assert!(signature.is_extern);
        assert_eq!(signature.abi, CallingConvention::C);
    }

    #[test]
    fn color_hex_packing() {
        // Test that Color::from_hex produces the expected RGBA values
        let red = Color::from_hex(0xFF0000FF);
        assert_eq!(red.r, 255);
        assert_eq!(red.g, 0);
        assert_eq!(red.b, 0);
        assert_eq!(red.a, 255);

        let green = Color::from_hex(0x00FF00FF);
        assert_eq!(green.r, 0);
        assert_eq!(green.g, 255);
        assert_eq!(green.b, 0);
        assert_eq!(green.a, 255);

        let blue = Color::from_hex(0x0000FFFF);
        assert_eq!(blue.r, 0);
        assert_eq!(blue.g, 0);
        assert_eq!(blue.b, 255);
        assert_eq!(blue.a, 255);

        // Verify roundtrip
        assert_eq!(red.to_hex(), 0xFF0000FF);
        assert_eq!(green.to_hex(), 0x00FF00FF);
        assert_eq!(blue.to_hex(), 0x0000FFFF);
    }

    #[test]
    fn color_size_for_ffi() {
        // Color must be 4 bytes for correct C ABI
        assert_eq!(std::mem::size_of::<Color>(), 4);
        assert_eq!(std::mem::align_of::<Color>(), 1);
    }

    #[test]
    fn vector2_size_for_ffi() {
        // Vector2 must be 8 bytes for correct C ABI
        assert_eq!(std::mem::size_of::<Vector2>(), 8);
        assert_eq!(std::mem::align_of::<Vector2>(), 4);
    }
}
