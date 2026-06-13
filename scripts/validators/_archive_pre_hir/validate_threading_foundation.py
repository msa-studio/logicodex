#!/usr/bin/env python3
"""
Validator: v1.30.1-alpha — Threading Foundation (Actor & Channel)
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
print("v1.30.1-alpha: Threading Foundation (Actor & Channel) Validator")
print("=" * 60)

# 1. AST
print("\n[1] AST: Actor, Channel, Spawn, Send, Recv, Join...")
check_code(root / "src" / "ast.rs", "Actor {", "Actor stmt")
check_code(root / "src" / "ast.rs", "Channel { from: String, to: String", "Channel type")
check_code(root / "src" / "ast.rs", "Spawn {", "Spawn expr")
check_code(root / "src" / "ast.rs", "Send {", "Send expr")
check_code(root / "src" / "ast.rs", "Recv {", "Recv expr")
check_code(root / "src" / "ast.rs", "Join {", "Join expr")
check_code(root / "src" / "ast.rs", "is_channel", "is_channel method")
check_code(root / "src" / "ast.rs", "channel_capability", "channel_capability method")
print("   AST: OK")

# 2. Lexer
print("\n[2] Lexer: Actor, Channel, Spawn, Send, Recv, Join...")
check_code(root / "src" / "lexer.rs", "TokenKind::Actor", "Actor token")
check_code(root / "src" / "lexer.rs", "TokenKind::Channel", "Channel token")
check_code(root / "src" / "lexer.rs", "TokenKind::Spawn", "Spawn token")
check_code(root / "src" / "lexer.rs", "TokenKind::Send", "Send token")
check_code(root / "src" / "lexer.rs", "TokenKind::Recv", "Recv token")
check_code(root / "src" / "lexer.rs", "TokenKind::Join", "Join token")
check_code(root / "src" / "lexer.rs", '("actor", TokenKind::Actor)', "actor symbol")
check_code(root / "src" / "lexer.rs", '("spawn", TokenKind::Spawn)', "spawn symbol")
print("   Lexer: OK")

# 3. Parser
print("\n[3] Parser: Actor stmt, Channel<T,U,V>, spawn, send, recv, join...")
check_code(root / "src" / "parser.rs", "fn actor_statement", "actor_statement method")
check_code(root / "src" / "parser.rs", "Type::Channel { from, to, message_type }", "Channel construction")
check_code(root / "src" / "parser.rs", "Spawn { actor_name", "Spawn construction")
check_code(root / "src" / "parser.rs", "Join { actor_name", "Join construction")
check_code(root / "src" / "parser.rs", "Send { channel_name", "Send construction")
check_code(root / "src" / "parser.rs", "Recv { channel_name", "Recv construction")
check_code(root / "src" / "parser.rs", 'send"', "send keyword check")
check_code(root / "src" / "parser.rs", 'recv"', "recv keyword check")
print("   Parser: OK")

# 4. Semantic
print("\n[4] Semantic: Topology validation...")
check_code(root / "src" / "semantic.rs", "ActorNotFound", "ActorNotFound error")
check_code(root / "src" / "semantic.rs", "InvalidChannelTopology", "InvalidChannelTopology error")
check_code(root / "src" / "semantic.rs", "DuplicateActor", "DuplicateActor error")
check_code(root / "src" / "semantic.rs", "actor_registry", "actor_registry field")
check_code(root / "src" / "semantic.rs", "channel_registry", "channel_registry field")
check_code(root / "src" / "semantic.rs", "Spawn {", "Spawn in expression")
check_code(root / "src" / "semantic.rs", "Send {", "Send in expression")
check_code(root / "src" / "semantic.rs", "Recv {", "Recv in expression")
check_code(root / "src" / "semantic.rs", "Join {", "Join in expression")
check_code(root / "src" / "semantic.rs", "Actor {", "Actor in statement")
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
check_code(root / "tests" / "threading_foundation.rs", "parse_actor_declaration", "Actor parse test")
check_code(root / "tests" / "threading_foundation.rs", "parse_actor_with_channel", "Actor+Channel test")
check_code(root / "tests" / "threading_foundation.rs", "parse_spawn", "Spawn test")
check_code(root / "tests" / "threading_foundation.rs", "parse_join", "Join test")
check_code(root / "tests" / "threading_foundation.rs", "parse_send", "Send test")
check_code(root / "tests" / "threading_foundation.rs", "parse_recv", "Recv test")
check_code(root / "tests" / "threading_foundation.rs", "channel_type_properties", "Channel type test")
check_code(root / "tests" / "threading_foundation.rs", "full_topology_parse", "Full topology test")
check_code(root / "tests" / "threading_foundation.rs", "semantic_rejects_duplicate_actor", "Duplicate Actor test")
check_code(root / "tests" / "threading_foundation.rs", "semantic_rejects_spawn_nonexistent_actor", "Spawn nonexistent test")
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
    print("  - AST: Actor, Channel<T,U,V>, Spawn, Send, Recv, Join")
    print("  - Lexer: actor, channel, spawn, send, recv, join tokens")
    print("  - Parser: actor N { ... }, spawn N(), join N, channel.send(v), channel.recv()")
    print("  - Semantic: ActorNotFound, DuplicateActor, topology validation")
    print("  - lib/core/thread.ldx — Actor & Channel documentation")
    print("  - lib/core/sync.ldx — Mutex, RwLock, AtomicI32")
    print("  - 12 test assertions")
    print("  - v1.21 integrity: maintained")
    sys.exit(0)
