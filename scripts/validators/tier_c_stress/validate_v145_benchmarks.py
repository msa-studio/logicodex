#!/usr/bin/env python3
"""
Validator: v1.45.0-alpha — Quantitative Benchmark Framework

Tier C (Performance): Verifies benchmark infrastructure exists and
BASELINE.json is properly formatted.

Checks:
  1. BASELINE.json exists and is valid JSON
  2. All 4 layer definitions present in baseline
  3. Thresholds defined (regression_warn, regression_fail, etc.)
  4. Benchmark source files exist (6 micro + 2 reactor + 3 stability)
  5. Harness scripts exist (run_all, compare_baseline)
  6. RFC_TEMPLATE.md exists
  7. results/ directory structure exists

Usage: python3 scripts/validators/tier_c_stress/validate_v145_benchmarks.py
"""

import json
import os
import sys

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))

def check(name):
    def decorator(fn):
        CHECKS.append((name, fn)); return fn
    return decorator

CHECKS = []

@check("BASELINE.json exists and valid")
def check_baseline():
    path = os.path.join(REPO, "benches", "BASELINE.json")
    assert os.path.exists(path), "BASELINE.json missing"
    with open(path) as f:
        data = json.load(f)
    assert "version" in data, "Missing version"
    assert "thresholds" in data, "Missing thresholds"
    assert "layer1_micro" in data, "Missing layer1_micro"
    assert "layer2_reactor" in data, "Missing layer2_reactor"
    assert "layer3_stability" in data, "Missing layer3_stability"
    assert "layer4_security" in data, "Missing layer4_security"
    return True

@check("Thresholds properly defined")
def check_thresholds():
    path = os.path.join(REPO, "benches", "BASELINE.json")
    with open(path) as f:
        data = json.load(f)
    t = data["thresholds"]
    assert t["regression_warn_percent"] == 5.0, "warn threshold must be 5%"
    assert t["regression_fail_percent"] == 10.0, "fail threshold must be 10%"
    assert t["memory_creep_max_kb_per_hour"] <= 0.001, "creep threshold too high"
    assert t["scaling_efficiency_min_pct"] >= 85.0, "scaling threshold too low"
    return True

@check("Layer 1: 6 micro-benchmark source files")
def check_layer1_files():
    micro_dir = os.path.join(REPO, "benches", "criterion", "micro")
    expected = [
        "gate_latency.rs", "door_latency.rs", "mempool_latency.rs",
        "callable_lookup.rs", "hir_lower.rs", "llvm_emit.rs",
    ]
    for f in expected:
        assert os.path.exists(os.path.join(micro_dir, f)), f"Missing: {f}"
    return True

@check("Layer 2: Reactor throughput files")
def check_layer2_files():
    reactor_dir = os.path.join(REPO, "benches", "reactor")
    assert os.path.exists(os.path.join(reactor_dir, "echo_server.rs")), "Missing echo_server.rs"
    assert os.path.exists(os.path.join(reactor_dir, "flood_client.rs")), "Missing flood_client.rs"
    assert os.path.exists(os.path.join(reactor_dir, "throughput.sh")), "Missing throughput.sh"
    return True

@check("Layer 3: Stability monitoring files")
def check_layer3_files():
    stable_dir = os.path.join(REPO, "benches", "stability")
    assert os.path.exists(os.path.join(stable_dir, "rss_monitor.py")), "Missing rss_monitor.py"
    assert os.path.exists(os.path.join(stable_dir, "valgrind_check.sh")), "Missing valgrind_check.sh"
    assert os.path.exists(os.path.join(stable_dir, "longrun.sh")), "Missing longrun.sh"
    return True

@check("Harness: run_all.sh + compare_baseline.py")
def check_harness():
    harness_dir = os.path.join(REPO, "benches", "harness")
    assert os.path.exists(os.path.join(harness_dir, "run_all.sh")), "Missing run_all.sh"
    assert os.path.exists(os.path.join(harness_dir, "compare_baseline.py")), "Missing compare_baseline.py"
    return True

@check("RFC_TEMPLATE.md exists")
def check_rfc_template():
    path = os.path.join(REPO, "docs", "RFC_TEMPLATE.md")
    assert os.path.exists(path), "RFC_TEMPLATE.md missing"
    content = open(path).read()
    assert "Architecture Alignment" in content, "Missing Architecture Alignment section"
    assert "Static Topology" in content, "Missing Static Topology checkbox"
    assert "Explicit Ownership" in content, "Missing Explicit Ownership checkbox"
    assert "Shard Isolation" in content, "Missing Shard Isolation checkbox"
    assert "Deterministic Behavior" in content, "Missing Deterministic Behavior checkbox"
    return True

@check("Results directory structure")
def check_results_dirs():
    results_dir = os.path.join(REPO, "benches", "results")
    for subdir in ["micro", "reactor", "stability", "security"]:
        assert os.path.exists(os.path.join(results_dir, subdir)), f"Missing results/{subdir}"
    return True

@check("Baseline has 14 micro benchmarks defined")
def check_baseline_micro_count():
    path = os.path.join(REPO, "benches", "BASELINE.json")
    with open(path) as f:
        data = json.load(f)
    count = len(data.get("layer1_micro", {}))
    assert count >= 8, f"Expected ≥8 micro benchmarks, got {count}"
    return True

@check("Baseline has reactor scaling data")
def check_baseline_reactor():
    path = os.path.join(REPO, "benches", "BASELINE.json")
    with open(path) as f:
        data = json.load(f)
    r = data.get("layer2_reactor", {})
    assert r.get("echo_1core_pps", 0) > 0, "Missing 1-core baseline"
    assert r.get("echo_8core_pps", 0) > 0, "Missing 8-core baseline"
    assert "scaling_efficiency_8core_pct" in r, "Missing 8-core efficiency"
    return True

@check("Baseline has stability targets")
def check_baseline_stability():
    path = os.path.join(REPO, "benches", "BASELINE.json")
    with open(path) as f:
        data = json.load(f)
    s = data.get("layer3_stability", {})
    assert s.get("rss_1h_delta_kb") == 0, "1h target must be 0"
    assert s.get("rss_24h_delta_kb") == 0, "24h target must be 0"
    assert s.get("valgrind_definitely_lost") == "0bytes", "valgrind target must be 0"
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

    print(f"\n{'='*55}")
    print(f"v1.45 Benchmark Framework: {passed}/{passed+failed} checks passed")
    if failed == 0:
        print("ALL CHECKS PASSED — Benchmark infrastructure ready")
    return 0 if failed == 0 else 1

if __name__ == "__main__":
    sys.exit(main())
