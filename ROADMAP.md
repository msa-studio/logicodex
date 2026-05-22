# Logicodex Practical Roadmap — current logicodex v 1.21 alpha

This roadmap describes how **Logicodex** should progress from the current alpha compiler baseline toward a more complete systems-language ecosystem. It intentionally separates what is implemented today from what remains a prototype objective or a long-term research goal.

> **Roadmap principle:** Logicodex should earn stronger claims through reproducible builds, executable examples, validation scripts, measured performance, and clearly documented safety boundaries.

| Horizon | Primary focus | Risk addressed | Acceptance signal |
|---|---|---|---|
| Short term | Compiler-core stability | The language vision outpaces verified implementation. | Rust 1.75 builds, validators, release artifacts, and representative `.ldx` examples pass consistently. |
| Medium term | Tooling and specification discipline | Contributors lack a stable editing and diagnostic workflow. | Formatter, LSP diagnostics, type-system rules, and unsafe capability gates are documented and tested. |
| Long term | Systems and security research | Security and freestanding objectives are mistaken for completed production features. | WebAssembly, migration tooling, runtime attestation, and freestanding examples are implemented with benchmarks and target-specific tests. |

## Milestone 1: Stabilize the Alpha Compiler Core

The first priority is to keep the current compiler pipeline reproducible. This means preserving Rust Edition 2021 compatibility, retaining the Rust 1.75 validation floor, maintaining the pinned LLVM 15 dependency path, and expanding executable examples that demonstrate lexing, parsing, semantic checks, LLVM IR generation, and native-oriented object output.

| Issue | Status | Owner | Practical acceptance signal |
|---|---|---|---|
| Issue #01 — Grammar baseline | Completed for v1.21-alpha baseline | Mohamad Supardi Abdul | The grammar document matches lexer/parser behavior for the currently implemented language subset. |
| Issue #02 — UB and provenance design note | Completed as specification baseline | Mohamad Supardi Abdul | The document clearly identifies intended safety boundaries without claiming all long-term runtime behavior is already implemented. |
| Issue #03 — Native example suite | Open | TBD | A small set of `.ldx` programs compiles through the documented pipeline and has expected-output checks. |
| Issue #04 — CI-oriented validation | Open | TBD | `cargo check`, release build, and validation scripts can be run from a clean checkout with documented dependencies. |

## Milestone 2: Tighten Language Semantics and Diagnostics

The second priority is to make the language easier to reason about. The project should document nominal typing, inference boundaries, casts, unsafe capability gates, and target-specific restrictions before adding broad new language features.

| Issue | Status | Owner | Practical acceptance signal |
|---|---|---|---|
| Issue #05 — Nominal type system boundaries | Open | TBD | Type rules distinguish inference, explicit annotations, casts, and compiler-enforced invariants. |
| Issue #06 — Pointer and hardware-region gates | Open | TBD | Hosted and freestanding memory operations are separated by explicit syntax, diagnostics, and examples. |
| Issue #07 — Diagnostic quality pass | Open | TBD | Common user errors produce actionable messages with source locations and suggested fixes. |

## Milestone 3: Build Developer Tooling

Developer tooling should follow the stabilized language subset rather than racing ahead of it. Formatting and editor feedback will make the dual-syntax concept easier to maintain in collaborative use.

| Issue | Status | Owner | Practical acceptance signal |
|---|---|---|---|
| Issue #08 — `ldx-fmt` formatter | Open | TBD | Representative Logicodex examples are formatted into a canonical style without changing meaning. |
| Issue #09 — LSP diagnostics | Open | TBD | Syntax and semantic feedback work in at least one supported editor. |
| Issue #10 — Documentation examples | Open | TBD | README, manual, and specification examples are synchronized with compiler behavior. |

## Milestone 4: Prototype Portable Targets

WebAssembly and cross-platform targets remain valuable objectives, but they should be introduced through small prototypes with clear limitations. The project should avoid describing a target as supported until a representative program can be built and executed in that environment.

| Issue | Status | Owner | Practical acceptance signal |
|---|---|---|---|
| Issue #11 — WebAssembly prototype | Long-term objective | TBD | A documented `.wasm` generation path can compile and run one representative Logicodex program. |
| Issue #12 — Cross-platform benchmark harness | Long-term objective | TBD | Benchmarks are reproducible, documented, and runnable across declared target platforms. |
| Issue #13 — Release artifact refresh workflow | Open | TBD | Archives are regenerated only after build and validator evidence is captured. |

## Milestone 5: Treat Security and Freestanding Work as Research Objectives

Runtime memory attestation, Golden Hash planning, hard fail-stop behavior, and freestanding hardware access are ambitious directions. They should be framed as **research and engineering objectives** until implemented, reviewed, and measured. The near-term task is to define threat models, safe opt-in flags, target-specific behavior, and tests that prove the compiler emits the expected runtime hooks.

| Objective | Current status | Long-term success signal |
|---|---|---|
| Runtime memory attestation | Design contract and plan generation | Digest insertion, verifier stubs, threat model, overhead measurements, and tamper tests exist. |
| Fail-stop mitigation | Roadmap model | Hosted process abort and freestanding halt/reset behavior are implemented only where safe, documented, and explicitly selected. |
| Freestanding support | Experimental target profile | Linker scripts, bootloader integration notes, hardware-region policies, and minimal examples are validated. |
| Migration assistant | Conceptual roadmap | Translation output is reviewable, testable, and clearly marked as assisted migration rather than automatic proof of correctness. |

## Tracking Notes

The roadmap should be updated only when implementation evidence changes. Completed items should cite the files, tests, examples, or release assets that prove the claim. When an item remains a goal, the documentation should use terms such as **planned**, **prototype**, **experimental**, **research objective**, or **long-term objective** rather than implying production readiness.
