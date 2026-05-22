// =========================================================================
// Project: Logicodex Language Engine (Phase 1 MVP Upgrade)
// Version: v1.0.1-alpha (Internal Security & OS Freestanding Test)
// Architect & Creator: Mohamad Supardi Abdul (mymsastudio@gmail.com)
// Copyright (c) 2026. All Rights Reserved.
// Licensed under permissive dual-license: MIT & Apache License 2.0
// =========================================================================
pub mod target;

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
