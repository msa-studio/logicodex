# Chapter 2: Falsafah Rekabentuk Utama

> *"Syntax adalah antara muka manusia. Semantik adalah kontrak mesin."*

Logicodex dibina di atas 5 prinsip rekabentuk yang menentukan setiap keputusan arkitektur — dari lexer sehinggalah ke backend. Prinsip-prinsip ini bukan aspirasi abstrak; mereka adalah kontrak kejuruteraan yang setiap komponen compiler mesti patuhi.

---

## Prinsip 1: Determinisme Absolute {#determinisme}

### Definisi

Program Logicodex mesti menghasilkan output yang sama untuk input yang sama, setiap kali, tanpa bergantung pada:
- Urutan penjadualan thread
- Keadaan memori yang tidak ditentukan
- Keadaan race condition
- Tingkah laku undefined behavior

### Implementasi Teknikal

Determinisme dicapai melalui 4 mekanisme:

| Mekanisme | Fail Terlibat | Apa yang Dihalang |
|---|---|---|
| **Static Topology** | `src/tier2/topology.rs` | Struktur program (shards, channels, gates) dikenal pasti pada masa kompil, bukan runtime |
| **Explicit Ownership** | `src/tier2/gate.rs`, `lib/core/sync.ldx` | Setiap sumber (memori, fail, socket) mempunyai satu pemilik; tiada perkongsian tidak sengaja |
| **Shard Isolation** | `src/tier2/shard.rs` | Kod selari berjalan pada core CPU berasingan tanpa mutable state kongsi |
| **Capability Gates** | `src/tier2/capability_ir.rs` | Akses kepada operasi berbahaya (hardware, rangkaian, fail) memerlukan kebenaran eksplisit |

### Justifikasi: Kenapa Determinisme Begitu Penting?

Perbincangan awal pembangunan Logicodex sering kembali kepada satu persoalan: *"Adakah determinisme terlalu ketat? Banyak bahasa berjaya tanpa jaminan sekuat ini."*

Jawapannya: **determinisme bukan sekadar ciri; ia adalah asas keselamatan dan kebolehpercayaan.**

Satu program yang tidak deterministik:
- Tidak boleh diuji dengan betul (pass hari ini, fail esok)
- Tidak boleh diaudit dengan selamat (auditor tidak pasti apa yang akan berlaku)
- Tidak boleh dijanakan oleh AI dengan yakin (LLM menghasilkan kod yang mungkin berfungsi atau mungkin tidak)
- Tidak boleh digunakan dalam sistem kritikal keselamatan (medikal, avionik, tenaga nuklear)

> **"Mustahil untuk mengalami race condition atau memory leak"** — kerana semuanya diverifikasi pada masa kompil.

Pernyataan ini bukan tuntutan marketing. Ia adalah tuntutan kejuruteraan yang disokong oleh 148 validator checks yang lulus, termasuk 400+ unit tests, 20 file benchmark, dan 14 alpha releases tanpa regression.

---

## Prinsip 2: Zero Runtime Mediation {#zero-runtime}

### Definisi

Tiada proses runtime yang memediasikan operasi program. Tiada:
- Garbage collector yang berjalan pada latar belakang
- Runtime check yang memakan CPU cycles
- Interpreter atau VM yang menterjemahkan bytecode
- Dynamic dispatch yang tidak dapat diramalkan

### Implementasi Teknikal

Semua semak keselamatan berlaku pada **masa kompil**, bukan masa jalan:

| Sistem | Semakan pada Masa Kompil? | Kos Runtime |
|---|---|---|
| **Capability Gates** | Ya — topology verify sebelum codegen | **ZERO** |
| **Type checking** | Ya — semua dalam `src/semantic.rs` | **ZERO** |
| **Ownership transfer** | Ya — `UseAfterSend` ditolak semasa semantik | **ZERO** |
| **Gate permission** | Ya — `.cap` file generated at compile | **ZERO** |
| **Taint analysis** | Sebahagian — FSM berjalan pada runtime, tetapi transisi deterministic | Minimal (integer comparison) |

### Justifikasi: Kenapa Zero Runtime?

Perbincangan semasa rekabentuk v1.32 (Capability Fabric) menimbulkan persoalan: *"Adakah lebih selamat untuk semak kebenaran pada runtime? Kalau salah, kita boleh detect dan hentikan."*

Jawapannya menolak premis tersebut. **Semakan runtime adalah anti-pattern dalam sistem pengaturcaraan.** Ia bermaksud:
1. Anda tidak cukup yakin dengan analisis anda untuk memberi jaminan pada masa kompil
2. Anda sengaja membiarkan program dihantar ke pengguna dengan lubang keselamatan yang mungkin "ditangkap" pada runtime
3. Anda menerima overhead runtime sebagai "kos keselamatan"

Logicodex mengambil pendekatan berbeza: **kalau tidak boleh buktikan keselamatan pada masa kompil, program itu tidak dikompil.** Ini bukan sikap pragmatik — ini adalah sikap kejuruteraan sistem.

Analogi: Dalam pembinaan jambatan, anda tidak membiarkan jambatan digunakan dan pasang sensor "kalau-runuh-lari" — anda pastikan jambatan itu kukuh sebelum dibuka.

---

## Prinsip 3: Pendedahan Kompleksiti Secara Progresif {#progresif}

### Definisi

Pengguna baru Logicodex mesti dapat menulis program pertama mereka dalam beberapa minit. Jurutera sistem berpengalaman mesti dapat mengakses semua kawalan peringkat rendah. Kompleksiti mesti didedahkan secara beransur-ansur, bukan secara mengejut.

### Spektrum Pengguna

| Tahap | Contoh Pengguna | Syntax yang Digunakan | Capabilities |
|---|---|---|---|
| **Pemula** | Pelajar sekolah | `MULA`, `BINA x = 5`, `PAPAR x`, `TAMAT` | Variabel, aritmetik, cetakan |
| **Pengguna Pertengahan** | Pelajar universiti | `program`, `fn main() {`, `let x: I32 = 5` | Struct, function, FFI ringan |
| **Pakar** | Jurutera sistem | `{`, `let`, `actor`, `gate` | Hardware, network, unsafe |
| **AI Agent** | LLM code generator | Verbose pseudocode alias | Semua — tetapi melalui gate |

### Justifikasi: Kenapa Progresif?

Semasa perbincangan tentang v1.21 baseline, persoalan timbul: *"Adakah alias Melayu terlalu ringan? Pelajar mungkin tidak belajar 'programming yang sebenar'."*

Jawapan kita: **alias Melayu bukan "pemula mode" — mereka adalah permukaan pertama dari satu kontinuum**. Seorang pelajar yang belajar dengan `MULA`/`TAMAT` tidak perlu "unlearn" apa-apa apabila beralih ke `{`/`}`. Mereka belajar konsep yang sama dengan sintaks yang berbeza. Perubahan ini bukan perubahan paradigma — hanya perubahan ketebalan ungkapan.

Pendedahan progresif ini juga kritikal untuk **AI-assisted generation**: LLM menghasilkan kod yang lebih baik apabila sintaks mengekspresikan niat dengan jelas. Kod yang verbose dan berniat jelas lebih mudah untuk diverifikasi oleh compiler dan dianalisis oleh pengguna manusia.

---

## Prinsip 4: Capability-Based Security {#capability}

### Definisi

Akses kepada sumber berbahaya (fail, rangkaian, hardware, audio) tidak diberikan secara default. Setiap akses mesti melalui **gate** yang dideklarasikan secara eksplisit, diverifikasi pada masa kompil, dan direkodkan dalam fail audit (`.cap`).

### Model Capability dalam Logicodex

Logicodex mengamalkan **static capability fabric** — keselamatan bukan runtime library, melainkan compiler feature:

```text
┌─────────────────────────────────────┐
│  Aplikasi Logicodex (.ldx)          │
│  service WebServer {                │
│      port: 8080,                    │
│      requires: Net.Admin,  ◄── GATE │
│      handler: WebHandler            │
│  }                                  │
└──────────────┬──────────────────────┘
               │
               ▼
┌─────────────────────────────────────┐
│  CapabilityGraph IR (v1.35)         │
│  IRGateEdge {                       │
│      domain: Net,                   │
│      operation: Admin,              │
│      capability_ref: CapRef("N1")  │
│  }                                  │
└──────────────┬──────────────────────┘
               │
               ▼
┌─────────────────────────────────────┐
│  Topology Verify (v1.32)            │
│  - Check all gates declared         │
│  - Check no privilege escalation    │
│  - Generate .cap audit file         │
└──────────────┬──────────────────────┘
               │
               ▼
┌─────────────────────────────────────┐
│  Output: Native ELF / WASM / .cap   │
│  - Native: inlined gate checks      │
│  - WASM: capability → WASI mapping  │
│  - .cap: audit trail for supply     │
└─────────────────────────────────────┘
```

### Justifikasi: Kenapa Capability-Based?

Perbincangan semasa rekabentuk v1.32 menimbulkan persoalan: *"Kenapa tidak gunakan model permission Unix-style (read/write/execute)?"*

Jawapan: Model permission Unix adalah **coarse-grained dan runtime-checked**. Ia memisahkan akses kepada fail, proses, dan rangkaian pada peringkat sistem operasi — tetapi tidak memberikan jaminan pada peringkat aplikasi. Sebuah program dengan permission "network" boleh melakukan apa sahaja pada rangkaian, termasuk membuka connection ke server jahat.

Model capability Logicodex adalah **fine-grained dan compile-time verified**:
- `Net.Send` ≠ `Net.Recv` ≠ `Net.Admin`
- `Audio.Main` ≠ `Audio.Callback` (callback melalui StrictAudioContext)
- `HW.GPIO` ≠ `HW.DMA` ≠ `HW.Timer`

Ini bermaksud walaupun seorang penyerang berjaya menyusupi kod, mereka tidak boleh akses sumber yang tidak dideklarasikan — kerana **gate tidak wujud dalam output binari** (gate hanya wujud dalam IR dan topology verify).

> **"Capability gates exist only at compile time. They leave zero footprint at runtime."**

---

## Prinsip 5: Alias-to-Canonical Lexing {#alias}

### Definisi

Logicodex menggunakan **dynamic dictionary mapping** yang membolehkan pelbagai permukaan sintaks memetakan ke token kanonikal yang sama. Lexer membaca `dict/core_map.json` dan menormalkan semua input sebelum parsing bermula.

### Cara Kerja

```json
{
  "MULA": "BeginBlock",
  "BEGIN": "BeginBlock",
  "{": "BeginBlock",
  "TAMAT": "EndBlock",
  "END": "EndBlock",
  "}": "EndBlock",
  "BINA": "Let",
  "let": "Let",
  "CREATE": "Let",
  "PAPAR": "Print",
  "print": "Print",
  "DISPLAY": "Print"
}
```

Penting: `core_map.json` digunakan **secara eksklusif oleh Lexer** semasa tokenization. Parser tidak pernah melihat `MULA` atau `{` — parser melihat `TokenKind::Start`. Ini bukan macro system (yang menulis semula teks), dan bukan syntax desugaring (yang berlaku pada parser side). Ini adalah **token-level normalization**.

### Kebaikan

| Aspek | Kebaikan |
|---|---|
| **Lokalisasi** | `MULA`/`TAMAT` untuk pembangun Melayu, `BEGIN`/`END` untuk Inggeris |
| **Pendidikan** | Pelajar boleh menggunakan kata-kata semula jadi mereka |
| **AI generation** | LLM boleh menghasilkan pseudocode verbose yang jelas niatnya |
| **Domain-specific** | Vokabulari khusus domain boleh ditambah tanpa mengubah compiler |
| **Professional** | Pakar boleh menggunakan shorthand `{`, `let`, `fn` |

### Justifikasi: Kenapa Bukan Multi-Dialect?

Pertanyaan kritikal: *"Kenapa ini bukan pelbagai dialek bahasa yang berbeza?"*

Jawapan: **kerana dialek berpecah ekosistem.** Jika `MULA`/`TAMAT` dan `{`/`}` adalah dialek berbeza, kita akan mendapat:
- Library yang hanya ditulis dalam satu dialek
- Dokumentasi yang perlu diterjemahkan antara dialek
- Komuniti yang terpecah antara "pembangun Melayu" dan "pembangun canonical"

Dengan alias-to-canonical, semua program — sama ada ditulis dengan alias Melayu, alias Inggeris, atau shorthand pakar — mengkompil ke AST yang identik. **Tidak ada "fragmentasi ekosistem" kerana tiada "dialek berbeza".**

Analogi: Ia seperti keyboard QWERTY dan AZERTY — layout berbeza, tetapi huruf yang dihasilkan sama. Anda tidak perlu "menterjemah" dokumen antara pengguna QWERTY dan AZERTY.

---

## Ringkasan 5 Prinsip

| Prinsip | Apa yang Dilindungi | Apa yang Ditolak |
|---|---|---|
| **Determinisme Absolute** | Kebolehramalan, kebolehujian, auditabiliti | Race condition, memory leak, UB |
| **Zero Runtime Mediation** | Prestasi maksimum, jaminan masa nyata | GC pause, runtime check, VM overhead |
| **Pendedahan Progresif** | Kurva pembelajaran lembut, AI-friendly | Cognitive overload, syntax gatekeeping |
| **Capability-Based Security** | Akses terkawal, audit trail, supply-chain security | Permission coarse-grained, runtime check |
| **Alias-to-Canonical** | Lokalisasi, pendidikan, ekosistem tunggal | Fragmentasi dialek, rewrite macro |

Setiap prinsip ini saling mengukuh: alias-to-canonical membolehkan AI dan pelajar menulis kod dengan jelas; capability-based security memastikan kod itu selamat sebelum berjalan; zero runtime mediation memastikan kod itu pantas; determinisme memastikan kod itu boleh diuji; dan pendedahan progresif memastikan semua peringkat kemahiran boleh menggunakan bahasa yang sama.
