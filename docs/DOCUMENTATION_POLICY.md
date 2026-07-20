# Documentation Policy & Phase Gate

**Status:** Adopted
**Why this exists:** Documentation drifted badly — at adoption the repo held ~99
markdown files / ~24k lines of mixed currency, with stale version claims and code
samples that no longer parse. This policy makes documentation currency a **phase
gate**: drift is caught and fixed as part of normal change, not left to rot.

## Document tiers

Every documentation file belongs to exactly one tier.

### 1. Current authority entry point

[`architecture/current-authority.md`](architecture/current-authority.md) is the
single navigation and decision entry point for current invariants, debt
disposition, and work sequence. It does not override live source and executable
tests for implemented behavior. No other document may define a competing active
sequence.

### 2. Scoped current evidence

These must be current within their named scope and updated in the same batch as
the behavior or decision they describe:

- `README.md` and `examples/` — verified front door and syntax evidence;
- explicitly linked architecture, governance, lifecycle, and contract records;
- `CHANGELOG.md` — current change record;
- `ROADMAP_v2.md` — long-horizon phase baseline and history, not current
  work-sequence authority.

An entire directory is never authoritative merely because of its path. Code
samples presented as current must pass `logicodex check`.

### 3. Reference (kept current best-effort)
Useful, updated when touched, but not release-blocking.

- `SPECIFICATION.md`, `docs/HANDBOOK.md`, `CONTRIBUTING.md`, and guide material
  carrying an explicit stale/reference banner;
  (Earlier reference docs — `MANUAL.md`, `GETTING_STARTED.md`, `GRAMMAR_ANALYSIS.md`,
  `SYNTAX_ANALYSIS.md`, `GrammarandDictionary.md`, `ENVIRONMENT_SETUP.md` — were
  archived to `docs/archive/` as they predated the single-engine architecture.)
- `docs/guide/src/ch07`+ that describe **planned/profile** features must carry a
  status banner (see below).

### 4. Planned / profile (label, don't pretend)
Docs for features that are not yet built (runtime profiles, actors, channels,
network reactor, full freestanding, WASM linking, raylib-on-HIR). These MUST begin
with a banner:

> **Status: PLANNED — not yet implemented.** Describes intended design; code here
> may not compile on the current build.

Per the runtime doctrine, runtime features are opt-in profiles; their docs are
planned until the profile ships.

### 5. Historical (frozen, archived)
Snapshots of past versions. Never edited for currency; they are a record.

- `docs/archive/*`, `spec/v1.11-alpha/*`, `spec/v1.21-alpha/*`, version-stamped design notes.
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

1. `README.md` (front door) — done; `GETTING_STARTED.md` archived (dual-engine, to be rewritten)
2. `SPECIFICATION.md`, `docs/guide/src/ch01`-`ch06` (core syntax)
3. Reference tier (sweep stale version/metric claims)
4. Banner the planned/profile guide chapters (`ch07`+)
5. Archive anything obsolete not already in `docs/archive/` — done (dual-engine docs + one-shot scripts archived)

This backlog is worked incrementally; each repaired doc must satisfy the phase gate
above before it is considered done.
