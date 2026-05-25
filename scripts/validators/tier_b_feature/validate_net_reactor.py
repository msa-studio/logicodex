#!/usr/bin/env python3
"""
Validator: v1.33.0-alpha — Network Reactor Foundation
"""
from pathlib import Path
import sys

root = Path(__file__).resolve().parents[1]
errors = []

def check(path, pattern, desc):
    if not path.exists():
        errors.append(f"MISSING: {path.relative_to(root)}")
        return False
    if pattern not in path.read_text(encoding="utf-8"):
        errors.append(f"{path.relative_to(root)}: {desc}")
        return False
    return True

print("=" * 60)
print("v1.33.0-alpha: Network Reactor Foundation Validator")
print("=" * 60)

checks = [
    ("Module structure", [
        ("src/net/mod.rs", "pub mod connection", "connection module"),
        ("src/net/mod.rs", "pub mod event", "event module"),
        ("src/net/mod.rs", "pub mod policy", "policy module"),
        ("src/net/mod.rs", "pub mod reactor", "reactor module"),
        ("src/net/mod.rs", "pub mod service", "service module"),
        ("src/lib.rs", "pub mod net", "net in lib.rs"),
    ]),
    ("Event types", [
        ("src/net/event.rs", "pub enum EventKind", "EventKind"),
        ("src/net/event.rs", "pub struct Event", "Event"),
        ("src/net/event.rs", "pub enum Action", "Action"),
        ("src/net/event.rs", "Hangup", "Hangup"),
        ("src/net/event.rs", "fn is_terminal", "is_terminal"),
    ]),
    ("Backpressure policies", [
        ("src/net/policy.rs", "pub enum BackpressurePolicy", "BackpressurePolicy"),
        ("src/net/policy.rs", "pub enum BackpressureDecision", "BackpressureDecision"),
        ("src/net/policy.rs", "Block", "Block policy"),
        ("src/net/policy.rs", "DropOldest", "DropOldest policy"),
        ("src/net/policy.rs", "pub struct PolicyConfig", "PolicyConfig"),
        ("src/net/policy.rs", "fn high_throughput", "high_throughput config"),
    ]),
    ("RAII Connection", [
        ("src/net/connection.rs", "pub enum TaintState", "TaintState"),
        ("src/net/connection.rs", "Healthy", "Healthy"),
        ("src/net/connection.rs", "Suspicious", "Suspicious"),
        ("src/net/connection.rs", "Closing", "Closing"),
        ("src/net/connection.rs", "pub struct Connection", "Connection"),
        ("src/net/connection.rs", "impl Drop for Connection", "RAII Drop"),
        ("src/net/connection.rs", "fn handle_event", "handle_event"),
        ("src/net/connection.rs", "fn check_timeout", "check_timeout"),
        ("src/net/connection.rs", "fn check_taint", "check_taint"),
    ]),
    ("Service registry", [
        ("src/net/service.rs", "pub struct Service", "Service"),
        ("src/net/service.rs", "pub struct ServiceRegistry", "ServiceRegistry"),
        ("src/net/service.rs", "fn register", "register"),
        ("src/net/service.rs", "DuplicatePort", "DuplicatePort"),
        ("src/net/service.rs", "GateVerificationError", "GateVerificationError"),
    ]),
    ("Reactor event loop", [
        ("src/net/reactor.rs", "pub struct Reactor", "Reactor"),
        ("src/net/reactor.rs", "fn register", "register"),
        ("src/net/reactor.rs", "fn process_event", "process_event"),
        ("src/net/reactor.rs", "fn run_once", "run_once"),
        ("src/net/reactor.rs", "fn shutdown_all", "shutdown_all"),
        ("src/net/reactor.rs", "impl Drop for Reactor", "Reactor Drop"),
    ]),
    ("AST Service", [
        ("src/ast.rs", "Service {", "Service variant"),
        ("src/ast.rs", "port: u16", "port field"),
        ("src/ast.rs", "handler: String", "handler field"),
        ("src/ast.rs", "policy: String", "policy field"),
    ]),
    ("Lexer Service tokens", [
        ("src/lexer.rs", "TokenKind::Service", "Service token"),
        ("src/lexer.rs", "TokenKind::Port", "Port token"),
        ("src/lexer.rs", "TokenKind::Block", "Block token"),
        ("src/lexer.rs", "TokenKind::DropOldest", "DropOldest token"),
    ]),
    ("Parser service_statement", [
        ("src/parser.rs", "fn service_statement", "service_statement"),
        ("src/parser.rs", "TokenKind::Service", "Service match"),
    ]),
    ("Semantic Service validation", [
        ("src/semantic.rs", "Stmt::Service", "Service semantic"),
        ("src/semantic.rs", "invalid policy", "policy validation"),
    ]),
    ("Tests", [
        ("tests/net_reactor_foundation.rs", "taint_state_default_healthy", "taint test"),
        ("tests/net_reactor_foundation.rs", "connection_new", "connection test"),
        ("tests/net_reactor_foundation.rs", "reactor_register_connection", "reactor test"),
        ("tests/net_reactor_foundation.rs", "parse_service_manifest", "parser test"),
    ]),
    ("v1.32 Capability", [
        ("src/tier2/gate.rs", "GateRef", "GateRef"),
        ("src/tier2/topology.rs", "CapabilityTopology", "CapabilityTopology"),
    ]),
]

for section, section_checks in checks:
    ok = 0
    for file, pattern, desc in section_checks:
        if check(root / file, pattern, desc):
            ok += 1
    status = "OK" if ok == len(section_checks) else f"{ok}/{len(section_checks)}"
    print(f"  [{section}]: {status}")

# v1.21
print("  [v1.21 integrity]...")
import subprocess
result = subprocess.run([sys.executable, str(root / "scripts" / "validate_v121_executable_logic.py")], capture_output=True, text=True, cwd=str(root))
v121_ok = "passed" in result.stdout.lower()
print(f"  {'OK' if v121_ok else 'FAILED'}")
if not v121_ok:
    errors.append("v1.21 validator failed")

print("\n" + "=" * 60)
if errors:
    print(f"FAILED: {len(errors)} error(s)")
    for e in errors:
        print(f"  - {e}")
    sys.exit(1)

print("ALL CHECKS PASSED — v1.33.0-alpha: Network Reactor Foundation")
print("=" * 60)
