<!--
  PHASE TRANSITION AUDIT TEMPLATE
  Logicodex Language Project

  INSTRUCTIONS:
  1. Copy this file to: .roadmap/audits/YYYY-MM-DD_phase-N-to-phase-M_audit.md
  2. Fill in ALL sections completely. Partial audits are REJECTED.
  3. Attach evidence links (CI runs, commit ranges, test reports).
  4. Obtain both signatures BEFORE transition.
  5. Open a PR adding the completed audit file for maintainer review.

  This audit MUST be completed and approved before ANY phase transition.
  No exceptions. No shortcuts.
-->

# Phase Transition Audit: `{SOURCE_PHASE}` → `{TARGET_PHASE}`

> **Status:** Active phase-transition audit template. Resolve current phase and
> work sequence from [`current-authority.md`](../docs/architecture/current-authority.md);
> use [`ROADMAP_v2.md`](../ROADMAP_v2.md) only for long-horizon target criteria
> and history.

---

## 1. Audit Metadata

| Field | Value |
|-------|-------|
| **Audit Date** | `YYYY-MM-DD` |
| **Source Phase** | `{SOURCE_PHASE}` and version resolved from current authority / `Cargo.toml` |
| **Target Phase** | `{TARGET_PHASE}` (e.g., next approved roadmap phase) |
| **Auditor** | `@github-username` |
| **Auditor Role** | `Core Maintainer / Technical Lead / Release Engineer / External Auditor` |
| **Target Commit Hash** | `aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa` (full SHA of commit being audited) |
| **Previous Phase Commit** | `bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb` (full SHA of last commit in source phase) |
| **CI Run Link** | `https://github.com/logicodex/logicodex/actions/runs/RUN_ID` |
| **Previous Audit** | Link to prior phase audit file (or "N/A — first transition") |
| **Days in Source Phase** | `NNN` |

### Version Under Audit

| Component | Version |
|-----------|---------|
| logicodex crate | `vX.Y.Z[-tag]` |
| Rust toolchain | `1.XX` |
| LLVM | `15.x` |
| CI check count | `NNN/NNN` |

---

## 2. Phase Entry Criteria Checklist

<!--
  Copy the owner-approved entry criteria for the TARGET phase. Resolve active
  sequence from docs/architecture/current-authority.md and long-horizon target
  criteria from ROADMAP_v2.md.
  Every criterion MUST have evidence attached.
  Mark each item: [ ] Pending / [~] Partial / [x] Complete / [!] Blocked
-->

### 2A. General Entry Criteria (All Transitions)

| # | Criterion | Status | Evidence Link | Notes |
|---|-----------|--------|---------------|-------|
| G1 | Required CI stability evidence window is green per target criteria | [ ] | CI run link | |
| G2 | Zero critical security vulnerabilities (dependabot, cargo-audit) | [ ] | Audit report link | |
| G3 | Architecture change-control compliance: no unauthorized invariant changes | [ ] | Gatekeeper run log | |
| G4 | RFC process documented and tooling ready | [ ] | CONTRIBUTING.md §RFCs | |
| G5 | Team capacity confirmed for target phase scope | [ ] | Team capacity doc / comment | |

### 2B. Transition-Specific Criteria

| # | Criterion | Status | Evidence Link | Notes |
|---|-----------|--------|---------------|-------|
| P1 | Source-phase exit criteria and owner-locked sequence items are complete | [ ] | Current authority, closing evidence | |
| P2 | Canonical quick and full-integrity gates pass at the audited commit | [ ] | CI run and local evidence | |
| P3 | Architecture change control is satisfied; no unapproved invariant change exists | [ ] | Gatekeeper logs, RFC if applicable | |
| P4 | Documentation, lifecycle, version, and provenance records match live behavior | [ ] | Documentation diff, validators | |
| P5 | Residual debt is classified with explicit ownership and blocking disposition | [ ] | Current authority, debt records | |
| P6 | The owner explicitly approves entry into `{TARGET_PHASE}` | [ ] | Approval record | |

### 2C. Target Phase Readiness

| # | Criterion | Status | Evidence Link | Notes |
|---|-----------|--------|---------------|-------|
| T1 | Target scope, entry criteria, and exit criteria are documented | [ ] | Approved roadmap/authority record | |
| T2 | Required labels, validators, and CI gates are available | [ ] | Gatekeeper and CI evidence | |
| T3 | Any architecture-controlled work has approved RFC evidence before implementation | [ ] | RFC / owner approval | |
| T4 | Dependencies and owner capacity for the target scope are confirmed | [ ] | Dependency and capacity record | |

### Criteria Summary

- **Complete:** `N` / **Total:** `M`
- **Blocked items:** (list any `!` items and escalation plan)
- **Waivers requested:** (none, or list with justification)

---

## 3. Deliverables Verification

<!--
  For each deliverable expected from the source phase, provide:
  - Location (path, URL, release asset)
  - Verification method (checksum, CI artifact, manual inspection)
  - Quality assessment
-->

### 3A. Code Deliverables

| # | Deliverable | Location | Verification | Quality | Status |
|---|-------------|----------|--------------|---------|--------|
| 1 | Release binary (`logicodex`) | `target/release/logicodex` / GitHub Release | CI build artifact | `stripped size, file output` | [ ] |
| 2 | Test suite (all tiers) | `tests/`, `scripts/validators/` | `cargo test` + validator scripts | `NNN/NNN pass` | [ ] |
| 3 | Example programs | `examples/*.ldx` | Example CI job | `NN examples pass` | [ ] |
| 4 | Phase-specific compiler deliverables | `{paths}` | Owning validators | Acceptance criteria met | [ ] |
| 5 | Stress/fuzz evidence (when required by target criteria) | `{artifact/path}` | Reproducible command | Target-specific threshold met | [ ] |

### 3B. Documentation Deliverables

| # | Deliverable | Location | Verification | Review Status | Status |
|---|-------------|----------|--------------|---------------|--------|
| 1 | Language reference | `docs/lang-ref/` | Coverage report | Tech-writer review | [ ] |
| 2 | Architecture docs | `docs/architecture/` | Coverage report | Peer review | [ ] |
| 3 | API documentation | `cargo doc --no-deps` output | CI doc build | `rustdoc` clean | [ ] |
| 4 | Migration guide (previous stable baseline → current baseline) | `docs/migrations/<previous>-to-<current>.md` | Manual inspection | Published | [ ] |
| 5 | Current authority reconciled | `docs/architecture/current-authority.md` | Authority validator | Accurate, owner-approved | [ ] |
| 6 | Long-horizon roadmap reconciled when affected | `ROADMAP_v2.md` | Git diff | No competing work sequence | [ ] |
| 7 | CHANGELOG.md | `CHANGELOG.md` | Version diff | All entries accounted | [ ] |

### 3C. Process Deliverables

| # | Deliverable | Location | Verification | Status |
|---|-------------|----------|--------------|--------|
| 1 | RFC template published | `.github/RFC_TEMPLATE.md` or GitHub Discussions | Exists, linked from CONTRIBUTING.md | [ ] |
| 2 | Alignment checks framework | `.github/ALIGNMENT_CHECKS.md` | 4 checks documented | [ ] |
| 3 | Gatekeeper CI active | `.github/workflows/gatekeeper.yml` | Merged, running, logs exist | [ ] |
| 4 | Issue triage complete | GitHub Issues | Zero untriaged issues >30 days old | [ ] |

### Deliverables Summary

- **Verified:** `N` / **Total:** `M`
- **Missing / Incomplete:** (list with remediation plan or waiver)

---

## 4. Test Results

### 4A. Automated Test Suite

| Test Tier | Count | Pass | Fail | Skip | Status |
|-----------|-------|------|------|------|--------|
| Unit tests (`cargo test`) | NNN | NNN | 0 | 0 | [ ] |
| Tier A validators (core) | NN | NN | 0 | — | [ ] |
| Tier B validators (feature) | NN | NN | 0 | — | [ ] |
| Formatting check (`cargo fmt`) | 1 | 1 | 0 | — | [ ] |
| Lint check (`cargo clippy`) | 1 | 1 | 0 | — | [ ] |
| Example compilation | NN | NN | 0 | — | [ ] |
| Target-phase focused gates | NN | NN | 0 | — | [ ] |
| **TOTAL** | **NNN** | **NNN** | **0** | **0** | **PASS / FAIL** |

### 4B. CI Stability History

| Window | Required | Actual | Status |
|--------|----------|--------|--------|
| Required transition evidence window | `{target criterion}` | `NN days` | [ ] |

### 4C. Fuzzing & Stress Results

| Test Suite | Iterations | Crashes | ICEs | Status |
|------------|------------|---------|------|--------|
| Required fuzz/stress suite | `{iterations}` | `{crashes}` | `{ICEs}` | [ ] |
| Validator false-positive rate (if required) | `{sample size}` | `{target}` | `{actual}` | [ ] |

### 4D. Performance Regression

| Benchmark | Baseline | Current | Delta | Threshold | Status |
|-----------|----------|---------|-------|-----------|--------|
| Compile time (hello.ldx) | `NN.Ns` | `NN.Ns` | `+X%` | `{target threshold}` | [ ] |
| Binary size (hello.ldx) | `NN KB` | `NN KB` | `+X%` | `{target threshold}` | [ ] |
| Memory usage (check 1KLOC) | `NN MB` | `NN MB` | `+X%` | `{target threshold}` | [ ] |

### 4E. CI Links

| Check | Latest Run | History |
|-------|------------|---------|
| CI (main) | `[Run #NNNN](URL)` | `[History](URL)` |
| Nightly | `[Run #NNNN](URL)` | `[History](URL)` |
| Gatekeeper | `[Run #NNNN](URL)` | `[History](URL)` |

---

## 5. Documentation Verification

### 5A. Documentation Completeness Matrix

| Document | Updated | Covers Target Phase | Reviewed | Location |
|----------|---------|---------------------|----------|----------|
| README.md | [ ] | N/A (general) | [ ] | `/README.md` |
| Current authority | [ ] | Yes — phase, sequence, and debt disposition | [ ] | `/docs/architecture/current-authority.md` |
| ROADMAP_v2.md | [ ] | Long-horizon target criteria/history when affected | [ ] | `/ROADMAP_v2.md` |
| SPECIFICATION.md | [ ] | Target surface reconciled | [ ] | `/SPECIFICATION.md` |
| CONTRIBUTING.md | [ ] | N/A | [ ] | `/CONTRIBUTING.md` |
| Language reference | [ ] | Approved target coverage criterion | [ ] | `/docs/lang-ref/` |
| Architecture docs | [ ] | Audited live state | [ ] | `/docs/architecture/` |
| Migration guide | [ ] | `{previous}` → `{target}` when required | [ ] | `/docs/migrations/` |
| CHANGELOG.md | [ ] | N/A (historical) | [ ] | `/CHANGELOG.md` |
| API docs (rustdoc) | [ ] | Yes — `cargo doc` clean | [ ] | CI artifact |

### 5B. Documentation Quality Gates

| Gate | Required | Result | Status |
|------|----------|--------|--------|
| No broken internal links | Yes | `linkchecker` output | [ ] |
| No broken external links | Yes | `linkchecker` output | [ ] |
| rustdoc builds warning-free | Yes | `cargo doc` log | [ ] |
| Code examples in docs compile | Yes | `rustdoc --test` or manual | [ ] |
| Spelling/grammar check | Yes | `typos` or manual review | [ ] |

### 5C. Documentation Sign-Off

| Role | Reviewer | Date | Approved |
|------|----------|------|----------|
| Technical accuracy | `@username` | `YYYY-MM-DD` | [ ] |
| Tech-writer review | `@username` | `YYYY-MM-DD` | [ ] |
| User-facing clarity | `@username` | `YYYY-MM-DD` | [ ] |

---

## 6. Known Issues

<!--
  List everything that is NOT done and WHY.
  Honesty here prevents surprises later.
  Each item must have: description, impact, mitigation, acceptability justification.
-->

### 6A. Outstanding Items

| # | Item | Expected | Actual | Gap | Impact | Mitigation | Acceptable? |
|---|------|----------|--------|-----|--------|------------|-------------|
| 1 | — | — | — | — | — | — | [ ] Yes / [ ] No |
| 2 | — | — | — | — | — | — | [ ] Yes / [ ] No |
| 3 | — | — | — | — | — | — | [ ] Yes / [ ] No |

### 6B. Deferred to Next Phase

| # | Item | Reason for Deferral | Risk Level | Tracking Issue |
|---|------|---------------------|------------|----------------|
| 1 | — | — | Low / Medium / High | `#NNN` |
| 2 | — | — | Low / Medium / High | `#NNN` |

### 6C. Technical Debt Register

| # | Debt Item | Filed Date | Estimated Effort | Ticket |
|---|-----------|------------|-----------------|--------|
| 1 | — | `YYYY-MM-DD` | `N days` | `#NNN` |
| 2 | — | `YYYY-MM-DD` | `N days` | `#NNN` |

### Known Issues Summary

- **Total outstanding:** `N`
- **Total deferred:** `N`
- **Total debt items:** `N`
- **Blockers ("No" in Acceptable column):** `N` (must be 0 for GO)

---

## 7. Risk Assessment

<!--
  Assess what could go wrong in the target phase.
  Rate each risk: Impact (1-5) × Probability (1-5) = Score (1-25)
  Score ≥ 15 = must have mitigation plan
-->

### 7A. Technical Risks

| # | Risk | Impact (1-5) | Probability (1-5) | Score | Mitigation Plan | Owner |
|---|------|-------------|-------------------|-------|-----------------|-------|
| 1 | `{target-specific technical risk}` | N | N | N | `{mitigation and rollback evidence}` | `@owner` |
| 2 | `{canonical-boundary or compatibility risk}` | N | N | N | `{mitigation and validation}` | `@owner` |
| 3 | `{toolchain, performance, or diagnostics risk}` | N | N | N | `{mitigation and threshold}` | `@owner` |

### 7B. Process Risks

| # | Risk | Impact (1-5) | Probability (1-5) | Score | Mitigation Plan | Owner |
|---|------|-------------|-------------------|-------|-----------------|-------|
| 1 | `{scope or approval risk}` | N | N | N | `{mitigation}` | `@owner` |
| 2 | `{capacity or sequencing risk}` | N | N | N | `{mitigation}` | `@owner` |

### 7C. External Risks

| # | Risk | Impact (1-5) | Probability (1-5) | Score | Mitigation Plan | Owner |
|---|------|-------------|-------------------|-------|-----------------|-------|
| 1 | `{toolchain or dependency risk}` | N | N | N | `{mitigation}` | `@owner` |
| 2 | `{security or ecosystem risk}` | N | N | N | `{mitigation}` | `@owner` |

### Risk Summary

| Category | High (≥15) | Medium (9-14) | Low (<9) |
|----------|-----------|---------------|----------|
| Technical | `N` | `N` | `N` |
| Process | `N` | `N` | `N` |
| External | `N` | `N` | `N` |
| **Total** | **`N`** | **`N`** | **`N`** |

**Highest scored risk:** `#N — [description]` (Score: `NN`)
**All high-score risks have mitigation plans:** [ ] Yes / [ ] No

---

## 8. GO / NO-GO Decision

### 8A. Audit Findings Summary

| Category | Items Checked | Passing | Blocking Failures |
|----------|--------------|---------|-------------------|
| Phase Entry Criteria | `N` | `N` | `N` |
| Deliverables | `N` | `N` | `N` |
| Test Results | `N` suites | `N` pass | `N` fail |
| Documentation | `N` docs | `N` updated | `N` missing |
| Known Issues | `N` items | `N` acceptable | `N` unacceptable |
| Risk Assessment | `N` high risks | `N` mitigated | `N` unmitigated |

### 8B. Decision Criteria

All criteria must be `YES` for a **GO** decision:

| # | Criterion | Met? |
|---|-----------|------|
| 1 | All CI checks passing (target commit) | [ ] YES / [ ] NO |
| 2 | Zero blocking failures in entry criteria | [ ] YES / [ ] NO |
| 3 | All deliverables verified or waived with justification | [ ] YES / [ ] NO |
| 4 | Documentation coverage meets phase threshold | [ ] YES / [ ] NO |
| 5 | Zero unacceptable known issues | [ ] YES / [ ] NO |
| 6 | All high-score (≥15) risks have mitigation plans | [ ] YES / [ ] NO |
| 7 | Both auditor and maintainer signatures obtained | [ ] YES / [ ] NO |

### 8C. Decision

| Outcome | Selected | Justification |
|---------|----------|---------------|
| **GO** — Approve transition | [ ] | All criteria met. Ready to begin `{TARGET_PHASE}`. |
| **NO-GO** — Remain in source phase | [ ] | Blockers identified. Remediate and re-audit. |
| **GO with CONDITIONS** — Approve with constraints | [ ] | Conditions listed below. |

### 8D. Conditions (if GO with CONDITIONS)

| # | Condition | Deadline | Owner | Verification Method |
|---|-----------|----------|-------|---------------------|
| 1 | — | `YYYY-MM-DD` | `@owner` | — |
| 2 | — | `YYYY-MM-DD` | `@owner` | — |

### 8E. If NO-GO: Remediation Plan

| # | Blocker | Remediation Action | Owner | Target Date |
|---|---------|-------------------|-------|-------------|
| 1 | — | — | `@owner` | `YYYY-MM-DD` |
| 2 | — | — | `@owner` | `YYYY-MM-DD` |

**Re-audit scheduled for:** `YYYY-MM-DD`

---

## 9. Signatures

### 9A. Auditor Sign-Off

> I certify that I have independently verified all items in this audit and that the findings are accurate to the best of my knowledge.

| Field | Value |
|-------|-------|
| **Auditor Name** | `Full Name` |
| **GitHub Handle** | `@username` |
| **Role** | `Core Maintainer / Technical Lead / Release Engineer / External Auditor` |
| **Date** | `YYYY-MM-DD` |
| **Decision** | [ ] GO / [ ] NO-GO / [ ] GO with CONDITIONS |
| **Signature** | `_________________________________` |
| **Comments** | |

### 9B. Maintainer Sign-Off

> I have reviewed this audit, concur with its findings, and authorize (or block) the phase transition.

| Field | Value |
|-------|-------|
| **Maintainer Name** | `Full Name` |
| **GitHub Handle** | `@username` |
| **Role** | `Project Lead / BDFL / Technical Director` |
| **Date** | `YYYY-MM-DD` |
| **Decision** | [ ] GO / [ ] NO-GO / [ ] GO with CONDITIONS |
| **Signature** | `_________________________________` |
| **Comments** | |

### 9C. Disagreement Resolution

If auditor and maintainer disagree on the decision, record the dispute here:

| Dispute | Auditor Position | Maintainer Position | Resolution |
|---------|---------------|---------------------|------------|
| — | — | — | — |

**Escalation path:** Unresolved disputes escalate to `@senior-maintainer` or project governance body per `GOVERNANCE.md`.

---

## Appendix A: Checklist Quick Reference

Use this summary during the audit to track overall progress:

```
[ ] Section 1 — Audit Metadata complete
[ ] Section 2 — All entry criteria reviewed with evidence
[ ] Section 3 — All deliverables verified
[ ] Section 4 — Test results attached and passing
[ ] Section 5 — Documentation verified and reviewed
[ ] Section 6 — All known issues documented and assessed
[ ] Section 7 — Risk assessment completed with mitigations
[ ] Section 8 — Decision recorded with justification
[ ] Section 9 — Both signatures obtained
[ ] File committed to .roadmap/audits/ as YYYY-MM-DD_phase-N-to-M_audit.md
[ ] PR opened for maintainer review
```

---

*This audit was generated from the Logicodex Phase Transition Audit Template.*
*Template version: 2.0.0 — aligned with the single current-authority model*
*Last template update: 2026-07-20*
