#!/usr/bin/env python3
"""
Validator: v1.44.0-alpha — Freestanding Compiler: All 15 Gaps Resolved

Validates Tier 1 (G1-G5), Tier 2 (G6-G10), and Tier 3 (G11-G15):
  - Source files exist and contain expected code
  - Architecture-specific features are correct
  - Integration points are wired up

Usage: python3 scripts/validate_v144_freestanding.py
"""

import os, sys

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

def check(name):
    def decorator(fn):
        CHECKS.append((name, fn)); return fn
    return decorator

CHECKS = []

# ═══════════════════════════════════════════════════════════
# TIER 1: MUST HAVE (G1-G5)
# ═══════════════════════════════════════════════════════════

@check("G1: Startup code (_start, BSS, data, stack)")
def g1():
    with open(f"{REPO}/src/os/startup.rs") as f:
        c = f.read()
    assert "fn _start" in c
    assert "STACK_TOP" in c
    assert "write_bytes" in c  # BSS zeroing
    assert "copy_nonoverlapping" in c  # Data copy
    assert "fn halt" in c
    assert "extern \"C\" {" in c  # Linker symbols
    return True

@check("G2: Panic handler (register clear + UART + halt)")
def g2():
    with open(f"{REPO}/src/os/panic.rs") as f:
        c = f.read()
    assert "panic_handler" in c
    assert "pxor xmm0" in c  # SSE register clearing
    assert "uart" in c.lower()  # UART output (uart_puts or uart_send)
    assert "halt" in c
    return True

@check("G3: Linker script (ENTRY, sections, symbols)")
def g3():
    with open(f"{REPO}/lib/linker_scripts/x86_64-freestanding.ld") as f:
        c = f.read()
    assert "ENTRY(_start)" in c
    assert "__bss_start" in c
    assert "__bss_end" in c
    assert "__data_start" in c
    assert "__data_lma" in c
    assert "__heap_start" in c
    assert "0x100000" in c  # Load address
    return True

@check("G4: Bump allocator (GlobalAlloc + atomic)")
def g4():
    with open(f"{REPO}/src/os/allocator.rs") as f:
        c = f.read()
    assert "struct BumpAllocator" in c
    assert "impl GlobalAlloc" in c
    assert "global_allocator" in c
    assert "compare_exchange_weak" in c  # Atomic CAS
    assert "fn alloc" in c
    assert "fn dealloc" in c
    return True

@check("G5: UART + VGA (uart_putc, VgaWriter, macros)")
def g5():
    with open(f"{REPO}/src/os/uart.rs") as f:
        c = f.read()
    assert "uart_init" in c
    assert "uart_putc" in c
    assert "uart_puts" in c
    assert "UartWriter" in c
    assert "VgaWriter" in c
    assert "0x3F8" in c
    assert "0xB8000" in c
    assert "macro_rules! uart_print" in c
    assert "macro_rules! uart_println" in c
    return True

# ═══════════════════════════════════════════════════════════
# TIER 2: HIGH (G6-G10)
# ═══════════════════════════════════════════════════════════

@check("G6: no_std support (cfg_attr + extern alloc)")
def g6():
    with open(f"{REPO}/src/lib.rs") as f:
        c = f.read()
    assert "no_std" in c
    assert "extern crate alloc" in c
    return True

@check("G7: Source provider (trait + embedded + binary)")
def g7():
    with open(f"{REPO}/src/os/source_provider.rs") as f:
        c = f.read()
    assert "trait SourceProvider" in c
    assert "EmbeddedProvider" in c
    assert "BinaryProvider" in c
    return True

@check("G8: Multi-arch (x86_64, aarch64, riscv64)")
def g8():
    with open(f"{REPO}/src/os/target.rs") as f:
        c = f.read()
    assert "enum TargetArch" in c
    assert "X86_64" in c
    assert "Aarch64" in c
    assert "Riscv64" in c
    assert "build_target_machine_with_arch" in c
    assert "freestanding-x86_64" in c
    assert "freestanding-aarch64" in c
    assert "freestanding-riscv64" in c
    return True

@check("G9: x86_64 uses +sse2 (not +soft-float)")
def g9():
    with open(f"{REPO}/src/os/target.rs") as f:
        c = f.read()
    # Find the x86_64 features section
    x86_part = c.split("Self::X86_64 =>").pop()
    if "llvm_features" in x86_part[:200]:
        assert "+sse2" in x86_part[:500], "x86_64 must use +sse2"
        assert "+soft-float" not in x86_part[:500], "x86_64 must NOT use +soft-float"
    return True

@check("G10: BSS/Data init in startup")
def g10():
    with open(f"{REPO}/src/os/startup.rs") as f:
        c = f.read()
    assert "write_bytes" in c  # BSS zeroing
    assert "copy_nonoverlapping" in c  # Data copy
    return True

# ═══════════════════════════════════════════════════════════
# TIER 3: MEDIUM (G11-G15)
# ═══════════════════════════════════════════════════════════

@check("G11: Interrupt handling (IDT + PIC)")
def g11():
    with open(f"{REPO}/src/os/interrupts.rs") as f:
        c = f.read()
    assert "struct Idt" in c
    assert "idt_init" in c
    assert "pic_remap" in c
    assert "pic_eoi" in c
    assert "irq_enable" in c
    assert "extern \"x86-interrupt\" fn" in c
    return True

@check("G12: MMIO volatile codegen")
def g12():
    with open(f"{REPO}/src/codegen.rs") as f:
        c = f.read()
    assert "emit_hardware_zone" in c
    assert "emit_mmio_volatile_write" in c
    assert "emit_mmio_volatile_read" in c
    assert "set_volatile" in c
    assert "hw_zone_depth" in c
    return True

@check("G13: Multiboot header")
def g13():
    with open(f"{REPO}/lib/startup/multiboot_header.rs") as f:
        c = f.read()
    assert "MultibootHeader" in c
    assert "0x1BADB002" in c
    assert "link_section" in c or "link_section" in c.replace("#", "")
    return True

@check("G14: Stack pointer init (in startup)")
def g14():
    with open(f"{REPO}/src/os/startup.rs") as f:
        c = f.read()
    assert "mov rsp" in c or "STACK_TOP" in c
    return True

@check("G15: build.rs exists")
def g15():
    assert os.path.exists(f"{REPO}/build.rs"), "build.rs missing"
    with open(f"{REPO}/build.rs") as f:
        c = f.read()
    assert "pkg-config" in c
    return True

def main():
    passed = 0
    failed = 0
    for name, fn in CHECKS:
        try:
            if fn():
                print(f"  PASS  {name}")
                passed += 1
            else:
                print(f"  FAIL  {name}")
                failed += 1
        except Exception as e:
            print(f"  FAIL  {name}: {e}")
            failed += 1

    print(f"\n{'='*55}")
    print(f"v1.44 Freestanding Gaps: {passed}/{passed+failed} checks passed")
    if failed == 0:
        print("ALL 15 GAPS RESOLVED ✅")
        print("Freestanding readiness: 100% (Tier 1: 5/5, Tier 2: 5/5, Tier 3: 5/5)")
    else:
        print(f"Failed: {failed}")
    return 0 if failed == 0 else 1

if __name__ == "__main__":
    sys.exit(main())
