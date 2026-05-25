# Logicodex

**Deterministic Systems Programming Language**  
*Alias-to-canonical · Actor-model concurrency · Capability security · LLVM-backed*  
**v1.45.0-alpha**

```text
 _                 _               _
| |    ___   __ _ (_)  ___  ___   __| |  ___ __  __
| |   / _ \ / _` || | / __|/ _ \ / _` | / _ \\ \/ /
| |__| (_) | (_| || || (__| (_) | (_| ||  __/ >  <
|_____\___/ \__, ||_| \___|\___/ \__,_| \___|/_/\_\
            |___/
```

---

## What is Logicodex?

Logicodex is a systems programming language that eliminates race conditions, memory leaks, and undefined behavior at **compile time** — without runtime overhead.

```logicodex
program hello {
    fn main() -> I32 {
        print "Hello, Logicodex!";
        return 0;
    }
}
```

Write the same code in Malay, English, or expert shorthand — all compile to identical native binaries:

```logicodex
PROGRAM hello
FUNGSI utama() -> I32
MULA
    PAPAR "Halo, Logicodex!"
    PULANG 0
TAMAT
TAMAT PROGRAM
```

| Feature | Status |
|---|---|
| Compiler (lexer → parser → AST → semantic → LLVM) | ✅ Stable |
| Actor-model concurrency (zero-copy channels) | ✅ Stable |
| Capability security (compile-time gates, zero runtime cost) | ✅ Stable |
| Network reactor (epoll, direct syscalls) | ✅ Stable |
| Sharded runtime (per-CPU-core, real OS threads) | ✅ Stable |
| WASM backend (wasm32-unknown-unknown) | ✅ Stable |
| Raylib FFI (54 functions) + Audio (22 functions) | ✅ Stable |
| Freestanding compiler (x86_64 / aarch64 / riscv64) | ✅ Stable |
| Benchmark framework (4 layers) | ✅ Stable |

**148/148 checks passing — zero regression across 14 releases.**

---

## Quick Start

```bash
# Install dependencies (Rust 1.75+, LLVM 15)
# Ubuntu/Debian:
sudo apt-get install llvm-15-dev libclang-15-dev

# Clone and build
git clone https://github.com/msa-studio/logicodex.git
cd logicodex
RUSTFLAGS="-L/usr/lib/llvm-15/lib" cargo build --release

# Run your first program
./target/release/logicodex examples/01_hello.ldx -o hello
./hello
# → Hello, Logicodex!

# Run validators
cargo test --locked                               # Unit tests (must pass)
python3 scripts/validators/tier_a_core/*.py       # Core validators
python3 scripts/validators/tier_b_feature/*.py    # Feature validators
python3 scripts/validators/tier_c_stress/*.py     # Stress validators (CI only)

---

## Documentation Map

Logicodex documentation is organized into **4 core documents**, each with a specific scope. Follow the links below for detailed information.

| Document | Scope | For Whom |
|---|---|---|
| **[`SPECIFICATION.md`](SPECIFICATION.md)** | The contract: language spec, architecture, roadmap, governance | Everyone who wants to understand what Logicodex is and where it's going |
| **[`CHANGELOG.md`](CHANGELOG.md)** | The history: version changes, decision log, evolution timeline | Contributors, historians, anyone tracing decisions |
| **[`docs/HANDBOOK.md`](docs/HANDBOOK.md)** | The guide: tutorials, API reference, examples, troubleshooting | Users writing Logicodex code |
| **`docs/white-paper/`** | Experimental Compiler Philosophy (wiki, ~4,500 LOC) | Engineers, researchers, those who want to understand *why* each decision was made |
| **`docs/guide/`** | Functions And Guide (wiki, ~6,000 LOC) | Users who want comprehensive function reference |

> **Rule of thumb:**
> - **"What is Logicodex?"** → `SPECIFICATION.md`
> - **"What happened when?"** → `CHANGELOG.md`
> - **"How do I use it?"** → `docs/HANDBOOK.md`
> - **"Why was it built this way?"** → `docs/white-paper/` (wiki)
> - **"What functions are available?"** → `docs/guide/` (wiki)

---

## Project Stats

| Metric | Value |
|---|---|
| **Total LOC** | ~43,600 |
| **Rust Source** | ~19,600 (49 files) |
| **Logicodex Library** | ~2,000 (13 `.ldx` files) |
| **Tests** | ~9,230 (34 files, 400+ assertions) |
| **Validators** | ~6,675 (28 files, 148 checks) |
| **Benchmarks** | ~2,200 (20 files, 4 layers) |
| **Wikis** | ~10,500 (40 files) |
| **Releases** | v1.21 → v1.45 (14 alpha releases) |
| **Targets** | Native (ELF), WASM, Freestanding |
| **Architectures** | x86_64, aarch64, riscv64 |

---

## License

Dual-licensed under MIT and Apache 2.0. See [`SPECIFICATION.md`](SPECIFICATION.md) § Governance for full details.

---

*Logicodex — v1.45.0-alpha · Architect: Mohamad Supardi Abdul · mymsastudio@gmail.com*
