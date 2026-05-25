# Chapter 2: Program Pertama Anda

Bab ini membimbing anda menulis, mengkompil, dan menjalankan program Logicodex pertama — dalam tiga gaya sintaks yang berbeza.

---

## Hello World dalam 3 Gaya {#hello}

### Gaya 1: Alias Melayu (Pemula)

```logicodex
PROGRAM hello_melayu

FUNGSI utama() -> I32
MULA
    PAPAR "Selamat datang ke Logicodex!"
    PULANG 0
TAMAT

TAMAT PROGRAM
```

Simpan sebagai `hello_melayu.ldx`.

### Gaya 2: Alias Inggeris (Sederhana)

```logicodex
PROGRAM hello_english

FUNCTION main() -> I32
BEGIN
    DISPLAY "Welcome to Logicodex!"
    RETURN 0
END

END PROGRAM
```

Simpan sebagai `hello_english.ldx`.

### Gaya 3: Canonical Shorthand (Pakar)

```logicodex
program hello_pro {
    fn main() -> I32 {
        print "Welcome to Logicodex!";
        return 0;
    }
}
```

Simpan sebagai `hello_pro.ldx`.

> **Perhatian:** Ketiga-tiga program ini mengkompil ke **binari yang identik**. Compiler tidak peduli sama ada anda menulis `MULA`, `BEGIN`, atau `{` — semuanya dinormalisasi ke token kanonikal yang sama.

---

## Kompilasi dan Jalankan {#kompilasi}

### Kompilasi

```bash
# Gaya Melayu
logicodex hello_melayu.ldx -o /tmp/hello_m
/tmp/hello_m
# Output: Selamat datang ke Logicodex!

# Gaya Inggeris
logicodex hello_english.ldx -o /tmp/hello_e
/tmp/hello_e
# Output: Welcome to Logicodex!

# Gaya Pakar
logicodex hello_pro.ldx -o /tmp/hello_p
/tmp/hello_p
# Output: Welcome to Logicodex!
```

### Menyemak Ralat Semantik

```bash
# Program tanpa fungsi utama — ralat
logicodex --check bad_program.ldx
```

Ralat yang dihasilkan:
```
error: Program tiada fungsi 'utama' atau 'main'
  --> bad_program.ldx:1:1
   |
 1 | PROGRAM tanpa_main
   | ^^^^^^^^^^^^^^^^^^ fungsi utama diperlukan
   |
   = bantuan: Tambah 'FUNGSI utama() -> I32 { ... }' dalam program anda
```

---

## Memahami Ralat Compiler {#ralat}

Logicodex menghasilkan ralat bilingual (Melayu + Inggeris) untuk semua semakan semantik.

### Jenis Ralat

| Kod | Bahasa Melayu | English | Apa Maksudnya |
|---|---|---|---|
| `E001` | `Simbol tidak diketahui` | `Unknown symbol` | Variable/fungsi tidak diisytiharkan |
| `E002` | `Tidak sepadan jenis` | `Type mismatch` | Menggunakan tipe A di tempat tipe B |
| `E003` | `Pembahagian dengan sifar` | `Division by zero` | Bahagian dengan 0 pada masa kompil |
| `E004` | `Gate tidak diizinkan` | `Gate not permitted` | Menggunakan capability tanpa keizinan |
| `E005` | `UseAfterSend` | `Use after send` | Menggunakan data selepas hantar ke channel |
| `E006` | `UseAfterMove` | `Use after move` | Menggunakan ownership selepas dipindahkan |
| `E007` | `Pelanggaran Audio` | `Audio violation` | Callback audio melanggar StrictAudioContext |
| `E008` | `Hardware gate dalam WASM` | `Hardware gate in WASM` | Gate hardware tidak dibenarkan dalam target WASM |
| `E009` | `Shard kosong` | `Empty shard` | Shard tiada actor yang diumpukkan |
| `E010` | `Privilege escalation` | `Privilege escalation` | Gate baharu tidak dalam baseline `.cap` |

### Contoh Ralat Lengkap

```logicodex
PROGRAM contoh_ralat

FUNGSI utama() -> I32
MULA
    BINA pesan SEBAGAI Text = "Hello"
    PAPAR pesan
    PAPAR pesan   -- ❌ ERROR: pesan mungkin sudah di-move (bergantung pada konteks)
    PULANG 0
TAMAT

TAMAT PROGRAM
```

Output compiler:
```
error[E006]: UseAfterMove
  --> contoh_ralat.ldx:7:5
   |
 6 |     PAPAR pesan
   |           ----- digunakan di sini
 7 |     PAPAR pesan
   |     ^^^^^^^^^^^ digunakan lagi selepas pemindahan
   |
   = bantuan: 'pesan' telah dipindahkan (moved) selepas penggunaan pertama.
              Gunakan 'salin' (clone) jika anda perlukan dua salinan.
   = help: 'pesan' has been moved after first use.
           Use 'clone' if you need two copies.
```

### Tip: Membaca Ralat

1. **Baca baris pertama** — kod ralat dan tajuk
2. **Baca lokasi** (`-->`) — fail dan nombor baris
3. **Baca mesej** (`= bantuan:`) — cadangan pembetulan
4. **Betulkan dan kompil semula**

---

## Latihan

1. Tulis program yang mencetak nama anda menggunakan tiga gaya sintaks
2. Cuba buat ralat `E001` (simbol tidak diketahui) dan baca mesej ralat
3. Cuba buat ralat `E003` (pembahagian sifar) — catat sama ada ia dikesan pada masa kompil atau runtime
