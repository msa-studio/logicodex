/* =========================================================================
   Logicodex — Linker Script for Freestanding x86_64
   
   Memory layout for bare-metal (no OS) execution:
   
   0x00000000 - 0x0009FFFF : Low 640KB (reserved, BIOS)
   0x000A0000 - 0x000FFFFF : VGA frame buffer + ROM (reserved)
   0x00100000 - 0x001FFFFF : Logicodex kernel code + data (1MB - 2MB)
   0x00200000 -              : Heap grows upward from 2MB
   
   Stack: grows downward from 0x200000 toward 0x100000
   
   Usage:
     ld -T lib/linker_scripts/x86_64-freestanding.ld \
        -o kernel.elf \
        target/x86_64-unknown-none/release/logicodex \
        lib/startup/crt0.o
   ========================================================================= */

ENTRY(_start)

SECTIONS
{
    /* ─── Kernel load address (1MB) ─── */
    . = 0x100000;

    /* ─── Text: executable code + read-only data ─── */
    .text : ALIGN(4K)
    {
        *(.text._start)       /* Entry point first */
        *(.text*)             /* All other code */
        *(.rodata*)           /* Read-only data */
    }

    /* ─── Data: initialized mutable data ─── */
    . = ALIGN(4K);
    __data_start = .;
    .data : AT(__data_lma) ALIGN(4K)
    {
        *(.data*)
    }
    __data_end = .;

    /* ─── BSS: zero-initialized data ─── */
    . = ALIGN(4K);
    __bss_start = .;
    .bss : ALIGN(4K)
    {
        *(.bss*)
        *(COMMON)             /* Fortran-style common symbols */
    }
    __bss_end = .;

    /* ─── LMA for data (follows text in ROM image) ─── */
    __data_lma = LOADADDR(.data);

    /* ─── Stack: defined symbol for startup code ─── */
    __stack_top = 0x200000;

    /* ─── Heap start: after BSS ─── */
    . = ALIGN(4K);
    __heap_start = .;

    /* ─── Discard unused sections ─── */
    /DISCARD/ :
    {
        *(.comment)
        *(.note*)
        *(.eh_frame*)
        *(.gnu*)
    }
}
