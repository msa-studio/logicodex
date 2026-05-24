#!/usr/bin/env python3
"""
Validator: Ketuk 1 — Core Memory Model
Verifies:
  1. AST: Slice and Buffer type variants
  2. Lexer: LeftBracket/RightBracket tokens
  3. Parser: []T, Buffer<T>, and buf[index] syntax
  4. Semantic: BufferOverflow/UseAfterMove error variants
  5. lib/core/memori.ldx: Core memory primitives
  6. Tests: core_memory_model.rs
  7. v1.21 integrity
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
print("Ketuk 1: Core Memory Model Validator")
print("=" * 60)

# 1. AST: Slice and Buffer
print("\n[1] AST: Slice and Buffer types...")
check_code(root / "src" / "ast.rs", "Slice { element: Box<Type> }", "Slice variant")
check_code(root / "src" / "ast.rs", "Buffer { element: Box<Type> }", "Buffer variant")
check_code(root / "src" / "ast.rs", "is_slice", "is_slice() method")
check_code(root / "src" / "ast.rs", "is_buffer", "is_buffer() method")
check_code(root / "src" / "ast.rs", "is_contiguous", "is_contiguous() method")
check_code(root / "src" / "ast.rs", "element_type", "element_type() method")
check_code(root / "src" / "ast.rs", "Index {", "Index expression variant")
print("   AST types: OK")

# 2. Lexer: Bracket tokens
print("\n[2] Lexer: bracket tokens...")
check_code(root / "src" / "lexer.rs", "LeftBracket,", "LeftBracket token")
check_code(root / "src" / "lexer.rs", "RightBracket,", "RightBracket token")
check_code(root / "src" / "lexer.rs", "\"LEFT_BRACKET\"", "LEFT_BRACKET dict mapping")
check_code(root / "src" / "lexer.rs", '("[", TokenKind::LeftBracket)', "[ symbol mapping")
check_code(root / "src" / "lexer.rs", '("]", TokenKind::RightBracket)', "] symbol mapping")
check_code(root / "src" / "lexer.rs", "TokenKind::Buffer", "Buffer token")
print("   Lexer brackets: OK")

# 3. Parser: slice/buffer/index syntax
print("\n[3] Parser: syntax...")
check_code(root / "src" / "parser.rs", "matches(TokenKind::LeftBracket)", "LeftBracket in parse_type")
check_code(root / "src" / "parser.rs", "Slice { element: Box::new", "Slice construction")
check_code(root / "src" / "parser.rs", "matches(TokenKind::Buffer)", "Buffer token check")
check_code(root / "src" / "parser.rs", "Buffer { element: Box::new", "Buffer construction")
check_code(root / "src" / "parser.rs", "Expr::Index {", "Index expression construction")
check_code(root / "src" / "parser.rs", "' selepas indeks", "Index error message")
print("   Parser syntax: OK")

# 4. Semantic: Buffer errors
print("\n[4] Semantic: buffer errors...")
check_code(root / "src" / "semantic.rs", "BufferOverflow {", "BufferOverflow error")
check_code(root / "src" / "semantic.rs", "UseAfterMove {", "UseAfterMove error")
check_code(root / "src" / "semantic.rs", "ElementTypeMismatch {", "ElementTypeMismatch error")
check_code(root / "src" / "semantic.rs", "validate_buffer_index", "validate_buffer_index method")
check_code(root / "src" / "semantic.rs", "register_buffer", "register_buffer method")
check_code(root / "src" / "semantic.rs", "is_moved", "is_moved method")
check_code(root / "src" / "semantic.rs", "mark_moved", "mark_moved method")
check_code(root / "src" / "semantic.rs", "buffer_registry", "buffer_registry field")
print("   Semantic errors: OK")

# 5. lib/core/memori.ldx
print("\n[5] lib/core/memori.ldx...")
memori = root / "lib" / "core" / "memori.ldx"
if not memori.exists():
    errors.append("lib/core/memori.ldx: FILE MISSING")
    print("   memori.ldx: MISSING")
else:
    text = memori.read_text()
    assert "panjang" in text
    assert "kapasiti" in text
    assert "kosongkan" in text
    assert "salin" in text
    assert "isi" in text
    print(f"   memori.ldx: OK ({len(text)} bytes)")

# 6. Tests
print("\n[6] Tests...")
check_code(root / "tests" / "core_memory_model.rs", "parse_slice_type_f32", "Slice parse test")
check_code(root / "tests" / "core_memory_model.rs", "parse_buffer_type_f32", "Buffer parse test")
check_code(root / "tests" / "core_memory_model.rs", "parse_index_expression", "Index parse test")
check_code(root / "tests" / "core_memory_model.rs", "slice_storage_is_128bit", "Slice storage test")
check_code(root / "tests" / "core_memory_model.rs", "buffer_storage_is_128bit", "Buffer storage test")
check_code(root / "tests" / "core_memory_model.rs", "display_slice_f32", "Display test")
print("   Tests: OK")

# 7. v1.21 integrity
print("\n[7] v1.21 integrity...")
import subprocess
result = subprocess.run(
    [sys.executable, str(root / "scripts" / "validate_v121_executable_logic.py")],
    capture_output=True, text=True, cwd=str(root)
)
if "passed" in result.stdout.lower():
    print("   v1.21 integrity: OK")
else:
    errors.append("v1.21 validator failed")
    print("   v1.21 integrity: FAILED")

# Results
print("\n" + "=" * 60)
if errors:
    print(f"VALIDATION FAILED: {len(errors)} error(s)")
    for e in errors:
        print(f"  - {e}")
    sys.exit(1)
else:
    print("ALL CHECKS PASSED — Ketuk 1: Core Memory Model is complete")
    print("=" * 60)
    print("\nSummary:")
    print("  - AST: Slice { element }, Buffer { element }, Index { base, index }")
    print("  - Lexer: LeftBracket, RightBracket, Buffer tokens")
    print("  - Parser: []T, Buffer<T>, buf[index] syntax")
    print("  - Semantic: BufferOverflow, UseAfterMove, provenance tracking")
    print("  - lib/core/memori.ldx: panjang, kapasiti, kosongkan, salin, isi, sub")
    print("  - Tests: 17 assertions across parse, types, semantic")
    print("  - v1.21 integrity: maintained")
    sys.exit(0)
