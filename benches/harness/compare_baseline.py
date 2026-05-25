#!/usr/bin/env python3
"""
Logicodex v1.45 — Baseline Comparison Tool

Compares benchmark results against BASELINE.json.
Reports regressions, warnings, and passes.

Usage: python3 compare_baseline.py [--baseline ../BASELINE.json] [--results ../results/]
"""

import argparse
import json
import os
import sys
from pathlib import Path


def load_json(path: str) -> dict:
    with open(path) as f:
        return json.load(f)


def check_micro(baseline: dict, results_dir: str) -> list:
    """Compare Layer 1 micro-benchmarks."""
    issues = []
    baseline_layer = baseline.get("layer1_micro", {})

    # Load actual results if they exist
    micro_results_file = os.path.join(results_dir, "micro", "latency.json")
    if not os.path.exists(micro_results_file):
        return [("info", "Layer 1: No results yet — run criterion benchmarks first")]

    results = load_json(micro_results_file)
    warn_pct = baseline["thresholds"]["regression_warn_percent"]
    fail_pct = baseline["thresholds"]["regression_fail_percent"]

    for bench_name, baseline_vals in baseline_layer.items():
        if bench_name not in results:
            continue
        actual_mean = results[bench_name].get("mean", 0)
        baseline_mean = baseline_vals.get("mean", 1)

        if baseline_mean == 0:
            continue

        pct_change = ((actual_mean - baseline_mean) / baseline_mean) * 100

        if pct_change > fail_pct:
            issues.append(("FAIL", f"{bench_name}: +{pct_change:.1f}% regression (>{fail_pct}%)", actual_mean, baseline_mean))
        elif pct_change > warn_pct:
            issues.append(("WARN", f"{bench_name}: +{pct_change:.1f}% regression (>{warn_pct}%)", actual_mean, baseline_mean))
        else:
            issues.append(("PASS", f"{bench_name}: {pct_change:+.1f}%", actual_mean, baseline_mean))

    return issues


def check_reactor(baseline: dict, results_dir: str) -> list:
    """Compare Layer 2 reactor throughput."""
    issues = []
    baseline_layer = baseline.get("layer2_reactor", {})

    throughput_file = os.path.join(results_dir, "reactor", "throughput.json")
    if not os.path.exists(throughput_file):
        return [("info", "Layer 2: No results yet — run throughput.sh first")]

    results = load_json(throughput_file)
    result_map = {r["cores"]: r["pps"] for r in results.get("results", [])}

    for cores in [1, 2, 4, 8]:
        key = f"echo_{cores}core_pps"
        if key not in baseline_layer or cores not in result_map:
            continue
        actual = result_map[cores]
        expected = baseline_layer[key]
        pct = ((actual - expected) / expected) * 100 if expected > 0 else 0

        if pct < -10:
            issues.append(("FAIL", f"{cores}-core PPS: {actual:.0f} vs {expected:.0f} ({pct:.1f}%)", actual, expected))
        elif pct < -5:
            issues.append(("WARN", f"{cores}-core PPS: {actual:.0f} vs {expected:.0f} ({pct:.1f}%)", actual, expected))
        else:
            issues.append(("PASS", f"{cores}-core PPS: {actual:.0f} vs {expected:.0f} ({pct:+.1f}%)", actual, expected))

    # Check scaling efficiency
    min_eff = baseline["thresholds"]["scaling_efficiency_min_pct"]
    for cores in [2, 4, 8]:
        eff_key = f"scaling_efficiency_{cores}core_pct"
        actual_eff = results.get("scaling_efficiency", {}).get(f"{cores}core_pct", 0)
        if actual_eff < min_eff:
            issues.append(("FAIL", f"{cores}-core efficiency: {actual_eff:.1f}% (target: ≥{min_eff}%)", actual_eff, min_eff))
        else:
            issues.append(("PASS", f"{cores}-core efficiency: {actual_eff:.1f}%", actual_eff, min_eff))

    return issues


def check_stability(baseline: dict, results_dir: str) -> list:
    """Compare Layer 3 stability results."""
    issues = []
    baseline_layer = baseline.get("layer3_stability", {})
    max_creep = baseline["thresholds"]["memory_creep_max_kb_per_hour"]

    for duration in ["1h", "6h", "24h"]:
        result_file = os.path.join(results_dir, "stability", f"stability_{duration}.json")
        if not os.path.exists(result_file):
            issues.append(("info", f"Layer 3 {duration}: No results yet"))
            continue

        result = load_json(result_file)
        slope = result.get("slope_kb_per_hour", 0)
        delta = result.get("rss_delta_kb", 0)
        verdict = result.get("verdict", "UNKNOWN")

        if verdict == "FAIL":
            issues.append(("FAIL", f"RSS {duration}: slope={slope:.6f} KB/h, Δ={delta} KB", slope, max_creep))
        elif verdict == "WARNING":
            issues.append(("WARN", f"RSS {duration}: slope={slope:.6f} KB/h", slope, max_creep))
        else:
            issues.append(("PASS", f"RSS {duration}: slope={slope:.6f} KB/h, Δ={delta} KB", slope, max_creep))

    # Valgrind
    valgrind_file = os.path.join(results_dir, "stability", "valgrind_result.json")
    if os.path.exists(valgrind_file):
        vg = load_json(valgrind_file)
        if vg.get("pass", False):
            issues.append(("PASS", "Valgrind: 0 leaks", 0, 0))
        else:
            issues.append(("FAIL", f"Valgrind: {vg.get('definitely_lost', 'unknown')} lost", 0, 0))
    else:
        issues.append(("info", "Valgrind: Not yet run"))

    return issues


def main():
    parser = argparse.ArgumentParser(description="Compare benchmark results against baseline")
    parser.add_argument("--baseline", default="../BASELINE.json", help="Path to BASELINE.json")
    parser.add_argument("--results", default="../results", help="Path to results directory")
    args = parser.parse_args()

    baseline_path = Path(args.baseline)
    results_dir = args.results

    if not baseline_path.exists():
        print(f"[ERROR] Baseline not found: {baseline_path}")
        sys.exit(1)

    baseline = load_json(str(baseline_path))
    print(f"=== Baseline: v{baseline.get('version', '?')} ({baseline.get('date', '?')}) ===\n")

    all_issues = []
    all_issues.extend(("L1-MICRO", *i) for i in check_micro(baseline, results_dir))
    all_issues.extend(("L2-REACTOR", *i) for i in check_reactor(baseline, results_dir))
    all_issues.extend(("L3-STABLE", *i) for i in check_stability(baseline, results_dir))

    passes = sum(1 for _, status, *_ in all_issues if status == "PASS")
    warns = sum(1 for _, status, *_ in all_issues if status == "WARN")
    fails = sum(1 for _, status, *_ in all_issues if status == "FAIL")
    infos = sum(1 for _, status, *_ in all_issues if status == "info")

    # Print results
    for layer, status, msg, *vals in all_issues:
        icon = {"PASS": "✅", "WARN": "⚠️", "FAIL": "❌", "info": "ℹ️"}.get(status, "?")
        print(f"  {icon} [{layer}] {msg}")

    print(f"\n{'='*50}")
    print(f"Summary: {passes} PASS, {warns} WARN, {fails} FAIL, {infos} INFO")

    if fails > 0:
        print(f"\n❌ REGRESSION DETECTED — Do not merge!")
        sys.exit(1)
    elif warns > 0:
        print(f"\n⚠️  Warnings present — Review before merge")
        sys.exit(0)
    else:
        print(f"\n✅ All checks pass — Clear to merge")
        sys.exit(0)


if __name__ == "__main__":
    main()
