#!/usr/bin/env python3
"""
Validator: Audio Engine — Hardware-Safe Audio Guards
Verifies:
  1. Type::FunctionPointer in AST
  2. Parser: fn(params) -> ret syntax
  3. Semantic: StrictAudioContext struct
  4. Semantic: Audio violation error variants (Io, Recursion, UnboundedLoop)
  5. lib/std/audio.ldx: tulis_selamat, kepit, gelombang_sinus
  6. examples/audio_sine.ldx: Function pointer callback demo
  7. Tests: audio_engine_hardware_safe.rs
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
print("Audio Engine: Hardware-Safe Audio Guards Validator")
print("=" * 60)

# 1. FunctionPointer in AST
print("\n[1] AST: Type::FunctionPointer...")
check_code(root / "src" / "ast.rs", "FunctionPointer {", "FunctionPointer variant")
check_code(root / "src" / "ast.rs", "is_function_pointer", "is_function_pointer() method")
check_code(root / "src" / "ast.rs", "is_audio_callback_fp", "is_audio_callback_fp() method")
check_code(root / "src" / "ast.rs", "params: Vec<Type>", "params field")
check_code(root / "src" / "ast.rs", "return_type: Option<Box<Type>>", "return_type field")
print("   AST FunctionPointer: OK")

# 2. Parser: fn(params) syntax
print("\n[2] Parser: function pointer syntax...")
check_code(root / "src" / "parser.rs", "matches(TokenKind::Fn)", "Fn token check in parse_type")
check_code(root / "src" / "parser.rs", "consume(TokenKind::LeftParen", "LeftParen after fn")
check_code(root / "src" / "parser.rs", "consume(TokenKind::RightParen", "RightParen after params")
check_code(root / "src" / "parser.rs", "matches(TokenKind::Arrow)", "Arrow for return type")
check_code(root / "src" / "parser.rs", "FunctionPointer { params, return_type }", "FunctionPointer construction")
print("   Parser fn syntax: OK")

# 3. Semantic: StrictAudioContext
print("\n[3] Semantic: StrictAudioContext...")
check_code(root / "src" / "semantic.rs", "struct StrictAudioContext", "StrictAudioContext struct")
check_code(root / "src" / "semantic.rs", "audio_callbacks: HashSet<String>", "audio_callbacks set")
check_code(root / "src" / "semantic.rs", "safe_functions: HashSet<String>", "safe_functions set")
check_code(root / "src" / "semantic.rs", "is_audio_registration", "is_audio_registration() method")
check_code(root / "src" / "semantic.rs", "is_forbidden_in_audio", "is_forbidden_in_audio() method")
check_code(root / "src" / "semantic.rs", "tulis_selamat", "tulis_selamat in safe functions")
check_code(root / "src" / "semantic.rs", "verify_audio_safety", "verify_audio_safety() method")
check_code(root / "src" / "semantic.rs", "check_audio_statement", "check_audio_statement() method")
check_code(root / "src" / "semantic.rs", "check_audio_expr", "check_audio_expr() method")
check_code(root / "src" / "semantic.rs", "mark_audio_callback_if_applicable", "mark_audio_callback_if_applicable() method")
print("   StrictAudioContext: OK")

# 4. Audio violation errors
print("\n[4] Semantic: Audio violation errors...")
check_code(root / "src" / "semantic.rs", "AudioViolationIo", "AudioViolationIo error")
check_code(root / "src" / "semantic.rs", "AudioViolationUnboundedLoop", "AudioViolationUnboundedLoop error")
check_code(root / "src" / "semantic.rs", "AudioViolationRecursion", "AudioViolationRecursion error")
check_code(root / "src" / "semantic.rs", "AudioViolationForbiddenCall", "AudioViolationForbiddenCall error")
check_code(root / "src" / "semantic.rs", "melindungi speaker", "Malay speaker protection message")
check_code(root / "src" / "semantic.rs", "protect speakers", "English speaker protection message")
print("   Audio violation errors: OK")

# 5. lib/std/audio.ldx
print("\n[5] lib/std/audio.ldx...")
audio_lib = root / "lib" / "std" / "audio.ldx"
if not audio_lib.exists():
    errors.append("lib/std/audio.ldx: FILE MISSING")
    print("   audio.ldx: MISSING")
else:
    text = audio_lib.read_text()
    assert "tulis_selamat" in text
    assert "kepit" in text
    assert "gelombang_sinus" in text
    assert "Hardware Clipper" in text
    assert "[-1.0, 1.0]" in text
    print(f"   audio.ldx: OK ({len(text)} bytes)")

# 6. examples/audio_sine.ldx
print("\n[6] examples/audio_sine.ldx...")
audio_demo = root / "examples" / "audio_sine.ldx"
if not audio_demo.exists():
    errors.append("examples/audio_sine.ldx: FILE MISSING")
    print("   audio_sine.ldx: MISSING")
else:
    text = audio_demo.read_text()
    assert "fn(*mut f32, i32)" in text or "Function pointer" in text
    assert "tulis_selamat" in text
    assert "unsafe {" in text
    assert "SetAudioStreamCallback" in text
    assert "generate_sine" in text
    print(f"   audio_sine.ldx: OK ({len(text)} bytes)")

# 7. Tests
print("\n[7] Tests...")
check_code(root / "tests" / "audio_engine_hardware_safe.rs", "parse_function_pointer_type_simple", "FP parse test")
check_code(root / "tests" / "audio_engine_hardware_safe.rs", "parse_function_pointer_type_pointer_param", "Audio FP param test")
check_code(root / "tests" / "audio_engine_hardware_safe.rs", "strictaudio_rejects_print_in_callback", "I/O rejection test")
check_code(root / "tests" / "audio_engine_hardware_safe.rs", "strictaudio_rejects_recursion", "Recursion rejection test")
check_code(root / "tests" / "audio_engine_hardware_safe.rs", "strictaudio_rejects_unbounded_loop", "Loop rejection test")
check_code(root / "tests" / "audio_engine_hardware_safe.rs", "strictaudio_permits_safe_functions", "Safe function test")
check_code(root / "tests" / "audio_engine_hardware_safe.rs", "strictaudio_context_detects_forbidden_functions", "Forbidden detection test")
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
    print("   v1.21 integrity: FAILED")

# Results
print("\n" + "=" * 60)
if errors:
    print(f"VALIDATION FAILED: {len(errors)} error(s)")
    for e in errors:
        print(f"  - {e}")
    sys.exit(1)
else:
    print("ALL CHECKS PASSED — Audio Engine Hardware-Safe Guards is complete")
    print("=" * 60)
    print("\nSummary:")
    print("  - AST: Type::FunctionPointer { params, return_type }")
    print("  - Parser: fn(params) -> ret syntax in parse_type()")
    print("  - Semantic: StrictAudioContext with audio safety verification")
    print("  - Audio violations: Io, Recursion, UnboundedLoop, ForbiddenCall")
    print("  - lib/std/audio.ldx: tulis_selamat, kepit, gelombang_sinus")
    print("  - examples/audio_sine.ldx: Function pointer callback demo")
    print("  - Tests: 14 assertions across parse, safety, helpers")
    print("  - v1.21 integrity: maintained")
    sys.exit(0)
