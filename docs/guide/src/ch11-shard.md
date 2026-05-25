# Chapter 11: Shard dan Penjadualan

Shard adalah unit penjadualan statik yang diikat kepada satu CPU core.

---

## Konsep Shard dan Core {#konsep}

### Apa Itu Shard?

- **Shard** = Unit penjadualan (watak actor yang diumpukkan ke satu core)
- **Core** = Core CPU fizikal (hardware)
- **Affinity** = Ikatan shard ke core (tidak berpindah)

```text
CPU dengan 4 Core:
┌──────────────────────────────────────────┐
│  Core 0    Core 1    Core 2    Core 3   │
│  ┌────┐   ┌────┐   ┌────┐   ┌────┐    │
│  │ S0 │   │ S1 │   │ S2 │   │ S3 │    │
│  │ A1 │   │ A2 │   │ A3 │   │ A4 │    │
│  │ A5 │   │    │   │ A6 │   │    │    │
│  └────┘   └────┘   └────┘   └────┘    │
│  WebServer Logger   Worker   Monitor   │
└──────────────────────────────────────────┘

S0-S3 = Shards (diikat ke core masing-masing)
A1-A6 = Actors (diumpukkan ke shard)
```

### Kenapa Static Affinity?

| Aspek | Static (Logicodex) | Dynamic (Thread pool) |
|---|---|---|
| Cache behavior | Konsisten (data sentiasa pada core sama) | Tidak konsisten (data berpindah core) |
| Latency | Dapat diramalkan | Tidak dapat diramalkan |
| Debugging | Mudah (actor sentiasa pada core sama) | Sukar (actor berpindah core) |
| Scaling | Linear (setiap core = satu shard) | Sub-linear (contention) |

---

## Topologi Statik {#topologi}

### Definisi Topologi

Topologi didefinisikan pada masa kompil, bukan runtime:

```logicodex
-- topology.ldx — definisi shard
TOPI_SHARD aplikasi {
    -- Shard 0: Web (core 0)
    shard Web {
        core: 0,
        pelakon: [WebServer, StaticFileHandler],
        bajet_memori: 256MB,
    }
    
    -- Shard 1: Log (core 1)
    shard Log {
        core: 1,
        pelakon: [Logger, MetricsCollector],
        bajet_memori: 128MB,
    }
    
    -- Shard 2: Worker (core 2)
    shard Compute {
        core: 2,
        pelakon: [Worker1, Worker2],
        bajet_memori: 512MB,
    }
    
    -- Shard 3: Monitor (core 3)
    shard Monitor {
        core: 3,
        pelakon: [HealthChecker],
        bajet_memori: 64MB,
    }
    
    -- Door (saluran antara shard)
    pintu {
        Web.Logger     --> Log.LoggerInput,
        Compute.Output --> Log.ComputeLog,
        Monitor.Check  --> Web.HealthCheck,
    }
}
```

### Verifikasi Topologi

Compiler akan semak:

| Semakan | Apa | Ralat Jika Gagal |
|---|---|---|
| ActorDuplication | Tiada actor pada >1 shard | `E009: Actor 'X' diumpukkan ke multiple shards` |
| ShardOverflow | Jumlah actors ≤ kapasiti | `E011: Shard 'X' melebihi kapasiti` |
| BudgetExceeded | Jumlah budget ≤ RAM | `E012: Bajet memori melebihi RAM` |
| InvalidCoreId | Core ID wujud | `E013: Core ID X tidak wujud` |
| OrphanActor | Semua actor diumpukkan | `E014: Actor 'X' tidak diumpukkan` |
| NoCycle | Tiada cycle dalam door graph | `E015: Cycle dalam door graph` |

### Dynamic Core Detection

```logicodex
-- Dapatkan jumlah core pada runtime
BINA core_count SEBAGAI I32 = parallelisme_tersedia()
PAPAR "Core tersedia: " + core_count

-- Dapatkan core semasa
BINA core_sekarang SEBAGAI I32 = core_sekarang()
PAPAR "Berjalan pada core: " + core_sekarang
```

---

## Memory Budgeting {#memory}

### Per-Shard Budget

Setiap shard mempunyai bajet memori yang ditetapkan:

```logicodex
shard Compute {
    core: 2,
    pelakon: [Worker1, Worker2],
    bajet_memori: 512MB,   -- shard ini tidak boleh melebihi 512MB
}
```

### Monitoring Penggunaan

```logicodex
FUNGSI periksa_bajet(shard: &Shard) -> Void
MULA
    BINA guna SEBAGAI USize = shard.penggunaan_memori()
    BINA bajet SEBAGAI USize = shard.bajet_memori
    
    BINA peratus SEBAGAI F64 = (guna SEBAGAI F64 / bajet SEBAGAI F64) * 100.0
    
    JIKA peratus > 90.0
        PAPAR "AMARAN: Shard '" + shard.nama + "' menggunakan " + peratus + "% bajet!"
    TAMAT_JIKA
    
    JIKA peratus > 95.0
        PAPAR "KRITIKAL: Memori hampir habis! Mengaktifkan Gugur_Terlama policy."
        shard.dasar = Gugur_Terlama
    TAMAT_JIKA
TAMAT
```

---

## Latihan

1. Tulis topologi untuk aplikasi chat dengan 4 shard: Gateway, Auth, Chat, Notify
2. Kira bajet memori jika setiap connection memerlukan 4KB dan anda menjangkakan 10,000 concurrent connections
