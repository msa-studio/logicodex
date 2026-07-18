#!/usr/bin/env python3
"""Validate active runtime and generated-artifact version labels."""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
import re


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
