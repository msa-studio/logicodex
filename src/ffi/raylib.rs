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
use crate::layout::LayoutEngine;
use crate::types::StructFieldLayout as StructField;
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
                StructField {
                    name: "r".into(),
                    ty: ids.u8_,
                    alignment_bytes: 0,
                    offset_bytes: 0,
                    size_bytes: 0,
                },
                StructField {
                    name: "g".into(),
                    ty: ids.u8_,
                    alignment_bytes: 0,
                    offset_bytes: 0,
                    size_bytes: 0,
                },
                StructField {
                    name: "b".into(),
                    ty: ids.u8_,
                    alignment_bytes: 0,
                    offset_bytes: 0,
                    size_bytes: 0,
                },
                StructField {
                    name: "a".into(),
                    ty: ids.u8_,
                    alignment_bytes: 0,
                    offset_bytes: 0,
                    size_bytes: 0,
                },
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
                StructField {
                    name: "x".into(),
                    ty: ids.f32_,
                    alignment_bytes: 0,
                    offset_bytes: 0,
                    size_bytes: 0,
                },
                StructField {
                    name: "y".into(),
                    ty: ids.f32_,
                    alignment_bytes: 0,
                    offset_bytes: 0,
                    size_bytes: 0,
                },
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
                StructField {
                    name: "x".into(),
                    ty: ids.f32_,
                    alignment_bytes: 0,
                    offset_bytes: 0,
                    size_bytes: 0,
                },
                StructField {
                    name: "y".into(),
                    ty: ids.f32_,
                    alignment_bytes: 0,
                    offset_bytes: 0,
                    size_bytes: 0,
                },
                StructField {
                    name: "width".into(),
                    ty: ids.f32_,
                    alignment_bytes: 0,
                    offset_bytes: 0,
                    size_bytes: 0,
                },
                StructField {
                    name: "height".into(),
                    ty: ids.f32_,
                    alignment_bytes: 0,
                    offset_bytes: 0,
                    size_bytes: 0,
                },
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
                StructField {
                    name: "id".into(),
                    ty: ids.u32_,
                    alignment_bytes: 0,
                    offset_bytes: 0,
                    size_bytes: 0,
                },
                StructField {
                    name: "width".into(),
                    ty: ids.i32_,
                    alignment_bytes: 0,
                    offset_bytes: 0,
                    size_bytes: 0,
                },
                StructField {
                    name: "height".into(),
                    ty: ids.i32_,
                    alignment_bytes: 0,
                    offset_bytes: 0,
                    size_bytes: 0,
                },
                StructField {
                    name: "mipmaps".into(),
                    ty: ids.i32_,
                    alignment_bytes: 0,
                    offset_bytes: 0,
                    size_bytes: 0,
                },
                StructField {
                    name: "format".into(),
                    ty: ids.i32_,
                    alignment_bytes: 0,
                    offset_bytes: 0,
                    size_bytes: 0,
                },
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
    // Core types
    Color, Image, Rectangle, Texture2D, Vector2, Vector3,
    // Audio types (v1.43)
    AudioCallback, AudioStream, Music, Sound, Wave,
    // Key constants
    KEY_A, KEY_B, KEY_BACKSPACE, KEY_C, KEY_D, KEY_DELETE, KEY_DOWN, KEY_E,
    KEY_EIGHT, KEY_ENTER, KEY_ESCAPE, KEY_F, KEY_FIVE, KEY_FOUR, KEY_G, KEY_H,
    KEY_I, KEY_INSERT, KEY_J, KEY_K, KEY_L, KEY_LEFT, KEY_LEFT_ALT,
    KEY_LEFT_CONTROL, KEY_LEFT_SHIFT, KEY_LEFT_SUPER, KEY_M, KEY_MINUS, KEY_N,
    KEY_NINE, KEY_O, KEY_ONE, KEY_P, KEY_PERIOD, KEY_Q, KEY_R, KEY_RIGHT,
    KEY_RIGHT_ALT, KEY_RIGHT_CONTROL, KEY_RIGHT_SHIFT, KEY_RIGHT_SUPER, KEY_S,
    KEY_SEMICOLON, KEY_SEVEN, KEY_SIX, KEY_SLASH, KEY_SPACE, KEY_T, KEY_TAB,
    KEY_THREE, KEY_TWO, KEY_U, KEY_UP, KEY_V, KEY_W, KEY_X, KEY_Y, KEY_Z,
    KEY_ZERO, MOUSE_BUTTON_LEFT, MOUSE_BUTTON_MIDDLE, MOUSE_BUTTON_RIGHT,
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

// ─── Audio (v1.43) ───
// Safe wrappers for Raylib audio functions.
// All audio functions require the Audio.Main capability gate.

/// Initialize the audio device. Must be called before any audio function.
pub unsafe fn init_audio_device() {
    raylib_sys::InitAudioDevice();
}

/// Close the audio device and release all audio resources.
pub unsafe fn close_audio_device() {
    raylib_sys::CloseAudioDevice();
}

/// Check if the audio device is ready.
pub unsafe fn is_audio_device_ready() -> bool {
    raylib_sys::IsAudioDeviceReady()
}

/// Set master volume (0.0 to 1.0).
pub unsafe fn set_master_volume(volume: f32) {
    raylib_sys::SetMasterVolume(volume);
}

// ─── Sound (short audio) ───

/// Load a sound from a file.
pub unsafe fn load_sound(file_name: &str) -> Sound {
    let c_name = std::ffi::CString::new(file_name).expect("filename contains null byte");
    raylib_sys::LoadSound(c_name.as_ptr())
}

/// Unload a sound and free its memory.
pub unsafe fn unload_sound(sound: Sound) {
    raylib_sys::UnloadSound(sound);
}

/// Play a sound.
pub unsafe fn play_sound(sound: Sound) {
    raylib_sys::PlaySound(sound);
}

/// Stop a playing sound.
pub unsafe fn stop_sound(sound: Sound) {
    raylib_sys::StopSound(sound);
}

/// Check if a sound is currently playing.
pub unsafe fn is_sound_playing(sound: Sound) -> bool {
    raylib_sys::IsSoundPlaying(sound)
}

/// Set sound volume (0.0 to 1.0).
pub unsafe fn set_sound_volume(sound: Sound, volume: f32) {
    raylib_sys::SetSoundVolume(sound, volume);
}

/// Set sound pitch (1.0 = normal).
pub unsafe fn set_sound_pitch(sound: Sound, pitch: f32) {
    raylib_sys::SetSoundPitch(sound, pitch);
}

// ─── Music (streaming) ───

/// Load music from a file (streams from disk).
pub unsafe fn load_music_stream(file_name: &str) -> Music {
    let c_name = std::ffi::CString::new(file_name).expect("filename contains null byte");
    raylib_sys::LoadMusicStream(c_name.as_ptr())
}

/// Unload music stream and free resources.
pub unsafe fn unload_music_stream(music: Music) {
    raylib_sys::UnloadMusicStream(music);
}

/// Play a music stream.
pub unsafe fn play_music_stream(music: Music) {
    raylib_sys::PlayMusicStream(music);
}

/// Stop a music stream.
pub unsafe fn stop_music_stream(music: Music) {
    raylib_sys::StopMusicStream(music);
}

/// Pause a music stream.
pub unsafe fn pause_music_stream(music: Music) {
    raylib_sys::PauseMusicStream(music);
}

/// Resume a paused music stream.
pub unsafe fn resume_music_stream(music: Music) {
    raylib_sys::ResumeMusicStream(music);
}

/// Check if music is playing.
pub unsafe fn is_music_stream_playing(music: Music) -> bool {
    raylib_sys::IsMusicStreamPlaying(music)
}

/// Update music stream (call every frame).
pub unsafe fn update_music_stream(music: Music) {
    raylib_sys::UpdateMusicStream(music);
}

/// Set music volume (0.0 to 1.0).
pub unsafe fn set_music_volume(music: Music, volume: f32) {
    raylib_sys::SetMusicVolume(music, volume);
}

/// Seek to a position in the music (seconds).
pub unsafe fn seek_music_stream(music: Music, position: f32) {
    raylib_sys::SeekMusicStream(music, position);
}

// ─── Audio Stream (real-time / callback) ───

/// Load an audio stream for real-time audio.
pub unsafe fn load_audio_stream(sample_rate: u32, sample_size: u32, channels: u32) -> AudioStream {
    raylib_sys::LoadAudioStream(sample_rate, sample_size, channels)
}

/// Unload an audio stream.
pub unsafe fn unload_audio_stream(stream: AudioStream) {
    raylib_sys::UnloadAudioStream(stream);
}

/// Play an audio stream.
pub unsafe fn play_audio_stream(stream: AudioStream) {
    raylib_sys::PlayAudioStream(stream);
}

/// Stop an audio stream.
pub unsafe fn stop_audio_stream(stream: AudioStream) {
    raylib_sys::StopAudioStream(stream);
}

/// Check if an audio stream is playing.
pub unsafe fn is_audio_stream_playing(stream: AudioStream) -> bool {
    raylib_sys::IsAudioStreamPlaying(stream)
}

/// Set a callback for real-time audio generation.
/// # Safety
/// The callback runs on the audio thread (ISR-like).
/// The callback function name should be registered with StrictAudioContext
/// via `Analyzer::register_audio_callback()` for safety validation.
pub unsafe fn set_audio_stream_callback(stream: AudioStream, callback: AudioCallback) {
    raylib_sys::SetAudioStreamCallback(stream, callback);
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

    // ─── Audio (v1.43: Sound + Music + AudioStream) ───
    // All audio functions require Audio.Main capability gate.
    // Audio stream callbacks are validated by StrictAudioContext.
    register_fn!("InitAudioDevice", &[], ids.unit);
    register_fn!("CloseAudioDevice", &[], ids.unit);
    register_fn!("IsAudioDeviceReady", &[], ids.bool_);
    register_fn!("SetMasterVolume", &[ids.f32_], ids.unit);
    // Sound
    register_fn!("LoadSound", &[c_string], ids.i64_); // returns Sound handle
    register_fn!("UnloadSound", &[ids.i64_], ids.unit);
    register_fn!("PlaySound", &[ids.i64_], ids.unit);
    register_fn!("StopSound", &[ids.i64_], ids.unit);
    register_fn!("IsSoundPlaying", &[ids.i64_], ids.bool_);
    // Music
    register_fn!("LoadMusicStream", &[c_string], ids.i64_); // returns Music handle
    register_fn!("UnloadMusicStream", &[ids.i64_], ids.unit);
    register_fn!("PlayMusicStream", &[ids.i64_], ids.unit);
    register_fn!("StopMusicStream", &[ids.i64_], ids.unit);
    register_fn!("IsMusicStreamPlaying", &[ids.i64_], ids.bool_);
    register_fn!("UpdateMusicStream", &[ids.i64_], ids.unit);
    register_fn!("SetMusicVolume", &[ids.i64_, ids.f32_], ids.unit);
    register_fn!("SeekMusicStream", &[ids.i64_, ids.f32_], ids.unit);
    // Audio Stream
    register_fn!("LoadAudioStream", &[ids.i32_, ids.i32_, ids.i32_], ids.i64_); // returns stream handle
    register_fn!("UnloadAudioStream", &[ids.i64_], ids.unit);
    register_fn!("PlayAudioStream", &[ids.i64_], ids.unit);
    register_fn!("StopAudioStream", &[ids.i64_], ids.unit);
    register_fn!("IsAudioStreamPlaying", &[ids.i64_], ids.bool_);

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
