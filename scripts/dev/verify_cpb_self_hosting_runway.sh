#!/usr/bin/env bash
set -euo pipefail

echo "---------->>> VERIFY CPB SELF-HOSTING RUNWAY START"

echo "---------->>> STATUS"
git status -sb

echo "---------->>> WHITESPACE"
git diff --check

echo "---------->>> REQUIRED DOCS"
test -f docs/architecture/cpb-self-hosting-runway.md
test -f docs/architecture/compiler-subset.md
test -f docs/architecture/stdlib-core-design-doctrine.md
test -f docs/stdlib_stage0.md
test -f docs/stdlib_contract_versioning.md
test -f docs/architecture/stdlib-migration-status.md
echo "OK: required CPB runway docs exist"

echo "---------->>> REQUIRED CPB TERMS"
grep -q "CPB-0: Contract Discipline" docs/architecture/cpb-self-hosting-runway.md
grep -q "CPB-1: Bootstrap Surface" docs/architecture/cpb-self-hosting-runway.md
grep -q "CPB-2: Bootstrap Stdlib Slice" docs/architecture/cpb-self-hosting-runway.md
grep -q "CPB-5: First Self-Hosting Loop" docs/architecture/cpb-self-hosting-runway.md
grep -q "compiler subset" docs/architecture/cpb-self-hosting-runway.md
grep -q "Required Language Surface" docs/architecture/compiler-subset.md
grep -q "Required Stdlib Surface" docs/architecture/compiler-subset.md
grep -q "Phase 1 Library Blocker Priority" docs/architecture/compiler-subset.md
grep -q "P1-B1: Text/String" docs/architecture/compiler-subset.md
grep -q "P1-B2: Option/Result/Error" docs/architecture/compiler-subset.md
grep -q "P1-B5: File/IO" docs/architecture/compiler-subset.md
grep -q "Package Manager Hold Rule" docs/architecture/compiler-subset.md
grep -q "Real Development Tools Gate" docs/architecture/compiler-subset.md
grep -q "Canonical Modern API Rule" docs/architecture/stdlib-core-design-doctrine.md
grep -q "Legacy Compatibility Rule" docs/architecture/stdlib-core-design-doctrine.md
grep -q "Backward Compatibility Rule" docs/architecture/stdlib-core-design-doctrine.md
grep -q "Self-Hosting Priority Rule" docs/architecture/stdlib-core-design-doctrine.md
grep -q "Package Manager Hold Rule" docs/architecture/stdlib-core-design-doctrine.md
grep -q "Development Tools Hold Rule" docs/architecture/stdlib-core-design-doctrine.md
grep -q "First Proof Programs" docs/architecture/compiler-subset.md
grep -q "token classifier" docs/architecture/compiler-subset.md
grep -q "module path normalizer" docs/architecture/compiler-subset.md
grep -q "diagnostic formatter" docs/architecture/compiler-subset.md
grep -q "bootstrap stdlib slice" docs/architecture/cpb-self-hosting-runway.md
grep -q "Legacy modules must not be repaired ad hoc" docs/architecture/cpb-self-hosting-runway.md
echo "OK: required CPB terms present"

echo "---------->>> README POINTER"
grep -q "cpb-self-hosting-runway.md" README.md
grep -q "compiler-subset.md" README.md
grep -q "stdlib-core-design-doctrine.md" README.md
echo "OK: README points to CPB runway"

echo "---------->>> STAGE0 REGRESSION"
./scripts/dev/verify_stdlib_stage0.sh

echo "---------->>> VERIFY CPB SELF-HOSTING RUNWAY END: OK"
