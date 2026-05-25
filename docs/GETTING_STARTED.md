# Logicodex — Getting Started Guide
## For Beginners: From First Line to Production

**Version:** v1.45.0-alpha | **Date:** 2026-05-25

---

## Table of Contents

1. [What is Logicodex?](#what-is-logicodex)
2. [Architecture Overview](#architecture-overview)
3. [Installation](#installation)
4. [Your First Program](#your-first-program)
5. [Language Basics](#language-basics)
6. [Threading & Concurrency](#threading--concurrency)
7. [The Capability System (Security)](#the-capability-system)
8. [Graphics with Raylib](#graphics-with-raylib)
9. [Audio Programming](#audio-programming)
10. [Compilation Targets](#compilation-targets)
11. [Common Patterns](#common-patterns)
12. [Troubleshooting](#troubleshooting)

---

## What is Logicodex?

Logicodex is a **deterministic systems programming language** designed for high-reliability applications — games, embedded systems, network services, and safety-critical software.

### Why "Deterministic"?

Most programming languages allow behavior that depends on runtime conditions (race conditions, memory leaks, undefined behavior). Logicodex eliminates these at **compile time** through:

- **Static topology** — Your program's structure (shards, channels, gates) is known before it runs
- **Explicit ownership** — Every resource has exactly one owner; no accidental sharing
- **Shard isolation** — Parallel code runs on separate CPU cores with no shared mutable state
- **Capability security** — Access to dangerous operations (hardware, network, files) requires explicit permission

### What This Means for You

| Problem in Other Languages | Logicodex Solution |
|---|---|
| Memory leaks | RAII auto-cleanup — memory freed when owner drops |
| Race conditions | Shard isolation — no shared state between parallel units |
| Null pointer crashes | No null pointers by design — use `Option<T>` |
| Use-after-free | Ownership transfer — can't use after giving away |
| Security vulnerabilities | Capability gates — explicit permission for dangerous ops |
| Slow debugging | Deterministic behavior — same input = same output, every time |

---

## Architecture Overview

Logicodex is not just a language — it is a **complete system architecture**. Understanding this helps you use its features effectively.

```
┌─────────────────────────────────────────────────────────────┐
│                    YOUR .ldx SOURCE CODE                      │
│  (functions, variables, unsafe blocks, Raylib calls)         │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│  1. PARSER  →  AST (Abstract Syntax Tree)                   │
│     Validates syntax, extracts capability requirements        │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│  2. SEMANTIC ANALYZER                                       │
│     • Type checking (no implicit conversions)               │
│     • Ownership tracking (who owns what)                    │
│     • Capability verification (do you have permission?)     │
│     • StrictAudioContext (audio callback validation)        │
│     • FfiGatekeeper (unsafe call validation)                │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│  3. HIR LOWERING  →  High-Level IR                          │
│     Normalizes AST into structured intermediate form          │
│     (expressions, control flow, function calls)              │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│  4. CODEGEN  →  LLVM IR  →  Machine Code                    │
│     • Native (ELF)     — Linux/macOS executables            │
│     • WASM             — WebAssembly modules                │
│     • Freestanding     — Bare metal (no OS)                 │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│  5. RUNTIME (if needed)                                     │
│     • Sharded Reactor  — 1 shard per CPU core               │
│     • Network Reactor  — epoll-based I/O (no libc)          │
│     • Host Reactor     — Guest↔Host mediation (WASM)        │
│     • Bump Allocator   — O(1) allocation, no fragmentation  │
└─────────────────────────────────────────────────────────────┘
```

### Key Architectural Principles

**1. Shard Isolation (Parallelism Without Pain)**
```
Traditional threading:       Logicodex shards:
Thread A ←→ shared memory    Shard 0 → Channel → Shard 1
Thread B ←→ (need locks)     (no shared state, ever)
         ↑                    
    Race conditions!          Deterministic, lock-free
```

**2. Gate/Door Security Model**
```
Your Code                    Capability Gates
─────────                    ────────────────
PlaySound()    ──────gate──→  Audio.Main ✓ ALLOWED
InitWindow()   ──────gate──→  Graphics ✓ ALLOWED
fopen()        ──────gate──→  File.IO ✗ DENIED (no capability)
```

**3. Zero-Copy Channels**
```
Channel<T> transfer ownership — data moves, not copies:
  Shard A: let data = [1,2,3]     ← owns data
  Channel.send(data)              ← transfers ownership
  Shard B: let received = recv()  ← now owns data
  Shard A: print(data)            ← COMPILE ERROR! (moved)
```

---

## Installation

### Prerequisites

```bash
# Required
rustc --version    # Rust 1.80+ (rustc --version to check)
cargo --version    # Cargo (comes with Rust)

# For graphics/audio (Raylib)
sudo apt install libraylib-dev     # Ubuntu/Debian
brew install raylib                 # macOS

# For benchmarking (optional)
cargo install cargo-criterion
sudo apt install valgrind           # For memory leak checks
```

### Clone and Build

```bash
git clone https://github.com/MohamadSupardi/Logicodex.git
cd Logicodex

# Build the compiler
cargo build --release

# Verify installation
./target/release/logicodex --version
# Output: Logicodex v1.45.0-alpha

# Run all validators (should show 148/148 passing)
python3 scripts/validators/tier_c_stress/validate_v145_benchmarks.py
```

---

## Your First Program

### Hello World (`hello.ldx`)

```ldx
// hello.ldx — Your first Logicodex program
let name = "World"
print "Hello, " + name + "!"
```

Compile and run:
```bash
./target/release/logicodex hello.ldx -o hello
./hello
# Output: Hello, World!
```

### Variables and Types

```ldx
// Variables are statically typed (inferred)
let count = 42          // i32 (integer)
let pi = 3.14159        // f64 (floating point)
let active = true       // bool
let message = "Hello"   // String

// Explicit types when needed
let score: i64 = 1000000
let rate: f32 = 0.5

// No null — variables must be initialized
// let x: i32         // ERROR: uninitialized variable
// let x: i32 = null  // ERROR: null does not exist
```

### Control Flow

```ldx
// If/else
if score >= 90 {
    print "Excellent!"
} else if score >= 70 {
    print "Good"
} else {
    print "Keep trying"
}

// While loop (with explicit condition)
let i = 0
while i < 10 {
    print i
    i = i + 1
}

// For loop (range-based)
for j in 0..5 {
    print j  // 0, 1, 2, 3, 4
}

// Loop with break/continue
let k = 0
loop {
    k = k + 1
    if k % 2 == 0 { continue }
    if k > 10 { break }
    print k  // 1, 3, 5, 7, 9
}
```

### Functions

```ldx
// Function definition
fn add(a: i32, b: i32) -> i32 {
    return a + b
}

// Void function (no return value)
fn greet(name: String) {
    print "Hello, " + name + "!"
}

// Usage
let result = add(5, 3)
print result        // 8
greet("Logicodex")  // Hello, Logicodex!
```

---

## Language Basics

### Type System

| Type | Description | Example |
|---|---|---|
| `i32` | 32-bit signed integer | `42`, `-7` |
| `i64` | 64-bit signed integer | `1000000000` |
| `f32` | 32-bit float | `3.14` |
| `f64` | 64-bit float | `3.14159265359` |
| `bool` | Boolean | `true`, `false` |
| `String` | UTF-8 string | `"Hello"` |
| `T[]` | Array | `[1, 2, 3]` |
| `()` | Unit (void) | — |

### Operators

```ldx
// Arithmetic
let a = 10 + 5    // 15
let b = 10 - 5    // 5
let c = 10 * 5    // 50
let d = 10 / 5    // 2
let e = 10 % 3    // 1 (modulo)

// Comparison
let eq = (5 == 5)     // true
let ne = (5 != 3)     // true
let gt = (5 > 3)      // true
let lt = (5 < 10)     // true

// Logical
let and = true && false   // false
let or  = true || false   // true
let not = !true           // false
```

### Structs

```ldx
// Define a struct
struct Point {
    x: f32,
    y: f32,
}

// Create instance
let origin = Point { x: 0.0, y: 0.0 }
let p = Point { x: 10.0, y: 20.0 }

// Access fields
print p.x   // 10.0
print p.y   // 20.0

// Struct with methods (in .ldx, use functions)
fn distance(a: Point, b: Point) -> f32 {
    let dx = b.x - a.x
    let dy = b.y - a.y
    // Math functions available via CallableRegistry
    return sqrt(dx * dx + dy * dy)
}
```

---

## Threading & Concurrency

Logicodex uses **actor-model concurrency** — no shared mutable state, ever.

### The Shard Model

```ldx
// Define an actor (concurrent unit)
actor Counter {
    let value = 0
    
    // Handle messages
    on Increment {
        value = value + 1
    }
    
    on GetValue -> i32 {
        return value
    }
}

// Spawn the actor
let counter = spawn Counter

// Send messages (non-blocking)
counter ! Increment
counter ! Increment

// Request-response (blocking with timeout)
let current = counter ? GetValue  // 2
print current
```

### Channels (Direct Communication)

```ldx
// Create a channel between two shards
let (tx, rx) = channel::<String>()

// In Shard A: send data
let message = "Hello from A"
tx.send(message)      // Ownership transfers to channel
// message is now invalid here!

// In Shard B: receive data
let received = rx.recv()   // Ownership transfers to B
print received             // "Hello from A"
```

### Spawn with Affinity

```ldx
// Pin a shard to a specific CPU core
let compute = spawn Worker on core 2

// Shard runs exclusively on core 2
// No cache conflicts with other shards
```

### Why This Design?

| Traditional Threads | Logicodex Shards |
|---|---|
| Shared memory + locks | No shared state |
| Race conditions possible | Deterministic by design |
| Difficult to debug | Easy to reason about |
| Mutex overhead | Lock-free channels |
| Non-deterministic scheduling | Explicit affinity |

---

## The Capability System

Logicodex uses **compile-time capabilities** for security. Dangerous operations require explicit permission.

### Built-in Capabilities

| Capability | Grants Access To |
|---|---|
| `Audio.Main` | Play audio, load sounds/music |
| `Audio.Rakam` | Record audio |
| `Graphics` | Windowing, drawing (Raylib) |
| `Net.Admin` | Network administration |
| `Net.User` | Basic network I/O |
| `File.IO` | File read/write |
| `HW.GPIO` | Hardware GPIO pins |
| `HW.DMA` | Direct memory access |

### Using Capabilities

```ldx
// Declare required capabilities at top of file
// (This is a manifest — the compiler checks it)

// To use Raylib graphics:
unsafe {
    // All Raylib calls require unsafe block (FFI boundary)
    InitWindow(800, 600, "My Game")
    
    while !WindowShouldClose() {
        BeginDrawing()
        ClearBackground(Color(0, 0, 0, 255))
        DrawText("Hello!", 10, 10, 20, Color(255, 255, 255, 255))
        EndDrawing()
    }
    
    CloseWindow()
}
```

### Hardware Zone (Direct Memory Access)

For embedded/bare-metal programming:

```ldx
// Access hardware directly (freestanding target only)
hw_unsafe {
    // Write to VGA text buffer (x86_64 bare metal)
    // Address 0xB8000 = VGA memory
    let vga = 0xB8000 as *mut u8
    *vga = b'H'
    *(vga + 2) = b'i'
}
```

The `hw_unsafe` block:
- Generates **volatile memory operations** (bypass CPU cache)
- Validates addresses at compile time where possible
- Only available with `--target freestanding`

---

## Graphics with Raylib

Logicodex integrates with **Raylib** for graphics. 54 functions are available through the FFI layer.

### Window Setup

```ldx
// examples/window_demo.ldx
unsafe {
    InitWindow(800, 600, "My Logicodex App")
    SetTargetFPS(60)
    
    let white = Color(255, 255, 255, 255)
    let red = Color(255, 0, 0, 255)
    let blue = Color(0, 0, 255, 255)
    
    while !WindowShouldClose() {
        BeginDrawing()
        ClearBackground(Color(0, 0, 0, 255))
        
        // Draw shapes
        DrawRectangle(100, 100, 200, 150, red)
        DrawCircle(400, 300, 50, blue)
        DrawText("Hello Logicodex!", 250, 250, 30, white)
        
        // Check input
        if IsKeyPressed(KEY_SPACE) {
            print "Space pressed!"
        }
        
        if IsMouseButtonPressed(MOUSE_BUTTON_LEFT) {
            let x = GetMouseX()
            let y = GetMouseY()
            print "Click at: " + x + ", " + y
        }
        
        EndDrawing()
    }
    
    CloseWindow()
}
```

### Available Raylib Functions

**Windowing (9):** `InitWindow`, `CloseWindow`, `WindowShouldClose`, `SetTargetFPS`, `GetFPS`, `GetFrameTime`, `GetScreenWidth`, `GetScreenHeight`

**Drawing (9):** `BeginDrawing`, `EndDrawing`, `ClearBackground`, `DrawText`, `DrawRectangle`, `DrawCircle`, `DrawLine`, `DrawRectangleLines`, `DrawPixel`

**Textures (3):** `LoadTexture`, `DrawTexture`, `UnloadTexture`

**Input (7):** `IsKeyDown`, `IsKeyPressed`, `GetKeyPressed`, `IsMouseButtonPressed`, `GetMouseX`, `GetMouseY`, `GetMousePosition`

**Colors:** 26 predefined colors, or create with `Color(r, g, b, a)`

**Math Utilities (safe, no unsafe needed):** `clamp(v, min, max)`, `lerp(a, b, t)`, `remap(v, l1, h1, l2, h2)`, `normalize(v, low, high)`

### Struct Constructors

```ldx
// Color(r, g, b, a) → packed into u32 for efficient GPU transfer
let red = Color(255, 0, 0, 255)

// Vector2(x, y) → 8-byte struct
let pos = Vector2(100.0, 200.0)

// Rectangle(x, y, width, height) → 16-byte struct
let box = Rectangle(50.0, 50.0, 200.0, 100.0)
```

---

## Audio Programming

Logicodex has 22 Raylib audio functions integrated with the **StrictAudioContext** security system.

### Basic Audio

```ldx
// examples/audio_demo.ldx
unsafe {
    InitAudioDevice()
    
    // Load and play a sound (short audio, fully in memory)
    let jump_sound = LoadSound("assets/jump.wav")
    PlaySound(jump_sound)
    
    // Load and stream music (large files, decoded on the fly)
    let music = LoadMusicStream("assets/background.mp3")
    PlayMusicStream(music)
    
    // Main loop — must update music stream every frame
    while !WindowShouldClose() {
        UpdateMusicStream(music)
        
        if IsKeyPressed(KEY_SPACE) {
            PlaySound(jump_sound)
        }
        
        // Control volume
        SetMusicVolume(music, 0.5)  // 50% volume
    }
    
    // Cleanup
    UnloadSound(jump_sound)
    UnloadMusicStream(music)
    CloseAudioDevice()
}
```

### Real-Time Audio (Callbacks)

For synthesizers, audio effects, or procedural audio:

```ldx
// WARNING: Audio callbacks run on the audio thread (ISR-like)
// StrictAudioContext validates these at compile time

// These operations are FORBIDDEN in audio callbacks:
// • Print / DrawText (I/O operations)
// • Self-recursion (calling the callback from itself)
// • Unbounded loops (loop { } without break)
// • malloc / free / spawn (memory/thread operations)

// ALLOWED in audio callbacks:
// • Math operations (add, mul, sin, etc.)
// • Read from pre-allocated buffers
// • Write to output buffer

fn my_audio_callback(buffer: *mut f32, frames: u32) {
    // Generate sine wave
    for i in 0..frames {
        let t = i as f32 * 0.01
        let sample = sin(t * 440.0)  // 440 Hz A note
        unsafe {
            *buffer.offset(i as isize) = sample * 0.5  // 50% volume
        }
    }
}
```

### Audio Functions Reference

| Category | Functions |
|---|---|
| **Device** | `InitAudioDevice`, `CloseAudioDevice`, `IsAudioDeviceReady`, `SetMasterVolume` |
| **Sound** | `LoadSound`, `UnloadSound`, `PlaySound`, `StopSound`, `IsSoundPlaying` |
| **Music** | `LoadMusicStream`, `UnloadMusicStream`, `PlayMusicStream`, `StopMusicStream`, `PauseMusicStream`, `ResumeMusicStream`, `IsMusicStreamPlaying`, `UpdateMusicStream`, `SetMusicVolume`, `SeekMusicStream` |
| **Stream** | `LoadAudioStream`, `UnloadAudioStream`, `PlayAudioStream`, `StopAudioStream`, `IsAudioStreamPlaying` |

---

## Compilation Targets

Logicodex supports **three compilation backends**:

### 1. Native (Default)

```bash
# Compiles to ELF executable for your host OS
./target/release/logicodex myprogram.ldx -o myprogram
./myprogram
```

**Best for:** Desktop applications, games, CLI tools

### 2. WebAssembly (WASM)

```bash
./target/release/logicodex myprogram.ldx --target wasm -o myprogram.wasm
```

**Best for:** Web deployment, sandboxed environments

**Limitations:**
- Raylib graphics functions are **blocked** (WASM has no direct GPU access)
- Use WebGL/Canvas API through the WASM host instead
- Audio goes through `wasi:io/custom` host interface
- Math utilities (`clamp`, `lerp`, `remap`, `normalize`) work normally

**Host integration:** The Host Reactor (v1.41) mediates hardware access:
```
Your WASM code → Host Reactor → GPIO / Timer / DMA
```

### 3. Freestanding (Bare Metal)

```bash
# x86_64 (default)
./target/release/logicodex myprogram.ldx --target freestanding -o kernel.o

# Other architectures
./target/release/logicodex myprogram.ldx --target freestanding-aarch64 -o kernel.o
./target/release/logicodex myprogram.ldx --target freestanding-riscv64 -o kernel.o
```

**Best for:** Operating systems, bootloaders, embedded firmware, hypervisors

**What you get:**
- No OS dependency (no libc, no std::fs, no std::process)
- Custom `_start` entry point (sets stack, zeros BSS, copies data)
- Bump allocator (O(1) allocation, no fragmentation)
- UART output at 0x3F8 (COM1 serial port)
- VGA text mode at 0xB8000 (80×25 character display)
- `hw_unsafe { }` blocks for direct memory-mapped I/O
- Multiboot header for GRUB compatibility

**Linking:**
```bash
# Link with the provided linker script
ld -T lib/linker_scripts/x86_64-freestanding.ld \
   -o kernel.elf myprogram.o

# Run in QEMU
qemu-system-x86_64 -kernel kernel.elf
```

---

## Common Patterns

### Pattern 1: Game Loop with Raylib

```ldx
// Minimal game skeleton
unsafe {
    InitWindow(800, 600, "Game")
    SetTargetFPS(60)
    
    let player_x = 400
    let player_y = 300
    let speed = 5
    
    while !WindowShouldClose() {
        // UPDATE
        if IsKeyDown(KEY_RIGHT) { player_x = player_x + speed }
        if IsKeyDown(KEY_LEFT)  { player_x = player_x - speed }
        if IsKeyDown(KEY_DOWN)  { player_y = player_y + speed }
        if IsKeyDown(KEY_UP)    { player_y = player_y - speed }
        
        // DRAW
        BeginDrawing()
        ClearBackground(Color(20, 20, 20, 255))
        DrawCircle(player_x, player_y, 20, Color(0, 255, 0, 255))
        EndDrawing()
    }
    
    CloseWindow()
}
```

### Pattern 2: Producer-Consumer with Channels

```ldx
// Parallel data processing

// Producer shard
actor Producer {
    let count = 0
    on Produce -> i32 {
        count = count + 1
        return count * count  // Square numbers
    }
}

// Consumer shard
actor Consumer {
    let sum = 0
    on Consume(value: i32) {
        sum = sum + value
        print "Received: " + value + ", Sum: " + sum
    }
}

// Main
let producer = spawn Producer
let consumer = spawn Consumer

for i in 0..10 {
    let value = producer ? Produce
    consumer ! Consume(value)
}
```

### Pattern 3: Bare-Metal Output

```ldx
// Freestanding target: Write to serial port
// No unsafe block needed — freestanding target assumes direct access

fn uart_print(s: String) {
    for ch in s {
        // UART at 0x3F8, wait for transmitter ready, then send
        // (compiler generates this inline)
        uart_send_byte(ch as u8)
    }
}

fn main() {
    uart_print("Hello from bare metal!\n")
    
    // Halt CPU (no OS to return to)
    halt()
}
```

### Pattern 4: Capability-Gated File I/O

```ldx
// File access requires File.IO capability
// (enforced at compile time)

// Read a file
let contents = read_file("data/config.txt")
print contents

// Write a file
write_file("output/result.txt", "Processing complete")

// These would fail WITHOUT the File.IO capability in your manifest:
// read_file("/etc/passwd")   // DENIED — no File.IO capability
```

---

## Troubleshooting

### "Cargo.toml version mismatch" in validators

```bash
# Validators check for version consistency
# If you see this, ensure all docs reference the same version
grep -r "v1.4[0-5]" Cargo.toml README.md CHANGELOG.md
```

### "Raylib function not found"

```bash
# Check Raylib is installed and detected
python3 build.rs  # Shows Raylib detection status

# If not found, set path manually:
export RAYLIB_DIR=/path/to/raylib

# Or skip Raylib (build without it):
export RAYLIB_NO_LINK=1
```

### "Permission denied" on /proc (RSS monitor)

```bash
# RSS monitor reads /proc/[pid]/status
# Run with sufficient privileges:
sudo python3 benches/stability/rss_monitor.py <pid> --interval 60

# Or monitor your own processes (no sudo needed)
python3 benches/stability/rss_monitor.py $$ --interval 60
```

### WASM compilation fails

```bash
# Ensure LLVM WASM backend is available
llc --version | grep wasm

# If missing, install:
sudo apt install llvm-18  # or latest version
```

### "undefined reference to _start" (freestanding)

```bash
# Freestanding programs need the linker script
ld -T lib/linker_scripts/x86_64-freestanding.ld \
   -o kernel.elf myprogram.o

# Missing startup code? Ensure src/os/startup.rs is compiled:
cargo build --release
```

### Slow compilation

```bash
# Use release mode (significantly faster runtime)
cargo build --release

# For development, use debug mode (faster compile)
cargo build
```

### Getting Help

| Resource | Location |
|---|---|
| Architecture docs | `docs/ARCHITECTURE.md` |
| Maintenance report | `docs/MAINTENANCE_v1441.md` |
| Benchmark plan | `docs/BENCHMARK_PLAN_v145.md` |
| RFC template | `docs/RFC_TEMPLATE.md` |
| Changelog | `CHANGELOG.md` |
| Roadmap | `ROADMAP.md` |
| Validators | `scripts/validators/` (Tier A/B/C) |

---

## Quick Reference Card

```
COMPILE
  Native:      logicodex file.ldx -o output
  WASM:        logicodex file.ldx --target wasm -o output.wasm
  Freestanding: logicodex file.ldx --target freestanding -o kernel.o

TYPES
  i32, i64, f32, f64, bool, String, T[]

CONTROL FLOW
  if/else, while, for, loop { break; continue; }

FUNCTIONS
  fn name(param: Type) -> ReturnType { ... }

THREADING
  actor Name { ... }     // Define
  spawn Name             // Instantiate
  shard ! Message        // Send (non-blocking)
  shard ? Request        // Request-response
  channel::<T>()         // Direct channel

RAYLIB (unsafe block required)
  InitWindow(w, h, "title")
  while !WindowShouldClose() { BeginDrawing(); ...; EndDrawing() }
  DrawText/DrawCircle/DrawRectangle/ClearBackground
  Color(r, g, b, a)      // Struct constructor

AUDIO (unsafe block required)
  InitAudioDevice()
  LoadSound/LoadMusicStream/PlaySound/PlayMusicStream
  UpdateMusicStream(music)  // Every frame for music

MATH (safe — no unsafe needed)
  clamp(v, min, max), lerp(a, b, t)
  remap(v, l1, h1, l2, h2), normalize(v, low, high)

VALIDATORS
  Tier A (7):  Core integrity — build stops on failure
  Tier B (13): Feature correctness — warning on failure
  Tier C (8):  Platform/performance — CI only
  Total: 148/148 passing
```

---

*This guide was written for Logicodex v1.45.0-alpha.*
*For the latest updates, check `CHANGELOG.md` and `ROADMAP.md`.*
*Architecture details: `docs/ARCHITECTURE.md`*
*Benchmark framework: `docs/BENCHMARK_PLAN_v145.md`*
