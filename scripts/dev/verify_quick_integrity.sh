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

LOG_DIR="${LOG_DIR:-/tmp/logicodex-quick-integrity}"
SUMMARY="$LOG_DIR/summary.log"
FAILURES="$LOG_DIR/failures.list"

rm -rf "$LOG_DIR"
mkdir -p "$LOG_DIR"
: >"$SUMMARY"
: >"$FAILURES"

FOCUSED_TEST="${FOCUSED_TEST:-}"

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
      sleep 15

      if kill -0 "$pid" 2>/dev/null; then
        elapsed=$((elapsed + 15))
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

CHANGED_FILES="$(
  {
    git diff --name-only HEAD
    git ls-files --others --exclude-standard
  } |
    sed '/^[[:space:]]*$/d' |
    sort -u
)"

echo "---------->>> QUICK INTEGRITY START"
echo "log_dir=$LOG_DIR"

run_gate \
  diff_check \
  git diff --check

run_gate \
  architecture_control \
  ./scripts/dev/verify_architecture_change_control.sh

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
  cargo_check \
  cargo check --all-targets

run_gate \
  quick_script_syntax \
  bash -n scripts/dev/verify_quick_integrity.sh

run_gate \
  full_script_syntax \
  bash -n scripts/dev/verify_full_integrity.sh

run_gate \
  workflow_yaml \
  python3 -c \
  'from pathlib import Path; import yaml; [yaml.safe_load(Path(p).read_text()) for p in [".github/workflows/ci.yml", ".github/workflows/gatekeeper.yml"]]'

if echo "$CHANGED_FILES" |
   grep -qE '^(src/|build\.rs$|Cargo\.(toml|lock)$)'
then
  run_gate \
    library_tests \
    cargo test --quiet --lib

  run_gate \
    semantic_authority \
    cargo test --quiet --test semantic_authority_boundary
fi

while IFS= read -r file; do
  case "$file" in
    tests/*.rs)
      test_name="$(
        basename "$file" .rs
      )"

      run_gate \
        "test_${test_name}" \
        cargo test --quiet --test "$test_name"
      ;;
  esac
done <<<"$CHANGED_FILES"

if [ -n "$FOCUSED_TEST" ]; then
  run_gate \
    "focused_${FOCUSED_TEST}" \
    cargo test --quiet --test "$FOCUSED_TEST"
fi

echo "---------->>> QUICK INTEGRITY SUMMARY"
cat "$SUMMARY"

if [ -s "$FAILURES" ]; then
  echo "---------->>> FAILURE CONTEXT"

  while IFS= read -r name; do
    log="$LOG_DIR/${name}.log"

    echo "===== ${name} ====="

    grep -nEi \
      'error|failed|failure|panic|LLVM|linker|traceback|scannererror' \
      "$log" |
      tail -n 40 || true

    tail -n 60 "$log"
  done <"$FAILURES"

  echo "quick_integrity=FAIL"
  echo "---------->>> QUICK INTEGRITY END"
  exit 1
fi

echo "quick_integrity=PASS"
echo "---------->>> QUICK INTEGRITY END"
