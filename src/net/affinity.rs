// =========================================================================
// Logicodex v1.34.0-alpha — Network Reactor: CPU Affinity
//
// Platform abstraction untuk CPU affinity.
// Teras: Linux (sched_setaffinity), stub untuk macOS/Windows.
//
// Prinsip: Setiap shard dipin ke satu CPU core — tidak berpindah.
// Ini mengelakkan context switch overhead dan cache thrashing.
// =========================================================================

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
                write!(f, "CPU affinity tidak disokong pada platform ini (Linux sahaja)")
            }
            AffinityError::InvalidCore(id) => {
                write!(f, "Core ID {} tidak sah", id)
            }
            AffinityError::SystemError(msg) => {
                write!(f, "Ralat sistem: {}", msg)
            }
        }
    }
}

impl std::error::Error for AffinityError {}

/// Pin thread semasa ke CPU core tertentu.
/// Linux: guna sched_setaffinity
/// macOS/Windows: stub (log warning)
pub fn set_cpu_affinity(core_id: u32) -> Result<(), AffinityError> {
    if core_id >= num_cpus() as u32 {
        return Err(AffinityError::InvalidCore(core_id));
    }

    #[cfg(target_os = "linux")]
    {
        // Dalam produksi:
        //   use libc::{cpu_set_t, CPU_SET, sched_setaffinity, CPU_ZERO, size_t};
        //   unsafe {
        //       let mut cpuset: cpu_set_t = std::mem::zeroed();
        //       CPU_ZERO(&mut cpuset);
        //       CPU_SET(core_id as usize, &mut cpuset);
        //       let result = sched_setaffinity(0, size_of::<cpu_set_t>(), &cpuset);
        //       if result != 0 {
        //           return Err(AffinityError::SystemError(format!("sched_setaffinity returned {}", result)));
        //       }
        //   }
        // Stub untuk v1.34.0-alpha (sandbox — tiada libc):
        eprintln!("logicodex v1.34.0-alpha: Pin thread to core {} (Linux — stub)", core_id);
        Ok(())
    }

    #[cfg(target_os = "macos")]
    {
        eprintln!("logicodex v1.34.0-alpha: CPU affinity on macOS is a stub — using default scheduler");
        Err(AffinityError::UnsupportedPlatform)
    }

    #[cfg(target_os = "windows")]
    {
        eprintln!("logicodex v1.34.0-alpha: CPU affinity on Windows is a stub — using default scheduler");
        Err(AffinityError::UnsupportedPlatform)
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        Err(AffinityError::UnsupportedPlatform)
    }
}

/// Dapatkan bilangan CPU cores yang tersedia.
pub fn num_cpus() -> usize {
    // Dalam produksi: num_cpus::get() atau std::thread::available_parallelism()
    // Stub: anggar 4 cores (paling umum)
    4
}

/// Dapatkan core ID semasa (thread yang sedang berjalan).
pub fn current_core_id() -> u32 {
    // Dalam produksi: sched_getcpu() pada Linux
    // Stub: 0
    0
}

/// Verifikasi core_id adalah sah untuk platform ini.
pub fn is_valid_core(core_id: u32) -> bool {
    (core_id as usize) < num_cpus()
}
