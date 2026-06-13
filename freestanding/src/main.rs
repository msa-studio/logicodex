#![no_std]
#![no_main]

use core::arch::global_asm;
use core::panic::PanicInfo;
use logicodex_os::{interrupts, uart};

global_asm!(
    r#"
.section .multiboot_header, "a"
.align 4
    .long 0x1BADB002
    .long 0x00000000
    .long 0 - 0x1BADB002

.section .text
.code32
.global _start
_start:
    cli
    mov esp, offset stack_top

    mov dx, 0x3F8
    mov al, 0x42
    out dx, al

    cld
    mov edi, offset pml4
    xor eax, eax
    mov ecx, (3 * 4096) / 4
    rep stosd

    mov edi, offset pml4
    mov eax, offset pdpt
    or eax, 0x3
    mov [edi], eax

    mov edi, offset pdpt
    mov eax, offset pd
    or eax, 0x3
    mov [edi], eax

    mov edi, offset pd
    mov eax, 0x83
    mov ecx, 512
.fill_pd:
    mov [edi], eax
    add eax, 0x200000
    add edi, 8
    loop .fill_pd

    mov eax, offset pml4
    mov cr3, eax

    mov eax, cr4
    or eax, 1 << 5
    mov cr4, eax

    mov ecx, 0xC0000080
    rdmsr
    or eax, 1 << 8
    wrmsr

    mov eax, cr0
    or eax, 1 << 31
    mov cr0, eax

    lgdt [gdt64_pointer]
    push 0x08
    mov eax, offset long_mode_start
    push eax
    retf

.code64
long_mode_start:
    mov ax, 0x10
    mov ss, ax
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    call kmain
.hang:
    hlt
    jmp .hang

.section .rodata
.align 8
gdt64:
    .quad 0x0000000000000000
    .quad 0x00209A0000000000
    .quad 0x0000920000000000
gdt64_pointer:
    .word gdt64_pointer - gdt64 - 1
    .quad gdt64

.section .bss
.align 4096
pml4: .skip 4096
pdpt: .skip 4096
pd:   .skip 4096
.align 16
stack_bottom: .skip 16384
stack_top:
"#
);

#[inline]
unsafe fn outb(port: u16, val: u8) {
    core::arch::asm!("out dx, al", in("dx") port, in("al") val,
        options(nomem, nostack, preserves_flags));
}

#[no_mangle]
pub extern "C" fn kmain() -> ! {
    unsafe { uart::uart_init(); }
    uart::uart_puts("boot\r\n");
    unsafe { interrupts::idt_init(); }
    uart::uart_puts("idt\r\n");
    unsafe { core::arch::asm!("int3") };
    uart::uart_puts("Logicodex\r\n");
    unsafe { outb(0xf4, 0x10); }
    loop { unsafe { core::arch::asm!("hlt") }; }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    uart::uart_puts("\r\nPANIC");
    if let Some(loc) = info.location() {
        uart::uart_puts(" at ");
        uart::uart_puts(loc.file());
        uart::uart_puts(":");
        uart::uart_decimal(loc.line() as u64);
    }
    uart::uart_newline();
    loop { unsafe { core::arch::asm!("hlt") }; }
}
