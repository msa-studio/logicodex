#!/usr/bin/env bash
# Build freestanding kernel -> multiboot elf32 -> (optional) boot in QEMU.
set -e
# IMPORTANT: clear any inherited RUSTFLAGS (the parent Makefile exports
# -L/usr/lib/llvm-15/lib). An env RUSTFLAGS OVERRIDES .cargo/config.toml's
# [target.x86_64-unknown-none].rustflags, dropping relocation-model=static
# and -Tlinker.ld -> the freestanding link fails with R_X86_64 reloc errors.
unset RUSTFLAGS
# Force a clean kernel rebuild: cargo's incremental cache can hold a stale
# failed-link state across edits to the linker.ld / asm relocs.
cargo build
K=target/x86_64-unknown-none/debug/logicodex-kernel
objcopy -O elf32-i386 "$K" "$K.elf32"
echo "kernel: $K.elf32"
if [ "${1:-}" = "boot" ]; then
  set +e   # QEMU's clean exit (33) is non-zero by design; do not abort on it
  qemu-system-x86_64 -kernel "$K.elf32" \
    -device isa-debug-exit,iobase=0xf4,iosize=0x04 \
    -serial stdio -display none -no-reboot &
  qpid=$!
  ( sleep 10; kill $qpid 2>/dev/null ) &
  wait $qpid
  printf '\n>>> QEMU_EXIT_CODE=%s (33 = clean isa-debug-exit)\n' "$?"
fi
