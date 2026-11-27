# Master Roadmap Adoption Report

Status: Governance evidence record — not a roadmap
Recorded: 2026-11-27
Branch: `docs/master-roadmap-adoption-20261127`
Base: `9a162580b5dd8aa5c0ce083ea023ae721d840f48` (`origin/main`, merge of PR #105)

This record documents the adoption of
[`master-roadmap.md`](master-roadmap.md) as the sole long-term roadmap. It is
normative governance evidence for that adoption only. It defines no direction,
stage placement, or work sequence. Direction and stage placement belong to the
master roadmap; the active execution sequence belongs only to
[`current-authority.md`](current-authority.md).

Evidence labels used below:

- **[LIVE]** — verified in this worktree against live files, git history, or
  validator output during this change.
- **[OWNER]** — owner-authorized decision (the supplied master roadmap or the
  approved adoption plan).
- **[HIST]** — historical provenance from a superseded record.
- **[UNKNOWN]** — unresolved; not established from live records and not
  invented here.

Owner rule applied throughout: the supplied master roadmap is authoritative and
was copied byte-for-byte. No review finding, audit finding, validator preference,
or agent judgment changed its content. Every conflict is recorded in this report
only; the roadmap text was not amended.

## Live baseline

- [LIVE] `origin/main` and this branch's `HEAD` are both
  `9a162580b5dd8aa5c0ce083ea023ae721d840f48`, the merge of PR #105.
- [LIVE] Recent merges on `origin/main`: PR #105 (CPB-2 assignment/return
  closure), PR #104 (CPB-2 callable context migration), PR #103 (release
  toolchain), PR #102 (CI toolchain candidate).
- [LIVE] PR #100 and PR #101 are in state `CLOSED` (queried read-only).
- [LIVE] `Cargo.toml` version is `0.46.0-alpha`; unchanged by this change.
- [LIVE] `current-authority.md` owner-locked sequence before and after this
  change is exactly: `CPB-2 Callable and Function Type Closure`,
  `CPB-2 Assignment and Return Compatibility`, `CPB-2 Diagnostic End-to-End`.
- [LIVE] Governance label `phase-1` remains the active label in
  `current-authority.md`.
- [OWNER] Master roadmap section 3 states `cargo test --locked` 609 passed,
  0 failed, 3 ignored at the baseline. [LIVE] The final full-integrity run for
  this change passed `full_cargo_test` with 612 enumerated test entries; the
  per-outcome breakdown was not captured in the gate summary, so the 609 figure
  is neither confirmed nor contradicted here.

## Canonical roadmap

- Path: `docs/architecture/master-roadmap.md` (new file).
- Source: owner-supplied document `doc_ef66561a0741_master-roadmap.md`
  (19445 bytes).
- SHA-256 of source and of the repository copy:
  `1256ce4441eb2ee07376bad8eb0da18a0a505fd3d617c6f1a4ab490c9ad1deb4`.
- [LIVE] Copied before any other edit; `cmp` reported no difference and both
  SHA-256 digests match. Re-verified after all edits (see `Validation results`).
- [OWNER] Content: eight stages A–H in order; Stage C defines Beta; Stage G is
  Production Community Compiler / Stable `1.0.0` with LIM-5 full self-hosting;
  Stage H is Backend Independence (LIM-3 LDB/VM, LIM-4 native backend),
  post-stable and separately authorized; CPB-2 split into CPB-2S (Stage A) and
  CPB-2L (Stage B).

## Roadmap authority inventory

| Document | Authority before | Authority after | Content retained |
|---|---|---|---|
| `docs/architecture/master-roadmap.md` | Did not exist | Sole long-term roadmap: direction and stage placement | Verbatim owner content |
| `docs/architecture/current-authority.md` | Sole active execution authority; `ROADMAP_v2.md` long-horizon context | Sole active execution authority; points to the master roadmap for direction | Sequence, invariants, gates unchanged |
| `ROADMAP_v2.md` | Active long-horizon baseline | Historical phase baseline, provenance only | All phase content retained; notice added |
| `docs/architecture/cpb-self-hosting-runway.md` | Active planning document | Subordinate CPB capability requirements | All CPB phases retained; placement notice added |
| `docs/architecture/cpb-next-roadmap-blockers.md` | Blocker inventory with long-horizon list | Subordinate P0–P2 blocker inventory | All blockers retained; long-horizon list marked historical |
| `.github/ROADMAP_POLICY.md` | Compliance process; logged records into `ROADMAP_v2.md` | Compliance process; records go to `CHANGELOG.md` and current authority | Policy retained; version 1.2.0 |
| `docs/DOCUMENTATION_POLICY.md` | `ROADMAP_v2.md` as scoped current evidence | Master roadmap named; `ROADMAP_v2.md` provenance only | Tiers unchanged |
| `README.md`, `.github/AUDIT_TEMPLATE.md`, `AGENTS.md` | Pointed to `ROADMAP_v2.md` for long-horizon criteria | Point to the master roadmap for stage placement and closure | Existing pointers kept |
| LIM-1 to LIM-5 planning document | [LIVE] No repository document found | Master roadmap is the defining record for LIM placement | Not invented |
| `Cargo.toml` | Release version authority | Release version authority (unchanged) | Unchanged |
| Historical release roadmaps in `docs/archive/**`, `spec/v*/**` | Provenance only | Provenance only (unchanged) | Unchanged |

## Capability traceability matrix

Each row has exactly one classification from: `RETAINED`, `RELOCATED`,
`ALREADY IMPLEMENTED`, `SUPERSEDED BY APPROVED DESIGN`,
`DEFERRED WITH OWNER APPROVAL`, `MISSING`, `CONFLICTING`. "Stage" is the master
roadmap placement; `—` means no placement exists in the master roadmap.

| ID | Requirement | Original source | Stage | Classification | Evidence / gap |
|---|---|---|---|---|---|
| T01 | SSM-D4 transition gate | `current-authority.md` SSM-D4 gate | A | RETAINED | [LIVE] Gate text unchanged; integration status not altered by this change |
| T02 | CPB-2S item 1: Callable and Function Type Closure | `current-authority.md` sequence item 1 | A | RETAINED | [LIVE] PR #104 merged (`a77adfd`); [OWNER] formal closure not recorded; not closed here |
| T03 | CPB-2S item 2: Assignment and Return Compatibility | `current-authority.md` sequence item 2 | A | RETAINED | [LIVE] PR #105 merged (`9a16258`); formal closure not recorded; not closed here |
| T04 | CPB-2S item 3: Diagnostic End-to-End | `current-authority.md` sequence item 3 | A | RETAINED | [LIVE] Next in sequence; not started by this change |
| T05 | P0.1 diagnostics hardening, incl. removal of unsafe fallback-to-zero | `cpb-next-roadmap-blockers.md` P0.1 | A (active pipeline), then cross-stage P0 track | RELOCATED | [LIVE] `CHANGELOG.md` Unreleased records partial fallback-zero conversions; remainder open |
| T06 | P0.2 type checker hardening (args, returns, array/slice, pointer/FFI, layout) | `cpb-next-roadmap-blockers.md` P0.2 | A–D via P0 track | RELOCATED | [OWNER] Section 7 P0 track splits scope per stage |
| T07 | P0.3 trust and documentation hygiene | `cpb-next-roadmap-blockers.md` P0.3 | All, cross-stage track | RELOCATED | [OWNER] Section 7 trust-state and documentation hygiene track |
| T08 | CPB-0 Contract Discipline | `cpb-self-hosting-runway.md` CPB-0 | — | MISSING | [HIST] Runway records `Status: Done`; [UNKNOWN] no master-roadmap stage placement and no section 8 closure record |
| T09 | CPB-1 Bootstrap Surface | `cpb-self-hosting-runway.md` CPB-1; `compiler-subset.md` | — | MISSING | [LIVE] `compiler-subset.md` is `Active CPB-1 planning document`; [UNKNOWN] no stage placement or closure status |
| T10 | CPB-2L bootstrap stdlib slice | `cpb-self-hosting-runway.md` CPB-2 | B | RELOCATED | [LIVE] Runway: prelude/text/option/result Stage 1 slices exist; array, path, file, io pending |
| T11 | P1.1 `core.array` Stage 1 | `cpb-next-roadmap-blockers.md` P1.1 | B | RELOCATED | [LIVE] Fixed local array foundation exists; parameter/return barrier recorded in runway |
| T12 | P1.2 `core.text` / `core.string` Stage 2 | `cpb-next-roadmap-blockers.md` P1.2 | B | RELOCATED | [OWNER] Stage B CPB-2L scope |
| T13 | P1.3 `std.path` | `cpb-next-roadmap-blockers.md` P1.3 | B | RELOCATED | [LIVE] `std-path-foundation.md` lexical foundation only |
| T14 | P1.4 `std.file` / `std.io`, `IoError` model | `cpb-next-roadmap-blockers.md` P1.4 | B | RELOCATED | [LIVE] `std-file-io-boundary.md` boundary record only |
| T15 | CPB-3 compiler API freeze boundary | `cpb-self-hosting-runway.md` CPB-3 | B (end of stage) | RELOCATED | [OWNER] Stage B scope |
| T16 | P2.1 FFI-C1 libm | `cpb-next-roadmap-blockers.md` P2.1 | C | RELOCATED | [OWNER] Stage C FFI gate order |
| T17 | P2.2 package metadata reader | `cpb-next-roadmap-blockers.md` P2.2 | C | RELOCATED | [OWNER] Covered by section 5.2 range P2.1–P2.6; not named in Stage C scope bullets |
| T18 | P2.3 `ldx-pkg` local MVP | `cpb-next-roadmap-blockers.md` P2.3 | C | RELOCATED | [OWNER] Stage C scope, no network registry |
| T19 | P2.4–P2.6 FFI-C2 custom C, FFI-C3 sqlite3, FFI-C4 Raylib | `cpb-next-roadmap-blockers.md` P2.4–P2.6 | C | RELOCATED | [OWNER] Stage C FFI gate order |
| T20 | Package manager hold rule | `cpb-next-roadmap-blockers.md`; `compiler-subset.md` | C (after B foundations) | RETAINED | [LIVE] Rule text unchanged; consistent with Stage B before C |
| T21 | CPB-4 compiler subset programs | `cpb-self-hosting-runway.md` CPB-4 | C | RELOCATED | [OWNER] Stage C scope |
| T22 | CPB-5 first self-hosting loop | `cpb-self-hosting-runway.md` CPB-5 | D (D1) | RELOCATED | [OWNER] D1, not blocked by traits |
| T23 | Runway legacy rule, CPB entry and exit gates | `cpb-self-hosting-runway.md` | All CPB work | RETAINED | [LIVE] Text unchanged; validator terms preserved |
| T24 | ROADMAP_v2 Phase 1 HARDEN umbrella (P1-D1–P1-D8, P1-A1–P1-A10) | `ROADMAP_v2.md` Phase 1 | — | MISSING | [OWNER] Section 5.2 maps Phases 2–5 only; [HIST] most P1-A items unchecked; no placement for the umbrella |
| T25 | P1-D5 x86_64 QEMU boot | `ROADMAP_v2.md` P1-D5 / P1-A5 | — | ALREADY IMPLEMENTED | [HIST] P1-A5 ticked; [LIVE] `freestanding_boot_evidence` and `freestanding_boot_e2e` passed in the final full-integrity run |
| T26 | P1-D6 HIR decision | `ROADMAP_v2.md` P1-D6 | — | ALREADY IMPLEMENTED | [LIVE] `docs/architecture/hir-decision.md` exists; HIR sole path is a locked current fact |
| T27 | P1-D1 / P1-D7 14-day CI and test stability | `ROADMAP_v2.md` P1-D1, P1-D7 | — | SUPERSEDED BY APPROVED DESIGN | [HIST] Maturity matrix: historical 14-day requirement accepted via freeze-exit decision of 2026-07-13 |
| T28 | P2-D1 nominal type system with 10+ integration tests | `ROADMAP_v2.md` P2-D1 | — | MISSING | [HIST] Matrix lists structs/enums lowering as FULL; [UNKNOWN] P2-D1 criteria not placed in any stage |
| T29 | P2-D2 type inference boundaries | `ROADMAP_v2.md` P2-D2 | — | MISSING | No stage placement in the master roadmap |
| T30 | P2-D5 bilingual diagnostic regression coverage for all existing error variants | `ROADMAP_v2.md` P2-D5 | — | MISSING | [OWNER] Bilingual track is cross-stage, but this explicit coverage item is not placed |
| T31 | P2-D3 `ldx-fmt`, P2-D4 LSP foundation | `ROADMAP_v2.md` Phase 2 | D (D6) | RELOCATED | [OWNER] Section 5.2 and D6 |
| T32 | P3-D1 actor pool, P3-D5 deterministic execution, P3-D6 supervision | `ROADMAP_v2.md` Phase 3 | E | RELOCATED | [OWNER] Stage E optional concurrent runtime profile |
| T33 | P3-D2 zero-copy SPSC channels | `ROADMAP_v2.md` P3-D2 | E | RELOCATED | [OWNER] Stage E channels; throughput criterion not restated |
| T34 | P3-D3 capability hardware gate (debug panic / release error, 5+ tests) | `ROADMAP_v2.md` P3-D3 | — | MISSING | [OWNER] Stage E has runtime capability enforcement; hardware-gate criteria not placed |
| T35 | P3-D4 cross-shard messaging / sharded runtime | `ROADMAP_v2.md` P3-D4 | — | MISSING | No stage placement for sharding |
| T36 | P3-A3 Miri-clean runtime audit | `ROADMAP_v2.md` P3-A3 | — | MISSING | No stage placement for Miri evidence |
| T37 | P3-D7 runtime benchmarks | `ROADMAP_v2.md` P3-D7 | All, cross-stage track | RELOCATED | [OWNER] Section 7 performance and security regression track |
| T38 | P5-D7 network reactor completion | `ROADMAP_v2.md` P5-D7 | E | RELOCATED | [OWNER] Stage E networking (moved from historical Phase 5) |
| T39 | P4-D1/D2/D4/D5/D9 aarch64, riscv64, QEMU boots, multi-arch CI | `ROADMAP_v2.md` Phase 4 | F | RELOCATED | [OWNER] Stage F; ARM64 and RISC-V freestanding in QEMU |
| T40 | P4-D6 WASM linker with no emscripten dependency | `ROADMAP_v2.md` P4-D6 | E | SUPERSEDED BY APPROVED DESIGN | [OWNER] Stage E WASM/Browser uses Emscripten; recorded as conflict C1 |
| T41 | P4-D7 / P4-D8 WASM runtime and CI | `ROADMAP_v2.md` P4-D7, P4-D8 | E | RELOCATED | [OWNER] Stage E WASI (Wasmtime) and Browser profiles; target profile per Stage E |
| T42 | P4-D10 Raylib FFI completion | `ROADMAP_v2.md` P4-D10 | C (desktop), E (browser) | RELOCATED | [OWNER] FFI-C4 and Stage E exit proof |
| T43 | P5-D1 `ldx-pkg` full package manager with registry | `ROADMAP_v2.md` P5-D1 | G (registry); C (local MVP) | RELOCATED | [OWNER] Stage G package registry |
| T44 | P5-D2 self-hosting with three-stage bootstrap | `ROADMAP_v2.md` P5-D2 | G | RELOCATED | [OWNER] LIM-5, reproducible bootstrap exit proof |
| T45 | P5-D3 formal semantics | `ROADMAP_v2.md` P5-D3 | D | RELOCATED | [OWNER] LIM-1 across D; PDF artifact and external reviewer criteria not restated |
| T46 | P5-D4 stable release as `v2.0.0` | `ROADMAP_v2.md` P5-D4 | G | SUPERSEDED BY APPROVED DESIGN | [OWNER] Stable Community is `1.0.0` (decision 10); `2.0.0` only for breaking API change |
| T47 | Phase completion tags `v1.49.0`, `v1.59.0`, `v1.69.0`, `v1.99.0` | `ROADMAP_v2.md` entry criteria | — | SUPERSEDED BY APPROVED DESIGN | [OWNER] Section 6 version conventions; `Cargo.toml` is release authority |
| T48 | P5-D5 public API and compatibility policy | `ROADMAP_v2.md` P5-D5 | G | RELOCATED | [OWNER] Stage G public API and compatibility policy |
| T49 | P5-D5 `#[stable]` attribute and `docs/stability.md` deprecation policy | `ROADMAP_v2.md` P5-D5 / P5-A5 | — | MISSING | [LIVE] `docs/stability.md` absent; mechanism not placed |
| T50 | Phase 5 post-mortem document | `ROADMAP_v2.md` Phase 5 exit proof 6 | — | MISSING | [LIVE] `docs/postmortem/` absent; not placed |
| T51 | P5-D6 ecosystem seed (3+ published packages) | `ROADMAP_v2.md` P5-D6 | — | MISSING | Not placed; Stage G registry only |
| T52 | P5-A10 security audit | `ROADMAP_v2.md` P5-A10 | G | RELOCATED | [OWNER] Stage G security review |
| T53 | Independent auditor sign-off before any phase transition | `ROADMAP_v2.md` summary; `.github/ROADMAP_POLICY.md` 1.3 | All | CONFLICTING | [OWNER] Section 8 item 8 requires owner authorization; independent audit not mentioned; conflict C3 |
| T54 | Cross-phase regression requirements | `ROADMAP_v2.md` cross-phase section | All | RELOCATED | [OWNER] Section 8 item 3 regression evidence |
| T55 | Risk register | `ROADMAP_v2.md` risk register | — | RETAINED | [HIST] Retained in place as provenance; no master-roadmap equivalent |
| T56 | LIM-1 to LIM-5 source requirements | Master roadmap only | D, D, H, H, G | MISSING | [LIVE] No LIM planning document in the repository; master roadmap is the defining record; nothing invented |
| T57 | Bilingual syntax and diagnostics (`dict/core_map.json`) | Master roadmap invariants; existing practice | All | RETAINED | [LIVE] `dict/core_map.json` exists |
| T58 | LLVM sole production backend | `current-authority.md` locked facts | Until H | RETAINED | [LIVE] Locked fact now references Stage H authorization |
| T59 | Enterprise / Assurance outside Community roadmap | Blockers, runway non-goals | — | RETAINED | [OWNER] Section 2 rule 3; consistent |
| T60 | Six-item long-horizon list (Compiler Foundation … Assurance Edition) | `cpb-next-roadmap-blockers.md` | — | SUPERSEDED BY APPROVED DESIGN | [LIVE] Marked historical in place; stages A–H govern |
| T61 | Emergency override, violation, and reversion logging in `ROADMAP_v2.md` | `.github/ROADMAP_POLICY.md` 4.3, 7.3, 8.4, 9.2–9.5 | — | RELOCATED | [LIVE] Redirected to `CHANGELOG.md` and current authority |
| T62 | Policy amendment via RFC and 7-day discussion | `.github/ROADMAP_POLICY.md` 10.1 | — | CONFLICTING | [OWNER] Adoption approved directly by owner; conflict C4 |

## Current execution status

- [LIVE] The only active execution sequence remains the three-item list in
  `current-authority.md`, unchanged in wording and order. This change added a
  non-numbered note that those items are CPB-2S in Stage A.
- [OWNER] Stage A is current. Items 1 and 2 have merged implementations; item 3
  is next. No item is recorded as closed, and this change records no closure.
- [LIVE] No stage B–H work is authorized by this change.

## Retained technical requirements

All requirements in the superseded or subordinate records remain in place,
unchanged in substance:

- `ROADMAP_v2.md`: all phase deliverables, audit checklists, exit proofs, open
  issue lists, regression policy, risk register, and glossary are retained as
  historical provenance. Only the status line, last-updated date, an adoption
  notice, and a document-history row were added.
- `cpb-self-hosting-runway.md`: CPB-0 to CPB-5, non-goals, legacy rule,
  entry/exit gates, and blocker status sections are retained. A placement notice
  and the CPB-2L identifier were added.
- `cpb-next-roadmap-blockers.md`: P0–P2 blockers, package manager hold rule, and
  work-sequence boundary are retained. A placement note was added and the
  long-horizon list was marked historical.
- `.github/ROADMAP_POLICY.md`: alignment checks, RFC rules, PG rules, feature
  classes, PR checklists, override process, and consequences are retained; only
  record destinations, authority pointers, and policy version changed.

## Missing or conflicting requirements

Missing (reported, not invented):

- **ROADMAP_v2 HARDEN umbrella** — Phase 1 has no master-roadmap mapping (T24).
- **Nominal, inference, and diagnostic coverage items** — P2-D1, P2-D2, P2-D5
  are not placed (T28–T30).
- **Runtime sharding, Miri, capability hardware gate** — P3-D4, P3-A3, and
  P3-D3 hardware-gate criteria are not placed (T34–T36).
- **API stable policy and post-mortem** — the `#[stable]` mechanism,
  deprecation policy document, and post-mortem requirement are not placed
  (T49, T50). The general API and compatibility policy is placed in Stage G (T48).
- **CPB-0 / CPB-1 exact placement** — neither has a stage in the master
  roadmap; CPB-0 is recorded `Done` only in the runway, and CPB-1 has no
  recorded closure (T08, T09).
- **LIM source absence** — no LIM-1 to LIM-5 planning document exists in the
  repository; the master roadmap is the defining record (T56).
- Ecosystem seed packages (T51).

Conflicts (master roadmap preserved in every case):

- **C1 Emscripten** — `ROADMAP_v2.md` P4-D6 requires no Emscripten dependency;
  master roadmap Stage E WASM/Browser uses Emscripten. Resolved by owner rule in
  favor of the master roadmap; the historical statement is marked superseded,
  not removed.
- **C2 Version** — `ROADMAP_v2.md` targets `v2.0.0` as stable and phase tags
  `v1.49.0`–`v1.99.0`; the master roadmap sets Stable Community at `1.0.0` with
  stage version conventions. Resolved by owner rule; historical targets are
  obsolete as normative targets, and `Cargo.toml` remains release authority.
- **C3 Sign-off** — `ROADMAP_v2.md` and `.github/ROADMAP_POLICY.md` 1.3 require
  independent auditor sign-off (maintainer may not self-audit); master roadmap
  section 8 requires owner authorization and does not mention independent
  audit. Not resolved here; listed as an outstanding owner decision.
- **C4 Policy amendment process** — `.github/ROADMAP_POLICY.md` 10.1 requires
  an RFC and a 7-day discussion for amendments; this change amended the policy
  (1.2.0) under direct owner approval. Listed as an outstanding owner decision.
- **C5 Stage H wording** — the adoption plan describes Stage H as a "committed
  strategic Full Capabilities milestone"; the master roadmap text describes it
  as post-stable and separately authorized and does not use the term "Full
  Capabilities". The roadmap text is preserved verbatim; no wording was added.

## Updated authority references

- `docs/architecture/current-authority.md` — authority order item 5 names the
  master roadmap; new `Long-term direction` section; LLVM locked fact references
  Stage H; CPB-2S note under the sequence; subordinate records table adds the
  master roadmap and this report and re-describes runway, blockers, and
  `ROADMAP_v2.md`.
- `ROADMAP_v2.md` — status line now historical provenance (still contains
  `historical freeze exited`); adoption notice; history row.
- `.github/ROADMAP_POLICY.md` — banner, 3.3, 3.4, 4.3, 7.3 step 7, 8.4, 9.2,
  9.3, 9.5, 10 history, appendix A.
- `docs/architecture/cpb-self-hosting-runway.md`,
  `docs/architecture/cpb-next-roadmap-blockers.md` — placement notices.
- `docs/DOCUMENTATION_POLICY.md`, `README.md`, `.github/AUDIT_TEMPLATE.md` —
  master roadmap pointers; `ROADMAP_v2.md` described as provenance only.
- `CHANGELOG.md` — Unreleased `Governance` entry.

Out-of-manifest references observed and not changed:
`.github/workflows/gatekeeper.yml` lists `ROADMAP_v2.md` (and a non-existent
`ROADMAP.md`) only as documentation paths for change detection; it treats
neither as authority. `.github/CODEOWNERS` has a global default owner covering
the new files.

## Agent instruction alignment

- [LIVE] `AGENTS.md` is the only agent instruction file; no repository
  `CLAUDE.md` exists.
- `AGENTS.md` still points first to `current-authority.md`, now names the master
  roadmap as the sole source of direction and stage placement, states that
  placement is not authorization, forbids editing the master roadmap without
  owner approval, and adds `ROADMAP_v2.md` and subordinate phase orders to the
  list of non-authority sources.

## Governance validator impact

No validator was changed. Literal contracts were preserved:

- `verify_authority_docs.py` / `test_authority_docs.py` — required headings, the
  exact three-item sequence, `phase-1`, size-policy text, and all pointers to
  `docs/architecture/current-authority.md` are intact. The heading for the active
  sequence still appears only in `current-authority.md`; the master roadmap and
  this report do not contain it.
- `verify_architecture_change_control.sh` — reads the first `**Status:**` line of
  `ROADMAP_v2.md`, which still contains `historical freeze exited` and no quote
  characters. It needs a Python interpreter with PyYAML; the final validation
  environment provided one, and the gate passed. An interpreter without PyYAML
  fails this gate for environmental reasons, not because of repository content.
- `verify_cpb_self_hosting_runway.sh` — all required CPB terms and pointers are
  intact, including the `CPB-2: Bootstrap Stdlib Slice` heading.
- Version-reference hygiene scans tracked and untracked files; the master
  roadmap and the edited files pass.
- `.github/ROADMAP_POLICY.md` contains no unqualified roadmap pointer, and
  `.github/AUDIT_TEMPLATE.md` contains none of the stale audit tokens.

## Validation results

Final validation ran on 2026-11-27 against the complete candidate worktree,
after the last report edit, in a user-local validation environment with an
LLVM 23 toolchain (23.1.6, as reported for the validation run; the gate logs do
not record the patch version). Each result is the command's own exit status.
Durable logs are kept outside the repository and are not cited here by path.

| Gate | Result |
|---|---|
| `cmp` + SHA-256, owner source vs `docs/architecture/master-roadmap.md` | PASS — `roadmap_cmp=True`; SHA-256 `1256ce4441eb2ee07376bad8eb0da18a0a505fd3d617c6f1a4ab490c9ad1deb4` |
| `git diff --check` | PASS (0) |
| `python3 -B scripts/dev/test_authority_docs.py` | PASS (0) — all 14 self-test scenarios |
| `python3 -B scripts/dev/verify_authority_docs.py` | PASS (0) — 12 pointer files, 3 evidence files |
| `./scripts/dev/verify_architecture_change_control.sh` | PASS (0) — workflow structure, label scenarios, policy summary evaluator, stale policy text |
| `python3 -B scripts/dev/test_version_reference_hygiene.py` | PASS (0) |
| `BASE_SHA=$(git merge-base origin/main HEAD) python3 -B scripts/dev/verify_version_reference_hygiene.py` | PASS (0) — base `9a162580…`; toolchain policy llvm23/rust1.100; release workflow contract PASS |
| `python3 -B scripts/dev/test_code_lifecycle.py` | PASS (0) |
| `python3 -B scripts/dev/verify_code_lifecycle.py` | PASS (0) |
| `./scripts/dev/verify_cpb_self_hosting_runway.sh` | PASS (0) — including the Stage 0 stdlib regression |
| `make quick` | PASS (0) — all 13 gates rc=0, `quick_integrity=PASS` |
| `make integrity` | PASS (0) — all 23 gates rc=0 (including `cargo_clippy`, `release_build`, `full_cargo_test`, `semantic_authority`, `contract_cases`, `stdlib_stage0`, `cpb_runway`, freestanding boot evidence and e2e); 612 enumerated test entries; `full_integrity=PASS` |
| Changed-path allowlist | PASS — exactly the 12 manifest paths; nothing outside the allowlist |
| Protected source/test paths | PASS — none touched |
| Report completeness | PASS — `report_sections_missing=[]`; 62 traceability rows, each with exactly one permitted classification; all named audit gaps present |
| Portability | PASS — `private_path_hits=False`; no private absolute filesystem path, credential, or token in changed documentation |
| Relative links in changed documents | PASS — no broken links except the pre-existing `(URL)` placeholders in the audit template |

Earlier runs on a host without an LLVM 23 toolchain failed only at
compile-dependent gates; those were environmental failures and are superseded
by the final run above. No validation gate remains outstanding for this change.

## Changed-file inventory

Exactly the approved manifest:

1. `docs/architecture/master-roadmap.md` (new, verbatim owner content)
2. `docs/architecture/current-authority.md`
3. `ROADMAP_v2.md`
4. `.github/ROADMAP_POLICY.md`
5. `docs/architecture/cpb-next-roadmap-blockers.md`
6. `docs/architecture/cpb-self-hosting-runway.md`
7. `docs/DOCUMENTATION_POLICY.md`
8. `README.md`
9. `.github/AUDIT_TEMPLATE.md`
10. `AGENTS.md`
11. `CHANGELOG.md`
12. `docs/architecture/master-roadmap-adoption-report.md` (new, this record)

## Outstanding owner decisions

1. C3: whether independent auditor sign-off remains required in addition to
   section 8 owner authorization.
2. C4: whether the 1.2.0 policy amendment needs a retroactive RFC record.
3. Stage placement and closure status for CPB-0 and CPB-1 (T08, T09).
4. Disposition of the ROADMAP_v2 Phase 1 HARDEN umbrella and its unchecked audit
   items (T24).
5. Placement, deferral, or retirement of unplaced items: T28–T30, T34–T36,
   T49–T51, plus the formal-semantics PDF and external-review criteria (T45).
6. Whether LIM-1 to LIM-5 need separate source documents (T56).
7. Whether the `phase-1` governance label should later move to stage
   terminology; validators currently require it unchanged.
8. Whether formal closure of CPB-2S items 1 and 2 should be recorded under
   section 8.

## Unauthorized implementation confirmation

This change is documentation and governance only. No source, test, compiler,
Cargo manifest, lockfile, workflow, validator, release version, tag, push,
merge, release, or milestone closure was authorized or performed. No
historical document was deleted. No future-stage work was activated.

## Next authorized implementation task

`CPB-2 Diagnostic End-to-End` — item 3 of the unchanged sequence in
[`current-authority.md`](current-authority.md) (Stage A, CPB-2S). It is not
started by this change and remains subject to normal scoping and review.
