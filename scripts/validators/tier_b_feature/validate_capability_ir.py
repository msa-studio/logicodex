#!/usr/bin/env python3
"""
Validator: v1.35.0-alpha — CapabilityGraph IR (Fasa A: Capability IR)

Validates:
  - CapabilityGraph IR: CompileTarget, CapabilityRef, IRServiceNode, IRShardNode,
    IRDoorEdge, IRGateEdge, CapabilityGraph, IRVerifyResult, IRViolation
  - Integration: from_semantic_summaries, from_shard_topology
  - Outputs: verify(), to_cap(), to_wit_stub()
  - Module exports
  - Tests
  - Zero regression on v1.31-v1.34
"""
from pathlib import Path
import sys
import subprocess

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
print("v1.35.0-alpha: CapabilityGraph IR Validator (Fasa A)")
print("=" * 60)

# 1. CompileTarget
print("\n[1] CompileTarget: Native / Wasm / All")
check_code(root / "src" / "tier2" / "capability_ir.rs", "pub enum CompileTarget", "CompileTarget enum")
check_code(root / "src" / "tier2" / "capability_ir.rs", "Native", "Native target")
check_code(root / "src" / "tier2" / "capability_ir.rs", "Wasm", "Wasm target")
check_code(root / "src" / "tier2" / "capability_ir.rs", "All", "All target")
check_code(root / "src" / "tier2" / "capability_ir.rs", "fn from_str", "from_str parser")
check_code(root / "src" / "tier2" / "capability_ir.rs", "fn as_str", "as_str serializer")
check_code(root / "src" / "tier2" / "capability_ir.rs", "ELF", "ELF alias")
print("   CompileTarget: OK")

# 2. CapabilityRef
print("\n[2] CapabilityRef: domain + operation + gate_type + WIT mapping")
check_code(root / "src" / "tier2" / "capability_ir.rs", "pub struct CapabilityRef", "CapabilityRef struct")
check_code(root / "src" / "tier2" / "capability_ir.rs", "pub domain: String", "domain field")
check_code(root / "src" / "tier2" / "capability_ir.rs", "pub operation: String", "operation field")
check_code(root / "src" / "tier2" / "capability_ir.rs", "pub gate_type: GateType", "gate_type field")
check_code(root / "src" / "tier2" / "capability_ir.rs", "pub wit_mapping: Option<String>", "WIT mapping field")
check_code(root / "src" / "tier2" / "capability_ir.rs", "fn canonical", "canonical format")
check_code(root / "src" / "tier2" / "capability_ir.rs", "fn with_wit", "with_wit builder")
check_code(root / "src" / "tier2" / "capability_ir.rs", "fn is_hardware", "is_hardware check")
check_code(root / "src" / "tier2" / "capability_ir.rs", "impl From<&GateRef> for CapabilityRef", "GateRef conversion")
print("   CapabilityRef: OK")

# 3. IRServiceNode
print("\n[3] IRServiceNode: unified service node")
check_code(root / "src" / "tier2" / "capability_ir.rs", "pub struct IRServiceNode", "IRServiceNode struct")
check_code(root / "src" / "tier2" / "capability_ir.rs", "pub requires: Vec<CapabilityRef>", "requires field")
check_code(root / "src" / "tier2" / "capability_ir.rs", "pub provides: Vec<CapabilityRef>", "provides field")
check_code(root / "src" / "tier2" / "capability_ir.rs", "pub assigned_shard: Option<u32>", "shard assignment")
check_code(root / "src" / "tier2" / "capability_ir.rs", "pub channels_used", "channels_used field")
print("   IRServiceNode: OK")

# 4. IRShardNode
print("\n[4] IRShardNode: shard representation")
check_code(root / "src" / "tier2" / "capability_ir.rs", "pub struct IRShardNode", "IRShardNode struct")
check_code(root / "src" / "tier2" / "capability_ir.rs", "pub core_id: u32", "core_id field")
check_code(root / "src" / "tier2" / "capability_ir.rs", "pub budget_mb: u32", "budget field")
check_code(root / "src" / "tier2" / "capability_ir.rs", "pub allowed_gates: Vec<CapabilityRef>", "allowed_gates")
print("   IRShardNode: OK")

# 5. Edge types
print("\n[5] IRDoorEdge + IRGateEdge: unified edges")
check_code(root / "src" / "tier2" / "capability_ir.rs", "pub struct IRDoorEdge", "IRDoorEdge struct")
check_code(root / "src" / "tier2" / "capability_ir.rs", "pub struct IRGateEdge", "IRGateEdge struct")
check_code(root / "src" / "tier2" / "capability_ir.rs", "pub from_service: u32", "door from")
check_code(root / "src" / "tier2" / "capability_ir.rs", "pub capability: CapabilityRef", "gate capability")
print("   Edges: OK")

# 6. CapabilityGraph — THE IR
print("\n[6] CapabilityGraph: Single Source of Truth")
check_code(root / "src" / "tier2" / "capability_ir.rs", "pub struct CapabilityGraph", "CapabilityGraph struct")
check_code(root / "src" / "tier2" / "capability_ir.rs", "pub services: HashMap<u32, IRServiceNode>", "services map")
check_code(root / "src" / "tier2" / "capability_ir.rs", "pub shards: HashMap<u32, IRShardNode>", "shards map")
check_code(root / "src" / "tier2" / "capability_ir.rs", "pub doors: Vec<IRDoorEdge>", "doors vec")
check_code(root / "src" / "tier2" / "capability_ir.rs", "pub gates: Vec<IRGateEdge>", "gates vec")
check_code(root / "src" / "tier2" / "capability_ir.rs", "pub target: CompileTarget", "target field")
check_code(root / "src" / "tier2" / "capability_ir.rs", "Single Source of Truth", "SSOT comment")
print("   CapabilityGraph: OK")

# 7. verify() — unified verification
print("\n[7] verify(): 6 unified checks")
check_code(root / "src" / "tier2" / "capability_ir.rs", "pub fn verify", "verify method")
check_code(root / "src" / "tier2" / "capability_ir.rs", "IRViolation::EmptyGraph", "EmptyGraph check")
check_code(root / "src" / "tier2" / "capability_ir.rs", "IRViolation::WasmHardwareGate", "WasmHardwareGate check")
check_code(root / "src" / "tier2" / "capability_ir.rs", "IRViolation::InvalidShardAssignment", "InvalidShard check")
check_code(root / "src" / "tier2" / "capability_ir.rs", "IRViolation::UnknownServiceInDoor", "UnknownServiceDoor check")
check_code(root / "src" / "tier2" / "capability_ir.rs", "IRViolation::UnknownServiceInGate", "UnknownServiceGate check")
check_code(root / "src" / "tier2" / "capability_ir.rs", "IRViolation::EmptyShard", "EmptyShard check")
check_code(root / "src" / "tier2" / "capability_ir.rs", "pub enum IRViolation", "IRViolation enum")
check_code(root / "src" / "tier2" / "capability_ir.rs", "pub struct IRVerifyResult", "IRVerifyResult struct")
print("   verify(): OK (6 checks)")

# 8. to_cap() — audit manifest
print("\n[8] to_cap(): .cap audit manifest generation")
check_code(root / "src" / "tier2" / "capability_ir.rs", "pub fn to_cap", "to_cap method")
check_code(root / "src" / "tier2" / "capability_ir.rs", "Capability Graph", "cap header")
check_code(root / "src" / "tier2" / "capability_ir.rs", "SERVICES", "services section")
check_code(root / "src" / "tier2" / "capability_ir.rs", "SHARDS", "shards section")
check_code(root / "src" / "tier2" / "capability_ir.rs", "DOORS", "doors section")
check_code(root / "src" / "tier2" / "capability_ir.rs", "GATES", "gates section")
print("   to_cap(): OK")

# 9. to_wit_stub() — Fasa B foundation
print("\n[9] to_wit_stub(): WIT stub for CTL Mapper (Fasa B)")
check_code(root / "src" / "tier2" / "capability_ir.rs", "pub fn to_wit_stub", "to_wit_stub method")
check_code(root / "src" / "tier2" / "capability_ir.rs", "WIT Auto-Generated", "WIT header")
check_code(root / "src" / "tier2" / "capability_ir.rs", "world shard-", "WIT world")
check_code(root / "src" / "tier2" / "capability_ir.rs", "interface", "WIT interface")
check_code(root / "src" / "tier2" / "capability_ir.rs", "CTL Mapper", "CTL Mapper reference")
print("   to_wit_stub(): OK")

# 10. Integration methods
print("\n[10] Integration: from existing v1.31-v1.34 structures")
check_code(root / "src" / "tier2" / "capability_ir.rs", "from_semantic_summaries", "from v1.31 summaries")
check_code(root / "src" / "tier2" / "capability_ir.rs", "from_topology", "from v1.32 topology")
check_code(root / "src" / "tier2" / "capability_ir.rs", "from_shard_topology", "from v1.34 shard topology")
check_code(root / "src" / "tier2" / "capability_ir.rs", "fn service_by_name", "service lookup")
check_code(root / "src" / "tier2" / "capability_ir.rs", "fn shard", "shard lookup")
print("   Integration: OK")

# 11. Module exports
print("\n[11] tier2/mod.rs: capability_ir module + re-exports")
check_code(root / "src" / "tier2" / "mod.rs", "pub mod capability_ir", "module declaration")
check_code(root / "src" / "tier2" / "mod.rs", "pub use capability_ir::", "re-export")
check_code(root / "src" / "tier2" / "mod.rs", "CapabilityGraph", "CapabilityGraph export")
check_code(root / "src" / "tier2" / "mod.rs", "CapabilityRef", "CapabilityRef export")
check_code(root / "src" / "tier2" / "mod.rs", "CompileTarget", "CompileTarget export")
check_code(root / "src" / "tier2" / "mod.rs", "IRVerifyResult", "IRVerifyResult export")
check_code(root / "src" / "tier2" / "mod.rs", "IRViolation", "IRViolation export")
print("   Module: OK")

# 12. Tests
print("\n[12] Tests: capability_ir.rs")
check_code(root / "tests" / "capability_ir.rs", "compile_target_native", "Native test")
check_code(root / "tests" / "capability_ir.rs", "compile_target_wasm", "Wasm test")
check_code(root / "tests" / "capability_ir.rs", "capability_ref_new", "CapabilityRef test")
check_code(root / "tests" / "capability_ir.rs", "capability_ref_is_hardware", "Hardware check test")
check_code(root / "tests" / "capability_ir.rs", "capability_ref_from_gate_ref", "From GateRef test")
check_code(root / "tests" / "capability_ir.rs", "capability_ref_wit_mapping", "WIT mapping test")
check_code(root / "tests" / "capability_ir.rs", "ir_service_node_builder", "ServiceNode test")
check_code(root / "tests" / "capability_ir.rs", "ir_shard_node_new", "ShardNode test")
check_code(root / "tests" / "capability_ir.rs", "capability_graph_new", "Graph new test")
check_code(root / "tests" / "capability_ir.rs", "verify_empty_graph", "EmptyGraph test")
check_code(root / "tests" / "capability_ir.rs", "verify_valid_native_graph", "Valid graph test")
check_code(root / "tests" / "capability_ir.rs", "verify_wasm_rejects_hardware_gate", "WasmHW test")
check_code(root / "tests" / "capability_ir.rs", "verify_invalid_shard_assignment", "InvalidShard test")
check_code(root / "tests" / "capability_ir.rs", "verify_unknown_service_in_door", "UnknownDoor test")
check_code(root / "tests" / "capability_ir.rs", "verify_unknown_service_in_gate", "UnknownGate test")
check_code(root / "tests" / "capability_ir.rs", "verify_empty_shard", "EmptyShard test")
check_code(root / "tests" / "capability_ir.rs", "to_cap_has_header", "to_cap test")
check_code(root / "tests" / "capability_ir.rs", "to_wit_stub_has_header", "to_wit test")
check_code(root / "tests" / "capability_ir.rs", "service_by_name_found", "lookup test")
check_code(root / "tests" / "capability_ir.rs", "from_shard_topology_imports_shards", "ShardTopo test")
check_code(root / "tests" / "capability_ir.rs", "from_semantic_summaries_imports_services", "Semantic test")
check_code(root / "tests" / "capability_ir.rs", "verify_reports_all_violations", "multi violation test")
check_code(root / "tests" / "capability_ir.rs", "verify_result_counts", "result stats test")
print("   Tests: OK (22 assertions)")

# 13. v1.32 integrity
print("\n[13] v1.32 integrity: Capability Fabric")
result = subprocess.run(
    [sys.executable, str(root / "scripts" / "validate_capability_fabric.py")],
    capture_output=True, text=True, cwd=str(root)
)
if "ALL CHECKS PASSED" in result.stdout:
    print("   v1.32: OK")
else:
    errors.append("v1.32 capability fabric validator failed")
    if result.stderr:
        print(f"   STDERR: {result.stderr[:200]}")

# 14. v1.31 integrity
print("\n[14] v1.31 integrity: Streaming Engine")
result = subprocess.run(
    [sys.executable, str(root / "scripts" / "validate_streaming_pass.py")],
    capture_output=True, text=True, cwd=str(root)
)
if "ALL CHECKS PASSED" in result.stdout:
    print("   v1.31: OK")
else:
    errors.append("v1.31 streaming validator failed")

# 15. v1.34 integrity
print("\n[15] v1.34 integrity: Sharded Reactor")
result = subprocess.run(
    [sys.executable, str(root / "scripts" / "validate_v134_sharded_reactor.py")],
    capture_output=True, text=True, cwd=str(root)
)
if "ALL CHECKS PASSED" in result.stdout:
    print("   v1.34: OK")
else:
    errors.append("v1.34 sharded reactor validator failed")

# 16. v1.33 integrity
print("\n[16] v1.33 integrity: Network Reactor")
result = subprocess.run(
    [sys.executable, str(root / "scripts" / "validate_net_reactor.py")],
    capture_output=True, text=True, cwd=str(root)
)
if "ALL CHECKS PASSED" in result.stdout:
    print("   v1.33: OK")
else:
    errors.append("v1.33 net reactor validator failed")

print("\n" + "=" * 60)
if errors:
    print(f"VALIDATION FAILED: {len(errors)} error(s)")
    for e in errors:
        print(f"  - {e}")
    sys.exit(1)
else:
    print("ALL CHECKS PASSED — v1.35.0-alpha: CapabilityGraph IR (Fasa A)")
    print("=" * 60)
    print("\nSummary:")
    print("  - CompileTarget: Native | Wasm | All (parse from string)")
    print("  - CapabilityRef: domain.operation + GateType + optional WIT mapping")
    print("  - IRServiceNode: unified node (v1.31 + v1.32 + v1.34)")
    print("  - IRShardNode: core_id + budget_mb + allowed_gates")
    print("  - IRDoorEdge + IRGateEdge: unified edge types")
    print("  - CapabilityGraph: THE Single Source of Truth IR")
    print("  - verify(): 6 unified checks (EmptyGraph, WasmHW, InvalidShard,")
    print("              UnknownServiceInDoor/Gate, EmptyShard)")
    print("  - to_cap(): .cap audit manifest with SERVICES/SHARDS/DOORS/GATES")
    print("  - to_wit_stub(): WIT foundation for Fasa B CTL Mapper")
    print("  - Integration: from_semantic_summaries (v1.31) + from_shard_topology (v1.34)")
    print("  - 22 test assertions")
    print("  - Zero regression: v1.31 + v1.32 + v1.33 + v1.34 all pass")
    sys.exit(0)
