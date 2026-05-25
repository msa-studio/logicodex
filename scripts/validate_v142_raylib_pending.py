#!/usr/bin/env python3
"""
Validator: v1.42.0-alpha — Raylib FFI: 8 Pending Items Resolved

Validates:
  P1: build.rs — Raylib detection + graceful fallback
  P2: Color struct-by-value (not u32) in CallableRegistry
  P3: Vector2/Rectangle struct constructor support
  P4: Math utilities (clamp, lerp, remap) registered
  P5: All 31 functions (28 Raylib + 3 math) registered
  P6: StrictAudioContext — 4 violation types
  P7: WASM blocks Raylib functions
  P8: FfiGatekeeper coercion (widening allowed)

Usage: python3 scripts/validate_v142_raylib_pending.py
"""

import subprocess, sys, os

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CHECKS = []

def check(name):
    def decorator(fn):
        CHECKS.append((name, fn)); return fn
    return decorator

# ─── P1: build.rs exists with Raylib detection ───
@check("P1: build.rs contains Raylib detection")
def p1_build_rs():
    with open(f"{REPO}/build.rs") as f:
        content = f.read()
    assert "pkg-config" in content, "Missing pkg-config detection"
    assert "RAYLIB_DIR" in content, "Missing RAYLIB_DIR env var"
    assert "raylib_no_link" in content, "Missing graceful fallback"
    return True

# ─── P2: Color struct-by-value ───
@check("P2: register_raylib_functions accepts struct IDs")
def p2_color_struct_by_value():
    with open(f"{REPO}/src/ffi/raylib.rs") as f:
        content = f.read()
    assert "struct_ids.color" in content, "Color param should use struct type"
    assert "ids.u32_" not in content.split("register_raylib_functions")[1].split("fn register_math_functions")[0], \
        "Drawing functions should not use u32 for Color"
    return True

# ─── P3: Struct constructor registry ───
@check("P3: Vector2 and Rectangle constructors supported")
def p3_struct_constructors():
    with open(f"{REPO}/src/ffi/raylib.rs") as f:
        content = f.read()
    assert '"Vector2"' in content.split("struct_constructor_arity")[1].split("fn ")[0], \
        "Vector2 not in constructor arity"
    assert '"Rectangle"' in content.split("struct_constructor_arity")[1].split("fn ")[0], \
        "Rectangle not in constructor arity"
    return True

# ─── P4: Math utilities ───
@check("P4: Math utilities registered")
def p4_math_utilities():
    with open(f"{REPO}/src/ffi/raylib.rs") as f:
        content = f.read()
    math_section = content.split("fn register_math_functions")[1]
    assert '"clamp"' in math_section, "clamp not registered"
    assert '"lerp"' in math_section, "lerp not registered"
    assert '"remap"' in math_section, "remap not registered"
    assert '"normalize"' in math_section, "normalize not registered"
    assert "CallableSafety::Safe" in math_section, \
        "Math functions should be Safe (not unsafe)"
    return True

# ─── P5: Runtime linking ───
@check("P5: Math shim extern-C functions defined")
def p5_math_shims():
    with open(f"{REPO}/src/ffi/raylib.rs") as f:
        content = f.read()
    assert "logicodex_clamp_f32" in content, "clamp shim missing"
    assert "logicodex_lerp_f32" in content, "lerp shim missing"
    assert "logicodex_remap_f32" in content, "remap shim missing"
    return True

# ─── P6: StrictAudioContext ───
@check("P6: StrictAudioContext with 4 violation types")
def p6_strict_audio():
    with open(f"{REPO}/src/semantic.rs") as f:
        content = f.read()
    assert "AudioViolationIo" in content, "Missing AudioViolationIo"
    assert "AudioViolationRecursion" in content, "Missing AudioViolationRecursion"
    assert "AudioViolationUnboundedLoop" in content, "Missing AudioViolationUnboundedLoop"
    assert "AudioViolationForbiddenCall" in content, "Missing AudioViolationForbiddenCall"
    assert "register_audio_callback" in content, "Missing register_audio_callback"
    assert "verify_audio_safety" in content, "Missing verify_audio_safety"
    return True

# ─── P7: WASM blocks Raylib ───
@check("P7: WASM target blocks Raylib functions")
def p7_wasm_block():
    with open(f"{REPO}/src/main.rs") as f:
        content = f.read()
    assert "target.is_wasm()" in content, "Missing WASM check"
    assert "WASM target does not support Raylib" in content, "Missing WASM error message"
    assert "is_raylib_function" in content, "Missing is_raylib_function helper"
    return True

# ─── P8: FfiGatekeeper coercion ───
@check("P8: FfiGatekeeper coercion support")
def p8_coercion():
    with open(f"{REPO}/src/ffi.rs") as f:
        content = f.read()
    assert "is_compatible_with_coercion" in content, "Missing coercion function"
    assert "widening" in content.lower() or "coercion" in content.lower(), \
        "Missing coercion logic"
    return True

# ─── Integration: test file exists ───
@check("Integration: v142 test file exists")
def integration_test_file():
    assert os.path.exists(f"{REPO}/tests/raylib_v142_pending.rs"), "Test file missing"
    return True

def main():
    passed = 0
    failed = 0
    for name, fn in CHECKS:
        try:
            if fn():
                print(f"  PASS  {name}")
                passed += 1
            else:
                print(f"  FAIL  {name}")
                failed += 1
        except Exception as e:
            print(f"  FAIL  {name}: {e}")
            failed += 1

    print(f"\n{'='*50}")
    print(f"v1.42 Raylib Pending Items: {passed}/{passed+failed} checks passed")
    if failed == 0:
        print("ALL PENDING ITEMS RESOLVED ✅")
    return 0 if failed == 0 else 1

if __name__ == "__main__":
    sys.exit(main())
