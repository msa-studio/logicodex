# ❖ Logicodex Undefined Behavior & Pointer Provenance Specification (v1.21-alpha)

## 1. Industry-Derived Layer Classification
Logicodex categorizes semantic violations based on established low-level language paradigms to facilitate seamless optimization mapping via the LLVM backend:
- **Linear Layer (C-Style Paradigms):** Focuses on raw pointer arithmetic, memory-mapped offsets, and volatile I/O actions. Pointer offsets (e.g., `ptr + 5`) preserve Strict Provenance. Hardcoded literal casts via `addr` generate Hardware/Wild Provenance, forcing the optimizer to assume aliasing and bypass unsafe memory optimization drops.
- **Object-Oriented Layer (C++ Style Paradigms):** Focuses on flat struct layouts, deterministic sequential memory placement, and scoped destructor functions (drop semantics). Re-use of expired memories or double execution of object destruction logic is strictly treated as an explicit object boundary violation.
- **Safety Layer (Rust-Style Paradigms):** Focuses on strict compile-time index bounds checking and deterministic automatic resource cleanup via RAII patterns.

## 2. Zero-Overhead General Error Severity Classification
If a runtime error escapes static compilation analysis or triggers during active runtime attestation, the compiler handles response routines through three structural severity tiers injected directly into the LLVM IR pipeline:
- **🔴 TIER 1: CRITICAL (Hardware & Machine Layer Rupture):** Triggers upon Golden Hash integrity failure or unmapped physical memory access in freestanding mode. It bypasses hosted OS routines, emitting naked machine instructions to trigger an immediate process termination or standard signal (Hosted) or forces a CPU Triple Fault / Hardware Watchdog Reset (Freestanding Bare-Metal) to completely freeze the execution environment and contain threats.
- **🟡 TIER 2: MEDIUM (Process & Execution Layer Failure):** Triggers upon dynamic division by zero, runtime resource depletion, or structural thread deadlocks. It terminates the active thread or isolated sub-process cleanly, returns a panic exit code (e.g., `exit(1)`), and flushes resource drops to standard error logs without bringing down the machine.
- **🟢 TIER 3: LOW (Non-Critical Warning Layer):** Triggers upon safe casting integer truncation, benign unsigned math wrap-around, or deprecated library calls. It operates at zero execution speed deduction, emitting a standard error diagnostic trace or shifting execution safely into user-defined localized `catch` statement blocks.
