# Logicodex v1.30 Main Readiness and Safe Merge Plan

**Author:** Manus AI  
**Branch prepared:** `sim/v130-resume`  
**Target branch:** `main`  
**Current production compiler line:** **current Logicodex v1.21 alpha**  
**Merge status:** Prepared for Pull Request review; not merged to `main` by this document.

## Executive Summary

This document records the safe path for merging the dormant Logicodex v1.30.0-alpha subsystem work from `sim/v130-resume` into `main`. The branch has been synchronized with `origin/main`, validated locally, and documented so reviewers can understand the change boundary before approving any merge.

The most important safety property is that v1.30 is **not activated as the default compiler pipeline**. The existing Logicodex v1.21-alpha command path remains the default path for normal `check`, `compile`, and related commands. The v1.30 work is introduced as a dormant, opt-in subsystem with the developer command `v130-check`, which first runs v1.21 semantic validation and only then probes the v1.30 subsystem.

> **Merge principle:** merge only through Pull Request review, after validation passes and reviewers confirm that the change remains dormant, non-destructive, and compatible with current Logicodex v1.21 alpha.

## Branch and Repository State

| Item | Value |
|---|---|
| Repository | `msa-studio/logicodex` |
| Default branch | `main` |
| Source branch | `sim/v130-resume` |
| Current v1.30 implementation commit | `7956a37` — `Implement dormant v1.30.0-alpha subsystem roadmap` |
| Relationship to `origin/main` before documentation commit | `0` commits behind, `1` commit ahead |
| Merge strategy recommended | Pull Request with squash merge after approval |

## Scope of the Change

The change introduces dormant architecture building blocks for the v1.30 line. It intentionally avoids replacing the current compiler path. The v1.30 implementation is designed to support future activation work after additional parser-to-HIR parity and executable-output parity checks.

| Area | Files | Purpose |
|---|---|---|
| CLI opt-in gate | `src/main.rs` | Adds `v130-check` without changing default v1.21-alpha commands. |
| Type system substrate | `src/types.rs` | Adds deterministic type IDs, type refs, interning, and equivalence checks. |
| Layout substrate | `src/layout.rs` | Adds natural and packed layout computation with deterministic diagnostics. |
| FFI substrate | `src/ffi.rs` | Adds callable registry, argument validation, and unsafe-boundary checks. |
| HIR substrate | `src/hir.rs` | Adds AST/HIR structures, lowering context, symbol/local mapping, and diagnostics. |
| Semantic gate | `src/semantic_gate.rs` | Adds opt-in HIR traversal checks for loop legality and unsafe FFI calls. |
| Codegen contract | `src/codegen_contract.rs` | Adds backend contract boundary and mock backend tests. |
| Diagnostic contract | `src/span.rs` | Adds span, spanned value, diagnostic severity, and bilingual diagnostic structure. |
| Architecture roadmap | `spec/v1.30.0-alpha/v130_architecture_design.md` | Documents the improved staged roadmap for dormant v1.30 work. |
| Readiness documentation | `docs/release/V130_MAIN_READINESS.md` | Provides this safe merge plan and validation record. |
| Validation automation | `scripts/validate_v130_main_readiness.sh` | Re-runs the readiness validation and stores the log under `target/v130-main-readiness/`. |

## Validation Record

The readiness validation was run after confirming that `sim/v130-resume` was up to date with `origin/main`. The validation log is stored at `target/v130-main-readiness/main_readiness_validation.log` in the local working tree and can be regenerated with `scripts/validate_v130_main_readiness.sh`.

| Validation Step | Result | Interpretation |
|---|---:|---|
| `git rebase origin/main` | Passed | Branch is up to date with `origin/main`; no conflict encountered. |
| `cargo fmt --all -- --check` | Passed | Formatting is consistent. |
| `cargo check --locked` | Passed | The crate compiles under the locked dependency set. |
| `cargo test --locked` | Passed | 27 unit tests and 4 integration tests passed. |
| `v130-check examples/hello.ldx` | Passed | v1.21 validation and v1.30 dormant subsystem probe passed. |
| `v130-check examples/matematik.ldx` | Passed | v1.21 validation and v1.30 dormant subsystem probe passed. |
| `v130-check examples/perkakasan.ldx` | Passed | v1.21 validation and v1.30 dormant subsystem probe passed. |
| Remaining `todo` markers in audited v1.30 modules | None | Runtime TODO placeholders in the audited v1.30 surface were removed. |

Two sample files, `examples/01_tambah_pakar.ldx` and `examples/01_tambah_pemula.ldx`, fail under the existing `check` command before the v1.30 subsystem is reached. Therefore, their `v130-check` failure is classified as **baseline v1.21-alpha example compatibility**, not a v1.30 regression. The validation script records this as a compatibility matrix rather than treating it as a blocker for the dormant subsystem merge.

## Pull Request Review Checklist

Reviewers should confirm that the branch preserves the current compiler behavior while making the v1.30 subsystem inspectable and testable. The checklist below should be completed before any merge into `main`.

| Review Area | Required Check | Status Before Merge |
|---|---|---:|
| Default compiler path | Confirm `check` and `compile` still use the v1.21-alpha pipeline. | Required |
| v1.30 activation risk | Confirm `v130-check` is opt-in and not invoked by default commands. | Required |
| Build artifacts | Confirm no `target/` logs or generated artifacts are committed. | Required |
| Secrets | Confirm no credentials, tokens, or environment values are committed. | Required |
| Tests | Confirm local validation and any GitHub CI pass. | Required |
| Documentation | Confirm this readiness document and roadmap accurately describe the dormant status. | Required |
| Example compatibility | Confirm known `01_tambah_*` failures are baseline parser compatibility, not v1.30 regressions. | Required |

## Recommended Merge Procedure

The safest procedure is to create a Pull Request from `sim/v130-resume` into `main`. The PR should use this document as the review reference and should not be merged until CI and manual review pass.

```bash
git checkout sim/v130-resume
git fetch origin
git rebase origin/main
scripts/validate_v130_main_readiness.sh
git push --force-with-lease
```

After the branch is current and validation is clean, open or update the Pull Request. If the repository uses branch protection, wait for all checks to complete. If no branch protection exists, treat this document as the manual gate and require explicit approval before merging.

```bash
gh pr create \
  --base main \
  --head sim/v130-resume \
  --title "Implement dormant v1.30.0-alpha subsystem roadmap" \
  --body-file target/v130-main-readiness/PR_BODY.md
```

For the final merge, the recommended method is **squash merge**. This keeps `main` history concise while preserving the full branch discussion in the PR. If the team prefers preserving the branch commit exactly, a normal merge commit is also acceptable, but direct push to `main` is not recommended.

## Rollback Plan

If the PR is merged and a regression is later discovered, revert the merge commit or squash commit from `main`. Because the v1.30 subsystem is dormant and opt-in, the expected operational blast radius is limited to build/test compatibility, not default compiler execution. The rollback command should be run only after identifying the merged commit hash on `main`.

```bash
git checkout main
git pull origin main
git revert <merged_commit_hash>
git push origin main
```

## Post-Merge Follow-Up

After merging into `main`, create a follow-up issue or milestone for the next v1.30 activation phase. That next phase should not immediately replace the compiler pipeline. It should first add parser-to-v1.30 AST bridge tests, HIR parity tests, and executable-output parity checks against current Logicodex v1.21 alpha behavior.

| Follow-Up Work | Rationale | Recommended Timing |
|---|---|---|
| Parser-to-v1.30 AST bridge | Connect real parsed programs to the dormant HIR path. | After dormant subsystem merge. |
| Parity tests against v1.21-alpha | Prove that accepted programs keep equivalent semantics. | Before any activation. |
| Example compatibility cleanup | Decide whether `01_tambah_*` files should be updated or parser behavior extended. | Before release notes. |
| Codegen integration plan | Map the HIR/codegen contract to the existing backend carefully. | After parity confidence improves. |
| Feature flag policy | Decide whether future v1.30 commands require cargo features, CLI flags, or environment gates. | Before broader testing. |

## Final Recommendation

This branch is ready for Pull Request review, not for direct push into `main`. The safe merge decision is to open the PR, attach the validation result, request review, and merge only after all checks pass. The branch should be described as **dormant v1.30.0-alpha subsystem groundwork for current Logicodex v1.21 alpha**, not as an active v1.30 compiler release.
