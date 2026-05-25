# Chapter 13: Raylib FFI — Semua Fungsi

Logicodex menyokong 54 fungsi Raylib melalui FFI. Bab ini menerangkan setiap fungsi dengan contoh.

---

## Inisialisasi dan Window {#inisialisasi}

### InitWindow
```logicodex
InitWindow(lebar: I32, tinggi: I32, tajuk: Text) -> Void
-- Membuka tetingkap aplikasi
-- Contoh:
InitWindow(800, 600, "Aplikasi Saya")
```

### CloseWindow
```logicodex
CloseWindow() -> Void
-- Menutup tetingkap dan membersihkan sumber
-- Mesti dipanggil sebelum program tamat
```

### WindowShouldClose
```logicodex
WindowShouldClose() -> Bool
-- Mengembalikan true jika pengguna menutup tetingkap atau tekan ESC
-- Contoh:
SEMENTARA !WindowShouldClose()
    -- game loop
TAMAT_SEMENTARA
```

### SetTargetFPS
```logicodex
SetTargetFPS(fps: I32) -> Void
-- Menetapkan frame rate target
-- Contoh:
SetTargetFPS(60)   -- 60 FPS
```

### GetScreenWidth / GetScreenHeight
```logicodex
GetScreenWidth() -> I32
GetScreenHeight() -> I32
-- Mendapatkan dimensi tetingkap
-- Contoh:
BINA lebar SEBAGAI I32 = GetScreenWidth()
BINA tinggi SEBAGAI I32 = GetScreenHeight()
```

---

## Menggambar (Draw Functions) {#draw}

### ClearBackground
```logicodex
ClearBackground(warna: Color) -> Void
-- Membersihkan skrin dengan warna
-- Contoh:
ClearBackground(RAYWHITE)
ClearBackground(Color { r: 100, g: 150, b: 200, a: 255 })
```

### BeginDrawing / EndDrawing
```logicodex
BeginDrawing() -> Void
EndDrawing() -> Void
-- Mesti dipanggil berpasangan
-- Semua draw calls berada di antara BeginDrawing dan EndDrawing
```

### DrawText
```logicodex
DrawText(teks: Text, x: I32, y: I32, saizFont: I32, warna: Color) -> Void
-- Menulis teks pada skrin
-- Contoh:
DrawText("Halo Dunia!", 190, 200, 20, DARKGRAY)
DrawText("Skor: " + skor, 10, 10, 16, RED)
```

### DrawRectangle
```logicodex
DrawRectangle(posX: I32, posY: I32, lebar: I32, tinggi: I32, warna: Color) -> Void
-- Menggambar segiempat penuh
-- Contoh:
DrawRectangle(100, 100, 200, 150, BLUE)
```

### DrawRectangleLines
```logicodex
DrawRectangleLines(posX: I32, posY: I32, lebar: I32, tinggi: I32, warna: Color) -> Void
-- Menggambar batas segiempat (hanya garis)
-- Contoh:
DrawRectangleLines(100, 100, 200, 150, DARKBLUE)
```

### DrawCircle
```logicodex
DrawCircle(pusatX: I32, pusatY: I32, jejari: F32, warna: Color) -> Void
-- Menggambar bulatan
-- Contoh:
DrawCircle(400, 300, 50.0, RED)
```

### DrawLine
```logicodex
DrawLine(startPosX: I32, startPosY: I32, endPosX: I32, endPosY: I32, warna: Color) -> Void
-- Menggambar garis
-- Contoh:
DrawLine(0, 0, 800, 600, GREEN)
```

### DrawTriangle
```logicodex
DrawTriangle(v1: Vector2, v2: Vector2, v3: Vector2, warna: Color) -> Void
-- Menggambar segitiga
-- Contoh:
DrawTriangle(
    Vector2 { x: 400.0, y: 100.0 },
    Vector2 { x: 300.0, y: 300.0 },
    Vector2 { x: 500.0, y: 300.0 },
    PURPLE
)
```

### DrawPixel
```logicodex
DrawPixel(posX: I32, posY: I32, warna: Color) -> Void
-- Menggambar satu pixel
-- Contoh:
DrawPixel(400, 300, WHITE)
```

### DrawPoly
```logicodex
DrawPoly(pusat: Vector2, sisi: I32, jejari: F32, putaran: F32, warna: Color) -> Void
-- Menggambar poligon beraturan
-- Contoh:
DrawPoly(Vector2 { x: 400.0, y: 300.0 }, 6, 80.0, 0.0, ORANGE)  -- hexagon
```

---

## Input (Keyboard, Mouse) {#input}

### Keyboard

```logicodex
IsKeyPressed(key: I32) -> Bool    -- True semasa frame key ditekan
IsKeyDown(key: I32) -> Bool       -- True selagi key ditekan

-- Key constants
KEY_NULL, KEY_APOSTROPHE, KEY_COMMA, KEY_MINUS, KEY_PERIOD,
KEY_SLASH, KEY_ZERO - KEY_NINE, KEY_SEMICOLON, KEY_EQUAL,
KEY_A - KEY_Z, KEY_LEFT_BRACKET, KEY_BACKSLASH, KEY_RIGHT_BRACKET,
KEY_BACKSPACE, KEY_SPACE, KEY_ENTER, KEY_TAB, KEY_ESCAPE,
KEY_INSERT, KEY_DELETE, KEY_RIGHT, KEY_LEFT, KEY_DOWN, KEY_UP,
KEY_F1 - KEY_F12, KEY_LEFT_SHIFT, KEY_LEFT_CONTROL, KEY_LEFT_ALT
```

### Mouse

```logicodex
GetMousePosition() -> Vector2     -- Dapatkan posisi cursor
IsMouseButtonPressed(button: I32) -> Bool  -- True semasa klik
IsMouseButtonDown(button: I32) -> Bool     -- True selagi butang ditekan
GetMouseWheelMove() -> F32        -- Dapatkan pergerakan scroll wheel

-- Button constants
MOUSE_BUTTON_LEFT, MOUSE_BUTTON_RIGHT, MOUSE_BUTTON_MIDDLE
```

---

## Texture dan Math {#texture}

### Texture

```logicodex
LoadTexture(namaFail: Text) -> Texture2D     -- Muat texture dari fail
UnloadTexture(texture: Texture2D) -> Void    -- Bebaskan texture
DrawTexture(texture: Texture2D, posX: I32, posY: I32, tint: Color) -> Void
DrawTextureEx(texture: Texture2D, position: Vector2, rotation: F32, scale: F32, tint: Color) -> Void
DrawTextureRec(texture: Texture2D, source: Rectangle, position: Vector2, tint: Color) -> Void
DrawTexturePro(texture: Texture2D, source: Rectangle, dest: Rectangle, origin: Vector2, rotation: F32, tint: Color) -> Void
```

### Math Utilities

```logicodex
Clamp(nilai: F32, min: F32, max: F32) -> F32     -- Hadkan nilai
Lerp(start: F32, end: F32, amount: F32) -> F32   -- Interpolasi linear
Normalize(nilai: F32, start: F32, end: F32) -> F32  -- Normalisasi
Remap(nilai: F32, inputStart: F32, inputEnd: F32, outputStart: F32, outputEnd: F32) -> F32
```

### Color Constants

```logicodex
LIGHTGRAY, GRAY, DARKGRAY, YELLOW, GOLD, ORANGE, PINK, RED, MAROON,
GREEN, LIME, DARKGREEN, SKYBLUE, BLUE, DARKBLUE, PURPLE, VIOLET,
DARKPURPLE, BEIGE, BROWN, DARKBROWN, WHITE, BLACK, BLANK, MAGENTA, RAYWHITE
```

### Struct Definitions

```logicodex
STRUKTUR Vector2 { x: F32, y: F32 }
STRUKTUR Rectangle { x: F32, y: F32, width: F32, height: F32 }
STRUKTUR Color { r: U8, g: U8, b: U8, a: U8 }
STRUKTUR Texture2D { id: U32, width: I32, height: I32, mipmaps: I32, format: I32 }
```
