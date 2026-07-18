// =========================================================================
// Logicodex v1.33.0-alpha — Network Reactor: RAII Connection
//
// "Killer Feature": Setiap koneksi = objek RAII.
// Jika koneksi tamat (timeout, terputus, malicious), objek di-drop
// secara deterministik. close(fd) + padam rekod dalam Reactor.
// Tiada socket leak. Tiada GC. 100% deterministik.
//
// Taint State Machine:
//   Healthy → Suspicious → Closing
//      ↑______________|
//   Healthy = normal operation
//   Suspicious = pattern anomali detected (bisa kembali Healthy)
//   Closing = akan di-drop (deterministic cleanup)
// =========================================================================

use super::event::{Action, Event, EventKind};
use super::policy::{BackpressureDecision, PolicyConfig};

/// Keadaan kebersihan (taint) satu koneksi.
/// State machine: Healthy → Suspicious → Closing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaintState {
    /// Operasi normal — tiada isu
    Healthy,
    /// Pattern anomali dikesan — boleh kembali Healthy atau ke Closing
    Suspicious,
    /// Akan di-drop — deterministic cleanup imminent
    Closing,
}

impl TaintState {
    /// Adakah koneksi masih aktif (boleh baca/tulis)?
    pub fn is_active(&self) -> bool {
        !matches!(self, TaintState::Closing)
    }

    /// Adakah koneksi perlu dipantau dengan lebih ketat?
    pub fn requires_monitoring(&self) -> bool {
        matches!(self, TaintState::Suspicious)
    }

    /// Tukar ke Closing jika Suspicious terlalu lama.
    pub fn escalate(self) -> Self {
        match self {
            TaintState::Healthy => TaintState::Healthy,
            TaintState::Suspicious => TaintState::Closing,
            TaintState::Closing => TaintState::Closing,
        }
    }

    /// Recover dari Suspicious ke Healthy.
    pub fn recover(self) -> Self {
        match self {
            TaintState::Suspicious => TaintState::Healthy,
            other => other,
        }
    }
}

impl Default for TaintState {
    fn default() -> Self {
        TaintState::Healthy
    }
}

// ─── Connection ───
/// Satu koneksi rangkaian yang diuruskan oleh Reactor.
/// RAII: apabila Connection di-drop, fd ditutup secara automatik.
pub struct Connection {
    /// File descriptor (opaque — pengguna tak boleh access secara langsung)
    fd: i32,
    /// Port yang dipautkan
    pub port: u16,
    /// Nama servis yang mengendalikan koneksi ini
    pub service_name: String,
    /// Keadaan taint (Healthy/Suspicious/Closing)
    pub taint: TaintState,
    /// Bilangan bytes yang diterima
    pub bytes_received: u64,
    /// Bilangan bytes yang dihantar
    pub bytes_sent: u64,
    /// Bilangan kesalahan berturut-turut
    pub consecutive_errors: u32,
    /// Timestamp koneksi dibuat (ms since epoch)
    pub created_at_ms: u64,
    /// Timestamp terakhir aktiviti (ms since epoch)
    pub last_activity_ms: u64,
    /// Konfigurasi polisi
    config: PolicyConfig,
    /// Sudah ditutup? (elak double-close)
    closed: bool,
}

impl Connection {
    /// Cipta Connection baru (daripada accept()).
    pub fn new(fd: i32, port: u16, service_name: impl Into<String>, config: PolicyConfig) -> Self {
        let now = now_ms();
        Self {
            fd,
            port,
            service_name: service_name.into(),
            taint: TaintState::Healthy,
            bytes_received: 0,
            bytes_sent: 0,
            consecutive_errors: 0,
            created_at_ms: now,
            last_activity_ms: now,
            config,
            closed: false,
        }
    }

    /// Baca data dari koneksi. Mengembalikan bytes yang dibaca.
    /// v1.37: Uses SYS_RECV syscall directly (no libc).
    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize, ConnectionError> {
        if !self.taint.is_active() {
            return Err(ConnectionError::ConnectionClosed);
        }
        let n = unsafe {
            crate::os::syscall::sys_recv(self.fd, buf.as_mut_ptr(), buf.len(), 0)
        };
        if n < 0 {
            self.consecutive_errors += 1;
            if self.consecutive_errors >= 3 {
                self.taint = TaintState::Closing;
            }
            Err(ConnectionError::ReadTimeout)
        } else if n == 0 {
            // Peer closed connection
            self.taint = TaintState::Closing;
            Err(ConnectionError::ConnectionClosed)
        } else {
            let n = n as usize;
            self.bytes_received += n as u64;
            self.last_activity_ms = now_ms();
            self.consecutive_errors = 0;
            Ok(n)
        }
    }

    /// Tulis data ke koneksi.
    /// v1.37: Uses SYS_SEND syscall directly (no libc).
    pub fn write(&mut self, buf: &[u8]) -> Result<usize, ConnectionError> {
        if !self.taint.is_active() {
            return Err(ConnectionError::ConnectionClosed);
        }
        let n = unsafe {
            crate::os::syscall::sys_send(self.fd, buf.as_ptr(), buf.len(), 0)
        };
        if n < 0 {
            self.consecutive_errors += 1;
            if self.consecutive_errors >= 3 {
                self.taint = TaintState::Closing;
            }
            Err(ConnectionError::WriteTimeout)
        } else {
            let n = n as usize;
            self.bytes_sent += n as u64;
            self.last_activity_ms = now_ms();
            self.consecutive_errors = 0;
            Ok(n)
        }
    }

    /// Proses satu event daripada Reactor.
    pub fn handle_event(&mut self, event: &Event) -> Action {
        match event.kind {
            EventKind::Readable => {
                // Baca data
                Action::Keep
            }
            EventKind::Writable => {
                Action::Keep
            }
            EventKind::Error => {
                self.consecutive_errors += 1;
                if self.consecutive_errors >= 3 {
                    self.taint = TaintState::Closing;
                    Action::Close
                } else {
                    Action::Keep
                }
            }
            EventKind::Hangup | EventKind::Closed => {
                self.taint = TaintState::Closing;
                Action::Close
            }
        }
    }

    /// Check timeout — tukar ke Closing jika idle terlalu lama.
    pub fn check_timeout(&mut self) -> Action {
        let idle = now_ms() - self.last_activity_ms;
        if idle > self.config.read_timeout_ms {
            self.taint = TaintState::Closing;
            Action::Close
        } else {
            Action::Keep
        }
    }

    /// Check taint — proses state machine.
    pub fn check_taint(&mut self) -> Action {
        match self.taint {
            TaintState::Closing => Action::Close,
            TaintState::Suspicious => {
                // Suspicious terlalu lama → Closing
                let idle = now_ms() - self.last_activity_ms;
                if idle > self.config.read_timeout_ms / 2 {
                    self.taint = TaintState::Closing;
                    Action::Close
                } else {
                    Action::Keep
                }
            }
            TaintState::Healthy => Action::Keep,
        }
    }

    /// Aplikasi polisi backpressure.
    pub fn apply_backpressure(&self, decision: BackpressureDecision) -> Action {
        match decision {
            BackpressureDecision::Close => Action::Close,
            BackpressureDecision::Wait => Action::Keep,
            BackpressureDecision::Drop => Action::Keep,
            BackpressureDecision::Reject => Action::Keep,
            BackpressureDecision::Proceed => Action::Keep,
        }
    }

    /// Tutup konegsi (manual — biasanya dilakukan oleh Drop).
    /// v1.37: Uses SYS_CLOSE syscall directly.
    pub fn shutdown(&mut self) {
        if !self.closed {
            self.closed = true;
            self.taint = TaintState::Closing;
            crate::os::syscall::sys_close(self.fd);
        }
    }

    /// Dapatkan fd (read-only reference).
    pub fn fd(&self) -> i32 {
        self.fd
    }

    /// Check jika koneksi sudah ditutup.
    pub fn is_closed(&self) -> bool {
        self.closed
    }

    /// Umur konegsi dalam ms.
    pub fn age_ms(&self) -> u64 {
        now_ms() - self.created_at_ms
    }

    /// Idle time dalam ms.
    pub fn idle_ms(&self) -> u64 {
        now_ms() - self.last_activity_ms
    }
}

/// RAII Drop: Apabila Connection keluar dari scope, fd ditutup secara deterministik.
impl Drop for Connection {
    fn drop(&mut self) {
        self.shutdown();
        eprintln!(
            "logicodex: Connection fd={} closed (RAII) — service='{}', rx={}B tx={}B age={}ms",
            self.fd, self.service_name, self.bytes_received, self.bytes_sent, self.age_ms()
        );
    }
}

/// Error koneksi.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionError {
    ConnectionClosed,
    ReadTimeout,
    WriteTimeout,
    BufferFull,
    Tainted,
}

impl std::fmt::Display for ConnectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnectionError::ConnectionClosed => write!(f, "Connection closed"),
            ConnectionError::ReadTimeout => write!(f, "Read timeout"),
            ConnectionError::WriteTimeout => write!(f, "Write timeout"),
            ConnectionError::BufferFull => write!(f, "Buffer full"),
            ConnectionError::Tainted => write!(f, "Connection tainted"),
        }
    }
}

impl std::error::Error for ConnectionError {}

// ─── Helper ───
/// Current timestamp dalam ms (monotonic — v1.37: guna clock_gettime).
fn now_ms() -> u64 {
    crate::os::syscall::clock_gettime_monotonic_ms()
}

/// Statistik global koneksi (untuk monitoring / .cap audit).
#[derive(Debug, Default)]
pub struct ConnectionStats {
    /// Total koneksi diterima
    pub total_accepted: u64,
    /// Total konegsi ditutup (RAII)
    pub total_closed: u64,
    /// Total konegsi ditamatkan kerana taint
    pub total_taint_closed: u64,
    /// Total konegsi ditamatkan kerana timeout
    pub total_timeout_closed: u64,
    /// Total bytes diterima
    pub total_bytes_received: u64,
    /// Total bytes dihantar
    pub total_bytes_sent: u64,
}
