# Foreign-Language Types: C Interop vs Translation Scaffolding

Logicodex distinguishes **two separate concerns** that both involve C (and, later,
other source languages). They are intentionally kept apart.

## 1. FFI C interop — *real and working*

Calling existing C functions is supported today through `extern "C"` blocks
(the default ABI is `"C"`). This is genuine, link-level interop: you declare a C
function's signature in Logicodex and the linker resolves it. This is how the
raylib bindings work.

For declaring FFI signatures, `TypeRegistry` exposes convenience C-type aliases
that return the matching native `TypeId`:

| FFI alias    | Native |
|--------------|--------|
| `c_int()`    | I32    |
| `c_uint()`   | U32    |
| `c_long()`   | I64    |
| `c_char()`   | I8     |
| `c_uchar()`  | U8     |
| `c_float()`  | F32    |
| `c_double()` | F64    |

These exist for hand-written FFI signatures. They are **not** a translation
mechanism and they do not read C source.

## 2. Translation scaffolding — *prepared, inert*

When a front-end that *translates* another language's source into Logicodex is
built, it needs to map every foreign scalar type onto a native Logicodex
`PrimitiveType` **before** HIR lowering, so that codegen only ever sees native
types and carries zero foreign-type overhead.

That mapping lives in the `LegacyType` enum (`src/types.rs`) and its
`canonical_native()` method. `LegacyType` is deliberately **inert**: nothing on
the normal compilation path produces or consumes it. It exists only so a future
translator has a single, extensible place to record source-language semantics.

### C scalar mapping (`LegacyType` → native `PrimitiveType`)

| C type                  | `LegacyType`        | Native |
|-------------------------|---------------------|--------|
| `char`                  | `CChar`             | I8     |
| `signed char`           | `CSignedChar`       | I8     |
| `unsigned char`         | `CUnsignedChar`     | U8     |
| `short`                 | `CShort`            | I16    |
| `unsigned short`        | `CUnsignedShort`    | U16    |
| `int`                   | `CInt`              | I32    |
| `unsigned int`          | `CUnsignedInt`      | U32    |
| `long`                  | `CLong`             | I64    |
| `unsigned long`         | `CUnsignedLong`     | U64    |
| `long long`             | `CLongLong`         | I64    |
| `unsigned long long`    | `CUnsignedLongLong` | U64    |
| `size_t`                | `CSizeT`            | U64    |
| `ssize_t`               | `CSsizeT`           | I64    |
| `intptr_t`              | `CIntPtr`           | I64    |
| `uintptr_t`             | `CUIntPtr`          | U64    |
| `ptrdiff_t`             | `CPtrDiffT`         | I64    |
| `wchar_t` (Linux/macOS) | `CWCharT`           | I32    |
| `char16_t`              | `CChar16T`          | U16    |
| `char32_t`              | `CChar32T`          | U32    |
| `_Bool` / `bool`        | `CBool`             | Bool   |
| `float`                 | `CFloat`            | F32    |
| `double`                | `CDouble`           | F64    |
| `long double`           | `CLongDouble`       | F64 *  |
| `void`                  | `CVoid`             | Unit   |

\* `long double` is approximated as F64 — Logicodex has no f80/f128 type.

### Out of scope for `LegacyType`

- **Pointers** (`void*`, `char*`, `T*`) are not scalars; they are represented as
  pointer types elsewhere, not via `canonical_native()`.
- **`stdint.h` exact-width types** (`int8_t` … `uint64_t`) map 1:1 onto the
  native widths (I8 … U64) by name, so a translator can lower them directly
  without a dedicated `LegacyType` variant.

### Pascal family

The same enum already carries a Pascal scalar set (`PascalShortInt`,
`PascalByte`, `PascalSmallInt`, `PascalWord`, `PascalInteger`, `PascalLongWord`,
`PascalInt64`) as a template for how additional source languages are added:
extend `LegacyType` and `canonical_native()`, nothing else.

## The `c` / `C_INTEROP` keyword

The lexer reserves `c`, `C_INTEROP`, `C_LEGACY`, and `C_LUAR` as
`TokenKind::CInterop`, but the parser does **not** currently wire this keyword to
any construct. It is reserved for future interop/translation syntax and is inert
today. (Because `c` is a reserved keyword, it must not be used as an identifier.)

## Status summary

| Capability                        | Status            |
|-----------------------------------|-------------------|
| Call C functions via `extern "C"` | Working           |
| FFI C-type aliases (`c_int`, …)   | Present, for FFI signatures |
| `LegacyType` C scalar mapping     | Present, inert (translation-only) |
| `LegacyType` Pascal scalar mapping| Present, inert (translation-only) |
| C source → Logicodex translator   | Not started       |
| `c` / `C_INTEROP` keyword wiring  | Reserved, unwired |
