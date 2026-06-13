//! Freestanding panic handler — STABLE port of the compiler's src/os/panic.rs.
//!
//! Fail-stop for bare-metal: report to UART, wipe sensitive GP registers
//! (security: clear potential crypto-key material), then halt.
//!
//! Differences from the original (toolchain constraints on stable 1.75):
//!   * `info.message()` omitted (PanicInfo::message is nightly-only here).
//!   * xmm register wipe omitted (needs SSE enabled in the kernel; deferred).
//! Gated on `target_os = "none"` so a hosted std build keeps its own handler.

use crate::uart;
use core::panic::PanicInfo;

#[cfg(target_os = "none")]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    uart::uart_puts("\r\n\r\n!!! LOGICODEX PANIC !!!\r\n");
    if let Some(loc) = info.location() {
        uart::uart_puts("  File: ");
        uart::uart_puts(loc.file());
        uart::uart_puts(":");
        uart::uart_decimal(loc.line() as u64);
        uart::uart_newline();
    }
    uart::uart_puts("  Strategy: clear_sensitive_registers_and_halt\r\n");
    uart::uart_puts("!!! HALTING CPU !!!\r\n");

    // Security: wipe GP registers that may hold secrets before halting.
    unsafe {
        core::arch::asm!(
            "xor rax, rax",
            "xor rbx, rbx",
            "xor rsi, rsi",
            "xor r8, r8",
            "xor r9, r9",
            "xor r10, r10",
            "xor r11, r11",
            "xor r12, r12",
            "xor r13, r13",
            "xor r14, r14",
            "xor r15, r15",
            options(nomem, nostack)
        );
    }
    crate::startup::halt()
}
