# Logicodex Undefined Behavior and Pointer Provenance Design Baseline (v1.21-alpha)

This document records the current design baseline for Undefined Behavior and Pointer Provenance in **current logicodex v 1.21 alpha**. It is intentionally written as a practical specification target rather than a claim that every long-term safety mechanism is already complete.

## 1. Industry-Derived Layer Classification

Logicodex categorizes semantic violations using familiar low-level language concerns so future LLVM backend work can map checks and diagnostics into explicit compiler behavior.

| Layer | Current purpose | Long-term objective |
|---|---|---|
| Linear layer | Describe raw pointer arithmetic, memory-mapped offsets, and volatile I/O boundaries. | Preserve provenance information precisely enough that optimization does not erase required hardware or aliasing constraints. |
| Object-oriented layer | Describe flat struct layouts, deterministic placement, and scoped cleanup goals. | Detect object-boundary violations such as expired memory reuse or double cleanup once those features are implemented. |
| Safety layer | Describe bounds checking, resource cleanup, and safer default behavior. | Expand compile-time and runtime checks while measuring overhead and documenting target-specific behavior. |

## 2. Severity Classification Baseline

The severity model is a roadmap for classifying responses, not a blanket claim of zero runtime cost. Each tier should be implemented, benchmarked, and tested before being described as ready for production use.

| Tier | Description | Practical current framing |
|---|---|---|
| Tier 1: Critical | Intended for executable-integrity failure or unsafe hardware-region access in explicitly selected freestanding contexts. | Treat as a long-term fail-stop objective requiring target-specific implementation, review, and tests. |
| Tier 2: Medium | Intended for dynamic division by zero, runtime resource depletion, or isolated execution failure. | Implement as normal process or function failure paths first, then improve cleanup behavior as the runtime matures. |
| Tier 3: Low | Intended for warnings such as safe integer truncation, benign wrap-around, or deprecated library use. | Prefer diagnostics and metadata until a concrete runtime behavior is necessary and measured. |

## 3. Documentation Rule

Future documentation should distinguish **implemented compiler behavior**, **prototype behavior**, and **long-term objectives**. Stronger terms about verification, overhead, forced termination, or hardware readiness should be used only when the repository contains corresponding source implementation, tests, and build instructions.
