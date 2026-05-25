# Logicodex Wiki Documentation

Direktori ini mengandungi **2 wiki berasingan** yang melengkapi satu sama lain untuk membantu pengguna dan kontributor Logicodex.

---

## Wiki 1: Experimental Compiler Philosophy and Architecture

**Lokasi:** [`docs/white-paper/`](./white-paper/)

**Untuk siapa:** Jurutera sistem, pengkaji bahasa pengaturcaraan, kontributor, dan sesiapa yang ingin memahami **kenapa** Logicodex direka begitu rupa — serta bagaimana falsafah compiler eksperimental ini berevolusi melalui 14 alpha releases.

> **Hubungan dengan White Paper Asal:** Dokumen ini adalah **evolusi hidup** daripada [`WHITE_PAPER.md`](../../WHITE_PAPER.md) di root repositori. Kedua-dua dokumen telah **disejajarkan** (aligned) secara sistematik:
> - WHITE_PAPER.md = spesifikasi baseline formal v1.21 dengan nota evolusi dan peta jalan bertier (COMPLETED/RESEARCH/LONG-TERM)
> - Wiki ini = evolusi v1.21–v1.45 dengan justifikasi setiap keputusan dan peta jalan masa depan yang sama tier
>
> Tiada percanggahan antara kedua-dua dokumen. Setiap tuntutan dalam WHITE_PAPER.md yang ditanda **COMPLETED** mempunyai bukti validator dalam repositori ini.

**Kandungan:**
- Falsafah rekabentuk (5 prinsip utama)
- Evolusi arkitektur dari v1.21 hingga v1.45 (9 fasa)
- **Refleksi perbincangan dan justifikasi** keputusan arkitektur:
  - Kenapa Actor Model + Channel?
  - Kenapa Zero-Copy Ownership Transfer?
  - Kenapa Compile-Time Capability?
  - Kenapa Taint FSM?
  - Kenapa CapabilityGraph IR sebagai "Single Source of Truth"?
  - Kenapa CTL Mapper "Project INTO, not Borrow FROM"?
  - Kenapa 3 Backend Target serentak?
  - Kenapa StrictAudioContext?
  - Kenapa Architecture Freeze + RFC Process?
- Model concurrency (Actor + Channel + Shard + Reactor)
- Model keselamatan (Gate + Door + Service + Taint FSM)
- Intermediate Representations (HIR → CapabilityGraph → CTL → WIT)
- Pengurusan projek (Architecture Freeze, RFC, Validator Tiering, Benchmark)
- Perbandingan dengan C/C++, Rust, Zig, Go, dan WASM-first languages

**Format:** MDBook — boleh dibina sebagai static site:
```bash
cd docs/white-paper
mdbook build
# Output di docs/white-paper/book/
```

---

## Wiki 2: Logicodex Functions And Guide

**Lokasi:** [`docs/guide/`](./guide/)

**Untuk siapa:** Pembangun yang ingin belajar **cara** menggunakan Logicodex — dari program pertama sehingga aplikasi produksi.

**Kandungan:**
- Pemasangan dan konfigurasi
- Program pertama (Hello World dalam 3 gaya)
- Sintaks dan token (Alias Melayu vs Canonical)
- Tipe data (asas, komposit, pointer, khas)
- Pengawalan aliran (if, match, for, while)
- Fungsi dan skop
- Sistem capability (Gate, Door, Service)
- Concurrency (Actor, Channel, Shard, Reactor)
- Grafik dengan Raylib (54 fungsi)
- Audio programming (22 fungsi + StrictAudioContext)
- Target kompilasi (Native, WASM, Freestanding)
- Proses build lengkap
- Pustaka standard
- Resepi dan contoh (HTTP Server, Grafik, Audio, Bare Metal)
- Penyelesaian masalah
- Jadual fungsi lengkap Raylib dan Audio

**Format:** MDBook — boleh dibina sebagai static site:
```bash
cd docs/guide
mdbook build
# Output di docs/guide/book/
```

---

## Perbezaan antara 2 Wiki

| Aspek | Experimental Philosophy | Functions And Guide |
|---|---|---|
| **Fokus** | "Kenapa" | "Cara" |
| **Pembaca** | Jurutera, pengkaji, kontributor | Pembangun, pelajar, pengguna |
| **Gaya** | Naratif, reflektif, akademik | Praktikal, tutorial, rujukan |
| **Kod** | Sedikit (ilustrasi konsep) | Banyak (contoh lengkap, resepi) |
| **Bahasa** | Melayu + Istilah Teknikal | Melayu dengan penjelasan |
| **Tujuan** | Memahami arkitektur | Menulis kod Logicodex |

---

## Membaca Wiki

### Cara 1: MDBook (Disyorkan)

```bash
# Pasang mdbook
cargo install mdbook

# Bina Experimental Philosophy Wiki
cd docs/white-paper
mdbook build
mdbook serve  # Buka http://localhost:3000

# Bina Functions Guide
cd docs/guide
mdbook build
mdbook serve  # Buka http://localhost:3000
```

### Cara 2: Markdown Langsung

Semua fail adalah Markdown biasa. Anda boleh membaca terus:
- `docs/white-paper/src/SUMMARY.md` — kandungan Experimental Philosophy
- `docs/guide/src/SUMMARY.md` — kandungan Functions Guide

### Cara 3: GitHub Wiki (Masa Depan)

Fail-fail ini direka untuk dipindahkan ke GitHub Wiki apabila projek mencapai tahap stabil (post-v2.0).

---

## Struktur Direktori

```
docs/
├── wiki/
│   └── README.md          # Dokumen ini
├── white-paper/           # Wiki 1: Experimental Compiler Philosophy
│   ├── book.toml          # Konfigurasi MDBook
│   └── src/               # Sumber Markdown
│       ├── SUMMARY.md     # Kandungan
│       ├── title.md       # Halaman judul
│       ├── ch01-kenapa.md # Ch 1: Kenapa Logicodex Wujud?
│       ├── ch02-falsafah.md # Ch 2: Falsafah Rekabentuk
│       ├── ch03-evolusi.md  # Ch 3: Evolusi Arkitektur v1.21-v1.45
│       ├── ch04-justifikasi.md # Ch 4: Justifikasi Keputusan
│       ├── ch05-concurrency.md # Ch 5: Model Concurrency
│       ├── ch06-security.md    # Ch 6: Model Keselamatan
│       ├── ch07-ir.md          # Ch 7: Intermediate Representations
│       ├── ch08-governance.md  # Ch 8: Pengurusan Projek
│       ├── ch09-perbandingan.md # Ch 9: Perbandingan Bahasa
│       ├── ch10-kesimpulan.md   # Ch 10: Kesimpulan dan Masa Depan
│       ├── appendix-glosari.md  # Appendix A: Glosari
│       ├── appendix-timeline.md # Appendix B: Timeline
│       └── appendix-rujukan.md  # Appendix C: Rujukan
├── guide/                 # Wiki 2: Functions And Guide
│   ├── book.toml          # Konfigurasi MDBook
│   └── src/               # Sumber Markdown
│       ├── SUMMARY.md     # Kandungan
│       ├── title.md       # Halaman judul
│       ├── ch01-installasi.md  # Ch 1: Pemasangan
│       ├── ch02-hello.md       # Ch 2: Program Pertama
│       ├── ch03-sintaks.md     # Ch 3: Sintaks
│       ├── ch04-tipe.md        # Ch 4: Tipe Data
│       ├── ch05-aliran.md      # Ch 5: Pengawalan Aliran
│       ├── ch06-fungsi.md      # Ch 6: Fungsi dan Skop
│       ├── ch07-gate.md        # Ch 7: Gate
│       ├── ch08-door.md        # Ch 8: Door
│       ├── ch09-security.md    # Ch 9: Keamanan
│       ├── ch10-actor.md       # Ch 10: Actor
│       ├── ch11-shard.md       # Ch 11: Shard
│       ├── ch12-reactor.md     # Ch 12: Reactor
│       ├── ch13-raylib.md      # Ch 13: Raylib Functions
│       ├── ch14-audio.md       # Ch 14: Audio Programming
│       ├── ch15-target.md      # Ch 15: Target Kompilasi
│       ├── ch16-build.md       # Ch 16: Proses Build
│       ├── ch17-stdlib.md      # Ch 17: Pustaka Standard
│       ├── ch18-resepi.md      # Ch 18: Resepi
│       ├── ch19-troubleshoot.md # Ch 19: Troubleshooting
│       ├── raylib-functions.md  # Jadual Fungsi Raylib
│       └── audio-functions.md   # Jadual Fungsi Audio
```

---

## Statistik

| Metric | Exp. Philosophy | Functions Guide | Total |
|---|---|---|---|
| **Chapters** | 10 | 19 | 29 |
| **Appendices** | 3 | 2 | 5 |
| **Estimated LOC** | ~4,500 | ~6,000 | ~10,500 |
| **Code examples** | ~30 | ~80 | ~110 |
| **Tables** | ~25 | ~35 | ~60 |

---

*Dokumen ini adalah sebahagian daripada Logicodex v1.45.0-alpha documentation suite.*
