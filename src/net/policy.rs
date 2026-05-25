// =========================================================================
// Logicodex v1.33.0-alpha — Network Reactor: Backpressure Policies
//
// Polisi backpressure yang menentukan tindakan apabila buffer
// servis penuh. Ini adalah Fasa 2 capability yang dibina dari
// awal untuk integrasi mudah.
//
// Polisi:
//   Block     — Blokir penerimaan data sehingga buffer kosong
//   DropOldest — Buang data paling lama, teruskan
//   Error     — Kembalikan error kepada pengirim
// =========================================================================

use super::connection::TaintState;

/// Polisi backpressure untuk satu servis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackpressurePolicy {
    /// Blokir — tunggu sehingga buffer ada ruang
    /// Mengelakkan kehilangan data tetapi menambah latency
    Block,
    /// Buang data paling lama — teruskan operasi
    /// Sesuai untuk real-time streaming (sensor data)
    DropOldest,
    /// Kembalikan error — pengirim mesti cuba semula
    /// Sesuai untuk protokol yang support retry
    Error,
}

impl Default for BackpressurePolicy {
    fn default() -> Self {
        BackpressurePolicy::Block // paling selamat
    }
}

impl BackpressurePolicy {
    /// Parse daripada string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Block" | "block" | "Blok" => Some(BackpressurePolicy::Block),
            "DropOldest" | "drop_oldest" | "BuangLama" => Some(BackpressurePolicy::DropOldest),
            "Error" | "error" | "Ralat" => Some(BackpressurePolicy::Error),
            _ => None,
        }
    }

    /// Format sebagai string.
    pub fn as_str(&self) -> &'static str {
        match self {
            BackpressurePolicy::Block => "Block",
            BackpressurePolicy::DropOldest => "DropOldest",
            BackpressurePolicy::Error => "Error",
        }
    }

    /// Apakah tindakan apabila buffer penuh?
    /// Mengembalikan keputusan BackpressureDecision.
    pub fn apply(&self, taint: TaintState) -> BackpressureDecision {
        match self {
            BackpressurePolicy::Block => BackpressureDecision::Wait,
            BackpressurePolicy::DropOldest => BackpressureDecision::Drop,
            BackpressurePolicy::Error => {
                // Jika koneksi sudah Suspicious, tutup terus
                if taint == TaintState::Suspicious {
                    BackpressureDecision::Close
                } else {
                    BackpressureDecision::Reject
                }
            }
        }
    }
}

/// Keputusan daripada aplikasi polisi backpressure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackpressureDecision {
    /// Tunggu sehingga buffer kosong
    Wait,
    /// Buang data paling lama
    Drop,
    /// Tolak dengan error
    Reject,
    /// Tutup koneksi terus
    Close,
    /// Teruskan — buffer masih ada ruang
    Proceed,
}

/// Konfigurasi polisi untuk satu servis.
#[derive(Debug, Clone)]
pub struct PolicyConfig {
    /// Polisi backpressure
    pub backpressure: BackpressurePolicy,
    /// Saiz buffer maksimum (bytes)
    pub max_buffer_size: usize,
    /// Timeout baca (ms)
    pub read_timeout_ms: u64,
    /// Timeout tulis (ms)
    pub write_timeout_ms: u64,
    /// Maksimum koneksi serentak
    pub max_connections: usize,
}

impl Default for PolicyConfig {
    fn default() -> Self {
        Self {
            backpressure: BackpressurePolicy::Block,
            max_buffer_size: 64 * 1024,       // 64KB
            read_timeout_ms: 30_000,            // 30s
            write_timeout_ms: 30_000,           // 30s
            max_connections: 1024,
        }
    }
}

impl PolicyConfig {
    /// Konfigurasi untuk servis berprestasi tinggi (game server, etc.)
    pub fn high_throughput() -> Self {
        Self {
            backpressure: BackpressurePolicy::DropOldest,
            max_buffer_size: 256 * 1024,        // 256KB
            read_timeout_ms: 5_000,             // 5s
            write_timeout_ms: 5_000,
            max_connections: 4096,
        }
    }

    /// Konfigurasi untuk servis kritikal (banking, control systems)
    pub fn critical() -> Self {
        Self {
            backpressure: BackpressurePolicy::Error,
            max_buffer_size: 16 * 1024,         // 16KB
            read_timeout_ms: 10_000,            // 10s
            write_timeout_ms: 10_000,
            max_connections: 256,
        }
    }
}
