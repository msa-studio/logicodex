# Chapter 6: Fungsi dan Skop

---

## Mendefinisikan Fungsi {#define}

### Sintaks Asas

```logicodex
FUNGSI nama_fungsi(param1: Tipe1, param2: Tipe2) -> ReturnType
MULA
    -- badan fungsi
    PULANG nilai
TAMAT
```

### Contoh Lengkap

```logicodex
-- Fungsi tanpa parameter dan tanpa return
FUNGSI say_hello() -> Void
MULA
    PAPAR "Hello!"
TAMAT

-- Fungsi dengan parameter
FUNGSI tambah(a: I32, b: I32) -> I32
MULA
    PULANG a + b
TAMAT

-- Fungsi dengan multiple parameter
FUNGSI luas_segiempat(panjang: F64, lebar: F64) -> F64
MULA
    PULANG panjang * lebar
TAMAT

-- Fungsi dengan parameter struct
FUNGSI jarak(p1: Point, p2: Point) -> F64
MULA
    BINA dx SEBAGAI F64 = p2.x - p1.x
    BINA dy SEBAGAI F64 = p2.y - p1.y
    PULANG punca_kuasa_dua(dx*dx + dy*dy)
TAMAT
```

---

## Parameter dan Return Type {#parameter}

### Parameter dengan Nilai Default

```logicodex
FUNGSI greet(nama: Text, greeting: Text = "Halo") -> Text
MULA
    PULANG greeting + ", " + nama + "!"
TAMAT

-- Penggunaan
greet("Ali")              -- "Halo, Ali!"
greet("Ali", "Selamat pagi")  -- "Selamat pagi, Ali!"
```

### Multiple Return Values (Tuple)

```logicodex
FUNGSI min_max(arr: &[I32]) -> (I32, I32)
MULA
    BINA min SEBAGAI I32 = arr[0]
    BINA max SEBAGAI I32 = arr[0]
    
    UNTUK n DARI arr
        JIKA n < min
            min = n
        TAMAT_JIKA
        JIKA n > max
            max = n
        TAMAT_JIKA
    TAMAT_UNTUK
    
    PULANG (min, max)
TAMAT

-- Penggunaan
BINA keputusan SEBAGAI (I32, I32) = min_max(&[3, 1, 4, 1, 5])
PAPAR "Min: " + keputusan.0 + ", Max: " + keputusan.1
```

### Generic Functions

```logicodex
-- Fungsi generic (tipe T)
FUNGSI pertukar<T>(a: &T, b: &T) -> Void
MULA
    BINA sementara SEBAGAI T = *a
    *a = *b
    *b = sementara
TAMAT

-- Penggunaan
BINA x SEBAGAI I32 = 5
BINA y SEBAGAI I32 = 10
pertukar(&x, &y)
-- Sekarang x = 10, y = 5
```

---

## Skop dan Lifetime Variable {#skop}

### Skop Blok

```logicodex
FUNGSI contoh_skop() -> Void
MULA
    BINA a SEBAGAI I32 = 10    -- skop: seluruh fungsi
    
    MULA                          -- blok dalaman
        BINA b SEBAGAI I32 = 20  -- skop: blok dalaman sahaja
        PAPAR a                   -- ✅ boleh akses a
        PAPAR b                   -- ✅ boleh akses b
    TAMAT
    
    PAPAR a                       -- ✅ boleh akses a
    -- PAPAR b                     -- ❌ RALAT: b tidak wujud di sini
TAMAT
```

### Shadowing

```logicodex
FUNGSI contoh_shadow() -> Void
MULA
    BINA x SEBAGAI I32 = 5
    PAPAR x              -- 5
    
    BINA x SEBAGAI I32 = x + 1   -- shadow: x baru = 6
    PAPAR x              -- 6
    
    MULA
        BINA x SEBAGAI I32 = 100  -- shadow dalam blok
        PAPAR x          -- 100
    TAMAT
    
    PAPAR x              -- 6 (x asal selepas blok)
TAMAT
```

### Ownership dan Move

```logicodex
FUNGSI contoh_ownership() -> Void
MULA
    BINA s1 SEBAGAI Text = "Hello"
    BINA s2 SEBAGAI Text = s1    -- ownership dipindahkan ke s2
    
    -- PAPAR s1                    -- ❌ RALAT: s1 sudah di-move
    PAPAR s2                        -- ✅ s2 memiliki ownership
    
    BINA s3 SEBAGAI Text = salin(s2)   -- salin (bukan move)
    PAPAR s2                        -- ✅ masih boleh guna
    PAPAR s3                        -- ✅ s3 adalah salinan
TAMAT
```

---

## Closure dan Callback {#closure}

### Closure (Fungsi Tanpa Nama)

```logicodex
-- Closure yang menangkap variable dari skop luar
FUNGSI buat_pendarab(faktor: I32) -> Fungsi(I32) -> I32
MULA
    PULANG |x: I32| => x * faktor   -- closure menangkap 'faktor'
TAMAT

FUNGSI utama() -> I32
MULA
    BINA ganda_dua SEBAGAI Fungsi(I32)->I32 = buat_pendarab(2)
    BINA ganda_tiga SEBAGAI Fungsi(I32)->I32 = buat_pendarab(3)
    
    PAPAR ganda_dua(5)    -- 10
    PAPAR ganda_tiga(5)   -- 15
    PULANG 0
TAMAT
```

### Callback untuk Audio

```logicodex
-- Callback audio (mesti patuh StrictAudioContext)
FUNGSI audio_callback(buffer: &mut [F32], frames: U32) -> Void
MULA
    UNTUK i DARI 0 HINGGA frames
        -- Gelombang sinus 440Hz
        buffer[i] = sin(2.0 * PI * 440.0 * i SEBAGAI F64 / 44100.0) SEBAGAI F32
    TAMAT_UNTUK
TAMAT

-- Daftarkan callback
SetAudioStreamCallback(stream, audio_callback)
```

---

## FFI: Memanggil Fungsi C

### Deklarasi FFI

```logicodex
-- Deklarasikan fungsi C
FFI c "m" {
    FN cos(x: F64) -> F64
    FN sin(x: F64) -> F64
    FN sqrt(x: F64) -> F64
}

FUNGSI utama() -> I32
MULA
    BINA sudut SEBAGAI F64 = 3.14159 / 4.0   -- 45 darjah
    PAPAR "cos(45°) = " + cos(sudut)
    PAPAR "sin(45°) = " + sin(sudut)
    PULANG 0
TAMAT
```

### Memanggil Raylib

```logicodex
-- Raylib functions sudah didaftarkan dalam CallableRegistry
-- Gunakan sahaja seperti fungsi Logicodex

FUNGSI utama() -> I32
MULA
    InitWindow(800, 600, "Contoh Raylib")
    SetTargetFPS(60)
    
    SEMENTARA !WindowShouldClose()
    MULA
        BeginDrawing()
        ClearBackground(RAYWHITE)
        DrawText("Halo Logicodex!", 190, 200, 20, DARKGRAY)
        EndDrawing()
    TAMAT_SEMENTARA
    
    CloseWindow()
    PULANG 0
TAMAT
```
