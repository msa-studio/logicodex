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
| `capability_check` | 🟡 | 🟡 | ⛔ | ⛔ | ⛔ | 🟡 | Types only, no enforcement |

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

- **capability_check** — capability *types* and a vocabulary gate exist (the
  `check` path validates malformed capability declarations), but there is no
  runtime enforcement. The `safe` profile is therefore runtime-pending.

## How to read "Status"

- **Real** — implemented, proven end-to-end, guarded by a passing test.
- **Runtime-pending** — language surface works; runtime backend deferred; failure
  is loud and honest.
- **Not started / Types only** — exactly what it says; listed so the gap is
  visible rather than implied to work.
