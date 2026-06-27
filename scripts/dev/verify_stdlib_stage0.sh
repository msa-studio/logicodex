#!/usr/bin/env bash
set -euo pipefail

echo "---------->>> VERIFY STDLIB STAGE0 START"

echo "---------->>> STATUS"
git status -sb

echo "---------->>> WHITESPACE"
git diff --check

echo "---------->>> STALE DOC PHRASES"
if git grep -n -E "There is no %|There is no \`%\`|BitXor yet|There is also no \`\^\`|First wave|Second wave" -- docs README.md 2>/dev/null; then
  echo "---------->>> STALE DOC PHRASES FOUND"
  exit 1
else
  echo "OK: no known stale stdlib doc phrases"
fi

echo "---------->>> CONTRACT METADATA"
python3 tools/verify_stdlib_contracts.py

echo "---------->>> CONTRACT HASH EVIDENCE"
python3 tools/verify_stdlib_contracts.py --emit-hashes

echo "---------->>> CONTRACT ORACLE CASES"
BIN="target/debug/logicodex"
if [ ! -x "$BIN" ]; then
  echo "debug binary missing; building..."
  cargo build
fi
python3 tools/verify_stdlib_contracts.py --run-cases --bin "$BIN"

echo "---------->>> FOCUSED STDLIB TESTS"
cargo test --test stdlib_core_math --test stdlib_core_assert --test stdlib_core_bits --test stdlib_core_compare --test stdlib_core_bool --test stdlib_core_range -- --nocapture

echo "---------->>> VERIFY STDLIB STAGE0 END: OK"
