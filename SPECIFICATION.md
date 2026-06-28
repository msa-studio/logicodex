> ⚠️ **NOT UPDATED — will revisit.** This document predates the current syntax/architecture and may contain stale information. Authoritative current references: `README.md`, `examples/`, and `docs/architecture/`. Tracked under `docs/DOCUMENTATION_POLICY.md`.

# Logicodex Specification

## v0.46.0-alpha — The Deterministic Systems Programming Language

**Author:** Mohamad Supardi Abdul (`mymsastudio@gmail.com`)  
**Status:** Alpha  
**Last Updated:** 2026-05-25

> **Scope:** This document is the **single specification source** for Logicodex. It defines the language, architecture, roadmap, and governance. For implementation history, see [`CHANGELOG.md`](CHANGELOG.md). For user tutorials and API reference, see [`docs/HANDBOOK.md`](docs/HANDBOOK.md). For detailed philosophical justifications, see [`docs/white-paper/`](docs/white-paper/).

---

## Table of Contents

1. [Philosophy](#1-philosophy)
2. [Language Specification](#2-language-specification)
3. [Architecture](#3-architecture)
4. [Capability Security Model](#4-capability-security-model)
5. [Concurrency Model](#5-concurrency-model)
6. [Compilation Targets](#6-compilation-targets)
7. [Roadmap](#7-roadmap)
8. [Governance](#8-governance)
9. [References](#9-references)

---

## 1. Philosophy

### 1.1 The Problem

Software engineering is polarized: high-level languages are easy to write but slow; systems languages are fast but hostile to humans and AI. Logicodex proposes a **third path**: syntax adapts to the user, semantics remain statically checked and native.

```text
[ Malay/English Alias Code (.ldx) ] ──► (Lexer + core_map.json) ──► [ Unified Token Stream ]
                                                                            │
[ Expert Canonical Code (.ldx) ] ──► (Lexer + core_map.json) ──► [ Abstract Syntax Tree ]
                                                                           │
[ Native Binary ] ◄── (LLVM Backend O3) ◄── [ LLVM IR Generation ]
```

### 1.2 Core Principles

| Principle | Meaning | Enforcement |
|---|---|---|
| **Determinism** | Same input → same output, every time | Static topology, explicit ownership, shard isolation |
| **Zero Runtime Mediation** | No GC, no VM, no interpreter | All checks at compile time |
| **Progressive Disclosure** | Beginner starts simple, expert gets full control | Alias-to-canonical lexing |
| **Capability Security** | Dangerous operations require explicit permission | Compile-time gates, zero runtime cost |
| **Alias-to-Canonical** | One language, many surfaces, single AST | `dict/core_map.json` normalization |

### 1.3 Syntax Surfaces

All surfaces compile to the same AST through `dict/core_map.json`:

| Malay | English | Canonical | Canonical Token |
|---|---|---|---|
| `MULA` | `BEGIN` / `START` | `{` | `BeginBlock` |
| `TAMAT` | `END` / `FINISH` | `}` | `EndBlock` |
| `BINA` | `CREATE` / `LET` | `let` | `Let` |
| `PAPAR` | `DISPLAY` / `PRINT` | `print` | `Print` |
| `PULANG` | `RETURN` | `return` | `Return` |
| `FUNGSI` | `FUNCTION` | `fn` | `Fn` |
| `JIKA` | `IF` | `if` | `If` |

> **Key:** The canonical token is what the parser sees. Malay, English, and shorthand are all surfaces into the same compiler frontend. See HANDBOOK § Syntax for the full alias table.

---

## 2. Language Specification

### 2.1 EBNF Grammar

```bnf
program         ::= header block
header          ::= ("program" | "PROGRAM") identifier
block           ::= begin_stmt stmt* end_stmt
begin_stmt      ::= "MULA" | "BEGIN" | "START" | "{"
end_stmt        ::= "TAMAT" | "END" | "FINISH" | "}"
stmt            ::= var_decl
                  | assignment
                  | if_stmt
                  | while_stmt
                  | for_stmt
                  | match_stmt
                  | return_stmt
                  | block
                  | print_stmt
                  | unsafe_stmt
                  | actor_decl
                  | service_decl
                  | channel_decl
                  | spawn_stmt
                  | send_stmt
                  | recv_stmt
                  | join_stmt
                  | use_type_stmt

var_decl        ::= ("BINA" | "CREATE" | "let") identifier ("SEBAGAI" | "AS") type ("=" expr)?
assignment      ::= identifier "=" expr
if_stmt         ::= if_header expr block (else_if_clause)* (else_clause)?
if_header       ::= "JIKA" | "IF"
else_if_clause  ::= "LAIN_JIKA" | "ELSE_IF" | "else if" expr block
else_clause     ::= "LAIN" | "ELSE" | "else" block
while_stmt      ::= ("SEMENTARA" | "WHILE") expr block
for_stmt        ::= ("UNTUK" | "FOR") identifier ("DARI" | "IN") expr block
match_stmt      ::= ("PADAN" | "MATCH") expr "{" match_arm* "}"
match_arm       ::= pattern "=>" (expr | block)
return_stmt     ::= ("PULANG" | "RETURN" | "return") expr?
print_stmt      ::= ("PAPAR" | "PRINT" | "DISPLAY" | "print") expr
unsafe_stmt     ::= ("ZON_TIDAK_SELAMAT" | "hw_unsafe") block
actor_decl      ::= ("actor" | "PELAKON") identifier "{" actor_body "}"
service_decl    ::= ("service" | "PERKHIDMATAN") identifier "{" service_body "}"
channel_decl    ::= ("channel" | "SALURAN") "<" identifier "," identifier "," type ">"
spawn_stmt      ::= ("spawn" | "HIDUPKAN") identifier "(" arg_list? ")"
send_stmt       ::= identifier "." ("send" | "hantar") "(" expr ")"
recv_stmt       ::= identifier "." ("recv" | "terima") "("
join_stmt       ::= ("join" | "sertai") identifier
use_type_stmt   ::= ("GUNA_JENIS" | "USE_TYPE") type

expr            ::= term (("+" | "-" | "||") term)*
term            ::= factor (("*" | "/" | "%" | "&&") factor)*
factor          ::= number | string | bool | identifier
                  | "(" expr ")"
                  | "-" factor
                  | "!" factor
                  | function_call
                  | field_access
                  | array_index

function_call   ::= identifier "(" arg_list? ")"
field_access    ::= identifier "." identifier
array_index     ::= identifier "[" expr "]"
arg_list        ::= expr ("," expr)*
pattern         ::= identifier ("::" identifier)?
                  | identifier "(" pattern_args? ")"
                  | "_"

number          ::= integer | float
integer         ::= digit+ ("i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32" | "u64")?
float           ::= digit+ "." digit+ ("f32" | "f64")?
string          ::= "\"" char* "\""
bool            ::= ("BENAR" | "true") | ("PALSU" | "false")
identifier      ::= letter (letter | digit | "_")*
type            ::= "I32" | "I64" | "F64" | "Bool" | "Text" | "Void"
                  | identifier  /* user-defined types */

letter          ::= "a".."z" | "A".."Z" | "_"
digit           ::= "0".."9"
char            ::= any Unicode character except "\""
```

### 2.2 Type System

| Type | Size | Range / Notes |
|---|---|---|
| `I8` | 8-bit | -128 to 127 |
| `I16` | 16-bit | -32,768 to 32,767 |
| `I32` | 32-bit | -2,147,483,648 to 2,147,483,647 |
| `I64` | 64-bit | -9×10¹⁸ to 9×10¹⁸ |
| `F32` | 32-bit | IEEE 754 single |
| `F64` | 64-bit | IEEE 754 double |
| `Bool` | 1-bit | `true` / `false` |
| `Text` | variable | UTF-8 string |
| `Void` | — | Return type for no value |

Composite types: `struct`, `enum`, `array[`, `]`, `Channel<T>`, `Option<T>`, `Result<T,E>`.

> Status note: `Option<T>` and `Result<T,E>` are accepted in the surface grammar,
> but only the monomorphic `Option<I64>` and `Result<I64,I64>` forms are currently
> compiler-proven and contract-backed (Stage 1 `core.option` / `core.result`).
> Generic `T` / `E` payloads, combinators, and non-I64 error payloads parse but are
> not yet lowered. See `docs/architecture/result-option-foundation.md`.

Pointer types: `&T` (safe reference), `*T` / `PTR<T>` (raw pointer, freestanding only).

### 2.3 Semantic Checks

The semantic analyzer enforces:

| Check | Description | Error Code |
|---|---|---|
| **Type checking** | No implicit conversions; explicit `SEBAGAI` cast required | E001 |
| **Division by zero** | Detected at compile time for constant expressions | E002 |
| **Name resolution** | All identifiers must be declared before use | E003 |
| **UseAfterSend** | Cannot use a value after sending it through a channel | E004 |
| **UseAfterMove** | Cannot use ownership after it has been transferred | E005 |
| **Capability gate** | Cannot call dangerous operations without declaring the gate | E006 |
| **Audio violation** | Audio callbacks cannot do I/O, recursion, or allocation | E007 |
| **WASM hardware gate** | Hardware gates forbidden in WASM target | E008 |

> Diagnostics status. The codes `E001`–`E008` above name the intended semantic
> checks. What is implemented today is a structured `Diagnostic` type
> (`src/span.rs`) with bilingual `message_ms` / `message_en`, a `Severity`
> (Error / Warning / Info), an optional primary span, and notes. The backing
> `DiagnosticCode` enum currently covers `TypeMismatch`, `DivisionByZero`,
> `DuplicateDefinition`, `LayoutError`, `ParserUnsupportedFeature`,
> `FfiBoundaryViolation`, and `UnsafeBoundaryViolation`; these are not yet wired
> to literal `E0xx` strings. Plain `ParseError` / `CompileError` paths also still
> exist. The richer "diagnostic intelligence" layer (LDX-DIP: AI-queryable,
> contract-fact-driven, self-treatment suggestions) is a roadmap goal, not an
> implemented subsystem.

---

## 3. Architecture

### 3.1 Compiler Pipeline

```
.ldx Source ──► Lexer ──► Parser ──► AST ──► Semantic ──► HIR ──► CapabilityGraph ──► LLVM IR ──► Object/Binary
               ↑            ↑         ↑           ↑            ↑            ↑
         core_map.json  EBNF grammar  TypeRegistry  Ownership    Topology     Target triple
                                         Coercion      tracking    verify       (native/wasm/freestanding)
```

| Stage | Key Files | Responsibility |
|---|---|---|
| **Lexer** | `src/lexer.rs` | Tokenization with `core_map.json` alias normalization |
| **Parser** | `src/parser.rs` | Recursive-descent → AST |
| **Semantic** | `src/semantic.rs` | Type check, ownership, capability, audio safety |
| **HIR** | `src/hir.rs` | High-level IR (v1.36) for structured codegen |
| **CapabilityGraph** | `src/tier2/capability_ir.rs` | Unified IR: service + gate + shard + door nodes |
| **CTL Mapper** | `src/tier2/ctl_mapper.rs` | Capability → WIT mapping for WASM |
| **Codegen** | `src/codegen.rs` | AST → LLVM IR → object file |

### 3.2 Key Design Decisions

| Decision | Chosen Approach | Why | Alternatives Rejected |
|---|---|---|---|
| **Concurrency model** | Actor + SPSC channel + static shard | Deterministic, zero race, >85% scaling | Thread+mutex (non-deterministic), async/await (colored functions) |
| **Memory management** | Explicit ownership + RAII | Zero runtime cost, no GC pause | GC (Go/Java — pause), Borrow checker (Rust — cognitive overhead) |
| **Security model** | Compile-time capability gates | Zero runtime cost, fail-safe | Runtime permission check (overhead), Sandbox only (not fine-grained) |
| **Backend** | LLVM via Inkwell | Mature optimizer, multi-target | Custom backend (5-10 year gap) |
| **FFI** | C ABI through `extern "C"` + safe wrappers | Raylib (54 functions) working | WASM-only FFI (limited) |
| **Syscalls** | Direct syscall (no libc) | Zero dependency, deterministic | libc (glibc version issues) |

### 3.3 Repository Structure

```
logicodex/
├── src/                    # Rust compiler source (~19,600 LOC)
│   ├── lexer.rs            # Dictionary-aware tokenizer
│   ├── parser.rs           # Recursive-descent parser
│   ├── semantic.rs         # Type/ownership/capability checker
│   ├── hir.rs              # High-level IR
│   ├── codegen.rs          # LLVM backend
│   ├── ffi/                # FFI bindings (Raylib, C ABI)
│   ├── tier2/              # Capability + IR (gate, topology, shard, CTL)
│   ├── net/                # Network reactor (epoll, connection, taint)
│   └── os/                 # OS primitives (syscall, startup, allocator, UART, IDT)
├── lib/                    # Logicodex standard library (~2,000 LOC)
│   ├── core/               # Thread, sync, ring_buffer, scheduler, memori, result, file
│   ├── std/                # Audio types
│   └── startup/            # Multiboot header, linker scripts
├── dict/core_map.json      # Alias-to-canonical mapping
├── tests/                  # Unit tests (~9,230 LOC, 400+ assertions)
├── scripts/validators/     # 148 validation checks (Tier A/B/C)
├── benches/                # 4-layer benchmark framework
├── docs/
│   ├── HANDBOOK.md         # ⬅ User guide, tutorials, API reference
│   ├── white-paper/        # ⬅ Detailed philosophical justifications (wiki)
│   └── guide/              # ⬅ Comprehensive function reference (wiki)
├── SPECIFICATION.md        # ⬅ This document: spec + architecture + roadmap
├── CHANGELOG.md            # ⬅ Version history and decision log
└── README.md               # ⬅ Entry point
```

---

## 4. Capability Security Model

Logicodex uses **static capability fabric** — all security checks happen at compile time, leaving **zero runtime cost**.

### 4.1 Gate Types

| Gate Type | Use Case | Example |
|---|---|---|
| **DirectCall** | Inline-able safe functions | `math::sqrt()` |
| **Message** | Async SPSC communication | Network I/O, file access |
| **Hardware** | Bare-metal operations only | GPIO, DMA, Timer |

### 4.2 Service Manifest

```logicodex
service WebServer {
    port: 8080,
    requires: [Net.Admin, Storage.Read("/www")],
    handler: handle_http,
    policy: Block,
}
```

### 4.3 Verification

The topology verifier checks 5 invariants before compilation:

1. No actor duplication across shards
2. No contract violation (calling without required gate)
3. No orphan actors (unassigned to any shard)
4. No cycles in the door graph
5. No empty shards

If any check fails, **compilation is aborted**.

### 4.4 Taint FSM

Each network connection has a state machine:

```
Healthy ──error──▶ Suspicious ──threshold──▶ Closing ──▶ close(fd)
   ▲                    │ (recovery)
   └────────────────────┘
```

| State | Meaning | I/O Allowed? |
|---|---|---|
| **Healthy** | Normal operation | Yes |
| **Suspicious** | Errors increasing | Yes (monitored) |
| **Closing** | Threshold exceeded | **No** — cleanup only |

### 4.5 Audit Trail

Every compilation produces a `.cap` file recording all gates used:

```cap
[service WebServer]
port=8080
handler=handle_http
policy=Block

[gate Net.Admin]
domain=Net
operation=Admin
verified=true
checksum=sha256:a1b2c3...
```

### 4.6 Supply-Chain Security

The `diff_topology()` function compares two `.cap` files and detects privilege escalation (new gates not present in the baseline).

---

## 5. Concurrency Model

### 5.1 Actor

An actor is an isolated computation unit with its own state. Actors communicate only through message passing — no shared mutable state.

```logicodex
actor Worker {
    let ch: Channel<Worker, Collector, Data>
    // process messages...
}
```

### 5.2 Channel (SPSC Ring Buffer)

Single-producer, single-consumer ring buffer using atomic operations:

```
Ring Buffer (capacity = N)
┌─────────────────────────────────┐
│ [0]  [1]  [2]  ...  [N-1]      │
│  │    │    │         │          │
│  ▼    ▼    ▼         ▼          │
│ ┌─┐  ┌─┐  ┌─┐      ┌─┐         │
│ │A│  │B│  │ │  ... │ │         │
│ └─┘  └─┘  └─┘      └─┘         │
│  ▲              ▲               │
│  │              │               │
│ producer    consumer            │
│ (head++)    (tail++)            │
└─────────────────────────────────┘
```

| Operation | Blocking? | Cost |
|---|---|---|
| `send()` | Yes (if full) | O(1) — atomic write |
| `recv()` | Yes (if empty) | O(1) — atomic read |
| `try_send()` | No | O(1) |
| `try_recv()` | No | O(1) |
| `timeout_recv(ms)` | With timeout | O(1) |

### 5.3 Shard

A shard is a static scheduling unit bound to one CPU core:

- Each shard runs on exactly one core (static affinity via `sched_setaffinity`)
- One reactor loop per shard (epoll/kqueue/IOCP)
- Memory budget per shard prevents runaway allocation
- Cross-shard communication via Door (dedicated SPSC channel)

### 5.4 Scaling Efficiency

| Cores | Efficiency | Notes |
|---|---|---|
| 2 | ~95% | Near-linear |
| 4 | ~90% | Good scaling |
| 8 | ~85% | Benchmarked target |
| 16+ | TBD | Expected 70-80% |

---

## 6. Compilation Targets

### 6.1 Native (ELF)

```bash
logicodex input.ldx -o output          # Default: Linux x86_64 ELF
```

| Platform | Format | Status |
|---|---|---|
| Linux x86_64 | ELF | ✅ |
| Linux aarch64 | ELF | ✅ |
| macOS x86_64 | Mach-O | ✅ |
| macOS aarch64 | Mach-O | ✅ |

### 6.2 WebAssembly

```bash
logicodex --target wasm input.ldx -o output.wasm
wasm-ld --no-entry -o final.wasm output.wasm
```

| Feature | Status |
|---|---|
| `wasm32-unknown-unknown` target | ✅ |
| `+bulk-memory,+mutable-globals,+sign-ext` | ✅ |
| WIT generation via CTL Mapper | ✅ |
| Hardware gate → Host Reactor | ✅ |
| WASI capability verification | 🔬 Research (v1.46) |

**WASM constraint:** Hardware gates (`HW.*`) are **forbidden** in WASM target. They must route through `logicodex:host-reactor`.

### 6.3 Freestanding (Bare Metal)

```bash
logicodex --target freestanding input.ldx -o kernel.o
```

| Architecture | LLVM Triple | Code Model |
|---|---|---|
| x86_64 | `x86_64-unknown-none` | Kernel |
| aarch64 | `aarch64-unknown-none` | Small |
| riscv64 | `riscv64gc-unknown-none-elf` | Medium |

Freestanding components:
- `_start` entry point (stack init, BSS zero, data copy)
- `#[panic_handler]` (SSE clear, UART output, halt)
- Bump allocator (`#[global_allocator]`, AtomicUsize CAS)
- UART driver (x86_64 port I/O) + VGA text mode
- IDT (256 entries) + PIC remap
- Multiboot header (GRUB-compatible)

---

## 7. Roadmap

### 7.1 Completed (v1.21 – v1.45)

| Version | Feature | Evidence |
|---|---|---|
| v1.21 | Core compiler (lexer, parser, AST, semantic, LLVM) | 9/9 checks |
| v1.30 | Actor-model concurrency, zero-copy channels, 4-Ketuk IO | 24/24 checks |
| v1.31 | Streaming compiler (2-Pass Engine, SemanticSummary) | 6/6 checks |
| v1.32 | Capability Fabric (Gate/Door, topology verify, `.cap`) | 10/10 checks |
| v1.33 | Network Reactor (compile-time) | 13/13 checks |
| v1.34 | Sharded Reactor (compile-time) | 11/11 checks |
| v1.35 | CapabilityGraph IR (unified) | 16/16 checks |
| v1.36 | CTL Mapper (WIT generation) | 12/12 checks |
| v1.37 | Network Runtime (LIVE epoll, syscalls) | 16/16 checks |
| v1.38 | 8 deferred items resolved | 12/12 checks |
| v1.39 | Sharded Runtime (LIVE threads, CPU affinity) | 10/10 checks |
| v1.40 | WASM Backend (wasm32-unknown-unknown) | 13/13 checks |
| v1.41 | Host Reactor (Guest↔Host HW mediation) | 20/20 checks |
| v1.42 | Raylib FFI (54 functions resolved) | 9/9 checks |
| v1.43 | Raylib Audio (22 functions + StrictAudioContext) | 80/80 checks |
| v1.44 | Freestanding Compiler (15 gaps, 3 archs) | 15/15 checks |
| v1.44.1 | Foundation polish (validator tiering, maintenance) | 10/10 checks |
| v1.45 | Benchmark Framework (4 layers, BASELINE.json) | 6/6 checks |

**Total: 148/148 checks, 14 releases, 0 regression.**

### 7.2 Research (Under Active Exploration)

| Feature | Status | Risk |
|---|---|---|
| v1.46 — Streaming WASM verification | 🔬 Research | WASM threads unstable |
| v2.00 — Pointer Provenance Engine (5-level) | 🔬 Research | 12-18 months R&D |
| Benchmark Layer 4 (Security stress) | 🔬 Research | Validation incomplete |

### 7.3 Long-Term (Requires RFC Under Architecture Freeze)

| Feature | Depends On |
|---|---|
| `ldx-fmt` formatter | Parser snapshot |
| LSP Server | `ldx-fmt` + HIR stable |
| Global Token Registry | Network runtime |
| Logicodex Migrator | Pointer Provenance L5 |
| Runtime Self-Attestation | Freestanding runtime |
| Browser Playground | WASM streaming |
| Full Bootloader | 3-arch freestanding |
| AI Repair Loop | LSP + Migrator |

### 7.4 Architecture Freeze (v1.45+)

As of v1.45, Logicodex is under **Architecture Freeze**. No new features without an RFC passing 4 alignment checks:

- [ ] **Static Topology** — preserves compile-time structure verification
- [ ] **Explicit Ownership** — preserves zero-cost memory safety
- [ ] **Shard Isolation** — preserves deterministic concurrency
- [ ] **Deterministic Behavior** — preserves reproducible execution

RFC template: `docs/RFC_TEMPLATE.md`

---

## 8. Governance

### 8.1 Open Source

Dual-licensed: **MIT** and **Apache 2.0**. All compiler source, specifications, examples, and documentation are open.

### 8.2 Trademark

The names **Logicodex**, **Logicodex Language**, and the ASCII logo are trademark-protected. A fork may use the code under the license but may not claim to be the official Logicodex compiler.

### 8.3 Contribution Model

| Area | Policy |
|---|---|
| Compiler source | Open (MIT/Apache 2.0) |
| Syntax specification | Open (MIT/Apache 2.0) |
| Examples and docs | Open with attribution |
| Official name/logo | Trademark-protected |

### 8.4 Decision Authority

- **Architect** (Mohamad Supardi Abdul): Strategic direction, architecture freeze/unfreeze, RFC approval
- **Contributors**: Bug fixes, tests, documentation, features via RFC
- **AI Assistant**: Architecture exploration, friction discovery, documentation drafting

---

## 9. References

1. **Logicodex v1.21 Baseline** — [`docs/archive/WHITE_PAPER_v121.md`](docs/archive/WHITE_PAPER_v121.md) — Original formal specification
2. **Experimental Compiler Philosophy** — [`docs/white-paper/`](docs/white-paper/) — Detailed design justifications
3. **Functions And Guide** — [`docs/guide/`](docs/guide/) — Complete function reference
4. **LLVM** — https://llvm.org/
5. **WebAssembly** — https://webassembly.org/
6. **Rust FFI** — https://doc.rust-lang.org/nomicon/ffi.html
7. **Intel Intrinsics** — https://www.intel.com/content/www/us/en/docs/intrinsics-guide/
8. **Capability Security** — Dennis & Van Horn (1966), Miller et al. (2003)
9. **Actor Model** — Hewitt, Bishop & Steiger (1973)

---

*Logicodex Specification — v0.46.0-alpha · Last updated 2026-06-14*  
*For questions: mymsastudio@gmail.com*
