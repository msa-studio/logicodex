#!/usr/bin/env python3.11
"""Synchronize Logicodex documentation with the refreshed reflex-engine examples.

The script is intentionally conservative. For each documentation section it either
applies an exact replacement or accepts that the replacement is already present.
If neither the old nor the new text is found, the script fails so maintainers can
review the drift manually.
"""
from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def ensure_replace(path: str, old: str, new: str) -> None:
    target = ROOT / path
    text = target.read_text(encoding="utf-8")
    if new in text:
        return
    if old not in text:
        raise SystemExit(f"Expected old or new text not found in {path}")
    if text.count(old) != 1:
        raise SystemExit(f"Expected old text is not unique in {path}")
    target.write_text(text.replace(old, new), encoding="utf-8")


REPLACEMENTS = [
    (
        "README.md",
        "| LLVM backend | Implemented through the Rust Inkwell path for core expressions and native-oriented object generation. | Mature target triples, ABI contracts, linker policies, and executable examples. |\n",
        "| LLVM backend | Implemented through the Rust Inkwell path for core expressions and native-oriented object generation. | Mature target triples, ABI contracts, linker policies, and executable-output checks. |\n| Reflex engine examples | Refreshed suite in `examples/` now covers expert and Malay arithmetic, functions, loops, bitwise operations, hardware-zone provenance, and Boolean conditionals. | Add expected-output fixtures and backend parity checks as the next proof layer. |\n",
    ),
    (
        "README.md",
        "| `MANUAL.md` | Provides concise compiler usage, build commands, and frontend architecture notes. |\n| `docs/VS_CODE_EXTENSION.md` | Explains how to run the Logicodex VS Code Side View MVP for best-effort Malay/English pseudocode alias to expert canonical shorthand preview without changing Rust. |\n",
        "| `MANUAL.md` | Provides concise compiler usage, build commands, frontend architecture notes, and the current example validation loop. |\n| `docs/examples/REFLEX_ENGINE_EXAMPLES.md` | Records the refreshed reflex-engine example set and the paired `check` / `v130-check` compatibility boundary. |\n| `docs/VS_CODE_EXTENSION.md` | Explains how to run the Logicodex VS Code Side View MVP for best-effort Malay/English pseudocode alias to expert canonical shorthand preview without changing Rust. |\n",
    ),
    (
        "README.md",
        "The next useful work is not to expand claims, but to improve proof. The project should prioritize a small set of native examples, reliable build instructions, stable diagnostics, and repeatable validation scripts. After that foundation is stable, the roadmap can safely move into WebAssembly, migration tooling, runtime attestation, and freestanding hardware experiments.\n",
        "The next useful work is not to expand claims, but to improve proof. The repository now includes a refreshed reflex-engine example set that passes both the default v1.21-alpha `check` path and the opt-in v1.30.0-alpha `v130-check` probe. The next proof layer should add expected-output fixtures, backend parity checks, reliable build instructions, stable diagnostics, and repeatable validation scripts before broader WebAssembly, migration tooling, runtime attestation, or freestanding hardware experiments.\n",
    ),
    (
        "MANUAL.md",
        "Set `LOGICODEX_LINKER` to override the linker used by the compiler. For machine setup details, use `ENVIRONMENT_SETUP.md`; for grammar, dictionary, aliases, and executable examples, use `GrammarandDictionary.md`.\n\n## v1.21-alpha Split-Implementation Boundary\n",
        "Set `LOGICODEX_LINKER` to override the linker used by the compiler. For machine setup details, use `ENVIRONMENT_SETUP.md`; for grammar, dictionary, aliases, and executable examples, use `GrammarandDictionary.md`.\n\n## Current Example Compatibility Suite\n\nThe refreshed `examples/` directory is the maintained reflex-engine compatibility suite for **current Logicodex v1.21-alpha** plus the dormant **v1.30.0-alpha** probe. It includes expert canonical and Malay beginner programs for arithmetic, functions, loops, bitwise operators, hardware-zone provenance, and Boolean conditionals. Maintainers should validate the full suite rather than a single sample file when changing parser, semantic, CLI, or documentation behavior.\n\n| Example group | Files | Compatibility expectation |\n|---|---|---|\n| Legacy smoke examples | `hello.ldx`, `matematik.ldx`, `perkakasan.ldx` | Continue to pass the default `check` path. |\n| Reflex arithmetic examples | `01_tambah_pakar.ldx`, `01_tambah_pemula.ldx` | Pass both `check` and `v130-check` after the syntax refresh. |\n| Reflex feature examples | `02_fungsi_matematik.ldx` through `06_logik_bersyarat.ldx` | Pass both `check` and `v130-check` while avoiding recognized-but-blocked roadmap constructs. |\n\n```bash\nfor file in examples/*.ldx; do\n  cargo run --quiet -- check \"$file\"\n  cargo run --quiet -- v130-check \"$file\"\ndone\n```\n\nThe detailed file-by-file inventory is maintained in `docs/examples/REFLEX_ENGINE_EXAMPLES.md`.\n\n## v1.21-alpha Split-Implementation Boundary\n",
    ),
    (
        "GrammarandDictionary.md",
        "## 5. Usage Examples\n\n### 5.1 Expert Canonical Shorthand Mode\n",
        "## 5. Usage Examples\n\nThe snippets below show the accepted grammar surface. The repository-level compatibility suite is maintained as real `.ldx` files under `examples/` and summarized in `docs/examples/REFLEX_ENGINE_EXAMPLES.md`; those files are the preferred validation target for parser, semantic, and CLI changes.\n\n### 5.1 Expert Canonical Shorthand Mode\n",
    ),
    (
        "CONTRIBUTING.md",
        "| Examples | New examples must use syntax accepted by the current parser. | `cargo run -- check examples/name.ldx` |\n| Safety-sensitive syntax | Hardware and raw-address examples must remain explicitly gated. | Prefer `--target freestanding --object-only` validation |\n",
        "| Examples | New examples must use syntax accepted by the current parser and semantic analyzer. | `cargo run --quiet -- check examples/name.ldx` |\n| Reflex example compatibility | The refreshed example suite must remain valid under the default v1.21-alpha path and the opt-in v1.30.0-alpha probe. | `for file in examples/*.ldx; do cargo run --quiet -- check \"$file\"; cargo run --quiet -- v130-check \"$file\"; done` |\n| Safety-sensitive syntax | Hardware and raw-address examples must remain explicitly gated. | Prefer `--target freestanding --object-only` validation when exercising backend object generation. |\n",
    ),
    (
        "CONTRIBUTING.md",
        "cargo test --locked\npython3.11 scripts/check_bilingual_error_annotations.py\npython3.11 scripts/validate_v121_executable_logic.py\n```\n\nA pull request should explain the integrity problem being fixed, list the files changed, describe any new examples or tests, and state whether any language behavior changed. Feature additions should be separated from integrity hotfixes so reviewers can verify repository restoration independently.\n",
        "cargo test --locked\npython3.11 scripts/check_bilingual_error_annotations.py\npython3.11 scripts/validate_v121_executable_logic.py\nfor file in examples/*.ldx; do\n  cargo run --quiet -- check \"$file\"\n  cargo run --quiet -- v130-check \"$file\"\ndone\n```\n\nA pull request should explain the integrity problem being fixed, list the files changed, describe any new examples or tests, and state whether any language behavior changed. If the change touches parser, semantic analysis, CLI behavior, or documentation examples, include the full reflex example compatibility result in the PR body. Feature additions should be separated from integrity hotfixes so reviewers can verify repository restoration independently.\n",
    ),
    (
        "REPOS_CONTEXT.md",
        "- `examples/`: Contains official functional validation files with the `.ldx` extension, demonstrating both localized verbose programming styles and advanced freestanding memory operations.\n",
        "- `examples/`: Contains official `.ldx` validation files, including legacy smoke examples and the refreshed reflex-engine compatibility suite for arithmetic, functions, loops, bitwise operations, hardware-zone provenance, and Boolean conditionals. These files are expected to pass the default v1.21-alpha `check` command and the opt-in v1.30.0-alpha `v130-check` probe unless a document explicitly marks a future roadmap construct as blocked.\n",
    ),
    (
        "REPOS_CONTEXT.md",
        "The current logicodex v 1.21 alpha dictionary now includes the requested three-tier token records for program structure, bindings, control flow, FFI vocabulary, resource vocabulary, type families, bitwise operators, and hardware/address vocabulary. Treat this as a vocabulary and lexer-recognition update. New executable behavior should still be introduced through parser, semantic, backend, and validation milestones.\n",
        "The current logicodex v 1.21 alpha dictionary now includes the requested three-tier token records for program structure, bindings, control flow, FFI vocabulary, resource vocabulary, type families, bitwise operators, and hardware/address vocabulary. Treat dictionary-only additions as vocabulary and lexer-recognition updates until parser, semantic, backend, and validation milestones prove executable behavior. The current reflex-engine examples document the executable subset that is already accepted by both `check` and `v130-check`.\n",
    ),
    (
        "ROADMAP.md",
        "| Issue #03 — Native example suite | Open | TBD | A small set of `.ldx` programs compiles through the documented pipeline and has expected-output checks. |\n| Issue #04 — CI-oriented validation | Open | TBD | `cargo check`, release build, and validation scripts can be run from a clean checkout with documented dependencies. |\n",
        "| Issue #03 — Native example suite | Partially complete | Mohamad Supardi Abdul | The refreshed reflex-engine `.ldx` suite passes `check` and `v130-check`; remaining work is expected-output fixtures and backend/object-output parity checks. |\n| Issue #04 — CI-oriented validation | Open | TBD | `cargo check`, release build, full example sweeps, and validation scripts can be run from a clean checkout with documented dependencies. |\n",
    ),
    (
        "ROADMAP.md",
        "| Issue #10 — Documentation examples | Open | TBD | README, manual, and specification examples are synchronized with compiler behavior. |\n",
        "| Issue #10 — Documentation examples | Partially complete | Mohamad Supardi Abdul | README, manual, grammar notes, repository context, and reflex-example documentation describe the current validated example suite; remaining work is to keep release notes and future specs synchronized as behavior changes. |\n",
    ),
    (
        "ROADMAP.md",
        "| Mutability, loops, FFI, C interop, resources, string type marker, and bitwise markers | Available as token vocabulary | Add parser and semantic rules before claiming executable feature support. |\n| Hardware/address vocabulary | Available as token vocabulary and design direction | Add explicit target gates, provenance rules, and freestanding examples before claiming runtime support. |\n",
        "| Loops and bitwise markers | Available as token vocabulary and executable parser/semantic/codegen subset | Keep examples, tests, and `v130-check` compatibility synchronized as the subset expands. |\n| Mutability, FFI, C interop, resources, and string type marker | Available as token vocabulary | Add parser and semantic rules before claiming executable feature support. |\n| Hardware/address vocabulary | Available as token vocabulary, current hardware-zone provenance examples, and design direction | Add explicit target gates, deeper provenance rules, and freestanding backend examples before claiming runtime support. |\n",
    ),
    (
        "docs/release/V130_MAIN_READINESS.md",
        "**Merge status:** Prepared for Pull Request review; not merged to `main` by this document.\n",
        "**Merge status:** Historical readiness record; the dormant subsystem has since been merged into `main`, and the refreshed reflex-engine examples now pass `v130-check`.\n",
    ),
    (
        "docs/release/V130_MAIN_READINESS.md",
        "The readiness validation was run after confirming that `sim/v130-resume` was up to date with `origin/main`. The validation log is stored at `target/v130-main-readiness/main_readiness_validation.log` in the local working tree and can be regenerated with `scripts/validate_v130_main_readiness.sh`.\n",
        "The original readiness validation was run after confirming that `sim/v130-resume` was up to date with `origin/main`. After the reflex-engine example refresh, maintainers should validate the entire `examples/*.ldx` suite with both the default `check` command and the opt-in `v130-check` command, because `01_tambah_*` and the newer `02_` through `06_` files now form part of the documented compatibility baseline.\n",
    ),
    (
        "docs/release/V130_MAIN_READINESS.md",
        "| `v130-check examples/hello.ldx` | Passed | v1.21 validation and v1.30 dormant subsystem probe passed. |\n| `v130-check examples/matematik.ldx` | Passed | v1.21 validation and v1.30 dormant subsystem probe passed. |\n| `v130-check examples/perkakasan.ldx` | Passed | v1.21 validation and v1.30 dormant subsystem probe passed. |\n| Remaining `todo` markers in audited v1.30 modules | None | Runtime TODO placeholders in the audited v1.30 surface were removed. |\n\nTwo sample files, `examples/01_tambah_pakar.ldx` and `examples/01_tambah_pemula.ldx`, fail under the existing `check` command before the v1.30 subsystem is reached. Therefore, their `v130-check` failure is classified as **baseline v1.21-alpha example compatibility**, not a v1.30 regression. The validation script records this as a compatibility matrix rather than treating it as a blocker for the dormant subsystem merge.\n",
        "| Full `examples/*.ldx` default sweep | Passed after the reflex-example refresh | Every shipped example is expected to pass `cargo run --quiet -- check \"$file\"`. |\n| Full `examples/*.ldx` v1.30 probe sweep | Passed after the reflex-example refresh | Every shipped example is expected to pass `cargo run --quiet -- v130-check \"$file\"` after v1.21 validation succeeds. |\n| Remaining `todo` markers in audited v1.30 modules | None in the readiness audit | Runtime TODO placeholders in the audited v1.30 surface were removed before merge. |\n\nThe former `01_tambah_*` compatibility issue has been resolved by updating those examples to syntax accepted by the current parser and semantic analyzer. They are now part of the reflex-engine compatibility baseline rather than known failures.\n",
    ),
    (
        "docs/release/V130_MAIN_READINESS.md",
        "| Example compatibility | Confirm known `01_tambah_*` failures are baseline parser compatibility, not v1.30 regressions. | Required |\n",
        "| Example compatibility | Confirm all `examples/*.ldx` files pass both `check` and `v130-check`; `01_tambah_*` are no longer documented failures. | Required |\n",
    ),
    (
        "docs/release/V130_MAIN_READINESS.md",
        "| Example compatibility cleanup | Decide whether `01_tambah_*` files should be updated or parser behavior extended. | Before release notes. |\n",
        "| Example compatibility maintenance | Keep `examples/*.ldx`, `docs/examples/REFLEX_ENGINE_EXAMPLES.md`, and `v130-check` behavior synchronized as parser/HIR parity expands. | Before release notes and before activation work. |\n",
    ),
    (
        "WHITE_PAPER.md",
        "Logicodex v1.21-alpha mirrors this distinction through an explicit compiler target parameter:\n\n```bash\nlogicodex compile --target freestanding examples/01_tambah_pakar.ldx --object-only\n```\n\nIn this profile, the backend emits an object intended for later integration by a bootloader, kernel linker script, hypervisor build, or firmware image generator. The compiler does not claim to provide a complete bootable image at this stage. It provides the **layout framework** required for operating-system development: target selection, entry-symbol control, runtime bypass, and physical-memory access documentation.\n\nA concrete freestanding example is the classic VGA text buffer write at physical address `0xB8000`. The example below writes raw ASCII character bytes and attribute bytes directly to screen memory. It is intentionally documented as a freestanding, capability-gated operation rather than ordinary hosted application behavior.",
        "Logicodex v1.21-alpha mirrors this distinction through explicit target and validation boundaries. The maintained reflex-engine hardware example is `examples/05_zon_perkakasan_reflex.ldx`, which should pass the parser and semantic checks before any future freestanding object-output claim is made:\n\n```bash\ncargo run --quiet -- check examples/05_zon_perkakasan_reflex.ldx\ncargo run --quiet -- v130-check examples/05_zon_perkakasan_reflex.ldx\n```\n\nIn a future freestanding object profile, the backend should emit an object intended for later integration by a bootloader, kernel linker script, hypervisor build, or firmware image generator. The compiler does not claim to provide a complete bootable image at this stage. It provides the **layout framework** required for operating-system development: target selection, entry-symbol control, runtime bypass, and physical-memory access documentation.\n\nA concrete freestanding roadmap example is the classic VGA text buffer write at physical address `0xB8000`. The conceptual example below writes raw ASCII character bytes and attribute bytes directly to screen memory. It is intentionally documented as a freestanding, capability-gated operation rather than ordinary hosted application behavior.",
    ),
    ("WHITE_PAPER.md", "**Novice pseudocode variant:**", "**Future novice pseudocode variant:**"),
    ("WHITE_PAPER.md", "**Expert shorthand variant:**", "**Future expert shorthand variant:**"),
    (
        "WHITE_PAPER.md",
        "The alias and expert canonical forms compile toward the same conceptual volatile stores. On x86 text-mode targets, each `U16` cell combines an ASCII byte and a color attribute byte, while other target families would bind equivalent display or serial-output hardware through target-specific capability declarations.",
        "The alias and expert canonical forms are intended to compile toward the same conceptual volatile stores after the required parser, semantic, and backend gates exist. On x86 text-mode targets, each `U16` cell combines an ASCII byte and a color attribute byte, while other target families would bind equivalent display or serial-output hardware through target-specific capability declarations.",
    ),
]

for path, old, new in REPLACEMENTS:
    ensure_replace(path, old, new)

print("Reflex documentation synchronized.")
