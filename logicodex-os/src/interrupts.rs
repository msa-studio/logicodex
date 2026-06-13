//! Interrupt Descriptor Table (IDT) for freestanding x86_64 — STABLE port.
//!
//! Ported from the compiler's `src/os/interrupts.rs`, but using stable Rust:
//! exception entry points are assembly stubs (uniform error-code frame) that
//! dispatch to a common Rust handler, instead of the nightly-only
//! `extern "x86-interrupt"` ABI. Keeps the rich structure: 256-entry IDT,
//! named CPU exceptions (0-31), and 8259 PIC remap / IRQ control (32-47).

use crate::uart;
use core::arch::{asm, global_asm};

const IDT_ENTRIES: usize = 256;

#[repr(C, align(16))]
#[derive(Clone, Copy)]
struct IdtEntry {
    offset_low: u16,
    selector: u16,
    ist: u8,
    type_attr: u8,
    offset_mid: u16,
    offset_high: u32,
    _reserved: u32,
}
impl IdtEntry {
    const fn new(handler: u64) -> Self {
        IdtEntry {
            offset_low: (handler & 0xFFFF) as u16,
            selector: 0x08, // kernel code segment (GDT[1])
            ist: 0,
            type_attr: 0x8E, // present | ring 0 | interrupt gate
            offset_mid: ((handler >> 16) & 0xFFFF) as u16,
            offset_high: ((handler >> 32) & 0xFFFFFFFF) as u32,
            _reserved: 0,
        }
    }
    const fn null() -> Self {
        IdtEntry { offset_low: 0, selector: 0, ist: 0, type_attr: 0,
                   offset_mid: 0, offset_high: 0, _reserved: 0 }
    }
}

#[repr(C, align(16))]
struct Idt {
    entries: [IdtEntry; IDT_ENTRIES],
}
impl Idt {
    const fn new() -> Self {
        Idt { entries: [IdtEntry::null(); IDT_ENTRIES] }
    }
    fn set_handler(&mut self, index: usize, handler: u64) {
        self.entries[index] = IdtEntry::new(handler);
    }
}

#[repr(C, packed)]
struct IdtRegister {
    limit: u16,
    base: u64,
}

static mut IDT: Idt = Idt::new();
static mut IDTR: IdtRegister = IdtRegister { limit: 0, base: 0 };

const EXCEPTION_NAMES: [&str; 32] = [
    "Divide-by-zero", "Debug", "Non-maskable Interrupt", "Breakpoint",
    "Overflow", "Bound Range Exceeded", "Invalid Opcode", "Device Not Available",
    "Double Fault", "Coprocessor Segment Overrun", "Invalid TSS",
    "Segment Not Present", "Stack-Segment Fault", "General Protection Fault",
    "Page Fault", "Reserved", "x87 Floating-Point Exception", "Alignment Check",
    "Machine Check", "SIMD Floating-Point Exception", "Virtualization Exception",
    "Control Protection Exception", "Reserved", "Reserved", "Reserved",
    "Reserved", "Reserved", "Reserved", "Reserved", "Reserved", "Reserved",
    "Reserved",
];

// ─── 8259 PIC ───
const PIC1_COMMAND: u16 = 0x20;
const PIC1_DATA: u16 = 0x21;
const PIC2_COMMAND: u16 = 0xA0;
const PIC2_DATA: u16 = 0xA1;
const ICW1_INIT: u8 = 0x10;
const ICW1_ICW4: u8 = 0x01;
const ICW4_8086: u8 = 0x01;
const PIC_EOI: u8 = 0x20;

/// Remap PIC so IRQ0..15 land at vectors 32..47 (clear of CPU exceptions).
pub unsafe fn pic_remap() {
    let a = inb(PIC1_DATA);
    let b = inb(PIC2_DATA);
    outb(PIC1_COMMAND, ICW1_INIT | ICW1_ICW4);
    outb(PIC2_COMMAND, ICW1_INIT | ICW1_ICW4);
    outb(PIC1_DATA, 0x20);
    outb(PIC2_DATA, 0x28);
    outb(PIC1_DATA, 0x04);
    outb(PIC2_DATA, 0x02);
    outb(PIC1_DATA, ICW4_8086);
    outb(PIC2_DATA, ICW4_8086);
    outb(PIC1_DATA, a);
    outb(PIC2_DATA, b);
}

/// Send End-of-Interrupt to the PIC(s) for `irq`.
pub unsafe fn pic_eoi(irq: u8) {
    if irq >= 8 {
        outb(PIC2_COMMAND, PIC_EOI);
    }
    outb(PIC1_COMMAND, PIC_EOI);
}

/// Enable a specific IRQ line (0-15).
pub unsafe fn irq_enable(irq: u8) {
    let port = if irq < 8 { PIC1_DATA } else { PIC2_DATA };
    let line = irq % 8;
    let mask = inb(port);
    outb(port, mask & !(1 << line));
}

/// Disable a specific IRQ line (0-15).
pub unsafe fn irq_disable(irq: u8) {
    let port = if irq < 8 { PIC1_DATA } else { PIC2_DATA };
    let line = irq % 8;
    let mask = inb(port);
    outb(port, mask | (1 << line));
}

// ─── Exception entry stubs (stable, no nightly ABI) ───
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
    call exception_dispatch
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

extern "C" {
    static isr_stub_table: [u64; 32];
}

/// Install the 32 CPU-exception handlers and load the IDTR.
/// Call once after the GDT is active, before enabling interrupts.
pub unsafe fn idt_init() {
    let mut i = 0usize;
    while i < 32 {
        IDT.set_handler(i, isr_stub_table[i]);
        i += 1;
    }
    IDTR.limit = (core::mem::size_of::<Idt>() - 1) as u16;
    IDTR.base = core::ptr::addr_of!(IDT) as u64;
    asm!("lidt [{}]", in(reg) core::ptr::addr_of!(IDTR),
        options(readonly, nostack, preserves_flags));
}

/// Common Rust dispatcher for all CPU exceptions.
/// Prints the named exception; returns for breakpoint (#BP, a trap) so
/// execution continues, halts for fatal faults.
#[no_mangle]
extern "C" fn exception_dispatch(vector: u64, _error: u64) {
    let v = vector as usize;
    let name = if v < 32 { EXCEPTION_NAMES[v] } else { "Unknown" };
    uart::uart_puts("EXC ");
    uart::uart_decimal(vector);
    uart::uart_puts(" (");
    uart::uart_puts(name);
    uart::uart_puts(")\r\n");
    if vector == 3 {
        return; // breakpoint trap: return to continue
    }
    loop {
        unsafe { asm!("hlt") };
    }
}

// ─── Port I/O ───
#[inline]
unsafe fn inb(port: u16) -> u8 {
    let result: u8;
    asm!("in al, dx", out("al") result, in("dx") port,
        options(nomem, nostack, preserves_flags));
    result
}
#[inline]
unsafe fn outb(port: u16, value: u8) {
    asm!("out dx, al", in("dx") port, in("al") value,
        options(nomem, nostack, preserves_flags));
}
