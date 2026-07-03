# Merge-Readiness Audit -- `feature/stdlib-core` -> `main`

Audit type: full architecture + merge-safety audit
Source branch: `feature/stdlib-core`
Target branch: `main` (0.46.0-alpha, 2026-06-14; reported green CI for 14 days)
Method: offline tree comparison of both branch snapshots, contract-verifier
metadata run, and static inspection. The Rust toolchain and LLVM are not
available in the audit environment, so build/clippy/fmt/test results must be
confirmed by CI (see section 9).

## 1. Verdict

`feature/stdlib-core` is **architecturally safe to merge** to `main`. The change
is large but coherent, additive, generic (no module-specific compiler wiring),
contract-backed, and well-tested. Two **governance gates** and one
**documentation debt** must be cleared first; none is a code-correctness
problem:

1. Architecture-freeze gate will fail (frozen files modified) -- needs a
   maintainer `freeze-override` label. Justified; see section 5.
2. PR size gate (limit 500 lines) will trip -- the diff is far larger. Needs an
   explicit size acknowledgement/override.
3. CHANGELOG only records the Result/Option slice; the rest of the merged work
   is undocumented. Resolve before tagging the new phase (section 7).

Recommendation: **approve for merge after** the CHANGELOG is completed and the
two gate overrides are applied, conditional on a green CI run on the merge
commit.

## 2. Scope of change

Net-new source files (absent in `main`):

| File | Lines | Purpose |
|---|---|---|
| `src/module_loader.rs` | 553 | generic dotted-path module import + resolution |
| `src/lod.rs` | 374 | zero-dependency manifest / C-link dependency layer |
| `src/contract_metadata.rs` | 135 | contract metadata model |
| `src/runtime/runtime_actor.c` | -- | audited pthread actor/channel C backend |

Modified compiler source (changed-line counts from `diff`):

| File | main | stdlib-core | changed lines |
|---|---|---|---|
| `src/hir.rs` | 1656 | 2378 | 812 |
| `src/codegen.rs` | 1807 | 2151 | 662 |
| `src/parser.rs` | 1404 | 1640 | 364 |
| `src/main.rs` | 820 | 1156 | 362 |
| `src/ffi.rs` | 242 | 404 | 162 |
| `src/ast.rs` | 480 | 537 | 61 (frozen) |
| `src/semantic_gate.rs` | 460 | 507 | 47 |
| `src/lexer.rs` | 1020 | 1041 | 21 |
| `src/types.rs` | 741 | 758 | 17 |
| `src/layout.rs` | 353 | 361 | 8 |
| `src/semantic.rs` | 1331 | 1334 | 7 (frozen) |
| `src/semantic/registry.rs`, `src/tier2/pass.rs`, `src/os/linux.rs`, `src/lib.rs` | -- | -- | minor |

Other additions: `tools/verify_stdlib_contracts.py`, `scripts/dev/` verify
scripts, 10 contract-backed `lib/core/*` modules with `.std.toml` sidecars,
18 new docs (97 vs 79), and ~14 new test files.

`main` baseline for context: `lib/core` contains only the 11 legacy modules; it
has no `tools/`, no `scripts/dev/`, and no contract-backed stdlib. The delta is
effectively the entire CPB Phase 1 stdlib programme plus the module system, the
`lod` layer, the collections/result/option foundations, the actor-runtime
completion, and the contract-verification infrastructure.

## 3. `main` baseline audit

- CI (`.github/workflows/ci.yml`): three jobs -- `check` (rustfmt, clippy,
  cargo check), `test` (cargo build --release + full `cargo test`), and
  `freestanding` (QEMU multiboot boot evidence + boot-e2e + freestanding
  validator). Toolchain Rust 1.75.0, LLVM 15.
- Debt markers in `src`: 11 `TODO`, 11 `panic!(` -- consistent with a compiler
  codebase; no `unimplemented!`/`todo!`.
- `DEFERRED.md` present and tracks deferred work; identical on both branches.
- `lib/core` is legacy-only (pre-Stage-0). These legacy files are source
  material, not trusted modules.
- No blocking debt was found in `main` itself. It is a clean base to merge onto.

## 4. `feature/stdlib-core` audit

Architecture compliance (against the stdlib doctrine and the "compiler stays
generic" guardrail):

- Module loading is generic: `module_loader.rs` and `lod.rs` contain no
  `if module == "core.x"` branches. The only `core.math` occurrence is an
  example inside a doc comment for the generic path resolver.
- New AST nodes are generic language features (`QualifiedCall`, `ArrayLiteral`,
  `Option` `Some`/`None`, `ChannelCreate`, `Type::Array`, `Type::Option`,
  `Modulo`, `BitXor`), not per-library hacks.
- Stdlib modules are normal `.ldx` + `.std.toml`, discovered through the same
  import path as user code, and validated by `verify_stdlib_contracts.py`.

Contract verifier: ran `python3 tools/verify_stdlib_contracts.py` in the audit
environment -- **exit 0**, all 10 contract sidecars validated
(`assert`, `bits`, `bool`, `compare`, `math`, `range`, `text`, `option`,
`result`, `prelude`). Run-cases (`--run-cases`) require the compiled binary and
are exercised by CI, not by this static audit.

Tests: 158 `#[test]` functions across the test suite, including new e2e suites
for the module system, dotted paths, result/option foundation, collections
foundation, the modulo operator, root resolution, and each contract-backed
stdlib module. Spot checks confirm the new tests are real (they compile and run
`.ldx` through the actual binary and assert stdout), not stubs.

New capabilities are foundation-scoped honestly: e.g. the collections test file
states its limits explicitly (fixed local arrays, `I64` only, literal +
index read/write; no heap `Vec`/`List`, no slice passing yet).

## 5. Frozen-file analysis (override justification)

The architecture freeze covers `src/ast.rs` and `src/semantic.rs`. Both are
modified, so the gate will flag them. The changes are additive and low-risk:

`src/ast.rs` -- only new, documented additions: a `is_public` field
(module visibility), an actor `params` field (cross-actor capture), match
patterns `Some`/`None`, the `QualifiedCall` node, `ArrayLiteral`, Option
construction `Some`/`None`, `ChannelCreate`, the `Modulo`/`BitXor` operators,
`Type::Array`/`Type::Option`, an `is_option()` helper, and matching `Display`
arms. No existing variant semantics are removed or altered.

`src/semantic.rs` -- four small, defensive edits: `..` rest patterns to tolerate
the new fields, `Stmt::Actor { .., .. }` destructuring, extension of the
divide-by-zero constant check to `Modulo`, and adding `Modulo`/`BitXor` to the
allowed binary operators. No behavioural regression to existing checks.

Conclusion: these are exactly the kind of additive core-structure changes the
freeze is meant to surface for review, and they are safe to approve. A
`freeze-override` label is appropriate.

## 6. Merge governance / gatekeeper

`gatekeeper.yml` on this branch was updated so the size gate and the
architecture-freeze gate **SKIP** when the PR target is not `main`/`master`
(internal `feature/* -> feature/stdlib-core` integration). For the
`feature/stdlib-core -> main` PR, `base_ref == main`, so **both gates run**:

- Architecture freeze: will FAIL on `src/ast.rs` + `src/semantic.rs` unless the
  `freeze-override` label is applied by a maintainer (section 5 justifies it).
- Size gate: `SIZE_LIMIT = 500`. The merge diff is well into the thousands of
  lines and will exceed it; an explicit override/acknowledgement is required.

A minor, positive gatekeeper fix is also present: the freeze-violation comment
now JSON-encodes the violations value (`toJson(...)`) instead of raw backtick
interpolation, removing a script-injection/escaping hazard.

## 7. Debt findings

Primary (resolve before tagging the new phase):

- **CHANGELOG gap.** `CHANGELOG.md` adds only the Result/Option `[Unreleased]`
  block (42 lines). It does not record the module system, the `lod` layer, the
  `core.prelude` baseline, the collections foundation, the Stage 0 stdlib
  modules (`math`/`assert`/`bits`/`bool`/`compare`/`range`), `core.text`, the
  new `Modulo`/`BitXor` operators, or the contract-verifier infrastructure -- all
  of which this branch introduces relative to `main`. The CHANGELOG must be
  expanded to honestly describe the full merge before it represents a phase
  boundary.

Secondary (acceptable, but note):

- Debt markers grew modestly: `TODO` 11 -> 12, `panic!(` 11 -> 14. The added
  `panic!` in `module_loader.rs` is a test-only assertion helper; the added
  `TODO`s are documented limitations (e.g. partial `type_checker.rs`).
- Three `#[ignore]` tests exist in `tests/freestanding_v144_gaps.rs`; all are
  pre-existing, documented crt0 deferrals (issue #3), not new debt.
- `DEFERRED.md` is unchanged from `main`; no new untracked deferrals were added
  there, which is consistent with the deferrals being documented in-place.

Hygiene: no trailing whitespace was found in the sampled new source / `lib/core`
files.

## 8. Risk assessment

| Area | Risk | Notes |
|---|---|---|
| Compiler correctness | Low | additive features; large but coherent; 158 tests |
| Architecture drift | Low | generic loaders, contract-backed modules, no hardcoding |
| Frozen-file change | Low | additive, documented; override justified |
| Merge mechanics | Medium | freeze + size gates require maintainer overrides |
| Documentation | Medium | CHANGELOG omits most merged features |
| Build/test/fmt/clippy | Unverified here | must be confirmed green by CI on the merge commit |

## 9. Verification matrix

| Check | Verified in this audit | Must be confirmed by CI |
|---|---|---|
| Tree/scope diff | Yes | -- |
| Contract verifier (metadata) | Yes (exit 0) | -- |
| Contract run-cases (`--run-cases`) | No (needs binary) | Yes |
| `cargo build --release` | No (no toolchain) | Yes |
| `cargo test` (158 tests) | No | Yes |
| `cargo fmt --check` | No | Yes |
| `cargo clippy --all-targets` | No | Yes |
| Freestanding QEMU boot | No | Yes |
| Frozen-file content review | Yes | -- |
| Gatekeeper gate behaviour | Yes (static) | Yes (live PR) |

## 10. Pre-merge checklist

- [ ] Expand `CHANGELOG.md` to record module system, `lod`, `core.prelude`,
      collections foundation, Stage 0 stdlib modules, `core.text`,
      `Modulo`/`BitXor`, and the contract-verifier infrastructure.
- [ ] Green CI on the merge commit: `check` (fmt + clippy + cargo check +
      verify contracts), `test` (build + run-cases + full `cargo test`),
      `freestanding` (QEMU boot).
- [ ] Maintainer applies `freeze-override` for `src/ast.rs` + `src/semantic.rs`
      after reviewing section 5.
- [ ] Maintainer acknowledges/overrides the size gate for this foundation merge.
- [ ] Confirm `core-trust-state.md` (stdlib inventory) is present or scheduled
      as a fast-follow, so trust state is documented at the phase boundary.

## 11. Conclusion

The branch represents a clean, generic, contract-backed foundation step and is
safe to merge on the merits. Merge readiness is gated only by completing the
CHANGELOG, obtaining the two maintainer gate overrides, and a green CI run on the
merge commit. No code-level blocker was found.
