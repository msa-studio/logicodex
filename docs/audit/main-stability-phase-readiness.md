# Stability & Phase-Readiness Audit -- `main`

Audit type: baseline stability + debt audit of `main`
Target: `main` @ `0.46.0-alpha` (2026-06-14), reported green CI for 14 days
Purpose: certify `main` as a clean, stable base for the next phase (the
`feature/stdlib-core` foundation merge).
Method: offline snapshot inspection and static analysis. The Rust toolchain and
LLVM are not available in the audit environment; CI history cannot be replayed
from a snapshot. Claims about test/build/clippy/fmt outcomes are therefore
stated as "CI-guarded", and the 14-day green-CI claim is treated as reported,
not independently re-run here (see section 2).

## 1. Verdict

`main` is a **clean, stable, honestly-documented base** and is ready to serve as
the foundation for the next phase. Its tracked deferred backlog is essentially
cleared (25/26 done, 1 by design), its debt markers are either documented
limitations or invariant guards, and its CI guards the real surface
(compile + full test + freestanding boot). The few incomplete subsystems are
staged and unreachable on current paths, not latent bugs. No blocking debt was
found.

## 2. CI and the green-CI claim

CI (`.github/workflows/ci.yml`) runs three jobs on Rust 1.75.0 + LLVM 15:

- `check` -- `cargo fmt --all -- --check`, `cargo clippy --all-targets`,
  `cargo check --all-targets`.
- `test` -- `cargo build --release` + full `cargo test` (includes the
  shipped-examples semantic-check gate); test log uploaded as an artifact.
- `freestanding` -- `make boot-evidence` (multiboot -> long-mode -> serial
  "Logicodex" -> clean exit 33), `make boot-e2e` (compiles
  `examples/freestanding/showcase.ldx`, links into the kernel, boots in QEMU,
  asserts the program's own output), and the live freestanding validator.

This is a meaningful, drift-resistant gate set: it builds the compiler, runs the
whole suite, and proves an end-to-end `.ldx` -> bootable kernel path. The
"14-day green" status is consistent with the snapshot (no half-applied changes,
no obviously broken state), but cannot be independently re-derived from files
alone; it should be taken from the CI dashboard as the system of record.

## 3. Test inventory

93 `#[test]` functions across 6 suites:

| Suite | Tests |
|---|---|
| `tests/freestanding_v144_gaps.rs` | 32 |
| `tests/capability_fabric.rs` | 24 |
| `tests/shard_topology.rs` | 20 |
| `tests/wasm_codegen.rs` | 8 |
| `tests/e2e_pipeline.rs` | 5 |
| `tests/lexer_parser_test.rs` | 4 |

Ignored tests: 3, all in `freestanding_v144_gaps.rs`, all documented crt0
deferrals (issue #3): the live kernel uses a multiboot `_start` in
`freestanding/` while `src/os/startup.rs` is dormant. These are intentional,
clearly-annotated, and not hidden failures.

## 4. Debt audit

Debt markers in `src`: 11 `TODO`, 11 `panic!(`, no `unimplemented!`/`todo!`.

TODOs -- documented, non-blocking limitations:

- `src/semantic/type_checker.rs` (7) -- a partial type checker with explicit
  notes (no `FloatLiteral`/`Unary` AST variants yet, argument-type inference and
  CallableRegistry integration deferred). It is not on a path that fails current
  tests.
- `src/codegen.rs:1617` -- inkwell 0.4.0 limitation (`StructType::set_name`
  unavailable); name carried via the symbol table instead.
- `src/types.rs` (2) -- reverse/callable lookup not implemented.
- `src/tier2/ctl_mapper.rs:456` -- host-side logic generation placeholder string.

`panic!` -- almost all are legitimate invariant guards ("this should never
happen"): `unwrap_struct`/`unwrap_enum` on the wrong `TypeKind`, invalid
`TypeId` (BUG), unregistered struct/enum layout, unexpected HIR item,
`const_char_ptr` not pre-interned. `src/os/panic.rs` is the bare-metal panic
handler (by design).

Two `panic!`s are deferred-feature markers, not invariant guards:

- `src/types.rs:411` -- `get_size` for `Enum` "not yet implemented (Sprint 2.5)".
- `src/types.rs:453` -- `get_align` for `Enum` "not yet implemented (Sprint 2.5)".

These are reachable only if enum *layout* (tagged-union size/alignment) is
computed. Current code represents enums as scalar `i64` tags and never routes
through enum layout, so the path is unreached and CI stays green. This is a
tracked deferral, not a current-functionality defect, but it is the one real
debt item to keep visible: a real tagged-union layout will require implementing
these two arms.

## 5. Deferred-work backlog (`DEFERRED.md`)

`DEFERRED.md` reports **25/26 deferred items complete, 1 by design (H1, edition
routing)**. Categories A-I (codegen, network runtime, sharded runtime, WASM
codegen, host reactor, etc.) are marked done with commit references. This is
strong evidence that `main` is not carrying a hidden backlog: the deferred
ledger is essentially closed.

## 6. Staged / known-incomplete subsystems

These are intentionally-staged states, documented and consistent with a green
base; none is a defect on `main`'s tested surface:

- **Actor runtime.** `src/codegen.rs` declares the runtime ABI
  (`logicodex_spawn`, `logicodex_join`, `logicodex_channel_*`) as externs, but
  `main` ships no `runtime_actor.c` and neither `Makefile` nor `build.rs`
  compiles a C actor backend. So `main`'s actor support is codegen-ready but
  not runtime-linked; actor programs are not part of `main`'s tested, linked
  surface. The C runtime backend is completed on the incoming
  `feature/stdlib-core` branch.
- **Enum layout** -- deferred (Sprint 2.5), as in section 4.
- **Type checker** -- partial, as in section 4.

## 7. `lib/core` status

`lib/core` on `main` contains only the 11 pre-Stage-0 legacy modules
(`capability`, `file`, `gate`, `io_error`, `memori`, `result`, `ring_buffer`,
`scheduler`, `shard_manifest`, `sync`, `thread`). There is no `tools/`, no
`scripts/dev/`, and zero `.std.toml` contracts: `main` has no contract-backed
stdlib yet. These legacy files are source material, not trusted modules, and
must not be read as a working standard library. The contract-backed stdlib and
its verifier are introduced by the incoming branch.

## 8. Documentation honesty

The `0.46.0-alpha` changelog entry is accurate and self-aware: it records the
honest `1.x -> 0.x` version realignment (pre-1.0 alpha), the end-to-end
`.ldx -> kernel` boot path, the single-engine (HIR) cleanup that removed the
`v1_30` feature flag and dead non-HIR arms, the archival of 26 pre-HIR
validators, and concrete fixes. No overclaiming was observed in the entry. (The
changelog's older actor/channel mentions describe historical work; note that the
actor *runtime* is completed on the incoming branch, per section 6.)

## 9. Stability & phase-up assessment

| Dimension | State |
|---|---|
| Build/test gate | CI-guarded (3 jobs), reported green |
| Test suite | 93 tests; 3 documented ignores |
| Deferred backlog | 25/26 done, 1 by design |
| Debt markers | documented limitations + invariant guards |
| Latent defects on tested paths | none found |
| Known deferrals | enum layout, type checker, actor runtime (staged) |
| Documentation | honest, matches code |

`main` is stable and clean enough to anchor the next phase. The recommended
phase-up sequence is: (1) treat `main` @ `0.46.0-alpha` as the certified base;
(2) merge `feature/stdlib-core` per its separate merge-readiness audit; (3) the
merge commit's green CI becomes the new phase baseline.

## 10. Conclusion

No blocking debt exists on `main`. The only debt worth tracking forward is the
deferred enum tagged-union layout (`types.rs` size/align panics, Sprint 2.5),
which is currently unreachable. `main` is certified stable and ready to serve as
the base for the `feature/stdlib-core` foundation merge and the next phase.

---

Companion document: `docs/audit/stdlib-core-to-main-merge-readiness.md`
(branch merge-safety audit). The CHANGELOG/documentation debt identified there
must be resolved before the merge that opens the new phase.
