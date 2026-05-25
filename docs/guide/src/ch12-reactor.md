# Chapter 12: Reactor — Event Loop

Reactor adalah event loop yang memproses I/O events secara deterministik.

---

## Service dan Port Binding {#service}

### Membuat Service Rangkaian

```logicodex
PERKHIDMATAN EchoServer {
    port: 8080,
    keperluan: [Net.Admin],
    pengendali: handle_echo,
    dasar: Halang,
}

FUNGSI handle_echo(sambungan: Connection) -> Void
    PERLUKAN Net.Admin
MULA
    BINA buffer SEBAGAI [U8; 1024] = [0; 1024]
    
    SEMENTARA sambungan.aktif()
    MULA
        BINA n SEBAGAI I32 = sambungan.baca(&mut buffer)
        JIKA n > 0
            sambungan.tulis(&buffer[0..n])   -- echo balik
        TAMAT_JIKA
    TAMAT_SEMENTARA
    
    -- Connection ditutup automatik (RAII)
TAMAT
```

### Event Types

| Event | Makna | Tindakan |
|---|---|---|
| `EPOLLIN` | Data tersedia untuk baca | Panggil handler read |
| `EPOLLOUT` | Socket sedia untuk tulis | Panggil handler write |
| `EPOLLERR` | Ralat pada socket | Trigger Taint FSM |
| `EPOLLHUP` | Peer menutup connection | Tutup connection |

---

## Connection dan Taint FSM {#connection}

### Keadaan Connection

Setiap connection mempunyai keadaan taint:

```text
Healthy ──error──▶ Suspicious ──threshold──▶ Closing ──▶ closed
   ▲                    │
   │                    │ reset counter
   └────────────────────┘
```

### Mengakses Keadaan

```logicodex
FUNGSI handle_connection(conn: Connection) -> Void
MULA
    PAPAR "Connection dari: " + conn.alamat_remote()
    PAPAR "Keadaan: " + conn.keadaan_taint()
    
    -- Periksa sama ada connection dipercayai
    JIKA !conn.dipercayai()
        PAPAR "Connection tidak dipercayai — menolak"
        PULANG
    TAMAT_JIKA
    
    -- Proses seperti biasa
    proses_request(conn)
    
    PAPAR "Connection ditutup. Keadaan akhir: " + conn.keadaan_taint()
TAMAT
```

---

## Mengendalikan Serangan Rangkaian {#serangan}

### Slowloris Attack

Serangan yang menghantar data secara perlahan-lahan untuk mengekalkan connection terbuka.

```logicodex
-- Taint FSM akan detect:
-- 1. Connection yang lambat → Suspicious
-- 2. Timeout → Closing
-- 3. Connection ditutup

FUNGSI setup_slowloris_protection() -> Void
MULA
    -- Tetapkan threshold rendah untuk connection lambat
    TetapkanTaintThreshold(3)      -- 3 error sebelum Closing
    TetapkanTaintTimeout(5000)     -- 5 saat timeout
    TetapkanRecoveryWindow(10000)  -- 10 saat recovery window
TAMAT
```

### SYN Flood

Serangan yang menghantar banyak SYN tanpa ACK.

```logicodex
-- Backpressure akan handle:
-- 1. Terlalu banyak connection → channel penuh
-- 2. Policy Halang → tunggu
-- 3. Policy Gugur_Terlama → buang connection lama

PERKHIDMATAN ProtectedServer {
    port: 443,
    keperluan: [Net.Admin],
    pengendali: handle_request,
    dasar: Gugur_Terlama,   -- buang connection lama jika penuh
}
```

### Malformed Packets

Data yang tidak sah yang mungkin trigger bugs.

```logicodex
FUNGSI handle_request(conn: Connection) -> Void
MULA
    BINA buffer SEBAGAI [U8; 4096] = [0; 4096]
    
    BINA n SEBAGAI I32 = conn.baca(&mut buffer)
    JIKA n < 0
        -- EPOLLERR → taint akan trigger
        PAPAR "Ralat baca — connection akan ditutup oleh Taint FSM"
        PULANG
    TAMAT_JIKA
    
    -- Validasi data sebelum proses
    JIKA !validasi_header(&buffer)
        PAPAR "Header tidak sah — menolak request"
        conn.taint(TaintReason::InvalidData)
        PULANG
    TAMAT_JIKA
    
    -- Proses data yang sah
    proses_data(&buffer)
TAMAT
```

---

## Latihan

1. Tulis echo server yang menghantar balik setiap pesanan
2. Tulis HTTP server yang menghantar 404 untuk path tidak diketahui
3. Konfigurasikan Taint FSM untuk detect connection yang lambat (>10 saat tanpa aktiviti)
