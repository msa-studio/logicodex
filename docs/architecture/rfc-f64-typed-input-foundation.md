# RFC: F64 Typed Input Foundation

**Status:** Owner-approved design; implementation authorization pending
**Baseline:** `origin/main` `e5b650168ef595376793897702bf784812d03099`
**Related:** FFI-C1 libm runtime proof

## Decision summary

Logicodex will support source-level decimal literals through a staged, governed foundation:

- Unsuffixed decimal literals default to `F64`.
- Explicit `f32` and `f64` suffixes are supported.
- Implicit integer-to-floating conversion is rejected, including FFI arguments.
- Explicit casts are deferred to a separate RFC.
- IEEE-754 behavior, diagnostics, HIR representation, LLVM lowering, and C ABI behavior require explicit evidence.
- `semantic_gate` remains the sole Meaning Authority.
- HIR remains the canonical execution path.
- LLVM remains the sole production backend.
- `LayoutEngine` remains the target-sensitive layout authority.
- Existing integer programs and field-access grammar must remain compatible.

This document records the owner-approved design contract only. It does not authorize implementation, commit, push, merge, or release of production code.

## Current baseline evidence

At `origin/main` SHA `e5b650168ef595376793897702bf784812d03099`:

- F32/F64 exist in the type system and LLVM mapping.
- The lexer has integer literals and an `F64` type token, but no decimal-literal token path.
- `LiteralAst` has integer, boolean, string, and unit variants; no float literal variant.
- `9.0` is currently rejected because `.` is field-access syntax.
- `sqrt(9)` with an `F64` parameter is rejected by the canonical `semantic_gate` path as `I64` → `F64`.
- The existing FFI coercion matrix must not bypass the semantic authority path.

## Proposed grammar

The first foundation is intentionally bounded:

```text
DECIMAL := DIGITS '.' DIGITS [SUFFIX]
SUFFIX  := 'f32' | 'f64'
DIGITS  := [0-9]+
```

Accepted examples:

```text
9.0       // default F64
9.0f64    // explicit F64
9.0f32    // explicit F32
0.5       // default F64
12.500f32 // explicit F32
```

The first foundation does not add leading-dot literals, trailing-dot literals, hexadecimal floating literals, exponent notation, `NaN`, `Infinity`, implicit conversions, or explicit casts.

Decimal recognition must require `.` followed immediately by a decimal digit. Existing identifier field access remains unchanged. Malformed numeric forms fail with source-backed diagnostics rather than being silently reinterpreted.

## IEEE-754 contract

The staged implementation plan must define and test:

- binary64 conversion for unsuffixed literals;
- binary32 conversion for `f32` literals;
- binary64 conversion for `f64` literals;
- round-to-nearest, ties-to-even unless existing project policy requires otherwise;
- signed zero behavior;
- subnormal handling;
- overflow and underflow behavior;
- rejection or explicit policy for non-finite values;
- exact deterministic behavior across supported targets.

Unresolved host conversion behavior is a blocker; it must not be guessed.

## Diagnostic contract

Malformed literals, overflow/non-finite literals, and forbidden conversions require exact:

- diagnostic code;
- bilingual Malay/English message;
- `Error` severity;
- complete source span;
- deterministic record order;
- fail-closed behavior with no backend artifact.

The implementation plan must decide whether a new `NumericLiteralError` code is required or whether the existing parser diagnostic owner is sufficient. Duplicate diagnostic authority is prohibited.

## Authority and data flow

```text
lexer decimal token
  → parser literal + source span
  → AST representation
  → HIR F32/F64 representation
  → semantic_gate typing and validation
  → LLVM F32/F64 constant lowering
  → LayoutEngine/C ABI evidence
```

`semantic.rs::Analyzer` remains `LegacyReferenceOnly`. Dormant/FutureReserved paths remain inactive. Enterprise/private semantics remain outside community compilation.

## Staged implementation boundary

- **F64-1:** lexer, parser, AST/HIR representation, spans, malformed-literal diagnostics.
- **F64-2:** semantic typing, default/suffix rules, overflow/underflow/special-value policy, exact diagnostics.
- **F64-3:** LLVM constants, target layout, F32/F64 C ABI evidence.
- **F64-4:** source-to-executable integration, regression matrix, and deferred FFI-C1 libm proof.

Each stage requires its own approved exact scope, focused validation, full integrity, implementation review, and owner transition. No later stage may be implemented early.

## Owner gate

Required status transition before F64-1 production edits:

```text
OWNER_APPROVED_DESIGN_PENDING_IMPLEMENTATION_AUTHORIZATION
→ IMPLEMENTATION_AUTHORIZED
```

Hermes plan approval alone does not grant this transition. Merge, release, and production deployment remain separately gated.

## Evidence references

- Repository audit: `/home/msastudio/awang/n8n-cpb/reports/f64-typed-input-audit-20261026/f64-typed-input-foundation-evidence.md`
- Revised RFC packet: `/home/msastudio/awang/external-review/requests/logicodex-f64-typed-input-rfc-revised-20261026.md`
- Staged plan packet: `/home/msastudio/awang/external-review/requests/logicodex-f64-typed-input-staged-plan-20261026.md`
