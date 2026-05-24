#!/usr/bin/env python3
"""Ensure the v1.30 architecture skeleton documentation is registered.

This helper is intentionally documentation-only. It verifies that the canonical
v1.30 architecture design document exists and makes the README documentation map
contain a stable reference to it. It does not modify compiler runtime files.
"""

from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "spec" / "v1.30-alpha" / "v130_architecture_design.md"
README = ROOT / "README.md"

README_ENTRY = (
    "| `spec/v1.30-alpha/v130_architecture_design.md` | Defines the "
    "documentation-first v1.30 architecture baseline with Rust skeleton "
    "structures for spans, HIR, type registry, struct layout, enum "
    "representation, FFI signatures, unsafe gatekeeping, semantic validation, "
    "and codegen contracts. |"
)

REQUIRED_HEADINGS = [
    "# Logicodex v1.30 Systems-Grade Architecture Baseline",
    "## 2. Source Span Skeleton",
    "## 4. HIR Lowering Skeleton",
    "## 5. TypeRegistry and TypeId Skeleton",
    "## 6. StructLayout and LayoutEngine Skeleton",
    "## 8. CallableSignature, ABI, and FFI Gatekeeping Skeleton",
    "## 9. Semantic Gatekeeper Skeleton",
    "## 10. Codegen Contract Skeleton",
]


def main() -> int:
    if not DOC.exists():
        raise SystemExit(f"missing documentation file: {DOC}")

    text = DOC.read_text(encoding="utf-8")
    missing = [heading for heading in REQUIRED_HEADINGS if heading not in text]
    if missing:
        raise SystemExit("missing v1.30 documentation headings: " + ", ".join(missing))

    readme = README.read_text(encoding="utf-8")
    if README_ENTRY not in readme:
        anchor = "| `v121_execution_design.md` | Captures the executable-logic and provenance design track for the v1.21-alpha milestone. |"
        if anchor not in readme:
            raise SystemExit("README documentation map anchor not found")
        readme = readme.replace(anchor, anchor + "\n" + README_ENTRY)
        README.write_text(readme, encoding="utf-8")
        print("updated README documentation map")
    else:
        print("README documentation map already references v1.30 architecture design")

    print(f"verified {DOC.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
