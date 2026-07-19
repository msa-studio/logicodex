// =========================================================================
// Project: Logicodex Language Engine (Phase 2 Deployment Integration)
// Freestanding compiler support
// Architect & Creator: Mohamad Supardi Abdul (mymsastudio@gmail.com)
// Copyright (c) 2026. All Rights Reserved.
// Licensed under permissive dual-license: MIT & Apache License 2.0
// =========================================================================
pub mod syscall;
pub mod target;

// Freestanding support modules
// These modules are freestanding-only (bare metal / no OS).
// They are gated with #[cfg(target_os = "none")] so CI
// does not attempt to compile x86_64 port-I/O code on Linux.
#[cfg(target_os = "none")]
pub mod allocator;
#[cfg(target_os = "none")]
pub mod startup;
#[cfg(target_os = "none")]
pub mod uart;
// panic.rs has its own internal gating for both hosted and freestanding.
pub mod panic;

// Future freestanding modules (not yet wired for hosted compilation)
// pub mod interrupts;      // x86_64 IDT — needs #[cfg] gating
// pub mod source_provider; // needs #[cfg] gating for hosted vs freestanding

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "linux")]
pub use linux::runtime_assembly;

#[cfg(target_os = "windows")]
pub use windows::runtime_assembly;

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
pub fn runtime_assembly() -> &'static str {
    ".global logicodex_print_i64\nlogicodex_print_i64:\n    ret\n"
}
