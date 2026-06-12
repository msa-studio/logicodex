# Freestanding x86_64 boot (P1-D5)

Self-contained `no_std` bare-metal kernel proving the freestanding boot loop:
multiboot1 → 32-bit entry → long-mode transition (PAE + identity paging + GDT)
→ 64-bit → COM1 serial → clean QEMU exit.

Separate workspace (does NOT pull in the `logicodex` crate's std/inkwell deps).

## Build + boot
    ./build.sh boot
Expected serial: `BLogicodex`  (B = 32-bit stub reached; Logicodex = long mode + serial)
Expected QEMU exit code: 33  (isa-debug-exit, clean)

## Status
- [x] B1 minimal real boot (multiboot → long mode → serial)
- [x] B2 clean exit (isa-debug-exit)
- [ ] B3 harden: integrate src/os/ (uart/startup/interrupts), IDT-32 (g11),
      panic→UART (g2), MMIO codegen (g12), x86_64 SSE2 (g9)
- [ ] B4 P1-D5 complete
