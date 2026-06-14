#!/usr/bin/env bash
# Build a freestanding kernel from runtime + an optional .ldx-compiled program.
#
#   ./build.sh [boot] [path/to/program.ldx]
#
# Pipeline:
#   1. (if a .ldx is given) compile it to a freestanding ELF object via the
#      logicodex compiler — exports `main()`, imports `logicodex_print_i64`.
#   2. cargo build the kernel runtime (_start, kmain, UART, shim) and link the
#      .ldx object in, resolving `main`.
#   3. objcopy to multiboot elf32.
#   4. (if 'boot') run in QEMU and report the exit code.
set -e

# Clear inherited RUSTFLAGS — an env RUSTFLAGS overrides .cargo/config.toml's
# [target.x86_64-unknown-none].rustflags (relocation-model=static, -Tlinker.ld),
# breaking the freestanding link with R_X86_64 reloc errors.
unset RUSTFLAGS

BOOT=""
LDX=""
for arg in "$@"; do
  case "$arg" in
    boot) BOOT=1 ;;
    *.ldx) LDX="$arg" ;;
  esac
done

EXTRA_LINK=""
if [ -n "$LDX" ]; then
  echo ">>> compiling $LDX -> freestanding object"
  OBJ="$(pwd)/ldx_program.o"
  # The compiler lives in the parent workspace; build/run it from there.
  ( cd .. && cargo run --quiet -- compile "$LDX" \
       --target freestanding --object-only -o "$OBJ" )
  echo ">>> .ldx object: $OBJ"
  EXTRA_LINK="-C link-arg=$OBJ"
  # cargo does not track the externally-supplied .ldx object as an input, so
  # force a re-link by touching the entry source (otherwise a changed .ldx
  # silently reuses the previously-linked kernel).
  touch src/main.rs
fi

echo ">>> building kernel (+ linking .ldx object if present)"
if [ -n "$EXTRA_LINK" ]; then
  cargo rustc -- $EXTRA_LINK
else
  cargo build
fi

K=target/x86_64-unknown-none/debug/logicodex-kernel
objcopy -O elf32-i386 "$K" "$K.elf32"
echo "kernel: $K.elf32"

if [ -n "$BOOT" ]; then
  set +e   # QEMU's clean exit (33) is non-zero by design
  qemu-system-x86_64 -kernel "$K.elf32" \
    -device isa-debug-exit,iobase=0xf4,iosize=0x04 \
    -serial stdio -display none -no-reboot &
  qpid=$!
  ( sleep 10; kill $qpid 2>/dev/null ) &
  wait $qpid
  printf '\n>>> QEMU_EXIT_CODE=%s (33 = clean isa-debug-exit)\n' "$?"
fi
