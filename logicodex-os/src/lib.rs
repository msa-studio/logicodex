#![no_std]
//! Shared no_std freestanding runtime extracted from the Logicodex compiler's
//! `src/os/`. Used by the bare-metal kernel and (later) the compiler's
//! freestanding emission. Pure-core: no std, no inkwell.
pub mod uart;
pub mod interrupts;
