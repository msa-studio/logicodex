# Chapter 17: Pustaka Standard

Ringkasan pustaka standard Logicodex yang tersedia untuk semua program.

---

## Modul `core` — Primitif Asas {#core}

### Memori

```logicodex
-- lib/core/memori.ldx

STRUKTUR Slice<T> {
    data: PTR<T>,
    panjang: USize,
}

STRUKTUR Buffer<T> {
    data: PTR<T>,
    kapasiti: USize,
    panjang: USize,
}

FUNGSI Slice::baru(data: PTR<T>, panjang: USize) -> Slice<T>
FUNGSI Slice::kosong() -> Slice<T>
FUNGSI Slice::get(&self, indeks: USize) -> Option<&T>
FUNGSI Slice::set(&mut self, indeks: USize, nilai: T) -> Void

FUNGSI Buffer::baru(kapasiti: USize) -> Buffer<T>
FUNGSI Buffer::tambah(&mut self, nilai: T) -> Result<Void, Overflow>
FUNGSI Buffer::kosongkan(&mut self) -> Void
FUNGSI Buffer::sebagai_slice(&self) -> Slice<T>
```

### Sinkronisasi

```logicodex
-- lib/core/sync.ldx

FUNGSI tidur_ms(ms: U64) -> Void           -- Tidur selama ms milisaat
FUNGSI tidur_us(us: U64) -> Void           -- Tidur selama us mikro saat
FUNGSI yield_thread() -> Void              -- Yield kepada thread lain

FUNGSI atomic_tambah(ptr: &mut I32, nilai: I32) -> I32   -- Atomic add
FUNGSI atomic_banding_tukar(ptr: &mut I32, ekspek: I32, baru: I32) -> Result<I32, I32>
```

### Ring Buffer (SPSC)

```logicodex
-- lib/core/ring_buffer.ldx

STRUKTUR RingBuffer<T> {
    buffer: Buffer<T>,
    kepala: AtomicUSize,    -- write pointer
    ekor: AtomicUSize,      -- read pointer
}

FUNGSI RingBuffer::baru(kapasiti: USize) -> RingBuffer<T>
FUNGSI RingBuffer::hantar(&self, nilai: T) -> Void               -- blocking
FUNGSI RingBuffer::terima(&self) -> T                            -- blocking
FUNGSI RingBuffer::cuba_hantar(&self, nilai: T) -> Result<Void, Penuh>  -- non-blocking
FUNGSI RingBuffer::cuba_terima(&self) -> Option<T>                      -- non-blocking
FUNGSI RingBuffer::terima_timeout_ms(&self, timeout: U64) -> Option<T>  -- dengan timeout
FUNGSI RingBuffer::kosong(&self) -> Bool
FUNGSI RingBuffer::penuh(&self) -> Bool
```

### Penjadualan

```logicodex
-- lib/core/scheduler.ldx

ENUMERASI DasarBackpressure {
    Halang,
    Gugur_Terlama,
    Ralat,
}

FUNGSI dapat_core_count() -> I32           -- Jumlah core CPU
FUNGSI core_sekarang() -> I32              -- Core yang menjalankan kod ini
FUNGSI parallelisme_tersedia() -> I32      -- Parallelisme (biasanya = core count)
FUNGSI tetap_affinity(core: I32) -> Void   -- Tetapkan CPU affinity
```

---

## Modul `std` — Pustaka Standard {#std}

### Audio

```logicodex
-- lib/std/audio.ldx

ENUMERASI AudioViolation {
    PelanggaranIo,
    PelanggaranRekursi,
    PelanggaranLoopTakTerbatas,
    PelanggaranPanggilanDihalang,
}

STRUKTUR StrictAudioContext {
    pelanggaran: Vec<AudioViolation>,
}

FUNGSI StrictAudioContext::semak(fungsi: &HIRFunction) -> Result<Void, Vec<AudioViolation>>
```

### Fail

```logicodex
-- lib/core/file.ldx

STRUKTUR FailHandle {
    fd: I32,
}

FUNGSI buka_fail(laluan: Text, mod: ModAkses) -> Result<FailHandle, Text>
FUNGSI baca_fail(handle: &FailHandle, buffer: &mut [U8]) -> Result<I32, Text>
FUNGSI tulis_fail(handle: &FailHandle, data: &[U8]) -> Result<I32, Text>
FUNGSI tutup_fail(handle: FailHandle) -> Void    -- RAII: automatik jika drop
FUNGSI saiz_fail(laluan: Text) -> Result<I64, Text>

ENUMERASI ModAkses {
    Baca,
    Tulis,
    BacaTulis,
    CiptaTulis,
}
```

### Rangkaian

```logicodex
-- src/net/*.rs (Rust-level API)

STRUKTUR Connection {
    fd: I32,
    taint: KeadaanTaint,
}

ENUMERASI KeadaanTaint {
    Sihat,
    Mencurigakan { sejak: Masa },
    Menutup,
}

FUNGSI Connection::baru(fd: I32) -> Connection
FUNGSI Connection::baca(&self, buffer: &mut [U8]) -> Result<I32, Text>
FUNGSI Connection::tulis(&self, data: &[U8]) -> Result<I32, Text>
FUNGSI Connection::tutup(self) -> Void          -- RAII: automatik jika drop
FUNGSI Connection::aktif(&self) -> Bool
FUNGSI Connection::dipercayai(&self) -> Bool
FUNGSI Connection::keadaan_taint(&self) -> KeadaanTaint
```

---

## Modul `ffi` — Foreign Function Interface {#ffi}

### C ABI

```logicodex
-- Deklarasi FFI
FFI c "nama_library" {
    FN nama_fungsi(param1: Tipe1, param2: Tipe2) -> ReturnType
}

-- Contoh: Library matematik
FFI c "m" {
    FN cos(x: F64) -> F64
    FN sin(x: F64) -> F64
    FN tan(x: F64) -> F64
    FN sqrt(x: F64) -> F64
    FN pow(x: F64, y: F64) -> F64
    FN log(x: F64) -> F64
    FN exp(x: F64) -> F64
    FN fabs(x: F64) -> F64
    FN floor(x: F64) -> F64
    FN ceil(x: F64) -> F64
    FN fmod(x: F64, y: F64) -> F64
}
```

### Raylib

Semua 54 fungsi Raylib (lihat Chapter 13 dan Jadual Fungsi Lengkap di halaman akhir).

### CallableRegistry

Fungsi FFI didaftarkan dalam CallableRegistry:

```rust
// src/ffi/raylib.rs — Safe wrappers
register_raylib_functions(registry: &mut CallableRegistry);
register_raylib_audio_functions(registry: &mut CallableRegistry);

// Setiap fungsi didaftarkan dengan:
// - Nama
// - Tipe parameter dan return
// - Safety level (Safe / UnsafeRequired)
// - ABI (C calling convention)
```

---

## Fungsi Built-in (Intrinsic)

Fungsi yang tersedia tanpa import:

```logicodex
-- Memori
salin<T>(src: &T, dst: &mut T) -> Void       -- Salin data
pindah<T>(src: T, dst: &mut T) -> Void       -- Pindahkan ownership
saiz<T>() -> USize                            -- Saiz tipe dalam byte

-- Matematik
abs(x: I32) -> I32
abs_f(x: F64) -> F64
min(a: I32, b: I32) -> I32
max(a: I32, b: I32) -> I32
min_f(a: F64, b: F64) -> F64
max_f(a: F64, b: F64) -> F64

-- Tipe
sebagai<T, U>(nilai: T) -> U                  -- Type cast (hanya cast selamat)
jenis<T>() -> Text                            -- Nama tipe sebagai string

-- Debug
assert(syarat: Bool) -> Void                  -- Panic jika false
assert_eq<T>(a: T, b: T) -> Void             -- Panic jika tidak sama
pasca_keadaan(syarat: Bool) -> Void           -- Post-condition check
```
