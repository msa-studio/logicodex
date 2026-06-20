# Capability Matrix

Honest, per-feature status across the compiler pipeline. This is the single
source of truth for "what actually works end-to-end" versus "what is parsed or
code-generated but has no runtime yet".

Columns: **Parser** (recognized syntax) · **HIR** (lowered) · **Codegen**
(emits LLVM/calls) · **Runtime ABI** (symbol signature fixed) · **Runtime Impl**
(backend exists) · **Tests** (guarded in suite) · **Status**.

Legend: ✅ done · 🟡 partial · ⛔ none.

| Feature            | Parser | HIR | Codegen | Runtime ABI | Runtime Impl | Tests | Status |
|--------------------|:------:|:---:|:-------:|:-----------:|:------------:|:-----:|--------|
| `print`            | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | **Real** |
| `sleep`            | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | **Real** |
| `yield`            | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | **Real** |
| `spawn`            | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | **Real** (`--profile actor`) |
| `join`             | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | **Real** (`--profile actor`) |
| `channel_send`     | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | **Real (same-scope)** (`--profile actor`) |
| `channel_recv`     | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | **Real (same-scope)** (`--profile actor`) |
| `service_health`   | ⛔ | ⛔ | ⛔ | ⛔ | ⛔ | ⛔ | Not started |
| `service_metrics`  | ⛔ | ⛔ | ⛔ | ⛔ | ⛔ | ⛔ | Not started |
| `capability_check` | 🟡 | 🟡 | ⛔ | ⛔ | ⛔ | 🟡 | FFI extern: Real (default-deny); broader caps: types only |

\* `spawn`/`join` are guarded two ways: `actor_spawn_join_runs_in_a_real_thread`
proves the real runtime under `--profile actor` (deterministic 99 then 1), while
`actor_spawn_check_passes_but_compile_is_pending` proves that without
`--profile actor` the honest runtime-pending message still fires.

## Notes per feature

- **print / sleep / yield** — fully real. Backed by direct syscalls in the Linux
  runtime assembly (`write`, `nanosleep`, `sched_yield`); no Rust std linked.
  Proven end-to-end and guarded by `runtime_sleep_and_yield`.

- **spawn / join** — real under `--profile actor`. The actor body is lowered to a
  callable function; SPAWN runs it on a real OS thread via `pthread_create`, and
  JOIN waits via `pthread_join` using a handle codegen stores at spawn (ABI-1).
  Backed by the audited `src/runtime/runtime_actor.c` (linked with `-lpthread`).
  Without `--profile actor`, programs using them still fail honestly (guard).
- **channel_create / channel_send / channel_recv** — real **within a single
  scope** under `--profile actor`. `Channel::baru(N)` allocates an SPSC bounded
  buffer (pthread mutex + condvar in runtime_actor.c) and returns an i64 handle;
  `ch.send`/`ch.recv` are by-handle and block. Scope: SPSC, bounded, blocking
  only. **Cross-actor (Channel B.1b): Real**, via explicit capture — an actor
  declares a channel parameter and `SPAWN actor(ch)` passes the handle; codegen
  builds a ctx of i64 handle(s) + a wrapper and dispatches via
  `logicodex_spawn_ctx`. A channel used inside an actor body WITHOUT being a
  declared parameter is still rejected at check time (the
  compiler rejects this at check time with a clear message rather than
  deadlocking), and no `free`/`close`/`drop`/`timeout`/`select`/MPSC/broadcast.
  Message type is `I64` only for now.
- **channel_try_send / channel_try_recv / timeout_recv** — not built. Still
  Reserved; the blocking send/recv came first (B.1).
  The compiler refuses to build such programs (honest error). Planned over
  `pthread` mutex+condvar in runtime_actor.c (not Rust std).

- **service_health / service_metrics** — not started. A future `service` profile
  may adapt Lxdge's working epoll reactor (`LXDGE_EXTRACTION.md`), local/single-node.

- **capability_check** — two layers exist. (1) **FFI capability gate: Real** —
  extern "C" symbols are default-deny and must be in `ffi.allow` (see the FFI
  Capability Policy section below); enforced at check time. (2) The broader
  capability *types* / vocabulary gate (the `check` path validates malformed
  capability declarations) still has no general runtime enforcement, so the
  `safe` profile remains runtime-pending for non-FFI capabilities.

## FFI Capability Policy (zero-trust, default deny)

The FFI capability gate is **Real**: every `extern "C"` symbol a program calls
must be explicitly permitted, or the call is rejected at check time with a clear
bilingual diagnostic. This is the security door that exists BEFORE `lod` opens
the external C ecosystem.

**Classification (locked):**
- **Builtin / auto-allow** — the compiler-emitted runtime ABI (`logicodex_*`:
  spawn, spawn_ctx, join, channel_*, print_i64, yield, sleep). These are
  Logicodex's own shims over OS primitives, backed by the audited
  `runtime_actor.c`, never third-party C.
- **External C** — everything else (Raylib, sqlite, openssl, curl, zlib, ...).
  Denied by default; each symbol must be opted in via `ffi.allow`.

**Check order:** (1) runtime builtin -> allow; (2) symbol in `ffi.allow` ->
allow; (3) library-level opt-in (`ffi.allow_lib`) reserved for later; (4) else
deny.

**Diagnostic (denied call):**
```
Error: extern call 'sqlite3_open' denied by FFI capability policy.
       Declare it in ffi.allow before use.
```

**Policy source — now vs later.** The policy is currently held **in memory**
(`CapabilityPolicy::with_runtime_builtins()`), seeded with only the runtime
builtins. Symbols are added manually via `allow_symbol` (tests / embedding code).
`lod` will later populate `allowed_symbols` from `logicodex.toml [ffi.allow]`;
the deny-by-default contract lives in the compiler so it holds regardless of how
the policy is sourced.

**Raylib is external C** — it is NOT a builtin and requires explicit allow. A
demo that uses Raylib opts in with a preset:
```toml
[ffi]
allow = ["InitWindow", "CloseWindow", "WindowShouldClose",
         "BeginDrawing", "EndDrawing", "ClearBackground", "DrawText"]
```

**Not yet built:** library-level `ffi.allow_lib` enforcement (symbol-level is
authoritative for now); loading the policy from `logicodex.toml` (that is `lod`).

## How to read "Status"

- **Real** — implemented, proven end-to-end, guarded by a passing test.
- **Runtime-pending** — language surface works; runtime backend deferred; failure
  is loud and honest.
- **Not started / Types only** — exactly what it says; listed so the gap is
  visible rather than implied to work.
