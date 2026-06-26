#!/usr/bin/env python3
"""
Test negative cases for stdlib contract validator.

Verifies that the validator correctly rejects bad contracts.
"""

from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path
from typing import NamedTuple


class TestCase(NamedTuple):
    name: str
    contract_path: str
    expected_error_regex: str


REPO_ROOT = Path(__file__).resolve().parents[1]
FIXTURES_ROOT = REPO_ROOT / "tests" / "fixtures" / "stdlib_contracts" / "negative"

TEST_CASES = [
    TestCase(
        name="unknown schema key",
        contract_path="unknown_key.std.toml",
        expected_error_regex="unknown keys",
    ),
    TestCase(
        name="missing required table",
        contract_path="missing_table.std.toml",
        expected_error_regex="missing keys.*capabilities",
    ),
    TestCase(
        name="declared export not public in source",
        contract_path="export_not_public.std.toml",
        expected_error_regex="exports missing from contract",
    ),
    TestCase(
        name="public export missing from contract",
        contract_path="missing_export.std.toml",
        expected_error_regex="exports missing from contract",
    ),
    TestCase(
        name="forbidden import in core module",
        contract_path="forbidden_import_test.std.toml",
        expected_error_regex="forbidden import",
    ),
    TestCase(
        name="bad case expect_i64 type",
        contract_path="bad_case_type.std.toml",
        expected_error_regex="expect_i64 must be integer",
    ),
]


def run_validator(contract_path: Path) -> tuple[int, str, str]:
    """Run the validator and return (returncode, stdout, stderr)."""
    cmd = [
        "python3",
        str(REPO_ROOT / "tools" / "verify_stdlib_contracts.py"),
        str(contract_path),
    ]
    result = subprocess.run(cmd, capture_output=True, text=True, cwd=REPO_ROOT)
    return result.returncode, result.stdout, result.stderr


def main() -> int:
    print("=== Stdlib Contract Validator — Negative Test Suite ===")
    print()

    passed = 0
    failed = 0

    for test in TEST_CASES:
        contract_path = FIXTURES_ROOT / test.contract_path
        if not contract_path.exists():
            print(f"❌ {test.name}: fixture not found at {contract_path}")
            failed += 1
            continue

        returncode, stdout, stderr = run_validator(contract_path)

        # Validator should fail (non-zero exit code)
        if returncode == 0:
            print(f"❌ {test.name}: validator should have failed but passed")
            print(f"   stdout: {stdout}")
            failed += 1
            continue

        # Check that error message matches expected regex
        combined_output = stdout + stderr
        if not re.search(test.expected_error_regex, combined_output, re.IGNORECASE):
            print(f"❌ {test.name}: expected error regex not found")
            print(f"   expected regex: {test.expected_error_regex}")
            print(f"   output: {combined_output}")
            failed += 1
            continue

        print(f"✓ {test.name}")
        passed += 1

    print()
    print(f"Results: {passed} passed, {failed} failed")

    if failed > 0:
        print("❌ Some tests failed")
        return 1

    print("✅ All negative tests passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
