# Module System -- Stage 0 Design

Status: **approved, not yet implemented**. This document is the design of record
for the Logicodex module system. It is written before any code, so the contract
is fixed and reviewable first.

This document is English-canonical. Logicodex source examples use the English
keyword forms (`import`, `public`, `function`, `begin`, `end`, `return`,
`print`). The Malay forms (`GUNA`, `AWAM`, `FUNGSI`, ...) remain available as
user-facing aliases through the lexicon, but the canonical surface -- and all
compiler internals, tests, and architecture docs -- is English.

## Why a module system, and why now

After the actor runtime, channels, the FfiGatekeeper, and lod Stage 0, the
largest remaining bottleneck is not the compiler. It is: **how does a project
grow beyond one file?** Without modules, a standard library cannot exist, lod
cannot grow, a future HIR specification has nothing to describe, and every new
feature is worth less than it could be. The module system is the multiplier that
unblocks the standard library, lod, the HIR spec, and future frontends at once.

## Scope: Stage 0 only

Stage 0 is deliberately the smallest module system that is genuinely useful. It
is not a final design; it is the foundation later stages build on.

### In scope

- `import module;` -- bring another file's public items into scope under a name.
- `public function` / `public struct` / `public enum` -- exported items.
- Private by default -- an item with no `public` is module-local.
- Qualified access only -- `module.symbol`, never bare `symbol` across modules.
- Filesystem-relative resolution -- a dot is a directory separator.
- Dependency graph with cycle detection -- cycles are a clear error, not a hang.
- A module-aware symbol table -- every callable has a unique, module-qualified id.
- Reserved-prefix name mangling -- Logicodex symbols become `__ldx_mod_*`.
- Extern C symbols are never mangled -- they keep their raw ABI names.
- Root-manifest capabilities only -- one `logicodex.toml` for the whole program.
- Single-file backward compatibility -- a program with no `import` is unchanged.

### Out of scope (later stages)

Wildcard import (`import math.*`), import aliases (`import math as m`), selective
import (`import math.{add}`), re-export, cyclic modules, package imports (lod
paths), per-module manifests, and an explicit module header. These are
intentionally excluded so the Stage 0 contract stays small and provably correct.

## Canonical example

`math.ldx`:

```
public function add(a: I64, b: I64) -> I64 begin
    return a + b;
end

function secret() -> I64 begin
    return 99;
end
```

`main.ldx`:

```
import math;

print math.add(2, 3);
```

`math.add` is exported and callable. `math.secret` is private -- a call to it
from `main.ldx` is rejected at check time. Access is always qualified: you write
`math.add`, never a bare `add` that silently resolves across files.

## Design principles

The module system is shaped by one principle, learned the hard way from a class
of bugs where a wildcard arm or a registry lookup silently swallowed the truth,
or where one id-space aliased another:

> **A name's meaning travels with its node, and module-qualified names are unique
> by construction.** Resolution never guesses from structure or from a shared
> id-space.

Three concrete consequences:

1. **Qualified calls are their own node.** `math.add(2, 3)` parses to a dedicated
   `QualifiedCall { module, function, args }`, not a field access followed by a
   call. The call carries which module it targets; nothing has to be inferred.

2. **Reserved-prefix mangling makes collisions impossible.** A Logicodex function
   `add` in module `math` is emitted as `__ldx_mod_math__add`. User symbols are
   forbidden from starting with `__ldx_`, so a user can never collide with a
   mangled name -- the namespace is reserved, not merely hoped to be free.

3. **Extern C is never mangled.** A foreign symbol like `labs` or `sqlite3_open`
   keeps its exact ABI name. Mangling it would silently break linking. The
   mangler therefore distinguishes Logicodex functions (mangled) from extern C
   (raw) -- the same Logicodex-vs-foreign distinction the FfiGatekeeper draws.

## Resolution

`import` resolves against the filesystem, relative to the importing file. A dot
is a directory separator:

```
import math;          ->  ./math.ldx
import models.user;   ->  ./models/user.ldx
```

Resolution is deterministic: there is no search path, no implicit root, no
ambiguity. A module that cannot be found is an error that names the exact path
that was tried.

## Compilation model

This is the largest structural change. Today `compile()` takes one file. Modules
mean many files. To keep that change isolated, a separate, independently tested
**module loader** (mirroring how `src/lod.rs` isolates manifest handling) does
the multi-file work:

1. Start from the root file; parse it.
2. Find its `import` statements.
3. Resolve and parse each imported file, recursively.
4. Build a dependency graph, tracking visited modules.
5. Detect cycles -- a cycle is a clear error, never an infinite loop.
6. Produce a topological order.
7. Lower each module to HIR under its module-qualified namespace.
8. Feed the ordered HIR to the existing pipeline.

A single-file program with no imports skips the loader entirely and compiles
exactly as it does today.

## Symbol table and the HIR

The symbol table becomes module-aware. A callable is defined under its
module-qualified, mangled name, so its id is unique across the whole program by
construction. `math.add` and `stats.add` are different names
(`__ldx_mod_math__add` vs `__ldx_mod_stats__add`) and therefore different ids;
they cannot collide. A `QualifiedCall { module: "math", function: "add" }`
resolves to the mangled name directly -- the call already knows its module, so
there is no cross-file name guessing.

## Visibility

Every item is private to its module unless marked `public`. A `QualifiedCall`
into another module is allowed only if the target item is public; otherwise the
semantic gate emits a bilingual deny diagnostic. This is the same default-deny,
explicit-allow shape as the FfiGatekeeper, applied to the module boundary.

## Capability

Capabilities remain governed by the single root `logicodex.toml`. A module that
declares an `extern "C"` function still has that symbol gated by the root
manifest -- there are no per-module manifests. One manifest describes the whole
program's capability surface. This keeps the future capability map trivial: every
capability flows through one gate and one manifest, so a tool that lists what a
program can touch only has to read what the gate already collected.

## Acceptance criteria

The implementation is complete when all ten hold:

1. Single-file programs with no `import` still compile and run unchanged.
2. `import math; math.add(2, 3)` compiles, links, runs, and prints the result.
3. `math.secret()` is rejected because `secret` is not public.
4. A missing module reports the exact path that was searched.
5. Cyclic imports fail with a clear error, not a hang.
6. `math.add` and `stats.add` do not collide.
7. An extern C symbol from an imported module is gated by the root manifest.
8. Extern C symbols are not mangled.
9. The documentation uses English canonical examples.
10. Malay aliases are documented as future/secondary, not an MVP blocker.

## What this unblocks

With Stage 0 in place, the standard library can be written as ordinary modules,
lod can grow toward resolving real dependency paths, the HIR specification has a
module structure to describe, and future language frontends have a target that is
already organised into modules. That is the leverage: one foundation, many things
it makes possible.
