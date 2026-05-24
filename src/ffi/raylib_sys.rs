// =========================================================================
// Logicodex v1.30 — Raylib Raw FFI Declarations
// Sprint 1: Type System Foundation — FFI Layer
//
// Raw `extern "C"` bindings for the Raylib game development library.
// These are the low-level C function declarations with no safety wrappers.
// They are consumed by src/ffi/raylib.rs which provides the safe wrapper layer.
//
// Coverage: 20 core functions for windowing, drawing, textures, and input.
// Additional functions can be added following the same pattern.
// =========================================================================

// ─── Raylib C Types ───

/// Raylib Color struct (4 bytes: R, G, B, A)
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

/// Raylib Vector2 struct (8 bytes: x, y as f32)
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

/// Raylib Vector3 struct (12 bytes)
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// Raylib Rectangle struct (16 bytes: x, y, width, height as f32)
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rectangle {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Raylib Image struct (opaque handle)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Image {
    pub data: *mut u8,
    pub width: i32,
    pub height: i32,
    pub mipmaps: i32,
    pub format: i32,
}

/// Raylib Texture2D struct (opaque handle, 20 bytes)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Texture2D {
    pub id: u32,
    pub width: i32,
    pub height: i32,
    pub mipmaps: i32,
    pub format: i32,
}

/// Keyboard keys enumeration (subset of most common)
pub const KEY_NULL: i32 = 0;
pub const KEY_APOSTROPHE: i32 = 39;
pub const KEY_COMMA: i32 = 44;
pub const KEY_MINUS: i32 = 45;
pub const KEY_PERIOD: i32 = 46;
pub const KEY_SLASH: i32 = 47;
pub const KEY_ZERO: i32 = 48;
pub const KEY_ONE: i32 = 49;
pub const KEY_TWO: i32 = 50;
pub const KEY_THREE: i32 = 51;
pub const KEY_FOUR: i32 = 52;
pub const KEY_FIVE: i32 = 53;
pub const KEY_SIX: i32 = 54;
pub const KEY_SEVEN: i32 = 55;
pub const KEY_EIGHT: i32 = 56;
pub const KEY_NINE: i32 = 57;
pub const KEY_SEMICOLON: i32 = 59;
pub const KEY_EQUAL: i32 = 61;
pub const KEY_A: i32 = 65;
pub const KEY_B: i32 = 66;
pub const KEY_C: i32 = 67;
pub const KEY_D: i32 = 68;
pub const KEY_E: i32 = 69;
pub const KEY_F: i32 = 70;
pub const KEY_G: i32 = 71;
pub const KEY_H: i32 = 72;
pub const KEY_I: i32 = 73;
pub const KEY_J: i32 = 74;
pub const KEY_K: i32 = 75;
pub const KEY_L: i32 = 76;
pub const KEY_M: i32 = 77;
pub const KEY_N: i32 = 78;
pub const KEY_O: i32 = 79;
pub const KEY_P: i32 = 80;
pub const KEY_Q: i32 = 81;
pub const KEY_R: i32 = 82;
pub const KEY_S: i32 = 83;
pub const KEY_T: i32 = 84;
pub const KEY_U: i32 = 85;
pub const KEY_V: i32 = 86;
pub const KEY_W: i32 = 87;
pub const KEY_X: i32 = 88;
pub const KEY_Y: i32 = 89;
pub const KEY_Z: i32 = 90;
pub const KEY_SPACE: i32 = 32;
pub const KEY_ESCAPE: i32 = 256;
pub const KEY_ENTER: i32 = 257;
pub const KEY_TAB: i32 = 258;
pub const KEY_BACKSPACE: i32 = 259;
pub const KEY_INSERT: i32 = 260;
pub const KEY_DELETE: i32 = 261;
pub const KEY_RIGHT: i32 = 262;
pub const KEY_LEFT: i32 = 263;
pub const KEY_DOWN: i32 = 264;
pub const KEY_UP: i32 = 265;
pub const KEY_LEFT_SHIFT: i32 = 340;
pub const KEY_LEFT_CONTROL: i32 = 341;
pub const KEY_LEFT_ALT: i32 = 342;
pub const KEY_LEFT_SUPER: i32 = 343;
pub const KEY_RIGHT_SHIFT: i32 = 344;
pub const KEY_RIGHT_CONTROL: i32 = 345;
pub const KEY_RIGHT_ALT: i32 = 346;
pub const KEY_RIGHT_SUPER: i32 = 347;

// ─── Predefined Colors ───

impl Color {
    pub const LIGHTGRAY: Color = Color {
        r: 200,
        g: 200,
        b: 200,
        a: 255,
    };
    pub const GRAY: Color = Color {
        r: 130,
        g: 130,
        b: 130,
        a: 255,
    };
    pub const DARKGRAY: Color = Color {
        r: 80,
        g: 80,
        b: 80,
        a: 255,
    };
    pub const YELLOW: Color = Color {
        r: 253,
        g: 249,
        b: 0,
        a: 255,
    };
    pub const GOLD: Color = Color {
        r: 255,
        g: 203,
        b: 0,
        a: 255,
    };
    pub const ORANGE: Color = Color {
        r: 255,
        g: 161,
        b: 0,
        a: 255,
    };
    pub const PINK: Color = Color {
        r: 255,
        g: 109,
        b: 194,
        a: 255,
    };
    pub const RED: Color = Color {
        r: 230,
        g: 41,
        b: 55,
        a: 255,
    };
    pub const MAROON: Color = Color {
        r: 190,
        g: 33,
        b: 55,
        a: 255,
    };
    pub const GREEN: Color = Color {
        r: 0,
        g: 228,
        b: 48,
        a: 255,
    };
    pub const LIME: Color = Color {
        r: 0,
        g: 158,
        b: 47,
        a: 255,
    };
    pub const DARKGREEN: Color = Color {
        r: 0,
        g: 117,
        b: 44,
        a: 255,
    };
    pub const SKYBLUE: Color = Color {
        r: 102,
        g: 191,
        b: 255,
        a: 255,
    };
    pub const BLUE: Color = Color {
        r: 0,
        g: 121,
        b: 241,
        a: 255,
    };
    pub const DARKBLUE: Color = Color {
        r: 0,
        g: 82,
        b: 172,
        a: 255,
    };
    pub const PURPLE: Color = Color {
        r: 200,
        g: 122,
        b: 255,
        a: 255,
    };
    pub const VIOLET: Color = Color {
        r: 135,
        g: 60,
        b: 190,
        a: 255,
    };
    pub const DARKPURPLE: Color = Color {
        r: 112,
        g: 31,
        b: 126,
        a: 255,
    };
    pub const BEIGE: Color = Color {
        r: 211,
        g: 176,
        b: 131,
        a: 255,
    };
    pub const BROWN: Color = Color {
        r: 127,
        g: 106,
        b: 79,
        a: 255,
    };
    pub const DARKBROWN: Color = Color {
        r: 76,
        g: 63,
        b: 47,
        a: 255,
    };
    pub const WHITE: Color = Color {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    };
    pub const BLACK: Color = Color {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    };
    pub const BLANK: Color = Color {
        r: 0,
        g: 0,
        b: 0,
        a: 0,
    };
    pub const MAGENTA: Color = Color {
        r: 255,
        g: 0,
        b: 255,
        a: 255,
    };
    pub const RAYWHITE: Color = Color {
        r: 245,
        g: 245,
        b: 245,
        a: 255,
    };

    /// Pack RGBA components into a single Color.
    pub const fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Color { r, g, b, a }
    }

    /// Pack a hex value (e.g., 0xFF0000FF for opaque red) into a Color.
    pub const fn from_hex(hex: u32) -> Self {
        Color {
            r: ((hex >> 24) & 0xFF) as u8,
            g: ((hex >> 16) & 0xFF) as u8,
            b: ((hex >> 8) & 0xFF) as u8,
            a: (hex & 0xFF) as u8,
        }
    }

    /// Convert to u32 hex representation.
    pub const fn to_hex(&self) -> u32 {
        ((self.r as u32) << 24)
            | ((self.g as u32) << 16)
            | ((self.b as u32) << 8)
            | (self.a as u32)
    }
}

// ─── Raw FFI Declarations ───
// These are the actual C function signatures from Raylib.
// They are unsafe to call directly — use the wrapper layer in raylib.rs.

extern "C" {
    // ─── Windowing ───
    pub fn InitWindow(width: i32, height: i32, title: *const i8);
    pub fn CloseWindow();
    pub fn WindowShouldClose() -> bool;
    pub fn SetTargetFPS(fps: i32);
    pub fn GetFPS() -> i32;
    pub fn GetFrameTime() -> f32;
    pub fn GetTime() -> f64;
    pub fn GetScreenWidth() -> i32;
    pub fn GetScreenHeight() -> i32;

    // ─── Drawing ───
    pub fn BeginDrawing();
    pub fn EndDrawing();
    pub fn ClearBackground(color: Color);
    pub fn DrawText(text: *const i8, posX: i32, posY: i32, fontSize: i32, color: Color);
    pub fn DrawRectangle(posX: i32, posY: i32, width: i32, height: i32, color: Color);
    pub fn DrawCircle(centerX: i32, centerY: i32, radius: f32, color: Color);
    pub fn DrawLine(startPosX: i32, startPosY: i32, endPosX: i32, endPosY: i32, color: Color);
    pub fn DrawRectangleLines(posX: i32, posY: i32, width: i32, height: i32, color: Color);
    pub fn DrawPixel(posX: i32, posY: i32, color: Color);

    // ─── Textures ───
    pub fn LoadTexture(fileName: *const i8) -> Texture2D;
    pub fn DrawTexture(texture: Texture2D, posX: i32, posY: i32, tint: Color);
    pub fn UnloadTexture(texture: Texture2D);

    // ─── Input ───
    pub fn IsKeyDown(key: i32) -> bool;
    pub fn IsKeyPressed(key: i32) -> bool;
    pub fn GetKeyPressed() -> i32;
    pub fn IsMouseButtonPressed(button: i32) -> bool;
    pub fn GetMouseX() -> i32;
    pub fn GetMouseY() -> i32;
    pub fn GetMousePosition() -> Vector2;
}

/// Mouse button constants
pub const MOUSE_BUTTON_LEFT: i32 = 0;
pub const MOUSE_BUTTON_RIGHT: i32 = 1;
pub const MOUSE_BUTTON_MIDDLE: i32 = 2;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_size_is_4_bytes() {
        assert_eq!(std::mem::size_of::<Color>(), 4);
    }

    #[test]
    fn color_from_hex_roundtrip() {
        let color = Color::from_hex(0xFF0000FF);
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 0);
        assert_eq!(color.b, 0);
        assert_eq!(color.a, 255);
        assert_eq!(color.to_hex(), 0xFF0000FF);
    }

    #[test]
    fn color_predefined_constants() {
        assert_eq!(Color::RED.r, 230);
        assert_eq!(Color::GREEN.g, 228);
        assert_eq!(Color::BLUE.b, 241);
        assert_eq!(Color::WHITE.to_hex(), 0xFFFFFFFF);
        assert_eq!(Color::BLACK.to_hex(), 0x000000FF);
    }

    #[test]
    fn vector2_size_is_8_bytes() {
        assert_eq!(std::mem::size_of::<Vector2>(), 8);
    }

    #[test]
    fn texture2d_size() {
        // Texture2D should be 20 bytes: u32 + 4×i32
        assert_eq!(std::mem::size_of::<Texture2D>(), 20);
    }
}
