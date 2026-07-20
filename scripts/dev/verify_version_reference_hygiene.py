#!/usr/bin/env python3
"""Enforce canonical Logicodex version-reference hygiene."""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
import re
import sys

from version_reference_hygiene_runtime import (
    RuntimeVersionValidationError,
    validate_active_runtime_labels,
)


PRIMARY_SURFACES = (
    ".github/SECURITY.md",
    "src/main.rs",
    "docs/VS_CODE_EXTENSION.md",
    "docs/examples/REFLEX_ENGINE_EXAMPLES.md",
    "docs/guide/src/title.md",
    "docs/wiki/README.md",
    "extensions/vscode-logicodex/README.md",
    "extensions/vscode-logicodex/package.json",
    "extensions/vscode-logicodex/src/previewNormalizer.ts",
)

HOOK_FILES = (
    "scripts/dev/verify_quick_integrity.sh",
    "scripts/dev/verify_full_integrity.sh",
)

HISTORICAL_ZONE_REQUIREMENTS = (
    (
        "docs/archive/.zone-status.md",
        "HistoricalProvenance",
    ),
    (
        "scripts/_archive/.zone-status.md",
        "ArchivedZone",
    ),
    (
        "spec/v1.11-alpha/.zone-status.md",
        "HistoricalProvenance",
    ),
    (
        "spec/v1.21-alpha/.zone-status.md",
        "HistoricalProvenance",
    ),
    (
        "spec/v1.30.0-alpha/.zone-status.md",
        "HistoricalProvenance",
    ),
)

FORBIDDEN_CURRENT_CLAIMS = (
    re.compile(
        r"current Logicodex v1\.(?:21|30|45)",
        re.IGNORECASE,
    ),
    re.compile(
        r"current logicodex v 1\.21",
        re.IGNORECASE,
    ),
    re.compile(
        r"v1\.30\.0-alpha \(current engine\)",
        re.IGNORECASE,
    ),
    re.compile(
        r"LOGICODEX COMPILER v1\.30\.0-alpha",
    ),
    re.compile(
        r"logicodex 1\.30\.0-alpha",
    ),
    re.compile(
        r'version\s*=\s*"1\.30\.0-alpha"',
    ),
)


class ValidationError(RuntimeError):
    """Raised when version-reference hygiene fails."""

    def __init__(self, errors: list[str]) -> None:
        self.errors = errors
        super().__init__("\n".join(errors))


@dataclass(frozen=True)
class ValidationSummary:
    cargo_version: str
    checked_surfaces: int
    compatibility_selectors: int
    active_runtime_surfaces: int
    generated_version_authorities: int
    historical_zone_markers: int


def read_text(root: Path, relative: str) -> str:
    path = root / relative

    if not path.is_file():
        raise ValidationError(
            [f"required file is missing: {relative}"]
        )

    return path.read_text(
        encoding="utf-8",
        errors="strict",
    )


def cargo_version(root: Path) -> str:
    text = read_text(root, "Cargo.toml")

    match = re.search(
        r'(?m)^version\s*=\s*"([^"]+)"\s*$',
        text,
    )

    if not match:
        raise ValidationError(
            ["Cargo.toml package version is missing"]
        )

    return match.group(1)


def validate_repository(
    root: Path,
) -> ValidationSummary:
    errors: list[str] = []
    version = cargo_version(root)

    try:
        runtime_summary = (
            validate_active_runtime_labels(root)
        )
    except RuntimeVersionValidationError as error:
        errors.extend(error.errors)
        runtime_summary = None

    surfaces: dict[str, str] = {}

    for relative in PRIMARY_SURFACES:
        try:
            surfaces[relative] = read_text(
                root,
                relative,
            )
        except ValidationError as error:
            errors.extend(error.errors)

    for relative, text in surfaces.items():
        for pattern in FORBIDDEN_CURRENT_CLAIMS:
            for match in pattern.finditer(text):
                line = text.count(
                    "\n",
                    0,
                    match.start(),
                ) + 1

                errors.append(
                    f"{relative}:{line}: stale active "
                    f"version claim: {match.group(0)!r}"
                )

    main = surfaces.get("src/main.rs", "")

    if main.count('env!("CARGO_PKG_VERSION")') < 3:
        errors.append(
            "src/main.rs must derive banner, long version, "
            "and Clap version from CARGO_PKG_VERSION"
        )

    compatibility_markers = (
        'default_value = "v1.30"',
        'name = "v130-check"',
        "CompilerPipeline::V130",
    )

    for marker in compatibility_markers:
        if marker not in main:
            errors.append(
                "src/main.rs compatibility marker missing: "
                + marker
            )

    security = surfaces.get(
        ".github/SECURITY.md",
        "",
    )

    expected_security = (
        f"| v{version} (current Cargo release authority) "
        "| ✅ Current — best-effort alpha security support |"
    )

    if expected_security not in security:
        errors.append(
            ".github/SECURITY.md does not identify the "
            "Cargo version as current authority"
        )

    expected_current_markers = {
        "docs/VS_CODE_EXTENSION.md":
            f"current Logicodex v{version}",
        "docs/wiki/README.md":
            f"Logicodex Documentation — v{version}",
        "extensions/vscode-logicodex/README.md":
            f"current Logicodex v{version}",
        "extensions/vscode-logicodex/package.json":
            f"current Logicodex v{version}",
    }

    for relative, marker in expected_current_markers.items():
        if marker not in surfaces.get(relative, ""):
            errors.append(
                f"{relative}: canonical version marker missing"
            )

    guide = surfaces.get(
        "docs/guide/src/title.md",
        "",
    )

    if (
        "**Historical guide baseline:** v1.45.0-alpha "
        "(non-authoritative)"
        not in guide
    ):
        errors.append(
            "stale guide must label v1.45 as a historical, "
            "non-authoritative baseline"
        )

    reflex = surfaces.get(
        "docs/examples/REFLEX_ENGINE_EXAMPLES.md",
        "",
    )

    if (
        "current single HIR compiler path"
        not in reflex
        or "deprecated compatibility alias"
        not in reflex
    ):
        errors.append(
            "reflex examples must describe the current "
            "single-engine compatibility boundary"
        )

    preview = surfaces.get(
        "extensions/vscode-logicodex/"
        "src/previewNormalizer.ts",
        "",
    )

    if (
        "current single HIR compiler path is authoritative"
        not in preview
    ):
        errors.append(
            "Side View notice must point to the current "
            "single HIR compiler path"
        )

    for relative, expected_classification in (
        HISTORICAL_ZONE_REQUIREMENTS
    ):
        try:
            marker = read_text(root, relative)
        except ValidationError:
            errors.append(
                f"historical zone marker missing: {relative}"
            )
            continue

        expected_marker = (
            "Classification: "
            f"`{expected_classification}`"
        )

        if expected_marker not in marker:
            errors.append(
                "historical zone classification mismatch: "
                f"{relative} must contain {expected_marker!r}"
            )

    for relative in HOOK_FILES:
        try:
            hook = read_text(root, relative)
        except ValidationError as error:
            errors.extend(error.errors)
            continue

        for script in (
            "test_version_reference_hygiene.py",
            "verify_version_reference_hygiene.py",
        ):
            if script not in hook:
                errors.append(
                    f"{relative}: missing gate for {script}"
                )

    ci = read_text(
        root,
        ".github/workflows/ci.yml",
    )

    for hook in HOOK_FILES:
        for script in (
            "test_version_reference_hygiene.py",
            "verify_version_reference_hygiene.py",
        ):
            expected = (
                f"grep -Fq '{script}' {hook}"
            )

            if expected not in ci:
                errors.append(
                    ".github/workflows/ci.yml: "
                    f"missing hook assertion: {expected}"
                )

    if errors:
        raise ValidationError(errors)

    assert runtime_summary is not None

    return ValidationSummary(
        cargo_version=version,
        checked_surfaces=len(surfaces),
        compatibility_selectors=3,
        active_runtime_surfaces=(
            runtime_summary.checked_surfaces
        ),
        generated_version_authorities=(
            runtime_summary.generated_version_authorities
        ),
        historical_zone_markers=len(
            HISTORICAL_ZONE_REQUIREMENTS
        ),
    )


def main() -> int:
    root = Path(
        sys.argv[1]
        if len(sys.argv) > 1
        else "."
    ).resolve()

    try:
        summary = validate_repository(root)
    except ValidationError as error:
        for message in error.errors:
            print(
                f"version_reference_hygiene_error={message}",
                file=sys.stderr,
            )

        return 1

    print(
        "version_reference_hygiene=PASS"
    )
    print(
        f"cargo_version={summary.cargo_version}"
    )
    print(
        f"checked_surfaces={summary.checked_surfaces}"
    )
    print(
        "compatibility_selectors_preserved="
        f"{summary.compatibility_selectors}"
    )
    print(
        "active_runtime_surfaces="
        f"{summary.active_runtime_surfaces}"
    )
    print(
        "generated_version_authorities="
        f"{summary.generated_version_authorities}"
    )
    print(
        "historical_zone_markers="
        f"{summary.historical_zone_markers}"
    )

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
