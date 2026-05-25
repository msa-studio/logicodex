// =========================================================================
// Logicodex v1.39.0-alpha — Network Reactor: CPU Affinity
//
// Platform abstraction untuk CPU affinity.
// Linux: sched_setaffinity via direct syscall
// macOS: thread_policy_set (THREAD_AFFINITY_POLICY)
// Windows: SetThreadAffinityMask
//
// Prinsip: Setiap shard dipin ke satu CPU core — tidak berpindah.
// =========================================================================

use crate::os::syscall;

/// Error affinity.
#[derive(Debug, Clone)]
pub enum AffinityError {
    UnsupportedPlatform,
    InvalidCore(u32),
    SystemError(String),
}

impl std::fmt::Display for AffinityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AffinityError::UnsupportedPlatform => {
                write!(f, "CPU affinity tidak disokong pada platform ini")
            }
            AffinityError::InvalidCore(id) => {
                write!(f, "Core ID {} tidak sah (max: {})", id, num_cpus())
            }
            AffinityError::SystemError(msg) => write!(f, "Ralat sistem: {}", msg),
        }
    }
}

impl std::error::Error for AffinityError {}

// ─── C3: Linux — sched_setaffinity via syscall ───

#[cfg(target_os = "linux")]
pub fn set_cpu_affinity(core_id: u32) -> Result<(), AffinityError> {
    if core_id >= num_cpus() as u32 {
        return Err(AffinityError::InvalidCore(core_id));
    }

    // Build cpu_set_t: 512 bytes bitmap, set bit core_id
    let mut cpuset = [0u8; syscall::linux::CPU_SETSIZE];
    let byte_idx = (core_id / 8) as usize;
    let bit_idx = core_id % 8;
    if byte_idx < cpuset.len() {
        cpuset[byte_idx] |= 1 << bit_idx;
    }

    // sched_setaffinity(pid=0, cpusetsize, &cpuset)
    let ret = unsafe {
        syscall::linux::syscall3(
            syscall::linux::SYS_SCHED_SETAFFINITY,
            0, // current thread
            syscall::linux::CPU_SETSIZE as u64,
            cpuset.as_ptr() as u64,
        )
    };

    if ret != 0 {
        return Err(AffinityError::SystemError(format!(
            "sched_setaffinity(core={}) returned {}", core_id, ret
        )));
    }

    Ok(())
}

// ─── C4: macOS — thread_policy_set ───

#[cfg(target_os = "macos")]
pub fn set_cpu_affinity(core_id: u32) -> Result<(), AffinityError> {
    if core_id >= num_cpus() as u32 {
        return Err(AffinityError::InvalidCore(core_id));
    }

    // macOS tidak ada sched_setaffinity. Gunakan thread_policy_set
    // dengan THREAD_AFFINITY_POLICY jika tersedia (10.5+).
    // Fallback: teruskan tanpa affinity — log warning.
    eprintln!(
        "logicodex v1.39: macOS CPU affinity untuk core {} — "
        "thread_policy_set tidak digunakan dalam sandbox (memerlukan framework Mach)",
        core_id
    );
    Err(AffinityError::UnsupportedPlatform)
}

// ─── C5: Windows — SetThreadAffinityMask ───

#[cfg(target_os = "windows")]
pub fn set_cpu_affinity(core_id: u32) -> Result<(), AffinityError> {
    if core_id >= num_cpus() as u32 {
        return Err(AffinityError::InvalidCore(core_id));
    }

    // Windows: SetThreadAffinityMask(GetCurrentThread(), 1 << core_id)
    // Dalam sandbox: log diagnostic — memerlukan kernel32 linking.
    eprintln!(
        "logicodex v1.39: Windows CPU affinity untuk core {} — "
        "SetThreadAffinityMask memerlukan kernel32 (gunakan CallableRegistry FFI)",
        core_id
    );
    Err(AffinityError::UnsupportedPlatform)
}

// ─── Fallback untuk platform lain ───

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
pub fn set_cpu_affinity(core_id: u32) -> Result<(), AffinityError> {
    if core_id >= num_cpus() as u32 {
        return Err(AffinityError::InvalidCore(core_id));
    }
    Err(AffinityError::UnsupportedPlatform)
}

// ─── num_cpus — gunakan std::thread::available_parallelism ───

/// Dapatkan bilangan CPU cores yang tersedia.
pub fn num_cpus() -> usize {
    std::thread::available_parallelism()
        .map(|p| p.get())
        .unwrap_or(4) // fallback: 4 cores
}

// ─── current_core_id — Linux: sched_getcpu ───

/// Dapatkan core ID semasa (thread yang sedang berjalan).
#[cfg(target_os = "linux")]
pub fn current_core_id() -> u32 {
    let cpu = unsafe {
        syscall::linux::syscall0(syscall::linux::SYS_SCHED_GETCPU)
    };
    if cpu < 0 { 0 } else { cpu as u32 }
}

#[cfg(not(target_os = "linux"))]
pub fn current_core_id() -> u32 {
    0 // fallback
}

/// Verifikasi core_id adalah sah untuk platform ini.
pub fn is_valid_core(core_id: u32) -> bool {
    (core_id as usize) < num_cpus()
}

/// Get affinity info string untuk diagnostic.
pub fn affinity_info() -> String {
    format!(
        "CPU cores: {} | current core: {} | platform: {}-{}",
        num_cpus(),
        current_core_id(),
        std::env::consts::OS,
        std::env::consts::ARCH,
    )
}
