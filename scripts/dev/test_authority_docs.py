#!/usr/bin/env python3
"""Regression self-test for verify_authority_docs.py."""

from __future__ import annotations

import subprocess
import sys
import tempfile
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
VERIFIER = REPO / "scripts/dev/verify_authority_docs.py"
AUTHORITY = "docs/architecture/current-authority.md"
ACTIVE = Path("docs/architecture/current-authority.md")


def put(root: Path, relative: str | Path, text: str) -> None:
    path = root / relative
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


def fixture(root: Path) -> None:
    authority = """# Logicodex Current Authority
Status: Active
## Authority order
## Change-size and architecture boundary
- Any pull request above 500 changed lines carries `size-exception`.
- Documentation updates have no upper line cap.
- Architecture-altering work is outside standing authority.
## Locked current facts
- `phase-1`
## SSM-D4 transition gate
## Active owner-locked sequence
1. `CPB-2 Callable and Function Type Closure`
2. `CPB-2 Assignment and Return Compatibility`
3. `CPB-2 Diagnostic End-to-End`
## Residual debt disposition
## Subordinate evidence records
## Update rule
"""
    put(root, ACTIVE, authority)
    put(root, "AGENTS.md", AUTHORITY)
    put(root, "README.md", AUTHORITY)
    put(root, "ROADMAP_v2.md", AUTHORITY)
    put(root, ".github/workflows/gatekeeper.yml", AUTHORITY)
    put(root, ".github/AUDIT_TEMPLATE.md", AUTHORITY)
    put(root, "docs/DOCUMENTATION_POLICY.md", "architecture/current-authority.md")
    put(
        root,
        "docs/governance/architecture-change-control.md",
        "Documentation updates have no upper line cap.\n"
        "above 500 changed lines uses `size-exception`.\n"
        "does not make it an architecture change.\n",
    )
    put(
        root,
        ".github/ROADMAP_POLICY.md",
        f"Current work-sequence authority is `{AUTHORITY}`.\n",
    )
    put(root, "docs/architecture/cpb-next-roadmap-blockers.md", ACTIVE.name)
    put(root, "docs/architecture/code-lifecycle-inventory.md", "current-authority.md\nvalidator active\n")
    put(root, "docs/architecture/semantic-lifecycle-status.md", "current-authority.md\nsemantic evidence\n")
    put(root, "docs/architecture/version-reference-classification.md", "current-authority.md\nversion evidence\n")
    put(root, "docs/architecture/CONTRACT_EXTENSION_ARCHITECTURE.md", "current-authority.md\nhistorical implementation record\n")
    for relative in (
        "SPECIFICATION.md",
        "docs/GAPS_ASSESSMENT.md",
        "docs/HANDBOOK.md",
        "docs/guide/src/title.md",
        "docs/white-paper/src/title.md",
    ):
        put(root, relative, f"Start with `{AUTHORITY}`.\n")


def run(root: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [sys.executable, "-B", str(VERIFIER), "--root", str(root)],
        capture_output=True,
        text=True,
        check=False,
    )


def passes(name: str, root: Path) -> None:
    result = run(root)
    if result.returncode:
        raise AssertionError(f"{name}: expected PASS\n{result.stdout}{result.stderr}")
    print(f"self_test_{name}=PASS")


def fails(name: str, root: Path, expected: str) -> None:
    result = run(root)
    output = result.stdout + result.stderr
    if result.returncode == 0:
        raise AssertionError(f"{name}: unexpectedly passed")
    if expected not in output:
        raise AssertionError(f"{name}: missing {expected!r}\n{output}")
    print(f"self_test_{name}=PASS")


def main() -> int:
    with tempfile.TemporaryDirectory(prefix="logicodex-authority-") as directory:
        root = Path(directory)

        fixture(root)
        passes("valid_fixture", root)

        fixture(root)
        (root / ACTIVE).unlink()
        fails("missing_authority", root, "missing required file")

        fixture(root)
        put(root, "AGENTS.md", "no pointer\n")
        fails("missing_agent_pointer", root, "authority pointer missing")

        fixture(root)
        blocker = root / "docs/architecture/cpb-next-roadmap-blockers.md"
        blocker.write_text(blocker.read_text() + "\n## Active owner-locked sequence\n")
        fails("duplicate_sequence", root, "exactly one owner")

        fixture(root)
        authority = root / ACTIVE
        authority.write_text(
            authority.read_text().replace(
                "1. `CPB-2 Callable and Function Type Closure`\n"
                "2. `CPB-2 Assignment and Return Compatibility`",
                "1. `CPB-2 Assignment and Return Compatibility`\n"
                "2. `CPB-2 Callable and Function Type Closure`",
            )
        )
        fails("reordered_sequence", root, "exact three-item list")

        fixture(root)
        put(root, "docs/architecture/semantic-lifecycle-status.md", "semantic evidence\n")
        fails("missing_evidence_pointer", root, "authority pointer missing")

        fixture(root)
        put(root, "docs/governance/architecture-change-control.md", "old size policy\n")
        fails("missing_documentation_size_policy", root, "governance size policy missing")

        fixture(root)
        put(root, "docs/architecture/code-lifecycle-inventory.md", "Add a lifecycle validator in SSM-D3\n")
        fails("stale_ssm_d3", root, "stale SSM-D3")

        fixture(root)
        put(root, "docs/architecture/cpb-next-roadmap-blockers.md", "merge stdlib-core foundation to main\ncurrent-authority.md\n")
        fails("stale_stdlib_merge", root, "stale stdlib-core")

        fixture(root)
        put(
            root,
            "docs/architecture/CONTRACT_EXTENSION_ARCHITECTURE.md",
            "current-authority.md\nRecommended order:\n",
        )
        fails("stale_contract_order", root, "stale contract implementation order")

        fixture(root)
        policy = root / ".github/ROADMAP_POLICY.md"
        policy.write_text(policy.read_text() + "Check ROADMAP.md.\n")
        fails("unqualified_roadmap", root, "unqualified ROADMAP.md")

        fixture(root)
        put(root, ".github/AUDIT_TEMPLATE.md", f"{AUTHORITY}\nCopy from ROADMAP.md.\n")
        fails("stale_audit_roadmap", root, "stale audit-template authority")

        fixture(root)
        put(root, ".github/AUDIT_TEMPLATE.md", f"{AUTHORITY}\nold v1.45 example\n")
        fails("stale_audit_version", root, "stale audit-template authority")

        fixture(root)
        put(root, "SPECIFICATION.md", "Authoritative current references: `README.md`, `examples/`, and `docs/architecture/`.\n")
        fails("generic_architecture_authority", root, "generic architecture authority")

    print("authority_docs_self_test=PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
