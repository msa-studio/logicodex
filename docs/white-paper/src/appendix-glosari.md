# Appendix A: Glosari Istilah

Istilah-istilah utama yang digunakan dalam white paper ini dan dokumentasi Logicodex.

---

## A

| Istilah | Definisi |
|---|---|
| **Actor** | Unit komputasi terpencil (isolated computation unit) yang berkomunikasi hanya melalui message passing. Tiada shared mutable state antara actors. |
| **Alias-to-Canonical** | Mekanisme lexer Logicodex yang memetakan pelbagai permukaan sintaks (Melayu, Inggeris, shorthand pakar) ke token kanonikal yang sama. |
| **Architecture Freeze** | Keadaan di mana tiada ciri arkitektur baharu boleh ditambah tanpa melalui RFC process dan mendapat kelulusan arkitek. |

## B

| Istilah | Definisi |
|---|---|
| **Backpressure** | Mekanisme mengendalikan aliran data apabila producer lebih pantas daripada consumer. Tiga policy: `Block`, `DropOldest`, `Error`. |
| **BASELINE.json** | Fail gold standard yang menyimpan nilai benchmark rujukan dengan regression thresholds (5% warn, 10% fail). |
| **Bump Allocator** | Alokator memori O(1) menggunakan pointer bump dengan AtomicUsize CAS. Tiada fragmentasi. |

## C

| Istilah | Definisi |
|---|---|
| **Capability** | Keupayaan untuk mengakses sumber tertentu (fail, rangkaian, hardware). Dideklarasikan secara eksplisit dan diverifikasi pada masa kompil. |
| **CapabilityGraph IR** | "Single Source of Truth" IR (v1.35) yang menyatukan SemanticSummary, CapabilityTopology, dan ShardTopology ke dalam satu struktur graph. |
| **CapabilityGate** | Komponen keselamatan yang menyatakan akses apa yang diperlukan oleh service atau actor. Dihalang pada masa kompil jika tidak diizinkan. |
| **Canonical** | Bentuk standard token yang digunakan oleh parser dan semantic analyzer — bebas daripada alias permukaan. |
| **CTL Mapper** | Capability Translation Layer (v1.36) yang memetakan model capability Logicodex ke WASI/WIT untuk target WASM. |

## D

| Istilah | Definisi |
|---|---|
| **Determinism** | Sifat program yang menghasilkan output sama untuk input sama, setiap kali, tanpa bergantung pada keadaan runtime. |
| **Door** | Cross-shard channel — transport data antara actor pada shard berbeza menggunakan SPSC ring buffer. |

## E

| Istilah | Definisi |
|---|---|
| **ELF** | Executable and Linkable Format — format binari untuk Linux dan sistem Unix-like. |
| **epoll** | Linux system call untuk I/O event notification yang efisien. Logicodex menggunakan epoll melalui direct syscall (tiada libc). |

## F

| Istilah | Definisi |
|---|---|
| **Freestanding** | Target kompilasi tanpa sistem operasi — kod berjalan secara langsung pada hardware (bare metal). |
| **FFI** | Foreign Function Interface — mekanisme memanggil fungsi dari bahasa lain (biasanya C). Logicodex menggunakan FFI untuk Raylib. |
| **FSM** | Finite State Machine — model computation dengan keadaan terhingga dan transisi antara keadaan. Digunakan untuk Taint FSM. |

## G

| Istilah | Definisi |
|---|---|
| **Gate** | Kontrak keselamatan masa kompil. Tiga jenis: `DirectCall`, `Message`, `Hardware`. |
| **GatePermissions** | Struktur yang mengekalkan senarai allowlist untuk setiap operasi hardware dalam Host Reactor. |

## H

| Istilah | Definisi |
|---|---|
| **HardwareZone** | Struktur pelacakan claim/release pin hardware yang menghalang double-use pin. |
| **HIR** | High-Level Intermediate Representation — perwakilan peringkat pertama selepas AST dalam pipeline compiler. |
| **Host Reactor** | Komponen (v1.41) yang memediasi akses hardware dari guest WASM ke host hardware melalui protokol GuestRequest/HostResponse. |

## I

| Istilah | Definisi |
|---|---|
| **IDT** | Interrupt Descriptor Table — tabel 256 entri untuk menangani interrupts dan exceptions pada x86_64 freestanding. |
| **IR** | Intermediate Representation — perwakilan perantara kod sumber antara parsing dan code generation. |
| **ISR** | Interrupt Service Routine — fungsi yang dipanggil apabila interrupt berlaku. Mesti selesai dalam masa yang ketat. |

## L

| Istilah | Definisi |
|---|---|
| **LLVM IR** | Intermediate Representation LLVM — perwakilan perantara yang digunakan oleh compiler infrastructure LLVM untuk optimization dan code generation. |
| **Lock-Free** | Algoritma yang tidak menggunakan locks (mutex) — mengandalkan atomic operations sahaja. |

## M

| Istilah | Definisi |
|---|---|
| **Memory Ordering** | Peraturan bagaimana operasi memori diordering dalam sistem multi-core. Logicodex menggunakan `Release`/`Acquire`. |

## O

| Istilah | Definisi |
|---|---|
| **Ownership** | Model memori di mana setiap nilai mempunyai pemilik tunggal. Ownership boleh dipindahkan (move) tetapi tidak disalin secara tidak sengaja. |

## P

| Istilah | Definisi |
|---|---|
| **PIC** | Programmable Interrupt Controller — hardware (Intel 8259) yang menguruskan interrupts pada x86_64. |
| **Progressive Disclosure** | Prinsip pendedahan kompleksiti secara beransur-ansur — pemula bermula mudah, pakar mendapat kawalan penuh. |

## R

| Istilah | Definisi |
|---|---|
| **RAII** | Resource Acquisition Is Initialization — pattern di mana sumber (file descriptor, memori) diperoleh semasa pembinaan dan dilepaskan semasa penghancuran automatik. |
| **Reactor** | Event loop untuk I/O deterministik menggunakan epoll/kqueue/IOCP. |
| **RFC** | Request for Comments — proses formal untuk mencadangkan perubahan arkitektur semasa Architecture Freeze. |
| **Ring Buffer** | Struktur data buffer melingkar (circular) yang digunakan untuk SPSC channel. |

## S

| Istilah | Definisi |
|---|---|
| **Service** | Actor khas yang mendengar pada satu port rangkaian dan memproses connections masuk. |
| **Shard** | Unit penjadualan statik yang diikat kepada satu CPU core. |
| **SPSC** | Single-Producer Single-Consumer — queue yang hanya mempunyai satu penulis dan satu pembaca. |
| **Static Topology** | Model di mana struktur program (shards, channels, gates) dikenal pasti pada masa kompil, bukan runtime. |
| **StrictAudioContext** | Kontrak keselamatan masa kompil untuk fungsi audio callback yang melarang 4 jenis pelanggaran. |
| **Supply-Chain Security** | Keselamatan yang memastikan kod yang dikompil tidak diubah oleh pihak ketiga melalui fail `.cap` dan privilege escalation detection. |

## T

| Istilah | Definisi |
|---|---|
| **Taint FSM** | Finite State Machine yang mengekalkan keadaan "kesihatan" setiap connection rangkaian: `Healthy → Suspicious → Closing`. |
| **Topology** | Peta struktur program: service nodes, gate edges, shard nodes, dan door edges. |

## W

| Istilah | Definisi |
|---|---|
| **WASM** | WebAssembly — format instruksi binari portable yang boleh berjalan dalam browser dan persekitaran lain. |
| **WASI** | WebAssembly System Interface — interface standard untuk akses filesystem, rangkaian, dan resources dalam WASM. |
| **WIT** | WASM Interface Types — format IDL untuk menghuraikan interface antara komponen WASM. |
| **Zero-Copy** | Teknik memindahkan data tanpa menyalin — hanya pointer/metadata dipindahkan, bukan payload. |
| **Zero Runtime Mediation** | Prinsip Logicodex di mana tiada proses runtime memediasi operasi program — semua semakan pada masa kompil. |
