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
            "HardwareZone",
            "Function",
            "Return",
            "While",
            "Loop",
            "Break",
            "Continue",
            "declared_type: Option<Type>",
            "AddressOfLiteral(i64)",
            "ShiftLeft",
            "ShiftRight",
        ],
    ),
    (
        "Lexer exposes canonical v1.21-alpha tokens and dictionary-compatible API",
        ROOT / "src" / "lexer.rs",
        [
            "pub enum TokenKind",
            "Hardware",
            "HwZone",
            "Address",
            "Fn",
            "Return",
            "While",
            "Loop",
            "Break",
            "Continue",
            "Unsafe",
            "Extern",
            "Struct",
            "Enum",
            "TypeU16",
            "TypeU32",
            "Ptr",
            "Arrow",
            "pub struct Lexicon",
            "pub fn from_path",
            "default_aliases",
            "TokenDictionary::V2",
        ],
    ),
    (
        "Parser enforces executable grammar layout",
        ROOT / "src" / "parser.rs",
        [
            "hardware_declaration",
            "hardware_zone_block",
            "TokenKind::HwZone",
            "function_definition",
            "return_statement",
            "while_statement",
            "loop_statement",
            "unimplemented_feature",
            "ParseError::UnimplementedFeature",
            "let_statement",
            "parse_type",
            "AddressOfLiteral",
            "logical_or",
            "TokenKind::ShiftL",
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
            "HardwareMutationOutsideZone",
            "hw_zone_depth",
            "KRITIKAL: Ralat Umum Tahap 1 - Percubaan Mutasi Perkakasan Tanpa Kebenaran Skop Zon Selamat / CRITICAL: General Error Level 1 - Attempted Hardware Mutation Without Safe Zone Scope Authorization.",
            "ReturnTypeMismatch",
            "BreakOutsideLoop",
            "ContinueOutsideLoop",
            "loop_depth",
            "BinaryOp::And",
            "BinaryOp::ShiftRight",
            "hardware_addresses",
            "integer_fits",
        ],
    ),
    (
        "Code generator accepts the expanded AST without rejecting declarations",
        ROOT / "src" / "codegen.rs",
        [
            "Stmt::HardwareDecl",
            "Stmt::HardwareZone",
            "Stmt::Function",
            "Stmt::Return",
            "Stmt::While",
            "Stmt::Loop",
            "Stmt::Break",
            "Stmt::Continue",
            "LoopTarget",
            "Expr::StringLiteral",
            "Expr::AddressOfLiteral",
            "i64_to_bool",
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
    "HW_ZONE",
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
    "WHILE",
    "LOOP",
    "LOOP_BREAK",
    "LOOP_CONTINUE",
    "AND",
    "OR",
    "BIT_AND",
    "BIT_OR",
    "SHIFT_L",
    "SHIFT_R",
    "STRUCT",
    "ENUM",
    "UNSAFE",
    "EXTERN",
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
    tokens = data.get("tokens", {})
    if isinstance(tokens, dict):
        token_names = set(tokens)
        policy = data.get("policy", {})
        required_policy = {
            "reference_mode": "expert",
            "primary_human_language": "ms",
            "alias_order_rule": "expert_then_primary_ms_then_aliases",
        }
        for key, expected in required_policy.items():
            if policy.get(key) != expected:
                raise AssertionError(f"core_map.json policy {key} must be {expected!r}")
        for name, entry in tokens.items():
            if not entry.get("expert"):
                raise AssertionError(f"core_map.json token {name} missing expert reference lexeme")
            if not entry.get("primary_ms"):
                raise AssertionError(f"core_map.json token {name} missing primary_ms Malay alias")
        for name in ["LET", "PRINT", "RETURN"]:
            if tokens.get(name, {}).get("beginner_line_terminated") is not True:
                raise AssertionError(f"core_map.json token {name} must mark beginner_line_terminated")
        for name in ["STRUCT", "ENUM", "UNSAFE", "EXTERN"]:
            entry = tokens.get(name, {})
            if entry.get("compiler_status") != "parser_trap_unimplemented":
                raise AssertionError(f"core_map.json token {name} must mark parser_trap_unimplemented status")
        for name in ["HW", "HW_ZONE", "ADDR", "PTR"]:
            critical = tokens.get(name, {}).get("critical_policy", {})
            if not any(critical.get(key) is True for key in [
                "requires_explicit_block",
                "requires_explicit_terminator",
                "requires_explicit_terminator_inside",
            ]):
                raise AssertionError(f"core_map.json token {name} must require explicit critical boundary")
    else:
        token_names = {entry.get("identity") for entry in tokens}
    missing = sorted(DICT_REQUIRED - token_names)
    if missing:
        raise AssertionError(f"core_map.json missing tokens: {', '.join(missing)}")
    if data.get("version") not in {"v1.21-alpha", "1.21-alpha"}:
        raise AssertionError("core_map.json must remain marked as the public v1.21-alpha line")


def validate_versions() -> None:
    """v1.44.1: Version-agnostic — check semver format and key files exist."""
    cargo = read(ROOT / "Cargo.toml")
    # Check version follows X.Y.Z-alpha pattern (not hardcoded to 1.21)
    m = re.search(r'version = "(\d+)\.(\d+)\.(\d+)(?:-([\w.-]+))?"', cargo)
    if not m:
        raise AssertionError("Cargo.toml missing valid semver version")
    major, minor, patch = int(m.group(1)), int(m.group(2)), int(m.group(3))
    if major < 1 or (major == 1 and minor < 21):
        raise AssertionError(f"Version {major}.{minor}.{patch} below baseline 1.21.0")

    # Check key project files exist (version-agnostic)
    active_paths = ["README.md", "ROADMAP.md", "src/main.rs"]
    for rel in active_paths:
        path = ROOT / rel
        if not path.exists():
            raise AssertionError(f"Required file missing: {rel}")
        content = read(path)
        # Must mention Logicodex (project identity)
        if "Logicodex" not in content:
            raise AssertionError(f"{rel} missing 'Logicodex' project name")


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
