# Chapter 8: Door — Saluran Data

Door (channel) adalah mekanisme komunikasi antara actor atau shard dalam Logicodex.

---

## Membuat Channel {#channel}

### Sintaks

```logicodex
-- Membuat channel dengan kapasiti
BINA ch SEBAGAI Channel<I32> = Channel::baru(100)

-- Channel untuk tipe komposit
BINA ch_msg SEBAGAI Channel<Message> = Channel::baru(50)

-- Channel untuk struct
BINA ch_data SEBAGAI Channel<DataPacket> = Channel::baru(10)
```

### Kapasiti Channel

| Kapasiti | Penggunaan |
|---|---|
| 1 | Synchronous (blocking send/receive) |
| 10-100 | Normal (buffered) |
| 1000+ | High-throughput (banyak data) |

---

## Send dan Receive {#sendrecv}

### Operasi Asas

```logicodex
PELAKON Producer {
    saluran: Channel<I32>,
    
    FUNGSI jalankan() -> Void
    MULA
        UNTUK i DARI 0 HINGGA 100
            saluran.hantar(i)       -- send (blocking jika penuh)
            PAPAR "Dihantar: " + i
        TAMAT_UNTUK
    TAMAT
}

PELAKON Consumer {
    saluran: Channel<I32>,
    
    FUNGSI jalankan() -> Void
    MULA
        UNTUK _ DARI 0 HINGGA 100
            BINA nilai SEBAGAI I32 = saluran.terima()   -- receive (blocking)
            PAPAR "Diterima: " + nilai
        TAMAT_UNTUK
    TAMAT
}
```

### Operasi Non-Blocking

```logicodex
PELAKON NonBlockingExample {
    saluran: Channel<I32>,
    
    FUNGSI jalankan() -> Void
    MULA
        -- Try send (tidak blocking)
        PADAN saluran.cuba_hantar(42) {
            Result::Ok(_)     => PAPAR "Berjaya dihantar",
            Result::Err(_)    => PAPAR "Channel penuh",
        }
        
        -- Try receive (tidak blocking)
        PADAN saluran.cuba_terima() {
            Option::Some(val) => PAPAR "Diterima: " + val,
            Option::None      => PAPAR "Channel kosong",
        }
        
        -- Receive dengan timeout
        PADAN saluran.terima_timeout_ms(1000) {
            Option::Some(val) => PAPAR "Diterima: " + val,
            Option::None      => PAPAR "Timeout!",
        }
    TAMAT
}
```

### Zero-Copy Transfer

```logicodex
-- Data besar dipindahkan tanpa salinan
STRUKTUR ImageFrame {
    data: Buffer<U8>,      -- 4MB buffer
    width: I32,
    height: I32,
    timestamp: U64,
}

PELAKON Camera {
    output: Channel<ImageFrame>,
    
    FUNGSI capture() -> Void
    MULA
        SEMENTARA BENAR
            BINA frame SEBAGAI ImageFrame = ambil_gambar_kamera()  -- 4MB
            output.hantar(frame)       -- ownership moved, TIADA salinan
            -- ❌ tidak boleh guna 'frame' selepas ini
        TAMAT_SEMENTARA
    TAMAT
}

PELAKON Processor {
    input: Channel<ImageFrame>,
    
    FUNGSI process() -> Void
    MULA
        SEMENTARA BENAR
            BINA frame SEBAGAI ImageFrame = input.terima()   -- ownership received
            proses_gambar(&frame)        -- proses 4MB data
            -- frame di-drop automatik di akhir scope
        TAMAT_SEMENTARA
    TAMAT
}
```

---

## Backpressure Policy {#backpressure}

### Setting Policy

Policy ditetapkan dalam service manifest:

```logicodex
-- Block policy (default)
PERKHIDMATAN ReliableService {
    keperluan: [Net.Admin],
    pengendali: handle_request,
    dasar: Halang,          -- tunggu jika penuh
}

-- DropOldest policy
PERKHIDMATAN StreamService {
    keperluan: [Net.Recv],
    pengendali: handle_stream,
    dasar: Gugur_Terlama,   -- buang data lama
}

-- Error policy
PERKHIDMATAN FastService {
    keperluan: [Net.Admin],
    pengendali: handle_fast,
    dasar: Ralat,            -- pulangkan ralat jika penuh
}
```

### Tingkah Laku Policy

```text
Producer --hantar()--> Channel (kapasiti = 5)

Block:
  [1][2][3][4][5] --> penuh! --> producer tunggu <-- data tidak hilang
  [1][2][3][4][5][6][7]...   <-- lambat tetapi selamat

DropOldest:
  [1][2][3][4][5] --> penuh! --> [6][2][3][4][5] --> [6][7][3][4][5]
  data terlama dibuang, terima data terkini

Error:
  [1][2][3][4][5] --> penuh! --> Result::Err(ChannelFull)
  producer mesti handle ralat sendiri
```

### Pilih Policy yang Betul

| Senario | Policy | Mengapa |
|---|---|---|
| HTTP request/response | Block | Request tidak boleh hilang |
| Video streaming | DropOldest | Frame terkini lebih penting |
| Real-time telemetry | DropOldest | Data lama tidak berguna |
| Financial transaction | Block | Setiap transaksi mesti diproses |
| Health monitoring | Error | Alert jika system overload |
