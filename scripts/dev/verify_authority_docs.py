#!/usr/bin/env python3
"""Fail closed when Logicodex current-authority documentation drifts."""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

AUTHORITY = Path("docs/architecture/current-authority.md")
AUTHORITY_REF = AUTHORITY.as_posix()
SEQUENCE_HEADING = "## Active owner-locked sequence"
SEQUENCE_ITEMS = (
    "`CPB-2 Callable and Function Type Closure`",
    "`CPB-2 Assignment and Return Compatibility`",
    "`CPB-2 Diagnostic End-to-End`",
)
POINTERS = {
    Path("AGENTS.md"): AUTHORITY_REF,
    Path("README.md"): AUTHORITY_REF,
    Path("ROADMAP_v2.md"): AUTHORITY_REF,
    Path(".github/ROADMAP_POLICY.md"): AUTHORITY_REF,
    Path(".github/workflows/gatekeeper.yml"): AUTHORITY_REF,
    Path(".github/AUDIT_TEMPLATE.md"): AUTHORITY_REF,
    Path("docs/DOCUMENTATION_POLICY.md"): "architecture/current-authority.md",
    Path("docs/architecture/code-lifecycle-inventory.md"): AUTHORITY.name,
    Path("docs/architecture/semantic-lifecycle-status.md"): AUTHORITY.name,
    Path("docs/architecture/version-reference-classification.md"): AUTHORITY.name,
    Path("docs/architecture/CONTRACT_EXTENSION_ARCHITECTURE.md"): AUTHORITY.name,
    Path("docs/architecture/cpb-next-roadmap-blockers.md"): AUTHORITY.name,
}
EVIDENCE = (
    Path("docs/architecture/code-lifecycle-inventory.md"),
    Path("docs/architecture/semantic-lifecycle-status.md"),
    Path("docs/architecture/version-reference-classification.md"),
)
STALE_BANNER_FILES = (
    Path("SPECIFICATION.md"),
    Path("docs/GAPS_ASSESSMENT.md"),
    Path("docs/HANDBOOK.md"),
    Path("docs/guide/src/title.md"),
    Path("docs/white-paper/src/title.md"),
)
GENERIC_AUTHORITY = (
    "Authoritative current references: `README.md`, `examples/`, and "
    "`docs/architecture/`."
)


def read(root: Path, relative: Path, errors: list[str]) -> str:
    path = root / relative
    if not path.is_file():
        errors.append(f"missing required file: {relative.as_posix()}")
        return ""
    return path.read_text(encoding="utf-8")


def active_markdown(root: Path):
    for path in root.rglob("*.md"):
        relative = path.relative_to(root)
        parts = set(relative.parts)
        if parts & {".git", "target", "node_modules"}:
            continue
        if "archive" in parts or "_archive" in parts:
            continue
        if relative.parts[:1] == ("spec",):
            continue
        yield relative, path.read_text(encoding="utf-8")


def validate(root: Path) -> list[str]:
    errors: list[str] = []
    authority = read(root, AUTHORITY, errors)
    for required in (
        "Status: Active",
        "## Authority order",
        "## Change-size and architecture boundary",
        "## Locked current facts",
        "## SSM-D4 transition gate",
        SEQUENCE_HEADING,
        "## Residual debt disposition",
        "## Subordinate evidence records",
        "## Update rule",
        "`phase-1`",
        "above 500 changed lines carries `size-exception`",
        "Documentation updates have no upper line cap",
        "Architecture-altering work is outside standing authority",
    ):
        if authority and required not in authority:
            errors.append(f"authority entrypoint missing: {required}")

    section = re.search(
        rf"^{re.escape(SEQUENCE_HEADING)}\s*$\n(.*?)(?=^## |\Z)",
        authority,
        flags=re.MULTILINE | re.DOTALL,
    )
    expected_sequence = [
        f"{number}. {item}" for number, item in enumerate(SEQUENCE_ITEMS, start=1)
    ]
    actual_sequence = (
        [
            line.strip()
            for line in section.group(1).splitlines()
            if re.match(r"^\d+\. ", line.strip())
        ]
        if section
        else []
    )
    if authority and actual_sequence != expected_sequence:
        errors.append("owner-locked sequence must be the exact three-item list")

    for relative, reference in POINTERS.items():
        text = read(root, relative, errors)
        if text and reference not in text:
            errors.append(
                f"authority pointer missing: {relative.as_posix()} -> {reference}"
            )

    for relative in EVIDENCE:
        read(root, relative, errors)

    heading_owners = [
        relative.as_posix()
        for relative, text in active_markdown(root)
        if SEQUENCE_HEADING in text
    ]
    if heading_owners != [AUTHORITY_REF]:
        errors.append(
            "active sequence must have exactly one owner: "
            + (", ".join(heading_owners) or "none")
        )

    lifecycle = read(root, EVIDENCE[0], errors)
    if "add a lifecycle validator in SSM-D3".lower() in lifecycle.lower():
        errors.append("stale SSM-D3 lifecycle-validator claim")
    blockers = read(root, Path("docs/architecture/cpb-next-roadmap-blockers.md"), errors)
    if "merge stdlib-core foundation to main".lower() in blockers.lower():
        errors.append("stale stdlib-core merge claim")
    contracts = read(
        root, Path("docs/architecture/CONTRACT_EXTENSION_ARCHITECTURE.md"), errors
    )
    if "Recommended order:" in contracts:
        errors.append("stale contract implementation order")

    policy = read(root, Path(".github/ROADMAP_POLICY.md"), errors)
    if re.search(r"(?<![_A-Za-z0-9])ROADMAP\.md", policy):
        errors.append("unqualified ROADMAP.md authority pointer")

    audit_template = read(root, Path(".github/AUDIT_TEMPLATE.md"), errors)
    stale_audit_tokens = ("ROADMAP.md", "v1.21", "v1.30", "v1.45", "v1.50")
    for token in stale_audit_tokens:
        if token in audit_template:
            errors.append(f"stale audit-template authority: {token}")

    control = read(
        root, Path("docs/governance/architecture-change-control.md"), errors
    )
    normalized_control = " ".join(control.split())
    for required in (
        "Documentation updates have no upper line cap.",
        "above 500 changed lines uses `size-exception`",
        "does not make it an architecture change.",
    ):
        if control and required not in normalized_control:
            errors.append(f"governance size policy missing: {required}")

    for relative in STALE_BANNER_FILES:
        text = read(root, relative, errors)
        if GENERIC_AUTHORITY in text:
            errors.append(f"generic architecture authority banner: {relative.as_posix()}")
        if text and AUTHORITY_REF not in text:
            errors.append(f"authority pointer missing from banner: {relative.as_posix()}")

    return errors


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[2])
    root = parser.parse_args().root.resolve()
    errors = validate(root)
    if errors:
        for error in errors:
            print(f"authority_docs_error={error}", file=sys.stderr)
        print(f"authority_docs_validation=FAIL errors={len(errors)}", file=sys.stderr)
        return 1
    print(f"authority_entrypoint={AUTHORITY_REF}")
    print(f"authority_pointer_files={len(POINTERS)}")
    print(f"authority_evidence_files={len(EVIDENCE)}")
    print("authority_docs_validation=PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
