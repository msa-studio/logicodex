> ⚠️ **NOT UPDATED — will revisit.** This document predates the current syntax/architecture and may contain stale information. Authoritative current references: `README.md`, `examples/`, and `docs/architecture/`. Tracked under `docs/DOCUMENTATION_POLICY.md`.

# Logicodex — Getting Started & Complete Syntax Reference

> **Version:** v1.45.0-alpha  
> **Engines:** v1.21 (default) | v1.30 (`--pipeline v1.30`)  
> **Syntax:** Bilingual — Malay aliases ↔ Expert canonical  
> **Last Updated:** 2026-05-29

---

## Table of Contents

1. [Quick Hello World](#1-quick-hello-world)
2. [Variables & Types](#2-variables--types)
3. [Operators](#3-operators)
4. [Control Flow](#4-control-flow)
5. [Functions](#5-functions)
6. [Comments](#6-comments)
7. [Strings](#7-STRings) — `--pipeline v1.30`
8. [Structs](#8-STRucts) — `--pipeline v1.30`
9. [Enums & Match](#9-enums--MATCH) — `--pipeline v1.30`
10. [Arrays](#10-arrays) — `--pipeline v1.30`
11. [Pointers](#11-pointers) — `--pipeline v1.30`
12. [Visibility](#12-visibility) — `--pipeline v1.30`
13. [Attributes](#13-attributes) — `--pipeline v1.30`
14. [Actor Runtime](#14-actor-runtime) — `--pipeline v1.30`
15. [Backpressure & Scheduler](#15-backpressure--scheduler) — `--pipeline v1.30`
16. [Keyword Reference](#16-keyword-reference)

---

## Legend

| Marker | Meaning |
|--------|---------|
| ✅ | Available in v1.21 (default) |
| 🔷 | Available in `--pipeline v1.30` |
| 📝 | Both Malay + Expert syntax shown |

---

## 1. Quick Hello World

```
// Malay: MULA = start block, PAPAR = print, TAMAT = end block
MULA
    PAPAR 42
TAMAT

// Expert: { } delimit block, PRINT statement, semicolon
{
    PRINT 42;
}
```

**Output:** `42`

---

## 2. Variables & Types

### Variable Declaration

```
// Malay: BINA = bind/create variable
BINA nama = "Ahmad"       // v1.30+ — strings supported with type inference
BINA umur = 25            // I32 (integer, inferred)
BINA gaji = 3500.50       // F64 (float, inferred)
BINA aktIF = benar        // Bool (boolean, inferred)

// Expert: LET = bind variable
LET nama = "Ahmad";       // v1.30 only
LET umur = 25;            // v1.21 — integer literal → I32
LET gaji = 3500.50;       // v1.21 — float literal → F64
LET aktIF = true;         // v1.21 — boolean literal → Bool
```

### Explicit Type Annotation

```
// Malay: type after name with : separator
BINA bilangan : I32 = 100
BINA besar : I64 = 9999999999
BINA suhu : F32 = 36.6
BINA panjang : F64 = 3.14159265359
BINA hidup : Bool = benar
BINA nama : TEKS = "Ahmad"      // String type annotation (v1.30+)

// Expert: same syntax, semicolon terminated
LET bilangan: I32 = 100;
LET besar: I64 = 9999999999;
LET suhu: F32 = 36.6;
LET panjang: F64 = 3.14159265359;
LET hidup: Bool = true;
LET nama: STR = "Ahmad";        // String type annotation (v1.30+)
```

### Mutable Binding (`MUT` / `MUTASI`)

> **Status:** 📝 The `MUT` / `MUTASI` keyword is recognized by the lexer but has **no semantic effect**. Variables are mutable by default — you can reassign any variable without marking it `MUT`.

```
// Malay: MUTASI has no effect (variables are mutable by default)
MUTASI BINA x = 10      // 'MUTASI' is accepted but ignored
x = 20                  // OK: reassignment works without mut
PAPAR x                 // 20

// Expert: MUT is accepted but ignored
LET MUT x = 10;         // 'mut' is accepted but ignored
x = 20;                 // OK: reassignment works
PRINT x;                // 20
```

### Available Types

| Type | Malay Literal | Expert Literal | Size | v1.21 | v1.30 |
|------|--------------|----------------|------|-------|-------|
| `I32` | integer | `42` | 32-bit signed | ✅ | ✅ |
| `I64` | large integer | `999999` | 64-bit signed | ✅ | ✅ |
| `F32` | small float | `3.14` | 32-bit float | ✅ | ✅ |
| `F64` | large float | `3.14159` | 64-bit float | ✅ | ✅ |
| `Bool` | `benar` / `palsu` | `TRUE` / `FALSE` | 1-bit | ✅ | ✅ |
| `String` | `"hello"` | `"hello"` | heap-allocated | 🔷 | 🔷 |
| `STR` / `TEKS` | `"hello"` | `"hello"` | heap-allocated | 🔷 | 🔷 |

---

## 3. Operators

### Arithmetic

```
// Malay: BINA + operasi
BINA a = 10
BINA b = 3
BINA tambah = a + 3           // 13
BINA tolak = a - 3            // 7
BINA darab = a * 3            // 30
BINA bahagi = a / 3           // 3 (integer division)
BINA baki = a % 3             // 1 (modulo)

// Expert: identical, semicolons
LET a = 10;
LET b = 3;
LET tambah = a + 3;           // 13
LET tolak = a - 3;            // 7
LET darab = a * 3;            // 30
LET bahagi = a / 3;           // 3
LET baki = a % 3;             // 1
```

### Comparison

```
// Malay: sama_dengan = equals, tak_sama = not equals
BINA x = 5
BINA y = 10
BINA sama = x == y            // false
BINA tak_sama = x != y        // true
BINA kecil = x < y            // true
BINA besar = x > y            // false
BINA kecil_sama = x <= y      // true
BINA besar_sama = x >= y      // false

// Expert: identical
LET x = 5;
LET y = 10;
LET sama = x == y;            // false
LET tak_sama = x != y;        // true
LET kecil = x < y;            // true
LET besar = x > y;            // false
LET kecil_sama = x <= y;      // true
LET besar_sama = x >= y;      // false
```

### Boolean

```
// Malay: DAN = AND, ATAU = OR, TIDAK = NOT
BINA p = benar
BINA q = palsu
BINA dan = p DAN q            // false
BINA atau = p ATAU q           // true

// Expert: &&, ||, !
LET p = true;
LET q = false;
LET dan = p && q;             // false
LET atau = p || q;            // true
```

### Unary

> **Status:** ✅ Now supported after parser fix.

```
// Malay: negation and logical not
BINA x = 5
BINA negatIF = -x               // -5
BINA tak_benar = !benar         // FALSE (logical not)
BINA tak_palsu = !palsu         // TRUE  (logical not)

// Expert: same syntax
LET x = 5;
LET negatIF = -x;               // -5
LET tak_benar = !true;          // false
LET tak_palsu = !false;         // true
```

### Bitwise

```
// Malay: DAN, ATAU, XATAU, GESERKIRI, GESERKANAN, JANGKAU
BINA m = 0b1100               // 12
BINA n = 0b1010               // 10
BINA bit_dan = m DAN n        // 0b1000 = 8
BINA bit_atau = m ATAU n      // 0b1110 = 14
BINA bit_xatau = m XATAU n    // 0b0110 = 6
BINA shift_kiri = m GESERKIRI 2    // 0b110000 = 48
BINA shift_kanan = m GESERKANAN 2  // 0b0011 = 3
BINA bit_jangkau = JANGKAU m       // 0b0011 = 3 (NOT)

// Expert: &, |, ^, <<, >>, ~
LET m = 0b1100;               // 12
LET n = 0b1010;               // 10
LET bit_and = m & n;          // 0b1000 = 8
LET bit_or = m | n;           // 0b1110 = 14
LET bit_xor = m ^ n;          // 0b0110 = 6
LET shift_left = m << 2;      // 0b110000 = 48
LET shift_right = m >> 2;     // 0b0011 = 3
LET bit_not = ~m;             // 0b0011 = 3
```

---

## 4. Control Flow

### If / Else If / Else

```
// Malay: JIKA = if, MAKA = then, SEBALIKNYA JIKA = ELSE if, MELAINKAN = else
JIKA umur >= 18 MAKA
MULA
    PAPAR 1
TAMAT
SEBALIKNYA JIKA umur >= 13 MAKA
MULA
    PAPAR 2
TAMAT
MELAINKAN
MULA
    PAPAR 3
TAMAT

// Expert: IF / ELSE IF / else
IF umur >= 18 {
    PRINT 1;
} ELSE IF umur >= 13 {
    PRINT 2;
} ELSE {
    PRINT 3;
}
```

### While Loop

```
// Malay: SELAGI = while, DAN benar = condition suffix (optional in expert)
BINA i = 0
SELAGI i < 5 DAN benar MAKA
MULA
    PAPAR i
    i = i + 1
TAMAT

// Expert: WHILE { }
LET i = 0;
WHILE i < 5 {
    PRINT i;
    i = i + 1;
}
```

### Infinite Loop

```
// Malay: ULANG = LOOP forever
BINA counter = 0
ULANG
MULA
    PAPAR counter
    counter = counter + 1
    JIKA counter >= 10 MAKA
    MULA
        HENTI
    TAMAT
TAMAT

// Expert: LOOP { }
LET counter = 0;
LOOP {
    PRINT counter;
    counter = counter + 1;
    IF counter >= 10 {
        break;
    }
}
```

### Break & Continue

```
// Malay: HENTI = break, LANGKAU = continue
SELAGI i < 100 DAN benar MAKA
MULA
    i = i + 1
    JIKA i % 2 == 0 MAKA
    MULA
        LANGKAU
    TAMAT
    PAPAR i
TAMAT

// Expert: break; / continue;
WHILE i < 100 {
    i = i + 1;
    IF i % 2 == 0 {
        continue;
    }
    PRINT i;
}
```

### For Loop

> **Status:** ❌ **Not supported.** There is no `for` keyword in the parser. Use `WHILE` with a counter instead.

```
// Instead of for (use WHILE with counter):
BINA i = 0
SELAGI i < 10 DAN benar MAKA
MULA
    PAPAR i
    i = i + 1
TAMAT

// Expert equivalent:
LET i = 0;
WHILE i < 10 {
    PRINT i;
    i = i + 1;
}
```

---

## 5. Functions

### Function Declaration

```
// Malay: FUNGSI = function, -> = RETURN type, PULANG = return
FUNGSI tambah(a: I32, b: I32) -> I32
MULA
    PULANG a + b
TAMAT

// Expert: fn, return
FN tambah(a: I32, b: I32) -> I32 {
    RETURN a + b;
}
```

### Void Function (no return)

```
// Malay: PAPAR in function body
FUNGSI ucap_hello() -> I32
MULA
    PAPAR 1
    PULANG 0
TAMAT

// Expert: returns I32 (must have RETURN type)
FN ucap_hello() -> I32 {
    PRINT 1;
    RETURN 0;
}
```

### Function Call

```
// Malay: call like expression
BINA hasil = tambah(10, 20)
PAPAR hasil

// Expert: identical
LET hasil = tambah(10, 20);
PRINT hasil;
```

---

## 6. Comments

```
// Malay: comments use // same as expert
// Ini adalah komen dalam Bahasa Melayu

// Expert:
// This is a comment in English

// Both support only single-line comments (//)
// No block comments (/* */) in v1.21
// Block comments available in v1.30 🔷
```

> **Case Sensitivity Note:** Keywords are **case-sensitive**. `BINA` works, but `bina` does not. `benar` (lowercase) is a boolean literal, but `BENAR` (uppercase) is also accepted as an alias. In general, write keywords exactly as documented — mixed case like `BiNa` or `bReAk` will **not** be recognized.
>
> | Form | Status | Example |
> |------|--------|---------|
> | `BINA` (uppercase) | ✅ Works | Malay keyword |
> | `bina` (lowercase) | ❌ Fails | Not recognized |
> | `LET` (lowercase) | ✅ Works | Expert keyword |
> | `TRUE` (lowercase) | ✅ Works | Boolean literal |
> | `benar` (lowercase) | ✅ Works | Malay boolean literal |
> | `bReAk` (mixed) | ❌ Fails | Not recognized |

---

## 7. Strings 🔧 `--pipeline v1.30`

```
// Malay: String literal with ""
BINA nama = "Logicodex"
BINA salam = "Selamat datang!"
PAPAR nama.length           // 9

// Expert: same syntax
LET nama = "Logicodex";
LET salam = "Selamat datang!";
PRINT nama.length;          // 9

// String concatenation
BINA penuh = "Hello" + " " + "World"    // "Hello World"

// Expert:
LET penuh = "Hello" + " " + "World";   // "Hello World"
```

---

## 8. Structs 🔧 `--pipeline v1.30`

### Struct Definition

```
// Malay: struct = struktur
struktur Point
MULA
    x: F64,
    y: F64
TAMAT

struktur Person
MULA
    nama: String,
    umur: I32
TAMAT

// Expert: struct { }
struct Point {
    x: F64,
    y: F64
}

struct Person {
    nama: String,
    umur: I32
}
```

### Struct Literal

```
// Malay: struct_name { field: value }
BINA titik = Point { x: 10.0, y: 20.0 }
BINA orang = Person { nama: "Ahmad", umur: 25 }

// Expert: identical
LET titik = Point { x: 10.0, y: 20.0 };
LET orang = Person { nama: "Ahmad", umur: 25 };
```

### Field Access

```
// Malay: dot notation
PAPAR titik.x       // 10.0
PAPAR titik.y       // 20.0
PAPAR orang.nama    // "Ahmad"

// Expert: identical
PRINT titik.x;      // 10.0
PRINT titik.y;      // 20.0
PRINT orang.nama;   // "Ahmad"
```

### Nested Structs

```
// Malay: struct within struct
struktur Circle
MULA
    pusat: Point,
    radius: F64
TAMAT

BINA bulat = Circle {
    pusat: Point { x: 0.0, y: 0.0 },
    radius: 5.0
}
PAPAR bulat.pusat.x     // 0.0

// Expert: identical
struct Circle {
    pusat: Point,
    radius: F64
}

LET bulat = Circle {
    pusat: Point { x: 0.0, y: 0.0 },
    radius: 5.0
};
PRINT bulat.pusat.x;   // 0.0
```

---

## 9. Enums & Match 🔧 `--pipeline v1.30`

### Enum Definition

```
// Malay: enum = enumerasi
enumerasi Status
MULA
    Aktif,
    TidakAktif,
    Diseret
TAMAT

// With values (repr)
enumerasi Priority
sebagai I32
MULA
    Rendah = 1,
    Sederhana = 5,
    Tinggi = 10
TAMAT

// Expert: enum { }
enum Status {
    Aktif,
    TidakAktif,
    Diseret
}

enum Priority as I32 {
    Rendah = 1,
    Sederhana = 5,
    Tinggi = 10
}
```

### Enum with Payload

```
// Malay: variants can carry data
enumerasi Mesej
MULA
    Henti,
    Gerak { x: I32, y: I32 },
    Tulis(String)
TAMAT

// Expert: identical
enum Mesej {
    Henti,
    Gerak { x: I32, y: I32 },
    Tulis(String)
}
```

### Match Expression

```
// Malay: cocok = match
BINA status = Status::Aktif
cocok status
MULA
    Status::AktIF -> PAPAR 1,
    Status::TidakAktIF -> PAPAR 2,
    Status::Diseret -> PAPAR 3
TAMAT

// Expert: MATCH { }
LET status = Status::Aktif;
MATCH status {
    Status::AktIF -> PRINT 1,
    Status::TidakAktIF -> PRINT 2,
    Status::Diseret -> PRINT 3
}
```

### Match with Payload

```
// Malay: destructure in match
BINA m = Mesej::Gerak { x: 10, y: 20 }
cocok m
MULA
    Mesej::Henti -> PAPAR 0,
    Mesej::Gerak { x, y } -> PAPAR x + y,
    Mesej::Tulis(s) -> PAPAR s.length
TAMAT

// Expert: identical
LET m = Mesej::Gerak { x: 10, y: 20 };
MATCH m {
    Mesej::Henti -> PRINT 0,
    Mesej::Gerak { x, y } -> PRINT x + y,
    Mesej::Tulis(s) -> PRINT s.length
}
```

### Match with Literal Patterns

> **Status:** ✅ Integer and string literal patterns are now supported in addition to `OK`/`ERR`/`_`.

```
// Malay: MATCH on literal values
cocok nilai
MULA
    0 -> PAPAR "sifar",
    1 -> PAPAR "satu",
    2 -> PAPAR "dua",
    "hello" -> PAPAR "greeting",
    _ -> PAPAR "lain"
TAMAT

// Expert: MATCH on literal values
MATCH nilai {
    0 -> PRINT "zero",
    1 -> PRINT "one",
    2 -> PRINT "two",
    "hello" -> PRINT "greeting",
    _ -> PRINT "other"
}
```

---

## 10. Arrays 🔧 `--pipeline v1.30`

### Array Declaration

```
// Malay: tatasusunan = array, fixed size
BINA nombor = [I32; 5]              // [0, 0, 0, 0, 0]
BINA nama = ["Ahmad", "Bakar", "Chin"]    // [String; 3]

// Expert: identical
LET nombor = [I32; 5];              // [0, 0, 0, 0, 0]
LET nama = ["Ahmad", "Bakar", "Chin"];  // [String; 3]
```

### Array Indexing

```
// Malay: [index] access
PAPAR nama[0]       // "Ahmad"
PAPAR nama[1]       // "Bakar"

// Assignment
nama[2] = "David"

// Expert: identical
PRINT nama[0];      // "Ahmad"
PRINT nama[1];      // "Bakar"
nama[2] = "David";
```

### Array Length

```
// Malay: .panjang
PAPAR nombor.panjang    // 5

// Expert: .length
PRINT nombor.length;    // 5
```

---

## 11. Pointers 🔧 `--pipeline v1.30`

```
// Malay: petunjuk = pointer, @ = address-of, ^ = dereference
BINA a = 42
BINA ptr_a = @a              // pointer to a
PAPAR ^ptr_a                 // 42 (dereference)

// Expert: & = address-of, * = dereference
LET a = 42;
LET ptr_a = &a;              // pointer to a
PRINT *ptr_a;                // 42

// Mutable pointer
BINA b = 10
BINA ptr_b = @b
^ptr_b = 20                  // dereference and assign
PAPAR b                      // 20

// Expert:
LET b = 10;
LET ptr_b = &b;
*ptr_b = 20;                // dereference and assign
PRINT b;                     // 20
```

---

## 12. Visibility 🔧 `--pipeline v1.30`

```
// Malay: terbuka = public, tertutup = private (default)
terbuka struktur Config
MULA
    tertutup api_key: String,
    terbuka nama: String
TAMAT

terbuka FUNGSI get_nama(c: Config) -> String
MULA
    PULANG c.nama
TAMAT

// Expert: pub / private (default)
pub struct Config {
    private api_key: String,
    pub nama: String
}

pub FN get_nama(c: Config) -> String {
    RETURN c.nama;
}
```

---

## 13. Attributes 🔧 `--pipeline v1.30`

```
// Malay: #[atribut] before declarations
#[debug]
struktur Point
MULA
    x: F64,
    y: F64
TAMAT

#[susun(rapat)]           // packed layout
#[terlihat(luar)]         // FFI visible
struktur RawHeader
MULA
    jenis: I32,
    saiz: I32
TAMAT

#[bahaya]                 // unsafe marker
FUNGSI raw_alloc(saiz: I64) -> Pointer
MULA
    // ... FFI call
TAMAT

// Expert: #[attribute]
#[debug]
struct Point {
    x: F64,
    y: F64
}

#[packed]
#[ffi_visible]
struct RawHeader {
    jenis: I32,
    saiz: I32
}

#[unsafe]
FN raw_alloc(saiz: I64) -> Pointer {
    // ... FFI call
}
```

---

## 14. Actor Runtime 🔧 `--pipeline v1.30`

### Spawn an Actor

```
// Malay: lakaran = spawn actor
lakaran TugasCounter
MULA
    BINA count = 0
    SELAGI benar DAN benar MAKA
    MULA
        terima Mesej::Tambah -> count = count + 1,
        terima Mesej::Dapatkan -> hantar count
    TAMAT
TAMAT

// Expert: actor { }
actor TugasCounter {
    LET count = 0;
    LOOP {
        receive Mesej::Tambah -> count = count + 1,
        receive Mesej::Dapatkan -> send count
    }
}
```

### Channel Send / Receive

```
// Malay: hantar = send, terima = receive
hantal saluran Mesej ke TugasCounter    // send channel
BINA balas = terima saluran dalam 5s     // receive with timeout

// Expert:
send channel Mesej to TugasCounter;
LET balas = receive channel within 5s;
```

### Join (Wait for Actor)

```
// Malay: sertai = join/wait
sertai TugasCounter dalam 10s     // wait up to 10 seconds

// Expert:
join TugasCounter within 10s;
```

### Complete Actor Example

```
// Malay: Actor with channels
enumerasi MesejKalkulator
MULA
    Tambah(I32, I32),
    Tolak(I32, I32),
    Keputusan
TAMAT

lakaran Kalkulator
MULA
    SELAGI benar DAN benar MAKA
    MULA
        terima MesejKalkulator::Tambah(a, b) -> hantar a + b,
        terima MesejKalkulator::Tolak(a, b) -> hantar a - b,
        terima MesejKalkulator::Keputusan -> HENTI
    TAMAT
TAMAT

FUNGSI main() -> I32
MULA
    hantar saluran MesejKalkulator::Tambah(10, 20) ke Kalkulator
    BINA hasil = terima saluran
    PAPAR hasil               // 30
    hantar MesejKalkulator::Keputusan ke Kalkulator
    sertai Kalkulator
    PULANG 0
TAMAT

// Expert: identical
enum MesejKalkulator {
    Tambah(I32, I32),
    Tolak(I32, I32),
    Keputusan
}

actor Kalkulator {
    LOOP {
        receive MesejKalkulator::Tambah(a, b) -> send a + b,
        receive MesejKalkulator::Tolak(a, b) -> send a - b,
        receive MesejKalkulator::Keputusan -> break
    }
}

FN main() -> I32 {
    send channel MesejKalkulator::Tambah(10, 20) to Kalkulator;
    LET hasil = receive channel;
    PRINT hasil;              // 30
    send MesejKalkulator::Keputusan to Kalkulator;
    join Kalkulator;
    RETURN 0;
}
```

---

## 15. Backpressure & Scheduler 🔧 `--pipeline v1.30`

```
// Malay: tekanan = backpressure, jadual = scheduler
#[tekanan(simpanan = 100)]        // buffer 100 messages
#[jadual(utama)]                  // main scheduler thread
lakaran PenerimaMesej
MULA
    SELAGI benar DAN benar MAKA
    MULA
        terima Mesej::Data(d) -> proses d,
        terima Mesej::Henti -> HENTI
    TAMAT
TAMAT

#[tekanan(hantar = 50)]           // sender backpressure: 50 msg/sec
#[jadual(berkala = 16ms)]         // periodic scheduling every 16ms
lakaran PenghantarMesej
MULA
    BINA i = 0
    ULANG
    MULA
        hantar Mesej::Data(i) ke PenerimaMesej
        i = i + 1
        JIKA i > 1000 MAKA HENTI
    TAMAT
TAMAT

// Expert:
#[backpressure(buffer = 100)]
#[scheduler(main)]
actor PenerimaMesej {
    LOOP {
        receive Mesej::Data(d) -> process d,
        receive Mesej::Henti -> break
    }
}

#[backpressure(send = 50)]
#[scheduler(periodic = 16ms)]
actor PenghantarMesej {
    LET i = 0;
    LOOP {
        send Mesej::Data(i) to PenerimaMesej;
        i = i + 1;
        IF i > 1000 { break; }
    }
}
```

---

## 16. Keyword Reference

### v1.21 Keywords (Available Now)

| Malay | Expert | Meaning |
|-------|--------|---------|
| MULA | `{` | Block start |
| TAMAT | `}` | Block end |
| BINA | `LET` | Variable declaration |
| PAPAR | `PRINT` | Print statement |
| PULANG | `RETURN` | Return from function |
| FUNGSI | `FN` | Function declaration |
| JIKA | `IF` | Conditional |
| MAKA | (then) | If body follows |
| SEBALIKNYA JIKA | `ELSE if` | Else-IF branch |
| MELAINKAN | `ELSE` | Else branch |
| SELAGI | `WHILE` | While LOOP |
| DAN | `&&` | Logical AND (also bitwise) |
| ULANG | `LOOP` | Infinite LOOP |
| HENTI | `BREAK` | Break LOOP |
| LANGKAU | `CONTINUE` | Continue LOOP |
| TIDAK | `!` | Logical NOT |
| benar | `TRUE` | Boolean TRUE |
| palsu | `FALSE` | Boolean FALSE |
| DAN | `&` | Bitwise AND |
| ATAU | `\|` | Bitwise OR |
| XATAU | `^` | Bitwise XOR |
| GESERKIRI | `<<` | Left shift |
| GESERKANAN | `>>` | Right shift |
| JANGKAU | `~` | Bitwise NOT |

### v1.30 Keywords (`--pipeline v1.30`)

| Malay | Expert | Meaning |
|-------|--------|---------|
| struktur | `struct` | Struct definition |
| enumerasi | `enum` | Enum definition |
| sebagai | `as` | Type cast / repr |
| cocok | `MATCH` | Pattern matching |
| terima | `receive` | Actor receive |
| hantar | `send` | Actor send / channel send |
| saluran | `channel` | Communication channel |
| lakaran | `actor` | Actor spawn |
| sertai | `join` | Wait for actor |
| petunjuk | `PTR` / `*` | Pointer type |
| @ | `&` | Address-of |
| ^ | `*` | Dereference |
| terbuka | `pub` | Public visibility |
| tertutup | `private` | Private visibility |
| tatasusunan | `[T; n]` | Array type |
| panjang | `length` | Array/string length |
| tekanan | `backpressure` | Backpressure attribute |
| jadual | `scheduler` | Scheduler attribute |
| berkala | `periodic` | Periodic scheduling |

> **Dictionary Dependency:** Malay aliases (e.g., `BINA`, `SELAGI`, `PAPAR`) require the file `dict/core_map.json` to be present at runtime (in the working directory). If this file is missing, the compiler falls back to hardcoded default aliases, but some Malay keywords and aliases may not be available. Ensure `dict/core_map.json` is distributed alongside the compiler binary.

---

## Running Examples

### v1.21 (default)
```bash
logicodex compile hello.ldx
# Compiles basic types, functions, control flow
```

### v1.30 Option Engine
```bash
logicodex --pipeline v1.30 compile hello_v130.ldx
# Parses + checks: strings, structs, enums, arrays, actors
# HIR lowering: active for all features above
# LLVM codegen: structs, functions, calls (extending to full set)
```

---


## Type Inference 🔧 `--pipeline v1.30`

Type annotation is **optional** in Logicodex. The compiler infers types from literals.

### Rule

```
If type is declared:
  compiler follows declared type and validates value compatibility.

If type is not declared:
  compiler infers type from literal or expression.

Once declared/inferred: variable type is fixed.
```

### Examples

```
// Malay: Type declared explicitly
BINA x: I32 = 1           // I32 (declared)
BINA y: I64 = 9999999999  // I64 (declared, large value)

// Malay: Type inferred from literal
BINA a = 1        // I32 (integer literal, fits I32)
BINA b = 3.14     // F64 (float literal, default F64)
BINA c = 2.5f     // F32 (suffix 'f' forces F32)
BINA d = 100000   // I64 (auto-upgraded: exceeds I32 range)

// Expert canonical:
LET x: I32 = 1;
LET a = 1;        // inferred: i32
LET b = 3.14;     // inferred: f64
LET c = 2.5f32;   // inferred: f32
```

### Inference Rules

| Literal | Inferred Type | Reason |
|---------|---------------|--------|
| `42` | I32 | Integer default |
| `2147483648` | I64 | Exceeds I32 max (auto-upgrade) |
| `3.14` | F64 | Float default |
| `2.5f` | F32 | `f` suffix forces F32 |
| `"text"` | String | String literal |
| `TRUE` / `FALSE` | Bool | Boolean literal |

### Compatibility Rules

```
VALID:   BINA x: I32 = 1        // 1 fits in I32
VALID:   BINA y: I64 = 1        // 1 fits in I64 (widen)
VALID:   BINA z: F64 = 1        // int → float allowed

ERROR:   BINA x: I32 = 9999999999   // exceeds I32 max
ERROR:   BINA y: I32 = 3.14         // float → int prohibited
ERROR:   BINA z: F32 = 3.14         // F64 literal → F32 narrowing
```

### Fixed Type After Inference

```
// Malay:
BINA x = 1        // inferred: I32
x = 2             // OK: still I32
x = 3.14          // ERROR: cannot change I32 to F64

// Expert:
LET x = 1;        // inferred: i32
x = 2;            // OK
x = 3.14;         // ERROR: type mismatch
```

> **Scope:** Type inference applies to v1.30+ (`--pipeline v1.30`).
> See `docs/design/TYPE-INFERENCE-RULES.md` for full specification.


## The v1.30 Option Engine

Logicodex has **two compiler pipelines**:

| Pipeline | Activation | Status | Use Case |
|----------|-----------|--------|----------|
| **v1.21** (default) | No flag needed | ✅ WORKING | Production compilation — native binaries |
| **v1.30 Option Engine** | `--pipeline v1.30` | ⚠️ PARTIAL | Advanced features — HIR lowered, LLVM extending |

### What is the Option Engine?

The v1.30 Option Engine is an **80%-complete HIR lowering pipeline** that adds:
- **Type system:** Strings, structs, enums, arrays, pointers
- **Control:** Match expressions, visibility, attributes
- **Concurrency:** Actor runtime (spawn, channels, join)
- **Safety:** Backpressure, scheduler hooks

It is called an **"Option Engine"** because:
1. It is **opt-in** — use `--pipeline v1.30` to activate
2. It does **not** affect v1.21 stability — both coexist
3. Features are **staged** — parse → HIR → LLVM codegen progressively

### How to Use

```bash
# v1.21 (default) — stable, produces binaries
logicodex compile hello.ldx

# v1.30 Option Engine — advanced features
logicodex --pipeline v1.30 compile advanced.ldx

# v1.30 check — validate without compiling
logicodex --pipeline v1.30 check myfile.ldx

# v1.30 subsystem self-check
logicodex v130-check examples/dormant/v1_30/raylib_spinning_box.ldx
```

### Feature Status Matrix

| Feature | Parser | HIR Lowering | LLVM Codegen |
|---------|--------|-------------|--------------|
| Strings | ✅ | ✅ | 🟡 |
| Structs | ✅ | ✅ | ✅ |
| Enums + Match | ✅ | ✅ | 🟡 |
| Arrays | ✅ | ✅ | 🟡 |
| Pointers | ✅ | ✅ | 🟡 |
| Actor (spawn/join) | ✅ | ✅ | ✅ |
| Actor (channels) | ✅ | ✅ | ✅ |
| Visibility | ✅ | ✅ | 🟡 |
| Attributes | ✅ | ✅ | 🟡 |

Legend: ✅ Complete | 🟡 Extending | ⚪ Not Started

## Summary: v1.30 Option Engine Capabilities

| Feature | v1.21 | v1.30 | Impact |
|---------|-------|-------|--------|
| Variables, arithmetic | ✅ | ✅ | Baseline |
| Functions | ✅ | ✅ | Baseline |
| Control flow | ✅ | ✅ | Baseline |
| Bitwise ops | ✅ | ✅ | Baseline |
| **Strings** | ❌ | ✅ Parsed | **Major** |
| **Structs** | ❌ | ✅ Parsed | **Major** |
| **Enums + Match** | ❌ | ✅ Parsed | **Major** |
| **Arrays** | ❌ | ✅ Parsed | **Major** |
| **Pointers** | ❌ | ✅ Parsed | **Major** |
| **Visibility** | ❌ | ✅ Parsed | **New** |
| **Attributes** | ❌ | ✅ Parsed | **New** |
| **Actor Runtime** | ❌ | ✅ HIR Lowered | **Major** |
| **Backpressure** | ❌ | 🟡 Design | **Major** |
| **Scheduler** | ❌ | 🟡 Design | **Major** |

---

> **v1.30 Option Engine Status (2026-06-01):**
>
> v1.30 is an 80%-complete HIR lowering pipeline. All features listed are **parsed**
> and **HIR-lowered**. LLVM codegen is active for structs, functions, and calls.
> The `--pipeline v1.30` flag activates the Option Engine without affecting v1.21.
>
> | Pipeline | Status |
> |----------|--------|
> | v1.21 (default) | ✅ WORKING — produces native binaries |
> | v1.30 Option Engine | ⚠️ PARTIAL — HIR active, LLVM codegen extending |
>
> See `V130_OPTION_ENGINE.md` for full capability matrix and activation path.
> See `docs/design/TYPE-INFERENCE-RULES.md` for type inference (Phase 2).
