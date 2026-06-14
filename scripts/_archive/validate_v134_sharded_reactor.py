#!/usr/bin/env python3
"""
Validator: v1.34.0-alpha — The Sharded Deterministic Reactor
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
print("v1.34.0-alpha: The Sharded Deterministic Reactor Validator")
print("=" * 60)

checks = [
    ("Tier 2: Shard Topology", [
        ("src/tier2/shard.rs", "pub struct ShardAssignment", "ShardAssignment"),
        ("src/tier2/shard.rs", "pub struct ShardTopology", "ShardTopology"),
        ("src/tier2/shard.rs", "pub struct ShardVerifyResult", "ShardVerifyResult"),
        ("src/tier2/shard.rs", "pub enum ShardViolation", "ShardViolation"),
        ("src/tier2/shard.rs", "pub struct ServiceGraph", "ServiceGraph"),
        ("src/tier2/shard.rs", "pub struct ServiceNode", "ServiceNode"),
        ("src/tier2/shard.rs", "pub struct CommEdge", "CommEdge"),
        ("src/tier2/shard.rs", "pub enum CommType", "CommType"),
        ("src/tier2/shard.rs", "pub struct DoorRef", "DoorRef"),
        ("src/tier2/shard.rs", "fn verify", "verify method"),
        ("src/tier2/shard.rs", "fn to_manifest_json", "JSON manifest"),
        ("src/tier2/shard.rs", "UnassignedService", "unassigned check"),
        ("src/tier2/shard.rs", "EmptyShard", "empty shard check"),
        ("src/tier2/shard.rs", "CoreConflict", "core conflict check"),
        ("src/tier2/shard.rs", "BudgetOverflow", "budget overflow check"),
        ("src/tier2/shard.rs", "ForbiddenDirectCrossShard", "cross-shard check"),
    ]),
    ("Tier 2: Module exports", [
        ("src/tier2/mod.rs", "pub mod shard", "shard module"),
        ("src/tier2/mod.rs", "ShardAssignment", "export"),
        ("src/tier2/mod.rs", "ShardTopology", "export"),
        ("src/tier2/mod.rs", "ShardVerifyResult", "export"),
        ("src/tier2/mod.rs", "ServiceGraph", "export"),
        ("src/tier2/mod.rs", "DoorRef", "export"),
    ]),
    ("Net: ShardLocalPool", [
        ("src/net/shard_local_pool.rs", "pub struct ShardLocalPool", "ShardLocalPool"),
        ("src/net/shard_local_pool.rs", "pub enum BudgetError", "BudgetError"),
        ("src/net/shard_local_pool.rs", "pub struct AcquireResult", "AcquireResult"),
        ("src/net/shard_local_pool.rs", "pub struct PoolStats", "PoolStats"),
        ("src/net/shard_local_pool.rs", "fn acquire", "acquire"),
        ("src/net/shard_local_pool.rs", "fn release", "release"),
        ("src/net/shard_local_pool.rs", "fn with_mb", "with_mb"),
        ("src/net/shard_local_pool.rs", "fn utilization", "utilization"),
    ]),
    ("Net: CPU Affinity", [
        ("src/net/affinity.rs", "pub fn set_cpu_affinity", "set_cpu_affinity"),
        ("src/net/affinity.rs", "pub fn num_cpus", "num_cpus"),
        ("src/net/affinity.rs", "pub fn is_valid_core", "is_valid_core"),
        ("src/net/affinity.rs", "AffinityError", "AffinityError"),
    ]),
    ("Net: ShardedReactor", [
        ("src/net/sharded_reactor.rs", "pub struct ShardedReactor", "ShardedReactor"),
        ("src/net/sharded_reactor.rs", "pub struct ShardInstance", "ShardInstance"),
        ("src/net/sharded_reactor.rs", "pub struct ShardStats", "ShardStats"),
        ("src/net/sharded_reactor.rs", "pub enum ShardedReactorError", "ShardedReactorError"),
        ("src/net/sharded_reactor.rs", "fn new", "ShardedReactor::new"),
        ("src/net/sharded_reactor.rs", "fn run", "run"),
        ("src/net/sharded_reactor.rs", "fn stop", "stop"),
        ("src/net/sharded_reactor.rs", "fn shutdown", "shutdown"),
        ("src/net/sharded_reactor.rs", "fn shard_for_service", "shard_for_service"),
        ("src/net/sharded_reactor.rs", "fn manifest_json", "manifest_json"),
        ("src/net/sharded_reactor.rs", "impl Drop for ShardedReactor", "RAII Drop"),
    ]),
    ("Net: Module integration", [
        ("src/net/mod.rs", "pub mod affinity", "affinity module"),
        ("src/net/mod.rs", "pub mod shard_local_pool", "shard_local_pool module"),
        ("src/net/mod.rs", "pub mod sharded_reactor", "sharded_reactor module"),
        ("src/net/mod.rs", "ShardLocalPool", "export"),
        ("src/net/mod.rs", "ShardedReactor", "export"),
        ("src/net/mod.rs", "ShardInstance", "export"),
    ]),
    ("Library: Shard manifest", [
        ("lib/core/shard_manifest.ldx", "shard WebShard", "WebShard example"),
        ("lib/core/shard_manifest.ldx", "door WebShard", "Door example"),
        ("lib/core/shard_manifest.ldx", "Anti-Pattern", "anti-pattern"),
    ]),
    ("Tests: Shard topology", [
        ("tests/shard_topology.rs", "shard_assignment_new", "assignment test"),
        ("tests/shard_topology.rs", "service_node_new", "node test"),
        ("tests/shard_topology.rs", "topology_valid_single_shard", "valid topo test"),
        ("tests/shard_topology.rs", "topology_valid_multi_shard_with_door", "multi-shard test"),
        ("tests/shard_topology.rs", "topology_unassigned_service", "unassigned test"),
        ("tests/shard_topology.rs", "topology_empty_shard", "empty shard test"),
        ("tests/shard_topology.rs", "topology_core_conflict", "core conflict test"),
        ("tests/shard_topology.rs", "topology_manifest_json", "JSON test"),
    ]),
    ("Tests: Sharded reactor", [
        ("tests/sharded_reactor.rs", "pool_acquire_success", "pool acquire test"),
        ("tests/sharded_reactor.rs", "pool_acquire_exceeds_budget", "budget exceeded test"),
        ("tests/sharded_reactor.rs", "shard_instance_from_assignment", "instance test"),
        ("tests/sharded_reactor.rs", "sharded_reactor_new_valid", "reactor new test"),
        ("tests/sharded_reactor.rs", "sharded_reactor_run", "reactor run test"),
        ("tests/sharded_reactor.rs", "sharded_reactor_three_shards", "3-shard test"),
    ]),
    ("v1.33 integrity", [
        ("src/net/reactor.rs", "pub struct Reactor", "v1.33 Reactor"),
        ("src/net/connection.rs", "pub struct Connection", "v1.33 Connection"),
        ("tests/net_reactor_foundation.rs", "taint_state_default_healthy", "v1.33 test"),
    ]),
    ("v1.32 integrity", [
        ("src/tier2/gate.rs", "GateRef", "v1.32 GateRef"),
        ("src/tier2/topology.rs", "CapabilityTopology", "v1.32 topology"),
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

print("ALL CHECKS PASSED — v1.34.0-alpha: The Sharded Deterministic Reactor")
print("=" * 60)
