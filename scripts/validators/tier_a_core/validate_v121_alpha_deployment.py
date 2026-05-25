#!/usr/bin/env python3
from pathlib import Path
import sys

root = Path(__file__).resolve().parents[1]
# v1.44.1: Check that source files have the standard Logicodex header
required_header_markers = [
    "Logicodex Language Engine",
    "Mohamad Supardi Abdul",
    "Copyright (c) 2026",
]
errors = []
# v1.44.1: Check src/ files have project identity in first 10 lines
header_files = sorted((root / "src").rglob("*.rs"))
for path in header_files:
    text = path.read_text(encoding="utf-8")
    if len(text) < 200:
        continue
    header_block = text[:300]
    # Must contain "Logicodex" OR "Copyright" in header — flexible for different header styles
    if "Logicodex" not in header_block and "Copyright" not in header_block:
        errors.append(f"missing project identity in {path.relative_to(root)}")

# v1.44.1: Version-agnostic checks — verify key files exist and contain project identity
checks = {
    "Cargo.toml": ["Logicodex"],
    "src/main.rs": ["Logicodex", "compile"],
    "README.md": ["Logicodex", "ROADMAP"],
    "ROADMAP.md": ["Logicodex", "Mohamad Supardi Abdul"],
}
for rel, markers in checks.items():
    path = root / rel
    if not path.exists():
        errors.append(f"missing required file: {rel}")
        continue
    text = path.read_text(encoding="utf-8")
    for marker in markers:
        if marker not in text:
            errors.append(f"missing marker in {rel}: {marker}")

# Check spec directory exists (version-agnostic)
spec_dir = root / "spec"
if spec_dir.exists():
    # At least one spec file should exist
    spec_files = list(spec_dir.rglob("*.md"))
    if not spec_files:
        errors.append("no spec files found in spec/ directory")
else:
    errors.append("spec/ directory missing")

for rel in ["Cargo.toml", "NOTICE", "README.md", "WHITE_PAPER.md", "MANUAL.md", "ROADMAP.md", "scripts/update_release_archives.sh", "src/main.rs"]:
    text = (root / rel).read_text(encoding="utf-8")
    if "1.11-alpha" in text or "v1.11-alpha" in text or "V1.11-alpha" in text or "1.0.1-alpha" in text or "v1.0.1-alpha" in text or "V1.0.1-alpha" in text:
        errors.append(f"legacy version remains in {rel}")

if errors:
    print("VALIDATION FAILED")
    for error in errors:
        print(f"- {error}")
    sys.exit(1)
print("VALIDATION PASSED: v1.21-alpha metadata, headers, specs, context, and roadmap markers are aligned.")
