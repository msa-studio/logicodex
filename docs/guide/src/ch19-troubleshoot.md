# Chapter 19: Penyelesaian Masalah

Panduan penyelesaian masalah untuk isu umum Logicodex.

---

## Ralat Kompilasi Umum {#kompilasi}

### Ralat: "Unknown symbol"

```
error[E001]: Unknown symbol
  --> file.ldx:10:5
   |
10 |     PAPAR x
   |         ^ unknown symbol 'x'
```

**Penyelesaian:**
- Pastikan variable `x` diisytiharkan sebelum digunakan
- Periksa ejaan variable
- Periksa skop — mungkin variable diisytiharkan dalam blok berbeza

```logicodex
-- ❌ SALAH
FUNGSI utama() -> I32
MULA
    PAPAR x     -- x belum diisytiharkan!
    BINA x SEBAGAI I32 = 5
    PULANG 0
TAMAT

-- ✅ BETUL
FUNGSI utama() -> I32
MULA
    BINA x SEBAGAI I32 = 5
    PAPAR x     -- ✅ x sudah diisytiharkan
    PULANG 0
TAMAT
```

### Ralat: "Type mismatch"

```
error[E002]: Type mismatch
  --> file.ldx:12:15
   |
12 |     BINA x: I32 = 3.14
   |                   ^^^^ expected I32, found F64
```

**Penyelesaian:**
- Gunakan suffix tipe: `3.14f32` atau `42i32`
- Gunakan cast: `3.14 SEBAGAI I32`
- Pastikan tipe variable betul

```logicodex
-- ✅ BETUL
BINA a SEBAGAI I32 = 42i32
BINA b SEBAGAI F64 = 3.14f64
BINA c SEBAGAI I32 = 3.14 SEBAGAI I32  -- cast dari F64 ke I32
```

### Ralat: "Division by zero"

```
error[E003]: Division by zero
  --> file.ldx:15:12
   |
15 |     PULANG a / 0
   |            ^^^^^ division by zero
```

**Penyelesaian:**
- Semak pembahagi sebelum bahagian
- Gunakan Result untuk handle error

```logicodex
FUNGSI bahagian_selamat(a: I32, b: I32) -> Result<I32, Text>
MULA
    JIKA b == 0
        PULANG Result::Err("Pembahagi sifar!")
    TAMAT_JIKA
    PULANG Result::Ok(a / b)
TAMAT
```

### Ralat: "UseAfterSend"

```
error[E005]: UseAfterSend
  --> file.ldx:20:5
   |
18 |     ch.hantar(data)
   |     --------------- data ownership moved here
19 |
20 |     PAPAR data
   |     ^^^^^^^^^^ use after send
```

**Penyelesaian:**
- Clone data sebelum hantar jika masih diperlukan
- Struktur kod supaya data tidak diperlukan selepas hantar

```logicodex
-- ✅ BETUL: Clone jika masih diperlukan
BINA data_salinan SEBAGAI Buffer<U8> = data.salin()
ch.hantar(data)
proses(data_salinan)   -- gunakan salinan

-- ✅ BETUL: Struktur semula
ch.hantar(data)
-- Tiada lagi penggunaan data selepas ini
```

---

## Masalah Runtime {#runtime}

### Program Hang / Tidak Responsif

| Punca | Penyelesaian |
|---|---|
| Channel blocking penuh | Guna `cuba_hantar` atau ubah ke `Gugur_Terlama` |
| Infinite loop | Pastikan ada condition terminate |
| Missing `yield` | Tambah `yield_thread()` dalam loop panjang |

### Memory Usage Tinggi

| Punca | Penyelesaian |
|---|---|
| Tidak Unload texture/audio | Panggil `UnloadTexture` / `UnloadMusicStream` |
| Clone berlebihan | Guna reference (`&T`) atau ownership transfer |
| Channel terlalu besar | Kurangkan kapasiti channel |

### Crash tanpa Ralat

| Punca | Penyelesaian |
|---|---|
| Stack overflow | Kurangkan rekursi atau saiz variable local |
| Invalid memory access | Periksa pointer dan bounds |
| Missing `CloseWindow` / `CloseAudioDevice` | Pastikan cleanup dipanggil |

---

## Masalah FFI dan Raylib {#ffi}

### Ralat: "Raylib function not found"

```
error: Raylib function 'DrawCircle' not found in CallableRegistry
```

**Penyelesaian:**
- Pastikan Raylib terpasang: `pkg-config --exists raylib && echo "OK"`
- Tetapkan `RAYLIB_DIR` jika Raylib di lokasi tidak standard
- Gunakan `RAYLIB_NO_LINK=1` untuk build tanpa Raylib

### Ralat: "FFI type mismatch"

```
error: FFI type mismatch for 'cos': expected F64, got F32
```

**Penyelesaian:**
- Pastikan tipe parameter sepadan dengan deklarasi C
- Gunakan suffix: `3.14f64` bukan `3.14f32`

### Grafik Tidak Papar

| Punca | Penyelesaian |
|---|---|
| Tiada `BeginDrawing`/`EndDrawing` | Pastikan berpasangan |
| Tiada `ClearBackground` | Panggil sebelum draw lain |
| Window terlalu cepat tutup | Tambah loop `!WindowShouldClose()` |

---

## Masalah WASM {#wasm}

### Ralat: "Hardware gate in WASM"

```
error[E008]: Hardware gate in WASM
  --> file.ldx:15:5
   |
15 |     keperluan: [HW.GPIO],
   |               ^^^^^^^^^^ hardware gate not allowed in WASM target
```

**Penyelesaian:**
- Gunakan Host Reactor untuk akses hardware dari WASM
- Pisahkan kod hardware ke modul berasingan

```logicodex
-- ❌ SALAH dalam WASM
PERKHIDMATAN Bad {
    keperluan: [HW.GPIO],
}

-- ✅ BETUL dalam WASM
PERKHIDMATAN Good {
    keperluan: [logicodex:host-reactor],
}
```

### Ralat: "WASM module too large"

**Penyelesaian:**
- Gunakan `wasm-opt` untuk optimize
- Kurangkan kod yang tidak diperlukan
- Guna `wasm-strip` untuk buang debug info

```bash
# Optimize dengan wasm-opt
wasm-opt -O3 -o output_optimized.wasm output.wasm

# Strip debug info
wasm-strip output.wasm
```

---

## Penyelesaian Masalah Berdasarkan Simptom

| Simptom | Kemungkinan Punca | Penyelesaian Cepat |
|---|---|---|
| `error while loading shared libraries` | Library tidak di dalam LD_LIBRARY_PATH | `export LD_LIBRARY_PATH=/usr/local/lib:$LD_LIBRARY_PATH` |
| `cannot find -lraylib` | Raylib tidak terpasang | Pasang Raylib atau `RAYLIB_NO_LINK=1` |
| `LLVM not found` | LLVM tidak di PATH | `export LLVM_SYS_150_PREFIX=/usr/lib/llvm-15` |
| `panicked at 'called Option::unwrap()'` | Bug dalam Logicodex | Laporkan issue dengan backtrace |
| Program berhenti tiba-tiba | `assert` gagal | Semak condition assert |
| Audio tidak keluar | Audio device sibuk | Tutup aplikasi audio lain |
| Frame rate rendah | Terlalu banyak draw calls | Kurangkan objek atau gunakan texture |

---

## Dapatkan Bantuan

| Saluran | Cara |
|---|---|
| **GitHub Issues** | https://github.com/mymsa/logicodex/issues |
| **Email** | mymsastudio@gmail.com |
| **Dokumentasi** | docs/white-paper/ dan docs/guide/ |
| **Contoh Kod** | examples/ dalam repositori |
| **Validator** | `cargo test --tier a` untuk semak integriti |
