# lod Stage 0: Manifest-driven C ABI linking

`lod` is the (future) Logicodex dependency manager. This document describes
**Stage 0** -- the smallest meaningful slice that exists today. Stage 0 is
deliberately tiny: it is NOT a package manager. It does exactly one thing, and
does it honestly.

## What Stage 0 is

Stage 0 reads an optional `logicodex.toml` next to the program being compiled and
turns it into two compiler inputs:

1. the set of FFI symbols the program is **allowed** to call (fed to the
   `CapabilityPolicy` that the FfiGatekeeper enforces), and
2. the external C libraries to **link** (fed to the linker as `-l<lib>`).

That is the whole job. Stage 0 proves the full external-C chain end to end:

```
extern "C" fn      ->  the program declares a foreign function
ffi.allow          ->  the manifest explicitly permits that symbol
FfiGatekeeper pass ->  the capability gate lets the call through
-l<lib> injected   ->  the library is added to the link command
native link        ->  the linker resolves the symbol against real C
program runs       ->  the foreign function actually executes
```

Without a manifest, nothing changes: the FfiGatekeeper's default-deny stays
fully in force, so a program that declares an `extern "C"` function it has not
been granted is rejected at check time. The manifest is the *only* way to widen
what a program may call, and every widening is explicit and auditable.

## What Stage 0 is NOT

Stage 0 deliberately does **not** include any of the following. They belong to
later stages and are intentionally absent so the contract stays small:

- no `lod` CLI (`lod init`, `lod add`, `lod build`, ...)
- no lockfile
- no online registry or package download
- no version resolution
- no C header binding generation (bindgen)
- no transitive dependency graph

If you need any of those today, you do not need them -- you need Stage 0, which is
just "declare what C you call, and what to link it against."

## Manifest format

The manifest is `logicodex.toml`, placed in the same directory as the `.ldx`
source. Every section is optional; an empty or absent manifest contributes
nothing (and leaves default-deny fully in force).

```toml
# Top-level direct allow: symbols not tied to a specific declared C library,
# or symbols that live in libc (which the toolchain links by default).
[ffi]
allow = ["labs"]

# Per-C-library dependency. The table key (here `libm`) is a name you choose.
#   link  = the -l name passed to the linker (libm -> "m")
#   allow = the FFI symbols this library is permitted to provide
[dependencies.c.libm]
link = "m"
allow = ["sqrt", "pow"]
```

### Merge rule

The compiler combines the manifest into the two channels like this:

```
allowed_symbols  +=  [ffi].allow  +  every [dependencies.c.*].allow
user_libs        +=  every [dependencies.c.*].link
```

`allowed_symbols` feeds the `CapabilityPolicy` (the security gate); `user_libs`
feeds the linker. The two are independent on purpose:

- A symbol can be **allowed without being linked** -- useful for libc symbols
  (like `labs`) that the toolchain links by default. The gate must still permit
  them, but no `-l` is needed.
- A library declared with `link` but whose symbols a program never calls simply
  contributes a harmless `-l` the linker may ignore.

This independence is honest about a real asymmetry: *permission* (may I call
this?) and *availability* (is the code present at link time?) are different
questions, and Stage 0 keeps them as separate, visible inputs.

## How it plugs into the compiler

Stage 0 is integrated directly into `compile` and `check` -- there is no separate
`lod` binary. Both paths call `lod::Manifest::discover(source)`:

- `compile` adds the allowed symbols to the `CapabilityPolicy` *and* extends
  `LinkSpec.user_libs` with the libraries to link.
- `check` adds the allowed symbols to the `CapabilityPolicy` only (it does not
  link), so a denied or allowed extern reports identically in both paths.

The manifest is parsed by a small hand-written parser (`src/lod.rs`) that accepts
exactly the subset of TOML the format above uses -- no external TOML crate is
pulled in, consistent with Logicodex's preference for owning its core.

## Provenance: the two link channels

The linker receives libraries from two distinct channels, never blurred:

| Channel        | Source                          | Example      | Who decides |
|----------------|---------------------------------|-------------|-------------|
| `runtime_libs` | the runtime/profile itself      | `pthread`    | Logicodex   |
| `user_libs`    | `[dependencies.c.*].link`       | `m`, `sqlite3` | the user (via manifest) |

`runtime_libs` are platform/core building blocks Logicodex decides on (for
example, the actor profile needs `pthread`). `user_libs` are external C the user
explicitly opted into. Auto-linking a `runtime_lib` never means "Logicodex
depends on a third-party C library"; it means a core primitive is in use. Stage 0
only ever adds to `user_libs`, never to `runtime_libs`.

## Worked example

`prog.ldx`:

```
extern "C" fn labs(x: I64) -> I64;
unsafe {
    BINA r: I64 = labs(7);
    PAPAR r;
}
```

`logicodex.toml` (next to `prog.ldx`):

```toml
[dependencies.c.libc]
link = "c"
allow = ["labs"]
```

Then:

```
logicodex check   prog.ldx     # validation succeeds (labs is allowed)
logicodex compile prog.ldx -o prog
./prog                         # prints 7 -- labs(7) ran from real libc
```

Remove the manifest and re-run `check`: the call to `labs` is denied by the FFI
capability policy, because nothing permits it. That is Stage 0 working as
intended -- the door is shut by default, and the manifest is the only key.

## Status

| Capability                                   | Status   |
|----------------------------------------------|----------|
| Read `[ffi].allow` into the capability gate  | **Real** |
| Read `[dependencies.c.*].allow` into the gate | **Real** |
| Read `[dependencies.c.*].link` into `user_libs` | **Real** |
| Deny an un-allowed extern (no manifest)      | **Real** |
| `check` and `compile` agree on allow/deny    | **Real** |
| Hand-written zero-dependency TOML subset parser | **Real** |
| `lod` CLI / lockfile / registry / versions / bindgen | Not started (later stages) |

"Real" = implemented and proven end-to-end (a program declaring `extern "C" fn
labs`, allowed via the manifest, links against libc and prints the real result).
