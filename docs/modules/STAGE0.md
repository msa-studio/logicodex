# Module System — Stage 0

Stage 0 is the first working slice of the Logicodex module system. It proves the
one chain that unblocks a standard library and multi-file projects:

```
import math;
math.add(2, 3);
```

A function defined in one file is called, by qualified name, from another, and
the whole program compiles, links, and runs as a single native binary.

This document is the canonical English description of what Stage 0 *is* and,
just as importantly, what it is *not*. Malay surface aliases are noted where they
exist, but English is the canonical form for all syntax, internal identifiers,
and documentation.

## Syntax

### Importing a module

```
import math;
```

`import <name>;` makes the functions of another module available under the
qualified prefix `<name>.`. A dotted name selects a nested file:

```
import models.user;     // resolves to ./models/user.ldx
```

Resolution is **filesystem-relative to the importing file**. A single component
`math` resolves to `./math.ldx`; each dot is a directory separator, and the final
component gains the `.ldx` extension. There is no search path and no package
index in Stage 0 — a module is just a file at a known relative location.

### Declaring an exported function

```
public function add(a: I64, b: I64) -> I64 begin
    return a + b;
end
```

A function is **private by default**. The `public` keyword exports it across the
module boundary so it can be called with a qualified call from another module. A
function without `public` is module-local: callable from within its own module,
invisible from outside.

### Calling across modules

```
import math;
math.add(2, 3);          // qualified cross-module call
```

A qualified call `module.function(args)` targets an exported function in an
imported module. The target must be `public`; a qualified call to a private
function is rejected (see *Visibility*).

### Calling within a module

Inside a module, functions call each other by bare (unqualified) name, and a
public function may call a private helper in the same module:

```
function normalize(n: I64) -> I64 begin
    return n + 100;
end

public function calc(x: I64) -> I64 begin
    return normalize(x);    // same-module call to a private helper: allowed
end
```

This is the everyday standard-library pattern: a small public surface backed by
private helpers. Privacy restricts access *from outside* the module, never
*within* it.

## Visibility

| Call site | Target | Allowed? |
|-----------|--------|----------|
| same module, unqualified | private function | yes |
| same module, unqualified | public function | yes |
| other module, qualified | public function | yes |
| other module, qualified | private function | **no — denied** |

A qualified call to a non-public function fails with a clear bilingual
diagnostic, for example:

```
Error: Function `math.add` is private (not declared `public`); it cannot be
called from outside module `math`
```

## What Stage 0 builds

- Single-file programs, unchanged. A program with no `import` behaves exactly as
  before; the module machinery is inert for it.
- `import` of a module file, resolved filesystem-relative (dot = directory).
- `public` / private functions, with visibility enforced on qualified calls.
- Qualified cross-module function calls (`math.add(...)`).
- Same-module unqualified calls, including a public function calling a private
  helper.
- A clear error for a missing module, naming the exact path searched.
- Rejection of import cycles with a clear error (no hang).
- Independent resolution of same-named functions in different modules — `math.add`
  and `stats.add` never collide.

### How non-collision works: name mangling

Every Logicodex function in a non-root module is lowered under a mangled internal
name in a reserved namespace:

```
math.add            ->  __ldx_mod_math__add
models.user.new     ->  __ldx_mod_models_user__new   (dots become underscores)
```

The root module keeps raw names, so existing single-file programs are byte-for-
byte unchanged. Because each module's symbols live under its own mangled prefix
in one shared symbol table, two modules can define `add` without clashing, and a
qualified call resolves straight to the one intended target.

User source may **not** define a symbol beginning with the reserved prefix
`__ldx_`; doing so is rejected. This makes a collision between a user name and a
mangled name impossible by construction.

`extern "C"` symbols are **never** mangled — a foreign symbol must keep its exact
ABI name (`labs`, `sqrt`, `sqlite3_open`) or linking breaks. The mangler draws
the same Logicodex-vs-foreign line the FFI capability gate does.

## What Stage 0 does NOT build

Stage 0 is deliberately **function-only across module boundaries**. The following
are out of scope and fail honestly rather than silently:

- **Cross-module `struct` / `enum` / type access.** A module may only export
  functions. A `struct` or `enum` in a module is rejected — a `public struct` at
  the parser (only `public function` is accepted), and a non-public one at
  lowering, with a diagnostic naming the Stage 0 limitation. Cross-module type
  namespaces, field access, and pattern matching are deferred to a later stage.
- **Selective import** (`import math.{add, sub}`). An import brings the whole
  module into qualified scope; there is no per-symbol import.
- **Aliases** (`import math as m`). No renaming on import.
- **Wildcard / unqualified import** (`import math.*`). Cross-module functions are
  always called qualified.
- **Re-export.** A module cannot re-export another module's functions.
- **Per-module manifests.** A module is a plain `.ldx` file; capability and link
  configuration live in the single root `logicodex.toml`.

Because modules are function-only, an `extern "C"` block inside an imported
module is not part of Stage 0; foreign linkage is configured at the root.

## Internals (for maintainers)

- The **module loader** (`src/module_loader.rs`) turns one root file into a
  topologically ordered, cycle-free list of parsed modules (dependencies first,
  root last). Parsing is injected as a closure so the loader stays a pure
  graph/ordering unit.
- **Lowering** runs every module on one shared `LoweringContext` (one symbol
  table, one type registry). A non-root module is lowered with
  `lower_module_program` (mangled names, no `main`-wrap); the root is lowered
  with `lower_program` (with `main`-wrap). All items merge into one `HirModule`
  fed to the unchanged semantic gate and codegen.
- A qualified call is its own HIR node carrying the target module explicitly, so
  resolution mangles straight to the target name rather than re-deriving it from
  a bare name. An unresolved qualified call is a clear error, never a silent
  fall-through.

## Malay aliases

Logicodex keeps Malay surface aliases for its keywords; the canonical English
forms used throughout this document (`import`, `public`, `function`, `begin`,
`end`) are the forms maintainers should write. Malay-aliased equivalents remain
accepted at the lexer for user-facing source, but English is canonical for all
internal code, HIR fields, tests, and documentation.

## Acceptance criteria

Stage 0 is gated by behaviour-level tests in
`tests/module_system_stage0.rs`, which drive the real binary over multi-file
fixtures: single-file backward compatibility, a working cross-module qualified
call, private-call denial, missing-module path reporting, cycle detection,
same-name non-collision, same-module private-helper calls, struct-in-module
rejection, and reserved-namespace rejection.
