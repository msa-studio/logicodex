# Chapter 3: Sintaks dan Token

Bab ini menerangkan sintaks Logicodex dalam kedalaman — dari alias bahasa sehingga tatasusila (grammar) lengkap.

---

## Alias Melayu vs Canonical {#alias}

### Peta Alias Lengkap

| Tujuan | Melayu | English | Canonical |
|---|---|---|---|
| Mulai blok | `MULA` | `BEGIN` / `START` | `{` |
| Tamat blok | `TAMAT` | `END` / `FINISH` | `}` |
| Deklarasi variable | `BINA` | `CREATE` / `LET` | `let` |
| Cetak output | `PAPAR` | `DISPLAY` / `PRINT` | `print` |
| Pulang nilai | `PULANG` | `RETURN` | `return` |
| Fungsi | `FUNGSI` | `FUNCTION` | `fn` |
| Program | `PROGRAM` | `PROGRAM` | `program` |
| Jika | `JIKA` | `IF` | `if` |
| Lain (else) | `LAIN` | `ELSE` | `else` |
| Lain jika | `LAIN_JIKA` | `ELSE_IF` | `else if` |
| Untuk (for) | `UNTUK` | `FOR` | `for` |
| Sementara (while) | `SEMENTARA` | `WHILE` | `while` |
| Patah (break) | `PATAH` | `BREAK` | `break` |
| Terus (continue) | `TERUS` | `CONTINUE` | `continue` |
| Padan (match) | `PADAN` | `MATCH` | `match` |
| Benar | `BENAR` | `TRUE` | `true` |
| Palsu | `PALSU` | `FALSE` | `false` |
| Guna tipe | `GUNA_JENIS` | `USE_TYPE` | `use` |
| Struktur | `STRUKTUR` | `STRUCT` | `struct` |
| Enumerasi | `ENUMERASI` | `ENUM` | `enum` |
| Pilihan | `PILIHAN` | `OPTION` | `option` |
| Keputusan | `KEPUTUSAN` | `RESULT` | `result` |
| Saluran | `SALURAN` | `CHANNEL` | `channel` |
| Pelakon | `PELAKON` | `ACTOR` | `actor` |
| Hidupkan | `HIDUPKAN` | `SPAWN` | `spawn` |
| Perkhidmatan | `PERKHIDMATAN` | `SERVICE` | `service` |
| Keperluan | `KEPERLUAN` | `REQUIRES` | `requires` |
| Pengendali | `PENGENDALI` | `HANDLER` | `handler` |
| Dasar | `DASAR` | `POLICY` | `policy` |
| Halang | `HALANG` | `BLOCK` | `block` |
| Gugur_Terlama | `GUGUR_TERLAMA` | `DROP_OLDEST` | `drop_oldest` |
| Ralat | `RALAT` | `ERROR` | `error` |

### Memilih Gaya

Anda boleh mencampurkan alias dalam satu program, tetapi **tidak disyorkan** — gunakan satu gaya secara konsisten:

```logicodex
-- ❌ Jangan campur gaya
MULA                          -- Melayu
    let x = 5;                -- Canonical
    JIKA x > 3                -- Melayu
        DISPLAY "Besar";      -- English
    END                       -- English
TAMAT                         -- Melayu
```

```logicodex
-- ✅ Gunakan satu gaya secara konsisten
MULA
    BINA x SEBAGAI I32 = 5
    JIKA x > 3
        PAPAR "Besar"
    TAMAT_JIKA
TAMAT
```

---

## Tatasusila (Grammar) Asas {#grammar}

### Program

```bnf
<program>       ::= <program_decl> <decl_list> <program_end>

<program_decl>  ::= "PROGRAM" <identifier>
                  | "program" <identifier> "{"

<program_end>   ::= "TAMAT PROGRAM"
                  | "END PROGRAM"
                  | "}"

<decl_list>     ::= <declaration>*

<declaration>   ::= <function_decl>
                  | <actor_decl>
                  | <service_decl>
                  | <struct_decl>
                  | <enum_decl>
                  | <use_decl>
                  | <variable_decl>
```

### Fungsi

```bnf
<function_decl> ::= "FUNGSI" <identifier> "(" <param_list>? ")" "->" <type> <block>
                  | "fn" <identifier> "(" <param_list>? ")" "->" <type> <block>

<param_list>    ::= <param> ("," <param>)*
<param>         ::= <identifier> ":" <type>

<block>         ::= "MULA" <stmt>* "TAMAT"
                  | "BEGIN" <stmt>* "END"
                  | "{" <stmt>* "}"

<stmt>          ::= <variable_decl>
                  | <assignment>
                  | <if_stmt>
                  | <for_stmt>
                  | <while_stmt>
                  | <match_stmt>
                  | <return_stmt>
                  | <expr_stmt>

<return_stmt>   ::= "PULANG" <expr>? ";"
                  | "RETURN" <expr>? ";"
                  | "return" <expr>? ";"
```

### Ungkapan (Expressions)

```bnf
<expr>          ::= <literal>
                  | <identifier>
                  | <binary_expr>
                  | <call_expr>
                  | <field_access>
                  | <array_index>
                  | "(" <expr> ")"

<binary_expr>   ::= <expr> <bin_op> <expr>
<bin_op>        ::= "+" | "-" | "*" | "/" | "%"
                  | "==" | "!=" | "<" | ">" | "<=" | ">="
                  | "&&" | "||"

<call_expr>     ::= <identifier> "(" <arg_list>? ")"
<arg_list>      ::= <expr> ("," <expr>)*

<field_access>  ::= <expr> "." <identifier>
<array_index>   ::= <expr> "[" <expr> "]"
```

### Literal

```bnf
<literal>       ::= <int_literal>
                  | <float_literal>
                  | <string_literal>
                  | <bool_literal>

<int_literal>   ::= [0-9]+ [ "I8" | "I16" | "I32" | "I64" | "ISize"
                          | "U8"  | "U16" | "U32" | "U64" | "USize" ]?

<float_literal> ::= [0-9]+ "." [0-9]+ [ "F32" | "F64" ]?

<string_literal> ::= "\"" [^\"]* "\""

<bool_literal>  ::= "BENAR" | "TRUE" | "true"
                  | "PALSU" | "FALSE" | "false"
```

---

## Komen dan Dokumentasi {#komen}

### Komen Baris Tunggal

```logicodex
-- Ini adalah komen dalam gaya Melayulogic
--- Ini juga komen (tiga dash)
// Ini adalah komen dalam gaya C
/// Ini adalah doc comment (akan dipapar dalam dokumentasi)
```

### Komen Pelbagai Baris

```logicodex
/*
  Ini adalah komen
  yang merentasi
  beberapa baris
*/
```

### Doc Comments

Doc comments menggunakan `///` dan akan dipapar dalam dokumentasi automatik:

```logicodex
/// Menghitung hasil tambah dua nombor
/// 
/// Parameter:
///   a: Nombor pertama
///   b: Nombor kedua
/// 
/// Pulang:
///   Hasil tambah a dan b
FUNGSI tambah(a: I32, b: I32) -> I32
MULA
    PULANG a + b
TAMAT
```

---

## Contoh Program Lengkap (Melayu)

```logicodex
PROGRAM kalkulator

-- Guna tipe yang diperlukan
GUNA_JENIS I32
GUNA_JENIS F64
GUNA_JENIS Text

-- Fungsi bantu
FUNGSI kuadrat(x: I32) -> I32
MULA
    PULANG x * x
TAMAT

FUNGSI is_genap(n: I32) -> Bool
MULA
    PULANG n % 2 == 0
TAMAT

-- Fungsi utama
FUNGSI utama() -> I32
MULA
    BINA nama SEBAGAI Text = "Pengguna"
    BINA umur SEBAGAI I32 = 25
    
    PAPAR "Halo, " + nama + "!"
    PAPAR "Umur anda: " + umur
    
    BINA tahun_lahir SEBAGAI I32 = 2026 - umur
    PAPAR "Anda lahir pada tahun: " + tahun_lahir
    
    BINA nombor SEBAGAI I32 = 7
    JIKA is_genap(nombor)
        PAPAR nombor + " adalah genap"
    LAIN
        PAPAR nombor + " adalah ganjil"
    TAMAT_JIKA
    
    BINA i SEBAGAI I32 = 1
    SEMENTARA i <= 5
        PAPAR "Isi padu " + i + " = " + kuadrat(i)
        i = i + 1
    TAMAT_SEMENTARA
    
    PULANG 0
TAMAT

TAMAT PROGRAM
```
