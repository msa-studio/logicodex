# Logicodex — Getting Started & Complete Syntax Reference

> **Version:** v1.45.0-alpha  
> **Engines:** v1.21 (default) | v1.30 (`--edition v1.30`)  
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
7. [Strings](#7-strings) — `--edition v1.30`
8. [Structs](#8-structs) — `--edition v1.30`
9. [Enums & Match](#9-enums--match) — `--edition v1.30`
10. [Arrays](#10-arrays) — `--edition v1.30`
11. [Pointers](#11-pointers) — `--edition v1.30`
12. [Visibility](#12-visibility) — `--edition v1.30`
13. [Attributes](#13-attributes) — `--edition v1.30`
14. [Actor Runtime](#14-actor-runtime) — `--edition v1.30`
15. [Backpressure & Scheduler](#15-backpressure--scheduler) — `--edition v1.30`
16. [Keyword Reference](#16-keyword-reference)

---

## Legend

| Marker | Meaning |
|--------|---------|
| ✅ | Available in v1.21 (default) |
| 🔷 | Available in `--edition v1.30` |
| 📝 | Both Malay + Expert syntax shown |

---

## 1. Quick Hello World

```
// Malay: MULA = start block, PAPAR = print, TAMAT = end block
MULA
    PAPAR 42
TAMAT

// Expert: { } delimit block, print statement, semicolon
{
    print 42;
}
```

**Output:** `42`

---

## 2. Variables & Types

### Variable Declaration

```
// Malay: BINA = bind/create variable
BINA nama = "Ahmad"       // Error in v1.21 — strings not supported
BINA umur = 25            // I32 (integer, inferred)
BINA gaji = 3500.50       // F64 (float, inferred)
BINA aktif = benar        // Bool (boolean, inferred)

// Expert: let = bind variable
let nama = "Ahmad";       // v1.30 only
let umur = 25;            // v1.21 — integer literal → I32
let gaji = 3500.50;       // v1.21 — float literal → F64
let aktif = true;         // v1.21 — boolean literal → Bool
```

### Explicit Type Annotation

```
// Malay: type after name with : separator
BINA bilangan : I32 = 100
BINA besar : I64 = 9999999999
BINA suhu : F32 = 36.6
BINA panjang : F64 = 3.14159265359
BINa hidup : Bool = benar

// Expert: same syntax, semicolon terminated
let bilangan: I32 = 100;
let besar: I64 = 9999999999;
let suhu: F32 = 36.6;
let panjang: F64 = 3.14159265359;
let hidup: Bool = true;
```

### Available Types

| Type | Malay Literal | Expert Literal | Size | v1.21 | v1.30 |
|------|--------------|----------------|------|-------|-------|
| `I32` | integer | `42` | 32-bit signed | ✅ | ✅ |
| `I64` | large integer | `999999` | 64-bit signed | ✅ | ✅ |
| `F32` | small float | `3.14` | 32-bit float | ✅ | ✅ |
| `F64` | large float | `3.14159` | 64-bit float | ✅ | ✅ |
| `Bool` | `benar` / `palsu` | `true` / `false` | 1-bit | ✅ | ✅ |
| `String` | `"hello"` | `"hello"` | heap-allocated | 🔷 | 🔷 |

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
let a = 10;
let b = 3;
let tambah = a + 3;           // 13
let tolak = a - 3;            // 7
let darab = a * 3;            // 30
let bahagi = a / 3;           // 3
let baki = a % 3;             // 1
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
let x = 5;
let y = 10;
let sama = x == y;            // false
let tak_sama = x != y;        // true
let kecil = x < y;            // true
let besar = x > y;            // false
let kecil_sama = x <= y;      // true
let besar_sama = x >= y;      // false
```

### Boolean

```
// Malay: DAN = AND, ATAU = OR, TIDAK = NOT
BINA p = benar
BINA q = palsu
BINA dan = p DAN q            // false
BINA atau = p ATAU q           // true
BINA tak = TIDAK p             // false

// Expert: &&, ||, !
let p = true;
let q = false;
let dan = p && q;             // false
let atau = p || q;            // true
let tak = !p;                 // false
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
let m = 0b1100;               // 12
let n = 0b1010;               // 10
let bit_and = m & n;          // 0b1000 = 8
let bit_or = m | n;           // 0b1110 = 14
let bit_xor = m ^ n;          // 0b0110 = 6
let shift_left = m << 2;      // 0b110000 = 48
let shift_right = m >> 2;     // 0b0011 = 3
let bit_not = ~m;             // 0b0011 = 3
```

---

## 4. Control Flow

### If / Else If / Else

```
// Malay: JIKA = if, MAKA = then, SEBALIKNYA JIKA = else if, MELAINKAN = else
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

// Expert: if / else if / else
if umur >= 18 {
    print 1;
} else if umur >= 13 {
    print 2;
} else {
    print 3;
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

// Expert: while { }
let i = 0;
while i < 5 {
    print i;
    i = i + 1;
}
```

### Infinite Loop

```
// Malay: ULANG = loop forever
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

// Expert: loop { }
let counter = 0;
loop {
    print counter;
    counter = counter + 1;
    if counter >= 10 {
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
while i < 100 {
    i = i + 1;
    if i % 2 == 0 {
        continue;
    }
    print i;
}
```

---

## 5. Functions

### Function Declaration

```
// Malay: FUNGSI = function, -> = return type, PULANG = return
FUNGSI tambah(a: I32, b: I32) -> I32
MULA
    PULANG a + b
TAMAT

// Expert: fn, return
fn tambah(a: I32, b: I32) -> I32 {
    return a + b;
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

// Expert: returns I32 (must have return type)
fn ucap_hello() -> I32 {
    print 1;
    return 0;
}
```

### Function Call

```
// Malay: call like expression
BINA hasil = tambah(10, 20)
PAPAR hasil

// Expert: identical
let hasil = tambah(10, 20);
print hasil;
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

---

## 7. Strings 🔷 `--edition v1.30`

```
// Malay: String literal with ""
BINA nama = "Logicodex"
BINA salam = "Selamat datang!"
PAPAR nama.length           // 9

// Expert: same syntax
let nama = "Logicodex";
let salam = "Selamat datang!";
print nama.length;          // 9

// String concatenation
BINA penuh = "Hello" + " " + "World"    // "Hello World"

// Expert:
let penuh = "Hello" + " " + "World";   // "Hello World"
```

---

## 8. Structs 🔷 `--edition v1.30`

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
let titik = Point { x: 10.0, y: 20.0 };
let orang = Person { nama: "Ahmad", umur: 25 };
```

### Field Access

```
// Malay: dot notation
PAPAR titik.x       // 10.0
PAPAR titik.y       // 20.0
PAPAR orang.nama    // "Ahmad"

// Expert: identical
print titik.x;      // 10.0
print titik.y;      // 20.0
print orang.nama;   // "Ahmad"
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

let bulat = Circle {
    pusat: Point { x: 0.0, y: 0.0 },
    radius: 5.0
};
print bulat.pusat.x;   // 0.0
```

---

## 9. Enums & Match 🔷 `--edition v1.30`

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
    Status::Aktif -> PAPAR 1,
    Status::TidakAktif -> PAPAR 2,
    Status::Diseret -> PAPAR 3
TAMAT

// Expert: match { }
let status = Status::Aktif;
match status {
    Status::Aktif -> print 1,
    Status::TidakAktif -> print 2,
    Status::Diseret -> print 3
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
let m = Mesej::Gerak { x: 10, y: 20 };
match m {
    Mesej::Henti -> print 0,
    Mesej::Gerak { x, y } -> print x + y,
    Mesej::Tulis(s) -> print s.length
}
```

---

## 10. Arrays 🔷 `--edition v1.30`

### Array Declaration

```
// Malay: tatasusunan = array, fixed size
BINA nombor = [I32; 5]              // [0, 0, 0, 0, 0]
BINA nama = ["Ahmad", "Bakar", "Chin"]    // [String; 3]

// Expert: identical
let nombor = [I32; 5];              // [0, 0, 0, 0, 0]
let nama = ["Ahmad", "Bakar", "Chin"];  // [String; 3]
```

### Array Indexing

```
// Malay: [index] access
PAPAR nama[0]       // "Ahmad"
PAPAR nama[1]       // "Bakar"

// Assignment
nama[2] = "David"

// Expert: identical
print nama[0];      // "Ahmad"
print nama[1];      // "Bakar"
nama[2] = "David";
```

### Array Length

```
// Malay: .panjang
PAPAR nombor.panjang    // 5

// Expert: .length
print nombor.length;    // 5
```

---

## 11. Pointers 🔷 `--edition v1.30`

```
// Malay: petunjuk = pointer, @ = address-of, ^ = dereference
BINA a = 42
BINA ptr_a = @a              // pointer to a
PAPAR ^ptr_a                 // 42 (dereference)

// Expert: & = address-of, * = dereference
let a = 42;
let ptr_a = &a;              // pointer to a
print *ptr_a;                // 42

// Mutable pointer
BINA b = 10
BINA ptr_b = @b
^ptr_b = 20                  // dereference and assign
PAPAR b                      // 20

// Expert:
let b = 10;
let ptr_b = &b;
*ptr_b = 20;                // dereference and assign
print b;                     // 20
```

---

## 12. Visibility 🔷 `--edition v1.30`

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

pub fn get_nama(c: Config) -> String {
    return c.nama;
}
```

---

## 13. Attributes 🔷 `--edition v1.30`

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
fn raw_alloc(saiz: I64) -> Pointer {
    // ... FFI call
}
```

---

## 14. Actor Runtime 🔷 `--edition v1.30`

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
    let count = 0;
    loop {
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
let balas = receive channel within 5s;
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
    loop {
        receive MesejKalkulator::Tambah(a, b) -> send a + b,
        receive MesejKalkulator::Tolak(a, b) -> send a - b,
        receive MesejKalkulator::Keputusan -> break
    }
}

fn main() -> I32 {
    send channel MesejKalkulator::Tambah(10, 20) to Kalkulator;
    let hasil = receive channel;
    print hasil;              // 30
    send MesejKalkulator::Keputusan to Kalkulator;
    join Kalkulator;
    return 0;
}
```

---

## 15. Backpressure & Scheduler 🔷 `--edition v1.30`

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
    loop {
        receive Mesej::Data(d) -> process d,
        receive Mesej::Henti -> break
    }
}

#[backpressure(send = 50)]
#[scheduler(periodic = 16ms)]
actor PenghantarMesej {
    let i = 0;
    loop {
        send Mesej::Data(i) to PenerimaMesej;
        i = i + 1;
        if i > 1000 { break; }
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
| BINA | `let` | Variable declaration |
| PAPAR | `print` | Print statement |
| PULANG | `return` | Return from function |
| FUNGSI | `fn` | Function declaration |
| JIKA | `if` | Conditional |
| MAKA | (then) | If body follows |
| SEBALIKNYA JIKA | `else if` | Else-if branch |
| MELAINKAN | `else` | Else branch |
| SELAGI | `while` | While loop |
| DAN | `&&` | Logical AND (also bitwise) |
| ULANG | `loop` | Infinite loop |
| HENTI | `break` | Break loop |
| LANGKAU | `continue` | Continue loop |
| TIDAK | `!` | Logical NOT |
| benar | `true` | Boolean true |
| palsu | `false` | Boolean false |
| DAN | `&` | Bitwise AND |
| ATAU | `\|` | Bitwise OR |
| XATAU | `^` | Bitwise XOR |
| GESERKIRI | `<<` | Left shift |
| GESERKANAN | `>>` | Right shift |
| JANGKAU | `~` | Bitwise NOT |

### v1.30 Keywords (`--edition v1.30`)

| Malay | Expert | Meaning |
|-------|--------|---------|
| struktur | `struct` | Struct definition |
| enumerasi | `enum` | Enum definition |
| sebagai | `as` | Type cast / repr |
| cocok | `match` | Pattern matching |
| terima | `receive` | Actor receive |
| hantar | `send` | Actor send / channel send |
| saluran | `channel` | Communication channel |
| lakaran | `actor` | Actor spawn |
| sertai | `join` | Wait for actor |
| petunjuk | `ptr` / `*` | Pointer type |
| @ | `&` | Address-of |
| ^ | `*` | Dereference |
| terbuka | `pub` | Public visibility |
| tertutup | `private` | Private visibility |
| tatasusunan | `[T; n]` | Array type |
| panjang | `length` | Array/string length |
| tekanan | `backpressure` | Backpressure attribute |
| jadual | `scheduler` | Scheduler attribute |
| berkala | `periodic` | Periodic scheduling |

---

## Running Examples

### v1.21 (default)
```bash
logicodex compile hello.ldx
# Compiles basic types, functions, control flow
```

### v1.30 (upgraded)
```bash
logicodex --edition v1.30 compile hello_v130.ldx
# Compiles strings, structs, enums, arrays, actors
```

---

## Summary: What v1.30 Unlocks

| Feature | v1.21 | v1.30 | Impact |
|---------|-------|-------|--------|
| Variables, arithmetic | ✅ | ✅ | Baseline |
| Functions | ✅ | ✅ | Baseline |
| Control flow | ✅ | ✅ | Baseline |
| Bitwise ops | ✅ | ✅ | Baseline |
| **Strings** | ❌ | ✅ | **Major** |
| **Structs** | ❌ | ✅ | **Major** |
| **Enums + Match** | ❌ | ✅ | **Major** |
| **Arrays** | ❌ | ✅ | **Major** |
| **Pointers** | ❌ | ✅ | **Major** |
| **Visibility** | ❌ | ✅ | **New** |
| **Attributes** | ❌ | ✅ | **New** |
| **Actor Runtime** | ❌ | ✅ | **Major** |
| **Backpressure** | ❌ | ✅ | **Major** |
| **Scheduler** | ❌ | ✅ | **Major** |

---

> This document describes syntax and capabilities as designed in the HIR module.
> v1.30 features require `--edition v1.30` and HIR → LLVM codegen integration.
> See `V130_OPTION_ENGINE.md` for activation path.
