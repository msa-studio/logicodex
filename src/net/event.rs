// =========================================================================
// Logicodex v1.33.0-alpha — Network Reactor: Event Types
//
// Event yang dihasilkan oleh Reactor apabila epoll_wait() mengesan
// perubahan pada file descriptor. Setiap event diproses secara
// deterministik — tiada race condition.
// =========================================================================

/// Jenis-jenis event rangkaian yang ditanganani oleh Reactor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventKind {
    /// Data tersedia untuk dibaca dari fd
    Readable,
    /// fd sedia untuk menulis
    Writable,
    /// Error berlaku pada fd
    Error,
    /// Koneksi di-tutup oleh pihak lawan (peer hangup)
    Hangup,
    /// Koneksi tamat (timeout atau ditutup secara paksa)
    Closed,
}

/// Satu event daripada Reactor — berkaitan satu fd dan jenis event.
#[derive(Debug, Clone)]
pub struct Event {
    /// File descriptor yang terlibat
    pub fd: i32,
    /// Jenis event
    pub kind: EventKind,
    /// Data tambahan (bytes available untuk Read, error code untuk Error)
    pub data: i64,
}

impl Event {
    /// Cipta event baru.
    pub fn new(fd: i32, kind: EventKind, data: i64) -> Self {
        Self { fd, kind, data }
    }

    /// Adakah event ini menandakan koneksi tamat?
    pub fn is_terminal(&self) -> bool {
        matches!(self.kind, EventKind::Hangup | EventKind::Closed | EventKind::Error)
    }

    /// Adakah event ini boleh membaca data?
    pub fn is_readable(&self) -> bool {
        self.kind == EventKind::Readable
    }

    /// Adakah event ini boleh menulis data?
    pub fn is_writable(&self) -> bool {
        self.kind == EventKind::Writable
    }
}

/// Keputusan tindakan selepas memproses satu event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// Teruskan memantau fd ini
    Keep,
    /// Tutup fd dan buang dari Reactor (RAII cleanup)
    Close,
    /// Tukar interest — hanya pantau Read atau Write sahaja
    Reregister { read: bool, write: bool },
}
