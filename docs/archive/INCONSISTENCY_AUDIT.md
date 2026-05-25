# Inconsistency Audit: All Documents vs WHITE_PAPER.md Baseline

**Audit Date:** 2026-05-25  
**Auditor:** AI-assisted systematic review  
**Baseline:** `WHITE_PAPER.md` (v1.21 formal specification + v1.45 evolution notice)  
**Scope:** All 25+ Markdown documents in the repository  
**Severity:** Critical / Moderate / Minor

---

## Executive Summary

| Severity | Count | Description |
|---|---|---|
| **CRITICAL** | 6 | Internal contradictions, version mismatches, stale data that misleads readers |
| **MODERATE** | 4 | Unsubstantiated claims, outdated terminology, missing evidence |
| **MINOR** | 3 | Terminology drift, formatting inconsistency, missing cross-references |
| **TOTAL** | **13** | |

> **Finding:** The most significant inconsistencies are **internal contradictions within README.md** (3 instances) and **stale status in ROADMAP.md** (2 instances). The baseline WHITE_PAPER.md itself is consistent after the recent alignment update. The wiki documents are internally consistent but contain one unsubstantiated claim.

---

## CRITICAL (6 items)

### C1: README.md — Validation Count Contradiction (102 vs 148)

**Location:** `README.md` line 131  
**Severity:** Critical  
**Type:** Internal contradiction

**Evidence:**
```markdown
# Line 19 in header: "Validators: 148/148 ✅"
# Line 131 in validation section: "102/102 checks passing — zero regression"
```

**Problem:** The README header claims 148/148 validators passing (correct for v1.45), but the detailed validation section below still shows the old v1.41 count of 102/102. A reader who reads the header gets a different number from a reader who reads the validation table.

**Impact:** Readers cannot trust the validator count. Contributors may report "missing validators" or think the count is wrong.

**Fix:** Update the validation section to 148/148 with the v1.42–v1.45 entries included.

---

### C2: README.md — Footer Version Mismatch (v1.41 vs v1.45)

**Location:** `README.md` line 205–207  
**Severity:** Critical  
**Type:** Version mismatch

**Evidence:**
```markdown
# Line 1: "# Logicodex Language — v1.45.0-alpha"
# Line 205: "*Logicodex Language — v1.41.0-alpha*"
```

**Problem:** The document footer says v1.41.0-alpha while the header says v1.45.0-alpha. This makes readers question which version the document actually describes.

**Impact:** Credibility damage — the project's most visible document contradicts itself.

**Fix:** Change footer to `v1.45.0-alpha`.

---

### C3: README.md — Total LOC Undercount (~37,500 vs ~43,600)

**Location:** `README.md` line 166  
**Severity:** Critical  
**Type:** Stale metric

**Evidence:**
```markdown
# README line 166: "TOTAL: 139 files, ~37,500"
# Actual count: ~43,600 (verified by wc -l across all tracked files)
```

**Problem:** The LOC count excludes the wiki documentation (~10,500 LOC), the CHANGELOG_ANALYSIS, and other v1.42+ documents. The file count (139) also doesn't include the 40+ new wiki files.

**Impact:** Under-represents project scale. Potential contributors see a smaller project than reality.

**Fix:** Update to actual counts: ~43,600 LOC, ~180+ files.

---

### C4: AUDIT.md — Never Updated Beyond v1.41

**Location:** `docs/AUDIT.md` line 2–5  
**Severity:** Critical  
**Type:** Stale document

**Evidence:**
```markdown
# Line 2: "# Logicodex Project Audit — v1.41.0-alpha"
# Line 4: "**Version**: v1.41.0-alpha"
# Line 6: "**Total LOC** | ~37,566"
# Line 22: "**Releases** | v1.21 → v1.41 (10 alpha releases)"
```

**Problem:** The full project audit document was written at v1.41 and never updated for v1.42 (Raylib FFI), v1.43 (Raylib Audio), v1.44 (Freestanding), v1.44.1 (Maintenance), or v1.45 (Benchmarks). It reports 102 validators (v1.41) instead of 148 (v1.45), 10 releases instead of 14, and ~37,566 LOC instead of ~43,600.

**Impact:** This is a referenced document (`README.md` line 174 points to it as "Full project audit"). Readers who follow this link get severely outdated information.

**Fix:** Either (a) update AUDIT.md for v1.45, or (b) add a prominent header noting it's the v1.41 audit and refer readers to CHANGELOG.md for current data.

---

### C5: ROADMAP.md — Milestone 4: WASM Still "Long-term" (Completed v1.40)

**Location:** `ROADMAP.md` lines 127–135  
**Severity:** Critical  
**Type:** Stale status

**Evidence:**
```markdown
# Line 133: "| Issue #11 — WebAssembly prototype | Long-term objective | TBD | ..."
# Line 134: "| Issue #12 — Cross-platform benchmark harness | Long-term objective | TBD | ..."
```

**Problem:** Milestone 4 lists WebAssembly prototype and cross-platform benchmark as "Long-term objective" with "TBD" owner. But these were completed in v1.40 (WASM Backend) and v1.45 (Benchmark Framework) respectively. The same ROADMAP.md document correctly marks v1.40 and v1.45 as COMPLETED in other sections.

**Impact:** Direct contradiction within the same document. Readers see v1.40 WASM marked COMPLETED in one table and "Long-term objective" in another.

**Fix:** Update Milestone 4 Issue #11 to "COMPLETED v1.40" and Issue #12 to "COMPLETED v1.45".

---

### C6: ROADMAP.md — Milestone 1 Status Table Outdated

**Location:** `ROADMAP.md` lines 34–41  
**Severity:** Critical  
**Type:** Stale status

**Evidence:**
```markdown
# Line 41: "| Issue #04 — CI-oriented validation | Open | TBD | ..."
# Line 113: "| Issue #05 — Nominal type system boundaries | Open | TBD | ..."
# Line 114: "| Issue #06 — Pointer and hardware-region gates | Open | TBD | ..."
```

**Problem:** Several "Open" issues in Milestone 1 and 2 were actually completed in later releases but never updated:
- Issue #04 CI validation — validator pipeline (Tier A/B/C) completed in v1.44.1
- Issue #06 Pointer/hardware gates — implemented in v1.44 freestanding

**Impact:** Milestones appear incomplete when they are actually done.

**Fix:** Cross-reference all "Open" items with actual implementation status and update accordingly.

---

## MODERATE (4 items)

### M1: ARCHITECTURE.md — Unsubstantiated Absolute Claim

**Location:** `docs/ARCHITECTURE.md` line 20  
**Severity:** Moderate  
**Type:** Overclaim

**Evidence:**
```markdown
> **"Mustahil untuk mengalami race condition atau memory leak"**
> — kerana semuanya diverifikasi pada masa kompil.
```

**Baseline Check:** WHITE_PAPER.md (Section 15) is careful to use qualifiers:
```markdown
"These tiers are documented as an engineering target so diagnostics and 
mitigation paths can be implemented, tested, and benchmarked before any 
measured-overhead or production-readiness claim is made."
```

**Problem:** The claim "Mustahil" (impossible) is an absolute statement. While the architecture strongly prevents race conditions through static topology and shard isolation, "impossible" cannot be proven — only "not yet observed under test conditions." The baseline white paper uses cautious language; the architecture doc does not.

**Impact:** If a race condition is ever found (e.g., in unsafe FFI code), this claim becomes false and damages credibility.

**Fix:** Change to: *"Race condition dan memory leak dicegah pada masa kompil melalui static topology verification dan shard isolation — tiada kejadian dilaporkan dalam 148 validator checks dan 400+ tests."* (Prevented at compile time — none reported in 148 checks and 400+ tests.)

---

### M2: v1.30-THREADING.md — Uses Pre-Rename Malay Keywords

**Location:** `docs/v1.30-THREADING.md` lines 14–16  
**Severity:** Moderate  
**Type:** Outdated terminology

**Evidence:**
```markdown
- **Kotak → Actor** = satu unit konkurensi (1 OS thread)
- **Pintu → Channel** = saluran komunikasi SPSC
```

**Context:** PR #29 (v1.30.1) renamed all internal concurrency keywords from Malay to English (`kotak`→`actor`, `pintu`→`channel`, `lahirkan`→`spawn`). The document title even says "v1.30.1-alpha" but still uses the old Malay terms as primary identifiers.

**Problem:** The document presents the renamed keywords as if Malay terms are the primary interface. New readers may try to use `kotak` and `pintu` in code, which will not work.

**Fix:** Add a prominent note: *"⚠️ Keywords were internationalized in PR #29. Internal AST uses English (`actor`, `channel`, `spawn`). Malay aliases remain available via `core_map.json`."*

---

### M3: GETTING_STARTED.md — "Compile-time Null Checks" Claim

**Location:** `docs/GETTING_STARTED.md` line 44  
**Severity:** Moderate  
**Type:** Unsubstantiated feature claim

**Evidence:**
```markdown
| Null pointer crashes | Compile-time null checks |
```

**Baseline Check:** WHITE_PAPER.md mentions "Name resolution", "Type inference", "Type checking", "Constant-folding checks", "Memory capability checks", and "FFI signature validation" — but **never mentions null pointer checks** as a compile-time feature.

**Problem:** Null pointer checking is not documented in any compiler document (not in semantic.rs, not in any validator, not in any test). The `Option<T>` type exists, but there's no evidence of automatic null-check elimination at compile time.

**Impact:** Beginners may expect the compiler to catch null dereferences, which it may not.

**Fix:** Either (a) implement and validate null pointer checks, or (b) change the claim to: *"No null pointers by design — use `Option<T>` for nullable values."*

---

### M4: README.md — Documentation Count Excludes Wikis

**Location:** `README.md` lines 163–167  
**Severity:** Moderate  
**Type:** Incomplete inventory

**Evidence:**
```markdown
| Documentation (`docs/` + root) | 17 | 3,578 |
```

**Problem:** The documentation count excludes the two new wikis (`docs/white-paper/` and `docs/guide/`) which contain 40 files and ~10,500 LOC. These are significant documentation assets.

**Impact:** The project appears to have less documentation than it actually does.

**Fix:** Update to: `| Documentation | 17 + 2 wikis (40 files) | 3,578 + 10,500 |`

---

## MINOR (3 items)

### m1: Terminology: "Alpha" vs "-alpha" Inconsistency

**Location:** Multiple documents  
**Severity:** Minor  
**Type:** Formatting

**Evidence:**
```markdown
WHITE_PAPER.md:   "v1.21-alpha" (with hyphen)
README.md:        "v1.45.0-alpha" (with hyphen and patch)
CHANGELOG.md:     "v1.30.1-alpha" (with patch)
ARCHITECTURE.md:  "v1.32.0-alpha" (full semver)
AUDIT.md:         "v1.41.0-alpha" (full semver)
```

**Problem:** Early documents use `v1.21-alpha` while later documents use `v1.45.0-alpha`. Not a contradiction but inconsistent formatting.

**Fix:** Standardize to full semver `v{MAJOR}.{MINOR}.{PATCH}-alpha` in all future updates.

---

### m2: Wiki book.toml — Language Code "ms" May Not Render

**Location:** `docs/white-paper/book.toml` and `docs/guide/book.toml`  
**Severity:** Minor  
**Type:** Technical

**Evidence:**
```toml
language = "ms"
```

**Problem:** MDBook's language field is typically used for `en`, `zh`, etc. "ms" (Malay) is a valid ISO code but may not be recognized by all MDBook themes for localization features.

**Fix:** Test `mdbook build` to confirm rendering. If issues occur, use `language = "en"` with Malay content (content language is independent of UI language).

---

### m3: Missing Cross-Reference: CHANGELOG_ANALYSIS Not Listed in README

**Location:** `README.md` Documentation table  
**Severity:** Minor  
**Type:** Missing reference

**Evidence:** The new `docs/CHANGELOG_ANALYSIS_v121_to_v145.md` document is not listed in README.md's documentation table.

**Fix:** Add entry: `| CHANGELOG_ANALYSIS_v121_to_v145.md | Comprehensive change analysis from v1.21 to v1.45 |`

---

## Alignment Verification: Baseline vs Reality

After identifying inconsistencies, we verified that the baseline WHITE_PAPER.md's claims are **accurately reflected** in the implementation:

| Baseline Claim (WHITE_PAPER.md) | Implementation Status | Verified? |
|---|---|---|
| "Phase 1 delivers the working core compiler infrastructure" | v1.21: lexer, parser, AST, semantic, LLVM | ✅ Yes |
| "Capability-marked memory regions" (`KAWASAN_PERKAKAS`/`hw`) | v1.44: `emit_hardware_zone()` + `emit_mmio_volatile_write/read()` | ✅ Yes |
| "Freestanding `_start` object profile" | v1.44: `src/os/startup.rs` with `_start`, BSS zero, data copy | ✅ Yes |
| "Runtime Memory Integrity Verification Engine" | v1.38 G1: `--secure` flag + `compute_module_hash` SHA-256 placeholder | ⚠️ Partial — placeholder only, not continuous attestation |
| "Golden Hash memory integrity planning" | v1.38 G1: Security plan document generation | ⚠️ Partial — planning only, not runtime verification |
| "WebAssembly target" | v1.40: `wasm32-unknown-unknown` with 3 LLVM features | ✅ Yes |
| "Logicodex Migrator Engine" | Not implemented | ✅ Correctly marked LONG-TERM |
| "Pointer Provenance Engine" | v1.21 baseline: Level 1 (ownership). Levels 2-5: RESEARCH | ✅ Correctly tiered |

**Conclusion:** The baseline WHITE_PAPER.md does not overclaim. Items marked as "long-term objectives" that are now COMPLETED have been validated. Items still marked LONG-TERM are indeed not yet implemented.

---

## Recommended Fixes Priority

| Priority | Item | Effort | File |
|---|---|---|---|
| **P0** | C2: Fix footer version | 1 line | `README.md` |
| **P0** | C1: Fix validation count 102→148 | ~10 lines | `README.md` |
| **P0** | C4: Add stale notice to AUDIT.md | 3 lines | `docs/AUDIT.md` |
| **P1** | C5: Fix ROADMAP.md Milestone 4 status | 2 lines | `ROADMAP.md` |
| **P1** | C6: Fix ROADMAP.md Milestone 1 status | 3 lines | `ROADMAP.md` |
| **P1** | C3: Update README LOC counts | 5 lines | `README.md` |
| **P2** | M1: Soften ARCHITECTURE.md absolute claim | 1 line | `docs/ARCHITECTURE.md` |
| **P2** | M2: Add rename note to v1.30 doc | 3 lines | `docs/v1.30-THREADING.md` |
| **P2** | M3: Fix null check claim | 1 line | `docs/GETTING_STARTED.md` |
| **P2** | M4: Add wikis to doc count | 1 line | `README.md` |
| **P3** | m1-m3: Formatting fixes | 5 lines | Multiple |

**Total effort:** ~35 lines across 6 files — approximately 15 minutes of work.

---

*Audit methodology: Systematic cross-reference of all claims in 25+ Markdown files against the baseline WHITE_PAPER.md, README.md, ROADMAP.md, ARCHITECTURE.md, AUDIT.md, CHANGELOG.md, GETTING_STARTED.md, 7 version-specific docs, and 40 wiki files. Each claim verified against source code, validator output, and git history.*
