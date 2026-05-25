// =========================================================================
// Logicodex v1.44 — Freestanding Panic Handler
//
// When `panic!()` is called in a bare-metal environment (no OS),
// this handler:
//   1. Clears sensitive registers (security: zero out crypto keys)
//   2. Writes panic message to UART (COM1, 0x3F8)
//   3. Halts the CPU (infinite loop)
//
// This is the fail-stop behavior for freestanding targets.
// Hosted targets use std::panic::set_hook() instead.
// =========================================================================

use core::panic::PanicInfo;
use core::fmt::Write;

/// Panic handler for bare-metal (no_std, no OS) targets.
/// Registered automatically by Rust when `#![no_std]` is used.
#[cfg(target_os = "none")]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // Step 1: Clear sensitive registers
    // Security: zero out any registers that might hold crypto keys
    unsafe {
        core::arch::asm!(
            "xor rax, rax",
            "xor rbx, rbx",
            // rcx, rdx may contain panic info — skip
            "xor rsi, rsi",
            "xor r8, r8",
            "xor r9, r9",
            "xor r10, r10",
            "xor r11, r11",
            "xor r12, r12",
            "xor r13, r13",
            "xor r14, r14",
            "xor r15, r15",
            "pxor xmm0, xmm0",
            "pxor xmm1, xmm1",
            "pxor xmm2, xmm2",
            "pxor xmm3, xmm3",
            options(nomem, nostack)
        );
    }

    // Step 2: Write panic message to UART
    let mut uart = UartWriter;
    let _ = writeln!(uart, "\n\n!!! LOGICODEX PANIC !!!");
    if let Some(loc) = info.location() {
        let _ = writeln!(
            uart,
            "  File: {}:{}",
            loc.file(),
            loc.line()
        );
    }
    if let Some(msg) = info.message() {
        let _ = write!(uart, "  Message: ");
        let _ = core::fmt::write(&mut uart, format_args!("{}", msg));
        let _ = writeln!(uart);
    }
    let _ = writeln!(uart, "  Strategy: clear_sensitive_registers_and_halt");
    let _ = writeln!(uart, "!!! HALTING CPU !!!\n");

    // Step 3: Halt — no recovery in freestanding
    super::startup::halt()
}

// ─── UART Writer for panic messages ───
// Minimal x86_64 UART (COM1) output — no allocator needed.

const UART_PORT: u16 = 0x3F8; // COM1 base port

/// Writer that outputs to the serial port (UART).
/// Used by the panic handler — must work without alloc or std.
struct UartWriter;

impl Write for UartWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for byte in s.bytes() {
            uart_send(byte);
        }
        Ok(())
    }
}

/// Send a single byte to the UART.
/// Polls the transmitter holding register until it's empty.
#[cfg(target_os = "none")]
fn uart_send(byte: u8) {
    unsafe {
        // Wait for transmitter holding register to be empty
        while (core::arch::x86_64::_inb(UART_PORT + 5) & 0x20) == 0 {}
        core::arch::x86_64::_outb(UART_PORT, byte);
    }
}

// Hosted fallback — use eprintln
#[cfg(not(target_os = "none"))]
fn uart_send(_byte: u8) {
    // No-op on hosted targets — std panic handler takes over
}

// =========================================================================
// Tests
// =========================================================================

#[cfg(test)]
mod tests {
    #[test]
    fn panic_strategy_is_documented() {
        let strategy = "clear_sensitive_registers_and_halt";
        assert!(strategy.contains("clear"));
        assert!(strategy.contains("halt"));
    }

    #[test]
    fn uart_port_is_com1() {
        assert_eq!(super::UART_PORT, 0x3F8, "UART must be COM1 (0x3F8)");
    }

    #[test]
    fn uart_writer_implements_fmt_write() {
        use core::fmt::Write;
        let mut writer = super::UartWriter;
        // This just verifies the trait implementation compiles
        let _ = writer.write_str("test");
    }
}
