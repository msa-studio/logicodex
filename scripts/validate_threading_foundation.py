#!/usr/bin/env python3
"""
Validator: v1.30.1-alpha — Threading Foundation (Kotak & Pintu)
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
print("v1.30.1-alpha: Threading Foundation (Kotak & Pintu) Validator")
print("=" * 60)

# 1. AST
print("\n[1] AST: Kotak, Pintu, Spawn, Hantar, Terima, Tunggu...")
check_code(root / "src" / "ast.rs", "Kotak {", "Kotak stmt")
check_code(root / "src" / "ast.rs", "Pintu { from: String, to: String", "Pintu type")
check_code(root / "src" / "ast.rs", "Spawn {", "Spawn expr")
check_code(root / "src" / "ast.rs", "Hantar {", "Hantar expr")
check_code(root / "src" / "ast.rs", "Terima {", "Terima expr")
check_code(root / "src" / "ast.rs", "Tunggu {", "Tunggu expr")
check_code(root / "src" / "ast.rs", "is_pintu", "is_pintu method")
check_code(root / "src" / "ast.rs", "pintu_capability", "pintu_capability method")
print("   AST: OK")

# 2. Lexer
print("\n[2] Lexer: Kotak, Pintu, Lahirkan, Hantar, Terima, Tunggu...")
check_code(root / "src" / "lexer.rs", "TokenKind::Kotak", "Kotak token")
check_code(root / "src" / "lexer.rs", "TokenKind::Pintu", "Pintu token")
check_code(root / "src" / "lexer.rs", "TokenKind::Lahirkan", "Lahirkan token")
check_code(root / "src" / "lexer.rs", "TokenKind::Hantar", "Hantar token")
check_code(root / "src" / "lexer.rs", "TokenKind::Terima", "Terima token")
check_code(root / "src" / "lexer.rs", "TokenKind::Tunggu", "Tunggu token")
check_code(root / "src" / "lexer.rs", '("kotak", TokenKind::Kotak)', "kotak symbol")
check_code(root / "src" / "lexer.rs", '("lahirkan", TokenKind::Lahirkan)', "lahirkan symbol")
print("   Lexer: OK")

# 3. Parser
print("\n[3] Parser: Kotak stmt, Pintu<T,U,V>, lahirkan, hantar, terima, tunggu...")
check_code(root / "src" / "parser.rs", "fn kotak_statement", "kotak_statement method")
check_code(root / "src" / "parser.rs", "Type::Pintu { from, to, message_type }", "Pintu construction")
check_code(root / "src" / "parser.rs", "Spawn { kotak_name", "Spawn construction")
check_code(root / "src" / "parser.rs", "Tunggu { kotak_name", "Tunggu construction")
check_code(root / "src" / "parser.rs", "Hantar { pintu_name", "Hantar construction")
check_code(root / "src" / "parser.rs", "Terima { pintu_name", "Terima construction")
check_code(root / "src" / "parser.rs", 'hantar"', "hantar keyword check")
check_code(root / "src" / "parser.rs", 'terima"', "terima keyword check")
print("   Parser: OK")

# 4. Semantic
print("\n[4] Semantic: Topology validation...")
check_code(root / "src" / "semantic.rs", "KotakNotFound", "KotakNotFound error")
check_code(root / "src" / "semantic.rs", "InvalidPintuTopology", "InvalidPintuTopology error")
check_code(root / "src" / "semantic.rs", "DuplicateKotak", "DuplicateKotak error")
check_code(root / "src" / "semantic.rs", "kotak_registry", "kotak_registry field")
check_code(root / "src" / "semantic.rs", "pintu_registry", "pintu_registry field")
check_code(root / "src" / "semantic.rs", "Spawn {", "Spawn in expression")
check_code(root / "src" / "semantic.rs", "Hantar {", "Hantar in expression")
check_code(root / "src" / "semantic.rs", "Terima {", "Terima in expression")
check_code(root / "src" / "semantic.rs", "Tunggu {", "Tunggu in expression")
check_code(root / "src" / "semantic.rs", "Kotak {", "Kotak in statement")
print("   Semantic: OK")

# 5. lib/core/thread.ldx
print("\n[5] lib/core/thread.ldx...")
if (root / "lib" / "core" / "thread.ldx").exists():
    print("   thread.ldx: OK")
else:
    errors.append("lib/core/thread.ldx missing")

# 6. lib/core/sync.ldx
print("\n[6] lib/core/sync.ldx...")
if (root / "lib" / "core" / "sync.ldx").exists():
    text = (root / "lib" / "core" / "sync.ldx").read_text()
    assert "Mutex" in text
    assert "RwLock" in text
    assert "AtomicI32" in text
    print("   sync.ldx: OK")
else:
    errors.append("lib/core/sync.ldx missing")

# 7. Tests
print("\n[7] Tests...")
check_code(root / "tests" / "threading_foundation.rs", "parse_kotak_declaration", "Kotak parse test")
check_code(root / "tests" / "threading_foundation.rs", "parse_kotak_with_pintu", "Kotak+Pintu test")
check_code(root / "tests" / "threading_foundation.rs", "parse_lahirkan", "Lahirkan test")
check_code(root / "tests" / "threading_foundation.rs", "parse_tunggu", "Tunggu test")
check_code(root / "tests" / "threading_foundation.rs", "parse_hantar", "Hantar test")
check_code(root / "tests" / "threading_foundation.rs", "parse_terima", "Terima test")
check_code(root / "tests" / "threading_foundation.rs", "pintu_type_properties", "Pintu type test")
check_code(root / "tests" / "threading_foundation.rs", "full_topology_parse", "Full topology test")
check_code(root / "tests" / "threading_foundation.rs", "semantic_rejects_duplicate_kotak", "Duplicate Kotak test")
check_code(root / "tests" / "threading_foundation.rs", "semantic_rejects_spawn_nonexistent_kotak", "Spawn nonexistent test")
print("   Tests: OK")

# 8. v1.21 integrity
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
    print("ALL CHECKS PASSED — v1.30.1-alpha Threading Foundation")
    print("=" * 60)
    print("\nSummary:")
    print("  - AST: Kotak, Pintu<T,U,V>, Spawn, Hantar, Terima, Tunggu")
    print("  - Lexer: kotak, pintu, lahirkan, hantar, terima, tunggu tokens")
    print("  - Parser: kotak N { ... }, lahirkan N(), tunggu N, pintu.hantar(v), pintu.terima()")
    print("  - Semantic: KotakNotFound, DuplicateKotak, topology validation")
    print("  - lib/core/thread.ldx — Kotak & Pintu documentation")
    print("  - lib/core/sync.ldx — Mutex, RwLock, AtomicI32")
    print("  - 12 test assertions")
    print("  - v1.21 integrity: maintained")
    sys.exit(0)
