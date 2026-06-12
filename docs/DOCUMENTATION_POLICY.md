# Documentation Policy & Phase Gate

**Status:** Adopted
**Why this exists:** Documentation drifted badly — at adoption the repo held ~99
markdown files / ~24k lines of mixed currency, with stale version claims and code
samples that no longer parse. This policy makes documentation currency a **phase
gate**: drift is caught and fixed as part of normal change, not left to rot.

## Document tiers

Every documentation file belongs to exactly one tier.

### 1. Authoritative (must always be current)
These describe how the language/compiler works *today*. A change that affects them
**must** update them in the same commit. Code samples in them **must** pass
`logicodex check`.

- `README.md` — front door
- `GETTING_STARTED.md` — first-run experience
- `SPECIFICATION.md` — language spec
- `docs/guide/src/ch01`-`ch06` — core syntax tutorial (variables, types, control
  flow, functions)
- `docs/architecture/*` — current architecture decisions (hir-decision,
  runtime-doctrine, foreign-types, capability-topology)
- `CHANGELOG.md`, `ROADMAP_v2.md`

### 2. Reference (kept current best-effort)
Useful, updated when touched, but not release-blocking.

- `MANUAL.md`, `docs/HANDBOOK.md`, `GRAMMAR_ANALYSIS.md`, `SYNTAX_ANALYSIS.md`,
  `GrammarandDictionary.md`, `ENVIRONMENT_SETUP.md`, `CONTRIBUTING.md`
- `docs/guide/src/ch07`+ that describe **planned/profile** features must carry a
  status banner (see below).

### 3. Planned / profile (label, don't pretend)
Docs for features that are not yet built (runtime profiles, actors, channels,
network reactor, full freestanding, WASM linking, raylib-on-HIR). These MUST begin
with a banner:

> **Status: PLANNED — not yet implemented.** Describes intended design; code here
> may not compile on the current build.

Per the runtime doctrine, runtime features are opt-in profiles; their docs are
planned until the profile ships.

### 4. Historical (frozen, archived)
Snapshots of past versions. Never edited for currency; they are a record.

- `docs/archive/*`, `spec/v1.11-alpha/*`, `spec/v1.21-alpha/*`,
  `v121_execution_design.md`, version-stamped design notes.
- When a doc becomes obsolete, move it here with a one-line note at top stating it
  is archived and what supersedes it.

## The phase gate

A change may not be marked release-ready unless:

1. **Authoritative docs are current** for the behaviour the change touches.
2. **Doc code samples pass.** Any fenced `ldx` sample in an Authoritative doc must
   be reproducible by `logicodex check`. Prefer referencing files under
   `examples/` (which are themselves check-gated by the test suite) over inlining
   unverified code.
3. **No false metrics.** No "N/N passing" or version claims that a fresh
   `cargo test` / `--version` does not reproduce.
4. **Planned features are labelled**, never presented as working.

## Verification hooks

- `examples/*.ldx` are checked by the `shipped_examples_pass_semantic_check`
  phase-gate test — they are the canonical, always-correct code reference. Docs
  should point at them.
- A future `tools/check-doc-samples` may extract fenced `ldx` blocks from
  Authoritative docs and run `check` on each; until it exists, sample currency is
  a manual review item on every doc-affecting change.

## Triage backlog (initial)

Repair order for the existing drift, highest impact first:

1. `README.md`, `GETTING_STARTED.md` (front door, stale versions + old syntax)
2. `SPECIFICATION.md`, `docs/guide/src/ch01`-`ch06` (core syntax)
3. Reference tier (sweep stale version/metric claims)
4. Banner the planned/profile guide chapters (`ch07`+)
5. Archive anything obsolete not already in `docs/archive/`

This backlog is worked incrementally; each repaired doc must satisfy the phase gate
above before it is considered done.
