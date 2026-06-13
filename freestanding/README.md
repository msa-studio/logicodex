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
- [~] B3: IDT-256 with 32 exception handlers (g11) DONE — int3 routes, iretq returns clean. Next: panic->UART (g2) DONE, MMIO (g12), SSE2 (g9), os/ integration
      panic→UART (g2), MMIO codegen (g12), x86_64 SSE2 (g9)
- [ ] B4 P1-D5 complete

## os/ integration (bridge demo kernel <-> compiler runtime)
- [x] O1: extract uart.rs -> shared no_std crate `logicodex-os`; kernel uses real `logicodex_os::uart` for all serial (verified: boot serial driven by extracted compiler runtime, exit 33)
- [x] O2: port interrupts.rs (rich IDT-256 + EXCEPTION_NAMES + 8259 PIC remap/IRQ) to STABLE in logicodex-os (asm-stub dispatch, not nightly x86-interrupt ABI); kernel uses logicodex_os::interrupts::idt_init. Verified: EXC 3 (Breakpoint) named via real EXCEPTION_NAMES, exit 33
- [~] O3: panic + allocator ported to stable in logicodex-os. Allocator: BumpAllocator (atomic-CAS alloc) verified via Vec heap test (sum=15). Fixed pre-existing - [~] O3: panic.rs ported to stable in logicodex-os (fail-stop: UART report + sensitive GP-register wipe + halt; #[panic_handler] gated target_os=none). Verified: "!!! LOGICODEX PANIC !!!" + File:line. Next: allocator, startupself->- [~] O3: panic.rs ported to stable in logicodex-os (fail-stop: UART report + sensitive GP-register wipe + halt; #[panic_handler] gated target_os=none). Verified: "!!! LOGICODEX PANIC !!!" + File:line. Next: allocator, startupmut self bug. Next: startup
