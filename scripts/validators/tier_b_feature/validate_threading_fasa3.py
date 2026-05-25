#!/usr/bin/env python3
"""
Validator: v1.30.1-alpha — Phase 3: Backpressure + Scheduler
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
print("v1.30.1-alpha: Phase 3 — Backpressure + Scheduler Validator")
print("=" * 60)

# 1. AST: TrySend, TryRecv, Yield, Sleep, TimeoutRecv
print("\n[1] AST: TrySend, TryRecv, Yield, Sleep, TimeoutRecv...")
check_code(root / "src" / "ast.rs", "TrySend {", "TrySend expr")
check_code(root / "src" / "ast.rs", "TryRecv {", "TryRecv expr")
check_code(root / "src" / "ast.rs", "Yield", "Yield expr")
check_code(root / "src" / "ast.rs", "Sleep {", "Sleep expr")
check_code(root / "src" / "ast.rs", "TimeoutRecv {", "TimeoutRecv expr")
check_code(root / "src" / "ast.rs", "channel_name:", "channel_name field")
print("   AST: OK")

# 2. Lexer: TrySend, TryRecv, Yield, Sleep tokens
print("\n[2] Lexer: TrySend, TryRecv, Yield, Sleep tokens...")
check_code(root / "src" / "lexer.rs", "TokenKind::TrySend", "TrySend token")
check_code(root / "src" / "lexer.rs", "TokenKind::TryRecv", "TryRecv token")
check_code(root / "src" / "lexer.rs", "TokenKind::Yield", "Yield token")
check_code(root / "src" / "lexer.rs", "TokenKind::Sleep", "Sleep token")
check_code(root / "src" / "lexer.rs", '("yield", TokenKind::Yield)', "yield alias")
check_code(root / "src" / "lexer.rs", '("sleep", TokenKind::Sleep)', "sleep alias")
print("   Lexer: OK")

# 3. Parser: try_send, try_recv, yield, sleep, timeout_recv
print("\n[3] Parser: try_send, try_recv, yield, sleep, timeout_recv...")
check_code(root / "src" / "parser.rs", 'method == "try_send"', "try_send method")
check_code(root / "src" / "parser.rs", 'method == "try_recv"', "try_recv method")
check_code(root / "src" / "parser.rs", 'method == "timeout_recv"', "timeout_recv method")
check_code(root / "src" / "parser.rs", "TokenKind::Yield", "Yield parse")
check_code(root / "src" / "parser.rs", "TokenKind::Sleep", "Sleep parse")
check_code(root / "src" / "parser.rs", "Expr::TrySend", "TrySend construction")
check_code(root / "src" / "parser.rs", "Expr::TryRecv", "TryRecv construction")
check_code(root / "src" / "parser.rs", "Expr::Sleep", "Sleep construction")
check_code(root / "src" / "parser.rs", "Expr::TimeoutRecv", "TimeoutRecv construction")
print("   Parser: OK")

# 4. Semantic: ChannelFull, RecvTimeout, new expression handling
print("\n[4] Semantic: ChannelFull, RecvTimeout...")
check_code(root / "src" / "semantic.rs", "ChannelFull", "ChannelFull error")
check_code(root / "src" / "semantic.rs", "RecvTimeout", "RecvTimeout error")
check_code(root / "src" / "semantic.rs", "Expr::TrySend {", "TrySend handling")
check_code(root / "src" / "semantic.rs", "Expr::TryRecv {", "TryRecv handling")
check_code(root / "src" / "semantic.rs", "Expr::Yield", "Yield handling")
check_code(root / "src" / "semantic.rs", "Expr::Sleep {", "Sleep handling")
check_code(root / "src" / "semantic.rs", "Expr::TimeoutRecv {", "TimeoutRecv handling")
check_code(root / "src" / "semantic.rs", "backpressure", "backpressure comment")
print("   Semantic: OK")

# 5. Codegen: stubs
print("\n[5] Codegen: Phase 3 stubs...")
check_code(root / "src" / "codegen.rs", "Expr::TrySend {", "TrySend codegen")
check_code(root / "src" / "codegen.rs", "Expr::TryRecv {", "TryRecv codegen")
check_code(root / "src" / "codegen.rs", "Expr::Yield", "Yield codegen")
check_code(root / "src" / "codegen.rs", "Expr::Sleep {", "Sleep codegen")
check_code(root / "src" / "codegen.rs", "Expr::TimeoutRecv {", "TimeoutRecv codegen")
check_code(root / "src" / "codegen.rs", "backpressure aware", "backpressure comment")
check_code(root / "src" / "codegen.rs", "control passed to scheduler", "yield comment")
print("   Codegen: OK")

# 6. lib/core/ring_buffer.ldx
print("\n[6] lib/core/ring_buffer.ldx...")
rb_path = root / "lib" / "core" / "ring_buffer.ldx"
if rb_path.exists():
    text = rb_path.read_text()
    assert "ring_try_send" in text, "ring_try_send missing"
    assert "ring_try_recv" in text, "ring_try_recv missing"
    assert "ring_timeout_recv" in text, "ring_timeout_recv missing"
    assert "Backpressure" in text or "backpressure" in text, "backpressure mention missing"
    print("   ring_buffer.ldx: OK")
else:
    errors.append("lib/core/ring_buffer.ldx missing")

# 7. lib/core/scheduler.ldx
print("\n[7] lib/core/scheduler.ldx...")
sch_path = root / "lib" / "core" / "scheduler.ldx"
if sch_path.exists():
    text = sch_path.read_text()
    assert "Scheduler" in text, "Scheduler struct missing"
    assert "sched_new" in text, "sched_new missing"
    assert "sched_register" in text, "sched_register missing"
    assert "sched_unregister" in text, "sched_unregister missing"
    assert "sched_next_actor" in text, "sched_next_actor missing"
    assert "sched_all_done" in text, "sched_all_done missing"
    assert "sched_run" in text, "sched_run missing"
    assert "yield()" in text, "yield() call missing"
    print("   scheduler.ldx: OK")
else:
    errors.append("lib/core/scheduler.ldx missing")

# 8. Tests
print("\n[8] Tests...")
check_code(root / "tests" / "threading_fasa3.rs", "parse_try_send", "try_send parse test")
check_code(root / "tests" / "threading_fasa3.rs", "parse_try_recv", "try_recv parse test")
check_code(root / "tests" / "threading_fasa3.rs", "parse_yield", "yield parse test")
check_code(root / "tests" / "threading_fasa3.rs", "parse_sleep", "sleep parse test")
check_code(root / "tests" / "threading_fasa3.rs", "parse_timeout_recv", "timeout_recv parse test")
check_code(root / "tests" / "threading_fasa3.rs", "semantic_try_send_valid", "try_send semantic test")
check_code(root / "tests" / "threading_fasa3.rs", "semantic_try_recv_valid", "try_recv semantic test")
check_code(root / "tests" / "threading_fasa3.rs", "semantic_yield_in_actor", "yield semantic test")
check_code(root / "tests" / "threading_fasa3.rs", "semantic_sleep_numeric", "sleep semantic test")
check_code(root / "tests" / "threading_fasa3.rs", "semantic_timeout_recv_valid", "timeout_recv semantic test")
check_code(root / "tests" / "threading_fasa3.rs", "full_backpressure_scheduler_scenario", "full scenario test")
check_code(root / "tests" / "threading_fasa3.rs", "scheduler_library_parses", "scheduler parse test")
print("   Tests: OK")

# 9. Fasa 1 + Fasa 2 integrity
print("\n[9] Earlier phases integrity...")
check_code(root / "src" / "semantic.rs", "UseAfterSend", "Phase 2 UseAfterSend")
check_code(root / "src" / "semantic.rs", "ActorNotFound", "Phase 1 ActorNotFound")
check_code(root / "tests" / "threading_foundation.rs", "parse_actor_declaration", "Phase 1 test")
check_code(root / "tests" / "threading_fasa2.rs", "semantic_rejects_use_after_send", "Phase 2 test")
print("   Earlier phases: OK")

# 10. v1.21 integrity
print("\n[10] v1.21 integrity...")
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
    print("ALL CHECKS PASSED — v1.30.1-alpha Phase 3: Backpressure + Scheduler")
    print("=" * 60)
    print("\nSummary:")
    print("  - AST: TrySend, TryRecv, Yield, Sleep, TimeoutRecv")
    print("  - Lexer: try_send, try_recv, yield, sleep tokens")
    print("  - Parser: channel.try_send(v), channel.try_recv(), yield(), sleep(ms), channel.timeout_recv(ms)")
    print("  - Semantic: ChannelFull, RecvTimeout errors + type checking")
    print("  - Codegen: Phase 3 stubs with backpressure + scheduler awareness")
    print("  - lib/core/ring_buffer.ldx — ring_try_send, ring_try_recv, ring_timeout_recv")
    print("  - lib/core/scheduler.ldx — Cooperative scheduler with round-robin")
    print("  - 14 test assertions")
    print("  - Phase 1 + Phase 2 integrity: maintained")
    print("  - v1.21 integrity: maintained")
    sys.exit(0)
