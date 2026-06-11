# HIR Decision (Phase 1 / Issue #02): ACTIVATE

**Decision:** ACTIVATE — the HIR is the compiler's sole execution path.
**Date:** 2026-06-10  **Phase:** 1 (HARDEN)  **Deliverable:** P1-D6 / P1-A6

## Context
The Maturity Matrix (v1.45) recorded the compiler pipeline as PARTIAL ("HIR exists
but is not on execution path") and HIR lowering as SKELETON/dormant. Two codegen
paths coexisted: a v1.21 AST-based emitter and a dormant HIR-based emitter. A
separate v1.21 Analyzer ran as an early semantic gate.

## Decision
Activate HIR as the single engine. Rationale:
- One pipeline is cheaper to maintain than two divergent ones.
- HIR is the natural home for the v1.30 type system (structs, enums, named types)
  that the AST emitter could not express.
- Removing the dual-path ambiguity eliminates a class of "which path ran?" bugs.

## What was implemented
Source now compiles: `.ldx` → Lexer → Parser → AST → **HIR lowering** →
**semantic_gate** → LLVM IR → native binary. Working end-to-end:
variables/assignment, functions/params/recursion, arithmetic/comparisons,
if-else, while, loop, break/continue, **structs** (construct, field read/write,
struct params), **enums + pattern matching** (`Enum::Variant` tags, `MATCH`
lowered to an if/else chain, wildcard).

## What was removed / retired
- v1.21 AST codegen (~646 lines): `compile_to_object`, `emit_program`/`emit_stmt`/
  `emit_expr`, hardware-zone & MMIO emitters (compiler-verified dead).
- v1.21 `Analyzer` retired from the pipeline; its still-valuable checks
  (duplicate-function, division-by-zero) ported into `semantic_gate`. `check`
  rewired to full v1.30 validation. Module kept for reference.
- `CompilerPipeline::V121` variant removed (`--pipeline v1.21` is now a deprecated
  alias for the single path). `lower_v121_program` → `lower_program`.

## Evidence
- Representative programs run correctly (e.g. struct field math → 30/1/101;
  enum match in a function with wildcard → 1/2/99/99).
- Dead-code removal was compiler-guided (orphan → `dead_code` → delete); shared
  helpers retained. Zero regression across struct/enum/match/function suites.

## Consequences
- Issue #02 is resolved (ACTIVATE).
- Maturity Matrix updated: Compiler pipeline PARTIAL→FULL, HIR lowering SKELETON→FULL.
- Phase 2-5 must not reopen the HIR decision (Architecture Freeze §3).

## Follow-up work completed after the decision (v1.46)
- Call return-type inference wired (Call expressions carry their callee's return type).
- Struct returns made sound via the sret ABI (caller-allocated buffer; no dangling pointers).
- Parser supports chained postfix field access (`buat().x`, `a.b.c`).
- Fixed-width integers (all 8 widths) with true wrapping at every boundary (init, assignment, fields, per-op arithmetic, params, returns, call results); uniform i64 representation retained.

## Known stopgaps (tracked, not blockers)
- Method calls on expression results (`buat().m()`) not yet supported.
