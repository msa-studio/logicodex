# Chapter 5: Model Concurrency — Actor + Channel + Shard + Reactor

> *"Concurrency dalam Logicodex bukan tentang kelajuan — ia tentang determinisme. Kelajuan datang sebagai side effect."*

---

## Actor: Unit Komputasi Terpencil {#actor}

### Definisi

Actor dalam Logicodex adalah unit komputasi terpencil (isolated computation unit) yang:
- Memiliki state sendiri (tidak dikongsi dengan actor lain)
- Berkomunikasi hanya melalui message passing (tidak ada shared memory)
- Dijadualkan pada satu shard (CPU core) secara statik
- Dibangunkan dan dihancurkan secara deterministik

### Sintaks

```logicodex
actor Worker {
    let state: I32 = 0;
    let ch: Channel<Message>;
    
    fn handle(msg: Message) {
        match msg {
            Work(data) => { state = state + data; },
            Stop       => { ch.send(Done); }
        }
    }
}

// Spawn actor
spawn Worker();
```

### Sifat-sifat Actor

| Sifat | Implementasi | Fail |
|---|---|---|
| **Isolasi** | Tiada mutable state dikongsi | `src/tier2/shard.rs` |
| **Komunikasi** | Hanya melalui SPSC channel | `lib/core/ring_buffer.ldx` |
| **Penjadualan** | Static assignment ke shard | `src/tier2/topology.rs` |
| **Lifecycle** | Deterministik: spawn → run → stop | `src/net/reactor.rs` |

---

## Channel: SPSC Ring Buffer Zero-Copy {#channel}

### Struktur Data

Channel Logicodex menggunakan **Single-Producer Single-Consumer (SPSC) ring buffer** yang lock-free:

```
Ring Buffer (size = N)
┌─────────────────────────────────────┐
│  [0]  [1]  [2]  [3]  ...  [N-1]   │
│   │    │    │    │         │        │
│   ▼    ▼    ▼    ▼         ▼        │
│  ┌─┐  ┌─┐  ┌─┐  ┌─┐      ┌─┐      │
│  │A│  │B│  │ │  │ │  ... │ │      │
│  └─┘  └─┘  └─┘  └─┘      └─┘      │
│   ▲         ▲                        │
│   │         │                        │
│ producer  consumer                   │
│ (write)   (read)                     │
└─────────────────────────────────────┘

Producer: write at head, advance head (Release ordering)
Consumer: read at tail, advance tail (Acquire ordering)
```

### Operasi

| Fungsi | Bloking? | Return | Kos |
|---|---|---|---|
| `ring_send()` | Ya | `()` | O(1) — atomic write |
| `ring_recv()` | Ya | `T` | O(1) — atomic read |
| `ring_try_send()` | Tidak | `Result<(), Full>` | O(1) |
| `ring_try_recv()` | Tidak | `Option<T>` | O(1) |
| `ring_timeout_recv()` | Dengan timeout | `Option<T>` | O(1) |

### Memory Ordering

```rust
// Producer (sender)
self.head.store(new_head, Ordering::Release);

// Consumer (receiver)
let tail = self.tail.load(Ordering::Acquire);
```

Pair `Release`/`Acquire` menjamin bahawa semua tulisan sebelum `Release` akan kelihatan kepada pembaca selepas `Acquire`. Ini adalah mekanisme sinkronisasi paling minimum yang diperlukan — tiada mutex, tiada semaphore.

### Zero-Copy Transfer

```logicodex
let buffer: Buffer<U8> = Buffer::new(4096);
// isi buffer...
ch.send(buffer);        // ownership moved — tiada salinan

// Di pihak penerima:
let received: Buffer<U8> = ch.recv();  // ownership received
```

Payload `Buffer<U8>` (4KB) tidak disalin. Hanya pointer dan metadata dipindahkan melalui ring buffer. Ini menjadikan latency < 100ns walaupun untuk payload besar.

---

## Door: Cross-Shard Transport {#door}

### Definisi

Door adalah channel yang menghubungkan actor pada shard berbeza. Ia menggunakan mekanisme SPSC yang sama, tetapi mungkin melibatkan cross-core communication (melalui shared cache line atau memory bus).

### Topologi Door

```text
┌─────────────┐      Door 0      ┌─────────────┐
│   Shard 0   │ ◄──────────────► │   Shard 1   │
│  (Core 0)   │                  │  (Core 1)   │
│             │      Door 1      │             │
│  [Actor A]  │ ◄──────────────► │  [Actor C]  │
│  [Actor B]  │                  │  [Actor D]  │
└─────────────┘                  └─────────────┘
       │                                │
       │          Door 2                │
       └──────────────────────────────► │
                                        │
                              ┌─────────────┐
                              │   Shard 2   │
                              │  (Core 2)   │
                              │             │
                              │  [Actor E]  │
                              └─────────────┘
```

### Door vs Channel

| Aspek | Channel (dalam shard) | Door (cross-shard) |
|---|---|---|
| Lokasi | Dua queue dalam shard sama | Queue mungkin pada core berbeza |
| Latency | ~50ns | ~100ns (cross-core cache coherence) |
| Memory ordering | Sama core | Cross-core Acquire/Release |
| Kapasiti | 1024 messages | 1024 messages |

---

## Shard: Unit Penjadualan pada Core CPU {#shard}

### Definisi

Shard adalah unit penjadualan statik yang diikat kepada satu CPU core. Setiap shard mengekalkan:
- Satu event loop (reactor)
- Satu atau lebih actors
- Satu set doors (cross-shard channels)
- Bajet memori (memory budget)

### ShardTopology

ShardTopology ditentukan pada masa kompil dan diverifikasi sebelum kod dihasilkan:

```rust
ShardTopology {
    shards: [
        ShardNode { core_id: 0, actors: ["WebServer", "Logger"], budget: 256MB },
        ShardNode { core_id: 1, actors: ["Worker"], budget: 512MB },
        ShardNode { core_id: 2, actors: ["Worker"], budget: 512MB },
    ]
}
```

### Verifikasi Topology

6 semakan dijalankan pada topology:

| Semakan | Apa Dihalang |
|---|---|
| `ActorDuplication` | Actor diumpukkan ke >1 shard |
| `ShardOverflow` | Jumlah actors melebihi kapasiti shard |
| `BudgetExceeded` | Jumlah budget melebihi RAM tersedia |
| `InvalidCoreId` | Core ID tidak wujud pada mesin target |
| `OrphanActor` | Actor tidak diumpukkan ke mana-mana shard |
| `CycleInDoorGraph` | Cycle dalam door graph (boleh menyebabkan deadlock) |

### CPU Affinity (v1.39)

Setiap shard diikat ke core melalui system call:

| Platform | Syscall | Fail |
|---|---|---|
| Linux | `sched_setaffinity` | `src/os/syscall.rs` |
| macOS | `thread_policy_set` | `src/os/syscall.rs` |
| Windows | `SetThreadAffinityMask` | `src/os/syscall.rs` |

```rust
// Linux implementation
unsafe {
    let mut cpu_set = std::mem::zeroed::<libc::cpu_set_t>();
    libc::CPU_SET(core_id, &mut cpu_set);
    libc::sched_setaffinity(0, std::mem::size_of::<libc::cpu_set_t>(), &cpu_set);
}
```

---

## Reactor: Event Loop Deterministik {#reactor}

### Definisi

Reactor adalah event loop yang memproses I/O events (rangkaian, timer, signals) secara deterministik. Ia menggunakan epoll (Linux), kqueue (macOS), atau IOCP (Windows) untuk multiplexer I/O.

### Arsitektur Reactor

```text
┌─────────────────────────────────┐
│           Reactor                │
│                                  │
│  ┌─────────┐  ┌───────────────┐ │
│  │  epoll  │  │  Connections  │ │
│  │  loop   │  │  (fd + Taint) │ │
│  │         │  │               │ │
│  │epoll_wait│  │Conn A: Healthy │ │
│  │   ▲     │  │Conn B: Closing │ │
│  │   │     │  └───────────────┘ │
│  │   │     │                    │
│  │   │     │  ┌───────────────┐ │
│  │ events  │  │  Backpressure │ │
│  │         │  │  Policy: Block│ │
│  └─────────┘  └───────────────┘ │
└─────────────────────────────────┘
```

### Event Processing

```rust
loop {
    let events = epoll_wait(epoll_fd, -1);  // blocking wait
    for event in events {
        match event {
            EPOLLIN  => connection.read(),
            EPOLLOUT => connection.write(),
            EPOLLERR => connection.taint(Error),  // trigger taint FSM
            EPOLLHUP => connection.close(),       // peer closed
        }
    }
}
```

### Integrasi dengan Service

```logicodex
service WebServer {
    port: 8080,
    requires: Net.Admin,
    handler: HttpHandler,
    policy: Block,
}
```

Service ini diterjemahkan ke:
1. `socket()` + `bind()` + `listen()` pada port 8080
2. `epoll_ctl(ADD)` untuk socket ke epoll loop
3. `HttpHandler` dipanggil apabila `EPOLLIN` diterima
4. Backpressure policy `Block` diterapkan kepada ring buffer

---

## Ringkasan Model Concurrency

| Komponen | Analogi | Fungsi |
|---|---|---|
| **Actor** | Pekerja dalam bilik berasingan | Unit komputasi terpencil |
| **Channel** | Tiub pneumatic dalam bilik | Komunikasi intra-shard |
| **Door** | Tiub pneumatic antara bilik | Komunikasi cross-shard |
| **Shard** | Bilik dengan pekerja tetap | Unit penjadualan pada core |
| **Reactor** | Pengurus bangunan | Event loop I/O deterministik |

Kesemua komponen ini bekerja bersama untuk membentuk model concurrency yang **100% deterministik** — tidak ada race condition, tidak ada deadlock, tidak ada memory leak yang disebabkan oleh concurrency.
