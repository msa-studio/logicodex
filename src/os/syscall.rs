// =========================================================================
// Logicodex v1.30 — Ketuk 4: Syscall Backend
//
// Direct system call emission for Linux x86_64 and Windows.
// No libc — bare-metal syscall instruction.
// =========================================================================

/// Linux x86_64 syscall numbers
#[cfg(target_os = "linux")]
pub mod linux {
    pub const SYS_READ: u64 = 0;
    pub const SYS_WRITE: u64 = 1;
    pub const SYS_OPEN: u64 = 2;
    pub const SYS_CLOSE: u64 = 3;
    pub const SYS_LSEEK: u64 = 8;
    pub const SYS_MMAP: u64 = 9;
    pub const SYS_MUNMAP: u64 = 11;
    pub const SYS_EXIT: u64 = 60;
    pub const SYS_FSTAT: u64 = 5;

    /// Emit raw syscall via `syscall` instruction.
    /// Arguments: rax = syscall number, rdi, rsi, rdx, r10, r8, r9 = args
    #[inline(always)]
    pub unsafe fn syscall1(n: u64, a1: u64) -> i64 {
        let ret: i64;
        core::arch::asm!(
            "syscall",
            inlateout("rax") n => ret,
            in("rdi") a1,
            out("rcx") _, out("r11") _,
            options(nostack, preserves_flags)
        );
        ret
    }

    #[inline(always)]
    pub unsafe fn syscall3(n: u64, a1: u64, a2: u64, a3: u64) -> i64 {
        let ret: i64;
        core::arch::asm!(
            "syscall",
            inlateout("rax") n => ret,
            in("rdi") a1, in("rsi") a2, in("rdx") a3,
            out("rcx") _, out("r11") _,
            options(nostack, preserves_flags)
        );
        ret
    }
}

/// Windows API wrapper (hosted mode)
#[cfg(target_os = "windows")]
pub mod windows {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    /// CreateFileW → ReadFile/WriteFile → CloseHandle
    pub fn open_file(path: &str, access: u32) -> Result<isize, i32> {
        // Handled by CallableRegistry FFI
        // kernel32!CreateFileW(path, access, ...)
        unimplemented!("Windows file syscall — deferred to hosted CallableRegistry")
    }
}

/// Emit syscall for file operations (Ketuk 4 codegen target)
pub fn emit_file_syscall(op: FileOp, fd: i32, buf: *mut u8, count: usize) -> i64 {
    #[cfg(target_os = "linux")]
    unsafe {
        match op {
            FileOp::Read => linux::syscall3(linux::SYS_READ, fd as u64, buf as u64, count as u64),
            FileOp::Write => linux::syscall3(linux::SYS_WRITE, fd as u64, buf as u64, count as u64),
            FileOp::Close => linux::syscall1(linux::SYS_CLOSE, fd as u64),
            FileOp::Seek => linux::syscall3(linux::SYS_LSEEK, fd as u64, buf as u64, count as u64),
        }
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (op, fd, buf, count);
        -1 // Unsupported platform
    }
}

#[derive(Debug, Clone, Copy)]
pub enum FileOp {
    Read,
    Write,
    Close,
    Seek,
}
