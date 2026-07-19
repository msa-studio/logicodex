// =========================================================================
// Syscall backend
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

    // Network runtime syscalls
    pub const SYS_RECV: u64 = 17; // recvfrom on some arch, but recv=17 on x86_64
    pub const SYS_SEND: u64 = 16; // sendto on some arch, but send=16 on x86_64
    pub const SYS_SOCKET: u64 = 41;
    pub const SYS_ACCEPT: u64 = 43;
    pub const SYS_BIND: u64 = 49;
    pub const SYS_LISTEN: u64 = 50;
    pub const SYS_EPOLL_CREATE1: u64 = 291;
    pub const SYS_EPOLL_CTL: u64 = 233;
    pub const SYS_EPOLL_WAIT: u64 = 232;
    pub const SYS_CLOCK_GETTIME: u64 = 228;
    pub const SYS_SCHED_SETAFFINITY: u64 = 203;
    pub const SYS_SCHED_GETCPU: u64 = 309;
    pub const CLOCK_MONOTONIC: u64 = 1;

    // CPU set size for sched_setaffinity (512 bytes = 4096 CPUs)
    pub const CPU_SETSIZE: usize = 512;

    // epoll_ctl op codes
    pub const EPOLL_CTL_ADD: i32 = 1;
    pub const EPOLL_CTL_MOD: i32 = 2;
    pub const EPOLL_CTL_DEL: i32 = 3;

    // epoll event flags
    pub const EPOLLIN: u32 = 0x001;
    pub const EPOLLOUT: u32 = 0x004;
    pub const EPOLLERR: u32 = 0x008;
    pub const EPOLLHUP: u32 = 0x010;
    pub const EPOLLET: u32 = 1 << 31; // edge-triggered
    pub const EPOLL_CLOEXEC: i32 = 0x80000;

    /// Emit raw syscall via `syscall` instruction.
    /// Arguments: rax = syscall number, rdi, rsi, rdx, r10, r8, r9 = args
    #[inline(always)]
    pub unsafe fn syscall0(n: u64) -> i64 {
        let ret: i64;
        core::arch::asm!(
            "syscall",
            inlateout("rax") n => ret,
            out("rcx") _, out("r11") _,
            options(nostack, preserves_flags)
        );
        ret
    }

    #[inline(always)]
    pub unsafe fn syscall2(n: u64, a1: u64, a2: u64) -> i64 {
        let ret: i64;
        core::arch::asm!(
            "syscall",
            inlateout("rax") n => ret,
            in("rdi") a1, in("rsi") a2,
            out("rcx") _, out("r11") _,
            options(nostack, preserves_flags)
        );
        ret
    }

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

    #[inline(always)]
    pub unsafe fn syscall4(n: u64, a1: u64, a2: u64, a3: u64, a4: u64) -> i64 {
        let ret: i64;
        core::arch::asm!(
            "syscall",
            inlateout("rax") n => ret,
            in("rdi") a1, in("rsi") a2, in("rdx") a3, in("r10") a4,
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
    /// Graceful fallback — returns error instead of panicking.
    pub fn open_file(path: &str, access: u32) -> Result<isize, i32> {
        eprintln!("logicodex: Windows open_file({path}, {access}) — Windows syscalls require hosted CallableRegistry FFI");
        Err(-1)
    }

    /// Windows fallback for sys_recv — not applicable on Windows.
    pub fn win_recv_fallback(_fd: i32, _buf: &mut [u8]) -> Result<usize, i32> {
        eprintln!("logicodex: Windows recv not implemented — use WSARecv via CallableRegistry");
        Err(-1)
    }

    /// Windows fallback for sys_send — not applicable on Windows.
    pub fn win_send_fallback(_fd: i32, _buf: &[u8]) -> Result<usize, i32> {
        eprintln!("logicodex: Windows send not implemented — use WSASend via CallableRegistry");
        Err(-1)
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

// =========================================================================
// Direct Syscall Wrappers
// =========================================================================

/// epoll_create1 — create an epoll instance.
/// Returns epoll fd on success, -1 on error.
pub fn epoll_create1(flags: i32) -> i32 {
    #[cfg(target_os = "linux")]
    unsafe {
        linux::syscall1(linux::SYS_EPOLL_CREATE1, flags as u64) as i32
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = flags;
        -1
    }
}

/// epoll_ctl — control interface for an epoll instance.
pub fn epoll_ctl(epoll_fd: i32, op: i32, fd: i32, events: u32) -> i32 {
    #[cfg(target_os = "linux")]
    unsafe {
        // epoll_event struct: { u32 events, u64 data (fd) }
        let mut event = [events, fd as u32];
        linux::syscall4(
            linux::SYS_EPOLL_CTL,
            epoll_fd as u64,
            op as u64,
            fd as u64,
            event.as_mut_ptr() as u64,
        ) as i32
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (epoll_fd, op, fd, events);
        -1
    }
}

/// epoll_wait — wait for an I/O event on an epoll file descriptor.
/// Returns number of file descriptors ready, -1 on error.
pub fn epoll_wait(epoll_fd: i32, events: *mut u8, maxevents: i32, timeout_ms: i32) -> i32 {
    #[cfg(target_os = "linux")]
    unsafe {
        linux::syscall4(
            linux::SYS_EPOLL_WAIT,
            epoll_fd as u64,
            events as u64,
            maxevents as u64,
            timeout_ms as u64,
        ) as i32
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (epoll_fd, events, maxevents, timeout_ms);
        -1
    }
}

/// SYS_RECV — receive data from a socket.
pub fn sys_recv(fd: i32, buf: *mut u8, count: usize, flags: i32) -> isize {
    #[cfg(target_os = "linux")]
    unsafe {
        linux::syscall4(
            linux::SYS_RECV,
            fd as u64,
            buf as u64,
            count as u64,
            flags as u64,
        ) as isize
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (fd, buf, count, flags);
        -1
    }
}

/// SYS_SEND — send data to a socket.
pub fn sys_send(fd: i32, buf: *const u8, count: usize, flags: i32) -> isize {
    #[cfg(target_os = "linux")]
    unsafe {
        linux::syscall4(
            linux::SYS_SEND,
            fd as u64,
            buf as u64,
            count as u64,
            flags as u64,
        ) as isize
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (fd, buf, count, flags);
        -1
    }
}

/// clock_gettime — get monotonic timestamp.
/// Returns nanoseconds since an unspecified epoch (monotonic).
pub fn clock_gettime_monotonic_ns() -> u64 {
    #[cfg(target_os = "linux")]
    unsafe {
        let mut ts = [0u64; 2]; // timespec: tv_sec, tv_nsec
        let ret = linux::syscall2(
            linux::SYS_CLOCK_GETTIME,
            linux::CLOCK_MONOTONIC,
            ts.as_mut_ptr() as u64,
        );
        if ret < 0 {
            0
        } else {
            ts[0] * 1_000_000_000 + ts[1] // sec * 1e9 + nsec
        }
    }
    #[cfg(not(target_os = "linux"))]
    {
        // Fallback to Rust std
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0)
    }
}

/// clock_gettime — return milliseconds (monotonic).
pub fn clock_gettime_monotonic_ms() -> u64 {
    clock_gettime_monotonic_ns() / 1_000_000
}

/// SYS_CLOSE — close a file descriptor.
pub fn sys_close(fd: i32) -> i32 {
    #[cfg(target_os = "linux")]
    unsafe {
        linux::syscall1(linux::SYS_CLOSE, fd as u64) as i32
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = fd;
        -1
    }
}
