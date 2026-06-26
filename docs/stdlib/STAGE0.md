# Logicodex Standard Library — Stage 0 (shipped)

Canonical English reference for what the standard library actually ships at
Stage 0. Design rationale lives in `docs/stdlib/DESIGN.md`; this document is the
"what shipped" record. Branch: `feature/stdlib-core`.

Stage 0 delivers the first two **pure-Logicodex** `core.*` modules, written in
`.ldx` and compiled through the same pipeline as user code. This is the first
full dogfood of the module system and the language together.

**Contract Validation:** Every official module must be accompanied by a `.std.toml`
contract sidecar. These contracts are strictly validated by a Python harness
(`tools/verify_stdlib_contracts.py`) during dev/CI. The Python harness is a dev/CI harness only.
It generates temporary Logicodex programs, compiles and runs them with the Logicodex compiler,
and compares bounded PAPAR stdout. The behavioral oracle is the compiled Logicodex program's stdout,
not Python calculation. Normal compilation does not validate these contracts.

---

## What shipped

### `core.math` (`lib/core/math.ldx`)

Pure integer/logic helpers, all `-> I64`, no extern C, bare-compatible.

```
abs_i64(n)              |n|
min_i64(a, b)
max_i64(a, b)
clamp_i64(x, lo, hi)
sign_i64(n)             -1 / 0 / 1
pow_i64(base, exp)      integer power, exp >= 0 (while-loop accumulator)
is_even(n)             (n & 1) == 0  -> 1 / 0
is_odd(n)              (n & 1) != 0  -> 1 / 0
```

There is no `%` (modulo) operator in the language, so `is_even` / `is_odd` use
the bitwise `& 1` form. The boolean-style predicates return `I64` 1/0 so they
`PAPAR` directly.

### `core.assert` (`lib/core/assert.ldx`)

Pure assertion predicates, `-> I64` (1 = true, 0 = false), no abort.

```
eq_i64(a, b)            1 when a == b
is_true(x)             1 when x is non-zero
```

A hard aborting assert needs `std` (profile-gated exit/panic) and is deferred to
`std.assert`. Stage 0 ships predicates only.

### Enabling changes

- **Dotted module paths** (Module System Stage 0.5): the parser accepts
  multi-segment module names at the import site (`import core.math;`) and the
  call site (`core.math.abs_i64(...)`). Loader and mangling were already
  dotted-aware.
- **Std-root resolution** (Option C): `core.*` / `std.*` resolve against the std
  root — `$LOGICODEX_STD`, then `<compiler-dir>/lib`, then `./lib`. Every other
  module stays filesystem-relative.
- **BitXor `^` operator** (related gap-fix): completes the bitwise set
  (`& | << >>` plus now `^`). C-like precedence `& > ^ > |`. Operator only — no
  `core.bits`, no `^=`, no bool xor. This unblocks future `core.bits` / `core.rand`.

---

## Acceptance criteria (all green)

`core.math` (tests/stdlib_core_math.rs):
```
import core.math; resolves via LOGICODEX_STD
abs_i64(-5)=5  min_i64(2,7)=2  max_i64(2,7)=7  clamp_i64(12,0,10)=10
sign_i64(-3)=-1  is_even(4)=1  is_odd(7)=1  pow_i64(2,10)=1024

Current expanded `core.math` also includes factorial/gcd/lcm helpers and small
arithmetic/predicate helpers. Contract sidecar coverage is authoritative:
`lib/core/math.std.toml` currently lists 16 exports and 32 oracle cases.
abs_i64(5)=5  abs_i64(0)=0  clamp in-range passthrough
single-file legacy examples still pass; no extern C; no logicodex.toml needed
```

`core.assert` (tests/stdlib_core_assert.rs):
```
eq_i64(5,5)=1  eq_i64(5,6)=0
is_true(1)=1  is_true(0)=0  is_true(-3)=1
cross-module dogfood: eq_i64(abs_i64(-5), 5)=1
```

`BitXor` (tests/bitxor.rs):
```
5^3=6  8^1=9  7^7=0
1|2^3=1   (proves ^ tighter than |)
6^2&3=4   (proves & tighter than ^)
xorshift(1) over <<13, >>7, <<17 = 1082269761
```

Module System Stage 0 (9 tests) and the lib unit suite (94 tests) stay green
throughout — zero regressions.

---

## Not built in Stage 0 (deferred, not rejected)

```
std.*  (io, fs, time, mem, str, mathf/libm, c)   needs extern-in-modules + profile
core.math expansion already landed:
- factorial_i64(n)
- gcd_i64(a, b)
- lcm_i64(a, b)
- square_i64(n)
- cube_i64(n)
- is_positive(n)
- is_negative(n)
- between_i64(n, low, high)
core.bits, core.rand                              now unblocked by BitXor
Option / Result public types
extern-in-modules (Module System Stage 1)
cross-module struct / enum / type / const
lod-based stdlib resolution
Malay user-facing aliases for stdlib names (English canonical stays internal)
```

---

## Notes for contributors

- **Reserved keywords collide with identifiers.** `result` is the `Result` type
  keyword and `c` is the `CInterop` keyword — neither can be a variable/param
  name in `.ldx`. `core.math.pow_i64` uses `acc`, `core.assert.is_true` uses `x`.
  `true`, `ok`, `err` (and capitalized forms) are also reserved.
- **e2e test harness must use unique temp dirs.** All tests in one binary share
  `std::process::id()`, so the temp-dir helper uses a per-process `AtomicU64`
  counter; otherwise concurrent tests race and `Drop` deletes each other's dirs.
- **Stdlib is dogfood.** `core.*` is the first real exercise of both the module
  system and the language end-to-end; a stdlib bug is usually a compiler/language
  signal worth investigating, not just a library fix.
