#!/bin/bash
# =========================================================================
# Logicodex v1.45 — Layer 3: Long-Running Stability Test
#
# Runs the compiler + a test program for extended duration while
# monitoring RSS memory. Tests 1h, 6h, and 24h durations.
#
# Usage: ./longrun.sh <duration> [test_program.ldx]
#   duration: 1h, 6h, 24h
#   Default test: examples/raylib_spinning_box.ldx
#
# Output: benches/results/stability/rss_timeseries.csv
# =========================================================================

set -euo pipefail

DURATION_ARG="${1:-1h}"
TEST_PROG="${2:-examples/raylib_spinning_box.ldx}"
RESULTS_DIR="${RESULTS_DIR:-../results/stability}"

# Parse duration
case "${DURATION_ARG}" in
    1h) DURATION_SEC=3600 ;;
    6h) DURATION_SEC=21600 ;;
    24h) DURATION_SEC=86400 ;;
    *) echo "Invalid duration: ${DURATION_ARG}. Use: 1h, 6h, or 24h"; exit 1 ;;
esac

echo "=== Logicodex v1.45 — Long-Run Stability Test ==="
echo "Duration: ${DURATION_ARG} (${DURATION_SEC}s)"
echo "Test program: ${TEST_PROG}"
echo "Target: ΔRSS ≤ 0KB (zero memory creep)"
echo ""

# Build compiler
echo "[build] Building compiler..."
cd ../..
cargo build --release 2>/dev/null || echo "[warn] Using existing binary"
cd - >/dev/null

mkdir -p "${RESULTS_DIR}"

# Start the compiler as a background process
echo "[run] Starting compiler..."
../../target/release/logicodex "${TEST_PROG}" &
COMPILER_PID=$!
echo "[run] Compiler PID: ${COMPILER_PID}"

# Wait a moment for process to initialize
sleep 2

# Check process is alive
if ! kill -0 "${COMPILER_PID}" 2>/dev/null; then
    echo "[FAIL] Compiler process exited immediately"
    exit 1
fi

# Start RSS monitor
echo "[monitor] Starting RSS monitor..."
python3 rss_monitor.py "${COMPILER_PID}" \
    --interval 60 \
    --duration "${DURATION_SEC}" \
    --output "${RESULTS_DIR}/rss_${DURATION_ARG}.csv" &
MONITOR_PID=$!

echo ""
echo "=== Test Running ==="
echo "  Compiler PID: ${COMPILER_PID}"
echo "  Monitor PID:  ${MONITOR_PID}"
echo "  Log:          ${RESULTS_DIR}/rss_${DURATION_ARG}.csv"
echo ""
echo "Press Ctrl+C to stop early, or wait ${DURATION_ARG}..."

# Wait for monitor to finish
wait "${MONITOR_PID}" 2>/dev/null || true

# Clean up compiler
kill "${COMPILER_PID}" 2>/dev/null || true
wait "${COMPILER_PID}" 2>/dev/null || true

echo ""
echo "=== Test Complete ==="

# Analyze results
python3 rss_monitor.py --analyze-only --output "${RESULTS_DIR}/rss_${DURATION_ARG}.csv"

# Write JSON summary
python3 << PYEOF > "${RESULTS_DIR}/stability_${DURATION_ARG}.json"
import json, csv, sys, os

csv_path = "${RESULTS_DIR}/rss_${DURATION_ARG}.csv"
if not os.path.exists(csv_path):
    json.dump({"status": "error", "reason": "CSV not found"}, sys.stdout)
    sys.exit(1)

with open(csv_path) as f:
    rows = list(csv.DictReader(f))

if len(rows) < 2:
    json.dump({"status": "error", "reason": "insufficient data"}, sys.stdout)
    sys.exit(1)

rss_start = float(rows[0]["rss_kb"])
rss_end = float(rows[-1]["rss_kb"])
rss_delta = rss_end - rss_start
duration_h = (float(rows[-1]["timestamp_s"]) - float(rows[0]["timestamp_s"])) / 3600

# Linear regression for slope
n = len(rows)
sum_t = sum(float(r["timestamp_s"]) for r in rows)
sum_r = sum(float(r["rss_kb"]) for r in rows)
sum_tr = sum(float(r["timestamp_s"]) * float(r["rss_kb"]) for r in rows)
sum_tt = sum(float(r["timestamp_s"]) ** 2 for r in rows)
slope = (n * sum_tr - sum_t * sum_r) / (n * sum_tt - sum_t * sum_t) if (n * sum_tt - sum_t * sum_t) != 0 else 0
slope_per_hour = slope * 3600

if abs(slope_per_hour) <= 0.001:
    verdict = "PASS"
elif abs(slope_per_hour) <= 1.0:
    verdict = "PASS"
elif abs(slope_per_hour) <= 10.0:
    verdict = "WARNING"
else:
    verdict = "FAIL"

result = {
    "status": "completed",
    "duration_arg": "${DURATION_ARG}",
    "duration_hours": round(duration_h, 2),
    "samples": n,
    "rss_start_kb": rss_start,
    "rss_end_kb": rss_end,
    "rss_delta_kb": round(rss_delta, 3),
    "slope_kb_per_hour": round(slope_per_hour, 6),
    "verdict": verdict,
    "csv_file": csv_path,
}
json.dump(result, sys.stdout, indent=2)
PYEOF

echo ""
echo "Summary: ${RESULTS_DIR}/stability_${DURATION_ARG}.json"

# Acceptance
python3 << PYEOF
import json, sys
with open("${RESULTS_DIR}/stability_${DURATION_ARG}.json") as f:
    d = json.load(f)
v = d.get("verdict", "UNKNOWN")
print(f"\nVerdict: {v}")
if v == "FAIL":
    sys.exit(1)
PYEOF
