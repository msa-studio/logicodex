# Chapter 1: Pemasangan dan Konfigurasi

Panduan ini membantu anda memasang Logicodex dan menyediakan persekitaran pembangunan.

---

## Keperluan Sistem {#keperluan}

### Minimum

| Komponen | Versi | Keterangan |
|---|---|---|
| **Rust** | 1.75+ | Compiler Logicodex ditulis dalam Rust |
| **LLVM** | 15 | Backend code generation |
| **OS** | Linux x86_64 | Platform utama yang disokong |
| **RAM** | 4GB | Untuk kompilasi |
| **Storage** | 2GB | Untuk source dan build artifacts |

### Disyorkan

| Komponen | Versi | Keterangan |
|---|---|---|
| **Rust** | 1.78+ | Versi terkini |
| **LLVM** | 17 | Optimizer terkini |
| **OS** | Linux x86_64 atau macOS | Kedua-dua disokong penuh |
| **RAM** | 8GB | Untuk kompilasi pantas |
| **Raylib** | 5.0+ | Untuk grafik dan audio (pilihan) |

### Pilihan

| Komponen | Versi | Keterangan |
|---|---|---|
| **wasm-ld** | Terkini | Untuk target WASM |
| **pkg-config** | Terkini | Untuk auto-detect Raylib |
| **Valgrind** | Terkini | Untuk memory leak checking |
| **Python 3** | 3.10+ | Untuk benchmark scripts |

---

## Memasang dari Sumber {#sumber}

### 1. Pasang Rust

```bash
# Menggunakan rustup (disyorkan)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Pastikan versi Rust >= 1.75
rustc --version
```

### 2. Pasang LLVM

```bash
# Ubuntu/Debian
sudo apt-get update
sudo apt-get install llvm-15 llvm-15-dev libclang-15-dev

# macOS
brew install llvm@15

# Arch Linux
sudo pacman -S llvm15
```

### 3. Pasang Raylib (Pilihan)

```bash
# Ubuntu/Debian
sudo apt-get install libraylib-dev

# macOS
brew install raylib

# Dari sumber
export RAYLIB_DIR=/path/to/raylib
```

### 4. Klon dan Bina Logicodex

```bash
# Klon repositori
git clone https://github.com/mymsa/logicodex.git
cd logicodex

# Bina
RUSTFLAGS="-L/usr/lib/llvm-15/lib" cargo build --release

# Hasil binary di:
./target/release/logicodex --version
```

### 5. Verifikasi Pemasangan

```bash
# Jalankan unit tests
cargo test --locked

# Jalankan semua tests
cargo test

# Kompil contoh program
cargo run --release -- examples/01_hello.ldx -o /tmp/hello
/tmp/hello
```

---

## Struktur Projek {#struktur}

```text
logicodex/
├── src/                      # Kod sumber Rust (17,083 LOC)
│   ├── lexer.rs              # Lexer (alias-to-canonical)
│   ├── parser.rs             # Parser (recursive-descent)
│   ├── semantic.rs           # Semantic analyzer
│   ├── hir.rs                # High-level IR
│   ├── codegen.rs            # LLVM code generation
│   ├── ffi/                  # FFI bindings
│   │   ├── raylib.rs         # Raylib safe wrappers
│   │   └── raylib_sys.rs     # Raylib raw bindings
│   ├── tier2/                # Capability + IR
│   │   ├── gate.rs           # Gate logic
│   │   ├── topology.rs       # Topology verification
│   │   ├── capability_ir.rs  # CapabilityGraph IR
│   │   ├── ctl_mapper.rs     # CTL Mapper
│   │   └── shard.rs          # Shard management
│   ├── net/                  # Network reactor
│   │   ├── reactor.rs        # epoll event loop
│   │   ├── connection.rs     # Connection + Taint FSM
│   │   └── sharded_reactor.rs # Sharded reactor
│   └── os/                   # OS-level code
│       ├── syscall.rs        # Direct syscalls
│       ├── startup.rs        # _start for freestanding
│       ├── panic.rs          # Panic handler
│       ├── allocator.rs      # Bump allocator
│       ├── uart.rs           # UART + VGA output
│       ├── interrupts.rs     # IDT + PIC
│       └── target.rs         # Target triple management
├── lib/                      # Library Logicodex (.ldx files)
│   ├── core/                 # Core library
│   │   ├── thread.ldx        # Actor/channel primitives
│   │   ├── sync.ldx          # Synchronization
│   │   ├── ring_buffer.ldx   # SPSC ring buffer
│   │   ├── scheduler.ldx     # Backpressure + scheduling
│   │   ├── memori.ldx        # Memory (Slice, Buffer)
│   │   ├── result.ldx        # Result<T,E>
│   │   ├── file.ldx          # File operations
│   │   └── capability.ldx    # Capability declarations
│   ├── std/                  # Standard library
│   │   └── audio.ldx         # Audio types
│   └── startup/              # Startup code
│       └── multiboot_header.rs # Multiboot header
├── dict/
│   └── core_map.json         # Alias-to-canonical mapping
├── scripts/                  # Validators (6,675 LOC)
│   └── validators/
│       ├── tier_a_core/      # Tier A (7 validators)
│       ├── tier_b_feature/   # Tier B (13 validators)
│       └── tier_c_stress/    # Tier C (8 validators)
├── benches/                  # Benchmark framework
│   ├── BASELINE.json         # Gold standard
│   ├── harness/              # Runner scripts
│   ├── micro/                # Layer 1: micro-benchmarks
│   ├── reactor/              # Layer 2: reactor throughput
│   ├── stability/            # Layer 3: stability tests
│   └── security/             # Layer 4: security stress
├── tests/                    # Unit tests (9,230 LOC)
├── docs/                     # Dokumentasi
│   ├── ARCHITECTURE.md       # Overview arkitektur
│   ├── GETTING_STARTED.md    # Panduan pemula
│   ├── white-paper/          # Wiki: White Paper
│   └── guide/                # Wiki: Functions And Guide (ini)
├── examples/                 # Contoh program .ldx
├── CHANGELOG.md              # Sejarah perubahan
├── ROADMAP.md                # Hala tuju masa depan
├── README.md                 # Overview projek
└── WHITE_PAPER.md            # White paper asal
```

---

## Ringkasan Perintah CLI

```bash
# Kompilasi ke Native (default)
logicodex input.ldx -o output

# Kompilasi ke WASM
logicodex --target wasm input.ldx -o output.wasm

# Kompilasi ke Freestanding
logicodex --target freestanding input.ldx -o kernel.o

# Kompilasi dengan keamanan tambahan
logicodex --secure input.ldx -o output

# Semak semantik sahaja (tanpa codegen)
logicodex --check input.ldx

# Papar LLVM IR (tanpa codegen)
logicodex --emit-ir input.ldx -o output.ll

# Papar bantuan
logicodex --help
```
