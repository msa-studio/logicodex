# Logicodex Makefile
# Simple task automation for common development operations

.PHONY: all build test test-all fmt lint bench clean install

LLVM_DIR ?= /usr/lib/llvm-15
RUSTFLAGS ?= -L$(LLVM_DIR)/lib
export RUSTFLAGS

all: build

build:
	cargo build --release

test: test-a

test-a:
	cargo test --tier a

test-b:
	cargo test --tier b

test-c:
	cargo test --tier c

test-all: test-a test-b test-c

fmt:
	cargo fmt

lint:
	cargo clippy -- -D warnings

bench:
	cd benches && bash run_all.sh quick

bench-full:
	cd benches && bash run_all.sh full

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
	cargo fmt --check
	@echo "=== Clippy ==="
	cargo clippy -- -D warnings
	@echo "=== Tier A tests ==="
	cargo test --tier a
	@echo "=== All checks passed ==="
