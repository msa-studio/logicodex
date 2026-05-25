# Chapter 15: Target Kompilasi

Logicodex menyokong tiga target kompilasi: Native, WASM, dan Freestanding.

---

## Native (ELF) {#native}

### Default Target

```bash
# Tanpa --target (default = native)
logicodex input.ldx -o output

# Atau secara eksplisit
logicodex --target native input.ldx -o output
```

### Output

| Platform | Format | Extension |
|---|---|---|
| Linux x86_64 | ELF executable | (tiada) |
| Linux aarch64 | ELF executable | (tiada) |
| macOS x86_64 | Mach-O | (tiada) |
| macOS aarch64 | Mach-O | (tiada) |

### Linked dengan

| Library | Dipakai oleh |
|---|---|
| `libc` (pilihan) | `printf` fallback |
| `libm` | Fungsi matematik (`sin`, `cos`, `sqrt`) |
| `libraylib` (pilihan) | Grafik dan audio |
| `libpthread` | Threading (walaupun kita gunakan direct syscall) |

### Contoh

```bash
# Kompilasi program biasa
logicodex hello.ldx -o hello
./hello

# Kompilasi dengan Raylib
logicodex game.ldx -o game
./game  # Memerlukan libraylib terpasang

# Kompilasi tanpa Raylib (fallback)
RAYLIB_NO_LINK=1 logicodex hello.ldx -o hello_no_gfx
```

---

## WebAssembly (WASM) {#wasm}

### Kompilasi ke WASM

```bash
logicodex --target wasm input.ldx -o output.wasm

# Linking dengan wasm-ld
wasm-ld --no-entry -o final.wasm output.wasm --export-all
```

### LLVM Features

```text
+bulk-memory     — Bulk memory operations (memcpy, memfill)
+mutable-globals — Mutable global variables
+sign-ext        — Sign-extension operators
```

### Batasan WASM

| Ciri | Native | WASM | Catatan |
|---|---|---|---|
| **File I/O** | ✅ Direct syscall | ✅ WASI filesystem | Melalui wasi:filesystem |
| **Network** | ✅ Direct syscall | ✅ WASI sockets | Melalui wasi:sockets |
| **Hardware** | ✅ Direct MMIO | ❌ **Dihalang** | Melalui Host Reactor sahaja |
| **Threading** | ✅ Native threads | ⚠️ Web Workers | WASM threads belum stabil |
| **Raylib** | ✅ Full | ❌ **Dihalang** | WASM blocks Raylib functions |
| **Freestanding** | ✅ | ❌ | WASM memerlukan host |

### Gate Hardware dalam WASM

```logicodex
-- ❌ RALAT: Gate hardware tidak dibenarkan dalam target WASM
PERKHIDMATAN BadWASM {
    keperluan: [HW.GPIO],  -- ❌ E008: Hardware gate in WASM
}

-- ✅ BETUL: Gunakan Host Reactor untuk hardware
PERKHIDMATAN GoodWASM {
    keperluan: [logicodex:host-reactor],  -- ✅ Melalui Host Reactor
}
```

### CTL Mapper Output

```bash
# Kompilasi + generate WIT
logicodex --target wasm --emit-wit app.ldx -o app.wasm
# Menghasilkan: app.wasm + app.wit
```

---

## Freestanding (Bare Metal) {#freestanding}

### Kompilasi Freestanding

```bash
# Default: x86_64
logicodex --target freestanding input.ldx -o kernel.o

# Spesifik arkitektur
logicodex --target freestanding-x86_64 input.ldx -o kernel.o
logicodex --target freestanding-aarch64 input.ldx -o kernel.o
logicodex --target freestanding-riscv64 input.ldx -o kernel.o
```

### LLVM Triple

| Arkitektur | LLVM Triple | Code Model |
|---|---|---|
| x86_64 | `x86_64-unknown-none` | Kernel |
| aarch64 | `aarch64-unknown-none` | Small |
| riscv64 | `riscv64gc-unknown-none-elf` | Medium |

### Ciri-ciri Freestanding

| Ciri | Hosted | Freestanding |
|---|---|---|
| `_start` | Disediakan OS | Anda tulis sendiri |
| `panic` | Disediakan OS | Anda tulis handler |
| `malloc` | Disediakan libc | Bump allocator |
| `printf` | Disediakan libc | UART/VGA output |
| `std` Rust | ✅ Penuh | ❌ Hanya `core` + `alloc` |

### Contoh Program Freestanding

```logicodex
PROGRAM baremetal

GUNA_JENIS I32
GUNA_JENIS PTR<U16>
GUNA_JENIS U16

-- VGA text buffer pada 0xB8000
TANDA KAWASAN_PERKAKAS VGA_TEXT SEBAGAI PTR<U16> = ALAMAT 0xB8000

FUNGSI tulis_vga(str: &[U8]) -> Void
MULA
    UNTUK i DARI 0 HINGGA str.panjang()
        -- 0x07 = light gray on black
        TULIS_VOLATIL(VGA_TEXT + i SEBAGAI U64, (0x07 SEBAGAI U16 << 8) | str[i] SEBAGAI U16)
    TAMAT_UNTUK
TAMAT

FUNGSI mula_sistem() -> I32
MULA
    tulis_vga("Logicodex Bare Metal!")
    
    -- Loop selamanya
    SEMENTARA BENAR
        -- halt
    TAMAT_SEMENTARA
    
    PULANG 0
TAMAT

TAMAT PROGRAM
```

### Linking untuk Boot

```bash
# Kompilasi
logicodex --target freestanding boot.ldx -o boot.o

# Link dengan linker script
ld -T lib/linker_scripts/x86_64-freestanding.ld -o kernel.bin boot.o

# Buat image boot
objcopy -O binary kernel.bin kernel.img

# Jalankan dengan QEMU
qemu-system-x86_64 -kernel kernel.img
```

---

## Ringkasan Target

```bash
# Native (default)
logicodex input.ldx -o output

# WASM
logicodex --target wasm input.ldx -o output.wasm

# Freestanding x86_64
logicodex --target freestanding input.ldx -o kernel.o

# Freestanding aarch64
logicodex --target freestanding-aarch64 input.ldx -o kernel.o

# Freestanding riscv64
logicodex --target freestanding-riscv64 input.ldx -o kernel.o
```
