#![no_std]
#![no_main]

use core::arch::global_asm;
use core::panic::PanicInfo;

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

global_asm!(
    r#"
.section .text
.code64

.macro ISR_NOERR n
.global isr\n
isr\n:
    push 0
    push \n
    jmp isr_common
.endm

.macro ISR_ERR n
.global isr\n
isr\n:
    push \n
    jmp isr_common
.endm

ISR_NOERR 0
ISR_NOERR 1
ISR_NOERR 2
ISR_NOERR 3
ISR_NOERR 4
ISR_NOERR 5
ISR_NOERR 6
ISR_NOERR 7
ISR_ERR   8
ISR_NOERR 9
ISR_ERR   10
ISR_ERR   11
ISR_ERR   12
ISR_ERR   13
ISR_ERR   14
ISR_NOERR 15
ISR_NOERR 16
ISR_ERR   17
ISR_NOERR 18
ISR_NOERR 19
ISR_NOERR 20
ISR_ERR   21
ISR_NOERR 22
ISR_NOERR 23
ISR_NOERR 24
ISR_NOERR 25
ISR_NOERR 26
ISR_NOERR 27
ISR_NOERR 28
ISR_ERR   29
ISR_ERR   30
ISR_NOERR 31

isr_common:
    push rax
    push rcx
    push rdx
    push rbx
    push rbp
    push rsi
    push rdi
    push r8
    push r9
    push r10
    push r11
    push r12
    push r13
    push r14
    push r15
    mov rdi, [rsp + 15*8]
    mov rsi, [rsp + 16*8]
    call exception_handler
    pop r15
    pop r14
    pop r13
    pop r12
    pop r11
    pop r10
    pop r9
    pop r8
    pop rdi
    pop rsi
    pop rbp
    pop rbx
    pop rdx
    pop rcx
    pop rax
    add rsp, 16
    iretq

.section .rodata
.align 8
.global isr_stub_table
isr_stub_table:
    .quad isr0,  isr1,  isr2,  isr3,  isr4,  isr5,  isr6,  isr7
    .quad isr8,  isr9,  isr10, isr11, isr12, isr13, isr14, isr15
    .quad isr16, isr17, isr18, isr19, isr20, isr21, isr22, isr23
    .quad isr24, isr25, isr26, isr27, isr28, isr29, isr30, isr31
"#
);

const COM1: u16 = 0x3F8;

#[inline]
unsafe fn outb(port: u16, val: u8) {
    core::arch::asm!("out dx, al", in("dx") port, in("al") val,
        options(nomem, nostack, preserves_flags));
}
#[inline]
unsafe fn inb(port: u16) -> u8 {
    let val: u8;
    core::arch::asm!("in al, dx", out("al") val, in("dx") port,
        options(nomem, nostack, preserves_flags));
    val
}
fn serial_init() {
    unsafe {
        outb(COM1 + 1, 0x00);
        outb(COM1 + 3, 0x80);
        outb(COM1 + 0, 0x01);
        outb(COM1 + 1, 0x00);
        outb(COM1 + 3, 0x03);
        outb(COM1 + 2, 0xC7);
        outb(COM1 + 4, 0x0B);
    }
}
fn serial_putc(c: u8) {
    unsafe {
        while inb(COM1 + 5) & 0x20 == 0 {}
        outb(COM1, c);
    }
}
fn serial_puts(s: &[u8]) {
    for &c in s { serial_putc(c); }
}
fn hex_digit(n: u8) -> u8 {
    if n < 10 { b'0' + n } else { b'a' + (n - 10) }
}
fn serial_hex2(b: u8) {
    serial_putc(hex_digit(b >> 4));
    serial_putc(hex_digit(b & 0x0F));
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
struct IdtEntry {
    off_low: u16,
    sel: u16,
    ist: u8,
    flags: u8,
    off_mid: u16,
    off_high: u32,
    zero: u32,
}
impl IdtEntry {
    const ZERO: Self = Self {
        off_low: 0, sel: 0, ist: 0, flags: 0, off_mid: 0, off_high: 0, zero: 0,
    };
}

#[repr(C, packed)]
struct IdtPtr {
    limit: u16,
    base: u64,
}

static mut IDT: [IdtEntry; 256] = [IdtEntry::ZERO; 256];

extern "C" {
    static isr_stub_table: [u64; 32];
}

fn make_entry(handler: u64) -> IdtEntry {
    IdtEntry {
        off_low: handler as u16,
        sel: 0x08,
        ist: 0,
        flags: 0x8E,
        off_mid: (handler >> 16) as u16,
        off_high: (handler >> 32) as u32,
        zero: 0,
    }
}

fn idt_init() {
    unsafe {
        let mut v = 0usize;
        while v < 32 {
            IDT[v] = make_entry(isr_stub_table[v]);
            v += 1;
        }
        let ptr = IdtPtr {
            limit: (core::mem::size_of::<[IdtEntry; 256]>() - 1) as u16,
            base: core::ptr::addr_of!(IDT) as u64,
        };
        core::arch::asm!("lidt [{}]", in(reg) &ptr,
            options(readonly, nostack, preserves_flags));
    }
}

#[no_mangle]
pub extern "C" fn exception_handler(vector: u64, _error: u64) {
    serial_puts(b"EXC ");
    serial_hex2(vector as u8);
    serial_putc(b'\n');
}

#[no_mangle]
pub extern "C" fn kmain() -> ! {
    serial_init();
    serial_puts(b"boot\n");
    idt_init();
    serial_puts(b"idt\n");
    unsafe { core::arch::asm!("int3") };
    serial_puts(b"Logicodex\n");
    unsafe { outb(0xf4, 0x10); }
    loop { unsafe { core::arch::asm!("hlt") }; }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop { unsafe { core::arch::asm!("hlt") }; }
}
