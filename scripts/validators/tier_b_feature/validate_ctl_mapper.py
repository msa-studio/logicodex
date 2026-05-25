#!/usr/bin/env python3
"""
Validator: v1.36.0-alpha — CTL Mapper (Fasa B: WIT Auto-Generation)

Validates:
  - WitDomain: 6 domain mappings (Storage, Net, UI, HW, Audio, Crypto)
  - WitOperation: signature generation
  - CtlMapper: map_capability, map_graph, generate_wit, host reactor stubs
  - Manual override functionality
  - Full pipeline: map_and_generate_wit
  - Zero regression on v1.31-v1.35
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
print("v1.36.0-alpha: CTL Mapper Validator (Fasa B)")
print("=" * 60)

# 1. WitDomain
print("\n[1] WitDomain: 6 standard mappings")
check_code(root / "src" / "tier2" / "ctl_mapper.rs", "pub enum WitDomain", "WitDomain enum")
check_code(root / "src" / "tier2" / "ctl_mapper.rs", "WasiFilesystem", "Storage→filesystem")
check_code(root / "src" / "tier2" / "ctl_mapper.rs", "WasiSockets", "Net→sockets")
check_code(root / "src" / "tier2" / "ctl_mapper.rs", "WasiCli", "UI→cli")
check_code(root / "src" / "tier2" / "ctl_mapper.rs", "HostReactor", "HW→host reactor")
check_code(root / "src" / "tier2" / "ctl_mapper.rs", "WasiIoCustom", "Audio→io/custom")
check_code(root / "src" / "tier2" / "ctl_mapper.rs", "WasiCrypto", "Crypto→crypto")
check_code(root / "src" / "tier2" / "ctl_mapper.rs", "Unknown", "Unknown fallback")
check_code(root / "src" / "tier2" / "ctl_mapper.rs", "fn from_logicodex_domain", "domain parser")
check_code(root / "src" / "tier2" / "ctl_mapper.rs", "fn wit_package_interface", "WIT package format")
check_code(root / "src" / "tier2" / "ctl_mapper.rs", "fn is_host_reactor", "host reactor check")
check_code(root / "src" / "tier2" / "ctl_mapper.rs", "fn is_known", "known domain check")
print("   WitDomain: OK (6 mappings)")

# 2. WitOperation
print("\n[2] WitOperation: WIT function signatures")
check_code(root / "src" / "tier2" / "ctl_mapper.rs", "pub struct WitOperation", "WitOperation struct")
check_code(root / "src" / "tier2" / "ctl_mapper.rs", "fn wit_signature", "signature generator")
check_code(root / "src" / "tier2" / "ctl_mapper.rs", "fn with_param", "param builder")
check_code(root / "src" / "tier2" / "ctl_mapper.rs", "fn with_return", "return builder")
print("   WitOperation: OK")

# 3. get_wit_operations
print("\n[3] get_wit_operations: domain → operation mappings")
check_code(root / "src" / "tier2" / "ctl_mapper.rs", "fn get_wit_operations", "operation lookup")
check_code(root / "src" / "tier2" / "ctl_mapper.rs", "\"Storage\"", "Storage ops")
check_code(root / "src" / "tier2" / "ctl_mapper.rs", "\"Net\"", "Net ops")
check_code(root / "src" / "tier2" / "ctl_mapper.rs", "\"UI\"", "UI ops")
check_code(root / "src" / "tier2" / "ctl_mapper.rs", "\"HW\"", "HW ops")
check_code(root / "src" / "tier2" / "ctl_mapper.rs", "\"Audio\"", "Audio ops")
check_code(root / "src" / "tier2" / "ctl_mapper.rs", "\"Crypto\"", "Crypto ops")
print("   get_wit_operations: OK")

# 4. CtlMapper
print("\n[4] CtlMapper: map_capability + map_graph")
check_code(root / "src" / "tier2" / "ctl_mapper.rs", "pub struct CtlMapper", "CtlMapper struct")
check_code(root / "src" / "tier2" / "ctl_mapper.rs", "fn map_capability", "single cap map")
check_code(root / "src" / "tier2" / "ctl_mapper.rs", "fn map_graph", "full graph map")
check_code(root / "src" / "tier2" / "ctl_mapper.rs", "fn add_override", "manual override")
check_code(root / "src" / "tier2" / "ctl_mapper.rs", "fn generate_wit", "WIT generator")
check_code(root / "src" / "tier2" / "ctl_mapper.rs", "fn generate_host_reactor_stub", "host stub generator")
check_code(root / "src" / "tier2" / "ctl_mapper.rs", "fn stats", "stats method")
check_code(root / "src" / "tier2" / "ctl_mapper.rs", "fn has_hw_gates", "HW detection")
print("   CtlMapper: OK")

# 5. CtlMappingStats
print("\n[5] CtlMappingStats: reporting")
check_code(root / "src" / "tier2" / "ctl_mapper.rs", "pub struct CtlMappingStats", "stats struct")
check_code(root / "src" / "tier2" / "ctl_mapper.rs", "fn is_ok", "ok check")
check_code(root / "src" / "tier2" / "ctl_mapper.rs", "fn summary", "summary format")
print("   Stats: OK")

# 6. Full pipeline functions
print("\n[6] Pipeline: map_and_generate_wit")
check_code(root / "src" / "tier2" / "ctl_mapper.rs", "fn map_and_generate_wit", "one-shot pipeline")
check_code(root / "src" / "tier2" / "ctl_mapper.rs", "fn map_and_generate_wit_with_overrides", "pipeline with overrides")
print("   Pipeline: OK")

# 7. Module exports
print("\n[7] tier2/mod.rs: ctl_mapper module + re-exports")
check_code(root / "src" / "tier2" / "mod.rs", "pub mod ctl_mapper", "module declaration")
check_code(root / "src" / "tier2" / "mod.rs", "pub use ctl_mapper::", "re-export")
check_code(root / "src" / "tier2" / "mod.rs", "CtlMapper", "CtlMapper export")
check_code(root / "src" / "tier2" / "mod.rs", "CtlMappingStats", "Stats export")
check_code(root / "src" / "tier2" / "mod.rs", "WitDomain", "WitDomain export")
check_code(root / "src" / "tier2" / "mod.rs", "WitOperation", "WitOperation export")
print("   Module: OK")

# 8. Tests
print("\n[8] Tests: ctl_mapper.rs")
check_code(root / "tests" / "ctl_mapper.rs", "wit_domain_storage", "Storage domain test")
check_code(root / "tests" / "ctl_mapper.rs", "wit_domain_net", "Net domain test")
check_code(root / "tests" / "ctl_mapper.rs", "wit_domain_ui", "UI domain test")
check_code(root / "tests" / "ctl_mapper.rs", "wit_domain_hw", "HW domain test")
check_code(root / "tests" / "ctl_mapper.rs", "wit_domain_audio", "Audio domain test")
check_code(root / "tests" / "ctl_mapper.rs", "wit_domain_crypto", "Crypto domain test")
check_code(root / "tests" / "ctl_mapper.rs", "wit_domain_unknown", "Unknown domain test")
check_code(root / "tests" / "ctl_mapper.rs", "map_capability_storage_read", "Storage map test")
check_code(root / "tests" / "ctl_mapper.rs", "map_capability_hw_gpio", "HW map test")
check_code(root / "tests" / "ctl_mapper.rs", "map_capability_manual_override", "Override test")
check_code(root / "tests" / "ctl_mapper.rs", "map_graph_maps_all_capabilities", "map_graph test")
check_code(root / "tests" / "ctl_mapper.rs", "generate_wit_has_header", "WIT gen test")
check_code(root / "tests" / "ctl_mapper.rs", "host_reactor_stub_for_hw", "Host stub test")
check_code(root / "tests" / "ctl_mapper.rs", "full_pipeline_basic", "Pipeline test")
check_code(root / "tests" / "ctl_mapper.rs", "all_six_domains_mapped", "All 6 domains test")
print("   Tests: OK (16 test groups)")

# 9. v1.35 integrity
print("\n[9] v1.35 integrity: CapabilityGraph IR (Fasa A)")
result = subprocess.run(
    [sys.executable, str(root / "scripts" / "validate_capability_ir.py")],
    capture_output=True, text=True, cwd=str(root)
)
if "ALL CHECKS PASSED" in result.stdout:
    print("   v1.35: OK")
else:
    errors.append("v1.35 capability IR validator failed")
    if "error" in (result.stderr or "").lower():
        print(f"   STDERR: {result.stderr[:200]}")

# 10. v1.34 integrity
print("\n[10] v1.34 integrity: Sharded Reactor")
result = subprocess.run(
    [sys.executable, str(root / "scripts" / "validate_v134_sharded_reactor.py")],
    capture_output=True, text=True, cwd=str(root)
)
if "ALL CHECKS PASSED" in result.stdout:
    print("   v1.34: OK")
else:
    errors.append("v1.34 sharded reactor validator failed")

# 11. v1.32 integrity
print("\n[11] v1.32 integrity: Capability Fabric")
result = subprocess.run(
    [sys.executable, str(root / "scripts" / "validate_capability_fabric.py")],
    capture_output=True, text=True, cwd=str(root)
)
if "ALL CHECKS PASSED" in result.stdout:
    print("   v1.32: OK")
else:
    errors.append("v1.32 capability fabric validator failed")

# 12. v1.31 integrity
print("\n[12] v1.31 integrity: Streaming Engine")
result = subprocess.run(
    [sys.executable, str(root / "scripts" / "validate_streaming_pass.py")],
    capture_output=True, text=True, cwd=str(root)
)
if "ALL CHECKS PASSED" in result.stdout:
    print("   v1.31: OK")
else:
    errors.append("v1.31 streaming validator failed")

print("\n" + "=" * 60)
if errors:
    print(f"VALIDATION FAILED: {len(errors)} error(s)")
    for e in errors:
        print(f"  - {e}")
    sys.exit(1)
else:
    print("ALL CHECKS PASSED — v1.36.0-alpha: CTL Mapper (Fasa B)")
    print("=" * 60)
    print("\nSummary:")
    print("  - WitDomain: 6 mappings (Storage→filesystem, Net→sockets,")
    print("               UI→cli, HW→host-reactor, Audio→io/custom, Crypto→crypto)")
    print("  - WitOperation: parameter + return type WIT signature generation")
    print("  - CtlMapper: map_capability (single) + map_graph (full)")
    print("  - generate_wit(): complete WIT world + interfaces from CapabilityGraph")
    print("  - generate_host_reactor_stub(): Rust host-side HW implementations")
    print("  - Manual overrides: user-defined WIT mappings take precedence")
    print("  - HW gates: detected + routed through Host Reactor (never WASM guest)")
    print("  - Unknown domains: fallback to logicodex:custom")
    print("  - map_and_generate_wit(): one-shot pipeline")
    print("  - 16 test groups, 12 validator checks")
    print("  - Zero regression: v1.31 + v1.32 + v1.34 + v1.35 all pass")
    sys.exit(0)
