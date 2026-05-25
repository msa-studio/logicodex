# Chapter 6: Model Keselamatan — Capability + Gate + Door + Service

> *"Keselamatan dalam Logicodex bukan runtime library — ia adalah compiler feature."*

---

## Gate: Kontrak Keselamatan Masa Kompil {#gate}

### Definisi

Gate adalah kontrak keselamatan yang menyatakan akses apa yang diperlukan oleh sebuah service atau actor. Gate dideklarasikan secara eksplisit dalam kod sumber, diverifikasi pada masa kompil, dan direkodkan dalam fail `.cap`.

### 3 Jenis Gate

| Jenis | Apa | Semakan | Kos Runtime |
|---|---|---|---|
| **DirectCall** | Fungsi inline-able yang selamat | Type compatibility | **Sifar** |
| **Message** | Async SPSC message passing | Channel topology | **Sifar** |
| **Hardware** | Akses bare-metal (GPIO, DMA, Timer) | Target freestanding | **Sifar** (native), **Dihalang** (WASM) |

### Sintaks Gate

```logicodex
service WebServer {
    port: 8080,
    requires: [
        Net.Admin,              // Message gate — async network I/O
        Storage.Read("/data"),  // Message gate — scoped file access
    ],
    handler: HttpHandler,
    policy: Block,
}

// DirectCall gate untuk math functions
let result = math::sqrt(2.0);  // DirectCall — inlined, zero cost
```

### Verifikasi Topology

Sebelum kod dihasilkan, topology verifier memastikan:

```rust
fn verify_topology(topo: &CapabilityTopology) -> Result<(), TopologyError> {
    // 1. Semua gate dideklarasikan
    for service in &topo.services {
        for gate in &service.required_gates {
            if !topo.known_gates.contains(gate) {
                return Err(UnknownGate(gate.clone()));
            }
        }
    }
    
    // 2. Tiada privilege escalation
    for (prev, curr) in topo.history.windows(2) {
        if curr.has_more_privileges_than(prev) {
            return Err(PrivilegeEscalation { 
                from: prev.version, 
                to: curr.version 
            });
        }
    }
    
    // 3. WASM target tidak ada hardware gate
    if topo.target == CompileTarget::Wasm {
        for gate in &topo.gates {
            if gate.is_hardware() {
                return Err(WasmHardwareGate(gate.domain.clone()));
            }
        }
    }
    
    Ok(())
}
```

### Supply-Chain Security: Fail `.cap`

Setiap kompilasi menghasilkan fail `.cap`:

```cap
; topology.cap — Logicodex Capability Audit Trail
; Generated: 2026-05-25T12:00:00Z
; Compiler: logicodex v1.45.0-alpha
; Target: Native (x86_64-unknown-linux-gnu)
; Source: webserver.ldx

[service WebServer]
port=8080
handler=HttpHandler
policy=Block

[gate Net.Admin]
type=Message
domain=Net
operation=Admin
verified=true
checksum=sha256:a1b2c3...

[gate Storage.Read]
type=Message
domain=Storage
operation=Read
path=/data
verified=true
checksum=sha256:d4e5f6...
```

---

## Service: Port-Based Actor + RAII Connection {#service}

### Definisi

Service adalah actor khas yang mendengar pada satu port rangkaian dan memproses connections masuk. Ia menggabungkan:
- Port binding (TCP/UDP)
- Connection management (RAII cleanup)
- Taint tracking (FSM per-connection)
- Backpressure (policy queue)

### Manifest Service

```logicodex
service WebServer {
    port: 8080,              // TCP port
    requires: Net.Admin,     // Capability gate
    handler: HttpHandler,    // Request handler function
    policy: Block,           // Backpressure policy
}
```

### RAII Connection Drop

Connection dalam Logicodex menggunakan RAII (Resource Acquisition Is Initialization) pattern:

```rust
struct Connection {
    fd: RawFd,
    taint: TaintState,
}

impl Drop for Connection {
    fn drop(&mut self) {
        // Deterministic cleanup — close(fd) dipanggil secara automatik
        unsafe { libc::close(self.fd); }
    }
}

// Penggunaan:
{
    let conn = Connection::new(fd);  // fd dibuka
    conn.process();                   // proses request
} // conn keluar scope → Drop::drop() → close(fd) automatik
```

Ini menjamin **tiada kebocoran socket** — walaupun panic, connection akan ditutup.

---

## Taint FSM: Healthy → Suspicious → Closing {#taint}

### Definisi

Taint FSM (Finite State Machine) mengekalkan keadaan "kesihatan" setiap connection. Ia mengesan error pattern dan menutup connection yang mencurigakan.

### Keadaan (States)

```text
                    error_count++
              ┌───────────────────────┐
              │                       │
         ┌────▼────┐            ┌─────┴───┐
         │ Healthy │            │Suspicious│
         │         │            │         │
         │ error=0 │            │ error>0 │
         │         │            │         │
         └────▲────┘            └─────┬───┘
              │                       │
              │ error_count=0         │ error_count > threshold
              │                       │
              └───────────────────────┘
                                      │
                                      ▼
                                  ┌────────┐
                                  │Closing │
                                  │        │
                                  │ close()│
                                  │ cleanup│
                                  └────────┘
```

| Keadaan | error_count | Tindakan I/O | Tindakan Penyembuhan |
|---|---|---|---|
| **Healthy** | 0 | Normal | — |
| **Suspicious** | 1..threshold | Normal, tetapi dipantau | Reset ke Healthy jika OK |
| **Closing** | > threshold | **Dihalang** — hanya cleanup | Tutup fd, RAII cleanup |

### Parameter Taint

| Parameter | Default | Apa |
|---|---|---|
| `error_threshold` | 5 | Berapa error sebelum Closing |
| `timeout_ms` | 30000 | Timeout untuk transisi Suspicious → Closing |
| `recovery_window` | 10000 | Tempoh tanpa error untuk kembali Healthy |

### Integrasi dengan Connection

```rust
impl Connection {
    fn check_taint(&mut self) {
        match self.taint.state {
            TaintState::Healthy => {
                if self.error_count > 0 {
                    self.taint = TaintState::Suspicious {
                        since: Instant::now(),
                    };
                }
            }
            TaintState::Suspicious { since } => {
                if self.error_count > TAINT_THRESHOLD {
                    self.taint = TaintState::Closing;
                } else if since.elapsed() > RECOVERY_WINDOW {
                    self.taint = TaintState::Healthy;
                    self.error_count = 0;
                }
            }
            TaintState::Closing => {
                // Tiada I/O dibenarkan
            }
        }
    }
    
    fn is_trustworthy(&self) -> bool {
        matches!(self.taint.state, TaintState::Healthy | TaintState::Suspicious { .. })
    }
}
```

---

## Backpressure: Block / DropOldest / Error {#backpressure}

### Definisi

Backpressure adalah mekanisme untuk mengendalikan aliran data apabila producer lebih pantas daripada consumer. Ia mengelakkan kehabisan memori akibat queue yang terlalu besar.

### 3 Policy

| Policy | Apa Berlaku | Apabila Digunakan |
|---|---|---|
| **Block** | Producer menunggu sehingga ruang tersedia | Latency boleh diterima, data tidak boleh hilang |
| **DropOldest** | Data lama dibuang, data baru diterima | Real-time streaming, data terkini lebih penting |
| **Error** | Pulangkan error kepada producer | Producer mesti handle overflow sendiri |

### Implementasi

```rust
enum BackpressurePolicy {
    Block,
    DropOldest,
    Error,
}

impl<T> RingBuffer<T> {
    fn enqueue(&mut self, item: T, policy: BackpressurePolicy) -> Result<(), BufferError> {
        if self.is_full() {
            match policy {
                BackpressurePolicy::Block => {
                    while self.is_full() {
                        spin_wait();  // atau yield
                    }
                }
                BackpressurePolicy::DropOldest => {
                    self.dequeue_oldest();  // buang data lama
                }
                BackpressurePolicy::Error => {
                    return Err(BufferError::Full);
                }
            }
        }
        self.push(item);
        Ok(())
    }
}
```

---

## Supply-Chain Security: Fail `.cap` + Privilege Escalation Detection {#supply}

### Fail `.cap` sebagai Audit Trail

Setiap kompilasi menghasilkan fail `.cap` yang:
1. Merekodkan semua capability yang digunakan
2. Menyertakan checksum (SHA-256) untuk integriti
3. Menyertakan timestamp untuk urutan waktu
4. Menyertakan versi compiler untuk reproducibility

### Privilege Escalation Detection

```rust
fn diff_topology(prev: &CapFile, curr: &CapFile) -> Vec<CapabilityDiff> {
    let mut diffs = Vec::new();
    
    for (service_name, curr_gates) in &curr.services {
        let prev_gates = prev.services.get(service_name);
        
        match prev_gates {
            None => diffs.push(CapabilityDiff::NewService(service_name.clone())),
            Some(prev_gates) => {
                for gate in curr_gates {
                    if !prev_gates.contains(gate) {
                        diffs.push(CapabilityDiff::PrivilegeEscalation {
                            service: service_name.clone(),
                            added_gate: gate.clone(),
                        });
                    }
                }
            }
        }
    }
    
    diffs
}
```

### Workflow Supply-Chain

```text
Developer                     CI/CD                         Production
    │                            │                               │
    ▼                            ▼                               ▼
┌─────────┐              ┌──────────────┐                ┌─────────────┐
│ Compile  │── .cap ────►│ Diff against │─── approve? ──►│ Deploy with │
│ (v1.2)   │              │ baseline.cap │                │ .cap audit  │
└─────────┘              └──────────────┘                └─────────────┘
                               │
                               ▼ (if escalation detected)
                        ┌──────────────┐
                        │ REJECT build │
                        │ Alert admin  │
                        └──────────────┘
```

---

## Ringkasan Model Keselamatan

| Komponen | Apa | Kos Runtime |
|---|---|---|
| **Gate** | Kontrak capability masa kompil | **Sifar** |
| **Topology Verify** | Semakan gate + privilege escalation | **Sifar** |
| **Service** | Port-based actor dengan RAII cleanup | Hanya `close(fd)` pada drop |
| **Taint FSM** | Healthy → Suspicious → Closing | Beberapa integer comparisons |
| **Backpressure** | Block / DropOldest / Error | Bergantung pada policy |
| **Fail `.cap`** | Audit trail | **Sifar** (fail output) |
| **Diff Topology** | Privilege escalation detection | **Sifar** (analisis fail) |

Keseluruhan model keselamatan mempunyai **kos runtime hampir sifar**. Semua semakan kritikal berlaku pada masa kompil. Runtime hanya perlu mengekalkan FSM taint (beberapa integer) dan backpressure policy — kedua-duanya adalah operasi O(1).
