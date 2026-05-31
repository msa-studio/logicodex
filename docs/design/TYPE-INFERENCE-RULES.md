# Type Inference Rules — Finalized

**Status:** FINALIZED — Architecture Decision  
**Authority:** Mohamad Supardi Abdul (@msa-studio), Lead Maintainer  
**Date:** 2026-05-31  
**Governance:** Per ROADMAP_POLICY.md §4.2 — Architecture decisions by lead maintainer  
**Scope:** Applies to v1.30+ (`--edition v1.30`)

---

## Rule (Definitive)

```
Type annotation is optional.

If type is declared:
  compiler must follow declared type and validate value compatibility.

If type is not declared:
  compiler must infer type from literal or expression.

Once declared/inferred:
  variable type is fixed.
```

---

## Rule Breakdown

### 1. Type Annotation is Optional

The programmer MAY include a type annotation. The compiler MUST NOT require it.

```
// Explicit type — VALID
BINA x: I32 = 1

// Implicit type — VALID (compiler infers I32)
BINA x = 1

// Both are acceptable. Both produce identical output.
```

### 2. If Type is Declared → Compiler Validates

When a type annotation is present, the compiler:
1. Uses the declared type as the variable's type
2. Validates that the assigned value is compatible
3. Rejects with error if incompatible

```
// VALID: 1 fits in I32
BINA x: I32 = 1

// VALID: 1 fits in I64
BINA y: I64 = 1

// ERROR: 9999999999 exceeds I32 max (2,147,483,647)
BINA z: I32 = 9999999999    → COMPILE ERROR: value 9999999999 out of range for I32

// ERROR: 3.14 cannot be assigned to I32
BINA w: I32 = 3.14          → COMPILE ERROR: F64 literal cannot be assigned to I32
```

### 3. If Type is Not Declared → Compiler Infers

When no type annotation is present, the compiler infers from the literal:

| Literal Form | Inferred Type | Rule |
|-------------|---------------|------|
| Integer, fits I32 | `I32` | Default integer type |
| Integer, exceeds I32, fits I64 | `I64` | Auto-upgrade to prevent overflow |
| Integer, exceeds I64 | **COMPILE ERROR** | Integer too large |
| Float, no suffix | `F64` | Default float precision |
| Float, `f` suffix | `F32` | Explicit lower precision |
| Boolean `benar`/`palsu` | `Bool` | Boolean type |
| Boolean `true`/`false` | `Bool` | Boolean type |
| String `"..."` | `String` | String type (v1.30) |

```
// Inference examples
BINA a = 1              → I32 (fits I32)
BINA b = 9999999999     → I64 (exceeds I32, auto-upgrade)
BINA c = 3.14159        → F64 (default float)
BINA d = 2.5f           → F32 (f suffix forces F32)
BINA e = benar          → Bool
BINA f = "hello"        → String (v1.30)

// Malay and Expert identical
let a = 1;              → I32
let b = 9999999999;     → I64
let c = 3.14159;        → F64
```

### 4. Once Set → Type is Fixed

After declaration (explicit or inferred), the type cannot change:

```
// Inferred as I32
BINA x = 1              // x is I32
x = 9999999999          // COMPILE ERROR: 9999999999 out of range for I32
                        // Must redeclare: BINA x: I64 = 9999999999

// Explicit I64
BINA y: I64 = 1         // y is I64
y = 9999999999          // VALID: fits I64

// Inferred as F64
BINA z = 3.14           // z is F64
z = 2.71828             // VALID: same type (F64)
```

---

## Inference Algorithm (Pseudocode)

```
fn resolve_variable_type(maybe_annotation: Option<Type>, value: Expression) -> Type {
    if let Some(declared_type) = maybe_annotation {
        // Rule 2: Validate declared type against value
        if !is_compatible(declared_type, value) {
            report_error(TypeMismatch { expected: declared_type, got: value.type() });
            return Type::Error;
        }
        return declared_type;
    } else {
        // Rule 3: Infer from literal
        return infer_from_literal(value);
    }
}

fn infer_from_literal(value: Expression) -> Type {
    match value {
        Literal::Integer(n) => {
            if n >= I32_MIN && n <= I32_MAX { Type::I32 }
            else if n >= I64_MIN && n <= I64_MAX { Type::I64 }
            else { report_error(IntegerTooLarge { value: n }); Type::Error }
        }
        Literal::Float(FloatSuffix::None) => Type::F64,
        Literal::Float(FloatSuffix::F) => Type::F32,
        Literal::Bool(_) => Type::Bool,
        Literal::String(_) => Type::String,  // v1.30
    }
}

fn is_compatible(target_type: Type, value: Expression) -> bool {
    match (target_type, value.type()) {
        (I32, I32) => true,
        (I64, I64) | (I64, I32) => true,  // I32 fits in I64
        (F64, F64) | (F64, F32) => true,  // F32 fits in F64
        (F32, F32) => true,
        (Bool, Bool) => true,
        (String, String) => true,
        _ => false,
    }
}
```

---

## Edge Cases

### Narrowing (I64 → I32)

```
// Explicit narrowing — VALID but requires check
BINA x: I32 = 1           // OK: 1 fits I32
BINA y: I32 = 100000      // OK: 100000 fits I32
BINA z: I32 = 9999999999  // ERROR: exceeds I32 range
```

### Float to Integer

```
// Float → Integer: NEVER allowed (even explicit)
BINA x: I32 = 3.14        // ERROR: F64 literal cannot be assigned to I32
BINA y: I64 = 3.14        // ERROR: F64 literal cannot be assigned to I64
// Must use explicit conversion: BINA x = cast<I32>(3.14)  // (future feature)
```

### Integer to Float

```
// Integer → Float: Allowed (lossless for small values)
BINA x: F64 = 42          // VALID: I32 fits exactly in F64
BINA y: F32 = 42          // VALID: I32 fits exactly in F32 (for small values)
```

---

## Malay vs Expert Syntax

Inference works identically in both syntaxes:

```
// Malay
BINA umur = 25                // I32
BINA gaji = 3500.50           // F64
BINA aktif = benar            // Bool
BINA nama = "Ahmad"           // String (v1.30)
BINA besar: I64 = 1           // I64 (explicit override)

// Expert
let umur = 25;                // I32
let gaji = 3500.50;           // F64
let aktif = true;             // Bool
let nama = "Ahmad";           // String (v1.30)
let besar: I64 = 1;           // I64 (explicit override)
```

---

## Implementation Status

| Phase | Status | Notes |
|-------|--------|-------|
| Design | ✅ Finalized | This document |
| RFC | ✅ Approved (by lead maintainer) | Authority: @msa-studio |
| Implementation | ⏳ Queued | Requires Phase 2 TYPE SYSTEM active |
| Tests | ⏳ Queued | After implementation |
| Documentation | ⏳ Queued | Update GETTING_STARTED.md after implementation |

---

## Changelog

| Date | Change | Authority |
|------|--------|-----------|
| 2026-05-31 | Initial rule finalized | @msa-studio |

---

> This is a finalized architecture decision, not an RFC proposal.
> Implementation is queued for Phase 2 TYPE SYSTEM.
> Rule is immutable unless amended via ROADMAP_POLICY.md §7 process.
