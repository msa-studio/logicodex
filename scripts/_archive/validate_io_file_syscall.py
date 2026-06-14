#!/usr/bin/env python3
"""
Validator: Ketuk 3 + 4 — File Handle ABI + Syscall Backend
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
print("Ketuk 3 + 4: File Handle ABI + Syscall Backend Validator")
print("=" * 60)

# K3: AST
print("\n[K3.1] AST: Opaque, MethodCall...")
check_code(root / "src" / "ast.rs", "Opaque { name: String }", "Opaque type")
check_code(root / "src" / "ast.rs", "MethodCall {", "MethodCall expr")
check_code(root / "src" / "ast.rs", "is_opaque", "is_opaque method")
check_code(root / "src" / "ast.rs", "opaque_name", "opaque_name method")
print("   AST: OK")

# K3: Lexer
print("\n[K3.2] Lexer: FileHandle, Dot...")
check_code(root / "src" / "lexer.rs", "TokenKind::FileHandle", "FileHandle token")
check_code(root / "src" / "lexer.rs", "TokenKind::Open", "Open token")
check_code(root / "src" / "lexer.rs", "TokenKind::Close", "Close token")
check_code(root / "src" / "lexer.rs", "TokenKind::Read", "Read token")
check_code(root / "src" / "lexer.rs", "TokenKind::Write", "Write token")
check_code(root / "src" / "lexer.rs", "TokenKind::Seek", "Seek token")
check_code(root / "src" / "lexer.rs", "TokenKind::Dot", "Dot token")
check_code(root / "src" / "lexer.rs", '(".", TokenKind::Dot)', ". symbol")
print("   Lexer: OK")

# K3: Parser
print("\n[K3.3] Parser: FileHandle, h.read()...")
check_code(root / "src" / "parser.rs", "Type::Opaque { name: \"FileHandle\"", "Opaque parse")
check_code(root / "src" / "parser.rs", "Expr::MethodCall {", "MethodCall construction")
check_code(root / "src" / "parser.rs", "check(TokenKind::Dot)", "Dot check")
print("   Parser: OK")

# K3: Semantic
print("\n[K3.4] Semantic: HandleNotOpen, HandlePermissionDenied...")
check_code(root / "src" / "semantic.rs", "HandleNotOpen", "HandleNotOpen error")
check_code(root / "src" / "semantic.rs", "HandlePermissionDenied", "HandlePermissionDenied error")
check_code(root / "src" / "semantic.rs", "register_handle", "register_handle method")
check_code(root / "src" / "semantic.rs", "is_handle_open", "is_handle_open method")
check_code(root / "src" / "semantic.rs", "check_handle_permission", "check_handle_permission method")
check_code(root / "src" / "semantic.rs", "Expr::MethodCall", "MethodCall in expression")
check_code(root / "src" / "semantic.rs", "handle_registry", "handle_registry field")
print("   Semantic: OK")

# K3: lib/core/file.ldx
print("\n[K3.5] lib/core/file.ldx...")
if (root / "lib" / "core" / "file.ldx").exists():
    print("   file.ldx: OK")
else:
    errors.append("lib/core/file.ldx missing")

# K4: syscall.rs
print("\n[K4.1] src/os/syscall.rs...")
if (root / "src" / "os" / "syscall.rs").exists():
    text = (root / "src" / "os" / "syscall.rs").read_text()
    assert "SYS_READ" in text
    assert "SYS_OPEN" in text
    assert "FileOp" in text
    print("   syscall.rs: OK")
else:
    errors.append("src/os/syscall.rs missing")

# K4: os/mod.rs includes syscall
print("\n[K4.2] os/mod.rs includes syscall...")
check_code(root / "src" / "os" / "mod.rs", "pub mod syscall", "syscall module")
print("   os/mod.rs: OK")

# K4: lib/runtime/io_syscalls.ldx
print("\n[K4.3] lib/runtime/io_syscalls.ldx...")
if (root / "lib" / "runtime" / "io_syscalls.ldx").exists():
    print("   io_syscalls.ldx: OK")
else:
    errors.append("lib/runtime/io_syscalls.ldx missing")

# Tests
print("\n[5] Tests...")
check_code(root / "tests" / "io_file_syscall.rs", "parse_filehandle_type", "FileHandle parse test")
check_code(root / "tests" / "io_file_syscall.rs", "parse_method_call_read", "MethodCall read test")
check_code(root / "tests" / "io_file_syscall.rs", "parse_method_call_close", "MethodCall close test")
check_code(root / "tests" / "io_file_syscall.rs", "full_io_lifecycle_parse", "Full lifecycle test")
check_code(root / "tests" / "io_file_syscall.rs", "syscall_linux_constants", "Syscall constants test")
print("   Tests: OK")

# v1.21 integrity
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
    print("ALL CHECKS PASSED — Ketuk 3 + 4: File Handle ABI + Syscall Backend")
    sys.exit(0)
