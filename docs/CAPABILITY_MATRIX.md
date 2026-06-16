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
| `channel_send`     | ✅ | ✅ | ✅ | ✅ | ⛔ | 🟡 | Runtime-pending (Phase B) |
| `channel_recv`     | ✅ | ✅ | ✅ | ✅ | ⛔ | 🟡 | Runtime-pending (Phase B) |
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
- **channel_*** — the front of the pipeline is ready (parsed, lowered, code-
  generated to the fixed channel ABI), but the runtime backend is not built yet.
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
