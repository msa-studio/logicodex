#!/usr/bin/env python3
"""Self-tests for version-reference hygiene validation."""

from __future__ import annotations

import os
from pathlib import Path
import subprocess
import tempfile

from verify_version_reference_hygiene import (
    RELEASE_WORKFLOW,
    ValidationError,
    validate_repository,
)
from version_reference_hygiene_runtime import (
    APPROVED_LLVM_MAJOR,
    APPROVED_RUST_MINOR,
    RETIRED_LLVM_MAJOR,
    RETIRED_RUST_MINOR,
    TOOLCHAIN_HISTORICAL_EXCEPTIONS,
)


REPO_ROOT = Path(__file__).resolve().parents[2]

# Retired tokens are assembled at runtime so this file never self-matches.
RETIRED_LLVM_LINE = (
    "sudo apt-get install llvm-" + RETIRED_LLVM_MAJOR + "-dev\n"
)
RETIRED_RUST_LINE = (
    'toolchain: "' + RETIRED_RUST_MINOR + '.0"\n'
)


def write(
    root: Path,
    relative: str,
    text: str,
) -> None:
    path = root / relative
    path.parent.mkdir(
        parents=True,
        exist_ok=True,
    )
    path.write_text(
        text,
        encoding="utf-8",
    )


def git(root: Path, *args: str) -> str:
    env = {
        key: value
        for key, value in os.environ.items()
        if not key.startswith("GIT_")
    }

    return subprocess.run(
        [
            "git",
            "-C",
            str(root),
            "-c",
            "user.name=hygiene-fixture",
            "-c",
            "user.email=hygiene-fixture@example.invalid",
            "-c",
            "commit.gpgsign=false",
            *args,
        ],
        check=True,
        capture_output=True,
        env=env,
        text=True,
    ).stdout.strip()


def commit_fixture(root: Path, message: str) -> str:
    git(root, "add", "-A")
    git(root, "commit", "-q", "--allow-empty", "-m", message)
    return git(root, "rev-parse", "HEAD")


def create_fixture(root: Path) -> None:
    version = "0.46.0-alpha"

    write(
        root,
        "Cargo.toml",
        '[package]\n'
        'name = "logicodex"\n'
        f'version = "{version}"\n',
    )

    write(
        root,
        ".github/SECURITY.md",
        "| Version | Status |\n"
        "|---|---|\n"
        f"| v{version} (current Cargo release authority) "
        "| ✅ Current — best-effort alpha security support |\n",
    )

    for relative, classification, description in (
        (
            "docs/archive/.zone-status.md",
            "HistoricalProvenance",
            "This directory is historical and non-authoritative.\n",
        ),
        (
            "scripts/_archive/.zone-status.md",
            "ArchivedZone",
            "This directory is not part of the active validation path.\n",
        ),
        (
            "spec/v1.11-alpha/.zone-status.md",
            "HistoricalProvenance",
            "This specification is historical and non-authoritative.\n",
        ),
        (
            "spec/v1.21-alpha/.zone-status.md",
            "HistoricalProvenance",
            "This specification is historical and non-authoritative.\n",
        ),
        (
            "spec/v1.30.0-alpha/.zone-status.md",
            "HistoricalProvenance",
            "This specification is historical and non-authoritative.\n",
        ),
    ):
        write(
            root,
            relative,
            f"Classification: `{classification}`\n\n"
            + description,
        )

    write(
        root,
        "src/main.rs",
        'env!("CARGO_PKG_VERSION");\n'
        'env!("CARGO_PKG_VERSION");\n'
        'env!("CARGO_PKG_VERSION");\n'
        'default_value = "v1.30"\n'
        'name = "v130-check"\n'
        "CompilerPipeline::V130\n",
    )

    write(
        root,
        "docs/VS_CODE_EXTENSION.md",
        f"current Logicodex v{version}\n",
    )

    write(
        root,
        "docs/examples/REFLEX_ENGINE_EXAMPLES.md",
        "current single HIR compiler path; "
        "v1.21 is a deprecated compatibility alias\n",
    )

    write(
        root,
        "docs/guide/src/title.md",
        "**Historical guide baseline:** v1.45.0-alpha "
        "(non-authoritative)\n",
    )

    write(
        root,
        "docs/wiki/README.md",
        f"Logicodex Documentation — v{version}\n",
    )

    write(
        root,
        "extensions/vscode-logicodex/README.md",
        f"current Logicodex v{version}\n",
    )

    write(
        root,
        "extensions/vscode-logicodex/package.json",
        '{"description":"current Logicodex '
        f'v{version}"}}\n',
    )

    write(
        root,
        "extensions/vscode-logicodex/"
        "src/previewNormalizer.ts",
        "current single HIR compiler path "
        "is authoritative\n",
    )

    for hook in (
        "scripts/dev/verify_quick_integrity.sh",
        "scripts/dev/verify_full_integrity.sh",
    ):
        write(
            root,
            hook,
            "python3 -B scripts/dev/"
            "test_version_reference_hygiene.py\n"
            'env BASE_SHA="${BASE_SHA:-}" \\\n'
            "python3 -B scripts/dev/"
            "verify_version_reference_hygiene.py\n",
        )

    write(
        root,
        ".github/workflows/ci.yml",
        "\n".join(
            f"grep -Fq '{script}' {hook}"
            for hook in (
                "scripts/dev/verify_quick_integrity.sh",
                "scripts/dev/verify_full_integrity.sh",
            )
            for script in (
                "test_version_reference_hygiene.py",
                "verify_version_reference_hygiene.py",
            )
        )
        + "\n"
        + "".join(
            f"grep -Fq 'BASE_SHA=\"${{BASE_SHA:-}}\"' {hook}\n"
            for hook in (
                "scripts/dev/verify_quick_integrity.sh",
                "scripts/dev/verify_full_integrity.sh",
            )
        )
        + "python3 -B scripts/dev/test_version_reference_hygiene.py\n"
        + "python3 -B scripts/dev/verify_version_reference_hygiene.py\n",
    )

    write(
        root,
        RELEASE_WORKFLOW,
        (REPO_ROOT / RELEASE_WORKFLOW).read_text(
            encoding="utf-8",
        ),
    )

    for relative, lines in (
        TOOLCHAIN_HISTORICAL_EXCEPTIONS.items()
    ):
        write(
            root,
            relative,
            "Historical record.\n"
            + "".join(line + "\n" for line in lines),
        )

    create_runtime_fixture(root)

    git(root, "init", "-q")
    commit_fixture(root, "fixture")


def create_runtime_fixture(root: Path) -> None:
    standard_surfaces = (
        "src/codegen.rs",
        "src/net/affinity.rs",
        "src/net/connection.rs",
        "src/net/reactor.rs",
        "src/net/sharded_reactor.rs",
        "src/os/syscall.rs",
        "src/semantic_gate.rs",
        "src/tier2/pass.rs",
    )

    for relative in standard_surfaces:
        write(
            root,
            relative,
            "// Historical milestone v1.39 is allowed.\n"
            'eprintln!("logicodex: active runtime text");\n',
        )

    write(
        root,
        "src/tier2/capability_ir.rs",
        "// Historical milestone v1.35 is allowed.\n"
        'env!("CARGO_PKG_VERSION");\n'
        'env!("CARGO_PKG_VERSION");\n',
    )

    write(
        root,
        "src/tier2/ctl_mapper.rs",
        "// Historical milestone v1.36 is allowed.\n"
        'env!("CARGO_PKG_VERSION");\n'
        'env!("CARGO_PKG_VERSION");\n',
    )

    write(
        root,
        "lib/linker_scripts/x86_64-freestanding.ld",
        "/* Logicodex — Linker Script "
        "for Freestanding x86_64 */\n",
    )


def expect_failure(
    name: str,
    root: Path,
    expected: str,
    base_sha: str | None = None,
) -> None:
    try:
        validate_repository(root, base_sha)
    except ValidationError as error:
        message = str(error)

        if expected not in message:
            raise RuntimeError(
                f"{name}: expected {expected!r}, "
                f"got {message!r}"
            ) from error

        print(f"self_test_{name}=PASS")
        return

    raise RuntimeError(
        f"{name}: validation unexpectedly passed"
    )


def run_self_test() -> None:
    with tempfile.TemporaryDirectory(
        prefix="logicodex-version-hygiene-",
    ) as temp:
        root = Path(temp)
        create_fixture(root)

        summary = validate_repository(root)

        if summary.cargo_version != "0.46.0-alpha":
            raise RuntimeError(
                "valid fixture Cargo version mismatch"
            )

        print("self_test_valid_fixture=PASS")

        security = root / ".github/SECURITY.md"
        valid_security = security.read_text(
            encoding="utf-8",
        )

        security.write_text(
            valid_security
            + "current Logicodex v1.21-alpha\n",
            encoding="utf-8",
        )

        expect_failure(
            "stale_current_claim",
            root,
            "stale active version claim",
        )

        security.write_text(
            valid_security,
            encoding="utf-8",
        )

        main = root / "src/main.rs"
        valid_main = main.read_text(
            encoding="utf-8",
        )

        main.write_text(
            valid_main
            + 'version = "1.30.0-alpha"\n',
            encoding="utf-8",
        )

        expect_failure(
            "hardcoded_cli_version",
            root,
            "stale active version claim",
        )

        main.write_text(
            valid_main.replace(
                'default_value = "v1.30"\n',
                "",
            ),
            encoding="utf-8",
        )

        expect_failure(
            "compatibility_selector_removed",
            root,
            "compatibility marker missing",
        )

        main.write_text(
            valid_main,
            encoding="utf-8",
        )

        guide = root / "docs/guide/src/title.md"

        guide.write_text(
            "**Historical guide baseline:** "
            "v1.45.0-alpha (non-authoritative)\n",
            encoding="utf-8",
        )

        validate_repository(root)
        print(
            "self_test_historical_reference_allowed=PASS"
        )

        archive_status = (
            root / "docs/archive/.zone-status.md"
        )
        valid_archive_status = archive_status.read_text(
            encoding="utf-8",
        )
        archive_status.unlink()

        expect_failure(
            "historical_zone_marker_missing",
            root,
            "historical zone marker missing",
        )

        archive_status.write_text(
            valid_archive_status.replace(
                "HistoricalProvenance",
                "CurrentAuthority",
            ),
            encoding="utf-8",
        )

        expect_failure(
            "historical_zone_classification_mismatch",
            root,
            "historical zone classification mismatch",
        )

    print(
        "version_reference_hygiene_self_test=PASS"
    )


def run_runtime_self_test() -> None:
    with tempfile.TemporaryDirectory(
        prefix="logicodex-runtime-version-hygiene-",
    ) as temp:
        root = Path(temp)
        create_fixture(root)

        validate_repository(root)
        print(
            "self_test_runtime_valid_fixture=PASS"
        )

        reactor = root / "src/net/reactor.rs"
        valid_reactor = reactor.read_text(
            encoding="utf-8",
        )

        reactor.write_text(
            valid_reactor
            + 'eprintln!("logicodex v1.39: stale");\n',
            encoding="utf-8",
        )

        expect_failure(
            "stale_runtime_label",
            root,
            "stale active runtime version label",
        )

        reactor.write_text(
            valid_reactor
            + "// Historical runtime milestone v1.39.\n",
            encoding="utf-8",
        )

        validate_repository(root)
        print(
            "self_test_runtime_historical_comment_allowed="
            "PASS"
        )

        reactor.write_text(
            valid_reactor,
            encoding="utf-8",
        )

        capability = (
            root / "src/tier2/capability_ir.rs"
        )

        valid_capability = capability.read_text(
            encoding="utf-8",
        )

        capability.write_text(
            valid_capability.replace(
                'env!("CARGO_PKG_VERSION")',
                '"v1.35.0-alpha"',
                1,
            ),
            encoding="utf-8",
        )

        expect_failure(
            "generated_header_authority",
            root,
            "generated headers must derive from "
            "CARGO_PKG_VERSION",
        )

        capability.write_text(
            valid_capability,
            encoding="utf-8",
        )

        linker = (
            root
            / "lib/linker_scripts/"
            "x86_64-freestanding.ld"
        )

        valid_linker = linker.read_text(
            encoding="utf-8",
        )

        linker.write_text(
            valid_linker.replace(
                "Logicodex — Linker Script",
                "Logicodex v1.44 — Linker Script",
            ),
            encoding="utf-8",
        )

        expect_failure(
            "stale_linker_header",
            root,
            "linker header must be version-agnostic",
        )

    print(
        "runtime_version_reference_hygiene_self_test=PASS"
    )


def run_toolchain_self_test() -> None:
    with tempfile.TemporaryDirectory(
        prefix="logicodex-toolchain-hygiene-",
    ) as temp:
        root = Path(temp)
        create_fixture(root)

        validate_repository(root)
        print("self_test_toolchain_valid_fixture=PASS")

        doc = root / "docs/VS_CODE_EXTENSION.md"
        valid_doc = doc.read_text(encoding="utf-8")

        doc.write_text(
            valid_doc + RETIRED_LLVM_LINE,
            encoding="utf-8",
        )

        expect_failure(
            "toolchain_retired_llvm_active_line",
            root,
            "docs/VS_CODE_EXTENSION.md:2: non-approved toolchain reference",
        )

        doc.write_text(
            valid_doc + RETIRED_RUST_LINE,
            encoding="utf-8",
        )

        expect_failure(
            "toolchain_retired_rust_contextual",
            root,
            "non-approved toolchain reference (worktree)",
        )

        doc.write_text(
            valid_doc
            + "ratio = " + RETIRED_RUST_MINOR + "\n"
            + "LLVM" + RETIRED_LLVM_MAJOR + "_DATA_LAYOUT\n",
            encoding="utf-8",
        )

        validate_repository(root)
        print("self_test_toolchain_bare_number_allowed=PASS")

        doc.write_text(
            valid_doc
            + "sudo apt-get install llvm-" + APPROVED_LLVM_MAJOR + "-dev\n"
            + "LLVM_SYS_" + APPROVED_LLVM_MAJOR + "1_PREFIX\n"
            + "Requires Rust " + APPROVED_RUST_MINOR + ".0 and LLVM "
            + APPROVED_LLVM_MAJOR + ".1.6\n"
            + "cargo +" + APPROVED_RUST_MINOR + ".0 build\n",
            encoding="utf-8",
        )

        validate_repository(root)
        print("self_test_toolchain_approved_versions_allowed=PASS")

        # Any non-approved LLVM major or Rust 1.x release is rejected,
        # not only the retired toolchain line.
        for name, line in (
            ("llvm_older_major", "Install LLVM " + "1" + "7 first\n"),
            ("llvm_newer_major", "brew install llvm@" + "2" + "4\n"),
            ("llvm_sys_prefix", "LLVM_SYS_" + "18" + "1_PREFIX=/opt\n"),
            ("clang_package", "apt-get install libclang-" + "1" + "8-dev\n"),
            ("rust_older_minor", "Requires Rust " + "1." + "78.0\n"),
            ("rust_newer_minor", "rustup default " + "1." + "101.0\n"),
            ("rust_channel", 'channel = "' + "1." + '80"\n'),
        ):
            doc.write_text(valid_doc + line, encoding="utf-8")

            expect_failure(
                f"toolchain_policy_{name}",
                root,
                "docs/VS_CODE_EXTENSION.md:2: non-approved toolchain "
                "reference (worktree)",
            )

        doc.write_text(valid_doc, encoding="utf-8")

        doc.write_text(
            valid_doc + RETIRED_LLVM_LINE,
            encoding="utf-8",
        )
        git(root, "add", "docs/VS_CODE_EXTENSION.md")
        doc.write_text(valid_doc, encoding="utf-8")

        expect_failure(
            "toolchain_staged_only_bytes",
            root,
            "non-approved toolchain reference (staged)",
        )

        git(root, "checkout", "HEAD", "--", "docs/VS_CODE_EXTENSION.md")
        validate_repository(root)

        binary = root / "assets/blob.bin"
        binary.parent.mkdir(parents=True, exist_ok=True)
        binary.write_bytes(
            b"\xff\xfe\x00" + RETIRED_LLVM_LINE.encode() + b"\x80"
        )

        expect_failure(
            "toolchain_untracked_raw_bytes",
            root,
            "assets/blob.bin:1: non-approved toolchain reference",
        )

        binary.unlink()

        archive = root / "docs/archive/OLD_SETUP.md"
        archive.write_text(RETIRED_LLVM_LINE, encoding="utf-8")

        expect_failure(
            "toolchain_untracked_zone_not_exempt",
            root,
            "docs/archive/OLD_SETUP.md:1",
        )

        base = git(root, "rev-parse", "HEAD")
        commit_fixture(root, "archive provenance")
        validate_repository(root)
        print("self_test_toolchain_tracked_zone_allowed=PASS")

        expect_failure(
            "toolchain_base_sha_zone_addition",
            root,
            "docs/archive/OLD_SETUP.md:1: non-approved toolchain "
            "reference added since BASE_SHA",
            base,
        )

        head = git(root, "rev-parse", "HEAD")
        summary = validate_repository(root, head)

        if summary.toolchain_base_sha != head:
            raise RuntimeError("base_sha summary mismatch")

        print("self_test_toolchain_base_sha_clean=PASS")

        for name, base_sha, expected in (
            ("malformed", "HEAD", "not a full lowercase object id"),
            ("unknown", "0" * 40, "does not resolve to a commit"),
        ):
            expect_failure(
                f"toolchain_base_sha_{name}",
                root,
                expected,
                base_sha,
            )

        # Historical-zone index/HEAD bypass regressions. The zone file
        # is committed provenance at `head`; new references must not
        # hide in the index or HEAD behind a clean worktree copy.
        zone_relative = "docs/archive/OLD_SETUP.md"
        zone = root / zone_relative
        clean_zone = zone.read_text(encoding="utf-8")
        added_zone_line = (
            "brew install llvm@" + RETIRED_LLVM_MAJOR + "\n"
        )

        zone.write_text(clean_zone + added_zone_line, encoding="utf-8")

        expect_failure(
            "toolchain_zone_worktree_edit",
            root,
            f"{zone_relative}:2: non-approved toolchain reference added "
            "to historical zone since HEAD (worktree)",
        )

        git(root, "add", zone_relative)
        zone.write_text(clean_zone, encoding="utf-8")

        expect_failure(
            "toolchain_zone_staged_hidden_by_worktree",
            root,
            f"{zone_relative}:2: non-approved toolchain reference added "
            "to historical zone since HEAD (staged)",
        )
        expect_failure(
            "toolchain_zone_staged_hidden_base_sha",
            root,
            f"{zone_relative}:2: non-approved toolchain reference added "
            "since BASE_SHA (staged)",
            head,
        )

        git(root, "reset", "-q", "--", zone_relative)
        validate_repository(root)
        validate_repository(root, head)

        zone.write_text(clean_zone + added_zone_line, encoding="utf-8")
        commit_fixture(root, "zone addition")
        zone.write_text(clean_zone, encoding="utf-8")

        # Without BASE_SHA, HEAD is the zone baseline and a clean
        # worktree copy adds nothing; BASE_SHA must still see HEAD.
        validate_repository(root)
        print("self_test_toolchain_zone_head_baseline=PASS")

        expect_failure(
            "toolchain_zone_committed_hidden_base_sha",
            root,
            f"{zone_relative}:2: non-approved toolchain reference added "
            "since BASE_SHA (HEAD)",
            head,
        )

        git(root, "add", zone_relative)

        expect_failure(
            "toolchain_zone_committed_hidden_staged_restore_base_sha",
            root,
            f"{zone_relative}:2: non-approved toolchain reference added "
            "since BASE_SHA (HEAD)",
            head,
        )

        commit_fixture(root, "zone restore")
        validate_repository(root, head)
        print("self_test_toolchain_zone_restored_base_sha_clean=PASS")

        gaps = root / "docs/GAPS_ASSESSMENT.md"
        valid_gaps = gaps.read_text(encoding="utf-8")
        first_exception = TOOLCHAIN_HISTORICAL_EXCEPTIONS[
            "docs/GAPS_ASSESSMENT.md"
        ][0]

        gaps.write_text(
            valid_gaps + first_exception + "\n",
            encoding="utf-8",
        )

        expect_failure(
            "toolchain_exception_used_twice",
            root,
            "docs/GAPS_ASSESSMENT.md:6: non-approved toolchain reference",
        )

        gaps.write_text(
            valid_gaps.replace(first_exception + "\n", ""),
            encoding="utf-8",
        )

        expect_failure(
            "toolchain_stale_exception",
            root,
            "stale historical toolchain exception",
        )

        gaps.write_text(
            valid_gaps.replace(
                first_exception,
                first_exception + " (edited)",
            ),
            encoding="utf-8",
        )

        expect_failure(
            "toolchain_exception_inexact",
            root,
            "docs/GAPS_ASSESSMENT.md:2: non-approved toolchain reference",
        )

        gaps.write_text(valid_gaps, encoding="utf-8")
        validate_repository(root)

    with tempfile.TemporaryDirectory(
        prefix="logicodex-toolchain-nogit-",
    ) as temp:
        root = Path(temp)
        create_fixture(root)
        subprocess.run(
            ["rm", "-rf", str(root / ".git")],
            check=True,
        )

        expect_failure(
            "toolchain_requires_git_work_tree",
            root,
            "requires a git work tree",
        )

    print("toolchain_reference_hygiene_self_test=PASS")


def run_release_workflow_self_test() -> None:
    with tempfile.TemporaryDirectory(
        prefix="logicodex-release-workflow-",
    ) as temp:
        root = Path(temp)
        create_fixture(root)

        validate_repository(root)
        print("self_test_release_valid_fixture=PASS")

        workflow = root / RELEASE_WORKFLOW
        valid = workflow.read_text(encoding="utf-8")

        release_if = next(
            line
            for line in valid.splitlines()
            if line.startswith("    if: ")
        )

        for name, old, new, expected in (
            (
                "publish_condition_removed",
                release_if + "\n",
                "",
                "exact fail-closed publication condition",
            ),
            (
                "publish_condition_weakened",
                "inputs.publish == true",
                "inputs.publish != false",
                "exact fail-closed publication condition",
            ),
            (
                "publish_default_true",
                "        type: boolean\n        default: false\n",
                "        type: boolean\n        default: true\n",
                "publish must be a required boolean",
            ),
            (
                "build_job_conditional",
                "    runs-on: macos-14\n",
                "    runs-on: macos-14\n"
                "    if: ${{ inputs.publish == true }}\n",
                "build-macos-aarch64 must not be conditional",
            ),
            (
                "global_write_permission",
                "permissions:\n  contents: read\n",
                "permissions:\n  contents: write\n",
                "global permissions must be contents: read",
            ),
            (
                "release_action_in_build_job",
                "      - name: Upload artifact\n",
                "      - uses: softprops/action-gh-release@x\n"
                "      - name: Upload artifact\n",
                "must not reference 'softprops/action-gh-release@'",
            ),
            (
                "macos_intel_runner_drift",
                "    runs-on: macos-15-intel\n",
                "    runs-on: macos-latest\n",
                "build-macos-x86 must run on macos-15-intel",
            ),
            (
                "linux_llvm_patch_drift",
                "23.1.6",
                "23.1.5",
                "build-linux missing toolchain marker",
            ),
            (
                "linux_llvm_package_pin_removed",
                '            [[ "$version" =~ ^1:23\\.1\\.6~ ]]\n',
                "",
                "build-linux missing toolchain marker",
            ),
            (
                "rust_toolchain_drift",
                "uses: dtolnay/rust-toolchain@1.100.0\n",
                "uses: dtolnay/rust-toolchain@stable\n",
                "missing toolchain marker "
                "'uses: dtolnay/rust-toolchain@1.100.0'",
            ),
            # Markers must be executable structure, not comments.
            (
                "rust_action_only_in_comment",
                "      - name: Install Rust 1.100.0\n"
                "        uses: dtolnay/rust-toolchain@1.100.0\n",
                "      # uses: dtolnay/rust-toolchain@1.100.0\n",
                "missing toolchain marker "
                "'uses: dtolnay/rust-toolchain@1.100.0'",
            ),
            (
                "build_command_in_shell_comment",
                "        run: cargo build --release --locked\n",
                "        run: |\n"
                "          # cargo build --release --locked\n"
                "          cargo build --release\n",
                "missing toolchain marker "
                "'run: cargo build --release --locked'",
            ),
            (
                "publish_condition_in_comment",
                release_if + "\n",
                "    #" + release_if[5:] + "\n",
                "exact fail-closed publication condition",
            ),
            (
                "linux_llvm_pin_in_shell_comment",
                '            [[ "$version" =~ ^1:23\\.1\\.6~ ]]\n',
                '            # [[ "$version" =~ ^1:23\\.1\\.6~ ]]\n',
                "build-linux missing toolchain marker",
            ),
            (
                "linux_env_in_comment",
                "      LLVM_SYS_231_PREFIX: /usr/lib/llvm-23\n",
                "      # LLVM_SYS_231_PREFIX: /usr/lib/llvm-23\n",
                "missing toolchain marker "
                "'LLVM_SYS_231_PREFIX: /usr/lib/llvm-23'",
            ),
            (
                "binary_arch_check_removed",
                "file target/release/logicodex | grep -Fq 'arm64'",
                "file target/release/logicodex",
                "build-macos-aarch64 missing toolchain marker",
            ),
            (
                "binary_check_before_build",
                "      - name: Build release binary\n"
                "        run: cargo build --release --locked\n\n"
                "      - name: Verify binary\n"
                "        run: |\n"
                "          set -euo pipefail\n"
                "          file target/release/logicodex\n"
                "          ls -lh target/release/logicodex\n"
                "          file target/release/logicodex | grep -Fq 'arm64'\n",
                "      - name: Verify binary\n"
                "        run: |\n"
                "          set -euo pipefail\n"
                "          file target/release/logicodex\n"
                "          ls -lh target/release/logicodex\n"
                "          file target/release/logicodex | grep -Fq 'arm64'\n\n"
                "      - name: Build release binary\n"
                "        run: cargo build --release --locked\n",
                "build-macos-aarch64 steps must run LLVM setup",
            ),
            (
                "flow_mapping_unparseable",
                "permissions:\n  contents: read\n",
                "permissions: { contents: read }\n",
                "unparseable workflow structure",
            ),
            (
                "duplicate_key_unparseable",
                "    runs-on: macos-14\n",
                "    runs-on: macos-14\n    runs-on: macos-14\n",
                "duplicate key 'runs-on'",
            ),
        ):
            if old not in valid:
                raise RuntimeError(
                    f"release_{name}: mutation anchor missing"
                )

            workflow.write_text(
                valid.replace(old, new),
                encoding="utf-8",
            )

            expect_failure(
                f"release_{name}",
                root,
                expected,
            )

        workflow.write_text(valid, encoding="utf-8")
        validate_repository(root)

    print("release_workflow_contract_self_test=PASS")


def main() -> int:
    run_self_test()
    run_runtime_self_test()
    run_toolchain_self_test()
    run_release_workflow_self_test()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
