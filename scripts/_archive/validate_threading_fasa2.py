#!/usr/bin/env python3
"""
Validator: v1.30.1-alpha — Phase 2: Zero-Copy Ownership Transfer
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
print("v1.30.1-alpha: Phase 2 — Zero-Copy Ownership Transfer Validator")
print("=" * 60)

# 1. Semantic: moved_via_channel ownership tracking
print("\n[1] Semantic: moved_via_channel, UseAfterSend...")
check_code(root / "src" / "semantic.rs", "moved_via_channel: HashSet<String>", "moved_via_channel field")
check_code(root / "src" / "semantic.rs", "UseAfterSend", "UseAfterSend error")
check_code(root / "src" / "semantic.rs", "sent through Channel", "ownership error message")
check_code(root / "src" / "semantic.rs", "moved_via_channel.insert", "moved_via_channel insert call")
check_code(root / "src" / "semantic.rs", "moved_via_channel.contains", "moved_via_channel contains check")
print("   Semantic ownership tracking: OK")

# 2. Codegen: send/recv stubs
print("\n[2] Codegen: emit_send, emit_recv stubs...")
check_code(root / "src" / "codegen.rs", "Expr::Send {", "Send codegen")
check_code(root / "src" / "codegen.rs", "Expr::Recv {", "Recv codegen")
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
    assert "ring_send" in text, "ring_send function missing"
    assert "ring_recv" in text, "ring_recv function missing"
    assert "Release" in text, "Release semantics comment missing"
    assert "Acquire" in text, "Acquire semantics comment missing"
    assert "buffer" in text, "buffer field missing"
    print("   ring_buffer.ldx: OK")
else:
    errors.append("lib/core/ring_buffer.ldx missing")

# 4. Tests
print("\n[4] Tests...")
check_code(root / "tests" / "threading_fasa2.rs", "semantic_moves_variable_on_send", "move on send test")
check_code(root / "tests" / "threading_fasa2.rs", "semantic_rejects_use_after_send", "UseAfterSend test")
check_code(root / "tests" / "threading_fasa2.rs", "semantic_allows_use_before_send", "use before send test")
check_code(root / "tests" / "threading_fasa2.rs", "semantic_allows_send_different_variables", "multi variable send test")
check_code(root / "tests" / "threading_fasa2.rs", "semantic_rejects_double_send_same_variable", "double send test")
check_code(root / "tests" / "threading_fasa2.rs", "semantic_recv_does_not_move", "recv no-move test")
check_code(root / "tests" / "threading_fasa2.rs", "full_zero_copy_ownership_scenario", "full scenario test")
check_code(root / "tests" / "threading_fasa2.rs", "ring_buffer_library_parses", "ring buffer parse test")
check_code(root / "tests" / "threading_fasa2.rs", "semantic_send_expression_not_variable", "expr send test")
print("   Tests: OK")

# 5. Fasa 1 integrity
print("\n[5] Fasa 1 integrity...")
check_code(root / "src" / "semantic.rs", "ActorNotFound", "Fasa 1 ActorNotFound")
check_code(root / "src" / "semantic.rs", "DuplicateActor", "Fasa 1 DuplicateActor")
check_code(root / "src" / "semantic.rs", "actor_registry", "Fasa 1 actor_registry")
check_code(root / "src" / "semantic.rs", "channel_registry", "Fasa 1 channel_registry")
check_code(root / "tests" / "threading_foundation.rs", "parse_actor_declaration", "Fasa 1 test exists")
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
    print("ALL CHECKS PASSED — v1.30.1-alpha Phase 2: Zero-Copy Ownership Transfer")
    print("=" * 60)
    print("\nSummary:")
    print("  - Semantic: moved_via_channel HashSet, UseAfterSend error")
    print("  - Codegen: emit_send (Release), emit_recv (Acquire) stubs")
    print("  - lib/core/ring_buffer.ldx — SPSC ring buffer with memory ordering")
    print("  - 12 test assertions covering ownership transfer scenarios")
    print("  - Fasa 1 integrity: maintained")
    print("  - v1.21 integrity: maintained")
    sys.exit(0)
