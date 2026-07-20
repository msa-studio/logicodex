#!/usr/bin/env bash
set -uo pipefail

SCRIPT_DIR="$(
  cd "$(dirname "${BASH_SOURCE[0]}")" &&
  pwd
)"

REPO_ROOT="$(
  cd "$SCRIPT_DIR/../.." &&
  pwd
)"

cd "$REPO_ROOT"

export CARGO_TERM_COLOR=never
export RAYLIB_NO_LINK=1
unset RUSTFLAGS

LOG_DIR="${LOG_DIR:-/tmp/logicodex-full-integrity}"
SUMMARY="$LOG_DIR/summary.log"
FAILURES="$LOG_DIR/failures.list"

rm -rf "$LOG_DIR"
mkdir -p "$LOG_DIR"
: >"$SUMMARY"
: >"$FAILURES"

run_gate() {
  local name="$1"
  shift

  local log="$LOG_DIR/${name}.log"
  local pid
  local heartbeat_pid
  local rc

  echo "gate_start=$name"

  "$@" >"$log" 2>&1 &
  pid=$!

  (
    local elapsed=0

    while kill -0 "$pid" 2>/dev/null; do
      sleep 20

      if kill -0 "$pid" 2>/dev/null; then
        elapsed=$((elapsed + 20))
        echo "gate_running=$name elapsed=${elapsed}s"
      fi
    done
  ) &

  heartbeat_pid=$!

  wait "$pid"
  rc=$?

  kill "$heartbeat_pid" 2>/dev/null || true
  wait "$heartbeat_pid" 2>/dev/null || true

  echo "gate_done=$name rc=$rc"

  printf '%-32s rc=%s\n' "$name" "$rc" |
    tee -a "$SUMMARY"

  if [ "$rc" -ne 0 ]; then
    echo "$name" >>"$FAILURES"
  fi
}

echo "---------->>> FULL INTEGRITY START"
echo "log_dir=$LOG_DIR"

run_gate \
  diff_check \
  git diff --check

run_gate \
  architecture_control \
  ./scripts/dev/verify_architecture_change_control.sh

run_gate \
  authority_docs_self_test \
  python3 -B scripts/dev/test_authority_docs.py

run_gate \
  authority_docs \
  python3 -B scripts/dev/verify_authority_docs.py

run_gate \
  version_reference_hygiene_self_test \
  python3 -B scripts/dev/test_version_reference_hygiene.py

run_gate \
  version_reference_hygiene \
  python3 -B scripts/dev/verify_version_reference_hygiene.py

run_gate \
  code_lifecycle_self_test \
  python3 -B scripts/dev/test_code_lifecycle.py

run_gate \
  code_lifecycle \
  python3 -B scripts/dev/verify_code_lifecycle.py

run_gate \
  cargo_fmt \
  cargo fmt --all -- --check

run_gate \
  cargo_clippy \
  cargo clippy --all-targets

run_gate \
  cargo_check \
  cargo check --all-targets

run_gate \
  release_build \
  cargo build --release

run_gate \
  test_inventory \
  cargo test --quiet -- --list

run_gate \
  full_cargo_test \
  cargo test --quiet

run_gate \
  semantic_authority \
  cargo test --quiet --test semantic_authority_boundary

run_gate \
  contract_metadata \
  python3 tools/verify_stdlib_contracts.py

run_gate \
  contract_hashes \
  python3 tools/verify_stdlib_contracts.py --emit-hashes

run_gate \
  contract_cases \
  python3 tools/verify_stdlib_contracts.py \
    --run-cases \
    --bin target/release/logicodex

run_gate \
  stdlib_stage0 \
  ./scripts/dev/verify_stdlib_stage0.sh

run_gate \
  cpb_runway \
  ./scripts/dev/verify_cpb_self_hosting_runway.sh

run_gate \
  freestanding_boot_evidence \
  make boot-evidence

run_gate \
  freestanding_boot_e2e \
  make boot-e2e

run_gate \
  freestanding_validator \
  python3 \
    scripts/validators/tier_c_stress/validate_v144_freestanding.py

TEST_COUNT="$(
  grep -cE ': test$' "$LOG_DIR/test_inventory.log" ||
  true
)"

echo "enumerated_test_entries=$TEST_COUNT" |
  tee -a "$SUMMARY"

echo "---------->>> FULL INTEGRITY SUMMARY"
cat "$SUMMARY"

if [ -s "$FAILURES" ]; then
  echo "---------->>> FAILURE CONTEXT"

  while IFS= read -r name; do
    log="$LOG_DIR/${name}.log"

    echo "===== ${name} ====="

    grep -nEi \
      'error|failed|failure|panic|LLVM|linker|not ok|traceback|scannererror' \
      "$log" |
      tail -n 50 || true

    tail -n 80 "$log"
  done <"$FAILURES"

  echo "full_integrity=FAIL"
  echo "---------->>> FULL INTEGRITY END"
  exit 1
fi

echo "full_integrity=PASS"
echo "---------->>> FULL INTEGRITY END"
