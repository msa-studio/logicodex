#!/usr/bin/env python3
"""
Verify Logicodex stdlib Stage 0 contract sidecars.

This is a dev/CI validation tool only.
It does not participate in normal compile, HIR, codegen, semantic analysis,
runtime profile selection, or capability enforcement.
"""

from __future__ import annotations

import argparse
import re
import sys
import tomllib
from pathlib import Path
from typing import Any


REPO_ROOT = Path(__file__).resolve().parents[1]

EXPECTED_TOP_LEVEL = {
    "contract",
    "module",
    "validation",
    "limits",
    "exports",
    "constraints",
    "capabilities",
    "cases",
}

EXPECTED_TABLE_KEYS = {
    "contract": {"version"},
    "module": {"name", "layer", "stage", "profile", "pure", "extern"},
    "validation": {"metadata", "static", "compile", "run", "stress"},
    "limits": {
        "compile_timeout_ms",
        "run_timeout_ms",
        "stdout_limit_bytes",
        "max_cases",
    },
    "exports": {"functions"},
    "constraints": {"allowed_imports", "forbidden_imports", "forbidden_features"},
    "capabilities": {"requires"},
}

EXPECTED_CASE_KEYS = {"name", "expr", "expect_i64"}

OFFICIAL_LAYERS = {"core", "std", "framework"}

CORE_FORBIDDEN_IMPORTS = {"std", "framework"}
CORE_FORBIDDEN_FEATURES = {"extern", "malloc", "free", "file", "network", "syscall"}


class ContractError(Exception):
    pass


def strip_line_comments(source: str) -> str:
    return "\n".join(line.split("//", 1)[0] for line in source.splitlines())


def load_toml(path: Path) -> dict[str, Any]:
    try:
        return tomllib.loads(path.read_text(encoding="utf-8"))
    except tomllib.TOMLDecodeError as exc:
        raise ContractError(f"{path}: invalid TOML: {exc}") from exc


def require_table(data: dict[str, Any], table: str) -> dict[str, Any]:
    value = data.get(table)
    if not isinstance(value, dict):
        raise ContractError(f"missing or invalid [{table}] table")
    return value


def assert_exact_keys(label: str, actual: set[str], expected: set[str]) -> None:
    extra = sorted(actual - expected)
    missing = sorted(expected - actual)
    if extra:
        raise ContractError(f"{label}: unknown keys: {extra}")
    if missing:
        raise ContractError(f"{label}: missing keys: {missing}")


def public_functions(source: str) -> set[str]:
    clean = strip_line_comments(source)
    return set(re.findall(r"\bpublic\s+function\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(", clean))


def imported_modules(source: str) -> list[str]:
    clean = strip_line_comments(source)
    return re.findall(r"\bimport\s+([A-Za-z_][A-Za-z0-9_]*(?:\.[A-Za-z_][A-Za-z0-9_]*)*)\s*;", clean)


def contains_forbidden_feature(source: str, feature: str) -> bool:
    clean = strip_line_comments(source)
    pattern = rf"\b{re.escape(feature)}\b"
    return re.search(pattern, clean) is not None


def contract_to_source_path(contract_path: Path, module_name: str) -> Path:
    layer = module_name.split(".", 1)[0]
    module_leaf = module_name.split(".")[-1]
    return contract_path.with_name(f"{module_leaf}.ldx")


def validate_contract(path: Path) -> None:
    data = load_toml(path)

    assert_exact_keys("top-level", set(data.keys()), EXPECTED_TOP_LEVEL)

    for table, expected in EXPECTED_TABLE_KEYS.items():
        actual_table = require_table(data, table)
        assert_exact_keys(f"[{table}]", set(actual_table.keys()), expected)

    cases = data.get("cases")
    if not isinstance(cases, list):
        raise ContractError("[[cases]] must exist as a list")
    for index, case in enumerate(cases):
        if not isinstance(case, dict):
            raise ContractError(f"case #{index}: must be a table")
        assert_exact_keys(f"case #{index}", set(case.keys()), EXPECTED_CASE_KEYS)

    contract = data["contract"]
    module = data["module"]
    validation = data["validation"]
    limits = data["limits"]
    exports = data["exports"]
    constraints = data["constraints"]
    capabilities = data["capabilities"]

    if contract["version"] != 0:
        raise ContractError("contract.version must be 0 for Stage 0")

    module_name = module["name"]
    layer = module["layer"]

    if layer not in OFFICIAL_LAYERS:
        raise ContractError(f"invalid module.layer: {layer!r}")

    if not isinstance(module_name, str) or not module_name.startswith(f"{layer}."):
        raise ContractError("module.name must start with '<layer>.'")

    if module["stage"] != 0:
        raise ContractError("module.stage must be 0 for Stage 0")

    if not isinstance(exports["functions"], list) or not all(isinstance(x, str) for x in exports["functions"]):
        raise ContractError("exports.functions must be a list of strings")

    if not isinstance(capabilities["requires"], list):
        raise ContractError("capabilities.requires must be a list")

    max_cases = limits["max_cases"]
    if len(cases) > max_cases:
        raise ContractError(f"case count {len(cases)} exceeds limits.max_cases {max_cases}")

    source_path = contract_to_source_path(path, module_name)
    if not source_path.exists():
        raise ContractError(f"source module missing: {source_path}")

    source = source_path.read_text(encoding="utf-8")

    actual_exports = public_functions(source)
    declared_exports = set(exports["functions"])

    missing = sorted(actual_exports - declared_exports)
    extra = sorted(declared_exports - actual_exports)
    if missing:
        raise ContractError(f"exports missing from contract: {missing}")
    if extra:
        raise ContractError(f"exports declared but not public in source: {extra}")

    imports = imported_modules(source)
    forbidden_imports = constraints["forbidden_imports"]

    for imported in imports:
        for forbidden in forbidden_imports:
            prefix = forbidden.removesuffix(".*")
            if imported == prefix or imported.startswith(f"{prefix}."):
                raise ContractError(f"forbidden import {imported!r} matched {forbidden!r}")

    if layer == "core":
        if module["profile"] != "core":
            raise ContractError("core.* modules must have profile = \"core\"")
        if module["pure"] is not True:
            raise ContractError("core.* modules must have pure = true")
        if module["extern"] is not False:
            raise ContractError("core.* modules must have extern = false")
        if set(forbidden_imports) < {"std.*", "framework.*"}:
            raise ContractError("core.* modules must forbid std.* and framework.*")
        if capabilities["requires"] != []:
            raise ContractError("core.* Stage 0 modules must require no capabilities")

    for feature in constraints["forbidden_features"]:
        if feature in CORE_FORBIDDEN_FEATURES and contains_forbidden_feature(source, feature):
            raise ContractError(f"forbidden feature token found in source: {feature}")

    for case in cases:
        if not isinstance(case["name"], str) or not case["name"]:
            raise ContractError("case.name must be non-empty string")
        if not isinstance(case["expr"], str) or not case["expr"].startswith(f"{module_name}."):
            raise ContractError(f"case.expr must start with {module_name}.")
        if not isinstance(case["expect_i64"], int):
            raise ContractError("case.expect_i64 must be integer")

    print(f"OK {path.relative_to(REPO_ROOT)}")
    print(f"  module: {module_name}")
    print(f"  exports: {', '.join(sorted(declared_exports))}")
    print(f"  cases: {len(cases)}")


def main() -> int:
    parser = argparse.ArgumentParser(description="Verify Logicodex stdlib .std.toml contracts")
    parser.add_argument(
        "contracts",
        nargs="*",
        help="Contract files to verify. Defaults to lib/**/*.std.toml",
    )
    args = parser.parse_args()

    if args.contracts:
        paths = [Path(p) for p in args.contracts]
    else:
        paths = sorted((REPO_ROOT / "lib").glob("**/*.std.toml"))

    if not paths:
        print("No stdlib contract files found", file=sys.stderr)
        return 1

    failed = False
    for path in paths:
        path = path.resolve()
        try:
            validate_contract(path)
        except ContractError as exc:
            failed = True
            print(f"FAIL {path.relative_to(REPO_ROOT)}: {exc}", file=sys.stderr)

    return 1 if failed else 0


if __name__ == "__main__":
    raise SystemExit(main())
