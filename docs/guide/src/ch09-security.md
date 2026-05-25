# Chapter 9: Keamanan dan Audit

Bab ini menerangkan sistem keamanan Logicodex dan cara mengaudit aplikasi anda.

---

## Fail `.cap` dan Supply-Chain {#cap}

### Apa Itu Fail `.cap`?

Fail `.cap` (capability file) adalah audit trail yang dihasilkan setiap kali anda mengkompil program Logicodex. Ia merekodkan semua capability gate yang digunakan oleh program anda.

### Struktur Fail `.cap`

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

### Lokasi Fail

Fail `.cap` dihasilkan dalam direktori yang sama dengan output kompilasi:

```bash
logicodex input.ldx -o /tmp/myapp
# Menghasilkan:
#   /tmp/myapp        (binari)
#   /tmp/myapp.cap    (audit trail)
```

### Verifikasi Fail `.cap`

```bash
# Semak integriti fail .cap
logicodex --verify-cap /tmp/myapp.cap

# Bandingkan dengan baseline
diff /tmp/myapp.cap /baseline/webserver.cap
```

---

## Privilege Escalation Detection {#privilege}

### Apa Itu Privilege Escalation?

Privilege escalation berlaku apabila versi baru program meminta lebih banyak capability daripada versi lama — tanpa justifikasi yang jelas. Ini boleh menjadi tanda:
- Kod berniat jahat (malicious code)
- Dependency yang tidak diperiksa
- Refactoring yang tidak sengaja menambah keperluan

### Mengesan Privilege Escalation

```bash
# Simpan baseline .cap
mv /tmp/myapp.cap /baseline/myapp_v1.cap

# Kompil versi baru
logicodex input_v2.ldx -o /tmp/myapp_v2

# Bandingkan
diff /baseline/myapp_v1.cap /tmp/myapp_v2.cap

# Output jika ada escalation:
# 4a5,8
# > [gate Storage.Write]
# > type=Message
# > domain=Storage
# > operation=Write
```

### Manual Detection

```logicodex
-- Anda juga boleh deteksi dalam kod
FUNGSI audit_topology() -> Void
    PERLUKAN Audit.Admin
MULA
    BINA baseline SEBAGAI CapFile = baca_cap("/baseline/app.cap")
    BINA semasa SEBAGAI CapFile = baca_cap("/build/app.cap")
    
    BINA perbezaan SEBAGAI Vec<CapabilityDiff> = diff_topology(&baseline, &semasa)
    
    JIKA !perbezaan.kosong()
        PAPAR "AMARAN: Privilege escalation dikesan!"
        UNTUK diff DARI perbezaan
            PAPAR "  - " + diff.huraian()
        TAMAT_UNTUK
        -- Hentikan deployment
        abort()
    TAMAT_JIKA
TAMAT
```

---

## Amalan Terbaik Keamanan {#amalan}

### 1. Gate Principle of Least Privilege

Hanya minta keizinan yang anda perlukan:

```logicodex
-- ❌ Terlalu banyak keizinan
PERKHIDMATAN Overprivileged {
    keperluan: [
        Net.Admin,          -- Admin? padahal cuma perlu hantar
        Storage.Read,       -- Semua fail? padahal cuma 1 direktori
        Storage.Write,      -- Tulis juga? tidak perlu
        HW.GPIO,            -- Hardware? ini perkhidmatan web
    ],
}

-- ✅ Minimum keizinan
PERKHIDMATAN Minimal {
    keperluan: [
        Net.Send,           -- Hanya hantar data
        Storage.Read("/data/public"),  -- Hanya direktori ini
    ],
}
```

### 2. Audit Setiap Kompilasi

```bash
#!/bin/bash
# pre_deploy.sh

set -e

# Kompil
logicodex app.ldx -o /tmp/app

# Verifikasi .cap
logicodex --verify-cap /tmp/app.cap

# Bandingkan dengan baseline
if [ -f baseline/app.cap ]; then
    diff baseline/app.cap /tmp/app.cap > /tmp/cap_diff.txt
    if [ -s /tmp/cap_diff.txt ]; then
        echo "PRIVILEGE ESCALATION DETECTED!"
        cat /tmp/cap_diff.txt
        exit 1
    fi
fi

# Deploy
mv /tmp/app /deploy/app
mv /tmp/app.cap /deploy/app.cap
```

### 3. Separate Dev/Prod Baselines

```bash
# Development — lebih permisif
cp baseline/dev.cap baseline/app.cap

# Production — lebih ketat
cp baseline/prod.cap baseline/app.cap
```

### 4. Monitor Runtime

```logicodex
-- Log setiap akses capability
FUNGSI log_access(gate: GateRef, operation: Text) -> Void
    PERLUKAN Audit.Log
MULA
    BINA entry SEBAGAI LogEntry = LogEntry {
        timestamp: masa_sekarang(),
        gate: gate,
        operation: operation,
        source: dapat_caller(),
    }
    tulis_log("/var/log/capability.log", entry)
TAMAT
```

### 5. Rotasi Baseline Secara Berkala

```bash
# Rotasi bulanan
0 0 1 * * /opt/logicodex/scripts/rotate_baseline.sh
```

---

## Ringkasan Keamanan

| Langkah | Mengapa | Seberapa Kerap |
|---|---|---|
| Periksa `.cap` setiap build | Pastikan tiada gate tidak dijangka | Setiap build |
| Diff dengan baseline | Detect privilege escalation | Setiap deployment |
| Minimum privilege | Kurang permukaan serangan | Semasa rekabentuk |
| Audit log | Trace akses capability | Real-time |
| Rotasi baseline | Elakkan baseline lama | Bulanan |
