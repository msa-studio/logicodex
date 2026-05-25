# Chapter 16: Proses Build Lengkap

Bab ini menerangkan proses build Logicodex dari kod sumber sehingga binari.

---

## Build Script dan Dependensi {#script}

### File `build.rs`

```rust
// build.rs — Logicodex build script
use std::env;
use std::process::Command;

fn main() {
    // 1. Detect Raylib
    if env::var("RAYLIB_NO_LINK").is_ok() {
        println!("cargo:warning=Raylib linking disabled (RAYLIB_NO_LINK=1)");
        return;
    }
    
    // 2. Try pkg-config
    if let Ok(output) = Command::new("pkg-config")
        .args(["--libs", "--cflags", "raylib"])
        .output()
    {
        if output.status.success() {
            let flags = String::from_utf8_lossy(&output.stdout);
            println!("cargo:rustc-link-search=raylib detected via pkg-config");
            for flag in flags.split_whitespace() {
                if flag.starts_with("-L") {
                    println!("cargo:rustc-link-search={}", &flag[2..]);
                } else if flag.starts_with("-l") {
                    println!("cargo:rustc-link-lib={}", &flag[2..]);
                }
            }
            return;
        }
    }
    
    // 3. Try RAYLIB_DIR environment variable
    if let Ok(raylib_dir) = env::var("RAYLIB_DIR") {
        println!("cargo:rustc-link-search={}/lib", raylib_dir);
        println!("cargo:rustc-link-lib=raylib");
        return;
    }
    
    // 4. Platform-specific paths
    #[cfg(target_os = "macos")]
    {
        println!("cargo:rustc-link-search=/opt/homebrew/lib");
        println!("cargo:rustc-link-lib=raylib");
    }
    
    #[cfg(target_os = "linux")]
    {
        println!("cargo:rustc-link-search=/usr/local/lib");
        println!("cargo:rustc-link-lib=raylib");
    }
    
    // 5. Graceful fallback — warning but don't fail
    println!("cargo:warning=Raylib not found — building without graphics support");
}
```

### Dependensi `Cargo.toml`

```toml
[package]
name = "logicodex"
version = "1.45.0-alpha"
edition = "2021"
build = "build.rs"

[dependencies]
inkwell = { version = "0.4", features = ["llvm15-0"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
libc = "0.2"

[dev-dependencies]
criterion = "0.5"

[features]
default = ["raylib"]
raylib = []
wasm = []

[[bin]]
name = "logicodex"
path = "src/main.rs"
```

---

## Kompilasi Cross-Platform {#cross}

### Linux

```bash
# Install dependencies
sudo apt-get install llvm-15-dev libclang-15-dev

# Build
RUSTFLAGS="-L/usr/lib/llvm-15/lib" cargo build --release

# Verify
cargo test --tier a
```

### macOS

```bash
# Install dependencies
brew install llvm@15 raylib

# Build
export RUSTFLAGS="-L$(brew --prefix llvm@15)/lib"
cargo build --release

# Verify
cargo test --tier a
```

### Windows (MSYS2)

```bash
# Install dependencies (dalam MSYS2)
pacman -S mingw-w64-x86_64-llvm mingw-w64-x86_64-raylib

# Build
export RAYLIB_DIR=/mingw64
RUSTFLAGS="-L/mingw64/lib" cargo build --release

# Verify
cargo test --tier a
```

---

## Optimisasi dan Debug {#optimisasi}

### Profile Build

```toml
# Cargo.toml
[profile.release]
opt-level = 3           # Optimisasi maksimum
lto = true              # Link-time optimization
codegen-units = 1       # Kod generation tunggal (lebih optimize)
panic = "abort"         # Abort on panic (lebih kecil)
strip = true            # Strip simbol

[profile.debug]
opt-level = 0           # Tiada optimisasi
debug = true            # Debug info penuh
overflow-checks = true  # Semakan integer overflow
```

### Saiz Binary

| Profile | Saiz | Masa Boot |
|---|---|---|
| `debug` | ~50MB | Lambat |
| `release` | ~5MB | Pantas |
| `release` + strip | ~2MB | Pantas |
| `release` + LTO | ~1.5MB | Paling pantas |

### Tips Optimisasi

```bash
# 1. Build release untuk produksi
cargo build --release

# 2. Check saiz binary
ls -lh target/release/logicodex

# 3. Analisa saiz (memerlukan cargo-bloat)
cargo bloat --release

# 4. Profile dengan perf
perf record ./target/release/logicodex app.ldx
perf report
```

### Debug Build dengan Simbol

```bash
# Build debug
RUSTFLAGS="-g" cargo build

# Debug dengan GDB
gdb ./target/debug/logicodex
(gdb) run app.ldx

# Debug dengan LLDB
lldb ./target/debug/logicodex
(lldb) run app.ldx
```

---

## Makefile

```makefile
# Makefile untuk Logicodex

.PHONY: all build test clean install

all: build

build:
	RUSTFLAGS="$(RUSTFLAGS)" cargo build --release

test:
	cargo test --tier a
	cargo test --tier b

test-all:
	cargo test --tier a
	cargo test --tier b
	cargo test --tier c

install:
	cp target/release/logicodex /usr/local/bin/
	mkdir -p /usr/local/share/logicodex/dict
	cp dict/core_map.json /usr/local/share/logicodex/dict/

clean:
	cargo clean
	rm -f *.o *.wasm *.cap

# Cross-compilation targets
build-x86_64:
	cargo build --release --target x86_64-unknown-linux-gnu

build-aarch64:
	cargo build --release --target aarch64-unknown-linux-gnu

build-wasm:
	cargo build --release --target wasm32-unknown-unknown
```
