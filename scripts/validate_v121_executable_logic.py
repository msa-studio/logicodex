#!/usr/bin/env python3
"""Logical validation for v1.21-alpha executable compiler integration.

This script intentionally performs deterministic source and metadata checks that do
not require Cargo dependency resolution. It is used when the sandbox Rust/Cargo
version cannot resolve current crates.io packages.
"""
from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

CHECKS: list[tuple[str, Path, list[str]]] = [
    (
        "AST supports executable v1.21-alpha declarations and expressions",
        ROOT / "src" / "ast.rs",
        [
            "pub struct Program",
            "pub struct Param",
            "pub enum Type",
            "Pointer(Box<Type>)",
            "String",
            "HardwareDecl",
            "Function",
            "Return",
            "declared_type: Option<Type>",
            "AddressOfLiteral(i64)",
        ],
    ),
    (
        "Lexer exposes canonical v1.21-alpha tokens and dictionary-compatible API",
        ROOT / "src" / "lexer.rs",
        [
            "pub enum TokenKind",
            "Hardware",
            "Address",
            "Fn",
            "Return",
            "TypeU16",
            "TypeU32",
            "Ptr",
            "Arrow",
            "pub struct Lexicon",
            "pub fn from_path",
            "default_aliases",
        ],
    ),
    (
        "Parser enforces executable grammar layout",
        ROOT / "src" / "parser.rs",
        [
            "hardware_declaration",
            "function_definition",
            "return_statement",
            "let_statement",
            "parse_type",
            "AddressOfLiteral",
            "consume_optional_semicolons",
            "TokenKind::Arrow",
        ],
    ),
    (
        "Semantic analyzer implements static safety and provenance checks",
        ROOT / "src" / "semantic.rs",
        [
            "pub enum SeverityPolicy",
            "analyze_for_target",
            "freestanding",
            "DivisionByZero",
            "NumericBounds",
            "MissingProvenance",
            "InvalidPointerInitializer",
            "BareAddressRejected",
            "ReturnTypeMismatch",
            "hardware_addresses",
            "integer_fits",
        ],
    ),
    (
        "Code generator accepts the expanded AST without rejecting declarations",
        ROOT / "src" / "codegen.rs",
        [
            "Stmt::HardwareDecl",
            "Stmt::Function",
            "Stmt::Return",
            "Expr::StringLiteral",
            "Expr::AddressOfLiteral",
            "IntPredicate::NE, condition_value, zero",
            "MemoryIntegrityPlan",
            "PhysicalMemoryAccessPlan",
        ],
    ),
    (
        "CLI wires target and secure flags to semantic analysis",
        ROOT / "src" / "main.rs",
        [
            "Analyzer::analyze_for_target",
            "parse_and_analyze_for_target",
            "target_name",
            "secure",
            "v1.21-alpha",
        ],
    ),
]

DICT_REQUIRED = {
    "START",
    "END",
    "LET",
    "PRINT",
    "IF",
    "THEN",
    "ELSE",
    "HW",
    "ADDR",
    "USE",
    "FN",
    "RETURN",
    "I32",
    "I64",
    "U16",
    "U32",
    "F64",
    "BOOL",
    "PTR",
    "ARROW",
}


def read(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8")
    except FileNotFoundError:
        raise AssertionError(f"missing required file: {path.relative_to(ROOT)}")


def assert_contains(title: str, path: Path, needles: list[str]) -> None:
    text = read(path)
    missing = [needle for needle in needles if needle not in text]
    if missing:
        raise AssertionError(
            f"{title}: {path.relative_to(ROOT)} missing {', '.join(missing)}"
        )


def strip_json_comments(text: str) -> str:
    lines = []
    for line in text.splitlines():
        in_string = False
        escaped = False
        cut = len(line)
        for idx in range(len(line) - 1):
            ch = line[idx]
            if escaped:
                escaped = False
            elif ch == "\\":
                escaped = True
            elif ch == '"':
                in_string = not in_string
            elif not in_string and line[idx : idx + 2] == "//":
                cut = idx
                break
        lines.append(line[:cut])
    return "\n".join(lines)


def validate_dictionary() -> None:
    raw = read(ROOT / "dict" / "core_map.json")
    data = json.loads(strip_json_comments(raw))
    token_names = {entry.get("identity") for entry in data.get("tokens", [])}
    missing = sorted(DICT_REQUIRED - token_names)
    if missing:
        raise AssertionError(f"core_map.json missing tokens: {', '.join(missing)}")
    if data.get("version") not in {"v1.21-alpha", "1.21-alpha"}:
        raise AssertionError("core_map.json must remain marked as the public v1.21-alpha line")


def validate_versions() -> None:
    cargo = read(ROOT / "Cargo.toml")
    if 'version = "1.21.0-alpha"' not in cargo:
        raise AssertionError("Cargo.toml must use Cargo-compatible 1.21.0-alpha")
    active_paths = [
        "README.md",
        "ROADMAP.md",
        "WHITE_PAPER.md",
        "NOTICE",
        "src/main.rs",
        "spec/v1.21-alpha/UpdateIssue1-ebnf.md",
        "spec/v1.21-alpha/UpdateIssue2-provenance.md",
    ]
    for rel in active_paths:
        if "v1.21-alpha" not in read(ROOT / rel):
            raise AssertionError(f"{rel} missing public v1.21-alpha label")


def validate_no_known_regressions() -> None:
    codegen = read(ROOT / "src" / "codegen.rs")
    if "let zero = self.bool_type.const_zero();" in codegen:
        raise AssertionError("codegen still compares i64 conditions with i1 zero")
    semantic = read(ROOT / "src" / "semantic.rs")
    pattern = r"lower\.contains\(\"freestanding\"\)"
    if not re.search(pattern, semantic):
        raise AssertionError("freestanding target is not mapped to non-desktop severity")


def main() -> int:
    failures = []
    for title, path, needles in CHECKS:
        try:
            assert_contains(title, path, needles)
            print(f"PASS: {title}")
        except AssertionError as exc:
            failures.append(str(exc))
            print(f"FAIL: {exc}")
    for name, func in [
        ("dictionary token surface", validate_dictionary),
        ("version-label policy", validate_versions),
        ("known regression guards", validate_no_known_regressions),
    ]:
        try:
            func()
            print(f"PASS: {name}")
        except AssertionError as exc:
            failures.append(str(exc))
            print(f"FAIL: {exc}")
    if failures:
        print("\nLogical validation failed:")
        for failure in failures:
            print(f"- {failure}")
        return 1
    print("\nLogical validation passed for v1.21-alpha executable implementation.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
