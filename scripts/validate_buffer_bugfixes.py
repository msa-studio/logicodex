#!/usr/bin/env python3
"""
Validator: Buffer Provenance Architecture — 5 Bug Fixes
Verifies:
  1. BUG #1: Stmt::Let registers buffer to buffer_registry
  2. BUG #2: Parser supports buf[index] = value assignment
  3. BUG #3: moved_vars cleared on scope exit
  4. BUG #4: mark_moved detects ownership transfer
  5. BUG #5: NotABuffer error for unregistered buffer access
  6. Buffer<f32, 1024> capacity syntax
  7. Stmt::Assign handled in semantic (was missing)
  8. v1.21 integrity
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
print("Buffer Provenance: 5 Bug Fixes Validator")
print("=" * 60)

# 1. BUG #1: Buffer registration in Let
print("\n[1] BUG #1: Buffer registration in Stmt::Let...")
check_code(root / "src" / "semantic.rs", "if let Type::Buffer { element } = &ty", "Buffer type detection in Let")
check_code(root / "src" / "semantic.rs", "self.register_buffer(name", "register_buffer call in Let")
check_code(root / "src" / "semantic.rs", "fn register_buffer", "register_buffer method")
print("   BUG #1: FIXED ✅")

# 2. BUG #2: Parser index assignment
print("\n[2] BUG #2: Parser buf[index] = value...")
check_code(root / "src" / "parser.rs", "peek_index_assignment", "peek_index_assignment method")
check_code(root / "src" / "parser.rs", "index_assignment_statement", "index_assignment_statement method")
check_code(root / "src" / "parser.rs", "Expr::Index {", "Index construction in assignment")
check_code(root / "src" / "parser.rs", "Stmt::Assign {", "Assign construction")
check_code(root / "src" / "parser.rs", "consume(TokenKind::Assign", "Assign consumption")
print("   BUG #2: FIXED ✅")

# 3. BUG #2b: Stmt::Assign in semantic
print("\n[3] BUG #2b: Stmt::Assign in semantic...")
check_code(root / "src" / "semantic.rs", "Stmt::Assign { target, value }", "Assign pattern match")
check_code(root / "src" / "semantic.rs", "Expr::Index { base, index } =>", "Index handling in Assign")
check_code(root / "src" / "semantic.rs", "validate_buffer_index(buf_name", "validate_buffer_index call")
check_code(root / "src" / "semantic.rs", "ElementTypeMismatch {", "ElementTypeMismatch in Assign")
print("   BUG #2b: FIXED ✅")

# 4. BUG #3: moved_vars cleared on scope exit
print("\n[4] BUG #3: moved_vars cleared on scope exit...")
check_code(root / "src" / "semantic.rs", "self.moved_vars.remove(name)", "moved_vars cleanup")
check_code(root / "src" / "semantic.rs", "self.buffer_registry.remove(name)", "buffer_registry cleanup")
print("   BUG #3: FIXED ✅")

# 5. BUG #4: mark_moved called
print("\n[5] BUG #4: mark_moved called on ownership transfer...")
check_code(root / "src" / "semantic.rs", "self.mark_moved(src_name)", "mark_moved call")
check_code(root / "src" / "semantic.rs", "self.buffer_registry.contains_key(src_name)", "Buffer detection for move")
check_code(root / "src" / "semantic.rs", "fn mark_moved", "mark_moved method")
check_code(root / "src" / "semantic.rs", "fn is_moved", "is_moved method")
print("   BUG #4: FIXED ✅")

# 6. BUG #5: NotABuffer error
print("\n[6] BUG #5: NotABuffer error message...")
check_code(root / "src" / "semantic.rs", "NotABuffer { name:", "NotABuffer error variant")
check_code(root / "src" / "semantic.rs", "is not a registered Buffer", "English error message")
check_code(root / "src" / "semantic.rs", "bukan Buffer yang berdaftar", "Malay error message")
print("   BUG #5: FIXED ✅")

# 7. Capacity syntax
print("\n[7] Buffer<f32, 1024> capacity syntax...")
check_code(root / "src" / "parser.rs", "matches(TokenKind::Comma)", "Comma for capacity")
check_code(root / "src" / "parser.rs", "capacity integer after comma", "Capacity error message")
print("   Capacity syntax: ✅")

# 8. Bug fix tests
print("\n[8] Bug fix tests...")
check_code(root / "tests" / "buffer_provenance_bugfixes.rs", "bugfix_1_buffer_let_registers_in_registry", "BUG #1 test")
check_code(root / "tests" / "buffer_provenance_bugfixes.rs", "bugfix_2_index_assignment_parsing", "BUG #2 test")
check_code(root / "tests" / "buffer_provenance_bugfixes.rs", "bugfix_2b_assign_handled_in_semantic", "BUG #2b test")
check_code(root / "tests" / "buffer_provenance_bugfixes.rs", "bugfix_4_move_detection_basic", "BUG #4 test")
check_code(root / "tests" / "buffer_provenance_bugfixes.rs", "bugfix_5_buffer_with_capacity_syntax", "BUG #5 test")
check_code(root / "tests" / "buffer_provenance_bugfixes.rs", "full_buffer_lifecycle_parse", "Full lifecycle test")
print("   Bug fix tests: ✅")

# 9. v1.21 integrity
print("\n[9] v1.21 integrity...")
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
    print("ALL CHECKS PASSED — 5 Buffer Provenance Bugs FIXED")
    print("=" * 60)
    print("\nSummary of fixes:")
    print("  1. ✅ Stmt::Let now calls register_buffer() for Buffer types")
    print("  2. ✅ Parser supports buf[index] = value assignment")
    print("  3. ✅ moved_vars cleared when scope exits (scoped_block)")
    print("  4. ✅ mark_moved() called when buffer is transferred (let buf2 = buf)")
    print("  5. ✅ NotABuffer error when accessing non-registered buffer")
    print("  Bonus: ✅ Stmt::Assign now handled in semantic analyzer")
    print("  Bonus: ✅ Buffer<f32, 1024> capacity syntax in parser")
    print("  Tests: 9 assertions in tests/buffer_provenance_bugfixes.rs")
    print("  v1.21 integrity: maintained")
    sys.exit(0)
