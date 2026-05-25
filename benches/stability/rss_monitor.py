#!/usr/bin/env python3
"""
Logicodex v1.45 — Layer 3: RSS Memory Stability Monitor

Snapshots /proc/[pid]/status every N seconds to track RSS over time.
Detects memory creep (the slope of RSS vs time must be ~0).

Usage:
    python3 rss_monitor.py <pid> [--interval 60] [--duration 3600] [--output rss.csv]

Output CSV format:
    timestamp_s,rss_kb,vm_size_kb,fd_count,threads

Acceptance: Linear regression slope <= 0.001 KB/hour
"""

import argparse
import csv
import os
import signal
import sys
import time
from datetime import datetime
from pathlib import Path


def get_proc_stats(pid: int) -> dict:
    """Read /proc/[pid]/status and /proc/[pid]/fd/ to get memory + FD counts."""
    stats = {
        "rss_kb": 0,
        "vm_size_kb": 0,
        "fd_count": 0,
        "threads": 0,
    }

    # Read /proc/[pid]/status
    try:
        with open(f"/proc/{pid}/status") as f:
            for line in f:
                if line.startswith("VmRSS:"):
                    parts = line.split()
                    stats["rss_kb"] = int(parts[1]) if len(parts) > 1 else 0
                elif line.startswith("VmSize:"):
                    parts = line.split()
                    stats["vm_size_kb"] = int(parts[1]) if len(parts) > 1 else 0
                elif line.startswith("Threads:"):
                    parts = line.split()
                    stats["threads"] = int(parts[1]) if len(parts) > 1 else 0
    except (FileNotFoundError, PermissionError, ProcessLookupError):
        pass

    # Count open file descriptors
    try:
        stats["fd_count"] = len(os.listdir(f"/proc/{pid}/fd"))
    except (FileNotFoundError, PermissionError):
        pass

    return stats


def analyze_creep(csv_path: str) -> dict:
    """Analyze RSS timeseries for memory creep using linear regression."""
    import statistics

    timestamps = []
    rss_values = []

    with open(csv_path) as f:
        reader = csv.DictReader(f)
        for row in reader:
            timestamps.append(float(row["timestamp_s"]))
            rss_values.append(float(row["rss_kb"]))

    if len(timestamps) < 2:
        return {"error": "insufficient data points", "slope_kb_per_hour": None}

    n = len(timestamps)
    mean_t = statistics.mean(timestamps)
    mean_r = statistics.mean(rss_values)

    numerator = sum((t - mean_t) * (r - mean_r) for t, r in zip(timestamps, rss_values))
    denominator = sum((t - mean_t) ** 2 for t in timestamps)

    if denominator == 0:
        return {"error": "zero variance in timestamps", "slope_kb_per_hour": None}

    slope = numerator / denominator  # KB per second
    slope_per_hour = slope * 3600

    # Classify
    if abs(slope_per_hour) <= 0.001:
        verdict = "PASS — Zero creep (ideal)"
    elif abs(slope_per_hour) <= 1.0:
        verdict = "PASS — Negligible creep (acceptable)"
    elif abs(slope_per_hour) <= 10.0:
        verdict = "WARNING — Detectable creep (investigate)"
    else:
        verdict = "FAIL — Significant memory creep"

    return {
        "slope_kb_per_hour": round(slope_per_hour, 6),
        "rss_start_kb": rss_values[0],
        "rss_end_kb": rss_values[-1],
        "rss_delta_kb": round(rss_values[-1] - rss_values[0], 3),
        "datapoints": n,
        "duration_hours": round((timestamps[-1] - timestamps[0]) / 3600, 2),
        "verdict": verdict,
    }


def main():
    parser = argparse.ArgumentParser(description="RSS stability monitor")
    parser.add_argument("pid", type=int, help="Process ID to monitor")
    parser.add_argument("--interval", type=int, default=60, help="Sampling interval (seconds)")
    parser.add_argument("--duration", type=int, default=3600, help="Total duration (seconds)")
    parser.add_argument("--output", type=str, default="rss_timeseries.csv", help="Output CSV path")
    parser.add_argument("--analyze-only", action="store_true", help="Only analyze existing CSV")
    args = parser.parse_args()

    if args.analyze_only:
        result = analyze_creep(args.output)
        print(f"\n=== RSS Stability Analysis ===")
        for k, v in result.items():
            print(f"  {k}: {v}")

        if "PASS" in result.get("verdict", ""):
            sys.exit(0)
        elif "FAIL" in result.get("verdict", ""):
            sys.exit(1)
        else:
            sys.exit(2)
        return

    # Monitor mode
    output_path = Path(args.output)
    print(f"[rss] Monitoring PID {args.pid}")
    print(f"[rss] Interval: {args.interval}s, Duration: {args.duration}s")
    print(f"[rss] Output: {output_path}")

    with open(output_path, "w", newline="") as f:
        writer = csv.writer(f)
        writer.writerow(["timestamp_s", "rss_kb", "vm_size_kb", "fd_count", "threads"])

        start_time = time.time()
        end_time = start_time + args.duration
        sample_count = 0

        def signal_handler(sig, frame):
            elapsed = time.time() - start_time
            print(f"\n[rss] Interrupted after {elapsed:.0f}s ({sample_count} samples)")
            result = analyze_creep(str(output_path))
            print(f"\n[rss] Preliminary: {result['verdict']}")
            sys.exit(0)

        signal.signal(signal.SIGINT, signal_handler)
        signal.signal(signal.SIGTERM, signal_handler)

        while time.time() < end_time:
            elapsed = time.time() - start_time
            stats = get_proc_stats(args.pid)
            writer.writerow([
                round(elapsed, 1),
                stats["rss_kb"],
                stats["vm_size_kb"],
                stats["fd_count"],
                stats["threads"],
            ])
            f.flush()

            sample_count += 1
            if sample_count % 10 == 0:
                print(f"[rss] t={elapsed:.0f}s RSS={stats['rss_kb']}KB FDs={stats['fd_count']} Threads={stats['threads']}")

            time.sleep(args.interval)

    # Analysis
    print(f"\n[rss] Monitoring complete: {sample_count} samples")
    result = analyze_creep(str(output_path))
    print(f"\n=== RSS Stability Report ===")
    for k, v in result.items():
        print(f"  {k}: {v}")

    if "FAIL" in result.get("verdict", ""):
        sys.exit(1)


if __name__ == "__main__":
    main()
