# Capability Topology — Current State & Future Work (Issue #07)

This documents what the capability subsystem enforces today, and why **provider
topology verification** is intentionally deferred.

## What is enforced today (active)

`check` runs a compile-time **vocabulary check**: every `Service` `requires`
clause must name a gate in the standard vocabulary (`all_standard_domains` in
`src/tier2/gate.rs`). A malformed gate (not `Domain.Operation`) is an error; a
well-formed gate outside the vocabulary is a warning. This is consistent with the
fabric's "zero runtime mediation" design — verification is purely compile-time.

The full standard gate vocabulary is reachable from source (e.g. `Net.Send`,
`Storage.Baca`, `UI.Papar`, `HW.GPIO`) thanks to the parser accepting
keyword-named members and keyword namespace bases.

## What is NOT enforced (deferred): provider topology

`src/tier2/topology.rs` contains a `CapabilityTopology::verify` that checks a
stronger property: **every required gate must have a provider**. It is
**dormant** — not invoked by the pipeline — and is deliberately left that way for
now.

### Why deferred

1. **Provider model is heuristic, not declared.** There is no `provides` surface
   syntax. Providers are *inferred* by `infer_gate_contract` from a function's
   name (e.g. `driver_*` / `hw_*` auto-provide HW gates) and requires are inferred
   from its body (e.g. `PAPAR` => requires `UI.Papar`). This inference is a sketch.

2. **It would over-flag normal programs.** Under the current inference, any
   program using `PAPAR` requires `UI.Papar`, but nothing provides `UI.Papar`
   (the name heuristic only provides HW gates). Enforcing topology as-is would
   reject most ordinary programs with spurious "missing provider" violations.

3. **Architecture boundary.** Topology verification operates on the `ast::Program`
   structure. The `check` entry point lives in the binary, which has its own
   `ast` module distinct from the library's. The current vocabulary check works
   because it only passes strings across the boundary; topology would require a
   library-side entry point that parses and verifies end-to-end.

### What a real implementation needs (revisit checklist)

- **A provider model**, one of:
  - *System-provided standard gates*: treat the standard vocabulary (UI, Storage,
    Net, HW, Audio, Crypto) as provided by the runtime/system, so topology only
    flags genuinely-unprovided custom gates; or
  - *A `provides` surface syntax*: let functions/modules declare the gates they
    provide, making the graph explicit rather than name-inferred.
- **Refined inference** so requires/provides reflect real usage, not heuristics.
- **A library entry point** (`logicodex::tier2::verify_topology(source) ->
  violations`) so the binary's `check` can run topology verification without
  crossing the bin/lib `ast` boundary.

### Impact of deferring

None on functionality. `PAPAR` and every other construct compile and run exactly
as before; the topology code stays dormant as it has always been. Only the
already-active vocabulary check runs in `check`. Deferring keeps Issue #07
honestly **PARTIAL** without regressing normal programs.

Revisit when a provider model is decided (system-provided gates vs `provides`
syntax) or when a concrete need arises.
