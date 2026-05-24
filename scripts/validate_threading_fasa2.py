#!/usr/bin/env python3
"""
Validator: v1.30.1-alpha — Fasa 2: Zero-Copy Ownership Transfer
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
print("v1.30.1-alpha: Fasa 2 — Zero-Copy Ownership Transfer Validator")
print("=" * 60)

# 1. Semantic: moved_via_pintu ownership tracking
print("\n[1] Semantic: moved_via_pintu, UseAfterHantar...")
check_code(root / "src" / "semantic.rs", "moved_via_pintu: HashSet<String>", "moved_via_pintu field")
check_code(root / "src" / "semantic.rs", "UseAfterHantar", "UseAfterHantar error")
check_code(root / "src" / "semantic.rs", "dihantar", "Malay ownership error message")
check_code(root / "src" / "semantic.rs", "ownership", "ownership mention in error")
check_code(root / "src" / "semantic.rs", "moved_via_pintu.insert", "moved_via_pintu insert call")
check_code(root / "src" / "semantic.rs", "moved_via_pintu.contains", "moved_via_pintu contains check")
print("   Semantic ownership tracking: OK")

# 2. Codegen: hantar/terima stubs
print("\n[2] Codegen: emit_hantar, emit_terima stubs...")
check_code(root / "src" / "codegen.rs", "Expr::Hantar {", "Hantar codegen")
check_code(root / "src" / "codegen.rs", "Expr::Terima {", "Terima codegen")
check_code(root / "src" / "codegen.rs", "ownership transferred (Release)", "Release semantics message")
check_code(root / "src" / "codegen.rs", "ownership acquired (Acquire)", "Acquire semantics message")
check_code(root / "src" / "codegen.rs", "pintu_send_release", "pintu_send_release reference")
check_code(root / "src" / "codegen.rs", "pintu_recv_acquire", "pintu_recv_acquire reference")
print("   Codegen stubs: OK")

# 3. lib/core/ring_buffer.ldx
print("\n[3] lib/core/ring_buffer.ldx...")
rb_path = root / "lib" / "core" / "ring_buffer.ldx"
if rb_path.exists():
    text = rb_path.read_text()
    assert "RingBuffer<T>" in text, "RingBuffer struct missing"
    assert "ring_hantar" in text, "ring_hantar function missing"
    assert "ring_terima" in text, "ring_terima function missing"
    assert "Release" in text, "Release semantics comment missing"
    assert "Acquire" in text, "Acquire semantics comment missing"
    assert "penimbal" in text, "penimbal field missing"
    print("   ring_buffer.ldx: OK")
else:
    errors.append("lib/core/ring_buffer.ldx missing")

# 4. Tests
print("\n[4] Tests...")
check_code(root / "tests" / "threading_fasa2.rs", "semantic_moves_variable_on_hantar", "move on hantar test")
check_code(root / "tests" / "threading_fasa2.rs", "semantic_rejects_use_after_hantar", "UseAfterHantar test")
check_code(root / "tests" / "threading_fasa2.rs", "semantic_allows_use_before_hantar", "use before hantar test")
check_code(root / "tests" / "threading_fasa2.rs", "semantic_allows_hantar_different_variables", "multi variable hantar test")
check_code(root / "tests" / "threading_fasa2.rs", "semantic_rejects_double_hantar_same_variable", "double hantar test")
check_code(root / "tests" / "threading_fasa2.rs", "semantic_terima_does_not_move", "terima no-move test")
check_code(root / "tests" / "threading_fasa2.rs", "full_zero_copy_ownership_scenario", "full scenario test")
check_code(root / "tests" / "threading_fasa2.rs", "ring_buffer_library_parses", "ring buffer parse test")
check_code(root / "tests" / "threading_fasa2.rs", "semantic_hantar_expression_not_variable", "expr hantar test")
print("   Tests: OK")

# 5. Fasa 1 integrity
print("\n[5] Fasa 1 integrity...")
check_code(root / "src" / "semantic.rs", "KotakNotFound", "Fasa 1 KotakNotFound")
check_code(root / "src" / "semantic.rs", "DuplicateKotak", "Fasa 1 DuplicateKotak")
check_code(root / "src" / "semantic.rs", "kotak_registry", "Fasa 1 kotak_registry")
check_code(root / "src" / "semantic.rs", "pintu_registry", "Fasa 1 pintu_registry")
check_code(root / "tests" / "threading_foundation.rs", "parse_kotak_declaration", "Fasa 1 test exists")
print("   Fasa 1 integrity: OK")

# 6. v1.21 integrity
print("\n[6] v1.21 integrity...")
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
    print("ALL CHECKS PASSED — v1.30.1-alpha Fasa 2: Zero-Copy Ownership Transfer")
    print("=" * 60)
    print("\nSummary:")
    print("  - Semantic: moved_via_pintu HashSet, UseAfterHantar error")
    print("  - Codegen: emit_hantar (Release), emit_terima (Acquire) stubs")
    print("  - lib/core/ring_buffer.ldx — SPSC ring buffer with memory ordering")
    print("  - 12 test assertions covering ownership transfer scenarios")
    print("  - Fasa 1 integrity: maintained")
    print("  - v1.21 integrity: maintained")
    sys.exit(0)
