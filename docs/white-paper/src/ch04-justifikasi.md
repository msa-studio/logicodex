# Chapter 4: Refleksi — Justifikasi Keputusan Arkitektur Utama

> *"Setiap keputusan dalam rekabentuk Logicodex adalah hasil perbincangan, cabaran, dan penapisan. Tiada keputusan dibuat secara sewenang-wenangnya."*

Bab ini merakam justifikasi mendalam untuk 9 keputusan arkitektur utama yang menentukan bentuk Logicodex hari ini. Setiap bahagian merangkumi: (a) persoalan asal yang timbul dalam perbincangan, (b) alternatif yang dipertimbangkan, (c) keputusan akhir, dan (d) implikasinya.

---

## 4.1 Kenapa Actor Model + Channel, Bukan Thread Biasa? {#actor}

### Persoalan Asal

> *"Kenapa tidak gunakan thread biasa dengan mutex? Semua bahasa buat begitu. Rust pun ada Mutex<T>."*

### Alternatif Dipertimbangkan

| Alternatif | Kekuatan | Kelemahan |
|---|---|---|
| **Thread + Mutex** (C/C++/Rust) | Mudah difahami, ekosistem besar | Non-deterministik, deadlock, priority inversion |
| **Thread + Lock-free structures** | Prestasi tinggi | Kompleksiti songang, ABA problem, memory ordering bugs |
| **Green threads / M:N** (Go) | Mudah digunakan | Latency tidak boleh diramal, konteks switch mahal |
| **Async/await** (Rust, JS) | Non-blocking | "Colored functions", state machine complexity, pin requirements |

### Keputusan: Actor Model + SPSC Channel

Setelah perbincangan mendalam, kita memilih **actor model** dengan **SPSC ring buffer**:

```text
actor Worker {
    let ch: Channel<Message>
    ch.send(data)    ──► SPSC Ring Buffer ──► ch.recv() on consumer
}         ▲                                    │
          │                                    ▼
     spawn Worker()                       actor Consumer {
                                              process(data)
                                          }
```

### Justifikasi

**1. Isolasi adalah Asas Determinisme**

Dalam model actor, setiap actor adalah unit terpencil (isolated). Tidak ada state dikongsi — semua komunikasi melalui message passing. Ini bermaksud:
- Tidak ada race condition (tiada state dikongsi untuk di"race")
- Tidak ada deadlock (tiada lock untuk di"deadlock")
- Tidak ada priority inversion (tiada lock untuk di"invert")

**2. SPSC Ring Buffer = Zero-Copy + Lock-Free**

Ring buffer kita menggunakan atomic operations (Release/Acquire memory ordering) tanpa mutex. Ownership data dipindahkan daripada producer ke consumer tanpa menyalin data:

```
Producer                    Consumer
   │    ring_send(ptr)        │
   │ ──────────────────────►  │
   │  (ownership transferred) │
   │                          │ ring_recv() → ptr
   │    ▲                     │   (now owns ptr)
   │    │                     │
   └────┘ (cannot use ptr    └──────► process(ptr)
            anymore — UseAfterSend
            would be caught at compile time)
```

**3. Model yang Sama dari Embedded ke Server**

Satu kelebihan model actor ialah ia skala dari sistem embedded (1 core, 2 actors) sehinggalah server (64 cores, 1000 actors). Tiada perubahan paradigma — hanya peningkatan kuantiti.

### Refleksi: Adakah Ini Keputusan yang Betul?

**Ya.** Selepas 400+ tests dan 20 benchmark, model actor telah terbukti:
- **Deterministik**: 100% — tiap-tiap test menghasilkan output yang sama
- **Pantas**: Door latency < 100ns (benchmark Layer 1)
- **Selamat**: Zero race condition dalam sejarah projek
- **Boleh skala**: Sharded reactor berjalan pada 8 core dengan >85% efficiency

Trade-off utama: message passing kurang efisien daripada shared memory untuk data besar. Kita atasi ini melalui **zero-copy ownership transfer** — data tidak disalin, hanya ownership dipindahkan.

---

## 4.2 Kenapa Zero-Copy Ownership Transfer? {#zerocopy}

### Persoalan Asal

> *"Message passing biasanya menyalin data. Bagaimana dengan payload besar (gambar 4K, audio buffer)?"*

### Alternatif Dipertimbangkan

| Alternatif | Kos | Keselamatan |
|---|---|---|
| **Salinan penuh** (Go, Erlang) | O(n) per message | Keselamatan memori terjamin |
| **Shared pointer + RC** (C++, Rust Arc) | O(1) tetapi reference counting | RC overhead, siklik reference |
| **Zero-copy ownership** (Logicodex) | O(1), tiada overhead | Compile-time ownership tracking |

### Keputusan: Zero-Copy Ownership Transfer

Data tidak disalin — ownership dipindahkan daripada sender ke receiver. Sebaik sahaja `ring_send()` dipanggil, sender **tidak lagi boleh** mengakses data tersebut. Compiler menolak `UseAfterSend` pada masa kompil.

```logicodex
let data: Buffer<U8> = Buffer::new(1024);
ch.send(data);           // ✅ ownership moved to channel
print data[0];           // ❌ COMPILE ERROR: UseAfterSend
```

### Justifikasi

**1. Prestasi Tanpa Kos**

Untuk payload 4MB (sebuah frame gambar), zero-copy bermaksud kita tidak menyalin 4MB. Ini adalah perbezaan antara 60 FPS dan 30 FPS dalam aplikasi grafik.

**2. Keselamatan Tanpa Runtime Cost**

Tiada reference counting, tiada garbage collection, tiada runtime check. Semua semakan ownership berlaku pada masa kompil. Selepas kompilasi, kod yang tinggal adalah ring buffer operation sederhana.

**3. Model yang Konsisten dengan Rust Ownership**

Logicodex mengadopsi konsep ownership Rust tetapi menyederhanakannya — tidak ada lifetime parameters, tidak borrow checker yang kompleks. Hanya satu peraturan: **selepas hantar, tidak boleh guna.**

### Refleksi

Zero-copy ownership adalah keputusan yang menentukan prestasi keseluruhan sistem. Tanpa ini, sharded reactor akan terbeban dengan salinan data antara core — menjadikan scaling efficiency rendah. Dengan zero-copy, kita mencapai >85% scaling efficiency pada 8 core.

---

## 4.3 Kenapa Compile-Time Capability, Bukan Runtime Check? {#compiletime}

### Persoalan Asal

> *"Adakah lebih selamat untuk semak kebenaran pada runtime? Kalau salah, kita boleh detect dan hentikan."*

### Analogi: Jambatan

Bayangkan dua cara membina jambatan:

| Pendekatan | Apa Berlaku | Keputusan |
|---|---|---|
| **Compile-time** (Logicodex) | Jurutera pastikan jambatan kukuh sebelum dibuka | ✅ Jambatan selamat |
| **Runtime check** (Lain) | Pasang sensor "kalau runuh, henti trafik" | ❌ Orang mungkin mati sebelum sensor berfungsi |

### Keputusan: Capability Fabric dengan Zero Runtime Cost

Semua gate hanya wujud dalam intermediate representation:

```text
Source (.ldx)          IR (CapabilityGraph)          Output (Native)
     │                          │                              │
     ▼                          ▼                              ▼
service WebServer {     IRServiceNode {                    Binary native
  port: 8080,               port: 8080,                     Tiada trace
  requires: Net.Admin, ──►   gates: [IRGateEdge {           capability
  handler: WebHandler,          domain: Net,                 (inlined)
  policy: Block                operation: Admin,
  }                            }]
                          }
```

### Justifikasi

**1. Kos Runtime Sifar**

Setelah topology verify lulus, gate tidak perlu lagi diperiksa. Compiler menghasilkan kod yang menganggap semua permission telah diverifikasi. Ini bermaksud:
- Tiada "permission check" pada setiap system call
- Tiada overhead capability pada hot path
- Prestasi setara dengan C/C++ (tiada runtime framework)

**2. Fail-Safe by Default**

Jika topology verify gagal, **program tidak dikompil.** Ini adalah "fail-safe" — sistem yang selamat gagal dalam keadaan selamat (program tidak berjalan), bukan "fail-dangerous" (program berjalan tanpa kebenaran).

**3. Audit Trail**

Fail `.cap` yang dihasilkan pada setiap kompilasi menyediakan audit trail untuk supply-chain security:

```cap
# topology.cap — generated by logicodex v1.32
[service WebServer]
port=8080
requires=Net.Admin
handler=WebHandler
policy=Block

[gate Net.Admin]
domain=Net
operation=Admin
verified=true
timestamp=2026-05-25T12:00:00Z
```

### Refleksi

Compile-time capability adalah prinsip yang paling sering dicabar. Banyak pembangun berpendapat bahawa "flexibility" runtime lebih praktikal. Tetapi dalam konteks Logicodex — sistem programming untuk aplikasi kritikal — compile-time verification adalah satu-satunya pendekatan yang dapat diterima.

---

## 4.4 Kenapa Taint FSM untuk Network? {#taint}

### Persoalan Asal

> *"Kenapa perlu Taint FSM? Kenapa tidak tutup connection terus apabila error?"*

### Masalah: Attack Surface Rangkaian

Rangkaian adalah permukaan serangan terbesar untuk mana-mana aplikasi server. Jenis serangan biasa:

| Serangan | Apa Yang Berlaku |
|---|---|
| **Slowloris** | Hantar request separuh, tunggu timeout, ulang — habiskan connection slots |
| **SYN Flood** | Hantar SYN tanpa ACK — habiskan half-open connections |
| **Malformed packet** | Hantar data tidak sah — trigger bug parser |
| **Resource exhaustion** | Buka banyak connections — habiskan file descriptors |

### Keputusan: 3-State Taint FSM

Setiap connection mempunyai keadaan (state) yang ditentukan secara deterministik:

```text
          error_count++ / timeout
Healthy ──────────────────────────► Suspicious
   ▲                                     │
   │          error_count = 0           │ error_count > threshold
   └────────────────────────────────────┘
                                        ▼
                                    Closing
                                        │
                                        ▼
                                  close(fd)
                                  RAII cleanup
```

| Keadaan | Makna | Tindakan |
|---|---|---|
| **Healthy** | Connection normal | Proses I/O seperti biasa |
| **Suspicious** | Error count meningkat, tetapi belum melepasi threshold | Terus proses dengan waspada, reset counter jika OK |
| **Closing** | Error count melepasi threshold | Tutup connection, cleanup resources |

### Justifikasi

**1. Graceful Degradation, Bukan Panic**

Taint FSM membolehkan aplikasi meneruskan operasi walaupun dibawah serangan. Connection jahat ditutup, tetapi aplikasi tidak runtuh.

**2. Deterministik**

Transisi state bergantung pada **error count** dan **timeout** — kedua-duanya adalah metrik yang boleh diukur secara objektif. Tiada keputusan "fuzzy" atau "heuristik".

**3. Integrasi dengan Backpressure**

Taint FSM bekerja bersama backpressure policy:
- `Block`: Tunggu sehingga ruang dalam ring buffer
- `DropOldest`: Buang data lama, terima data baru
- `Error`: Pulangkan error kepada caller

Kombinasi ini membolehkan aplikasi bertindak balas kepada serangan dengan cara yang ditentukan secara eksplisit — bukan dengan crash atau hang.

### Refleksi

Taint FSM adalah contoh bagaimana Logicodex menangani kompleksiti dunia nyata (serangan rangkaian) dengan mekanisme yang deterministik. Ia bukan "AI-powered intrusion detection" — ia adalah state machine sederhana yang tidak boleh diperdaya.

---

## 4.5 Kenapa CapabilityGraph IR sebagai "Single Source of Truth"? {#ir}

### Persoalan Asal

> *"Kita ada SemanticSummary (v1.31), CapabilityTopology (v1.32), dan ShardTopology (v1.34). Tiga struktur berasingan — ini fragile."*

### Masalah: Fragmentasi IR

Sebelum v1.35, tiga struktur berasingan mengekalkan maklumat yang bertindih:

| Struktur (Lama) | Apa | Fail |
|---|---|---|
| `SemanticSummary` (v1.31) | Effects, inline cost per service | `src/tier2/streaming.rs` |
| `CapabilityTopology` (v1.32) | Gate edges, capability references | `src/tier2/topology.rs` |
| `ShardTopology` (v1.34) | Shard assignments, cross-shard doors | `src/tier2/shard.rs` |

Masalah: perubahan pada satu struktur mungkin tidak diselaraskan dengan struktur lain. Ini mencipta "drift" yang boleh menyebabkan inkonsistensi.

### Keputusan: CapabilityGraph IR — Unified IR

v1.35 memperkenalkan **CapabilityGraph IR** yang menyatukan ketiga-tiga struktur ke dalam satu "Single Source of Truth":

```text
CapabilityGraph IR (v1.35)
│
├── IRServiceNode        (dari SemanticSummary v1.31)
│   ├── effects
│   ├── inline_cost
│   └── handler_ref
│
├── IRGateEdge           (dari CapabilityTopology v1.32)
│   ├── domain
│   ├── operation
│   └── capability_ref
│
├── IRShardNode          (dari ShardTopology v1.34)
│   ├── core_id
│   ├── memory_budget
│   └── assigned_services
│
└── IRDoorEdge           (baru — cross-shard channels)
    ├── from_shard
    ├── to_shard
    └── channel_type
```

### Output dari Satu IR

Dari satu CapabilityGraph, kita menjana pelbagai output:

| Output | Apa | Target |
|---|---|---|
| **Native ELF** | Inlined capability checks | Linux, macOS, embedded |
| **`.cap` file** | Audit trail | Supply-chain security |
| **WIT stub** | WASM interface types | WebAssembly |

### Justifikasi

**1. Konsistensi Dijamin**

Dengan satu IR, ketiga-tiga output (Native, `.cap`, WIT) sentiasa konsisten. Tidak mungkin Native ELF membenarkan gate yang WIT tidak benarkan — kerana kedua-duanya datang daripada sumber yang sama.

**2. Verifikasi Pusat**

6 semakan `verify()` dijalankan pada satu IR:

| Semakan | Apa Dihalang |
|---|---|
| `EmptyGraph` | Graph tanpa node |
| `WasmHardwareGate` | Gate hardware dalam target WASM |
| `InvalidShardAssignment` | Service diumpukkan ke shard tidak wujud |
| `UnknownServiceInDoor` | Door merujuk service tidak wujud |
| `UnknownServiceInGate` | Gate merujuk service tidak wujud |
| `EmptyShard` | Shard tanpa service |

**3. Extensibility**

IR baru boleh ditambah tanpa mengubah struktur sedia ada. Contoh: jika kita mahu menambah `IRGpuNode` untuk GPU compute, kita hanya menambah node baharu ke graph — tanpa mengubah service, gate, atau shard nodes.

### Refleksi

CapabilityGraph IR adalah contoh bagaimana perbincangan "kenapa kita ada 3 struktur berasingan?" membawa kepada rekabentuk yang lebih bersih. Ini adalah pattern "refactor to unify" yang berulang dalam sejarah pembangunan Logicodex — apabila pelbagai struktur mula "drift", kita satukan.

---

## 4.6 Kenapa CTL Mapper "Project INTO, Not Borrow FROM"? {#ctl}

### Persoalan Asal

> *"Kebanyakan bahasa compile ke WASM dan adaptasi diri kepada WASI. Kenapa Logicodex perlu model sendiri?"*

### Masalah: WASI Tidak Cukup Fine-Grained

WASI (WebAssembly System Interface) menyediakan interface standard untuk akses filesystem, rangkaian, dan resources lain. Tetapi model permission WASI adalah coarse-grained:

| WASI Permission | Apa Dibenarkan | Masalah |
|---|---|---|
| `wasi:filesystem` | Akses semua fail | Tidak boleh hadkan kepada satu direktori |
| `wasi:sockets` | Akses semua socket | Tidak boleh hadkan kepada satu port |
| `wasi:cli` | Akses semua environment | Tidak boleh hadkan kepada satu variable |

### Keputusan: CTL Mapper — Project Capability Logicodex INTO WASM

Logicodex tidak "meminjam" model WASI. Sebaliknya, CTL Mapper **memetakan model capability-native Logicodex ke dalam ekosistem WASM**:

```text
┌─────────────────────────────────────────────────┐
│              LOGICODEX CAPABILITY MODEL          │
│                                                  │
│  service WebServer {                             │
│      requires: Net.Admin,  ◄── fine-grained     │
│                 Storage.Read("/data"), ◄── path │
│                 HW.GPIO.Pin(13) ◄── pin number  │
│  }                                               │
└──────────────────┬───────────────────────────────┘
                   │ CTL Mapper (v1.36)
                   ▼
┌─────────────────────────────────────────────────┐
│              WASM ECOSYSTEM                      │
│                                                  │
│  wasi:sockets (Net.Admin)                       │
│  wasi:filesystem (Storage.Read — scoped)        │
│  logicodex:host-reactor (HW.GPIO.Pin(13))       │
│                                                  │
└─────────────────────────────────────────────────┘
```

### Pemetaan Domain

| Domain Logicodex | WIT Target | Hardware? |
|---|---|---|
| `Storage` | `wasi:filesystem` | Tidak |
| `Net` | `wasi:sockets` | Tidak |
| `UI` | `wasi:cli` | Tidak |
| `HW` | `logicodex:host-reactor` | **Hanya melalui Host Reactor** |
| `Audio` | `wasi:io/custom` | Tidak |
| `Crypto` | `wasi:crypto` | Tidak |

### Justifikasi

**1. Fine-Grained Control**

Dengan CTL Mapper, kita boleh hadkan akses kepada satu fail, satu port, satu pin GPIO — bukan "semua fail" atau "semua rangkaian."

**2. Hardware Isolation**

Gate hardware `HW.*` **tidak pernah** sampai ke guest WASM. Ia sentiasa dihantar ke Host Reactor, yang memutuskan sama ada membenarkan akses berdasarkan:
- GatePermissions (per-operation allowlists)
- HardwareZone (pin claim/release tracking)
- HostFunction dispatch (protokol GuestRequest/HostResponse)

**3. Extensibility**

CTL Mapper menyokong manual overrides — jika pemetaan automatik tidak mencukupi, pembangun boleh menentukan pemetaan tersuai:

```rust
ctl_mapper.add_override("HW.Timer", "myvendor:custom-timer");
```

### Refleksi

"Project INTO, not borrow FROM" adalah falsafah yang melampaui WASM. Ia bermaksud Logicodex tidak akan menyesuaikan diri kepada model asing yang kurang expressive. Sebaliknya, kita memetakan model kita yang lebih expressive ke dalam model asing, dengan kehilangan maklumat minimum.

---

## 4.7 Kenapa 3 Backend Target Serentak? {#backend}

### Persoalan Asal

> *"Mengapa kompilasi ke Native, WASM, DAN Freestanding? Bukan ke satu target sudah cukup kerja?"*

### Analisis Keperluan

| Target | Keperluan | Use Case |
|---|---|---|
| **Native (ELF)** | Prestasi maksimum, akses OS | Server, desktop, HPC |
| **WASM** | Portabilitas, sandboxed, browser | Web, plugin, serverless |
| **Freestanding** | Tiada OS, kawalan penuh hardware | Firmware, kernel, embedded |

### Keputusan: Tiga Target daripada Satu Sumber

Logicodex menyokong ketiga-tiga target daripada kod sumber `.ldx` yang sama — melalui flag `--target`:

```bash
logicodex input.ldx -o output                # Native (default)
logicodex --target wasm input.ldx -o output  # WebAssembly
logicodex --target freestanding input.ldx    # Bare metal
```

### Implementasi

| Target | LLVM Triple | Ciri-ciri |
|---|---|---|
| Native | `x86_64-unknown-linux-gnu` | ELF, linked dengan ld |
| WASM | `wasm32-unknown-unknown` | `+bulk-memory,+mutable-globals,+sign-ext` |
| Freestanding x86_64 | `x86_64-unknown-none` | `+sse2`, kernel code model |
| Freestanding aarch64 | `aarch64-unknown-none` | small code model |
| Freestanding riscv64 | `riscv64gc-unknown-none-elf` | medium code model |

### Justifikasi

**1. "Write Once, Run Anywhere" yang Sebenar**

Java menjanjikan "write once, run anywhere" tetapi melalui JVM interpreter. Logicodex menepati janji ini melalui kompilasi natif — kod `.ldx` dikompil ke binari natif untuk setiap target, bukan diinterpret.

**2. Use Case Lengkap**

Satu bahasa yang boleh digunakan untuk:
- Web application (WASM target)
- Server backend (Native target)
- IoT firmware (Freestanding target)
- Tanpa perubahan kod sumber (hanya flag berbeza)

**3. Capability Model Konsisten**

Gate `Net.Admin` bermaksud perkara yang sama pada semua target — implementasi backend berbeza, tetapi semantik capability sama:
- Native: inlined gate check + direct syscall
- WASM: capability → WASI mapping + Host Reactor untuk HW
- Freestanding: inlined gate check + MMIO volatile operations

### Refleksi

Menyokong 3 target adalah kerja besar — setiap target memerlukan backend code generation, validator, dan documentation sendiri. Tetapi ini adalah keputusan strategik: Logicodex bukan sekadar "bahasa untuk Linux" atau "bahasa untuk web" — ia adalah **platform sistem universal**.

---

## 4.8 Kenapa StrictAudioContext untuk Audio? {#audio}

### Persoalan Asal

> *"Audio callback dalam bahasa sistem selalu berbahaya. ISR audio mesti selesai dalam mikrodetik. Bagaimana Logicodex menanganinya?"*

### Masalah: Audio ISR Berbahaya

Audio callback (ISR — Interrupt Service Routine) berjalan pada frekuensi tinggi (biasanya 44.1kHz atau 48kHz). Ini bermaksud callback mesti selesai dalam ~20μs. Dalam masa ini, callback **tidak boleh**:
- Membuat alokasi memori (malloc boleh mengambil beratus-ratus μs)
- Memanggil fungsi blocking (I/O akan mengambil berpuluh-puluh ms)
- Berulang tak terbatas (ISR tidak pernah selesai)
- Memanggil dirinya sendiri (stack overflow)

### Keputusan: StrictAudioContext — 4 Violation Types

Logicodex menganalisis fungsi audio callback pada masa kompil dan menolak program yang melanggar mana-mana daripada 4 peraturan:

| Pelanggaran | Apa Dihalang | Mengapa |
|---|---|---|
| `AudioViolationIo` | `print`, `DrawText`, `InitWindow` dalam callback | I/O blocking melangkar timing real-time |
| `AudioViolationRecursion` | Fungsi memanggil dirinya sendiri | Stack overflow dalam ISR = system crash |
| `AudioViolationUnboundedLoop` | `loop { }` tanpa termination | ISR tidak pernah selesai — audio dropout |
| `AudioViolationForbiddenCall` | `malloc`, `free`, `spawn` | Alokasi memori dan thread creation terlalu lambat |

### Integrasi dengan Capability System

Audio functions dipetakan ke gate `Audio.Main`:

```text
SetAudioStreamCallback(stream, my_callback)
         │
         ▼
Analyzer::register_audio_callback("my_callback")
         │
         ▼
verify_audio_safety("my_callback")
         │
         ├── Check: Tiada I/O calls? ✅
         ├── Check: Tiada recursion? ✅
         ├── Check: Tiada unbounded loops? ✅
         ├── Check: Tiada forbidden calls? ✅
         │
         └── ALL PASSED → callback approved
```

### Justifikasi

**1. Safety Tanpa Runtime Cost**

StrictAudioContext adalah analisis statik — tiada runtime check dalam callback. Setelah diluluskan pada masa kompil, callback berjalan dengan overhead sifar.

**2. Tidak Perlu "Audio Expert" untuk Menulis Kod Selamat**

Pembangun biasa tidak perlu tahu tentang mikrodetik timing atau ISR constraints. Compiler akan menolak kod yang tidak selamat dan memberikan diagnostic yang jelas.

**3. Konsisten dengan Prinsip Capability**

Audio adalah sumber berbahaya (real-time constraints) — oleh itu, ia memerlukan gate sendiri (`Audio.Main`). Ini bukan pengecualian daripada model capability; ini adalah aplikasinya.

### Refleksi

StrictAudioContext adalah contoh bagaimana Logicodex menangani domain khusus (audio real-time) dengan model umum (capability + compile-time verification). Pattern ini boleh diulang untuk domain lain: video encoding, robotik control, signal processing.

---

## 4.9 Kenapa Architecture Freeze + RFC Process? {#freeze}

### Persoalan Asal

> *"Kita kena jaga integrity architecture dengan freeze dulu unless explicitly minta unfreeze. Tapi boleh buat minor adjustment untuk improve existing functions."*

### Masalah: Feature Creep Mengancam Integriti

Selepas 14 releases dalam masa beberapa bulan, Logicodex telah bertambah kompleksiti secara signifikan. Tanpa disiplin, "tambahan kecil" akan terus menambah teknologi debt:

| Risiko Tanpa Freeze | Contoh |
|---|---|
| Feature creep | "Tambah fitur X lah, kecil je" → akhirnya 50 "fitur kecil" |
| Architecture drift | Fitur baru tidak selaras dengan prinsip asal |
| Test fragmentation | Tiap-tiap fitur baru memerlukan test suite sendiri |
| Documentation rot | Dokumentasi tidak dapat mengekori perubahan |

### Keputusan: Architecture Freeze Policy

| Peraturan | Implementasi |
|---|---|
| **Tiada fitur baru** tanpa RFC | RFC mesti melalui 4 mandatory alignment checks |
| **Minor adjustment dibenarkan** | Perbaikan kepada fungsi sedia ada (validator, benchmark, polish) |
| **Unfreeze memerlukan justifikasi** | Arkitek mesti memberi sebab eksplisit |
| **RFC template standard** | `docs/RFC_TEMPLATE.md` dengan checklist lengkap |

### 4 Mandatory Alignment Checks (dari RFC Template)

```markdown
## RFC Template — Mandatory Alignment Checks

Sebelum submit RFC, pastikan keputusan anda selaras dengan:

- [ ] **Static Topology**: Adakah fitur ini mengekalkan model topology statik?
- [ ] **Explicit Ownership**: Adakah fitur ini mengekalkan model ownership eksplisit?
- [ ] **Shard Isolation**: Adakah fitur ini mengekalkan isolasi shard?
- [ ] **Deterministic Behavior**: Adakah fitur ini mengekalkan tingkah laku deterministik?

Sekurang-kurangnya 3 daripada 4 checks mesti lulus. RFC yang gagal 2+ checks
akan ditolak automatik tanpa review lanjut.
```

### Justifikasi

**1. Menjaga Integriti Arkitektur**

Freeze memastikan semua tambahan selaras dengan 5 prinsip asal. Tiada "hack" atau "shortcut" yang memudaratkan keseluruhan sistem.

**2. Membolehkan Penambahbaikan Berterusan**

Minor adjustments seperti validator tiering, benchmark framework, dan documentation updates dibenarkan — kerana mereka **memperbaiki** fungsi sedia ada tanpa menambah kompleksiti arkitektur.

**3. Menyediakan Proses untuk Perubahan**

RFC process menyediakan saluran untuk perubahan yang sah — jika seseorang mempunyai idea yang baik yang melanggar freeze, mereka boleh menulis RFC dan memohon unfreeze dengan justifikasi.

### Refleksi

Architecture Freeze adalah keputusan yang paling sukar dalam sejarah Logicodex — kerana ia memerlukan menolak tambahan yang "menarik" demi menjaga integriti yang "penting." Tetapi ini adalah tanda kematangan projek: sistem yang stabil memerlukan disiplin untuk kekal stabil.

---

## Ringkasan 9 Keputusan

| # | Keputusan | Kenapa | Trade-off |
|---|---|---|---|
| 1 | Actor + Channel | Determinisme, zero race | Message passing overhead (dineutralkan oleh zero-copy) |
| 2 | Zero-Copy Ownership | Prestasi O(1), keselamatan | Programmer mesti faham ownership |
| 3 | Compile-Time Capability | Zero runtime cost, fail-safe | Kurang "flexibility" runtime |
| 4 | Taint FSM | Graceful degradation, deterministik | Tiada AI-powered detection |
| 5 | CapabilityGraph IR | Konsistensi, verifikasi pusat | Kerja rekabentuk awal lebih banyak |
| 6 | CTL Mapper | Fine-grained WASM capability | Perlu maintain mapping rules |
| 7 | 3 Backend Target | Universal platform | 3x kerja backend |
| 8 | StrictAudioContext | Audio safety tanpa runtime cost | 4 violation rules untuk programmer ikut |
| 9 | Architecture Freeze | Integriti arkitektur | Tiada fitur baru tanpa RFC |

Setiap keputusan ini adalah hasil perbincangan panjang, pertimbangan alternatif, dan penilaian trade-off. Tiada keputusan dibuat secara sewenang-wenangnya — setiap satunya dipertahankan oleh prinsip dan validated oleh data.
