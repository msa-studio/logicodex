# Chapter 7: Gate — Keizinan Akses

Gate adalah sistem keamanan Logicodex yang mengawal akses kepada sumber (rangkaian, fail, hardware).

---

## Mengisytiharkan Gate {#isytihar}

### Gate dalam Service

```logicodex
PERKHIDMATAN WebServer {
    port: 8080,
    keperluan: [
        Net.Admin,              -- gate rangkaian
        Storage.Read("/data"),  -- gate fail (scoped)
    ],
    pengendali: HttpHandler,
    dasar: Halang,
}
```

### Gate dalam Actor

```logicodex
PELAKON FileProcessor {
    keperluan: [
        Storage.Read("/input"),
        Storage.Write("/output"),
    ],
    saluran: Channel<FileJob>,
}
```

---

## 3 Jenis Gate {#jenis}

| Jenis | Fungsi | Kos Runtime | Contoh |
|---|---|---|---|
| **DirectCall** | Fungsi inline-able | **Sifar** | Math, crypto, utility |
| **Message** | Async SPSC message | **Sifar** | Sensor, rangkaian, fail |
| **Hardware** | Bare-metal only | **Sifar** | GPIO, DMA, Timer |

### DirectCall Gate

```logicodex
-- Fungsi yang selamat untuk inline
FUNGSI sqrt(x: F64) -> F64
    PERLUKAN Math.Basic     -- DirectCall gate
MULA
    -- Compiler akan inline fungsi ini
    -- Tiada overhead panggilan
    PULANG x.punca_kuasa_dua()
TAMAT
```

### Message Gate

```logicodex
-- Komunikasi async melalui channel
PERKHIDMATAN DataCollector {
    port: 9090,
    keperluan: [
        Net.Recv,               -- Menerima data
        Storage.Write("/raw"),  -- Menulis ke storage
    ],
    pengendali: CollectHandler,
}
```

### Hardware Gate

```logicodex
-- Hanya dalam freestanding / dengan keizinan hardware
FUNGSI gpio_set(pin: U8, value: Bool) -> Void
    PERLUKAN HW.GPIO    -- Hardware gate
MULA
    BACA_VOLATIL(GPIO_BASE + pin SEBAGAI U64)
TAMAT

FUNGSI pwm_set(duty: U8) -> Void
    PERLUKAN HW.Timer   -- Hardware gate
MULA
    -- Konfigurasi timer untuk PWM
TAMAT
```

---

## Service Manifest {#service}

### Sintaks Lengkap

```logicodex
PERKHIDMATAN <nama> {
    port: <nombor>,              -- port TCP/UDP (jika perkhidmatan rangkaian)
    keperluan: [<gate>, ...],    -- senarai gate yang diperlukan
    pengendali: <fungsi>,        -- fungsi handler untuk request
    dasar: <backpressure>,       -- Halang / Gugur_Terlama / Ralat
}
```

### Backpressure Policy

| Dasar | Apa Berlaku | Bila Digunakan |
|---|---|---|
| **Halang** | Tunggu sehingga ruang tersedia | Data tidak boleh hilang |
| **Gugur_Terlama** | Buang data lama, simpan baru | Real-time streaming |
| **Ralat** | Pulangkan ralat kepada caller | Caller handle overflow |

### Contoh Lengkap

```logicodex
-- Perkhidmatan HTTP sederhana
PERKHIDMATAN HttpServer {
    port: 8080,
    keperluan: [
        Net.Admin,
        Storage.Read("/www"),
    ],
    pengendali: handle_http,
    dasar: Halang,
}

-- Perkhidmatan log (real-time)
PERKHIDMATAN LogCollector {
    port: 9090,
    keperluan: [
        Net.Recv,
        Storage.Write("/logs"),
    ],
    pengendali: handle_log,
    dasar: Gugur_Terlama,
}

-- Perkhidmatan hardware sensor
PERKHIDMATAN SensorReader {
    -- Tiada port (bukan perkhidmatan rangkaian)
    keperluan: [
        HW.GPIO,        -- Baca sensor
        HW.Timer,       -- Timing presisi
        Storage.Write("/data"),  -- Simpan bacaan
    ],
    pengendali: handle_sensor,
    dasar: Halang,
}
```

### Handler Function

```logicodex
FUNGSI handle_http(req: Request) -> Response
    PERLUKAN Net.Admin
MULA
    PADAN req.path {
        "/"         => PULANG Response::ok("<h1>Logicodex Server</h1>"),
        "/status"   => PULANG Response::ok("{ \"status\": \"ok\" }"),
        _           => PULANG Response::not_found(),
    }
TAMAT

FUNGSI handle_log(data: &[U8]) -> Void
    PERLUKAN Net.Recv, Storage.Write
MULA
    -- Tulis log ke storage
    tulis_fail("/logs/app.log", data)
TAMAT

FUNGSI handle_sensor() -> Void
    PERLUKAN HW.GPIO, HW.Timer, Storage.Write
MULA
    SEMENTARA BENAR
        BINA bacaan SEBAGAI F64 = baca_sensor_gpio()
        simpan_data("/data/sensor.txt", bacaan)
        tidur_ms(1000)  -- baca setiap 1 saat
    TAMAT_SEMENTARA
TAMAT
```
