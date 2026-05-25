# Chapter 5: Pengawalan Aliran

Semua konstruk pengawalan aliran dalam Logicodex — dalam tiga gaya sintaks.

---

## If / Else / Else If {#if}

### Sintaks

```logicodex
-- Gaya Melayu
JIKA syarat
    -- kod
LAIN_JIKA syarat_lain
    -- kod
LAIN
    -- kod
TAMAT_JIKA

-- Gaya English
IF condition
    -- code
ELSE_IF other_condition
    -- code
ELSE
    -- code
END_IF

-- Gaya Canonical
if condition {
    // code
} else if other_condition {
    // code
} else {
    // code
}
```

### Contoh

```logicodex
FUNGSI gred_markah(markah: I32) -> Text
MULA
    JIKA markah >= 90
        PULANG "A+"
    LAIN_JIKA markah >= 80
        PULANG "A"
    LAIN_JIKA markah >= 70
        PULANG "B"
    LAIN_JIKA markah >= 60
        PULANG "C"
    LAIN_JIKA markah >= 50
        PULANG "D"
    LAIN
        PULANG "F"
    TAMAT_JIKA
TAMAT
```

---

## Match (Pattern Matching) {#match}

### Sintaks

```logicodex
-- Gaya Melayu
PADAN nilai {
    corak1 => tindakan1,
    corak2 => tindakan2,
    _      => tindakan_default,
}

-- Gaya Canonical
match value {
    pattern1 => action1,
    pattern2 => action2,
    _        => default_action,
}
```

### Match pada Enum

```logicodex
ENUMERASI Warna {
    Merah,
    Hijau,
    Biru,
    RGB(U8, U8, U8),
}

FUNGSI deskripsi_warna(w: Warna) -> Text
MULA
    PADAN w {
        Warna::Merah          => PULANG "Warna panas",
        Warna::Hijau          => PULANG "Warna semula jadi",
        Warna::Biru           => PULANG "Warna sejuk",
        Warna::RGB(r, g, b)   => PULANG "RGB(" + r + "," + g + "," + b + ")",
    }
TAMAT
```

### Match pada Result

```logicodex
FUNGSI proses_fail(path: Text) -> I32
MULA
    BINA hasil SEBAGAI Result<Text, Text> = baca_fail(path)
    
    PADAN hasil {
        Result::Ok(data) => {
            PAPAR "Berjaya: " + data
            PULANG 0
        }
        Result::Err(err) => {
            PAPAR "Gagal: " + err
            PULANG 1
        }
    }
TAMAT
```

---

## For Loop dan While {#loop}

### For Loop

```logicodex
-- Gaya Melayu: UNTUK ... DARI ... HINGGA
UNTUK i DARI 0 HINGGA 10
    PAPAR "Iterasi: " + i
TAMAT_UNTUK

-- Gaya Canonical: for ... in
for i in 0..10 {
    print "Iteration: " + i;
}

-- For pada array
BINA nombor SEBAGAI [I32; 5] = [10, 20, 30, 40, 50]
UNTUK n DARI nombor
    PAPAR "Nombor: " + n
TAMAT_UNTUK
```

### While Loop

```logicodex
-- Gaya Melayu
SEMENTARA syarat
    -- kod
TAMAT_SEMENTARA

-- Gaya Canonical
while condition {
    // code
}

-- Contoh: Faktorial
FUNGSI faktorial(n: I32) -> I32
MULA
    BINA hasil SEBAGAI I32 = 1
    BINA i SEBAGAI I32 = 1
    
    SEMENTARA i <= n
        hasil = hasil * i
        i = i + 1
    TAMAT_SEMENTARA
    
    PULANG hasil
TAMAT
```

---

## Break, Continue, Return {#control}

### Break

```logicodex
-- Gaya Melayu: PATAH
UNTUK i DARI 0 HINGGA 100
    JIKA i > 10
        PATAH   -- keluar dari loop
    TAMAT_JIKA
TAMAT_UNTUK

-- Gaya Canonical: break
for i in 0..100 {
    if i > 10 {
        break;
    }
}
```

### Continue

```logicodex
-- Gaya Melayu: TERUS
UNTUK i DARI 0 HINGGA 10
    JIKA i % 2 == 0
        TERUS   -- langkau iterasi ini
    TAMAT_JIKA
    PAPAR i     -- hanya cetak nombor ganjil
TAMAT_UNTUK

-- Gaya Canonical: continue
for i in 0..10 {
    if i % 2 == 0 {
        continue;
    }
    print i;
}
```

### Return

```logicodex
-- Gaya Melayu: PULANG
FUNGSI max(a: I32, b: I32) -> I32
MULA
    JIKA a > b
        PULANG a
    TAMAT_JIKA
    PULANG b
TAMAT

-- Gaya Canonical: return
fn max(a: I32, b: I32) -> I32 {
    if a > b {
        return a;
    }
    return b;
}

-- Return awal (guard pattern)
FUNGSI bahagian_selamat(a: I32, b: I32) -> Result<I32, Text>
MULA
    JIKA b == 0
        PULANG Result::Err("Pembahagi sifar!")
    TAMAT_JIKA
    
    PULANG Result::Ok(a / b)
TAMAT
```

---

## Latihan

1. Tulis fungsi `fibbonaci(n: I32) -> I32` menggunakan `UNTUK`
2. Tulis fungsi `cari_terbesar(arr: &[I32]) -> Option<I32>` menggunakan `PADAN`
3. Tulis fungsi `jumlah_genap(n: I32) -> I32` yang menjumlahkan semua nombor genap dari 1 hingga n menggunakan `SEMENTARA` + `TERUS`
