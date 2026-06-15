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
| `spawn`            | ✅ | ✅ | ✅ | ✅ | ⛔ | ✅* | Runtime-pending (Phase B) |
| `join`             | ✅ | ✅ | ✅ | ✅ | ⛔ | ✅* | Runtime-pending (Phase B) |
| `channel_send`     | ✅ | ✅ | ✅ | ✅ | ⛔ | 🟡 | Runtime-pending (Phase B) |
| `channel_recv`     | ✅ | ✅ | ✅ | ✅ | ⛔ | 🟡 | Runtime-pending (Phase B) |
| `service_health`   | ⛔ | ⛔ | ⛔ | ⛔ | ⛔ | ⛔ | Not started |
| `service_metrics`  | ⛔ | ⛔ | ⛔ | ⛔ | ⛔ | ⛔ | Not started |
| `capability_check` | 🟡 | 🟡 | ⛔ | ⛔ | ⛔ | 🟡 | Types only, no enforcement |

\* The test guards the *honest-failure* path: a `spawn`/`join` program passes
`check` but `compile` stops with a runtime-pending message. It does not test a
working runtime (there isn't one yet).

## Notes per feature

- **print / sleep / yield** — fully real. Backed by direct syscalls in the Linux
  runtime assembly (`write`, `nanosleep`, `sched_yield`); no Rust std linked.
  Proven end-to-end and guarded by `runtime_sleep_and_yield`.

- **spawn / join / channel_*** — the whole front of the pipeline is ready: parsed,
  lowered, and code-generated as calls to the fixed actor ABI. What's missing is
  the runtime backend. The compiler refuses to build such programs (honest error),
  so they can't produce broken executables. Backend planned for Phase B over
  `pthread` (not Rust std). Guarded by `actor_spawn_check_passes_but_compile_is_pending`.

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
