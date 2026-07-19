# Lxdge Extraction Notes

> **Status:** Historical extraction record. Version references identify provenance and do not define current Logicodex behavior.

This document records what was (and was not) taken from the **Lxdge** project
during the runtime-profile work, and why. It exists so the provenance and honest
status of the runtime layer is auditable.

## Background

Lxdge is a sibling project by the same author, described in its own README as
"forked from Logicodex v1.45". Concretely, Lxdge's `src/` is a copy of the
Logicodex compiler at that point, with a set of `lxdge-*` runtime crates layered
on top. It is **not** an independent runtime library that Logicodex can simply
depend on.

The integration goal was: treat Lxdge as a *runtime-parts donor* for Logicodex's
runtime profiles — never merge its compiler fork back over Logicodex's (which has
since moved on: single HIR engine, freestanding kernel, 0.46.0-alpha).

## What was extracted in Phase D

**Nothing, yet.** Phase D's runtime builtins (`sleep`, `yield`) were written
directly as Linux syscall assembly in Logicodex's own `os::runtime_assembly()`.
They were not taken from Lxdge.

This is deliberate and honest: the premise that "Lxdge saves time on runtime
profiles" turned out to be only partly true (see below), so Phase D took the
path that best preserved Logicodex's minimal/native identity.

## Why the actor runtime is NOT from Lxdge

The most-wanted runtime piece is the actor/channel backend (`logicodex_spawn`,
etc.), which Logicodex codegen already emits calls to. Investigation found that
**Lxdge does not implement these symbols** either. Lxdge's `lxdge-runtime` is an
*edge service lifecycle* runtime (INIT → RUNNING → DRAINING, config, telemetry),
not an actor/threading runtime, and `lxdge-reactor::run()` is scaffolding (it
spawns an empty thread). So there was no working actor runtime to extract.

Conclusion: the actor runtime (Phase B) must be written fresh, over `pthread`.
Lxdge does not shortcut it.

## What Lxdge CAN contribute later (service profile)

Lxdge's one clearly-working runtime component is the sharded epoll reactor in
`lxdge-node/src/reactor_loop.rs` (~676 LOC): it has a real event loop, `/health`
and `/metrics` endpoints, and graceful shutdown. When the **service** profile is
implemented, this is the natural thing to adapt — kept local/single-node, with
no HA/clustering/orchestration brought across.

Honest per-component status observed in Lxdge (from its own STATUS.md):

| Lxdge component        | Reality                                   | Use to us |
|------------------------|-------------------------------------------|-----------|
| `lxdge-node` reactor   | Working epoll loop + health/metrics       | service profile (later) |
| `lxdge-runtime`        | Edge lifecycle/config/telemetry           | maybe, partial |
| `lxdge-allocator`      | arena/bump/slab, not e2e-tested           | optional |
| `lxdge-contracts`      | types                                     | maybe |
| `lxdge-capability`     | types only, no enforcement                | low (placeholder) |
| `lxdge-metrics`        | counters/histograms                       | service profile (later) |
| `lxdge-reactor`        | scaffolding (empty thread)                | low |
| `lxdge-wasm/gateway/proxy` | not implemented                       | none |

## Rule

Logicodex must remain a standalone language/compiler with optional local runtime
profiles. Lxdge stays an optional future HA/distributed/governed runtime — never
a mandatory dependency for normal Logicodex use.
