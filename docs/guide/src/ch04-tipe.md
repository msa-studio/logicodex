# Chapter 4: Tipe Data

Bab ini menerangkan semua tipe data dalam Logicodex — dari integer sehingga channel.

---

## Tipe Asas {#asas}

### Integer

| Tipe | Saiz (bit) | Julat | Suffix |
|---|---|---|---|
| `I8` | 8 | -128 s/d 127 | `42i8` |
| `I16` | 16 | -32,768 s/d 32,767 | `42i16` |
| `I32` | 32 | -2,147,483,648 s/d 2,147,483,647 | `42i32` atau `42` |
| `I64` | 64 | -9×10¹⁸ s/d 9×10¹⁸ | `42i64` |
| `ISize` | 64 (x86_64) | Platform-dependent | — |
| `U8` | 8 | 0 s/d 255 | `42u8` |
| `U16` | 16 | 0 s/d 65,535 | `42u16` |
| `U32` | 32 | 0 s/d 4,294,967,295 | `42u32` |
| `U64` | 64 | 0 s/d 1.8×10¹⁹ | `42u64` |
| `USize` | 64 (x86_64) | Platform-dependent | — |

### Float (Pecahan)

| Tipe | Saiz (bit) | Ketepatan | Suffix |
|---|---|---|---|
| `F32` | 32 (IEEE 754 single) | ~7 digit | `3.14f32` |
| `F64` | 64 (IEEE 754 double) | ~15 digit | `3.14f64` atau `3.14` |

### Boolean

| Tipe | Nilai | Alias |
|---|---|---|
| `Bool` | `true` / `false` | `BENAR` / `PALSU` |

### Text (String)

| Tipe | Saiz | Keterangan |
|---|---|---|
| `Text` | Dinamik | Owned string (heap-allocated) |
| `&Text` | — | Borrowed string reference |

---

## Tipe Komposit {#komposit}

### Struct

```logicodex
-- Definisi struct
STRUKTUR Point {
    x: F64,
    y: F64,
}

STRUKTUR Rectangle {
    posisi: Point,
    lebar: F64,
    tinggi: F64,
}

-- Penggunaan
FUNGSI utama() -> I32
MULA
    BINA titik SEBAGAI Point = Point { x: 10.0, y: 20.0 }
    BINA kotak SEBAGAI Rectangle = Rectangle {
        posisi: titik,
        lebar: 100.0,
        tinggi: 50.0,
    }
    
    PAPAR "Titik: (" + titik.x + ", " + titik.y + ")"
    PULANG 0
TAMAT
```

### Enum

```logicodex
ENUMERASI Status {
    Aktif,
    TidakAktif,
    Digantung,
    Dipadam,
}

ENUMERASI ResultCustom {
    Ok(I32),
    Err(Text),
}

-- Penggunaan
FUNGSI dapat_status(s: Status) -> Text
MULA
    PADAN s {
        Status::Aktif      => PULANG "Aktif",
        Status::TidakAktif => PULANG "Tidak aktif",
        Status::Digantung  => PULANG "Digantung",
        Status::Dipadam    => PULANG "Dipadam",
    }
TAMAT
```

### Array

```logicodex
-- Array statik (saiz tetap pada masa kompil)
BINA nombor SEBAGAI [I32; 5] = [1, 2, 3, 4, 5]
BINA kosong SEBAGAI [I32; 3] = [0; 3]  -- [0, 0, 0]

-- Akses
PAPAR nombor[0]    -- 1
PAPAR nombor[4]    -- 5

-- Slice (rujuk sebahagian array)
BINA sebahagian SEBAGAI &[I32] = &nombor[1..4]  -- [2, 3, 4]
```

---

## Tipe Pointer {#pointer}

### Referensi (`&T`)

```logicodex
-- Referensi (pinjam, tidak memindahkan ownership)
FUNGSI cetak_panjang(s: &Text) -> Void
MULA
    PAPAR "Panjang: " + s.panjang()
TAMAT

FUNGSI utama() -> I32
MULA
    BINA nama SEBAGAI Text = "Logicodex"
    cetak_panjang(&nama)   -- pinjam, ownership kekal
    PAPAR nama              -- ✅ masih boleh guna
    PULANG 0
TAMAT
```

### Pointer Mentah (`PTR<T>`)

```logicodex
-- Pointer mentah (freestanding / hardware access sahaja)
GUNA_JENIS PTR<U32>
GUNA_JENIS U32

-- Hardware zone (hanya dalam target freestanding)
TANDA KAWASAN_PERKAKAS GPIO_BASE SEBAGAI PTR<U32> = ALAMAT 0x40020000

FUNGSI baca_gpio() -> U32
MULA
    PULANG BACA_VOLATIL(GPIO_BASE)
TAMAT
```

### Perbezaan `&T` vs `PTR<T>`

| Aspek | `&T` (Reference) | `PTR<T>` (Raw Pointer) |
|---|---|---|
| Keselamatan | Selamat (compiler-verified) | Tidak selamat (programmer bertanggungjawab) |
| Dereference | Otomatik | Melalui `BACA_VOLATIL` / `TULIS_VOLATIL` |
| Target | Semua | Freestanding / hardware sahaja |
| Null | Tidak boleh | Boleh |
| Lifetime | Terbatas | Tidak terhad |

---

## Tipe Khas {#khas}

### `Result<T, E>`

```logicodex
GUNA_JENIS Result

-- Fungsi yang mungkin gagal
FUNGSI baca_fail(path: Text) -> Result<Text, Text>
MULA
    -- Jika berjaya
    PULANG Result::Ok(isi_fail)
    
    -- Jika gagal
    -- PULANG Result::Err("Fail tidak wujud")
TAMAT

-- Penggunaan
FUNGSI utama() -> I32
MULA
    BINA hasil SEBAGAI Result<Text, Text> = baca_fail("data.txt")
    
    PADAN hasil {
        Result::Ok(data)  => PAPAR "Data: " + data,
        Result::Err(err)  => PAPAR "Ralat: " + err,
    }
    
    PULANG 0
TAMAT
```

### `Option<T>`

```logicodex
GUNA_JENIS Option

-- Fungsi yang mungkin tiada nilai
FUNGSI cari_indeks(arr: &[I32], target: I32) -> Option<I32>
MULA
    UNTUK i DARI 0 HINGGA arr.panjang()
        JIKA arr[i] == target
            PULANG Option::Some(i SEBAGAI I32)
        TAMAT_JIKA
    TAMAT_UNTUK
    
    PULANG Option::None
TAMAT

-- Penggunaan
FUNGSI utama() -> I32
MULA
    BINA nombor SEBAGAI [I32; 5] = [10, 20, 30, 40, 50]
    BINA indeks SEBAGAI Option<I32> = cari_indeks(&nombor, 30)
    
    PADAN indeks {
        Option::Some(i) => PAPAR "Ditemui pada indeks: " + i,
        Option::None    => PAPAR "Tidak ditemui",
    }
    
    PULANG 0
TAMAT
```

### `Channel<T>`

```logicodex
GUNA_JENIS Channel

-- Membuat channel
BINA tx_rx SEBAGAI Channel<I32> = Channel::baru(100)  -- kapasiti 100

-- Sender
FUNGSI hantar_data(ch: Channel<I32>) -> Void
MULA
    ch.hantar(42)
TAMAT

-- Receiver
FUNGSI terima_data(ch: Channel<I32>) -> I32
MULA
    PULANG ch.terima()
TAMAT
```

---

## Ringkasan Tipe

```text
┌──────────────────────────────────────────────────────┐
│                    TIPE LOGICODEX                     │
├──────────────────────────────────────────────────────┤
│  ASAS                                                 │
│    I8, I16, I32, I64, ISize  (integer bertanda)      │
│    U8, U16, U32, U64, USize  (integer tak bertanda)  │
│    F32, F64                   (pecahan)              │
│    Bool                       (boolean)              │
│    Text                       (string)               │
├──────────────────────────────────────────────────────┤
│  KOMPOSIT                                             │
│    Struct, Enum, Array, Slice                        │
├──────────────────────────────────────────────────────┤
│  POINTER                                              │
│    &T (reference), PTR<T> (raw pointer)              │
├──────────────────────────────────────────────────────┤
│  KHAS                                                 │
│    Result<T,E>, Option<T>, Channel<T>                │
└──────────────────────────────────────────────────────┘
```
