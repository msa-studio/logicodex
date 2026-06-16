# Runtime ABI

Logicodex codegen lowers certain language constructs to calls into a small,
stable C-ABI runtime. This document is the contract for those symbols: their
signatures (as emitted by codegen), and their current implementation status.

All symbols use the C calling convention and operate on `i64` (Logicodex's
single integer width) and `*const i8` (C strings, used for actor/channel names).

## Symbol table

| Symbol                          | Signature                          | Status            | Backend |
|---------------------------------|------------------------------------|-------------------|---------|
| `logicodex_print_i64`           | `(i64) -> void`                    | **Real**          | `write(2)` syscall (runtime asm) |
| `logicodex_sleep`               | `(i64 ms) -> i64`                  | **Real**          | `nanosleep(2)` syscall |
| `logicodex_yield`               | `() -> i64`                        | **Real**          | `sched_yield(2)` syscall |
| `logicodex_spawn`               | `(*const i8 entry) -> i64`         | **Real**          | `pthread_create` (runtime_actor.c) |
| `logicodex_join`                | `(i64 handle) -> i64`              | **Real**          | `pthread_join` (runtime_actor.c) |
| `logicodex_channel_send`        | `(*const i8 name, i64 val) -> i64` | Reserved          | — (Phase B) |
| `logicodex_channel_recv`        | `(*const i8 name) -> i64`          | Reserved          | — (Phase B) |
| `logicodex_channel_try_send`    | `(*const i8 name, i64 val) -> i64` | Reserved          | — (Phase B) |
| `logicodex_channel_try_recv`    | `(*const i8 name) -> i64`          | Reserved          | — (Phase B) |
| `logicodex_timeout_recv`        | `(*const i8 name, i64 ms) -> i64`  | Reserved          | — (Phase B) |

"Real" = implemented and proven end-to-end. "Reserved" = codegen emits calls to
this symbol, the signature is fixed, but no runtime defines it yet. Compiling a
program that reaches a reserved symbol is blocked early with an honest error (see
PROFILES.md), so the reserved symbols never reach the linker today.

## Implemented symbols (detail)

### `logicodex_print_i64(i64) -> void`
Pre-existing. Prints a base-10 integer followed by a newline via a direct
`write(2)` syscall. Defined in `os::runtime_assembly()` (Linux) and, separately,
by the freestanding kernel runtime (which provides its own UART-backed shim).

### `logicodex_sleep(i64 ms) -> i64`
Sleeps for `ms` milliseconds. Builds a `struct timespec { tv_sec; tv_nsec; }` on
the stack (`tv_sec = ms / 1000`, `tv_nsec = (ms % 1000) * 1_000_000`) and invokes
`nanosleep(2)`. Returns 0. Interrupted sleeps are not resumed in this phase.
Lowered from `SLEEP(expr)`.

### `logicodex_yield() -> i64`
Yields the CPU via `sched_yield(2)`. Returns 0. Lowered from `YIELD()`.

### `logicodex_spawn(*const i8 entry) -> i64`

Starts the actor body on a new OS thread via `pthread_create(3)` and returns an
opaque handle (the `pthread_t` reinterpreted as `i64`). `entry` is a pointer to
the actor's lowered function `__actor_<name>` (ABI-1: a function pointer, never a
name — the runtime does no name lookup). Bad input fails honestly with a
provenance-tagged negative code (`LX_ERR_INVALID_ENTRY` for NULL, `LX_ERR_OS` if
`pthread_create` fails), never UB. Backed by the audited C runtime
`src/runtime/runtime_actor.c`, linked with `-lpthread` only under
`--profile actor`. Lowered from `SPAWN <name>()`.

### `logicodex_join(i64 handle) -> i64`

Waits for the actor identified by `handle` via `pthread_join(3)`. Returns 0 on
success, or a provenance-tagged negative code (`LX_ERR_INVALID_HANDLE` for handle
`<= 0`, e.g. a JOIN with no prior SPAWN; `LX_ERR_OS` if `pthread_join` fails),
never UB. Codegen owns the actor-name -> handle slot mapping, so the runtime
never sees a name. Lowered from `JOIN <name>`.

## Reserved symbols (detail)

`channel_*`/`timeout_recv` back the channel constructs (channel send/recv).
Codegen already declares and calls them with the signatures above, so the ABI is
fixed. Their runtime is deferred to a later step, which will implement them in C
over `pthread` (mutex+condvar channels in runtime_actor.c) — deliberately **not**
`std::thread`/`mpsc`, to keep the Rust std runtime out of generated executables.

Until then, the compiler refuses to build programs that use them, so a reserved
symbol never produces an "undefined reference" at link time.

## Platform notes

- The real backends live in `os::runtime_assembly()` for **Linux** (`src/os/linux.rs`),
  which is what CI (ubuntu) and normal WSL builds use.
- The non-Linux fallback (`src/os/mod.rs`) and Windows (`src/os/windows.rs`)
  currently provide only `print`. `sleep`/`yield` on those targets are a
  documented gap.
- The **freestanding** target provides its own `print` shim and does not link the
  Linux runtime assembly; `sleep`/`yield`/actor are not available there yet.
