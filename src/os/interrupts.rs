// =========================================================================
// Logicodex v1.44 — Interrupt Descriptor Table (IDT)
//
// x86_64 interrupt handling framework for freestanding targets.
// Sets up:
//   - IDT with 256 entries
//   - CPU exception handlers (0-31)
//   - Programmable Interrupt Controller (PIC) remapping
//   - Hardware IRQ handlers (32-47)
//
// # Safety
// This module uses `unsafe` extensively — it's inherently unsafe as it
// manipulates CPU control registers directly.
// =========================================================================

use core::arch::asm;

/// Number of IDT entries (256 = 32 CPU exceptions + 16 IRQs + 208 reserved).
const IDT_ENTRIES: usize = 256;

/// IDT entry — 128 bits (16 bytes) per entry.
#[repr(C, align(16))]
#[derive(Clone, Copy)]
struct IdtEntry {
    /// Bits 0-15 of handler address
    offset_low: u16,
    /// Code segment selector (GDT entry)
    selector: u16,
    /// IST index (0 = don't use IST)
    ist: u8,
    /// Type and attributes
    type_attr: u8,
    /// Bits 16-31 of handler address
    offset_mid: u16,
    /// Bits 32-63 of handler address
    offset_high: u32,
    /// Reserved (must be 0)
    _reserved: u32,
}

impl IdtEntry {
    /// Create a new IDT entry pointing to `handler`.
    const fn new(handler: u64) -> Self {
        IdtEntry {
            offset_low: (handler & 0xFFFF) as u16,
            selector: 0x08, // Kernel code segment (GDT[1])
            ist: 0,
            type_attr: 0x8E, // Present | Ring 0 | Interrupt gate
            offset_mid: ((handler >> 16) & 0xFFFF) as u16,
            offset_high: ((handler >> 32) & 0xFFFFFFFF) as u32,
            _reserved: 0,
        }
    }

    /// Create an unused (null) entry.
    const fn null() -> Self {
        IdtEntry {
            offset_low: 0,
            selector: 0,
            ist: 0,
            type_attr: 0,
            offset_mid: 0,
            offset_high: 0,
            _reserved: 0,
        }
    }
}

/// The Interrupt Descriptor Table.
/// Aligned to 16 bytes as required by the CPU.
#[repr(C, align(16))]
struct Idt {
    entries: [IdtEntry; IDT_ENTRIES],
}

impl Idt {
    const fn new() -> Self {
        Idt {
            entries: [IdtEntry::null(); IDT_ENTRIES],
        }
    }

    /// Set handler for interrupt vector `index`.
    fn set_handler(&mut self, index: usize, handler: u64) {
        assert!(index < IDT_ENTRIES);
        self.entries[index] = IdtEntry::new(handler);
    }
}

// ─── IDTR Register ───

/// IDT register — loaded with `lidt` instruction.
#[repr(C, packed)]
struct IdtRegister {
    /// Limit (size - 1)
    limit: u16,
    /// Base address of IDT
    base: u64,
}

// ─── Global IDT Instance ───

static mut IDT: Idt = Idt::new();
static mut IDTR: IdtRegister = IdtRegister { limit: 0, base: 0 };

// ─── Exception Names ───

const EXCEPTION_NAMES: [&str; 32] = [
    "Divide-by-zero",
    "Debug",
    "Non-maskable Interrupt",
    "Breakpoint",
    "Overflow",
    "Bound Range Exceeded",
    "Invalid Opcode",
    "Device Not Available",
    "Double Fault",
    "Coprocessor Segment Overrun",
    "Invalid TSS",
    "Segment Not Present",
    "Stack-Segment Fault",
    "General Protection Fault",
    "Page Fault",
    "Reserved",
    "x87 Floating-Point Exception",
    "Alignment Check",
    "Machine Check",
    "SIMD Floating-Point Exception",
    "Virtualization Exception",
    "Control Protection Exception",
    "Reserved", "Reserved", "Reserved", "Reserved",
    "Reserved", "Reserved", "Reserved", "Reserved",
    "Reserved", "Reserved",
];

// ─── PIC (8259 Programmable Interrupt Controller) ───

const PIC1_COMMAND: u16 = 0x20;
const PIC1_DATA: u16 = 0x21;
const PIC2_COMMAND: u16 = 0xA0;
const PIC2_DATA: u16 = 0xA1;

const ICW1_INIT: u8 = 0x10;
const ICW1_ICW4: u8 = 0x01;
const ICW4_8086: u8 = 0x01;
const PIC_EOI: u8 = 0x20;

/// Remap PIC to avoid conflicts with CPU exceptions (0-31).
/// IRQ0 → 32, IRQ1 → 33, ..., IRQ15 → 47
pub unsafe fn pic_remap() {
    // Save masks
    let a = inb(PIC1_DATA);
    let b = inb(PIC2_DATA);

    // Start initialization sequence (cascade mode)
    outb(PIC1_COMMAND, ICW1_INIT | ICW1_ICW4);
    outb(PIC2_COMMAND, ICW1_INIT | ICW1_ICW4);

    // ICW2: Vector offsets
    outb(PIC1_DATA, 0x20); // Master: vectors 32-39
    outb(PIC2_DATA, 0x28); // Slave: vectors 40-47

    // ICW3: Cascade identity
    outb(PIC1_DATA, 0x04); // Slave at IRQ2
    outb(PIC2_DATA, 0x02); // Cascade identity

    // ICW4: 8086 mode
    outb(PIC1_DATA, ICW4_8086);
    outb(PIC2_DATA, ICW4_8086);

    // Restore masks (disable all IRQs initially)
    outb(PIC1_DATA, a);
    outb(PIC2_DATA, b);
}

/// Send End-of-Interrupt (EOI) signal to PIC.
pub unsafe fn pic_eoi(irq: u8) {
    if irq >= 8 {
        outb(PIC2_COMMAND, PIC_EOI);
    }
    outb(PIC1_COMMAND, PIC_EOI);
}

/// Enable a specific IRQ line (0-15).
pub unsafe fn irq_enable(irq: u8) {
    let port = if irq < 8 { PIC1_DATA } else { PIC2_DATA };
    let irq_line = irq % 8;
    let mask = inb(port);
    outb(port, mask & !(1 << irq_line));
}

/// Disable a specific IRQ line (0-15).
pub unsafe fn irq_disable(irq: u8) {
    let port = if irq < 8 { PIC1_DATA } else { PIC2_DATA };
    let irq_line = irq % 8;
    let mask = inb(port);
    outb(port, mask | (1 << irq_line));
}

// ─── IDT Initialization ───

/// Initialize the Interrupt Descriptor Table.
/// Must be called once during early startup (after GDT, before interrupts).
pub unsafe fn idt_init() {
    // Set CPU exception handlers (0-31)
    for i in 0..32 {
        let handler = EXCEPTION_HANDLERS[i];
        IDT.set_handler(i, handler as u64);
    }

    // Set IRQ handlers (32-47)
    for i in 0..16 {
        IDT.set_handler(32 + i, IRQ_HANDLERS[i] as u64);
    }

    // Load IDTR
    let idt_ptr = &IDT as *const _ as u64;
    IDTR.limit = (core::mem::size_of::<Idt>() - 1) as u16;
    IDTR.base = idt_ptr;

    asm!("lidt [{}]", in(reg) &IDTR, options(readonly, nostack, preserves_flags));
}

// ─── Default Exception Handler ───

/// Default handler for CPU exceptions.
/// Prints exception info via UART and halts.
#[unsafe(no_mangle)]
extern "x86-interrupt" fn default_exception_handler(frame: InterruptStackFrame) {
    let vector = frame.vector as usize;
    let name = if vector < 32 {
        EXCEPTION_NAMES[vector]
    } else {
        "Unknown"
    };

    crate::os::uart::uart_puts("\n!!! CPU EXCEPTION !!!\n");
    crate::os::uart::uart_puts("  Vector: ");
    crate::os::uart::uart_decimal(vector as u64);
    crate::os::uart::uart_puts(" (");
    crate::os::uart::uart_puts(name);
    crate::os::uart::uart_puts(")\n");
    crate::os::uart::uart_puts("  RIP: ");
    crate::os::uart::uart_hex(frame.rip);
    crate::os::uart::uart_puts("\n");
    crate::os::uart::uart_puts("  RSP: ");
    crate::os::uart::uart_hex(frame.rsp);
    crate::os::uart::uart_puts("\n");

    // Halt — exception is fatal
    super::startup::halt();
}

/// Interrupt stack frame — pushed by CPU on interrupt.
#[repr(C)]
pub struct InterruptStackFrame {
    pub rip: u64,
    pub cs: u64,
    pub rflags: u64,
    pub rsp: u64,
    pub ss: u64,
    pub vector: u64, // We push this manually
}

// ─── Exception Handler Stubs ───
// These are generated by macro — one per exception vector.

macro_rules! exception_handler {
    ($name:ident, $vector:expr) => {
        #[unsafe(no_mangle)]
        extern "x86-interrupt" fn $name(frame: InterruptStackFrame) {
            let mut f = frame;
            f.vector = $vector;
            default_exception_handler(f);
        }
    };
}

exception_handler!(exc_divide_by_zero, 0);
exception_handler!(exc_debug, 1);
exception_handler!(exc_nmi, 2);
exception_handler!(exc_breakpoint, 3);
exception_handler!(exc_overflow, 4);
exception_handler!(exc_bound_range, 5);
exception_handler!(exc_invalid_opcode, 6);
exception_handler!(exc_device_not_available, 7);
exception_handler!(exc_double_fault, 8);
exception_handler!(exc_invalid_tss, 10);
exception_handler!(exc_segment_not_present, 11);
exception_handler!(exc_stack_segment, 12);
exception_handler!(exc_general_protection, 13);
exception_handler!(exc_page_fault, 14);

static EXCEPTION_HANDLERS: [fn(InterruptStackFrame); 32] = [
    exc_divide_by_zero, exc_debug, exc_nmi, exc_breakpoint,
    exc_overflow, exc_bound_range, exc_invalid_opcode, exc_device_not_available,
    exc_double_fault, default_handler_9, exc_invalid_tss, exc_segment_not_present,
    exc_stack_segment, exc_general_protection, exc_page_fault, default_handler_15,
    default_handler_16, default_handler_17, default_handler_18, default_handler_19,
    default_handler_20, default_handler_21, default_handler_22, default_handler_23,
    default_handler_24, default_handler_25, default_handler_26, default_handler_27,
    default_handler_28, default_handler_29, default_handler_30, default_handler_31,
];

// Stub handlers for reserved vectors
macro_rules! default_handler {
    ($name:ident, $vector:expr) => {
        #[unsafe(no_mangle)]
        extern "x86-interrupt" fn $name(frame: InterruptStackFrame) {
            let mut f = frame;
            f.vector = $vector;
            default_exception_handler(f);
        }
    };
}

default_handler!(default_handler_9, 9);
default_handler!(default_handler_15, 15);
default_handler!(default_handler_16, 16);
default_handler!(default_handler_17, 17);
default_handler!(default_handler_18, 18);
default_handler!(default_handler_19, 19);
default_handler!(default_handler_20, 20);
default_handler!(default_handler_21, 21);
default_handler!(default_handler_22, 22);
default_handler!(default_handler_23, 23);
default_handler!(default_handler_24, 24);
default_handler!(default_handler_25, 25);
default_handler!(default_handler_26, 26);
default_handler!(default_handler_27, 27);
default_handler!(default_handler_28, 28);
default_handler!(default_handler_29, 29);
default_handler!(default_handler_30, 30);
default_handler!(default_handler_31, 31);

// ─── IRQ Handlers ───

macro_rules! irq_handler {
    ($name:ident, $irq:expr) => {
        #[unsafe(no_mangle)]
        extern "x86-interrupt" fn $name(_frame: InterruptStackFrame) {
            // Send EOI
            unsafe { pic_eoi($irq); }
        }
    };
}

irq_handler!(irq_0, 0);
irq_handler!(irq_1, 1);
irq_handler!(irq_2, 2);
irq_handler!(irq_3, 3);
irq_handler!(irq_4, 4);
irq_handler!(irq_5, 5);
irq_handler!(irq_6, 6);
irq_handler!(irq_7, 7);
irq_handler!(irq_8, 8);
irq_handler!(irq_9, 9);
irq_handler!(irq_10, 10);
irq_handler!(irq_11, 11);
irq_handler!(irq_12, 12);
irq_handler!(irq_13, 13);
irq_handler!(irq_14, 14);
irq_handler!(irq_15, 15);

static IRQ_HANDLERS: [fn(InterruptStackFrame); 16] = [
    irq_0, irq_1, irq_2, irq_3, irq_4, irq_5, irq_6, irq_7,
    irq_8, irq_9, irq_10, irq_11, irq_12, irq_13, irq_14, irq_15,
];

// ─── Port I/O helpers ───

#[inline]
unsafe fn inb(port: u16) -> u8 {
    let result: u8;
    asm!("in al, dx", out("al") result, in("dx") port, options(nomem, nostack, preserves_flags));
    result
}

#[inline]
unsafe fn outb(port: u16, value: u8) {
    asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack, preserves_flags));
}

// =========================================================================
// Tests
// =========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idt_entry_size() {
        assert_eq!(core::mem::size_of::<IdtEntry>(), 16, "IDT entry must be 16 bytes");
    }

    #[test]
    fn idt_entry_new() {
        let entry = IdtEntry::new(0x123456789ABCDEF0);
        assert_eq!(entry.offset_low, 0xDEF0);
        assert_eq!(entry.offset_mid, 0x9ABC);
        assert_eq!(entry.offset_high, 0x12345678);
        assert_eq!(entry.selector, 0x08);
        assert_eq!(entry.type_attr, 0x8E);
    }

    #[test]
    fn idt_total_size() {
        assert_eq!(core::mem::size_of::<Idt>(), 256 * 16, "IDT must be 4096 bytes");
    }

    #[test]
    fn idt_register_size() {
        assert_eq!(core::mem::size_of::<IdtRegister>(), 10, "IDTR must be 10 bytes");
    }

    #[test]
    fn exception_names_count() {
        assert_eq!(EXCEPTION_NAMES.len(), 32, "Must have 32 exception names");
    }

    #[test]
    fn pic_ports_correct() {
        assert_eq!(PIC1_COMMAND, 0x20);
        assert_eq!(PIC1_DATA, 0x21);
        assert_eq!(PIC2_COMMAND, 0xA0);
        assert_eq!(PIC2_DATA, 0xA1);
    }

    #[test]
    fn irq_handlers_count() {
        assert_eq!(IRQ_HANDLERS.len(), 16, "Must have 16 IRQ handlers");
    }

    #[test]
    fn exception_handlers_count() {
        assert_eq!(EXCEPTION_HANDLERS.len(), 32, "Must have 32 exception handlers");
    }
}
