#!/usr/bin/env python3
"""Verify that Rust thiserror diagnostics in src/ and embedded generator templates are bilingual.

The check accepts formatted single-line or multi-line #[error(...)] attributes and
requires the diagnostic payload to contain the bilingual separator " / ".
"""
from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCOPES = [ROOT / "src", ROOT / "scripts"]
PATTERN = re.compile(r"#\[error\((.*?)\)\]", re.DOTALL)

failures: list[str] = []
for scope in SCOPES:
    for path in sorted(scope.rglob("*")):
        if path.suffix not in {".rs", ".py"} or not path.is_file():
            continue
        if path.name == "check_bilingual_error_annotations.py":
            continue
        text = path.read_text(encoding="utf-8")
        for match in PATTERN.finditer(text):
            payload = " ".join(match.group(1).split())
            if " / " not in payload:
                line = text.count("\n", 0, match.start()) + 1
                failures.append(f"{path.relative_to(ROOT)}:{line}: {payload}")

if failures:
    print("Non-bilingual #[error(...)] diagnostics found:")
    for item in failures:
        print(item)
    raise SystemExit(1)

print("PASS: all #[error(...)] diagnostics contain Malay / English bilingual text")
