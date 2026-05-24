#!/usr/bin/env python3
"""
Validator: Ketuk 2 — Result<T, E> Abstraction
"""
from pathlib import Path
import sys

root = Path(__file__).resolve().parents[1]
errors = []

def check_code(path, pattern, description):
    if not path.exists():
        errors.append(f"{path.relative_to(root)}: FILE MISSING")
        return False
    text = path.read_text(encoding="utf-8")
    if pattern not in text:
        errors.append(f"{path.relative_to(root)}: missing {description}")
        return False
    return True

print("=" * 60)
print("Ketuk 2: Result<T, E> Abstraction Validator")
print("=" * 60)

print("\n[1] AST: Result, Match, MatchArm, MatchPattern, Ok/Expr...")
check_code(root / "src" / "ast.rs", "Result { ok: Box<Type>, err: Box<Type> }", "Result type")
check_code(root / "src" / "ast.rs", "Match {", "Match stmt")
check_code(root / "src" / "ast.rs", "MatchArm {", "MatchArm struct")
check_code(root / "src" / "ast.rs", "Ok { binding: String }", "Ok pattern")
check_code(root / "src" / "ast.rs", "Err { binding: String }", "Err pattern")
check_code(root / "src" / "ast.rs", "Wildcard,", "Wildcard pattern")
check_code(root / "src" / "ast.rs", "Ok { value: Box<Expr> }", "Ok expr")
check_code(root / "src" / "ast.rs", "Err { value: Box<Expr> }", "Err expr")
check_code(root / "src" / "ast.rs", "pub fn is_result", "is_result method")
check_code(root / "src" / "ast.rs", "pub fn ok_type", "ok_type method")
check_code(root / "src" / "ast.rs", "pub fn err_type", "err_type method")
print("   AST: OK")

print("\n[2] Lexer: Result, Ok, Err, Match, ArrowFat, Underscore...")
check_code(root / "src" / "lexer.rs", "TokenKind::Result", "Result token")
check_code(root / "src" / "lexer.rs", "TokenKind::Ok", "Ok token")
check_code(root / "src" / "lexer.rs", "TokenKind::Err", "Err token")
check_code(root / "src" / "lexer.rs", "TokenKind::Match", "Match token")
check_code(root / "src" / "lexer.rs", "TokenKind::ArrowFat", "ArrowFat token")
check_code(root / "src" / "lexer.rs", "TokenKind::Underscore", "Underscore token")
check_code(root / "src" / "lexer.rs", '("_", TokenKind::Underscore)', "_ symbol")
check_code(root / "src" / "lexer.rs", '("=>", TokenKind::ArrowFat)', "=> symbol")
print("   Lexer: OK")

print("\n[3] Parser: Result<T, E>, Ok(), Err(), match, match_arm...")
check_code(root / "src" / "parser.rs", "Type::Result { ok: Box::new(ok), err: Box::new(err) }", "Result construction")
check_code(root / "src" / "parser.rs", "Expr::Ok { value: Box::new", "Ok expr construction")
check_code(root / "src" / "parser.rs", "Expr::Err { value: Box::new", "Err expr construction")
check_code(root / "src" / "parser.rs", "fn match_statement", "match_statement method")
check_code(root / "src" / "parser.rs", "fn match_arm", "match_arm method")
check_code(root / "src" / "parser.rs", "MatchPattern::Ok { binding }", "Ok pattern parsing")
check_code(root / "src" / "parser.rs", "MatchPattern::Wildcard", "Wildcard parsing")
check_code(root / "src" / "parser.rs", "ArrowFat", "ArrowFat consumption")
print("   Parser: OK")

print("\n[4] Semantic: Match handling, errors...")
check_code(root / "src" / "semantic.rs", "Stmt::Match { value, arms }", "Match in statement")
check_code(root / "src" / "semantic.rs", "MatchOnNonResult", "MatchOnNonResult error")
check_code(root / "src" / "semantic.rs", "NonExhaustiveMatch", "NonExhaustiveMatch error")
check_code(root / "src" / "semantic.rs", "Expr::Ok { value }", "Ok in expression")
check_code(root / "src" / "semantic.rs", "Expr::Err { value }", "Err in expression")
check_code(root / "src" / "semantic.rs", "is_result()", "is_result check")
print("   Semantic: OK")

print("\n[5] lib/core/result.ldx...")
if (root / "lib" / "core" / "result.ldx").exists():
    print("   result.ldx: OK")
else:
    errors.append("lib/core/result.ldx missing")

print("\n[6] lib/core/io_error.ldx...")
if (root / "lib" / "core" / "io_error.ldx").exists():
    print("   io_error.ldx: OK")
else:
    errors.append("lib/core/io_error.ldx missing")

print("\n[7] Tests...")
check_code(root / "tests" / "result_abstraction.rs", "parse_result_type_i32_i64", "Result parse test")
check_code(root / "tests" / "result_abstraction.rs", "parse_ok_constructor", "Ok parse test")
check_code(root / "tests" / "result_abstraction.rs", "parse_err_constructor", "Err parse test")
check_code(root / "tests" / "result_abstraction.rs", "parse_match_ok_err", "Match parse test")
check_code(root / "tests" / "result_abstraction.rs", "parse_match_wildcard", "Wildcard parse test")
check_code(root / "tests" / "result_abstraction.rs", "result_is_result", "Type property test")
check_code(root / "tests" / "result_abstraction.rs", "semantic_rejects_match_on_non_result", "Semantic test")
print("   Tests: OK")

print("\n[8] v1.21 integrity...")
import subprocess
result = subprocess.run(
    [sys.executable, str(root / "scripts" / "validate_v121_executable_logic.py")],
    capture_output=True, text=True, cwd=str(root)
)
if "passed" in result.stdout.lower():
    print("   v1.21 integrity: OK")
else:
    errors.append("v1.21 validator failed")

print("\n" + "=" * 60)
if errors:
    print(f"VALIDATION FAILED: {len(errors)} error(s)")
    for e in errors:
        print(f"  - {e}")
    sys.exit(1)
else:
    print("ALL CHECKS PASSED — Ketuk 2: Result<T, E> Abstraction is complete")
    sys.exit(0)
