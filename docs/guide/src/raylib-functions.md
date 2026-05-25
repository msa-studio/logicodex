# Jadual Fungsi Lengkap Raylib

Senarai lengkap 54 fungsi Raylib yang tersedia dalam Logicodex.

---

## Fungsi Inisialisasi dan Window (6)

| Fungsi | Parameter | Return | Fail |
|---|---|---|---|
| `InitWindow` | `lebar: I32, tinggi: I32, tajuk: Text` | `Void` | `raylib_sys.rs:23` |
| `CloseWindow` | — | `Void` | `raylib_sys.rs:24` |
| `WindowShouldClose` | — | `Bool` | `raylib_sys.rs:25` |
| `SetTargetFPS` | `fps: I32` | `Void` | `raylib_sys.rs:26` |
| `GetScreenWidth` | — | `I32` | `raylib_sys.rs:27` |
| `GetScreenHeight` | — | `I32` | `raylib_sys.rs:28` |

## Fungsi Menggambar (9)

| Fungsi | Parameter | Return | Fail |
|---|---|---|---|
| `ClearBackground` | `warna: Color` | `Void` | `raylib_sys.rs:30` |
| `BeginDrawing` | — | `Void` | `raylib_sys.rs:31` |
| `EndDrawing` | — | `Void` | `raylib_sys.rs:32` |
| `DrawText` | `teks: Text, x: I32, y: I32, saizFont: I32, warna: Color` | `Void` | `raylib_sys.rs:33` |
| `DrawRectangle` | `posX: I32, posY: I32, lebar: I32, tinggi: I32, warna: Color` | `Void` | `raylib_sys.rs:34` |
| `DrawRectangleLines` | `posX: I32, posY: I32, lebar: I32, tinggi: I32, warna: Color` | `Void` | `raylib_sys.rs:35` |
| `DrawCircle` | `pusatX: I32, pusatY: I32, jejari: F32, warna: Color` | `Void` | `raylib_sys.rs:36` |
| `DrawLine` | `startX: I32, startY: I32, endX: I32, endY: I32, warna: Color` | `Void` | `raylib_sys.rs:37` |
| `DrawPixel` | `posX: I32, posY: I32, warna: Color` | `Void` | `raylib_sys.rs:38` |

## Fungsi Input (8)

| Fungsi | Parameter | Return | Fail |
|---|---|---|---|
| `GetMousePosition` | — | `Vector2` | `raylib_sys.rs:40` |
| `IsMouseButtonPressed` | `button: I32` | `Bool` | `raylib_sys.rs:41` |
| `IsMouseButtonDown` | `button: I32` | `Bool` | `raylib_sys.rs:42` |
| `GetMouseWheelMove` | — | `F32` | `raylib_sys.rs:43` |
| `IsKeyPressed` | `key: I32` | `Bool` | `raylib_sys.rs:44` |
| `IsKeyDown` | `key: I32` | `Bool` | `raylib_sys.rs:45` |

## Fungsi Texture (6)

| Fungsi | Parameter | Return | Fail |
|---|---|---|---|
| `LoadTexture` | `namaFail: Text` | `Texture2D` | `raylib_sys.rs:47` |
| `UnloadTexture` | `texture: Texture2D` | `Void` | `raylib_sys.rs:48` |
| `DrawTexture` | `texture: Texture2D, posX: I32, posY: I32, tint: Color` | `Void` | `raylib_sys.rs:49` |
| `DrawTextureEx` | `texture: Texture2D, position: Vector2, rotation: F32, scale: F32, tint: Color` | `Void` | `raylib_sys.rs:50` |
| `DrawTextureRec` | `texture: Texture2D, source: Rectangle, position: Vector2, tint: Color` | `Void` | `raylib_sys.rs:51` |
| `DrawTexturePro` | `texture: Texture2D, source: Rectangle, dest: Rectangle, origin: Vector2, rotation: F32, tint: Color` | `Void` | `raylib_sys.rs:52` |

## Fungsi Matematik (4)

| Fungsi | Parameter | Return | Fail |
|---|---|---|---|
| `Clamp` | `nilai: F32, min: F32, max: F32` | `F32` | `raylib.rs:300` |
| `Lerp` | `start: F32, end: F32, amount: F32` | `F32` | `raylib.rs:301` |
| `Normalize` | `nilai: F32, start: F32, end: F32` | `F32` | `raylib.rs:302` |
| `Remap` | `nilai: F32, inStart: F32, inEnd: F32, outStart: F32, outEnd: F32` | `F32` | `raylib.rs:303` |

## Fungsi Tambahan (2)

| Fungsi | Parameter | Return | Fail |
|---|---|---|---|
| `DrawTriangle` | `v1: Vector2, v2: Vector2, v3: Vector2, warna: Color` | `Void` | `raylib_sys.rs:53` |
| `DrawPoly` | `pusat: Vector2, sisi: I32, jejari: F32, putaran: F32, warna: Color` | `Void` | `raylib_sys.rs:54` |

---

## Struct Raylib

| Struct | Medan | Saiz (bytes) |
|---|---|---|
| `Vector2` | `x: F32, y: F32` | 8 |
| `Vector3` | `x: F32, y: F32, z: F32` | 12 |
| `Rectangle` | `x: F32, y: F32, width: F32, height: F32` | 16 |
| `Color` | `r: U8, g: U8, b: U8, a: U8` | 4 |
| `Texture2D` | `id: U32, width: I32, height: I32, mipmaps: I32, format: I32` | 20 |
| `Image` | `data: PTR<Void>, width: I32, height: I32, mipmaps: I32, format: I32` | 24 |

## Color Constants

| Constant | Nilai (RGBA) |
|---|---|
| `LIGHTGRAY` | `{ r: 200, g: 200, b: 200, a: 255 }` |
| `GRAY` | `{ r: 130, g: 130, b: 130, a: 255 }` |
| `DARKGRAY` | `{ r: 80, g: 80, b: 80, a: 255 }` |
| `YELLOW` | `{ r: 253, g: 249, b: 0, a: 255 }` |
| `GOLD` | `{ r: 255, g: 203, b: 0, a: 255 }` |
| `ORANGE` | `{ r: 255, g: 161, b: 0, a: 255 }` |
| `PINK` | `{ r: 255, g: 109, b: 194, a: 255 }` |
| `RED` | `{ r: 230, g: 41, b: 55, a: 255 }` |
| `MAROON` | `{ r: 190, g: 33, b: 55, a: 255 }` |
| `GREEN` | `{ r: 0, g: 228, b: 48, a: 255 }` |
| `LIME` | `{ r: 0, g: 158, b: 47, a: 255 }` |
| `DARKGREEN` | `{ r: 0, g: 117, b: 44, a: 255 }` |
| `SKYBLUE` | `{ r: 102, g: 191, b: 255, a: 255 }` |
| `BLUE` | `{ r: 0, g: 121, b: 241, a: 255 }` |
| `DARKBLUE` | `{ r: 0, g: 82, b: 172, a: 255 }` |
| `PURPLE` | `{ r: 200, g: 122, b: 255, a: 255 }` |
| `VIOLET` | `{ r: 135, g: 60, b: 190, a: 255 }` |
| `DARKPURPLE` | `{ r: 112, g: 31, b: 126, a: 255 }` |
| `BEIGE` | `{ r: 211, g: 176, b: 131, a: 255 }` |
| `BROWN` | `{ r: 127, g: 106, b: 79, a: 255 }` |
| `DARKBROWN` | `{ r: 76, g: 63, b: 47, a: 255 }` |
| `WHITE` | `{ r: 255, g: 255, b: 255, a: 255 }` |
| `BLACK` | `{ r: 0, g: 0, b: 0, a: 255 }` |
| `BLANK` | `{ r: 0, g: 0, b: 0, a: 0 }` |
| `MAGENTA` | `{ r: 255, g: 0, b: 255, a: 255 }` |
| `RAYWHITE` | `{ r: 245, g: 245, b: 245, a: 255 }` |

---

## Key Constants

| Constant | Nilai |
|---|---|
| `KEY_NULL` | 0 |
| `KEY_APOSTROPHE` | 39 |
| `KEY_COMMA` | 44 |
| `KEY_MINUS` | 45 |
| `KEY_PERIOD` | 46 |
| `KEY_SLASH` | 47 |
| `KEY_ZERO` - `KEY_NINE` | 48 - 57 |
| `KEY_SEMICOLON` | 59 |
| `KEY_EQUAL` | 61 |
| `KEY_A` - `KEY_Z` | 65 - 90 |
| `KEY_BACKSPACE` | 259 |
| `KEY_SPACE` | 32 |
| `KEY_ENTER` | 257 |
| `KEY_TAB` | 258 |
| `KEY_ESCAPE` | 256 |
| `KEY_RIGHT` | 262 |
| `KEY_LEFT` | 263 |
| `KEY_DOWN` | 264 |
| `KEY_UP` | 265 |
| `KEY_F1` - `KEY_F12` | 290 - 301 |

## Mouse Button Constants

| Constant | Nilai |
|---|---|
| `MOUSE_BUTTON_LEFT` | 0 |
| `MOUSE_BUTTON_RIGHT` | 1 |
| `MOUSE_BUTTON_MIDDLE` | 2 |
