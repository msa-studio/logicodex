# Chapter 3: Dari Compiler Core ke Sistem Platform — Sejarah v1.21 hingga v1.45

> *"Logicodex bukan sekadar bahasa pengaturcaraan — ia adalah 'Hardware-Integrated Systems Platform' yang menggabungkan 6 tiang utama."*

Bab ini merakam perjalanan evolusi Logicodex melalui 9 fasa pembangunan, dari compiler core alpha sehingga sistem platform lengkap dengan framework benchmark kuantitatif. Setiap fasa mewakili keputusan arkitektur yang dilalui perbincangan mendalam antara arkitek manusia dan AI assistant.

---

## Fasa 1: Compiler Core v1.21 — Asas {#fasa1}

**Tarikh:** Awal 2026  
**Milestone:** Compiler core berfungsi — lexer, parser, AST, semantic analyzer, LLVM backend  
**Validator:** 9/9 checks passing

### Apa yang Dibina

Fasa 1 menetapkan asas compiler: `dict/core_map.json`-aware lexer, recursive-descent parser, AST construction, semantic analyzer, dan LLVM-Inkwell backend. Ini bukan proof-of-concept — ini adalah compiler core yang menghasilkan binari natif daripada kod `.ldx`.

### Komponen Utama

| Komponen | Fail | Tanggungjawab |
|---|---|---|
| Lexer | `src/lexer.rs` | Tokenisasi dengan `core_map.json` |
| Parser | `src/parser.rs` | Recursive-descent → AST |
| Semantic Analyzer | `src/semantic.rs` | Type checking, name resolution |
| LLVM Backend | `src/codegen.rs` | AST → LLVM IR → object file |
| Core Map | `dict/core_map.json` | Alias-to-canonical mapping |

### Refleksi Perbincangan

> *"Kita mesti pastikan compiler core ini kukuh sebelum tambah apa-apa fitur. Kalau asas tidak kukuh, bangunan akan runuh."*

Keputusan untuk menggunakan LLVM (melalui Inkwell binding Rust) dan bukan backend sendiri dicapai setelah pertimbangan: LLVM menyediakan 30+ tahun kejuruteraan optimizer dan code generation, menyokong semua target utama (x86_64, ARM64, WASM), dan mempunyai ekosistem tooling yang matang. Membina backend sendiri akan memerlukan 5-10 tahun untuk mencapai parity.

### Output
- 9 validator checks
- Contoh program: `print` arithmetic, variable binding, block scope
- Target: Linux x86_64 ELF

---

## Fasa 2: Threading + IO + Audio v1.30 — Demo Threading {#fasa2}

**Milestone:** Actor-model concurrency, zero-copy ownership transfer, 4-Ketuk IO architecture  
**Validator:** 104/104 checks passing (baseline 9 + threading 95)

### Apa yang Dibina

Fasa 2 membawa Logicodex dari "compiler yang menghasilkan program tunggal" ke "platform yang menyokong concurrency deterministik". Ini adalah perubahan paradigma besar — bukan sekadar menambah thread, tetapi menentukan model concurrency yang unik.

### 3 Fasa Threading

| Fasa | Komponen | Status |
|---|---|---|
| **Phase 1** (v1.30.1) | Actor & Channel, topology validation | 8/8 checks |
| **Phase 2** (v1.30.2) | Zero-copy ownership transfer via SPSC ring buffer | 6/6 checks |
| **Phase 3** (v1.30.3) | Backpressure policies (Block/DropOldest/Error) + scheduler | 10/10 checks |

### IO Architecture: 4-Ketuk (K1-K4)

| Ketuk | Komponen | Deskripsi |
|---|---|---|
| **K1** | Core Memory | `Slice<T>`, `Buffer<T>`, provenance tracking |
| **K2** | Result Abstraction | `Result<T,E>` dengan `Ok`/`Err`/`Match` |
| **K3+K4** | File Handle + Syscall | `SYS_READ`/`SYS_WRITE` melalui `src/os/syscall.rs` |

### Audio Engine: Hardware-Safe Guards

Audio engine diperkenalkan dengan **StrictAudioContext** — 4 jenis pelanggaran yang dilarang dalam audio callback:
- `AudioViolationIo` — tiada Print/DrawText/InitWindow dalam ISR audio
- `AudioViolationRecursion` — tiada panggilan rekursif sendiri
- `AudioViolationUnboundedLoop` — tiada loop tak terbatas
- `AudioViolationForbiddenCall` — tiada malloc/free/spawn

### Refleksi Perbincangan: Kenapa Actor Model?

> *"Kenapa tidak gunakan thread biasa dengan mutex? Semua bahasa buat begitu."*

Jawapan kita: **mutex adalah anti-deterministik.** Dua thread yang berkongsi mutable state melalui mutex mempunyai tingkah laku yang bergantung pada urutan penjadualan — yang tidak dapat diramalkan. Ini melanggar Prinsip 1 (Determinisme Absolute).

Model actor Logicodex:
- Setiap actor adalah unit komputasi terpencil (isolated computation unit)
- Komunikasi hanya melalui channel (tidak ada shared state)
- Ownership transfer adalah zero-copy (tidak ada salinan data)
- Ring buffer SPSC adalah lock-free (tidak ada mutex)

Ini menghasilkan concurrency yang **100% deterministik** — urutan penjadualan tidak mengubah output, kerana tidak ada state dikongsi.

---

## Fasa 3: Capability Security v1.32 — Pintu Keselamatan {#fasa3}

**Milestone:** Static Capability Fabric — Gate/Door split, compile-time verification  
**Validator:** 10/10 checks passing

### Apa yang Dibina

Fasa 3 memperkenalkan **Static Capability Fabric** — sistem keselamatan berasaskan keupayaan yang semua semakannya berlaku pada masa kompil, meninggalkan **kos runtime sifar**.

### Komponen Capability

| Komponen | Fail | Fungsi |
|---|---|---|
| **Gate** | `src/tier2/gate.rs` | Kontrak keselamatan masa kompil |
| **Topology** | `src/tier2/topology.rs` | Verifikasi topology, privilege escalation detection |
| **Door** | `src/tier2/shard.rs` | Cross-shard data transport (SPSC) |

### 3 Jenis Gate

| Jenis | Apa | Contoh |
|---|---|---|
| **DirectCall** | Inline-able sync | Math, crypto |
| **Message** | Async SPSC | Sensor, network |
| **Hardware** | Bare-metal sahaja | GPIO, DMA |

### Supply-Chain Security

Setiap kompilasi menghasilkan fail `.cap` — audit trail yang merekodkan semua capability yang digunakan. Fungsi `diff_topology()` boleh mengesan peningkatan keistimewaan (privilege escalation) antara versi.

### Refleksi Perbincangan

> *"Sistem keselamatan mesti zero-cost. Kalau ada overhead runtime, kita gagal."*

Ini adalah keputusan yang menentukan. Kita menolak semua model keselamatan yang melibatkan runtime check — termasuk sandboxing, capability list dinamik, dan permission runtime. Semua gate hanya wujud dalam IR dan topology verify. Sebaik sahaja program dikompil, **gate tidak lagi wujud** — hanya kod natif yang optimum tinggal.

---

## Fasa 4: Network Reactor v1.33-v1.37 — I/O Deterministik {#fasa4}

**Milestone:** Deterministic event-driven networking — RAII cleanup, taint FSM, epoll  
**Validator:** 29/29 checks passing (v1.33: 13, v1.37: 16)

### Apa yang Dibina

Fasa 4 menutup jurang antara compile-time verification dan runtime execution untuk operasi rangkaian. v1.33 memperkenalkan reactor model; v1.37 menjadikannya "hidup" dengan epoll dan syscall langsung.

### v1.33: Network Reactor (Compile-Time)

| Komponen | Fungsi |
|---|---|
| **RAII Connection** | `close(fd)` automatik apabila connection drop — **tiada kebocoran socket** |
| **Taint FSM** | `Healthy → Suspicious → Closing` untuk setiap connection |
| **Service Manifest** | Deklarasi service dengan `port`, `requires`, `handler`, `policy` |
| **Backpressure** | `Block`/`DropOldest`/`Error` policies pada ring buffer |

### v1.37: Network Runtime (LIVE)

| Komponen | Implementasi |
|---|---|
| **epoll event loop** | `epoll_create1`, `epoll_ctl`, `epoll_wait` via **direct syscall** (tiada libc) |
| **Socket I/O** | `SYS_RECV`, `SYS_SEND` via `src/os/syscall.rs` |
| **Monotonic timestamp** | `clock_gettime(CLOCK_MONOTONIC)` untuk taint timeout |
| **Event processing** | `EPOLLIN`/`EPOLLOUT`/`EPOLLERR`/`EPOLLHUP` dispatch |

### Refleksi Perbincangan: Direct Syscall vs libc

> *"Kenapa guna syscall langsung? libc lebih mudah dan portable."*

Jawapan: **libc adalah lapisan abstraksi yang memperkenalkan ketidakpastian.** Setiap panggilan libc melibatkan:
1. Dynamic linking yang bergantung pada versi glibc
2. Tingkah laku yang berbeza antara platform
3. Overhead fungsi pembungkus (wrapper)

Dengan direct syscall, Logicodex berhubung terus dengan kernel Linux. Ini memberikan:
- Determinisme (tingkah laku sama pada semua kernel Linux yang sama versi)
- Zero dependency (tidak perlu libc untuk freestanding)
- Kos minimum (tiada wrapper function)

Ini adalah keputusan "hardcore" tetapi diperlukan untuk Prinsip 2 (Zero Runtime Mediation).

---

## Fasa 5: Sharded Runtime v1.34-v1.39 — Multi-Core {#fasa5}

**Milestone:** Per-CPU-core reactor instances, real OS threads, CPU affinity  
**Validator:** 21/21 checks passing (v1.34: 11, v1.39: 10)

### Apa yang Dibina

Fasa 5 membolehkan Logicodex menggunakan semua core CPU dengan cara yang deterministik — setiap shard berjalan pada core yang ditetapkan, tidak ada migrasi thread, tidak ada contention.

### v1.34: Sharded Reactor (Compile-Time)

| Komponen | Fungsi |
|---|---|
| **ShardTopology** | Peta statik shard → core pada masa kompil |
| **Per-core instances** | Setiap core mempunyai reactor sendiri |
| **Cross-shard doors** | Channel SPSC antara shard |
| **Memory budgeting** | Batasan memori per-shard |

### v1.39: Sharded Runtime (LIVE)

| Komponen | Implementasi |
|---|---|
| **Real OS threads** | `std::thread::spawn` per shard dengan `JoinHandle` tracking |
| **CPU affinity** | `sched_setaffinity` (Linux), `thread_policy_set` (macOS), `SetThreadAffinityMask` (Windows) |
| **Dynamic core detection** | `available_parallelism()` |
| **Current core query** | `sched_getcpu` |

### Refleksi Perbincangan: Kenapa Sharding, Bukan Thread Pool?

> *"Thread pool lebih efisien — thread dikongsi, bukan diikat ke core."*

Jawapan: **thread pool adalah non-deterministik.** Sebuah tugas mungkin berjalan pada core 0 pada panggilan pertama dan core 3 pada panggilan kedua. Ini membuat cache behavior tidak diramalkan, timing tidak konsisten, dan debugging sukar.

Sharding Logicodex:
- Setiap shard diikat ke satu core (static affinity)
- Tidak ada migrasi thread
- Tidak ada konteks tukar antara core untuk shard yang sama
- Cache behavior konsisten (data sentiasa pada core yang sama)

Ini adalah trade-off: kita mengorbankan flexibilitasi penjadualan untuk determinisme. Untuk sistem yang memerlukan determinisme (robotik, audio real-time, sistem kawalan), ini adalah trade-off yang betul.

---

## Fasa 6: WASM + Host Reactor v1.40-v1.41 — Sandbox {#fasa6}

**Milestone:** Kompilasi ke WebAssembly, Guest ↔ Host hardware mediation  
**Validator:** 33/33 checks passing (v1.40: 13, v1.41: 20)

### v1.40: WASM Codegen Backend

| Aspek | Implementasi |
|---|---|
| **Target triple** | `wasm32-unknown-unknown` |
| **LLVM features** | `+bulk-memory,+mutable-globals,+sign-ext` |
| **CLI** | `--target wasm` |
| **Linking** | `wasm-ld --no-entry` |

### v1.41: Host Reactor

| Komponen | Fungsi |
|---|---|
| **HostReactor** | Mediasi akses hardware dari guest WASM |
| **GatePermissions** | Per-operation pin allowlists |
| **HardwareZone** | Pin claim/release tracking (menghalang double-use) |
| **HostFunction** | Enum: `GpioControl`, `TimerSet`, `DmaTransfer` |
| **Dispatch** | `GuestRequest`/`HostResponse` serialization |

### Prinsip Kritikal: "Project INTO, Not Borrow FROM"

> *"Kita tidak meminjam model WASI dan memaksakan programmer Logicodex memahaminya. Kita memetakan model capability Logicodex KE DALAM ekosistem WASM."*

| Domain Logicodex | WIT Target | Hardware? |
|---|---|---|
| `Storage` | `wasi:filesystem` | Tidak |
| `Net` | `wasi:sockets` | Tidak |
| `UI` | `wasi:cli` | Tidak |
| `HW` | `logicodex:host-reactor` | **Hanya melalui Host Reactor** |
| `Audio` | `wasi:io/custom` | Tidak |
| `Crypto` | `wasi:crypto` | Tidak |

> **"WASM Guest = Unit Logik — NO direct hardware access."**

Setiap akses hardware dari guest WASM **mesti** melalui Host Reactor. Guest tidak boleh terus membaca atau menulis GPIO, DMA, atau Timer. Ini memastikan sandbox WASM kekal selamat walaupun kod guest dihasilkan oleh LLM yang tidak boleh dipercayai.

### Refleksi Perbincangan: Capability-Native WASM

> *"Kebanyakan bahasa compile ke WASM dan harap WASI cukup. Kenapa Logicodex perlu model capability sendiri?"*

Jawapan: **kerana WASI tidak mempunyai konsep capability yang cukup fine-grained.** WASI menyediakan "filesystem" atau "sockets" sebagai satu blok — anda boleh akses semua fail atau tiada fail. Logicodex menyediakan keupayaan per-fail, per-socket, per-hardware-pin.

Dengan memetakan model capability Logicodex ke WASI (melalui CTL Mapper), kita mendapat kedua-dua: fine-grained capability dari Logicodex dan portabilitas/portabilitas WASM dari WASI.

---

## Fasa 7: Raylib FFI + Audio v1.42-v1.43 — Grafik & Audio {#fasa7}

**Milestone:** 54 fungsi Raylib (28 grafik + 4 matematik + 22 audio), StrictAudioContext  
**Validator:** 89/89 checks passing (v1.42: 9, v1.43: 80)

### Apa yang Dibina

Fasa 7 membolehkan Logicodex membuat aplikasi grafik dan audio melalui binding FFI ke Raylib — tetapi dengan lapisan keselamatan capability yang unik.

### Raylib FFI: 54 Fungsi

| Kategori | Fungsi | Status |
|---|---|---|
| **Grafik (28)** | InitWindow, CloseWindow, BeginDrawing, EndDrawing, ClearBackground, DrawText, DrawRectangle, DrawCircle, DrawLine, DrawRectangleLines, DrawPixel, DrawTriangle, DrawPoly, LoadTexture, UnloadTexture, DrawTexture, DrawTextureEx, DrawTextureRec, DrawTexturePro, GetMousePosition, IsMouseButtonPressed, IsMouseButtonDown, GetMouseWheelMove, IsKeyPressed, IsKeyDown, SetTargetFPS, GetScreenWidth, GetScreenHeight | ✅ Safe wrappers |
| **Matematik (4)** | Clamp, Lerp, Normalize, Remap | ✅ Math shims |
| **Audio (22)** | InitAudioDevice, CloseAudioDevice, IsAudioDeviceReady, SetMasterVolume, LoadSound, UnloadSound, PlaySound, StopSound, IsSoundPlaying, LoadMusicStream, UnloadMusicStream, PlayMusicStream, StopMusicStream, IsMusicStreamPlaying, UpdateMusicStream, SetMusicVolume, SeekMusicStream, LoadAudioStream, UnloadAudioStream, PlayAudioStream, StopAudioStream, IsAudioStreamPlaying | ✅ StrictAudioContext |

### StrictAudioContext Integration

Audio callback (digunakan oleh `SetAudioStreamCallback`) melalui 4 lapisan semakan:

| Semakan | Apa Dihalang | Mengapa |
|---|---|---|
| `AudioViolationIo` | Print/DrawText/InitWindow dalam ISR | ISR audio mesti real-time; I/O blocking melanggar itu |
| `AudioViolationRecursion` | Panggilan rekursif sendiri | Stack overflow dalam ISR = kernel panic |
| `AudioViolationUnboundedLoop` | `loop { }` tanpa henti | ISR mesti selesai dalam masa yang ditentukan |
| `AudioViolationForbiddenCall` | malloc/free/spawn | ISR tidak boleh mengalokasi memori atau membuat thread |

### Refleksi Perbincangan: Audio Safety

> *"Audio callback dalam bahasa sistem selalu berbahaya. Bagaimana Logicodex menanganinya?"*

Jawapan: melalui **StrictAudioContext** — sebuah kontrak statik yang mengesan 4 jenis pelanggaran dalam fungsi audio callback. Ini bukan runtime check (yang terlalu lambat untuk ISR); ini adalah analisis semantik masa kompil yang menolak program yang melanggar kontrak.

Analogi: Ia seperti sebuah kontrak antara programmer dan compiler — "Saya janji fungsi ini tidak melakukan I/O blocking" — dan compiler memastikan janji itu dipatuhi sebelum menghasilkan kod.

---

## Fasa 8: Freestanding Compiler v1.44 — Bare Metal {#fasa8}

**Milestone:** Semua 15 jurang freestanding diselesaikan — Logicodex kini compiler freestanding  
**Validator:** 15/15 checks passing

### 15 Jurang Freestanding (G1-G15)

| Tier | Gap | Fail | Apa |
|---|---|---|---|
| **MUST** | G1 | `src/os/startup.rs` | `_start` entry: set stack (2MB), zero BSS, copy data, call main, halt |
| **MUST** | G2 | `src/os/panic.rs` | `#[panic_handler]`: clear SSE registers, UART output, halt loop |
| **MUST** | G3 | `lib/linker_scripts/` | Memory layout: code at 1MB, stack 1-2MB, heap after BSS |
| **MUST** | G4 | `src/os/allocator.rs` | Bump allocator: AtomicUsize CAS, `#[global_allocator]`, OOM null |
| **MUST** | G5 | `src/os/uart.rs` | x86_64 port I/O: `uart_putc/puts/hex`, VGA text mode (0xB8000) |
| **HIGH** | G6 | `src/lib.rs` | `#![no_std]` + `extern crate alloc` + conditional re-exports |
| **HIGH** | G7 | `src/os/source_provider.rs` | `SourceProvider` trait: filesystem, embedded, binary providers |
| **HIGH** | G8 | `src/os/target.rs` | `TargetArch` enum (x86_64/aarch64/riscv64) |
| **HIGH** | G9 | `src/os/target.rs` | Fixed `+soft-float` → `+sse2` untuk x86_64 |
| **HIGH** | G10 | `src/os/startup.rs` | BSS zeroing + data copy dalam `_start` |
| **MED** | G11 | `src/os/interrupts.rs` | IDT (256 entries), 32 exception handlers, PIC remap |
| **MED** | G12 | `src/codegen.rs` | `emit_hardware_zone()` + `emit_mmio_volatile_write/read()` |
| **MED** | G13 | `lib/startup/multiboot_header.rs` | Multiboot header (0x1BADB002), GRUB-compatible |
| **MED** | G14 | `src/os/startup.rs` | Stack pointer init: `mov rsp, 0x200000` |
| **MED** | G15 | `build.rs` | Raylib detection (pkg-config, RAYLIB_DIR, platform paths) |

### 3 Sptektitektur

| Arkitektur | LLVM Triple | Ciri-ciri |
|---|---|---|
| **x86_64** (default) | `x86_64-unknown-none` | `+sse2`, kernel code model |
| **aarch64** | `aarch64-unknown-none` | (default), small code model |
| **riscv64** | `riscv64gc-unknown-none-elf` | (default), medium code model |

### Refleksi Perbincangan: Freestanding = Kebebasan Mutlak

> *"Freestanding bermaksud tiada OS. Tiada OS bermaksud tiada printf, tiada malloc, tiada fail system. Kita mesti bina semuanya dari sifar."*

Keputusan untuk menyokong freestanding bukan sekadar menambah `--target freestanding` flag. Ia bermaksud:
1. Tiada `std` Rust — hanya `core` dan `alloc`
2. Tiada sistem operasi — kita mesti tulis `_start` sendiri
3. Tiada `printf` — kita mesti tulis UART driver sendiri
4. Tiada `malloc` — kita mesti tulis allocator sendiri (bump allocator)
5. Tiada exception handling OS — kita mesti tulis IDT sendiri

Ini adalah 15 jurang yang mesti diselesaikan sebelum Logicodex boleh dianggap "freestanding compiler." Setiap jurang mewakili satu fungsi OS yang biasa diambil untuk granted.

---

## Fasa 9: Stabilisasi v1.44.1-v1.45 — Maintenance & Benchmark {#fasa9}

### v1.44.1: Foundation Polish

| Tugas | Apa |
|---|---|
| **Validator Tiering** | 3 tier: A (core/7), B (feature/13), C (platform/8) |
| **Dead Code Audit** | 1 TODO (by design), 0 `#[allow(unused)]`, 0 `todo!()` |
| **Test Coverage Analysis** | 69% test-to-source ratio (34/49 files) |
| **Security Micro-Audit** | 0 unwrap() dalam production, 141 unsafe blocks (dokumentasi penuh), 134 `as` casts (selamat) |
| **Documentation Drift Fix** | README, ARCHITECTURE, docs selaras |

### v1.45: Quantitative Benchmark Framework

| Layer | Apa | Fail |
|---|---|---|
| **Layer 1 Micro** | 6 benchmark Criterion | `gate_latency`, `door_latency`, `mempool_latency`, `callable_lookup`, `hir_lower`, `llvm_emit` |
| **Layer 2 Reactor** | Throughput echo server + flood client | `echo_server.rs`, `flood_client.rs`, `throughput.sh` |
| **Layer 3 Stability** | RSS monitor, valgrind, longrun | `rss_monitor.py`, `valgrind_check.sh`, `longrun.sh` |
| **Layer 4 Security** | Stress test stubs | `slowloris.py`, `syn_flood.py`, `malformed.py`, `fd_exhaustion.py` |

| Infrastruktur | Apa |
|---|---|
| `BASELINE.json` | Gold standard dengan regression thresholds (5% warn, 10% fail) |
| `run_all.sh` | All-layers runner (quick/full/compare modes) |
| `compare_baseline.py` | Regression detection tool |
| `RFC_TEMPLATE.md` | Architecture Freeze enforcement |

### Refleksi Perbincangan: Architecture Freeze

> *"Kita kena jaga integrity architecture dengan freeze dulu unless explicitly minta unfreeze. Tapi boleh buat minor adjustment untuk improve existing functions."*

Ini adalah keputusan penting dalam sejarah Logicodex. Selepas 14 releases dalam masa beberapa bulan, arkitektur telah mencapai tahap kematangan di mana:
1. Semua deferred items diselesaikan
2. Semua pending items diselesaikan
3. Semua validator lulus
4. Tiada regression

Architecture Freeze bermaksud:
- **Tidak ada fitur baru** tanpa melalui RFC process
- **RFC mesti lulus 4 mandatory checks**: Static Topology, Explicit Ownership, Shard Isolation, Deterministic Behavior
- **Minor adjustments dibenarkan** untuk memperbaiki fungsi sedia ada (contoh: validator tiering, benchmark framework)
- **Unfreeze memerlukan justifikasi eksplisit** dari arkitek

Ini bukan "projek mati" — ini adalah **projek matang**. Seperti kernel Linux yang mempunyai freeze window sebelum release, Logicodex mempunyai freeze untuk mengekalkan integriti arkitektur sambil membolehkan penambahbaikan berterusan.

---

## Ringkasan Statistik Evolusi

| Metrik | Nilai |
|---|---|
| **Jumlah LOC** | ~43,600 |
| **Validator checks** | 148/148 ✅ |
| **Deferred items** | 25/25 diselesaikan (1 by design) |
| **Releases** | v1.21 → v1.45 (14 releases) |
| **Backend targets** | Native (ELF), WASM (wasm32-unknown-unknown), Freestanding (x86_64/aarch64/riscv64) |
| **Unit tests** | 400+ |
| **Benchmark files** | 20 |
| **Architecture supports** | 3 (x86_64, aarch64, riscv64) |

### Garis Masa Visual

```
v1.21        v1.30         v1.32         v1.33-v1.37   v1.34-v1.39   v1.40-v1.41   v1.42-v1.43   v1.44         v1.44.1-v1.45
  │            │             │             │             │             │             │             │             │
  ▼            ▼             ▼             ▼             ▼             ▼             ▼             ▼             ▼
Compiler    Threading     Capability    Network       Sharded       WASM +        Raylib        Freestanding  Stabilisasi
Core        + IO + Audio  Security      Reactor       Runtime       Host          FFI +                       + Benchmark
                                                      (Multi-Core)  Reactor       Audio
```

Setiap fasa membina di atas fasa sebelumnya dengan **zero regression** — tiap-tiap release mengekalkan semua validator checks releases terdahulu. Ini adalah bukti bahawa asas yang dibina pada v1.21 adalah kukuh.
