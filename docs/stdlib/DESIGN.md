# Logicodex Standard Library — Stage 0 Design

**Status:** Stage 0 in progress on `feature/stdlib-core`.
**Scope of this document:** lock the namespace split, resolution rule, Stage 0
modules, the not-built list, and the testing rule. Nothing more. Implementation
follows immediately in `lib/core/*.ldx`.

The standard library is written **in Logicodex** (`.ldx`). It is therefore the
first real dogfood of both the module system and the language itself: every
stdlib module compiles, links, and runs through the same pipeline a user's code
does, and every module ships acceptance tests that compile + run + assert.

**Contract Validation:** Every official module must be accompanied by a `.std.toml`
contract sidecar. These contracts are strictly validated by a Python harness
(`tools/verify_stdlib_contracts.py`) during dev/CI. The Python harness is a dev/CI harness only.
It generates temporary Logicodex programs, compiles and runs them with the Logicodex compiler,
and compares bounded PAPAR stdout. The behavioral oracle is the compiled Logicodex program's stdout,
not Python calculation. Normal compilation does not validate these contracts.

---

## 1. `core` vs `std` split (locked)

The library is split into two namespaces by a single test: **does it need an
operating system?**

```
core.*   pure Logicodex      no extern C, no OS, no malloc, no profile
         bare-compatible     runs in bare / freestanding / native-zero
         foundation stdlib

std.*    OS / libc dependent  may use extern C, needs runtime profile
         profile-gated        only under --profile std / safe / actor / service
         requires FfiGatekeeper + root manifest
```

This split is not a copy of Rust's `core`/`std`; it falls out naturally from the
existing **Profile** system (`bare | std | safe | actor | service`). Putting
everything under `std` would destroy the freestanding story — the whole point is
that `core.*` can be imported from bare-metal code. So the split is deliberate
and load-bearing, not cosmetic.

Consequence for sequencing:
- `core.*` can be built **now**, within Module System Stage 0 constraints
  (function-only across boundaries, pure Logicodex).
- `std.*` is **deferred**: it needs extern-in-modules (gated by a root manifest),
  which is a Module System Stage 1 capability, not yet built.

---

## 2. Resolution rule (Option C, locked)

The reserved namespaces `core` and `std` resolve against the **std root**, tried
in this order:

```
1. $LOGICODEX_STD          explicit override
2. <compiler-dir>/lib      installed distribution, next to the binary
3. ./lib                   dev/test fallback (repo-root lib/)
```

Every other module stays **filesystem-relative** to the importing file, exactly
as in Module System Stage 0.

```
import core.math;   ->  <std-root>/core/math.ldx
import std.io;      ->  <std-root>/std/io.ldx      (deferred; reserved)
import mymod;       ->  ./mymod.ldx                (unchanged)
import a.b;         ->  ./a/b.ldx                  (unchanged)
```

`lod` is **not** used for stdlib resolution at this stage — that is a later
package-manager concern, too early here.

On-disk layout:

```
lib/
  core/
    math.ldx
    assert.ldx
  std/                 (reserved; empty in Stage 0)
```

### Prerequisite — dotted module paths (Module System Stage 0.5, done)

Stage 0 originally parsed only single-segment module names at the import site
and the call site. The `core.*` / `std.*` namespace requires multi-segment
dotted names (`import core.math;`, `core.math.abs_i64(...)`). This was closed as
**Module System Stage 0.5**: the parser now accepts dotted names in
`use_declaration` and in qualified calls; the loader and name mangling were
already dotted-aware.

---

## 3. Stage 0 modules

English canonical names only (expert, standardized, international). Malay aliases
may come later as documentation/aliases, never as the internal canonical name.

### `core.math`

Pure integer/logic helpers built from existing primitives. There is **no `%`
(modulo)** operator in the language, so `is_even` / `is_odd` use the bitwise
`& 1` form. There is also no `^` (BitXor) yet — irrelevant for these functions.

First wave (build + prove these before the rest):
```
abs_i64(n)             |n|
min_i64(a, b)
max_i64(a, b)
clamp_i64(x, lo, hi)
sign_i64(n)            -1 / 0 / 1
pow_i64(base, exp)     integer power (exp >= 0)
is_even(n)            (n & 1) == 0
is_odd(n)             (n & 1) != 0
```

Second wave (after the first wave passes — more edge cases):
```
gcd_i64(a, b)
lcm_i64(a, b)
factorial_i64(n)
```

### `core.assert`

Pure assertion predicates. A hard abort/exit needs `std` (profile-gated), so
Stage 0 ships **predicates** returning a boolean-style result, not an aborting
assert. The aborting variant belongs in `std.assert` later.
```
eq_i64(a, b)           true when a == b
is_true(c)             true when c is non-zero
```

`assert` lives in `core`, not `std`, because it needs no OS, libc, malloc, or
profile — by the section-1 test it is `core`.

---

## 4. Not built in Stage 0 (deferred, not rejected)

```
std.io   std.fs   std.time   std.mem   std.str
std.mathf / libm (sqrt, sin, pow via C ABI)
std.c    (C-string / pointer ABI bridge)
Option / Result public types
StatusCode enum / module constants
extern-in-modules        (Module System Stage 1)
cross-module struct / enum / type / const
lod-based stdlib resolution
core.bits                (blocked until BitXor `^` lands + bitwise audit)
core.rand                (pure Logicodex xorshift — belongs in core, later)
```

These are sequencing decisions, not value judgments. `core.math` + `core.assert`
is the cleanest test of "stdlib import + module system + pure-Logicodex
dogfooding" with zero new security surface.

---

## 5. Testing rule

Every stdlib module ships behaviour-level acceptance tests (compile + run +
assert on output/exit), in the style of `tests/module_system_stage0.rs`. A
module is not "done" until its acceptance tests are green and single-file legacy
examples still pass.

### `core.math` Stage 0 acceptance criteria
```
1. import core.math; resolves via the std root (LOGICODEX_STD)
2. core.math.abs_i64(-5)         -> 5
3. core.math.min_i64(2, 7)       -> 2
4. core.math.max_i64(2, 7)       -> 7
5. core.math.clamp_i64(12, 0, 10)-> 10
6. core.math.sign_i64(-3)        -> -1
7. core.math.is_even(4)          -> true     (1)
8. core.math.pow_i64(2, 10)      -> 1024
9. core.math.factorial_i64(5)    -> 120
10. core.math.gcd_i64(54, 24)    -> 6
11. core.math.lcm_i64(6, 8)      -> 24
12. core.math.square_i64(7)      -> 49
13. core.math.cube_i64(-3)       -> -27
14. core.math.between_i64(5,1,10)-> 1
9. single-file legacy examples still pass
10. no extern C used; works without logicodex.toml
```

### `core.assert` Stage 0 acceptance criteria
```
1. core.assert.eq_i64(abs_i64(-5), 5) is true
2. core.assert.is_true(1) is true; is_true(0) is false
3. pure Logicodex; usable from bare/freestanding
```
