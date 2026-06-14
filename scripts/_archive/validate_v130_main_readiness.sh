#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

LOG_DIR="target/v130-main-readiness"
mkdir -p "$LOG_DIR"
LOG_FILE="$LOG_DIR/main_readiness_validation.log"

{
  echo '=== timestamp ==='
  date -u +'%Y-%m-%dT%H:%M:%SZ'
  echo '=== repository ==='
  gh repo view --json nameWithOwner,url,defaultBranchRef
  echo '=== branch/status/divergence ==='
  git status -sb
  git rev-list --left-right --count origin/main...HEAD
  echo '=== latest commits ==='
  git log --oneline --decorate -3
  echo '=== remaining todo markers in v1.30 modules ==='
  grep -R "todo" -n \
    src/span.rs \
    src/types.rs \
    src/layout.rs \
    src/ffi.rs \
    src/hir.rs \
    src/semantic_gate.rs \
    src/codegen_contract.rs \
    src/main.rs || true
  echo '=== cargo fmt check ==='
  cargo fmt --all -- --check
  echo '=== cargo check locked ==='
  cargo check --locked
  echo '=== cargo test locked ==='
  cargo test --locked
  echo '=== v130-check baseline-passing examples ==='
  for f in examples/hello.ldx examples/matematik.ldx examples/perkakasan.ldx; do
    cargo run --quiet -- v130-check "$f"
  done
  echo '=== example compatibility matrix ==='
  for f in examples/*; do
    if [ -f "$f" ]; then
      echo "--- $f check"
      if cargo run --quiet -- check "$f"; then
        echo 'status=0'
      else
        echo "status=$?"
      fi
      echo "--- $f v130-check"
      if cargo run --quiet -- v130-check "$f"; then
        echo 'status=0'
      else
        echo "status=$?"
      fi
    fi
  done
} > "$LOG_FILE" 2>&1

cat "$LOG_FILE"
echo "Validation log written to $LOG_FILE"
