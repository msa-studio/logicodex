# ROADMAP_POLICY.md — Logicodex Governance & Anti-Feature-Creep Enforcement

**Project:** Logicodex Programming Language Compiler
**Version:** v0.46.0-alpha
**Maintainer:** msa-studio
**Policy Version:** 1.1.0
**Status:** ENFORCED — effective immediately upon merge
**Last Updated:** 2026-07-13

> **This document defines roadmap-compliance process. Current work-sequence authority is `docs/architecture/current-authority.md`; `ROADMAP_v2.md` is the long-horizon phase baseline.**
> **Architecture authority is `docs/governance/architecture-change-control.md`.**
> **No code change shall bypass these rules.**
> **Violations require public acknowledgment and corrective action.**

---

## Table of Contents

1. [Philosophy](#1-philosophy)
2. [The 4 Alignment Checks](#2-the-4-alignment-checks)
3. [Phase-Gated Development Rules](#3-phase-gated-development-rules)
4. [Phase Transition Process](#4-phase-transition-process)
5. [Feature Classification](#5-feature-classification)
6. [PR Requirements by Feature Class](#6-pr-requirements-by-feature-class)
7. [Emergency Override](#7-emergency-override)
8. [Enforcement](#8-enforcement)
9. [Consequences](#9-consequences)
10. [Version History](#10-version-history)

---

## 1. Philosophy

Logicodex operates under three non-negotiable principles. Every decision — every PR, every issue, every design discussion — must trace back to at least one of these.

### 1.1 Harden Before Expand

> **No new feature shall be added while existing features remain unaudited, untested, or undocumented.**

The historical v1.45.0-alpha audit revealed **22 capabilities**: 7 fully implemented, 9 partial, 5 skeleton. This is unacceptable. The project overclaims capability. We harden the 16 incomplete items before adding capability #23.

**Hardness levels (enforced definitions):**

| Level | Name | Criteria | Count |
|-------|------|----------|-------|
| 0 | Skeleton | Type definitions, empty traits, stub functions | Must reach Level 1 |
| 1 | Partial | Core logic present, some tests, basic docs | Must reach Level 2 |
| 2 | Complete | Full test coverage, documented, benchmarked, CI green | **Minimum acceptable** |
| 3 | Hardened | 2+ weeks stable in CI, user feedback incorporated, perf validated | Target for CORE features |

### 1.2 Proof Before Progress

> **Claims require evidence. Evidence requires reproducibility. Reproducibility requires CI.**

- A feature is **not "working"** until CI passes on it for 7 consecutive days.
- A feature is **not "complete"** until it has tests, documentation, and a passing benchmark.
- A feature is **not "stable"** until it survives 14 days of CI without flake or regression.

### 1.3 Audit Before Advance

> **No phase transition without independent audit. No exception.**

The project advances phase-by-phase. Each transition requires a published audit report using the `AUDIT_TEMPLATE.md` format. The maintainer (msa-studio) may not self-audit. Community reviewers or external contributors must validate.

---

## 2. The 4 Alignment Checks

**FOR ANY CHANGE** — bugfix, refactor, feature, doc update, dependency bump — **ALL 4 CHECKS MUST PASS.**

These checks are enforced via CI gatekeeper (see Section 8). A PR with any check failing **cannot be merged**.

### Check 1: Roadmap Alignment

**Question:** Is this change within the current phase's defined scope?

| Condition | Result |
|-----------|--------|
| Change addresses a current-phase item | PASS |
| Change hardens an existing feature (test/docs/perf) | PASS |
| Change is a bugfix for any existing feature | PASS |
| Change adds a capability from a future phase | **FAIL — block** |
| Change introduces a new feature not on the roadmap | **FAIL — block** |
| Change skips ahead to implement Phase N+1 before Phase N complete | **FAIL — block** |

**Evidence required:** PR description must reference the roadmap item by ID (e.g., `PHASE-2.3`, `ROADMAP-ITEM-#17`).

### Check 2: RFC Requirement

**Question:** Does this change require a Request for Comments (RFC)?

RFC is **MANDATORY** for:
- New syntax or language features
- New backend targets (e.g., WASM, ARM64, RISC-V)
- Changes to the compiler pipeline architecture
- Public API changes (breaking or additive)
- New runtime features (concurrency, security, GC)
- Changes to type system or inference

RFC is **NOT REQUIRED** for:
- Bug fixes (unless they change public behavior)
- Documentation improvements
- Performance optimizations (no API change)
- Refactoring with no behavioral change
- Dependency updates (minor/patch)

**RFC process:**
1. Open an issue with label `needs-rfc`
2. Write RFC using the RFC_TEMPLATE.md format
3. 48-hour minimum discussion period
4. Approved by maintainer + 1 community reviewer
5. RFC linked in PR description

### Check 3: Audit Trail

**Question:** Does this change include tests, documentation, and evidence of working?

**Required evidence by change type:**

| Change Type | Tests | Docs | Evidence |
|-------------|-------|------|----------|
| Bug fix | Regression test that fails before, passes after | Changelog entry | CI link |
| Feature | Unit + integration tests | API docs + user guide section | Screenshot / demo / benchmark |
| Refactor | Existing tests still pass + new edge cases | Updated if behavior changes | CI diff showing no regression |
| Performance | Benchmark (before/after) | Performance notes | Benchmark results in PR |

**Minimum test coverage by feature class:**
- CORE: >= 85% line coverage
- TARGET: >= 75% line coverage + CI on target platform
- RUNTIME: >= 80% line coverage + stress test
- TOOLING: >= 70% line coverage + user scenario test
- EXPERIMENTAL: >= 50% line coverage (sandboxed)

### Check 4: No Regression

**Question:** Does CI pass? Do all existing tests pass?

**Enforced gates:**
- [ ] All existing tests pass (no failures, no skips without justification)
- [ ] New tests added for this change pass
- [ ] Lint/clippy clean (no warnings introduced)
- [ ] Format check passes (`cargo fmt --check`)
- [ ] Benchmark comparison: no >5% regression without documented justification
- [ ] MSRV (Minimum Supported Rust Version) check passes
- [ ] If TARGET change: CI on all supported platforms passes

**CI must be green for 7 consecutive days post-merge.** If a merge introduces flakiness, it may be reverted (see Section 9).

---

## 3. Phase-Gated Development Rules

Logicodex development is organized into sequential phases. **These rules are absolute.**

### 3.1 Core Rules

| # | Rule | Enforcement |
|---|------|-------------|
| PG-1 | **ONLY work on current phase items.** | CI gatekeeper blocks PRs referencing future phases. |
| PG-2 | **NO new features from future phases.** | PR template auto-checks phase labels. |
| PG-3 | **NO skipping phases.** | Phase N+1 PRs rejected while Phase N exit criteria are unmet. |
| PG-4 | **NO adding items to a closed phase.** | Locked phases are immutable. New findings go to current phase backlog. |
| PG-5 | **Hardening work ALWAYS allowed.** | Tests, docs, perf, refactors for existing features are always permitted. |
| PG-6 | **Bugfixes ALWAYS allowed.** | Fixes for any existing feature are always permitted, regardless of phase. |

### 3.2 Emergency Override

See Section 7 for the **sole exception** to these rules. Overrides require:
- Documented justification
- 48-hour community review
- Approval by maintainer + 1 reviewer
- Post-hoc audit within 14 days

### 3.3 Phase Definition Template

Each phase in `ROADMAP_v2.md` must include:

```
## Phase N: [Name]
**Goal:** [One-sentence objective]
**Entry Criteria:** [What must be true to start this phase]
**Exit Criteria:** [Specific, measurable conditions to complete]
**Items:**
- [ ] ITEM-N.1: [Description] — Hardness target: [0→1 / 1→2 / 2→3]
- [ ] ITEM-N.2: [Description] — Hardness target: [0→1 / 1→2 / 2→3]
**Blocked by:** [Phase N-1] (cannot start until Phase N-1 exits)
**Blocks:** [Phase N+1] (cannot start until this phase exits)
```

### 3.4 Current Phase Tracking

`docs/architecture/current-authority.md` owns the active phase label and
owner-locked work sequence. `ROADMAP_v2.md` owns long-horizon phase definitions,
history, and audit context. The sequence must not be copied into this policy or
the long-horizon roadmap; both link to the current-authority entry point.

---

## 4. Phase Transition Process

Moving from Phase N to Phase N+1 is a **significant event** requiring formal process.

### 4.1 Prerequisites (ALL must pass)

1. **All Phase N exit criteria are met.**
   - Every checklist item in the phase is complete (checked).
   - Every item meets its hardness target.
   - No item is in "skeleton" or "partial" state.

2. **Independent audit completed using `AUDIT_TEMPLATE.md`.**
   - Auditor cannot be the person who implemented the majority of phase items.
   - For single-maintainer projects: external/community reviewer required.
   - Audit report published as a GitHub Discussion or pinned issue.

3. **All tests passing for 2 consecutive weeks (14 days).**
   - No flaky tests.
   - No allowed failures.
   - CI history link provided as evidence.

4. **Documentation updated.**
   - README reflects current capabilities accurately.
   - User guide covers all Phase N features.
   - Changelog has entries for all Phase N work.
   - API docs are generated and published.

5. **Audit report PR merged.**
   - PR titled: `[PHASE-TRANSITION] Phase N → Phase N+1 Audit Report`
   - Contains: full audit, evidence links, sign-off from auditor
   - Merged by maintainer after 48h review window

### 4.2 Transition PR Template

```markdown
## Phase Transition: N → N+1

### Exit Criteria Verification
- [ ] All Phase N items complete (link to checklist)
- [ ] Independent audit report: [link]
- [ ] 14-day CI stability evidence: [link to CI history]
- [ ] Documentation updated: [link to docs PR]

### Audit Summary
**Auditor:** @username
**Date:** YYYY-MM-DD
**Findings:** [PASS / PASS-with-notes / FAIL]
**Notes:** [Any observations]

### Risk Assessment
- [ ] No known blockers for Phase N+1
- [ ] Team/community has bandwidth for next phase
- [ ] Dependencies for Phase N+1 are available

### Sign-off
- [ ] Auditor sign-off
- [ ] Maintainer sign-off (@msa-studio)
- [ ] 48h community review completed
```

### 4.3 Reversion

If a phase transition is approved but later found premature:
- **Within 7 days:** Direct revert of transition PR.
- **After 7 days:** New transition PR required to move back.
- Both cases require public acknowledgment in ROADMAP_v2.md.

---

## 5. Feature Classification

Every feature, issue, and PR must be classified into **exactly one** of these categories. The classification determines review requirements, testing standards, and stability guarantees.

### 5.1 CORE — Compiler Pipeline

**Definition:** The lexical analyzer, parser, AST, semantic analyzer, type checker, IR generator, optimizer, and code generator.

**Governance:**
- Any change requires **RFC**.
- Minimum 85% test coverage.
- Must not break existing valid programs.
- Breaking changes require deprecation cycle (1 minor version).
- All changes reviewed by maintainer.

**Examples:** Parser grammar changes, type inference modifications, new AST node types, optimization passes.

### 5.2 TARGET — Platform Backends

**Definition:** Code generation targets (x86_64, ARM64, WASM, LLVM IR, etc.) and platform ABIs.

**Governance:**
- New backend requires **RFC**.
- Requires working CI on the target platform.
- Minimum 75% test coverage.
- Must pass target-specific test suite.
- Platform support tiers defined separately.

**Examples:** ARM64 backend, WASM output, custom calling conventions, cross-compilation support.

### 5.3 RUNTIME — Concurrency, Security, Memory

**Definition:** Runtime library, concurrency primitives, memory management, security features, foreign function interface.

**Governance:**
- New runtime features require **RFC**.
- Minimum 80% test coverage.
- Requires stress tests and benchmark comparison.
- Security-sensitive changes require security review label.
- Performance regression >2% must be justified.

**Examples:** Async runtime, memory allocator, sandboxing, FFI bindings, threading model.

### 5.4 TOOLING — Developer Experience

**Definition:** Formatter, LSP server, debugger support, build system, package manager, REPL.

**Governance:**
- New tools require documentation + user guide.
- Minimum 70% test coverage.
- Must include user scenario tests (integration-style).
- UX changes should have before/after demonstration.
- Stable tools follow semver; experimental tools follow `0.x`.

**Examples:** `logicodex fmt`, LSP implementation, REPL shell, package manager CLI.

### 5.5 EXPERIMENTAL — Research & Development

**Definition:** Sandbox for research features, prototypes, and speculative work.

**Governance:**
- **No stability guarantee whatsoever.**
- Must be behind `#[cfg(feature = "experimental-xxx")]`.
- Minimum 50% test coverage.
- Can be removed without deprecation notice.
- Cannot be enabled by default in any release.
- Must have `EXPERIMENTAL` banner in all docs.

**Examples:** New language features under research, alternative backends, prototype optimizations.

### 5.6 Classification Process

1. Every issue is labeled with one of: `class:core`, `class:target`, `class:runtime`, `class:tooling`, `class:experimental`.
2. PRs must reference the classification in their description.
3. Misclassified PRs will be re-labeled by maintainers.
4. Classification can be challenged via issue comment (48h resolution).

---

## 6. PR Requirements by Feature Class

### 6.1 Universal PR Requirements

Every PR must include:

```markdown
## PR Checklist (Universal)
- [ ] Linked to an issue (if applicable)
- [ ] PR title follows convention: `[CLASS] Brief description`
- [ ] Description explains what and why
- [ ] All 4 Alignment Checks pass (auto-enforced by CI)
- [ ] CHANGELOG.md entry added
- [ ] No merge conflicts
- [ ] CI is green
```

### 6.2 Class-Specific Requirements

#### CORE PR Checklist

```markdown
## CORE PR Checklist
- [ ] RFC approved and linked (if applicable)
- [ ] Test coverage >= 85% (codecov report attached)
- [ ] Unit tests for all new logic
- [ ] Integration tests for end-to-end scenarios
- [ ] No breaking changes OR deprecation cycle documented
- [ ] Parser/AST changes include grammar documentation
- [ ] Type system changes include inference examples
- [ ] Performance benchmark (no >5% regression)
- [ ] Documentation PR linked or included
```

#### TARGET PR Checklist

```markdown
## TARGET PR Checklist
- [ ] RFC approved and linked (if new backend)
- [ ] Test coverage >= 75%
- [ ] CI passes on target platform(s)
- [ ] Target-specific test suite added/updated
- [ ] ABI compliance verified (if applicable)
- [ ] Cross-compilation tested
- [ ] Platform tier documented
```

#### RUNTIME PR Checklist

```markdown
## RUNTIME PR Checklist
- [ ] RFC approved and linked (if applicable)
- [ ] Test coverage >= 80%
- [ ] Unit + integration + stress tests
- [ ] Benchmark comparison (before/after)
- [ ] No memory leaks (valgrind/miri clean where applicable)
- [ ] Security label applied if security-relevant
- [ ] Concurrency tests for thread-safety
```

#### TOOLING PR Checklist

```markdown
## TOOLING PR Checklist
- [ ] Test coverage >= 70%
- [ ] User scenario / integration tests
- [ ] User guide section added/updated
- [ ] CLI help text updated (if applicable)
- [ ] UX before/after (screenshot or description)
- [ ] Documentation published or PR linked
```

#### EXPERIMENTAL PR Checklist

```markdown
## EXPERIMENTAL PR Checklist
- [ ] Behind `experimental-xxx` feature flag
- [ ] Test coverage >= 50%
- [ ] `EXPERIMENTAL` banner in all docs
- [ ] No default-enable in any configuration
- [ ] Explicitly marked unstable in API
- [ ] Removal plan documented (even if tentative)
```

### 6.3 PR Title Convention

```
[CORE] parser: Add error recovery for missing semicolons
[TARGET] arm64: Implement leaf function optimization
[RUNTIME] async: Add task scheduler priority queue
[TOOLING] fmt: Support configurable indentation width
[EXPERIMENTAL] effects: Prototype algebraic effect handlers
```

### 6.4 Review Requirements

| Class | Reviewers Required | Reviewer Qualifications |
|-------|-------------------|------------------------|
| CORE | 2 | Maintainer + domain expert |
| TARGET | 2 | Maintainer + platform expert |
| RUNTIME | 2 | Maintainer + reviewer |
| TOOLING | 1 | Maintainer or tooling lead |
| EXPERIMENTAL | 1 | Any contributor (lightweight) |

---

## 7. Emergency Override

The Emergency Override is the **only mechanism** to bypass phase gates, RFC requirements, or classification rules. It is intentionally difficult to invoke.

### 7.1 Conditions Allowing Override

An Emergency Override is permitted **only** for:

| Condition | Example |
|-----------|---------|
| Critical security vulnerability | CVE affecting runtime memory safety |
| Build breakage blocking all development | CI completely broken for >48h |
| Data loss or corruption bug | Compiler generating incorrect code silently |
| Dependency with known CVE | Rust crate with security advisory |
| Legal/licensing issue | Dependency license incompatibility discovered |

### 7.2 NOT Valid for Override

- "This feature is cool"
- "Users are asking for it"
- "It would make development easier"
- "Other languages have it"
- Time pressure from external parties

### 7.3 Override Process

```
Step 1: Create issue with label emergency-override
Step 2: Fill out EMERGENCY_TEMPLATE.md:
        - Condition triggering override
        - Why normal process cannot be followed
        - Proposed change
        - Risk assessment
        - Rollback plan
Step 3: 48-hour minimum review window (no exceptions)
Step 4: Approval required from:
        - Maintainer (@msa-studio) AND
        - At least 1 community reviewer
Step 5: Implement with expedited review
Step 6: Post-hoc audit within 14 days
Step 7: Public report in ROADMAP_v2.md under "Emergency Overrides"
```

### 7.4 Emergency PR Requirements

Emergency PRs must:
- Reference the approved override issue
- Include `EMERGENCY:` prefix in title
- Have minimal scope (fix the emergency, nothing else)
- Be reverted if found to cause new issues
- Receive full retrospective review within 14 days

### 7.5 Annual Limit

**Maximum 3 emergency overrides per calendar year.** Exceeding this triggers policy review.

---

## 8. Enforcement

This policy is enforced through **automated gates**, **CODEOWNERS**, and **monthly audits**.

### 8.1 CI Gatekeeper

The CI pipeline serves as the primary enforcement mechanism:

```yaml
# .github/workflows/roadmap-compliance.yml (reference)
name: Roadmap Compliance Gate
on: [pull_request]
jobs:
  alignment-checks:
    runs-on: ubuntu-latest
    steps:
      - Check 1: Parse PR description for roadmap item reference
      - Check 2: Verify RFC linked if required by labels
      - Check 3: Verify test coverage threshold met
      - Check 4: Run full test suite, verify no regression
    failure: Post comment with failing checks, block merge
```

**Gatekeeper rules:**
- PR without roadmap reference → BLOCK
- PR adding future-phase feature → BLOCK
- PR with insufficient coverage → BLOCK
- PR with CI failure → BLOCK
- PR without required reviewers → BLOCK

### 8.2 CODEOWNERS Enforcement

```
# .github/CODEOWNERS
*                       @msa-studio
/compiler/              @msa-studio @logicodex/compiler-team
/runtime/               @msa-studio @logicodex/runtime-team
/targets/               @msa-studio @logicodex/targets-team
tooling/                @msa-studio @logicodex/tooling-team
rfcs/                   @msa-studio @logicodex/rfcs-team
ROADMAP_v2.md              @msa-studio
ROADMAP_POLICY.md       @msa-studio
```

CODEOWNERS ensures:
- No PR is merged without maintainer review
- Domain experts review changes in their areas
- Policy documents require maintainer approval

### 8.3 Monthly Audit of Roadmap Compliance

On the **first Monday of each month**, the following audit is conducted:

1. **Review all merged PRs** from the past month
2. **Classify each PR** by feature class
3. **Check compliance:**
   - Were all 4 Alignment Checks satisfied?
   - Were RFCs filed where required?
   - Were tests and docs included?
   - Any phase violations?
4. **Publish findings** as a GitHub Discussion titled `Roadmap Compliance Audit — YYYY-MM`
5. **Track violations** in a dedicated issue (#roadmap-violations)

**Audit responsible party:** Rotating community member (volunteer basis), overseen by maintainer.

### 8.4 Public Transparency

`docs/architecture/current-authority.md` must always reflect the current phase,
locked invariants, debt disposition, and active sequence. `ROADMAP_v2.md` keeps:

- Phase history (completed phases with dates)
- Any emergency overrides (with justification)
- Any policy violations (with corrective action)
- Next 2 phases (for visibility, not for work)

---

## 9. Consequences

Violating this policy has defined, escalating consequences.

### 9.1 Severity Levels

| Level | Violation Type | Examples |
|-------|---------------|----------|
| S1 | Minor procedural | Missing changelog entry, incomplete PR description |
| S2 | Moderate procedural | Missing tests, insufficient coverage, skipped RFC |
| S3 | Major procedural | Phase violation, unauthorized feature addition |
| S4 | Critical | Malicious bypass, repeated violations, broken policy |

### 9.2 Consequences by Severity

#### S1 — Minor
- **Action:** PR blocked until fixed
- **Tracking:** None (just fix and proceed)
- **Record:** Not publicly logged

#### S2 — Moderate
- **Action:** PR blocked + required changes documented
- **Tracking:** Comment on PR with required fixes
- **Record:** Noted in monthly audit
- **Repeat (>2 in a month):** Escalates to S3

#### S3 — Major
- **Action:**
  - PR blocked and may be closed
  - Feature may be reverted if already merged
  - Issue created documenting the violation
- **Tracking:**
  - Added to `#roadmap-violations` tracking issue
  - Public acknowledgment in ROADMAP_v2.md
  - 30-day improvement plan required
- **Record:** Permanent in project history

#### S4 — Critical
- **Action:**
  - Immediate revert of offending changes
  - Emergency policy review
  - Maintainer (@msa-studio) direct intervention
- **Tracking:**
  - Dedicated incident report
  - ROADMAP_v2.md updated with full details
  - Policy amendment may follow
- **Repeat:** Contributor may lose merge/push privileges

### 9.3 Revert Process

When a violation requires reversion:

1. Create issue: `[REVERT] Reason for reversion`
2. Open revert PR referencing the issue
3. Fast-track review (24h, not 48h)
4. Merge revert
5. Notify original PR author
6. Document in ROADMAP_v2.md under "Reversions"

### 9.4 Appeal Process

Contributors may appeal a violation determination:

1. Comment on the violation issue within 7 days
2. State grounds for appeal
3. Community discussion (48h)
4. Maintainer decision (final)

### 9.5 Public Acknowledgment Format

All S3 and S4 violations are recorded in ROADMAP_v2.md:

```markdown
## Policy Violations Log

### 2025-07
- **Severity:** S3
- **Date:** 2025-07-10
- **Description:** Unauthorized Phase 3 feature merged during Phase 2
- **PR:** #123
- **Corrective Action:** Feature reverted, PR closed, contributor coached
- **Status:** Resolved
```

---

## 10. Version History

| Version | Date | Description | Approved By |
|---------|------|-------------|-------------|
| 1.0.0 | 2025-07-17 | Initial policy adopted following v1.45.0-alpha audit findings (22 capabilities: 7 complete, 9 partial, 5 skeleton). Policy addresses overclaiming and feature creep. | @msa-studio |

### 10.1 Amendment Process

This policy may be amended via:

1. RFC proposing policy change
2. 7-day community discussion
3. Approval by maintainer (@msa-studio)
4. Version history updated
5. Announcement in GitHub Discussions

**No retroactive amendments.** Changes apply to future work only.

### 10.2 Policy Review Schedule

- **Quarterly:** Light review for effectiveness
- **Annually:** Full review with community input
- **Post-incident:** Review after any S4 violation
- **Post-phase-transition:** Review after each phase change

---

## Appendix A: Quick Reference Card

### For Contributors

```
BEFORE OPENING A PR:
  1. Is this in the current phase and sequence? → Check docs/architecture/current-authority.md
  2. Does this need an RFC? → Check Section 2.2
  3. Is my feature classified correctly? → Check Section 5
  4. Do I have tests + docs? → Check Section 6
  5. Does CI pass locally? → cargo test && cargo clippy

PR TITLE FORMAT:
  [CLASS] area: Description
  Example: [CORE] typechecker: Add inference for generic constraints
```

### For Reviewers

```
REVIEW CHECKLIST:
  1. Check roadmap alignment (is this current phase?)
  2. Check RFC requirement (is RFC linked if needed?)
  3. Check audit trail (tests, docs, evidence)
  4. Check no regression (CI green, coverage OK)
  5. Verify feature class label is correct
  6. Verify PR title follows convention
  7. Verify CHANGELOG entry present
```

### For Maintainers

```
MONTHLY RESPONSIBILITIES:
  1. Review monthly compliance audit
  2. Reconcile current-authority.md and ROADMAP_v2.md without duplicating sequence
  3. Check violation tracking issue
  4. Verify phase transition criteria (if approaching)

PHASE TRANSITION RESPONSIBILITIES:
  1. Verify all exit criteria met
  2. Appoint independent auditor
  3. Review audit report
  4. Open transition PR
  5. 48h community review
  6. Merge and announce
```

---

## Appendix B: Definitions

| Term | Definition |
|------|------------|
| **Feature creep** | Adding new features beyond the agreed scope, especially before existing features are hardened |
| **Hardening** | Improving an existing feature's test coverage, documentation, performance, and stability |
| **Phase** | A time-bounded development stage with specific goals, entry criteria, and exit criteria |
| **Skeleton** | A feature with type definitions and stub implementations but no working logic (Hardness 0) |
| **Partial** | A feature with core logic present but incomplete tests, docs, or edge cases (Hardness 1) |
| **Complete** | A feature with full tests, docs, and passing CI (Hardness 2) |
| **Hardened** | A feature that has been stable in CI for 2+ weeks with user validation (Hardness 3) |
| **RFC** | Request for Comments — a structured design document for significant changes |
| **CI** | Continuous Integration — automated test and build pipeline |
| **MSRV** | Minimum Supported Rust Version |
| **ABI** | Application Binary Interface |
| **LSP** | Language Server Protocol |

---

*"A compiler is not judged by the number of its features, but by the reliability of each one."*
*— Logicodex Governance Principle*

---

**END OF DOCUMENT**

*This policy is active. All contributors are expected to read and comply with this document before submitting changes.*
