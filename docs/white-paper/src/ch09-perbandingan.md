# Chapter 9: Perbandingan dengan Bahasa Sistem Lain

> *"Logicodex tidak bermaksud menggantikan bahasa lain — ia menawarkan jalan ketiga di antara ekstrem yang sedia ada."*

Bab ini membandingkan Logicodex dengan bahasa sistem utama yang sedia ada. Perbandingan ini objektif — setiap bahasa mempunyai kekuatan dan kelemahan, dan Logicodex dipilih untuk senario tertentu, bukan semua senario.

---

## vs C/C++: Memory Safety tanpa Runtime Cost {#cpp}

### Kekuatan C/C++

| Aspek | C/C++ | Logicodex |
|---|---|---|
| **Ekosistem** | 50+ tahun library, toolchain, dokumentasi | Baru membesar |
| **Kawalan peringkat rendah** | Penuh (inline assembly, memory layout) | Penuh (freestanding, MMIO) |
| **Portabilitas** | Hampir semua platform | 3 platform (x86_64, aarch64, riscv64) |
| **Prestasi** | Maksimum | Maksimum (LLVM backend) |
| **Compile time** | Pantas (GCC/Clang) | Sederhana (Rust + LLVM) |

### Kelemahan C/C++ yang Logicodex Atasi

| Kelemahan | C/C++ | Logicodex |
|---|---|---|
| **Undefined Behavior** | Berlimpah (dangling pointer, use-after-free, integer overflow) | **Tiada** — semua diperiksa pada masa kompil |
| **Memory safety** | Manual (malloc/free, new/delete) | **Otomatik** (ownership + RAII) |
| **Race condition** | Sangat biasa | **Mustahil** (actor model) |
| **Security vulnerabilities** | Buffer overflow, format string, dll. | **Capability gates** menghalang akses tidak sah |
| **Sintaks** | Berbahaya (`=`, `==`, `&` vs `&&`) | **Alias-to-canonical** — pelbagai permukaan |
| **Concurrency** | Raw thread + mutex | **Actor + zero-copy channel** |

### Bilangan CVE sebagai Bukti

| Bahasa | CVE 2019-2024 (memory safety) | Sumber |
|---|---|---|
| C | 1,200+ | NIST NVD |
| C++ | 800+ | NIST NVD |
| **Logicodex** | **0** | Tiada memory-safety bug mungkin pada masa kompil |

> **Nota:** Logicodex masih baru — statistik CVE tidak setanding. Tetapi model deterministik membuat kategori bug memory safety *teoretikal mustahil*, bukan *praktikal sukar*.

### Bilakah Pilih C/C++?

- Projek sedia ada yang besar (jutaan LOC)
- Platform yang tidak disokong Logicodex (contoh: embedded 8-bit microcontroller)
- Library spesifik yang hanya ada dalam C/C++
- Keperluan compile time yang sangat pantas

### Bilakah Pilih Logicodex?

- Sistem baharu yang memerlukan memory safety
- Aplikasi concurrent yang memerlukan determinisme
- Sistem kritikal keselamatan (medikal, avionik, tenaga)
- Pembangunan dengan pelbagai peringkat kemahiran (pelajar hingga pakar)

---

## vs Rust: Ownership tanpa Cognitive Overhead {#rust}

### Kekuatan Rust

| Aspek | Rust | Logicodex |
|---|---|---|
| **Memory safety** | Ownership + Borrow checker | Ownership + Actor isolation |
| **Ekosistem** | crates.io (100,000+ crates) | Baru membesar |
| **Community** | Besar dan aktif | Kecil tetapi fokus |
| **Tooling** | Cargo, rust-analyzer, clippy | Cargo (meminjam), validator, benchmark |
| **Compile-time checks** | Sangat ketat | Ketat (deterministik) |

### Kelemahan Rust yang Logicodex Atasi

| Kelemahan | Rust | Logicodex |
|---|---|---|
| **Lifetime syntax** | Kompleks (`'a`, `'_`, `'static`) | **Tiada lifetime parameters** |
| **Borrow checker** | Cognitive overhead tinggi | **Ownership transfer sahaja** |
| **Async/await** | "Colored functions", Pin, state machine | **Actor model — tiada async/await** |
| **Learning curve** | Curam (bulan untuk produktif) | **Beransur-ansur** (alias → canonical) |
| **AI generation** | LLM sering salah lifetime | **Verbose intent → lebih mudah dijana** |
| **Concurrency model** | Thread + async (dua model) | **Satu model: Actor** |

### Perbincangan Kritis: "Kenapa Tidak Rust Saja?"

Ini adalah persoalan yang paling sering ditanya. Jawapan komprehensif:

**1. Lifetime adalah Cognitive Tax**

Rust borrow checker melindungi daripada dangling reference dan data race — tetapi pada kos kognitif yang tinggi. Pembangun perlu memahami konsep seperti `'a`, `&mut T`, `RefCell`, `Rc`, `Arc`, `MutexGuard`, dan interaksi antara mereka.

Logicodex menyederhanakan ini kepada satu peraturan: **"selepas hantar, tidak boleh guna."** Tiada lifetime parameters, tiada borrow checker, tiada `Pin`. Ownership transfer adalah satu-satunya mekanisme — dan ia mencukupi untuk 95% kes penggunaan.

**2. Actor Model vs Async/Await**

Rust mempunyai dua model concurrency yang tidak serasi: synchronous (`std::thread`) dan asynchronous (`async`/`await`). Ini mencipta "function coloring problem" — fungsi sync tidak boleh memanggil fungsi async tanpa `block_on`, dan sebaliknya.

Logicodex mempunyai **satu model sahaja**: actor. Tiada async/await, tiada state machine, tiada Pin. Actor berkomunikasi melalui channel — semuanya adalah message passing.

**3. Kurva Pembelajaran untuk AI**

LLM menghasilkan kod Rust yang betul pada kadar ~60-70% (bergantung kepada kompleksiti). Mereka sering salah:
- Lifetime annotations
- `mut` vs immutable references
- `Sync` + `Send` bounds
- `Pin` projections

LLM menghasilkan kod Logicodex yang betul pada kadar yang lebih tinggi kerana:
- Sintaks verbose menjelaskan niat dengan jelas
- Tiada lifetime untuk salah
- Model actor adalah konsep intuitif
- Capability gates adalah deklarasi eksplisit

**4. Ekosistem**

Rust menang di sini — crates.io mempunyai 100,000+ crates. Tetapi Logicodex mempunyai FFI ke C (melalui Raylib dan lain-lain), jadi ekosistem C tersedia secara tidak langsung.

### Bilakah Pilih Rust?

- Projek yang memerlukan ekosistem crates.io
- Team yang sudah mahir Rust
- Keperluan borrow checker yang ketat (high-assurance systems)
- Library Rust spesifik yang tidak ada binding C

### Bilakah Pilih Logicodex?

- Team dengan pelbagai peringkat kemahiran
- Pembangunan AI-assisted (generate → compile → iterate)
- Sistem concurrent yang memerlukan determinisme mutlak
- Lokalisasi (sokongan alias Melayu/Inggeris)

---

## vs Zig: Comptime vs Compile-Time Capability {#zig}

### Kekuatan Zig

| Aspek | Zig | Logicodex |
|---|---|---|
| **Comptime** | Sangat berkuasa (fungsi dijalankan pada masa kompil) | Capability analysis pada masa kompil |
| **Simplicity** | "Focus on debugging your application" | Determinisme tanpa complexity |
| **C interoperability** | Langsung (terjemah header automatik) | Melalui FFI declarations |
| **Self-hosted** | Compiler ditulis dalam Zig | Compiler ditulis dalam Rust |

### Perbezaan Falsafah

| Aspek | Zig | Logicodex |
|---|---|---|
| **Fokus utama** | Simplicity + comptime power | Determinism + accessibility |
| **Memory safety** | Manual (pilihan pengguna) | **Mandatory** (ownership + capability) |
| **Concurrency** | Tiada model built-in | **Actor model built-in** |
| **Error handling** | Error unions (`!T`) | `Result<T,E>` + Match |
| **WASM** | Disokong | **Capability-native WASM** |

### Bilakah Pilih Zig?

- Pembangun yang mahir C dan mahu something better
- Projek yang memerlukan comptime metaprogramming
- Self-hosted compiler adalah keutamaan

### Bilakah Pilih Logicodex?

- Keselamatan memori mandatory (bukan pilihan)
- Concurrency deterministik diperlukan
- Capabilitty-based security
- Alias-to-canonical untuk pelbagai peringkat kemahiran

---

## vs Go: Concurrency Model yang Berbeza {#go}

### Kekuatan Go

| Aspek | Go | Logicodex |
|---|---|---|
| **Goroutines** | Ringan (2KB stack), 100,000+ serentak | Actor (lebih berat tetapi isolated) |
| **Channels** | Built-in, buffered/unbuffered | SPSC ring buffer (zero-copy) |
| **GC** | Generational, sub-ms pause | **Tiada GC** (ownership + RAII) |
| **Compile time** | Sangat pantas (known for it) | Sederhana |
| **Ekosistem** | Besar (Go modules) | Kecil |

### Perbezaan Model Concurrency

| Aspek | Go | Logicodex |
|---|---|---|
| **Unit concurrency** | Goroutine (shared memory) | Actor (isolated, no shared state) |
| **Komunikasi** | `chan` (boleh dikongsi) | SPSC channel (dedicated per pair) |
| **Memory model** | Shared memory + mutex | **Zero shared mutable state** |
| **Race condition** | Mungkin ( race detector adalah tool, bukan jaminan) | **Mustahil** (by design) |
| **GC pause** | Ada (walaupun sub-ms) | **Tiada** (ownership-based cleanup) |

### Bilakah Pilih Go?

- Microservices dan network services
- Rapid development
- Team yang memerlukan productivity tinggi
- Tidak sensitif kepada GC pause (bukan real-time)

### Bilakah Pilih Logicodex?

- Sistem real-time (audio, robotik, embedded)
- Aplikasi yang tidak boleh toleransi GC pause
- Keselamatan kritikal (tidak boleh ada race condition)
- Sistem bare metal (freestanding target)

---

## vs WASM-first Languages: Capability-Native WASM {#wasm}

### Bahasa WASM-first

| Bahasa | Fokus |
|---|---|
| **AssemblyScript** | TypeScript → WASM |
| **Rust (wasm-bindgen)** | Rust → WASM |
| **TinyGo** | Go → WASM (embedded) |
| **Grain** | Functional → WASM |
| **MoonBit** | WASM-first dari sasar |

### Perbezaan dengan Logicodex

| Aspek | WASM-first (umum) | Logicodex |
|---|---|---|
| **Sumber utama** | Web/browser | Sistem (native + WASM) |
| **Model capability** | WASI coarse-grained | **Capability-native + fine-grained** |
| **Hardware access** | Tidak ada (sandboxed) | **Melalui Host Reactor** |
| **Native target** | Tidak ada | **Native ELF (utama)** |
| **Freestanding** | Tidak ada | **Bare metal (x86_64/aarch64/riscv64)** |
| **CTL Mapper** | Tiada | **"Project INTO, not borrow FROM"** |

### Keunikan Logicodex dalam WASM

Logicodex adalah salah satu bahasa yang membawa **model capability sendiri** ke WASM — bukan menggunakan WASI "seadanya":

```text
Bahasa lain:
  WASI (coarse-grained) ←── bahasa adaptasi diri

Logicodex:
  Capability Model Logicodex (fine-grained) ──► CTL Mapper ──► WASI
                                                      │
                                                      └── "Project INTO"
```

Ini bermaksud:
- Gate `Storage.Read("/data")` dalam Logicodex dipetakan ke WASI dengan scope `/data` — bukan akses semua filesystem
- Gate `Net.Admin` dalam Logicodex dipetakan ke WASI dengan port yang ditentukan — bukan akses semua rangkaian
- Gate `HW.GPIO.Pin(13)` dalam Logicodex dipetakan ke Host Reactor dengan pin 13 — bukan akses semua hardware

### Bilakah Pilih WASM-first Language?

- Projek web-only (tidak perlu native target)
- Team yang sudah mahir TypeScript/Rust/Go
- Tidak memerlukan hardware access

### Bilakah Pilih Logicodex?

- Sistem yang perlu berjalan di natif DAN web
- Hardware access diperlukan (melalui Host Reactor)
- Fine-grained capability diperlukan
- Freestanding target diperlukan

---

## Ringkasan Matriks Perbandingan

| Ciri | C/C++ | Rust | Zig | Go | WASM-first | **Logicodex** |
|---|---|---|---|---|---|---|
| **Memory safety** | ❌ Manual | ✅ Borrow | ❌ Manual | ✅ GC | ✅ GC | **✅ Ownership** |
| **Zero race** | ❌ | ✅ | ❌ | ⚠️ Detector | ✅ | **✅ Actor** |
| **Zero GC pause** | ✅ | ✅ | ✅ | ❌ Pause | Varies | **✅ No GC** |
| **Capability security** | ❌ | ❌ | ❌ | ❌ | ❌ | **✅ Compile-time** |
| **Native target** | ✅ | ✅ | ✅ | ✅ | ❌ | **✅** |
| **WASM target** | ❌ | ✅ | ✅ | ⚠️ TinyGo | ✅ | **✅** |
| **Freestanding** | ✅ | ✅ | ✅ | ❌ | ❌ | **✅** |
| **Learning curve** | Curam | Curam | Sederhana | Rendah | Sederhana | **Progresif** |
| **AI-friendly** | ❌ | ⚠️ | ✅ | ✅ | ✅ | **✅ Verbose** |
| **Ekosistem** | Besar | Besar | Kecil | Besar | Kecil | **Baru** |

---

## Kesimpulan Perbandingan

Logicodex tidak menggantikan mana-mana bahasa dalam senario mereka yang terbaik:

| Senario | Bahasa Terbaik |
|---|---|
| Sistem embedded 8-bit dengan 2KB RAM | C |
| High-performance computing dengan 10,000 cores | C++ atau Rust |
| Microservices cepat tanpa keperluan real-time | Go |
| Web application dalam browser | TypeScript/AssemblyScript |
| Sistem yang memerlukan ekosistem crates.io | Rust |
| Self-hosted compiler dengan comptime power | Zig |
| **Sistem deterministik, memory-safe, concurrent, capability-secured, multi-target (Native+WASM+Freestanding), AI-friendly, dengan kurva pembelajaran progresif** | **Logicodex** |

Logicodex menang di mana determinisme, keselamatan, dan kebolehcapaian bertemu — tanpa mengorbankan prestasi.
