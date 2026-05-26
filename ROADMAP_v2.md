# Logicodex Project Roadmap v2.0

## Phase-Gated Development Plan

**Document Version:** 2.0  
**Last Updated:** 2025-07-15  
**Status:** Active — supersedes all previous roadmaps  
**Policy Reference:** See `ROADMAP_POLICY.md` for governance, RFC process, and change procedures  
**Maintainer:** Single maintainer (see `CONTRIBUTING.md` for co-maintainer recruitment)

---

## Executive Summary

Logicodex is at **v1.45.0-alpha**, maintained by a single developer. The codebase has **7/22 capabilities fully implemented**, **9/22 partially implemented**, and **5/22 at skeleton level**. This roadmap replaces aspirational timelines with **phase-gated milestones** where each phase has strict entry criteria, measurable deliverables, mandatory audit checkpoints, and exit proof requirements. **No phase may begin until the previous phase's audit is signed off.**

### Honest Maturity Matrix (v1.45.0-alpha)

| Domain | Status | Level | Notes |
|--------|--------|-------|-------|
| Bilingual alias-to-canonical | FULL | Production | 60 dict entries, 200+ aliases |
| Compile-time type checking | FULL | Production | 32 error variants |
| Bilingual error diagnostics | FULL | Production | Malay + English |
| Benchmark framework | FULL | Production | 4 layers, 6 Criterion benchmarks |
| Validator tiering | FULL | Production | Tier A=6, B=13, C=8 |
| Dual license + policy docs | FULL | Production | MPL-2.0 / MIT, SECURITY.md |
| Compiler pipeline | PARTIAL | HIR dormant, bypassed | HIR exists but is not on execution path |
| Actor-model | PARTIAL | Types + semantic complete, no runtime | Runtime is stub |
| Capability security | PARTIAL | Compile-time works, runtime stub | Issue #07 |
| Sharded runtime | PARTIAL | Threads + affinity exist, messaging missing | Issue #08 |
| Network reactor | PARTIAL | Syscalls exist, socket lifecycle missing | Issue #09 |
| WASM backend | PARTIAL | LLVM path exists, no linker/runtime/CI | Issue #10 |
| Freestanding x86_64 | PARTIAL | Code complete, never booted in QEMU | Issue #11 |
| Raylib FFI | PARTIAL | 55 wrappers, partial API coverage | Issue #12 |
| CI/CD | PARTIAL | Failing, 148/148 claim false | Issue #13 |
| HIR lowering | SKELETON | Dormant | Issue #02 |
| Deterministic execution | SKELETON | Framework only, no runtime integration | Issue #14 |
| Freestanding aarch64 | SKELETON | LLVM triple only | Issue #15 |
| Freestanding riscv64 | SKELETON | LLVM triple only | Issue #16 |
| Self-hosting | SKELETON | Not started | Issue #17 |
| Package manager | SKELETON | Not started | Issue #18 |
| LSP engine | SKELETON | Not started | Issue #05 |

---

## Architecture Freeze Policy

1. **Inter-phase architecture is frozen.** No architectural changes may be proposed during an active phase. All architectural RFCs must be filed against `ROADMAP_POLICY.md` and are evaluated only at phase boundaries.
2. **Intra-phase additions require auditor approval.** If a critical bug or security issue demands an architectural change mid-phase, it must be approved by the phase auditor (see `ROADMAP_POLICY.md` Section 4).
3. **HIR decision is Phase 1 only.** The decision to activate, remove, or redesign HIR is gated entirely to Phase 1. No HIR discussions in Phases 2-5.
4. **Backend additions frozen until Phase 4.** No new architecture backends may be proposed before Phase 4 entry audit.
5. **Self-hosting interface frozen at Phase 5 entry.** The compiler's public API (parser, typechecker, codegen interfaces) will be frozen at the Phase 5 entry audit to enable self-hosting work.

---

## RFC Process

All major changes must follow the RFC process defined in `ROADMAP_POLICY.md`:

| Change Type | RFC Required | Approval |
|-------------|-------------|----------|
| New feature | Yes | Phase auditor |
| Architectural change | Yes | Maintainer + auditor |
| Deprecation | Yes | Maintainer |
| Bug fix (no API change) | No | PR review |
| Documentation update | No | PR review |
| CI/infra change | No | PR review |

---

## How to Propose Changes

1. **Check `ROADMAP_POLICY.md`** for the current phase and frozen areas.
2. **File an issue** using the appropriate template (`feature-request.md`, `bug-report.md`, `rfc-proposal.md`).
3. **Reference the roadmap phase** in the issue title: `[PHASE-N] Title`.
4. **If RFC required**, create `rfcs/YYYY-MM-DD-title.md` following the template in `ROADMAP_POLICY.md`.
5. **Submit PR** only after issue/RFC approval. PRs without prior approval will be closed.

---

## Phase 1: HARDEN (Current — v1.45.x)

### Goal
Restore project credibility by fixing CI, hardening the compiler core, booting x86_64 in QEMU, making honest documentation, and removing all overclaims.

### Entry Criteria
- [x] Project exists and compiles locally
- [x] Audit findings documented (this roadmap)
- [x] Single maintainer acknowledges scope limitations

### Deliverables

| ID | Deliverable | Measurable Criteria | Owner |
|----|-------------|---------------------|-------|
| P1-D1 | CI green | All Tier A (6) + Tier B (13) validators pass on `main` branch for 2 consecutive weeks | Maintainer |
| P1-D2 | Honest validator count | Replace "148/148" with actual count (7 full + 9 partial + 5 skeleton = 21 tracked); remove false claims | Maintainer |
| P1-D3 | README maturity matrix | README.md contains honest maturity matrix matching Section 2 of this document | Maintainer |
| P1-D4 | Overclaim removal | All documentation files audited; no claim exceeds "PARTIAL" unless backed by passing tests | Maintainer |
| P1-D5 | x86_64 QEMU boot | Freestanding x86_64 kernel boots in QEMU, prints "Logicodex" to serial, exits cleanly | Maintainer |
| P1-D6 | HIR decision | Written decision in `docs/architecture/hir-decision.md`: activate, remove, or redesign with rationale | Maintainer |
| P1-D7 | Test stability | `cargo test` passes on stable Rust for 14 consecutive days on `main` | CI / Maintainer |
| P1-D8 | Contributing guide updated | `CONTRIBUTING.md` reflects single-maintainer reality; includes how to become co-maintainer | Maintainer |

### Audit Checklist

> **Auditor:** External or designated community member (see `ROADMAP_POLICY.md`).  
> **Sign-off required:** All items ticked, evidence linked.

- [ ] **P1-A1** — CI history review: Last 14 days of CI runs on `main` show all green (link to CI dashboard)
- [ ] **P1-A2** — Validator count audit: `grep -r "148" docs/ README.md` returns zero matches
- [ ] **P1-A3** — README audit: README.md maturity matrix matches this roadmap's Section 2
- [ ] **P1-A4** — Documentation audit: `docs/` directory scanned; all claims rated FULL/PARTIAL/SKELETON with issue links
- [ ] **P1-A5** — QEMU boot evidence: Screenshot or CI artifact showing serial output "Logicodex" from QEMU x86_64
- [ ] **P1-A6** — HIR decision document exists and is committed to `docs/architecture/hir-decision.md`
- [ ] **P1-A7** — Test stability log: 14 consecutive days of passing tests (CI history or manual log)
- [ ] **P1-A8** — Contributing guide reviewed and approved
- [ ] **P1-A9** — Security policy still current: `SECURITY.md` reviewed, no stale contacts
- [ ] **P1-A10** — Benchmark suite still runs: `cargo bench` completes without error

### Exit Proof

The following evidence must be provided to exit Phase 1:

1. **CI dashboard link** showing 14 consecutive green builds on `main`
2. **QEMU boot screenshot or CI artifact** with "Logicodex" serial output
3. **HIR decision document** committed at `docs/architecture/hir-decision.md`
4. **Documentation audit report** (can be a GitHub issue comment with checklist)
5. **Auditor sign-off** in the Phase 1 tracking issue (#TBD-phase1-audit)

### Target Date

**No target date — ship when audit passes.**

### Open Issues

- Issue #02 — HIR lowering (decision pending)
- Issue #07 — Capability security runtime (compile-time done, stubbed)
- Issue #08 — Sharded runtime messaging (threads exist, channels missing)
- Issue #09 — Network reactor socket lifecycle (syscalls done)
- Issue #10 — WASM linker + runtime (LLVM path done)
- Issue #11 — x86_64 freestanding QEMU boot (code done, needs integration test)
- Issue #12 — Raylib FFI completion (55 wrappers, needs more coverage)
- Issue #13 — CI/CD fix (failing, overclaims)
- Issue #14 — Deterministic execution framework (needs runtime hook)
- Issue #15 — aarch64 skeleton (triple only)
- Issue #16 — riscv64 skeleton (triple only)
- Issue #17 — Self-hosting (not started)
- Issue #18 — Package manager (not started)
- Issue #19 — README overclaims (meta)

---

## Phase 2: TYPE SYSTEM + TOOLING (v1.50.x)

### Goal
Complete the nominal type system, implement the formatter, and lay the LSP engine foundation.

### Entry Criteria

> **ALL of the following must be true. Phase 2 work MUST NOT begin until signed off.**

- [ ] Phase 1 audit fully passed (all P1-A1 through P1-A10 ticked)
- [ ] Auditor sign-off recorded in Phase 1 tracking issue
- [ ] `main` branch tagged with `v1.49.0` (Phase 1 completion marker)
- [ ] HIR decision implemented (if activate: HIR on execution path; if remove: HIR code deleted; if redesign: RFC accepted)
- [ ] No critical or high bugs open (GitHub label `severity:critical` or `severity:high` must be zero)

### Deliverables

| ID | Deliverable | Measurable Criteria | Owner |
|----|-------------|---------------------|-------|
| P2-D1 | Nominal type system | Structs and enums parse, typecheck, and codegen; 10+ integration tests passing (Issue #03) | TBD |
| P2-D2 | Type inference boundaries | Explicit inference limits documented; no "infinite inference" scenarios; 5 boundary tests | TBD |
| P2-D3 | `ldx-fmt` formatter | Handles all language constructs; `ldx-fmt --check` exits 0 on well-formatted code; 50+ unit tests (Issue #04) | TBD |
| P2-D4 | LSP engine foundation | `initialize`, `textDocument/didOpen`, `textDocument/didChange`, `textDocument/didClose` implemented; parses open files; basic diagnostics push (Issue #05) | TBD |
| P2-D5 | Diagnostic regression tests | All existing 32 error variants have test cases that verify both English and Malay output | TBD |

### Audit Checklist

- [ ] **P2-A1** — Nominal types: `cargo test nominal` passes with 10+ integration tests
- [ ] **P2-A2** — Type inference: Boundary tests document and verify inference limits
- [ ] **P2-A3** — Formatter: `ldx-fmt` installed via `cargo install`; formats 1000+ lines of sample code without error
- [ ] **P2-A4** — Formatter idempotency: `ldx-fmt file.ldx && ldx-fmt file.ldx` produces identical output (checksum match)
- [ ] **P2-A5** — LSP initialize: LSP client can connect, receive server capabilities, open a file
- [ ] **P2-A6** — LSP diagnostics: Opening a file with a type error pushes diagnostic to client within 2 seconds
- [ ] **P2-A7** — Diagnostic bilingual: Both English and Malay diagnostics appear correctly in LSP client
- [ ] **P2-A8** — No regression: All Phase 1 deliverables still pass (Tier A+B validators, QEMU boot)
- [ ] **P2-A9** — Documentation: `docs/types.md` and `docs/formatter.md` exist and match implementation
- [ ] **P2-A10** — Performance: Formatter runs at >1000 lines/sec on a mid-range laptop

### Exit Proof

1. **Test run output** showing all P2 tests passing + Phase 1 regression tests
2. **LSP demo video or screenshot** showing diagnostics in a client (VS Code, Neovim, or similar)
3. **Formatter idempotency proof** (script output or CI step)
4. **Auditor sign-off** in Phase 2 tracking issue

### Target Date

**No target date — ship when audit passes.**

### Open Issues

- Issue #03 — Nominal type system (primary deliverable)
- Issue #04 — `ldx-fmt` formatter (primary deliverable)
- Issue #05 — LSP engine foundation (primary deliverable)
- Issue #20 — Type inference boundaries (new, from P2-D2)
- Issue #21 — LSP diagnostic performance (new, from P2-A6)

---

## Phase 3: RUNTIME (v1.60.x)

### Goal
Implement the actor thread pool, zero-copy channels, capability hardware gates, cross-shard messaging, and deterministic execution.

### Entry Criteria

> **ALL of the following must be true. Phase 3 work MUST NOT begin until signed off.**

- [ ] Phase 2 audit fully passed (all P2-A1 through P2-A10 ticked)
- [ ] Auditor sign-off recorded in Phase 2 tracking issue
- [ ] `main` branch tagged with `v1.59.0` (Phase 2 completion marker)
- [ ] Nominal type system stable for 2 weeks (no type system bugs reported)
- [ ] No critical or high bugs open

### Deliverables

| ID | Deliverable | Measurable Criteria | Owner |
|----|-------------|---------------------|-------|
| P3-D1 | Actor thread pool | M:N scheduling; configurable pool size; 1000+ actors spawned in <1 second; gracefull shutdown | TBD |
| P3-D2 | Zero-copy SPSC channels | Lock-free single-producer single-consumer; 1M messages/sec throughput benchmarked; no `unsafe` data races (miri clean) | TBD |
| P3-D3 | Capability hardware gate | Runtime enforces compile-time capability tokens; unauthorized access panics in debug, returns error in release; 5+ integration tests | TBD |
| P3-D4 | Cross-shard messaging | Messages route between shards by affinity; no global lock on message dispatch; 3+ shard test | TBD |
| P3-D5 | Deterministic execution | Same input produces same output bit-for-bit; seeded RNG; deterministic scheduling order; 10+ determinism tests | TBD |
| P3-D6 | Actor supervision | Supervisor tree: one-for-one, one-for-all, rest-for-one strategies; 3+ supervision tests | TBD |
| P3-D7 | Runtime benchmarks | Criterion benchmarks for: spawn rate, message throughput, shard routing, capability check overhead | TBD |

### Audit Checklist

- [ ] **P3-A1** — Actor pool: `cargo test actor_pool` passes; spawn benchmark shows 1000+ actors in <1s
- [ ] **P3-A2** — SPSC channels: `cargo test spsc` passes; throughput benchmark >= 1M msg/sec
- [ ] **P3-A3** — Miri clean: `cargo miri test` passes on channel and actor code (no data races)
- [ ] **P3-A4** — Capability gates: `cargo test capability_runtime` passes; unauthorized access correctly rejected
- [ ] **P3-A5** — Cross-shard: `cargo test cross_shard` passes with 3+ simulated shards
- [ ] **P3-A6** — Determinism: `cargo test deterministic` passes; 10 tests verify bit-identical outputs
- [ ] **P3-A7** — Supervision: `cargo test supervision` passes; all 3 restart strategies verified
- [ ] **P3-A8** — Benchmarks: `cargo bench runtime` runs and reports baseline numbers
- [ ] **P3-A9** — No regression: All Phase 1+2 deliverables still pass
- [ ] **P3-A10** — Documentation: `docs/runtime.md` exists with architecture diagram and API reference

### Exit Proof

1. **Benchmark report** (`target/criterion/report/index.html` or CI artifact) showing runtime baselines
2. **Miri clean report** showing zero data races in actor/channel code
3. **Determinism test log** showing 10 consecutive identical outputs from seeded runs
4. **Auditor sign-off** in Phase 3 tracking issue

### Target Date

**No target date — ship when audit passes.**

### Open Issues

- Issue #07 — Capability security runtime (completion)
- Issue #08 — Sharded runtime messaging (completion)
- Issue #14 — Deterministic execution (completion)
- Issue #22 — Actor thread pool (new, from P3-D1)
- Issue #23 — Zero-copy SPSC channels (new, from P3-D2)
- Issue #24 — Actor supervision tree (new, from P3-D6)

---

## Phase 4: PLATFORMS (v1.70.x)

### Goal
Complete freestanding ports for aarch64 and riscv64, boot all three architectures in QEMU, and deliver a working WASM runtime with CI integration.

### Entry Criteria

> **ALL of the following must be true. Phase 4 work MUST NOT begin until signed off.**

- [ ] Phase 3 audit fully passed (all P3-A1 through P3-A10 ticked)
- [ ] Auditor sign-off recorded in Phase 3 tracking issue
- [ ] `main` branch tagged with `v1.69.0` (Phase 3 completion marker)
- [ ] Runtime stable for 2 weeks (no runtime crashes or data races reported)
- [ ] No critical or high bugs open

### Deliverables

| ID | Deliverable | Measurable Criteria | Owner |
|----|-------------|---------------------|-------|
| P4-D1 | aarch64 freestanding | Linker script + startup assembly + UART driver; compiles with `aarch64-unknown-none` triple | TBD |
| P4-D2 | riscv64 freestanding | Linker script + startup assembly + UART driver; compiles with `riscv64gc-unknown-none-elf` triple | TBD |
| P4-D3 | x86_64 QEMU boot (regression) | Still boots; still prints "Logicodex" (Phase 1 deliverable maintained) | TBD |
| P4-D4 | aarch64 QEMU boot | Boots in QEMU virt machine, prints "Logicodex aarch64" to PL011 UART | TBD |
| P4-D5 | riscv64 QEMU boot | Boots in QEMU virt machine, prints "Logicodex riscv64" to NS16550A UART | TBD |
| P4-D6 | WASM linker script | Custom linker script for WASM target; no emscripten dependency | TBD |
| P4-D7 | WASM runtime | Memory allocator + panic handler + basic libc replacements; `wasm32-unknown-unknown` target | TBD |
| P4-D8 | WASM CI integration | CI builds WASM target; WASM tests run in `wasmtime` or browser headless | TBD |
| P4-D9 | Multi-arch test matrix | CI tests x86_64, aarch64, and riscv64 in QEMU | TBD |
| P4-D10 | Raylib FFI completion | Coverage of core drawing + input + window APIs; example programs compile and run | TBD |

### Audit Checklist

- [ ] **P4-A1** — aarch64 compiles: `cargo build --target aarch64-unknown-none` succeeds for freestanding target
- [ ] **P4-A2** — riscv64 compiles: `cargo build --target riscv64gc-unknown-none-elf` succeeds for freestanding target
- [ ] **P4-A3** — x86_64 regression: QEMU x86_64 still boots and prints (Phase 1 maintained)
- [ ] **P4-A4** — aarch64 boot: QEMU aarch64 boots, serial output shows "Logicodex aarch64"
- [ ] **P4-A5** — riscv64 boot: QEMU riscv64 boots, serial output shows "Logicodex riscv64"
- [ ] **P4-A6** — WASM build: `cargo build --target wasm32-unknown-unknown` succeeds
- [ ] **P4-A7** — WASM tests: `cargo test --target wasm32-unknown-unknown` passes in WASM runtime
- [ ] **P4-A8** — WASM CI: CI job runs and passes for WASM target
- [ ] **P4-A9** — Multi-arch CI: CI runs QEMU tests for all 3 architectures
- [ ] **P4-A10** — Raylib FFI: Example programs compile and run; API coverage documented
- [ ] **P4-A11** — No regression: All Phase 1-3 deliverables still pass
- [ ] **P4-A12** — Documentation: `docs/platforms.md` with boot instructions for all 3 architectures + WASM

### Exit Proof

1. **CI matrix screenshot** showing green builds for x86_64, aarch64, riscv64, and WASM
2. **QEMU boot artifacts** for all 3 architectures with verifiable serial output
3. **WASM test run** showing tests passing in WASM runtime
4. **Auditor sign-off** in Phase 4 tracking issue

### Target Date

**No target date — ship when audit passes.**

### Open Issues

- Issue #10 — WASM linker + runtime (completion)
- Issue #11 — x86_64 freestanding (regression maintenance)
- Issue #12 — Raylib FFI completion (completion)
- Issue #15 — aarch64 freestanding (completion)
- Issue #16 — riscv64 freestanding (completion)
- Issue #25 — aarch64 linker script + startup (new, from P4-D1)
- Issue #26 — riscv64 linker script + startup (new, from P4-D2)
- Issue #27 — WASM memory allocator (new, from P4-D7)

---

## Phase 5: PRODUCTION (v2.0.x)

### Goal
Deliver a package manager, achieve self-hosting, publish formal semantics, and cut a stable (non-alpha) release.

### Entry Criteria

> **ALL of the following must be true. Phase 5 work MUST NOT begin until signed off.**

- [ ] Phase 4 audit fully passed (all P4-A1 through P4-A12 ticked)
- [ ] Auditor sign-off recorded in Phase 4 tracking issue
- [ ] `main` branch tagged with `v1.99.0` (Phase 4 completion marker / pre-production)
- [ ] All architectures stable for 2 weeks
- [ ] No critical or high bugs open
- [ ] Compiler public API documented (for self-hosting boundary)

### Deliverables

| ID | Deliverable | Measurable Criteria | Owner |
|----|-------------|---------------------|-------|
| P5-D1 | `ldx-pkg` package manager | Init, add, remove, build, test, publish commands; SemVer resolution; lockfile; registry protocol defined (Issue #18) | TBD |
| P5-D2 | Self-hosting compiler | Logicodex compiler can compile itself; output passes `cargo test` equivalent; 3-stage bootstrap verified | TBD |
| P5-D3 | Formal semantics | Operational semantics for core language published as PDF in `docs/formal/`; covers: expressions, types, effects, actor semantics | TBD |
| P5-D4 | Stable release | Version `v2.0.0` (non-alpha); changelog complete; migration guide from v1.x | TBD |
| P5-D5 | API stability | Public API semver-guaranteed; `#[stable]` attribute on public items; deprecation policy documented | TBD |
| P5-D6 | Ecosystem seed | 3+ example programs published as `ldx-pkg` packages; standard library documented | TBD |
| P5-D7 | Network reactor completion | Full socket lifecycle: bind, listen, accept, connect, send, recv, close; async/await integration | TBD |

### Audit Checklist

- [ ] **P5-A1** — Package manager: `ldx-pkg init`, `add`, `remove`, `build`, `test` all work; SemVer resolution produces correct dependency trees
- [ ] **P5-A2** — Self-hosting: 3-stage bootstrap successful (stage0=Rust, stage1=ldx, stage2=ldx-from-ldx); stage2 output matches stage1 bit-for-bit or documented diff
- [ ] **P5-A3** — Formal semantics: PDF exists in `docs/formal/semantics.pdf`; reviewed by at least one external reviewer
- [ ] **P5-A4** — Stable release: GitHub release `v2.0.0` exists; NOT marked as pre-release
- [ ] **P5-A5** — API stability: `#[stable]` attributes present; deprecation policy in `docs/stability.md`
- [ ] **P5-A6** — Ecosystem: 3+ packages installable via `ldx-pkg install`
- [ ] **P5-A7** — Network reactor: `cargo test network` passes; socket lifecycle fully tested
- [ ] **P5-A8** — No regression: All Phase 1-4 deliverables still pass (all platforms, all runtimes)
- [ ] **P5-A9** — Documentation: All user-facing docs complete: language reference, stdlib docs, package manager guide
- [ ] **P5-A10** — Security audit: Third-party security review completed or scheduled (see `SECURITY.md`)

### Exit Proof

1. **Bootstrap log** showing 3-stage build with checksums
2. **GitHub release page** for `v2.0.0` showing non-alpha, non-pre-release
3. **Formal semantics PDF** committed and linked
4. **Package manager demo** installing a published package
5. **Auditor sign-off** in Phase 5 tracking issue
6. **Post-mortem document** in `docs/postmortem/v2.0.0.md`

### Target Date

**No target date — ship when audit passes.**

### Open Issues

- Issue #05 — LSP engine (production hardening)
- Issue #09 — Network reactor (completion)
- Issue #17 — Self-hosting (primary deliverable)
- Issue #18 — Package manager (primary deliverable)
- Issue #28 — Formal semantics document (new, from P5-D3)
- Issue #29 — 3-stage bootstrap (new, from P5-D2)
- Issue #30 — API stability framework (new, from P5-D5)

---

## Cross-Phase Regression Requirements

Every phase must maintain all previous phase deliverables. The regression test suite grows monotonically:

| Phase | Regression Tests Required |
|-------|--------------------------|
| Phase 2 | All Phase 1 tests (Tier A+B, QEMU x86_64) |
| Phase 3 | All Phase 1+2 tests |
| Phase 4 | All Phase 1+2+3 tests + multi-arch QEMU |
| Phase 5 | All Phase 1-4 tests + self-hosting bootstrap |

### Regression Failure Policy

If a regression is detected during a phase:
1. Phase work PAUSES on new features
2. Regression is P0 priority
3. Fix must land before any new deliverable work resumes
4. Auditor must re-validate the fix

---

## Risk Register

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Single maintainer burnout | HIGH | CRITICAL | Co-maintainer recruitment starts Phase 1; documented in CONTRIBUTING.md |
| x86_64 QEMU boot harder than expected | MEDIUM | HIGH | Break into micro-deliverables: linker script, then assembly, then UART, then integration |
| HIR decision causes scope creep | MEDIUM | HIGH | Decision document MUST fit on 1 page; no HIR redesign in Phases 2-5 |
| WASM runtime requires async ecosystem | MEDIUM | MEDIUM | Scope WASM to sync-only for Phase 4; async deferred to Phase 5 |
| Self-hosting requires full stdlib | HIGH | HIGH | Self-hosting scope limited to compiler subset; stdlib built incrementally |
| Deterministic execution performance cost | MEDIUM | MEDIUM | Benchmark baseline in Phase 3; if >10% overhead, document and gate behind feature flag |
| aarch64/riscv64 hardware access limited | LOW | MEDIUM | QEMU-only for Phase 4; real hardware deferred to post-v2.0 |

---

## Glossary

| Term | Definition |
|------|------------|
| **HIR** | High-level Intermediate Representation — the dormant compiler IR |
| **Capability** | Security primitive: unforgeable token proving authorization |
| **Shard** | Isolated memory + thread domain in the sharded runtime |
| **SPSC** | Single-Producer Single-Consumer lock-free queue |
| **Freestanding** | Bare-metal target with no OS (no libc, no std) |
| **QEMU** | Emulator used for bare-metal testing without hardware |
| **Self-hosting** | Compiler written in its own source language |
| **Miri** | Rust's undefined behavior detector (used for `unsafe` audit) |
| **Tier A/B/C** | Validator tiers: A=required, B=important, C=nice-to-have |
| **RFC** | Request for Comments — formal proposal process |

---

## Document History

| Version | Date | Change |
|---------|------|--------|
| 1.0 | 2024 | Original aspirational roadmap (superseded) |
| 2.0 | 2025-07-15 | Phase-gated rewrite based on audit findings; honest maturity matrix; strict entry/exit criteria |

---

*This roadmap is a living document. Changes require RFC per `ROADMAP_POLICY.md`. No dates are promised. Ship when audit passes.*
