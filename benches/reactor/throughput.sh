#!/bin/bash
# =========================================================================
# Logicodex v1.45 — Layer 2: Reactor Throughput Runner
#
# Runs echo_server + flood_client across 1/2/4/8 cores.
# Measures PPS scaling efficiency.
#
# Usage: ./throughput.sh [duration_per_test_sec] [payload_size]
#   Default: 30s per test, 64B payload
#
# Requirements:
#   - cargo build --release (builds echo_server + flood_client)
#   - numactl or taskset (for CPU pinning)
#   - Linux with epoll support
# =========================================================================

set -euo pipefail

DURATION="${1:-30}"
PAYLOAD="${2:-64}"
RESULTS_DIR="${RESULTS_DIR:-../results/reactor}"
HOST="127.0.0.1"
BASE_PORT=29999

echo "=== Logicodex v1.45 — Reactor Throughput Benchmark ==="
echo "Duration per test: ${DURATION}s"
echo "Payload size: ${PAYLOAD}B"
echo ""

# Detect CPU cores
NUM_CORES=$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 4)
echo "Detected cores: ${NUM_CORES}"

# Build binaries
echo "[build] Building echo_server and flood_client..."
cd ../..
cargo build --release --bin echo_server --bin flood_client 2>/dev/null || {
    echo "[build] Building via rustc..."
    mkdir -p target/release
    rustc --edition 2021 -O benches/reactor/echo_server.rs -o target/release/echo_server
    rustc --edition 2021 -O benches/reactor/flood_client.rs -o target/release/flood_client
}
cd - >/dev/null

mkdir -p "${RESULTS_DIR}"
RESULTS_FILE="${RESULTS_DIR}/throughput.json"

# Initialize JSON
{
    echo "{"
    echo "  \"version\": \"1.45.0\","
    echo "  \"date\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\","
    echo "  \"machine\": \"$(hostname 2>/dev/null || echo 'unknown')\","
    echo "  \"cores_total\": ${NUM_CORES},"
    echo "  \"duration_per_test_s\": ${DURATION},"
    echo "  \"payload_size_B\": ${PAYLOAD},"
    echo "  \"results\": ["
} > "${RESULTS_FILE}"

FIRST=true

# Run tests for 1, 2, 4, 8 cores (or up to available)
for CORES in 1 2 4 8; do
    if [ "${CORES}" -gt "${NUM_CORES}" ]; then
        echo "[skip] ${CORES} cores > available ${NUM_CORES}"
        continue
    fi

    PORT=$((BASE_PORT + CORES))

    echo ""
    echo "=== Test: ${CORES} core(s) ==="

    # Start echo server pinned to first CORES-1 cores
    echo "[server] Starting echo_server on port ${PORT}..."
    ../../target/release/echo_server "${PORT}" 0 > "${RESULTS_DIR}/server_${CORES}.log" 2>&1 &
    SERVER_PID=$!
    sleep 1

    # Run flood client with CORES threads
    echo "[client] Flooding with ${CORES} thread(s) for ${DURATION}s..."
    CLIENT_OUTPUT=$(../../target/release/flood_client "${HOST}:${PORT}" "${CORES}" "${DURATION}" "${PAYLOAD}" 2>&1)
    echo "${CLIENT_OUTPUT}"

    # Extract PPS from output
    PPS=$(echo "${CLIENT_OUTPUT}" | grep "PPS:" | awk '{print $2}')
    PKTS=$(echo "${CLIENT_OUTPUT}" | grep "Packets:" | awk '{print $2}')

    # Kill server
    kill "${SERVER_PID}" 2>/dev/null || true
    wait "${SERVER_PID}" 2>/dev/null || true

    # Write JSON entry
    if [ "${FIRST}" = "true" ]; then
        FIRST=false
    else
        echo "," >> "${RESULTS_FILE}"
    fi

    {
        echo "    {"
        echo "      \"cores\": ${CORES},"
        echo "      \"pps\": ${PPS:-0},"
        echo "      \"packets_total\": ${PKTS:-0}"
        echo -n "    }"
    } >> "${RESULTS_FILE}"

done

# Close JSON and compute scaling efficiency
{
    echo ""
    echo "  ],"
    echo "  \"scaling_efficiency\": {"
} >> "${RESULTS_FILE}"

# Parse results to compute efficiency
python3 << 'PYEOF' >> "${RESULTS_FILE}"
import json, sys

try:
    with open("${RESULTS_FILE}") as f:
        content = f.read()
    # Remove trailing comma/brace
    data = json.loads(content.rstrip().rstrip(',').rstrip() + '}}')
    results = data.get("results", [])
    
    if not results:
        print('    "error": "no results"')
        sys.exit(0)
    
    baseline_pps = results[0]["pps"]
    efficiencies = []
    first = True
    
    for r in results[1:]:
        cores = r["cores"]
        pps = r["pps"]
        ideal = baseline_pps * cores
        eff = (pps / ideal * 100) if ideal > 0 else 0
        efficiencies.append((cores, eff))
        if not first:
            print(",")
        first = False
        print(f'    "{cores}core_pct": {eff:.1f}', end="")
    
    if not efficiencies:
        print('    "note": "single core only"')
    
except Exception as e:
    print(f'    "error": "{e}"', end="")
PYEOF

echo "" >> "${RESULTS_FILE}"
echo "  }" >> "${RESULTS_FILE}"
echo "}" >> "${RESULTS_FILE}"

echo ""
echo "=== Results written to ${RESULTS_FILE} ==="

# Quick summary
echo ""
echo "--- Scaling Summary ---"
python3 -c "
import json
with open('${RESULTS_FILE}') as f:
    d = json.load(f)
for r in d['results']:
    print(f\"  {r['cores']} core(s): {r['pps']:.0f} PPS\")
for k, v in d.get('scaling_efficiency', {}).items():
    if isinstance(v, float):
        print(f\"  Efficiency {k}: {v:.1f}%\")
"

echo ""
echo "Target: ≥85% efficiency at 8 cores"
echo "Done."
