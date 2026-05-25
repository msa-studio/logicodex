# Logicodex Makefile
# Simple task automation for common development operations

.PHONY: all build test test-all fmt lint bench clean install

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
