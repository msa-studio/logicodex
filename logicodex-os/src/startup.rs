//! Freestanding startup helpers — partial port of src/os/startup.rs.
//!
//! Intentionally OMITTED: the crt0 `_start`. The kernel's multiboot boot stub
//! owns `_start` (32-bit entry + long-mode transition); startup.rs's `_start`
//! assumes an already-64-bit environment and would conflict.
//!
//! DEFERRED (#3, trigger-based): full crt0 (zero-BSS + .data copy). Wire this
//! in only when the kernel needs it — i.e. when it relies on zeroed statics or
//! an initialized .data section. Until then the boot stub + explicit zeroing
//! suffice.
//!
//! Pure-core `halt()` is unconditional here (valid x86_64 asm on any target;
//! only ever *called* in freestanding context) so the shared crate stays
//! no_std-clean and never pulls in the hosted std::process::exit variant.

use core::arch::asm;

/// Halt the CPU permanently: disable interrupts, then loop on `hlt`.
pub fn halt() -> ! {
    loop {
        unsafe { asm!("cli", "hlt", options(nomem, nostack)) };
    }
}

/// Halt but leave interrupts enabled (wake on IRQ, then `hlt` again).
pub fn halt_interruptible() -> ! {
    loop {
        unsafe { asm!("hlt", options(nomem, nostack)) };
    }
}
