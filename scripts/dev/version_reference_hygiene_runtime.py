#!/usr/bin/env python3
"""Validate active runtime, generated-artifact, and toolchain labels."""

from __future__ import annotations

from collections import Counter
from dataclasses import dataclass
import os
from pathlib import Path
import re
import subprocess


ACTIVE_RUNTIME_SURFACES = (
    "lib/linker_scripts/x86_64-freestanding.ld",
    "src/codegen.rs",
    "src/net/affinity.rs",
    "src/net/connection.rs",
    "src/net/reactor.rs",
    "src/net/sharded_reactor.rs",
    "src/os/syscall.rs",
    "src/semantic_gate.rs",
    "src/tier2/capability_ir.rs",
    "src/tier2/ctl_mapper.rs",
    "src/tier2/pass.rs",
)

GENERATED_AUTHORITY_COUNTS = {
    "src/tier2/capability_ir.rs": 2,
    "src/tier2/ctl_mapper.rs": 2,
}

LINKER_SURFACE = (
    "lib/linker_scripts/x86_64-freestanding.ld"
)

LINKER_HEADER = (
    "Logicodex — Linker Script "
    "for Freestanding x86_64"
)

OLD_VERSION_RE = re.compile(
    r"\bv1\.[0-9]{2}"
    r"(?:\.[0-9]+)?"
    r"(?:-alpha)?\b",
    re.IGNORECASE,
)


class RuntimeVersionValidationError(RuntimeError):
    """Raised when active runtime version labels are stale."""

    def __init__(self, errors: list[str]) -> None:
        self.errors = errors
        super().__init__("\n".join(errors))


@dataclass(frozen=True)
class RuntimeVersionSummary:
    checked_surfaces: int
    generated_version_authorities: int


def read_text(
    root: Path,
    relative: str,
) -> str:
    path = root / relative

    if not path.is_file():
        raise RuntimeVersionValidationError(
            [f"required runtime surface is missing: {relative}"]
        )

    return path.read_text(
        encoding="utf-8",
        errors="strict",
    )


def is_comment_line(line: str) -> bool:
    stripped = line.strip()

    return stripped.startswith(
        (
            "//",
            "/*",
            "*",
            "#",
        )
    )


def validate_active_runtime_labels(
    root: Path,
) -> RuntimeVersionSummary:
    errors: list[str] = []
    surfaces: dict[str, str] = {}

    for relative in ACTIVE_RUNTIME_SURFACES:
        try:
            surfaces[relative] = read_text(
                root,
                relative,
            )
        except RuntimeVersionValidationError as error:
            errors.extend(error.errors)

    for relative, text in surfaces.items():
        if relative == LINKER_SURFACE:
            continue

        for line_number, line in enumerate(
            text.splitlines(),
            start=1,
        ):
            if is_comment_line(line):
                continue

            match = OLD_VERSION_RE.search(line)

            if match and '"' in line:
                errors.append(
                    f"{relative}:{line_number}: stale active "
                    "runtime version label: "
                    f"{match.group(0)!r}"
                )

    authority_count = 0

    for relative, expected_count in (
        GENERATED_AUTHORITY_COUNTS.items()
    ):
        text = surfaces.get(relative, "")
        actual_count = text.count(
            'env!("CARGO_PKG_VERSION")'
        )

        if actual_count < expected_count:
            errors.append(
                f"{relative}: generated headers must derive "
                "from CARGO_PKG_VERSION "
                f"({actual_count}/{expected_count})"
            )
        else:
            authority_count += expected_count

    linker = surfaces.get(
        LINKER_SURFACE,
        "",
    )

    linker_header_region = "\n".join(
        linker.splitlines()[:5]
    )

    if (
        LINKER_HEADER not in linker_header_region
        or OLD_VERSION_RE.search(linker_header_region)
    ):
        errors.append(
            f"{LINKER_SURFACE}: linker header must be "
            "version-agnostic"
        )

    if errors:
        raise RuntimeVersionValidationError(errors)

    return RuntimeVersionSummary(
        checked_surfaces=len(surfaces),
        generated_version_authorities=authority_count,
    )


# ------------------------------------------------------------------
# Toolchain version policy: the only approved active toolchain is
# LLVM 23 (23.1.6 pinned by the workflows) + Rust 1.100 (1.100.0).
# Every other LLVM major and Rust 1.x release in toolchain context is
# rejected outside historical zones and exact exceptions. The retired
# toolchain constants remain for self-tests. Token literals
# are assembled from fragments so that this source, the self-tests,
# and docs describing the guard never match.
# ------------------------------------------------------------------

APPROVED_LLVM_MAJOR = "2" + "3"
APPROVED_RUST_MINOR = "1." + "100"

RETIRED_LLVM_MAJOR = "1" + "5"
RETIRED_RUST_MINOR = "1." + "75"

_LLVM_SYS_APPROVED = (APPROVED_LLVM_MAJOR + "1").encode()
_RUST_MINOR_CAPTURE = rb"(?<![0-9.])(1\.[0-9]+)(?![0-9])"


@dataclass(frozen=True)
class _ToolchainPattern:
    regex: re.Pattern[bytes]
    approved: bytes


TOOLCHAIN_VERSION_PATTERNS = (
    # llvm-N, llvm@N, llvm N, llvm_N, llvmN, llvmN-0; identifiers such
    # as a `LLVMN_DATA_LAYOUT` constant are deliberately not matched.
    _ToolchainPattern(
        re.compile(
            rb"llvm[-@ _]?([0-9]+)(?![0-9A-Za-z_])",
            re.IGNORECASE,
        ),
        APPROVED_LLVM_MAJOR.encode(),
    ),
    # llvm-sys feature/env prefix: LLVM_SYS_<major><minor>_PREFIX.
    _ToolchainPattern(
        re.compile(rb"llvm_sys_([0-9]+)", re.IGNORECASE),
        _LLVM_SYS_APPROVED,
    ),
    _ToolchainPattern(
        re.compile(
            rb"\b(?:lib)?(?:clang|lld|polly)-([0-9]+)(?![0-9])",
            re.IGNORECASE,
        ),
        APPROVED_LLVM_MAJOR.encode(),
    ),
    # Rust is matched only in toolchain context, never as a bare number.
    _ToolchainPattern(
        re.compile(
            rb"\brust(?:c|up)?\b[^\n]{0,24}?" + _RUST_MINOR_CAPTURE,
            re.IGNORECASE,
        ),
        APPROVED_RUST_MINOR.encode(),
    ),
    _ToolchainPattern(
        re.compile(
            rb"\btoolchain\b[^\n]{0,32}?" + _RUST_MINOR_CAPTURE,
            re.IGNORECASE,
        ),
        APPROVED_RUST_MINOR.encode(),
    ),
    _ToolchainPattern(
        re.compile(rb"(?:^|\s)\+(1\.[0-9]+)(?![0-9])"),
        APPROVED_RUST_MINOR.encode(),
    ),
    _ToolchainPattern(
        re.compile(
            rb"\bchannel\s*=\s*[\"'](1\.[0-9]+)(?![0-9])",
            re.IGNORECASE,
        ),
        APPROVED_RUST_MINOR.encode(),
    ),
)

# Historical zones keep provenance; they mirror the zone markers that
# verify_version_reference_hygiene.py requires. Untracked files are
# never exempt.
TOOLCHAIN_HISTORICAL_ZONES = (
    re.compile(r"^docs/archive/"),
    re.compile(r"^scripts/_archive/"),
    re.compile(r"^spec/v[^/]+/"),
)

_L = RETIRED_LLVM_MAJOR
_R = RETIRED_RUST_MINOR

# Exact historical lines outside the zones. Each entry is allowed
# exactly once; an absent entry is a stale exception and fails.
TOOLCHAIN_HISTORICAL_EXCEPTIONS: dict[str, tuple[str, ...]] = {
    "docs/GAPS_ASSESSMENT.md": (
        f'**Critical fix:** Added `RUSTFLAGS: "-L/usr/lib/llvm-{_L}/lib"` '
        "to global `env:` section so all jobs can find LLVM libraries.",
        f"FROM rust:{_R}-slim",
        f"RUN apt-get update && apt-get install -y llvm-{_L}-dev "
        f"libclang-{_L}-dev pkg-config",
        f'RUN RUSTFLAGS="-L/usr/lib/llvm-{_L}/lib" cargo build --release',
    ),
    "docs/audit/main-stability-phase-readiness.md": (
        "CI (`.github/workflows/ci.yml`) runs three jobs on "
        f"Rust {_R}.0 + LLVM {_L}:",
    ),
    "docs/audit/stdlib-core-to-main-merge-readiness.md": (
        f"  validator). Toolchain Rust {_R}.0, LLVM {_L}.",
    ),
    # Mock IR placeholder ident from the v1.21 proof-of-concept demo.
    "examples/poc/03-native/build.sh": (
        '!1 = !{!"Logicodex v1.21 - LLVM ' + "1" + '7.0"}',
    ),
    "logicodex-os/src/panic.rs": (
        "//! Differences from the original "
        f"(toolchain constraints on stable {_R}):",
    ),
    "tests/llvm23_target_contract.rs": (
        f"// LLVM {_L} control contract, retained as the raw-diff baseline.",
    ),
}

BASE_SHA_RE = re.compile(r"^[0-9a-f]{40}(?:[0-9a-f]{24})?$")

_GIT_ENV_OVERRIDES = (
    "GIT_DIR",
    "GIT_WORK_TREE",
    "GIT_INDEX_FILE",
    "GIT_OBJECT_DIRECTORY",
    "GIT_COMMON_DIR",
)


@dataclass(frozen=True)
class ToolchainReferenceSummary:
    scanned_paths: int
    staged_blobs: int
    historical_exceptions: int
    base_sha: str | None


class _GitError(RuntimeError):
    pass


def _git(
    root: Path,
    *args: str,
    check: bool = True,
) -> subprocess.CompletedProcess[bytes]:
    env = {
        key: value
        for key, value in os.environ.items()
        if key not in _GIT_ENV_OVERRIDES
    }

    try:
        result = subprocess.run(
            ["git", "-C", str(root), *args],
            capture_output=True,
            env=env,
            check=False,
        )
    except OSError as error:
        raise _GitError(f"git unavailable: {error}") from error

    if check and result.returncode != 0:
        raise _GitError(
            f"git {' '.join(args)} failed: "
            + result.stderr.decode("utf-8", "replace").strip()
        )

    return result


def _nul_paths(output: bytes) -> list[str]:
    return [
        item.decode("utf-8", "surrogateescape")
        for item in output.split(b"\0")
        if item
    ]


def _is_unapproved(line: bytes) -> bool:
    return any(
        match.group(1).lower() != pattern.approved
        for pattern in TOOLCHAIN_VERSION_PATTERNS
        for match in pattern.regex.finditer(line)
    )


def _flagged_lines(data: bytes) -> list[tuple[int, bytes]]:
    return [
        (number, line)
        for number, line in enumerate(data.split(b"\n"), start=1)
        if _is_unapproved(line)
    ]


def _added_lines(
    baseline: bytes | None,
    data: bytes,
) -> list[tuple[int, bytes]]:
    """Flagged lines of ``data`` beyond the multiset in ``baseline``."""

    remaining = Counter(line for _, line in _flagged_lines(baseline or b""))
    added = []

    for number, line in _flagged_lines(data):
        if remaining[line] > 0:
            remaining[line] -= 1
        else:
            added.append((number, line))

    return added


def _read_blob(root: Path, spec: str) -> bytes | None:
    result = _git(root, "cat-file", "blob", spec, check=False)
    return result.stdout if result.returncode == 0 else None


def _diff_paths(root: Path, *args: str) -> set[str]:
    return set(
        _nul_paths(
            _git(
                root,
                "diff",
                "--name-only",
                "-z",
                "--no-renames",
                "--diff-filter=ACMRT",
                *args,
            ).stdout
        )
    )


def _in_historical_zone(relative: str) -> bool:
    return any(
        zone.search(relative)
        for zone in TOOLCHAIN_HISTORICAL_ZONES
    )


def _read_worktree(root: Path, relative: str) -> bytes | None:
    path = root / relative

    if path.is_symlink():
        return os.readlink(path).encode(
            "utf-8",
            "surrogateescape",
        )

    if not path.exists():
        return None

    return path.read_bytes()


def _show_line(line: bytes) -> str:
    return repr(line.decode("utf-8", "replace").strip())[:120]


def validate_toolchain_references(
    root: Path,
    base_sha: str | None = None,
) -> ToolchainReferenceSummary:
    """Fail closed on non-approved LLVM/Rust references in the tree.

    Scans raw bytes of every tracked and untracked (non-ignored)
    worktree file plus every staged blob. Tracked historical-zone
    content is exempt only where it matches HEAD: worktree or staged
    edits to a zone file may not add references beyond its HEAD blob.
    With ``base_sha``, also rejects references added since that
    commit anywhere, including historical zones, in each of HEAD, the
    index, and the worktree.
    """

    errors: list[str] = []

    try:
        toplevel = _git(root, "rev-parse", "--show-toplevel")
    except _GitError as error:
        raise RuntimeVersionValidationError(
            [f"toolchain reference scan requires a git work tree: {error}"]
        ) from error

    if Path(toplevel.stdout.decode().strip()).resolve() != root.resolve():
        raise RuntimeVersionValidationError(
            [
                "toolchain reference scan root is not the git "
                f"top level: {root}"
            ]
        )

    try:
        tracked = _nul_paths(_git(root, "ls-files", "-z", "--cached").stdout)
        untracked = set(
            _nul_paths(
                _git(
                    root,
                    "ls-files",
                    "-z",
                    "--others",
                    "--exclude-standard",
                ).stdout
            )
        )
        gitlinks = {
            entry.split(b"\t", 1)[1].decode("utf-8", "surrogateescape")
            for entry in _git(root, "ls-files", "-z", "-s").stdout.split(b"\0")
            if entry.startswith(b"160000 ")
        }
        staged = set(
            _nul_paths(
                _git(
                    root,
                    "diff",
                    "--cached",
                    "--name-only",
                    "-z",
                    "--no-renames",
                    "--diff-filter=ACMRT",
                ).stdout
            )
        )
        deleted = set(
            _nul_paths(_git(root, "ls-files", "-z", "-d").stdout)
        )
        has_head = (
            _git(
                root,
                "rev-parse",
                "--verify",
                "-q",
                "HEAD^{commit}",
                check=False,
            ).returncode
            == 0
        )
        # Paths whose worktree or index content differs from HEAD.
        # Without HEAD no zone content is committed provenance.
        dirty = (
            _diff_paths(root, "HEAD") | _diff_paths(root, "--cached", "HEAD")
            if has_head
            else None
        )
    except _GitError as error:
        raise RuntimeVersionValidationError([str(error)]) from error

    head_blobs: dict[str, bytes | None] = {}

    def zone_baseline(relative: str) -> bytes | None:
        if relative not in head_blobs:
            head_blobs[relative] = (
                _read_blob(root, f"HEAD:{relative}") if has_head else None
            )

        return head_blobs[relative]

    allowed: dict[str, Counter[bytes]] = {
        relative: Counter(line.encode("utf-8") for line in lines)
        for relative, lines in TOOLCHAIN_HISTORICAL_EXCEPTIONS.items()
    }
    used: dict[str, Counter[bytes]] = {
        relative: Counter() for relative in allowed
    }

    def check_blob(
        relative: str,
        data: bytes,
        source: str,
        count_exceptions: bool,
    ) -> None:
        if relative not in untracked and _in_historical_zone(relative):
            # Committed zone content is provenance; pending zone edits
            # are checked against the HEAD blob, never exempted.
            if dirty is not None and relative not in dirty:
                return

            for number, line in _added_lines(zone_baseline(relative), data):
                errors.append(
                    f"{relative}:{number}: non-approved toolchain "
                    f"reference added to historical zone since HEAD "
                    f"({source}): {_show_line(line)}"
                )

            return

        seen: Counter[bytes] = Counter()

        for number, line in _flagged_lines(data):
            seen[line] += 1

            if seen[line] <= allowed.get(relative, Counter())[line]:
                continue

            errors.append(
                f"{relative}:{number}: non-approved toolchain reference "
                f"({source}): {_show_line(line)}"
            )

        if count_exceptions and relative in used:
            used[relative] = seen

    scanned = 0
    staged_blobs = 0

    for relative in sorted(set(tracked) | untracked):
        if relative in gitlinks:
            continue

        try:
            data = _read_worktree(root, relative)
        except OSError as error:
            errors.append(f"{relative}: unreadable worktree file: {error}")
            continue

        if data is None:
            if relative in untracked:
                errors.append(f"{relative}: untracked file vanished")
            continue

        scanned += 1
        check_blob(relative, data, "worktree", True)

    for relative in sorted((staged | deleted) - gitlinks):
        try:
            blob = _git(root, "cat-file", "blob", f":{relative}")
        except _GitError as error:
            errors.append(f"{relative}: unreadable staged blob: {error}")
            continue

        staged_blobs += 1
        check_blob(relative, blob.stdout, "staged", False)

    for relative, expected in allowed.items():
        for line, count in expected.items():
            if used[relative][line] < count:
                errors.append(
                    f"{relative}: stale historical toolchain "
                    f"exception: {_show_line(line)}"
                )

    if base_sha is not None:
        errors.extend(_validate_base_additions(root, base_sha, gitlinks))

    if errors:
        raise RuntimeVersionValidationError(errors)

    return ToolchainReferenceSummary(
        scanned_paths=scanned,
        staged_blobs=staged_blobs,
        historical_exceptions=sum(
            len(lines) for lines in TOOLCHAIN_HISTORICAL_EXCEPTIONS.values()
        ),
        base_sha=base_sha,
    )


def _validate_base_additions(
    root: Path,
    base_sha: str,
    gitlinks: set[str],
) -> list[str]:
    """Reject references added since ``base_sha`` in HEAD, index, or worktree.

    Each view is compared independently so that committed or staged
    additions cannot be hidden behind a differing worktree copy.
    """

    if not BASE_SHA_RE.fullmatch(base_sha):
        return [f"BASE_SHA is not a full lowercase object id: {base_sha!r}"]

    if _git(root, "cat-file", "-e", f"{base_sha}^{{commit}}", check=False).returncode:
        return [f"BASE_SHA does not resolve to a commit: {base_sha}"]

    if _git(root, "merge-base", "--is-ancestor", base_sha, "HEAD", check=False).returncode:
        return [f"BASE_SHA is not an ancestor of HEAD: {base_sha}"]

    errors: list[str] = []

    try:
        changed = (
            _diff_paths(root, base_sha, "HEAD")
            | _diff_paths(root, "--cached", base_sha)
            | _diff_paths(root, base_sha)
        ) - gitlinks
    except _GitError as error:
        return [str(error)]

    for relative in sorted(changed):
        baseline = _read_blob(root, f"{base_sha}:{relative}")

        try:
            worktree = _read_worktree(root, relative)
        except OSError as error:
            errors.append(f"{relative}: unreadable worktree file: {error}")
            worktree = None

        checked: list[bytes] = []

        for source, data in (
            ("HEAD", _read_blob(root, f"HEAD:{relative}")),
            ("staged", _read_blob(root, f":{relative}")),
            ("worktree", worktree),
        ):
            if data is None or data in checked:
                continue

            checked.append(data)

            for number, line in _added_lines(baseline, data):
                errors.append(
                    f"{relative}:{number}: non-approved toolchain reference "
                    f"added since BASE_SHA ({source}): {_show_line(line)}"
                )

    return errors
