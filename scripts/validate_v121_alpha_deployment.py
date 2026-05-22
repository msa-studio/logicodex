#!/usr/bin/env python3
from pathlib import Path
import sys

root = Path(__file__).resolve().parents[1]
expected_header = """// =========================================================================
// Project: Logicodex Language Engine (Phase 2 Deployment Integration)
// Version: v1.21-alpha (Formal Specifications & Zero-Overhead Severity Model)
// Architect & Creator: Mohamad Supardi Abdul (mymsastudio@gmail.com)
// Copyright (c) 2026. All Rights Reserved.
// Licensed under permissive dual-license: MIT & Apache License 2.0
// ========================================================================="""
errors = []
header_files = [
    *sorted((root / "src").rglob("*.rs")),
    *sorted((root / "examples").rglob("*.ldx")),
    *sorted((root / "stdlib").rglob("*.ldx")),
    root / "dict" / "core_map.json",
]
for path in header_files:
    text = path.read_text(encoding="utf-8")
    if not text.startswith(expected_header):
        errors.append(f"header mismatch: {path.relative_to(root)}")

checks = {
    "Cargo.toml": ["version = \"1.21-alpha\""],
    "src/main.rs": ["version = \"1.21-alpha\"", "LOGICODEX COMPILER v1.21-alpha", "logicodex 1.21-alpha", "Formal Specifications & Zero-Overhead Severity Model"],
    "README.md": ["v1.21-alpha Phase 2 Deployment Integration", "Undefined Behavior and Pointer Provenance model", "zero-overhead Critical/Medium/Low severity architecture"],
    "WHITE_PAPER.md": ["v1.21-alpha Specification Synchronization", "Undefined Behavior and Pointer Provenance specification", "Critical", "Medium", "Low"],
    "ROADMAP.md": ["Issue #01 | [X] COMPLETED / SOLVED | Mohamad Supardi Abdul", "Issue #02 | [X] COMPLETED / SOLVED | Mohamad Supardi Abdul", "Layered error modeling (C/C++/Rust) integrated into specification", "Zero-overhead 3-tier severity mitigation architecture"],
    "REPOS_CONTEXT.md": ["# ❖ Logicodex Repository Context Document", "v1.21-alpha milestone", "spec/v1.21-alpha/UpdateIssue2-provenance.md"],
    "spec/v1.21-alpha/UpdateIssue1-ebnf.md": ["# ❖ Logicodex Formal Grammar Specification (v1.21-alpha)", "StringLiteral    ::= '\"' [^\"\\\\]* '\"'", "HardwareToken    ::= \"KAWASAN_PERKAKAS\" | \"hw\"", "AddressToken     ::= \"ALAMAT\" | \"addr\""],
    "spec/v1.21-alpha/UpdateIssue2-provenance.md": ["# ❖ Logicodex Undefined Behavior & Pointer Provenance Specification (v1.21-alpha)", "Linear Layer (C-Style Paradigms)", "Object-Oriented Layer (C++ Style Paradigms)", "Safety Layer (Rust-Style Paradigms)", "TIER 1: CRITICAL", "TIER 2: MEDIUM", "TIER 3: LOW"],
    "scripts/update_release_archives.sh": ["NAME=\"logicodex-v1.21-alpha\""],
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
