#!/usr/bin/env bash
set -euo pipefail

cargo fmt --all -- --check
cargo check --locked
cargo test --locked
python3.11 scripts/check_bilingual_error_annotations.py
python3.11 scripts/validate_v121_executable_logic.py
cargo run --quiet -- check examples/hello.ldx
cargo run --quiet -- check examples/matematik.ldx
cargo run --quiet -- compile examples/perkakasan.ldx --target freestanding --object-only --secure --output target/hotfix-audit/perkakasan.o
