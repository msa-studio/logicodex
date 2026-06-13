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
// isr_stub_table removed: absolute .quad relocs (R_X86_64_64) break
// static-model linking at 1MB. Handler addresses are taken in Rust instead
// (see idt_init), which emits RIP-relative LEA — no absolute data relocs.
"#
);

extern "C" {
    fn isr0();
    fn isr1();
    fn isr2();
    fn isr3();
    fn isr4();
    fn isr5();
    fn isr6();
    fn isr7();
    fn isr8();
    fn isr9();
    fn isr10();
    fn isr11();
    fn isr12();
    fn isr13();
    fn isr14();
    fn isr15();
    fn isr16();
    fn isr17();
    fn isr18();
    fn isr19();
    fn isr20();
    fn isr21();
    fn isr22();
    fn isr23();
    fn isr24();
    fn isr25();
    fn isr26();
    fn isr27();
    fn isr28();
    fn isr29();
    fn isr30();
    fn isr31();
}

/// Install the 32 CPU-exception handlers and load the IDTR.
/// Call once after the GDT is active, before enabling interrupts.
pub unsafe fn idt_init() {
    // Take each stub address individually: rustc emits a RIP-relative LEA
    // per call, avoiding any absolute pointer array in .rodata.
    IDT.set_handler(0, isr0 as usize as u64);
    IDT.set_handler(1, isr1 as usize as u64);
    IDT.set_handler(2, isr2 as usize as u64);
    IDT.set_handler(3, isr3 as usize as u64);
    IDT.set_handler(4, isr4 as usize as u64);
    IDT.set_handler(5, isr5 as usize as u64);
    IDT.set_handler(6, isr6 as usize as u64);
    IDT.set_handler(7, isr7 as usize as u64);
    IDT.set_handler(8, isr8 as usize as u64);
    IDT.set_handler(9, isr9 as usize as u64);
    IDT.set_handler(10, isr10 as usize as u64);
    IDT.set_handler(11, isr11 as usize as u64);
    IDT.set_handler(12, isr12 as usize as u64);
    IDT.set_handler(13, isr13 as usize as u64);
    IDT.set_handler(14, isr14 as usize as u64);
    IDT.set_handler(15, isr15 as usize as u64);
    IDT.set_handler(16, isr16 as usize as u64);
    IDT.set_handler(17, isr17 as usize as u64);
    IDT.set_handler(18, isr18 as usize as u64);
    IDT.set_handler(19, isr19 as usize as u64);
    IDT.set_handler(20, isr20 as usize as u64);
    IDT.set_handler(21, isr21 as usize as u64);
    IDT.set_handler(22, isr22 as usize as u64);
    IDT.set_handler(23, isr23 as usize as u64);
    IDT.set_handler(24, isr24 as usize as u64);
    IDT.set_handler(25, isr25 as usize as u64);
    IDT.set_handler(26, isr26 as usize as u64);
    IDT.set_handler(27, isr27 as usize as u64);
    IDT.set_handler(28, isr28 as usize as u64);
    IDT.set_handler(29, isr29 as usize as u64);
    IDT.set_handler(30, isr30 as usize as u64);
    IDT.set_handler(31, isr31 as usize as u64);
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
    crate::startup::halt()
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
