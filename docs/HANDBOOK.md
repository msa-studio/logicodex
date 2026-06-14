> ⚠️ **NOT UPDATED — will revisit.** This document predates the current syntax/architecture and may contain stale information. Authoritative current references: `README.md`, `examples/`, and `docs/architecture/`. Tracked under `docs/DOCUMENTATION_POLICY.md`.

# Logicodex Handbook

**User Guide · Tutorials · API Reference · Troubleshooting**

Version: v0.46.0-alpha  
Scope: This document teaches you how to use Logicodex. For the language specification and architecture, see [`../SPECIFICATION.md`](../SPECIFICATION.md). For philosophical justifications, see [`white-paper/`](white-paper/).

---

## Table of Contents

1. [Installation](#1-installation)
2. [Your First Program](#2-your-first-program)
3. [Syntax Reference](#3-syntax-reference)
4. [Types](#4-types)
5. [Control Flow](#5-control-flow)
6. [Functions](#6-functions)
7. [Actors and Concurrency](#7-actors-and-concurrency)
8. [Capability Gates](#8-capability-gates)
9. [Raylib Graphics](#9-raylib-graphics)
10. [Audio Programming](#10-audio-programming)
11. [Compilation Targets](#11-compilation-targets)
12. [Standard Library](#12-standard-library)
13. [Recipes](#13-recipes)
14. [Troubleshooting](#14-troubleshooting)

---

## 1. Installation

### Requirements

| Component | Minimum | Recommended |
|---|---|---|
| Rust | 1.75+ | 1.78+ |
| LLVM | 15 | 17 |
| OS | Linux x86_64 | Linux x86_64 or macOS |
| RAM | 4GB | 8GB |
| Raylib (optional) | — | 5.0+ |

### Install

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install LLVM (Ubuntu)
sudo apt-get install llvm-15-dev libclang-15-dev

# Install Raylib (optional, for graphics/audio)
sudo apt-get install libraylib-dev

# Clone and build
git clone https://github.com/msa-studio/logicodex.git
cd logicodex
RUSTFLAGS="-L/usr/lib/llvm-15/lib" cargo build --release

# Verify
./target/release/logicodex --version
```

### CLI Reference

```bash
logicodex input.ldx -o output              # Native (default)
logicodex --target wasm input.ldx -o out   # WebAssembly
logicodex --target freestanding input.ldx  # Bare metal
logicodex --check input.ldx                # Semantic check only
logicodex --emit-ir input.ldx -o out.ll    # LLVM IR output
logicodex --help                           # Full help
```

---

## 2. Your First Program

### Hello World (3 Styles)

**Malay:**
```logicodex
PROGRAM hello
FUNGSI utama() -> I32
MULA
    PAPAR "Halo, Logicodex!"
    PULANG 0
TAMAT
TAMAT PROGRAM
```

**English:**
```logicodex
PROGRAM hello
FUNCTION main() -> I32
BEGIN
    DISPLAY "Hello, Logicodex!"
    RETURN 0
END
END PROGRAM
```

**Canonical (Expert):**
```logicodex
program hello {
    fn main() -> I32 {
        print "Hello, Logicodex!";
        return 0;
    }
}
```

All three compile to **identical machine code**.

### Compile and Run

```bash
logicodex hello.ldx -o hello
./hello
# → Hello, Logicodex!
```

---

## 3. Syntax Reference

### Variable Declaration

```logicodex
BINA x SEBAGAI I32 = 10         -- Malay
let x: I32 = 10                  -- Canonical
CREATE x AS I32 = 10            -- English
```

### If / Else

```logicodex
JIKA x > 5
    PAPAR "Besar"
LAIN_JIKA x > 2
    PAPAR "Sederhana"
LAIN
    PAPAR "Kecil"
TAMAT_JIKA
```

### Match

```logicodex
PADAN status {
    Status::Ok(val)  => PULANG val,
    Status::Err(msg) => PAPAR msg,
    _                => PULANG 0,
}
```

### Loops

```logicodex
-- For loop
UNTUK i DARI 0 HINGGA 10
    PAPAR i
TAMAT_UNTUK

-- While loop
SEMENTARA x < 100
    x = x + 1
TAMAT_SEMENTARA
```

### Full Alias Table

| Purpose | Malay | English | Canonical |
|---|---|---|---|
| Begin block | `MULA` | `BEGIN` / `START` | `{` |
| End block | `TAMAT` | `END` / `FINISH` | `}` |
| Declare var | `BINA` | `CREATE` / `LET` | `let` |
| Print | `PAPAR` | `DISPLAY` / `PRINT` | `print` |
| Return | `PULANG` | `RETURN` | `return` |
| Function | `FUNGSI` | `FUNCTION` | `fn` |
| Program | `PROGRAM` | `PROGRAM` | `program` |
| If | `JIKA` | `IF` | `if` |
| Else | `LAIN` | `ELSE` | `else` |
| Else if | `LAIN_JIKA` | `ELSE_IF` | `else if` |
| For | `UNTUK` | `FOR` | `for` |
| While | `SEMENTARA` | `WHILE` | `while` |
| Break | `PATAH` | `BREAK` | `break` |
| Continue | `TERUS` | `CONTINUE` | `continue` |
| Match | `PADAN` | `MATCH` | `match` |
| True | `BENAR` | `TRUE` | `true` |
| False | `PALSU` | `FALSE` | `false` |
| Spawn | `HIDUPKAN` | `SPAWN` | `spawn` |
| Actor | `PELAKON` | `ACTOR` | `actor` |
| Channel | `SALURAN` | `CHANNEL` | `channel` |
| Service | `PERKHIDMATAN` | `SERVICE` | `service` |

---

## 4. Types

### Primitive Types

| Type | Size | Example |
|---|---|---|
| `I8` | 8-bit | `42i8` |
| `I16` | 16-bit | `1000i16` |
| `I32` | 32-bit | `42` |
| `I64` | 64-bit | `9000000000i64` |
| `F32` | 32-bit float | `3.14f32` |
| `F64` | 64-bit float | `3.14` |
| `Bool` | boolean | `true`, `false` |
| `Text` | string | `"hello"` |

### Composite Types

```logicodex
-- Struct
STRUKTUR Point {
    x: F64,
    y: F64,
}

-- Enum
ENUMERASI Status {
    Aktif,
    TidakAktif,
    Khas(I32),
}

-- Array
BINA arr SEBAGAI [I32; 5] = [1, 2, 3, 4, 5]
PAPAR arr[0]  -- 1

-- Option
Option::Some(42)
Option::None

-- Result
Result::Ok(data)
Result::Err("error message")
```

---

## 5. Control Flow

```logicodex
FUNGSI gred(markah: I32) -> Text
MULA
    JIKA markah >= 90
        PULANG "A+"
    LAIN_JIKA markah >= 80
        PULANG "A"
    LAIN_JIKA markah >= 70
        PULANG "B"
    LAIN
        PULANG "F"
    TAMAT_JIKA
TAMAT
```

---

## 6. Functions

```logicodex
-- Basic function
FUNGSI tambah(a: I32, b: I32) -> I32
MULA
    PULANG a + b
TAMAT

-- Generic function
FUNGSI pertukar<T>(a: &T, b: &T) -> Void
MULA
    BINA temp SEBAGAI T = *a
    *a = *b
    *b = temp
TAMAT
```

---

## 7. Actors and Concurrency

### Define an Actor

```logicodex
PELAKON Counter {
    BINA nilai SEBAGAI I32 = 0
    saluran: Channel<CounterMsg>
}

ENUMERASI CounterMsg {
    Tambah(I32),
    Dapatkan,
    Reset,
}
```

### Spawn and Communicate

```logicodex
BINA ch SEBAGAI Channel<CounterMsg> = Channel::baru(100)
BINA counter SEBAGAI Counter = Counter { nilai: 0, saluran: ch }
HIDUPKAN counter

ch.hantar(CounterMsg::Tambah(5))
ch.hantar(CounterMsg::Dapatkan)   -- prints: 5
```

### Important: Ownership Transfer

```logicodex
-- After send, you cannot use the data anymore
BINA data SEBAGAI Buffer<U8> = Buffer::new(4096)
ch.hantar(data)
-- ❌ PAPAR data[0]  -- COMPILE ERROR: UseAfterSend

-- To keep a copy, clone before sending
BINA salinan SEBAGAI Buffer<U8> = data.salin()
ch.hantar(data)
PAPAR salinan[0]  -- ✅ OK
```

---

## 8. Capability Gates

### Declare a Service with Gates

```logicodex
PERKHIDMATAN WebServer {
    port: 8080,
    keperluan: [
        Net.Admin,              -- network admin gate
        Storage.Read("/www"),   -- scoped file read
    ],
    pengendali: handle_http,
    dasar: Halang,               -- Block policy
}
```

### Backpressure Policies

| Policy | When Full | Use Case |
|---|---|---|
| `Block` / `Halang` | Wait for space | Reliable delivery |
| `DropOldest` / `Gugur_Terlama` | Drop oldest data | Real-time streaming |
| `Error` / `Ralat` | Return error | Caller handles overflow |

---

## 9. Raylib Graphics

### Quick Start

```logicodex
program graphics {
    fn main() -> I32 {
        InitWindow(800, 600, "My App")
        SetTargetFPS(60)

        while !WindowShouldClose() {
            BeginDrawing()
            ClearBackground(RAYWHITE)
            DrawText("Hello!", 190, 200, 20, DARKGRAY)
            DrawRectangle(100, 100, 200, 150, BLUE)
            DrawCircle(400, 300, 50, RED)
            EndDrawing()
        }

        CloseWindow()
        return 0
    }
}
```

### Key Functions (54 total)

| Category | Functions |
|---|---|
| **Window** | `InitWindow`, `CloseWindow`, `WindowShouldClose`, `SetTargetFPS`, `GetScreenWidth`, `GetScreenHeight` |
| **Drawing** | `ClearBackground`, `BeginDrawing`, `EndDrawing`, `DrawText`, `DrawRectangle`, `DrawRectangleLines`, `DrawCircle`, `DrawLine`, `DrawPixel`, `DrawTriangle`, `DrawPoly` |
| **Input** | `GetMousePosition`, `IsMouseButtonPressed`, `IsMouseButtonDown`, `GetMouseWheelMove`, `IsKeyPressed`, `IsKeyDown` |
| **Texture** | `LoadTexture`, `UnloadTexture`, `DrawTexture`, `DrawTextureEx`, `DrawTextureRec`, `DrawTexturePro` |
| **Math** | `Clamp`, `Lerp`, `Normalize`, `Remap` |

### Color Constants

`LIGHTGRAY`, `GRAY`, `DARKGRAY`, `YELLOW`, `GOLD`, `ORANGE`, `PINK`, `RED`, `MAROON`, `GREEN`, `LIME`, `DARKGREEN`, `SKYBLUE`, `BLUE`, `DARKBLUE`, `PURPLE`, `VIOLET`, `DARKPURPLE`, `BEIGE`, `BROWN`, `DARKBROWN`, `WHITE`, `BLACK`, `BLANK`, `MAGENTA`, `RAYWHITE`

### Key Constants

`KEY_UP`, `KEY_DOWN`, `KEY_LEFT`, `KEY_RIGHT`, `KEY_W`, `KEY_A`, `KEY_S`, `KEY_D`, `KEY_SPACE`, `KEY_ENTER`, `KEY_ESCAPE`, `KEY_F1`-`KEY_F12`

---

## 10. Audio Programming

### Audio Lifecycle

```logicodex
InitAudioDevice()           -- Must call first
-- ... load/play sounds ...
CloseAudioDevice()          -- Must call last
```

### Sound (Short Effects)

```logicodex
BINA jump SEBAGAI Sound = LoadSound("jump.wav")
PlaySound(jump)
-- ... when done ...
UnloadSound(jump)
```

### Music (Streaming)

```logicodex
BINA music SEBAGAI Music = LoadMusicStream("bgm.mp3")
PlayMusicStream(music)
SetMusicVolume(music, 0.5)

while !WindowShouldClose() {
    UpdateMusicStream(music)  -- Call every frame!
    -- ... render ...
}

StopMusicStream(music)
UnloadMusicStream(music)
```

### Audio Stream with Callback

```logicodex
BINA stream SEBAGAI AudioStream = LoadAudioStream(44100, 16, 1)
SetAudioStreamCallback(stream, my_callback)
PlayAudioStream(stream)
```

### ⚠️ StrictAudioContext: 4 Rules for Callbacks

Audio callbacks **must** follow these rules (enforced at compile time):

| Rule | Forbidden | Error |
|---|---|---|
| **No I/O** | `print`, `DrawText`, `InitWindow` | `AudioViolationIo` |
| **No Recursion** | Calling itself | `AudioViolationRecursion` |
| **No Unbounded Loops** | `loop { }` without termination | `AudioViolationUnboundedLoop` |
| **No Forbidden Calls** | `malloc`, `free`, `spawn` | `AudioViolationForbiddenCall` |

```logicodex
-- ✅ CORRECT callback
FUNGSI audio_callback(buffer: &mut [F32], frames: U32) -> Void
MULA
    UNTUK i DARI 0 HINGGA frames
        buffer[i] = sin(2.0 * PI * 440.0 * i SEBAGAI F64 / 44100.0) SEBAGAI F32
    TAMAT_UNTUK
TAMAT
```

---

## 11. Compilation Targets

| Target | Command | Output |
|---|---|---|
| **Native (Linux)** | `logicodex input.ldx -o out` | ELF executable |
| **Native (macOS)** | `logicodex input.ldx -o out` | Mach-O executable |
| **WASM** | `logicodex --target wasm input.ldx -o out.wasm` | WebAssembly module |
| **Freestanding x86_64** | `logicodex --target freestanding input.ldx` | Object file |
| **Freestanding aarch64** | `logicodex --target freestanding-aarch64 input.ldx` | Object file |
| **Freestanding riscv64** | `logicodex --target freestanding-riscv64 input.ldx` | Object file |

### Linking Freestanding

```bash
# Compile
logicodex --target freestanding boot.ldx -o boot.o

# Link with linker script
ld -T lib/linker_scripts/x86_64-freestanding.ld -o kernel.bin boot.o

# Create boot image
objcopy -O binary kernel.bin kernel.img

# Run in QEMU
qemu-system-x86_64 -kernel kernel.img
```

---

## 12. Standard Library

### Core Module (`lib/core/`)

| File | Provides |
|---|---|
| `thread.ldx` | Actor, Channel, spawn, send, recv, join |
| `sync.ldx` | sleep_ms, sleep_us, yield_thread, atomic operations |
| `ring_buffer.ldx` | SPSC ring buffer (send, recv, try_send, try_recv, timeout_recv) |
| `scheduler.ldx` | Backpressure policy, get_core_count, set_affinity |
| `memori.ldx` | Slice<T>, Buffer<T>, memory operations |
| `result.ldx` | Result<T,E>, Option<T> |
| `file.ldx` | open_file, read_file, write_file, close_file |
| `capability.ldx` | Gate declarations, topology verify |

### Audio Module (`lib/std/`)

| File | Provides |
|---|---|
| `audio.ldx` | StrictAudioContext, audio types |

### FFI Module (`src/ffi/`)

| File | Provides |
|---|---|
| `raylib.rs` | 54 Raylib function safe wrappers |
| `raylib_sys.rs` | Raw C bindings |

---

## 13. Recipes

### HTTP Server

```logicodex
PERKHIDMATAN HttpServer {
    port: 8080,
    keperluan: [Net.Admin, Storage.Read("/www")],
    pengendali: handle_http,
    dasar: Halang,
}

FUNGSI handle_http(conn: Connection) -> Void
MULA
    BINA buf SEBAGAI [U8; 4096] = [0; 4096]
    BINA n SEBAGAI I32 = conn.baca(&mut buf)
    JIKA n <= 0
        PULANG
    TAMAT_JIKA
    BINA req SEBAGAI Text = buf SEBAGAI Text
    BINA resp SEBAGAI Text
    JIKA req.mengandung("GET / ")
        resp = "HTTP/1.1 200 OK\r\n\r\n<h1>Logicodex</h1>"
    LAIN
        resp = "HTTP/1.1 404\r\n\r\nNot Found"
    TAMAT_JIKA
    conn.tulis(resp SEBAGAI &[U8])
TAMAT
```

### Interactive Graphics

```logicodex
program game {
    fn main() -> I32 {
        InitWindow(800, 600, "Game")
        SetTargetFPS(60)

        let player = { x: 400.0, y: 300.0, speed: 5.0 }

        while !WindowShouldClose() {
            if IsKeyDown(KEY_UP)    || IsKeyDown(KEY_W) { player.y -= player.speed }
            if IsKeyDown(KEY_DOWN)  || IsKeyDown(KEY_S) { player.y += player.speed }
            if IsKeyDown(KEY_LEFT)  || IsKeyDown(KEY_A) { player.x -= player.speed }
            if IsKeyDown(KEY_RIGHT) || IsKeyDown(KEY_D) { player.x += player.speed }

            player.x = Clamp(player.x, 20, 780)
            player.y = Clamp(player.y, 20, 580)

            BeginDrawing()
            ClearBackground(RAYWHITE)
            DrawCircle(player.x as I32, player.y as I32, 20, BLUE)
            EndDrawing()
        }
        CloseWindow()
        return 0
    }
}
```

---

## 14. Troubleshooting

### Common Errors

| Error | Meaning | Fix |
|---|---|---|
| `E001: Type mismatch` | Wrong type used | Check type annotations, use `SEBAGAI` cast |
| `E002: Division by zero` | Dividing by constant 0 | Check divisor before division |
| `E003: Unknown symbol` | Variable not declared | Declare before use |
| `E004: UseAfterSend` | Used value after sending | Clone before send if needed |
| `E005: UseAfterMove` | Used ownership after move | Use reference `&T` instead of value |
| `E006: Gate not permitted` | Missing capability gate | Add `requires: [Gate.Name]` to service/actor |
| `E007: AudioViolation` | Audio callback broke rules | Remove I/O/recursion/allocation from callback |
| `E008: WASM hardware gate` | HW gate in WASM target | Route through `logicodex:host-reactor` |

### Runtime Issues

| Symptom | Cause | Fix |
|---|---|---|
| Program hangs | Channel full, blocking send | Use `try_send` or increase capacity |
| High memory | Not unloading textures/audio | Call `UnloadTexture`/`UnloadMusicStream` |
| Audio not playing | `UpdateMusicStream` not called | Call every frame in game loop |
| Window blank | Missing `BeginDrawing`/`EndDrawing` | Ensure paired calls |

### Build Issues

| Error | Fix |
|---|---|
| `LLVM not found` | `export LLVM_SYS_150_PREFIX=/usr/lib/llvm-15` |
| `cannot find -lraylib` | Install Raylib or `RAYLIB_NO_LINK=1` |
| `unwrap on None` | This is a Logicodex bug — please report with backtrace |

---

*Logicodex Handbook — v0.46.0-alpha*  
*For the full specification: [`SPECIFICATION.md`](../SPECIFICATION.md)*  
*For detailed justifications: [`white-paper/`](white-paper/)*  
*For function reference: [`guide/`](guide/)*
