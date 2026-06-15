# Runtime Profiles

Logicodex compiles `.ldx` programs to native executables. A **runtime profile**
selects which runtime-ABI surface a program may use at link time. Profiles let
the language stay minimal by default while leaving room for richer runtimes
later, without making any of them mandatory.

Select a profile with `--profile` on `compile`:

```
logicodex compile app.ldx --profile std      # default
logicodex compile app.ldx --profile bare
```

## Status (Phase D)

| Profile   | Runtime surface                          | Status            |
|-----------|------------------------------------------|-------------------|
| `bare`    | `print` only                             | **Real**          |
| `std`     | `print` + `sleep` + `yield`              | **Real** (default)|
| `safe`    | `std` + capability enforcement           | Runtime-pending   |
| `actor`   | `std` + `spawn`/`join`/`channel`         | Runtime-pending (Phase B) |
| `service` | `std` + local reactor/health/metrics     | Runtime-pending   |

"Real" means the runtime is implemented and proven end-to-end (compile → link →
run). "Runtime-pending" means the language surface is parsed and type-checked,
and in some cases code-generated, but has no runtime backend yet — so `compile`
stops with an honest, actionable error rather than emitting a broken executable.

## What each profile means today

### `bare` — Real
No runtime beyond the `print` builtin (`logicodex_print_i64`, a direct `write`
syscall in the linked runtime assembly). This is the most minimal surface and
matches Logicodex's native/freestanding identity. Note: because `sleep`/`yield`
live in the same runtime assembly, they happen to be linkable here too; `bare`
is about *intent* (depend on nothing beyond print).

### `std` — Real (default)
Adds two real runtime builtins, both backed by direct Linux syscalls in
`os::runtime_assembly()` — **no Rust std runtime is linked into the generated
executable**:

- `SLEEP(ms)` → `logicodex_sleep(i64) -> i64`, via `nanosleep(2)`
- `YIELD()`   → `logicodex_yield()  -> i64`, via `sched_yield(2)`

Proven end-to-end: `PAPAR 1; YIELD(); SLEEP(300); PAPAR 2;` compiles, links, and
runs, with a measured ~0.3s pause confirming `nanosleep` fires.

### `safe` — Runtime-pending
Intended to add local capability checks on top of `std`. Capability *types*
exist in the codebase, but there is **no runtime enforcement** yet, so claiming
a working `safe` profile would be dishonest. `compile --profile safe` fails early
with that explanation.

### `actor` — Runtime-pending (Phase B)
Intended to add `spawn`/`join`/`channel` on top of `std`. The compiler already
*code-generates* calls to the actor ABI symbols (`logicodex_spawn`, etc.), but
**no runtime defines them**, so linking such a program would otherwise fail with
a bare "undefined reference". Two honest guards exist:

1. `compile --profile actor` is rejected at the CLI with a pending message.
2. Any program that uses `spawn`/`join`/`channel` (under any profile) is caught
   before codegen by an HIR scan (`first_pending_actor_op`) and stopped with a
   message pointing here.

`check` still passes for such programs — they are syntactically and semantically
valid; only the runtime is missing.

Planned implementation (Phase B): a C-ABI runtime using `pthread_create`/`join`
and a mutex+condvar channel — **not** Rust `std::thread`/`mpsc` — to preserve the
"no Rust std runtime in generated executables" property.

### `service` — Runtime-pending
Intended to add a local single-node reactor with health/metrics endpoints. Not
implemented. A future implementation may adapt the working sharded epoll reactor
from the Lxdge project (see `LXDGE_EXTRACTION.md`). No HA/clustering is planned
for this profile — local single-node only.

## Design rules

- bare/std must never depend on a heavy runtime or Rust std being linked in.
- A profile is only marked "Real" once it is proven end-to-end and guarded by a
  test in the suite.
- Pending profiles fail loudly and honestly; they never silently degrade or emit
  a program that crashes or fails to link with an opaque error.
