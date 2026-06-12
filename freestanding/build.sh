#!/usr/bin/env bash
# Build B1 freestanding kernel + produce multiboot-loadable elf32 + boot in QEMU.
set -e
cargo build --release
K=target/x86_64-unknown-none/release/logicodex-kernel
objcopy -O elf32-i386 "$K" "$K.elf32"
echo "kernel: $K.elf32"
[ "${1:-}" = "boot" ] && {
  timeout 10 qemu-system-x86_64 -kernel "$K.elf32" \
    -device isa-debug-exit,iobase=0xf4,iosize=0x04 \
    -serial stdio -display none -no-reboot
  echo "QEMU exit: $? (33 = clean isa-debug-exit)"
}
