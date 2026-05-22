# ❖ Logicodex Phase 2 Execution Roadmap

This roadmap tracks the open issues and architectural milestones established to transition **Logicodex** from a research/compiler initiative into a production-ready ecosystem, directly responding to the **V1.11-alpha technical evaluation**. For Phase 2, Milestone 1, it also records the completion status of Issue #01 and links the formal EBNF grammar artifact checked into the repository.

> **Roadmap principle:** Phase 2 must reduce execution risk by formalizing the language specification, enforcing canonical project conventions, and validating deployment targets through measurable tooling and benchmark work.

| Milestone | Priority | Primary Risk Addressed | Expected Outcome |
|---|---:|---|---|
| Formal Language Specification | High | Ambiguous syntax, undefined behavior, and type-system uncertainty | A stable specification baseline suitable for compiler, tooling, and contributor alignment. |
| Tooling & Canonical Convention Enforcement | High | Syntax fragmentation across novice, localized, and expert shorthand styles | Automated formatting and editor feedback that normalize collaborative development workflows. |
| Executable Benchmarking & Web-Native Deployments | Medium | Unverified performance and deployment claims | Reproducible benchmark suites and WebAssembly generation paths for browser-native execution. |

## 🎯 Milestone 1: Formal Language Specification (High Priority)

- [X] **Issue #01 — Formal EBNF Grammar Definition (COMPLETED / SOLVED):** Document the exact grammar rules for both **Novice Pseudocode** and **Expert Shorthand** to eliminate parsing ambiguity.
- [ ] **Issue #02 — Undefined Behavior Catalog & Pointer Provenance:** Define explicit rules governing raw pointer operations, physical memory-mapped boundaries, hosted-memory isolation, and freestanding memory-access constraints.
- [ ] **Issue #03 — Nominal Type System & Type Inference Boundaries:** Formalize rules for variable implicit annotations, nominal typing, strict type-checking invariants, and the boundaries between inference, explicit declaration, and unsafe operations.

## 🛠️ Milestone 2: Tooling & Canonical Convention Enforcement

- [ ] **Issue #04 — Development of `ldx-fmt` (The Logicodex Formatter):** Implement an automated code formatter to mitigate the risk of syntax fragmentation across collaborative multi-syntax environments.
- [ ] **Issue #05 — Language Server Protocol (LSP) Engine:** Build a baseline LSP server to inject real-time syntax checking, auto-completion, and compiler diagnostic feedback directly inside code studios such as **VS Code** and **Cursor**.

## 🚀 Milestone 3: Executable Benchmarking & Web-Native Deployments

- [ ] **Issue #06 — Bare-Metal & Cross-Platform Benchmarks:** Produce executable multi-platform test suites demonstrating native binary behavior and performance characteristics under LLVM optimization tiers, including comparative benchmark cases against established systems languages where applicable.
- [ ] **Issue #07 — WebAssembly Target Generation:** Implement direct `.wasm` generation pathways using the LLVM backend for browser-native execution and portable sandboxed deployment scenarios.

## Phase 2 Tracking Notes

This roadmap converts the evaluator’s recommendations into explicit project-management checkpoints. The milestones should be tracked as discrete implementation issues before additional production-readiness claims are made. The immediate emphasis is on specification discipline: **grammar formalization**, **undefined-behavior boundaries**, **type-system clarity**, and **canonical formatting conventions** must precede broader ecosystem expansion.

| Issue | Status | Owner | Acceptance Signal |
|---|---|---|---|
| Issue #01 | [X] COMPLETED / SOLVED | Mohamad Supardi Abdul (MSA Studio) | 1. Formal 4-Layer grammar checked in as a living document inside `spec/v1.11-alpha/UpdateIssue1-ebnf.md`.<br>2. Recursive-descent compiler entry pipeline verified to ingest token maps collision-free.<br>3. Concrete freestanding token productions (`hw` and `addr`) structurally declared to enable upcoming security capability gates. |
| Issue #02 | Open | TBD | A UB and pointer-provenance catalog defines allowed, unsafe, and forbidden memory behaviors. |
| Issue #03 | Open | TBD | Type rules distinguish inference, nominal declarations, casts, and compiler-enforced invariants. |
| Issue #04 | Open | TBD | `ldx-fmt` can format representative Logicodex examples into one canonical style. |
| Issue #05 | Open | TBD | LSP diagnostics, completion, and syntax feedback work in at least one supported editor. |
| Issue #06 | Open | TBD | Benchmarks are reproducible, documented, and runnable across the declared target platforms. |
| Issue #07 | Open | TBD | A documented `.wasm` generation path can compile and run a representative Logicodex program. |
