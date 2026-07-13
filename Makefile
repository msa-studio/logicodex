# Logicodex Makefile
# Simple task automation for common development operations

.PHONY: all build test test-all fmt lint bench clean install quick integrity boot boot-evidence boot-e2e

LLVM_DIR ?= /usr/lib/llvm-15
RUSTFLAGS ?= -L$(LLVM_DIR)/lib
export RUSTFLAGS

all: build

build:
	cargo build --release

test:
	cargo test

test-validators:
	@echo "=== Live validators (pre-HIR tiers archived under scripts/validators/_archive_pre_hir/) ==="
	@for v in scripts/validators/tier_c_stress/*.py; do echo "--- $$(basename $$v) ---"; python3 "$$v"; done

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
	@echo "=== Live freestanding validator ==="
	@for v in scripts/validators/tier_c_stress/*.py; do echo "--- $$(basename $$v) ---"; python3 "$$v"; done
	@echo "=== All checks passed ==="

# Freestanding x86_64 kernel: build -> elf32 -> QEMU boot (clean exit 33)
boot:
	cd freestanding && ./build.sh boot ../examples/freestanding/minimal.ldx

# CI-friendly: boot, capture serial, assert clean isa-debug-exit (33). Fails CI if not.
boot-evidence:
	@cd freestanding && ./build.sh boot ../examples/freestanding/minimal.ldx 2>&1 | tee /tmp/logicodex-boot.log
	@echo "--- boot evidence (serial capture) ---"
	@grep -q "QEMU_EXIT_CODE=33" /tmp/logicodex-boot.log && echo "PASS: clean boot, exit 33" || (echo "FAIL: no clean exit 33 in serial log" && exit 1)
	@grep -q "Logicodex" /tmp/logicodex-boot.log && echo "PASS: serial printed 'Logicodex'" || (echo "FAIL: 'Logicodex' not on serial" && exit 1)

# End-to-end #4: compile a .ldx program, link it into the freestanding kernel,
# boot in QEMU, and assert the program's own output appears on serial. Guards
# the .ldx -> bootable-kernel pipeline against regressions. CI-friendly.
boot-e2e:
	@cd freestanding && ./build.sh boot ../examples/freestanding/showcase.ldx 2>&1 | tee /tmp/logicodex-e2e.log
	@echo "--- e2e verification (showcase.ldx expected serial: 10 20 55 17 36) ---"
	@tr -d '\r' < /tmp/logicodex-e2e.log > /tmp/logicodex-e2e.clean
	@for v in 10 20 55 17 36; do \
		grep -qx "$$v" /tmp/logicodex-e2e.clean || { echo "FAIL: expected .ldx output '$$v' missing"; exit 1; }; \
	done
	@grep -q "QEMU_EXIT_CODE=33" /tmp/logicodex-e2e.clean || { echo "FAIL: no clean exit 33"; exit 1; }
	@grep -q "Logicodex" /tmp/logicodex-e2e.clean || { echo "FAIL: 'Logicodex' marker missing"; exit 1; }
	@echo "PASS: showcase.ldx compiled -> linked -> booted with correct output (10 20 55 17 36)"

# Fast change-aware validation for small development iterations.
quick:
	@FOCUSED_TEST="$(TEST)" ./scripts/dev/verify_quick_integrity.sh

# Complete pre-commit engine and governance integrity gate.
integrity:
	@./scripts/dev/verify_full_integrity.sh
