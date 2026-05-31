# RFC-002: Literal Type Inference

**Status:** PROPOSED  
**Phase:** 2 TYPE SYSTEM (v1.50.x)  
**Author:** msa-studio  
**Date:** 2026-05-31  
**Governance:** Per ROADMAP_POLICY.md §3.2 — RFC required for type system changes  
**Branch:** `feat/rfc-002-type-inference`

---

## 1. Summary

Allow the Logicodex compiler to automatically infer the type of a variable from its literal value at declaration, eliminating the need for explicit type annotations in common cases.

**Before (v1.21):**
```
BINA x: I32 = 1
BINA y: I64 = 9999999999
BINA z: F64 = 3.14
```

**After (v1.30+):**
```
BINA x = 1              // compiler infers I32
BINA y = 9999999999     // compiler infers I64 (auto-upgrades!)
BINA z = 3.14           // compiler infers F64
```

---

## 2. Motivation

### 2.1 Problem: Boilerplate

In v1.21, every variable declaration requires an explicit type annotation:
```
BINA a: I32 = 0
BINA b: I32 = 1
BINA c: I32 = 2
```

This is verbose and repetitive. The compiler can trivially determine that `1` fits in `I32`, `9999999999` requires `I64`, and `3.14` is a float.

### 2.2 Problem: Silent Overflow Risk

A beginner might write:
```
BINA x: I32 = 9999999999    // COMPILE ERROR (I32 overflow)
```

With type inference, the compiler automatically assigns `I64` because the value exceeds `I32` range — preventing a frustrating error for a common case.

### 2.3 Benefit: Beginner Friendly

Type inference reduces cognitive load for new users. They can write:
```
BINA count = 0
BINA name = "Ahmad"
BINA pi = 3.14159
```

Without needing to understand `I32` vs `I64` vs `F64` on day one.

---

## 3. Design

### 3.1 Inference Rules

| Literal Form | Inferred Type | Range Check | Example |
|-------------|---------------|-------------|---------|
| Integer (no suffix) | `I32` | `-2,147,483,648` to `2,147,483,647` | `BINA x = 42` → `I32` |
| Integer (no suffix) | `I64` | `-9,223,372,036,854,775,808` to `9,223,372,036,854,775,807` | `BINA y = 9999999999` → `I64` |
| Integer (overflow) | **COMPILE ERROR** | Exceeds `I64` | `BINA z = 99999999999999999999` → Error |
| Float (no suffix) | `F64` | Any valid float | `BINA a = 3.14` → `F64` |
| Float (`f` suffix) | `F32` | Explicit | `BINA b = 3.14f` → `F32` |
| Boolean | `Bool` | `benar`/`palsu` or `true`/`false` | `BINA c = benar` → `Bool` |
| String (v1.30) | `String` | Any string literal | `BINA d = "hello"` → `String` |

### 3.2 Explicit Override

Explicit type annotations **always** override inference:
```
BINA x: I64 = 1     // I64, not I32 — programmer's choice
BINA y: F32 = 3.14  // F32, not F64 — with precision warning
```

### 3.3 Type Immutability After Inference

Once inferred (or explicitly set), the type is **fixed**:
```
BINA x = 1          // I32 (inferred)
x = 9999999999      // COMPILE ERROR: I32 overflow
                    // Must redeclare: BINA x: I64 = 9999999999
```

### 3.4 Malay vs Expert Syntax

Both syntaxes support inference identically:
```
// Malay
BINA umur = 25
BINA gaji = 3500.50

// Expert
let umur = 25;
let gaji = 3500.50;
```

---

## 4. Implementation Plan

### 4.1 Where to Add Logic

**Stage:** Semantic analysis (`src/semantic.rs`)

When the semantic analyzer encounters a variable declaration **without** an explicit type, it checks the literal value and assigns the inferred type.

### 4.2 Pseudocode

```rust
// In semantic.rs, variable declaration handler
fn infer_type_from_literal(literal: &Literal) -> Type {
    match literal {
        Literal::Integer(value) => {
            if *value >= I32_MIN && *value <= I32_MAX {
                Type::I32
            } else if *value >= I64_MIN && *value <= I64_MAX {
                Type::I64
            } else {
                // Error: integer literal too large
                report_error(LiteralTooLarge { value, max: I64_MAX });
                Type::Error
            }
        }
        Literal::Float(_) => {
            Type::F64  // default float precision
        }
        Literal::FloatWithSuffix('f') => {
            Type::F32
        }
        Literal::Bool(_) => {
            Type::Bool
        }
        Literal::String(_) => {
            Type::String  // v1.30 only
        }
    }
}
```

### 4.3 Files to Modify

| File | Change | Est. Lines |
|------|--------|------------|
| `src/semantic.rs` | Add `infer_type_from_literal()` + wire into var declaration | ~80 |
| `src/parser.rs` | Support `f` suffix on float literals | ~20 |
| `src/ast.rs` | Add `Literal::FloatWithSuffix` variant | ~10 |
| `tests/type_inference.ldx` | Test cases | ~50 |
| `GETTING_STARTED.md` | Document inference rules | ~20 |

**Total: ~180 lines, 1-2 weeks.**

### 4.4 Test Cases

```
// Test: I32 inference
BINA a = 1
PAPAR a            // expect: 1 (I32)

// Test: I64 auto-upgrade
BINA b = 9999999999
PAPAR b            // expect: 9999999999 (I64)

// Test: F64 inference
BINA c = 3.14159
PAPAR c            // expect: 3.14159 (F64)

// Test: F32 suffix
BINA d: F32 = 2.5f
PAPAR d            // expect: 2.5 (F32)

// Test: Bool inference
BINA e = benar
PAPAR e            // expect: 1 (Bool)

// Test: explicit override
BINA f: I64 = 1
PAPAR f            // expect: 1 (I64, not I32)

// Test: error case
BINA g = 99999999999999999999   // COMPILE ERROR: too large
```

---

## 5. Alignment Checks

### 5.1 Phase Alignment

| Question | Answer |
|----------|--------|
| Is this in current phase scope? | Phase 2 = TYPE SYSTEM. Type inference is a type system feature. **✅ Aligned.** |

### 5.2 Component Maturity Alignment

| Question | Answer |
|----------|--------|
| Does the semantic analyzer exist? | Yes, `src/semantic.rs` with 32 error variants. **✅ Ready.** |
| Does the type system exist? | Yes, I32/I64/F32/F64/Bool types. **✅ Ready.** |

### 5.3 Dependency & Ecosystem Alignment

| Question | Answer |
|----------|--------|
| New dependencies? | No. Pure semantic analysis change. **✅ No deps.** |
| Breaking change? | No. Explicit types still work. Backward compatible. **✅ Safe.** |

### 5.4 Cost/Benefit Alignment

| Cost | Benefit |
|------|---------|
| ~180 lines, 1-2 weeks | Beginner-friendly, less boilerplate, overflow protection |
| **Low cost, high value → ✅ Worth it.** |

---

## 6. Risks

| Risk | Severity | Mitigation |
|------|----------|------------|
| User confusion ("Why is my variable I32 not I64?") | Low | Document inference rules in GETTING_STARTED.md |
| Mixed codebase (some auto, some explicit) | Low | Code style guide recommendation (not enforced) |
| Float precision surprise (`3.14` → F64, expected F32) | Low | Suffix `f` available for F32 override |

---

## 7. Decision

- [ ] **GO** — Approve, proceed with implementation
- [ ] **NO-GO** — Decline, keep explicit types only
- [ ] **CONDITIONAL GO** — Approve with modifications [specify]

---

## Related Documents

| Document | Purpose |
|----------|---------|
| `ROADMAP_v2.md` | Phase 2 TYPE SYSTEM scope |
| `ROADMAP_POLICY.md` | RFC process (§3.2) |
| `GETTING_STARTED.md` | Syntax reference (to be updated) |
| `V130_OPTION_ENGINE.md` | v1.30 capabilities context |

---

> This RFC is proposed for Phase 2 TYPE SYSTEM. It does not authorize implementation until Phase 1 exit audit is complete and Phase 2 is active.
