# Logicodex Makefile
# Simple task automation for common development operations

.PHONY: all build test test-all fmt lint bench clean install boot boot-evidence

LLVM_DIR ?= /usr/lib/llvm-15
RUSTFLAGS ?= -L$(LLVM_DIR)/lib
export RUSTFLAGS

all: build

build:
	cargo build --release

test:
	cargo test

test-validators:
	@echo "=== Tier A (core) ==="
	@for v in scripts/validators/tier_a_core/*.py; do echo "--- $$(basename $$v) ---"; python3 "$$v"; done
	@echo "=== Tier B (feature) ==="
	@for v in scripts/validators/tier_b_feature/*.py; do echo "--- $$(basename $$v) ---"; python3 "$$v"; done

test-all: test test-validators

fmt:
	cargo fmt

lint:
	cargo clippy

bench:
	cd benches/harness && bash run_all.sh quick

bench-full:
	cd benches/harness && bash run_all.sh full

clean:
	cargo clean
	find . -name "*.cap" -delete
	find . -name "*.o" -delete
	find . -name "*.wasm" -delete

install:
	cargo install --path .

dev-setup:
	@echo "Installing development dependencies..."
	@echo "Ubuntu/Debian: sudo apt-get install llvm-15-dev libclang-15-dev pkg-config"
	@echo "macOS: brew install llvm@15"
	@echo "Set env: export RUSTFLAGS=\"-L/usr/lib/llvm-15/lib\""

validate:
	@echo "=== Format check ==="
	cargo fmt --all -- --check
	@echo "=== Clippy ==="
	cargo clippy
	@echo "=== Unit tests ==="
	cargo test
	@echo "=== Tier A validators ==="
	@for v in scripts/validators/tier_a_core/*.py; do echo "--- $$(basename $$v) ---"; python3 "$$v"; done
	@echo "=== All checks passed ==="

# Freestanding x86_64 kernel: build -> elf32 -> QEMU boot (clean exit 33)
boot:
	cd freestanding && ./build.sh boot

# CI-friendly: boot, capture serial, assert clean isa-debug-exit (33). Fails CI if not.
boot-evidence:
	@cd freestanding && ./build.sh boot 2>&1 | tee /tmp/logicodex-boot.log
	@echo "--- boot evidence (serial capture) ---"
	@grep -q "QEMU_EXIT_CODE=33" /tmp/logicodex-boot.log && echo "PASS: clean boot, exit 33" || (echo "FAIL: no clean exit 33 in serial log" && exit 1)
	@grep -q "Logicodex" /tmp/logicodex-boot.log && echo "PASS: serial printed 'Logicodex'" || (echo "FAIL: 'Logicodex' not on serial" && exit 1)
