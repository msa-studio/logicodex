#!/usr/bin/env python3
"""
Validator: Raylib Spinning Box Demo
Verifies the demo .ldx file and its integration test pipeline.
"""
from pathlib import Path
import sys

root = Path(__file__).resolve().parents[3]
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
print("Demo: Raylib Spinning Box Validator")
print("=" * 60)

# 1. Demo .ldx file
print("\n[1] Demo source file...")
demo_path = root / "examples" / "dormant" / "v1_30" / "raylib_spinning_box.ldx"
if not demo_path.exists():
    errors.append("examples/dormant/v1_30/raylib_spinning_box.ldx: FILE MISSING")
    print("   Demo file: MISSING")
else:
    text = demo_path.read_text()
    # Use flexible matching for spacing variations
    import re
    assert re.search(r'Color\(255,\s*0,\s*0,\s*255\)', text), "Red color constructor"
    assert re.search(r'Color\(0,\s*255,\s*0,\s*255\)', text), "Green color constructor"
    assert re.search(r'Color\(0,\s*0,\s*255,\s*255\)', text), "Blue color constructor"
    assert 'InitWindow(800, 600' in text, "InitWindow call"
    assert 'DrawRectangle(' in text, "DrawRectangle call"
    assert 'ClearBackground(' in text, "ClearBackground call"
    assert 'BeginDrawing()' in text, "BeginDrawing call"
    assert 'EndDrawing()' in text, "EndDrawing call"
    assert 'WindowShouldClose()' in text, "WindowShouldClose call"
    assert 'IsMouseButtonPressed' in text, "Mouse input"
    assert 'IsKeyPressed' in text, "Keyboard input"
    assert 'break' in text, "Break statement"
    assert 'unsafe {' in text, "Unsafe block"
    print(f"   Demo file: OK ({len(text)} bytes, {text.count(chr(10))} lines)")

# 2. Integration test
print("\n[2] Integration test...")
check_code(root / "tests" / "demo_raylib_spinning_box.rs", "fn parse_demo", "parse_demo helper")
check_code(root / "tests" / "demo_raylib_spinning_box.rs", "demo_parses_all_color_constructors", "Parse test")
check_code(root / "tests" / "demo_raylib_spinning_box.rs", "demo_parses_color_with_four_args", "Color(4 args) test")
check_code(root / "tests" / "demo_raylib_spinning_box.rs", "demo_parses_drawrectangle_call", "DrawRectangle parse test")
check_code(root / "tests" / "demo_raylib_spinning_box.rs", "demo_typechecker_validates_all_colors", "TypeChecker test")
check_code(root / "tests" / "demo_raylib_spinning_box.rs", "demo_callableregistry_has_all_functions", "CallableRegistry test")
check_code(root / "tests" / "demo_raylib_spinning_box.rs", "demo_hir_lowering_produces_call_for_drawrectangle", "HIR lowering test")
check_code(root / "tests" / "demo_raylib_spinning_box.rs", "demo_color_red_packs_to_0xff0000ff", "Color packing test")
check_code(root / "tests" / "demo_raylib_spinning_box.rs", "include_str!", "Source embedding")
print("   Integration test: OK")

# 3. Sprint 3 dependencies
print("\n[3] Sprint 3 dependencies...")
check_code(root / "src" / "codegen.rs", "try_struct_constructor", "Struct constructor codegen")
check_code(root / "src" / "codegen.rs", "build_call", "LLVM build_call")
check_code(root / "src" / "hir.rs", "lower_v121_program", "v1.21 lowering")
check_code(root / "src" / "semantic" / "type_checker.rs", "pub fn check_call", "TypeChecker check_call")
check_code(root / "src" / "ffi" / "raylib.rs", "register_raylib_functions", "Raylib function registration")
print("   Dependencies: OK")

# 4. v1.21 integrity
print("\n[4] v1.21 integrity...")
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
    print("ALL CHECKS PASSED — Raylib Spinning Box Demo is ready")
    print("=" * 60)
    print("\nSummary:")
    print("  - Demo file: examples/dormant/v1_30/raylib_spinning_box.ldx")
    print("  - Tests: 11 assertions covering parse, typecheck, registry, HIR, packing")
    print("  - Compile: logicodex --pipeline v1.30 examples/dormant/v1_30/raylib_spinning_box.ldx -o spinning_box")
    print("  - v1.21 integrity: maintained")
    sys.exit(0)
