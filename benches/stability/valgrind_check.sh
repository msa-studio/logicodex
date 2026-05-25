#!/bin/bash
# =========================================================================
# Logicodex v1.45 — Layer 3: Valgrind Leak Check
#
# Runs the Logicodex compiler under valgrind to detect memory leaks.
# Target: 0 leaks, 0 errors.
#
# Usage: ./valgrind_check.sh [test_program.ldx] [timeout_sec]
#   Default: examples/raylib_spinning_box.ldx, 60s timeout
# =========================================================================

set -euo pipefail

TEST_PROG="${1:-examples/raylib_spinning_box.ldx}"
TIMEOUT="${2:-60}"
RESULTS_DIR="${RESULTS_DIR:-../results/stability}"
VALGRIND_LOG="${RESULTS_DIR}/valgrind.log"

echo "=== Logicodex v1.45 — Valgrind Leak Check ==="
echo "Test program: ${TEST_PROG}"
echo "Timeout: ${TIMEOUT}s"
echo ""

# Check valgrind is available
if ! command -v valgrind &>/dev/null; then
    echo "[SKIP] valgrind not installed. Install with: sudo apt install valgrind"
    # Write a skip result
    mkdir -p "${RESULTS_DIR}"
    cat > "${RESULTS_DIR}/valgrind_result.json" << 'JSON'
{
  "status": "skipped",
  "reason": "valgrind not installed",
  "leaks": null,
  "errors": null,
  "recommendation": "Install valgrind: sudo apt install valgrind"
}
JSON
    exit 0
fi

mkdir -p "${RESULTS_DIR}"

# Build the compiler in debug mode (for better valgrind symbols)
echo "[build] Building compiler..."
cd ../..
cargo build 2>/dev/null || echo "[warn] cargo build may have failed, trying existing binary"
cd - >/dev/null

echo "[valgrind] Running leak check (this may take ${TIMEOUT}s)..."
timeout "${TIMEOUT}" valgrind \
    --leak-check=full \
    --show-leak-kinds=all \
    --track-origins=yes \
    --error-exitcode=42 \
    --log-file="${VALGRIND_LOG}" \
    ../../target/debug/logicodex "${TEST_PROG}" \
    2>/dev/null || VG_EXIT=$?

VG_EXIT="${VG_EXIT:-0}"

# Parse valgrind output
LEAKS=$(grep -c "definitely lost:" "${VALGRIND_LOG}" 2>/dev/null || echo 0)
ERRORS=$(grep -c "ERROR SUMMARY:" "${VALGRIND_LOG}" 2>/dev/null || echo 0)

# Extract actual numbers
DEF_LOST=$(grep "definitely lost:" "${VALGRIND_LOG}" | awk '{print $4$5}' | head -1 || echo "0 bytes")
IND_LOST=$(grep "indirectly lost:" "${VALGRIND_LOG}" | awk '{print $4$5}' | head -1 || echo "0 bytes")
POSS_LOST=$(grep "possibly lost:" "${VALGRIND_LOG}" | awk '{print $4$5}' | head -1 || echo "0 bytes")
ERROR_SUM=$(grep "ERROR SUMMARY:" "${VALGRIND_LOG}" | awk '{print $4}' | head -1 || echo "0")

echo ""
echo "=== Valgrind Results ==="
echo "  Exit code: ${VG_EXIT}"
echo "  Definitely lost: ${DEF_LOST}"
echo "  Indirectly lost: ${IND_LOST}"
echo "  Possibly lost: ${POSS_LOST}"
echo "  Errors: ${ERROR_SUM}"

# Write JSON result
python3 << PYEOF > "${RESULTS_DIR}/valgrind_result.json"
import json, sys

data = {
    "status": "completed",
    "exit_code": ${VG_EXIT},
    "definitely_lost": "${DEF_LOST}",
    "indirectly_lost": "${IND_LOST}",
    "possibly_lost": "${POSS_LOST}",
    "error_summary": "${ERROR_SUM}",
    "pass": "${DEF_LOST}" == "0bytes" and "${IND_LOST}" == "0bytes",
    "log_file": "${VALGRIND_LOG}"
}

json.dump(data, sys.stdout, indent=2)
PYEOF

echo ""
echo "Result: ${RESULTS_DIR}/valgrind_result.json"

# Acceptance
if [ "${DEF_LOST}" = "0bytes" ] && [ "${IND_LOST}" = "0bytes" ]; then
    echo "[PASS] No memory leaks detected"
    exit 0
else
    echo "[FAIL] Memory leaks detected!"
    exit 1
fi
