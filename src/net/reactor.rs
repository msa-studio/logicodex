// =========================================================================
// Logicodex v1.33.0-alpha — Network Reactor: Single-Threaded Event Loop
//
// Reactor asas menggunakan epoll (Linux) atau abstraction layer.
// Event-driven, synchronous blocking — tiada async/await, tiada runtime berat.
//
// Fasa 1: Single-threaded event loop.
// Fasa 3 (akan datang): Sharded multi-core.
//
// Pola:
//   Reactor::new() → register(fd, interest) → run() → process events
// =========================================================================

use super::connection::{Connection, ConnectionStats, TaintState};
use super::event::{Action, Event, EventKind};
use super::policy::{BackpressureDecision, PolicyConfig};
use super::service::{ServiceRegistry, ServiceRegistryStats};
use std::collections::HashMap;

/// Interest — jenis event yang dipantau untuk satu fd.
#[derive(Debug, Clone, Copy)]
pub struct Interest {
    pub read: bool,
    pub write: bool,
}

impl Interest {
    /// Pantau baca sahaja.
    pub fn read() -> Self { Self { read: true, write: false } }
    /// Pantau tulis sahaja.
    pub fn write() -> Self { Self { read: false, write: true } }
    /// Pantau baca + tulis.
    pub fn read_write() -> Self { Self { read: true, write: true } }
}

/// The Reactor — core event loop.
/// Single-threaded, deterministic, zero-copy.
pub struct Reactor {
    /// Semua koneksi: fd → Connection
    connections: HashMap<i32, Connection>,
    /// Service registry
    registry: ServiceRegistry,
    /// Statistik global
    stats: ConnectionStats,
    /// Reactor sedang berjalan?
    running: bool,
    /// fd epoll (dalam produksi: epoll_create1(0))
    epoll_fd: i32,
    /// Bilangan event maksimum per epoll_wait
    max_events: usize,
}

impl Reactor {
    /// Cipta Reactor baru.
    pub fn new() -> Self {
        Self {
            connections: HashMap::new(),
            registry: ServiceRegistry::new(),
            stats: ConnectionStats::default(),
            running: false,
            epoll_fd: -1, // placeholder — dalam produksi: epoll_create1(0)
            max_events: 1024,
        }
    }

    /// Cipta Reactor dengan kapasiti tertentu.
    pub fn with_capacity(max_events: usize) -> Self {
        let mut r = Self::new();
        r.max_events = max_events;
        r
    }

    /// Daftarkan satu Connection ke dalam Reactor.
    /// Mengembalikan fd connection.
    pub fn register(&mut self, mut conn: Connection) -> Result<i32, ReactorError> {
        let fd = conn.fd();

        // Check duplicate fd
        if self.connections.contains_key(&fd) {
            return Err(ReactorError::DuplicateFd(fd));
        }

        // Check max connections
        if self.connections.len() >= conn.config.max_connections {
            conn.shutdown();
            return Err(ReactorError::MaxConnectionsReached);
        }

        // Dalam produksi: epoll_ctl(self.epoll_fd, EPOLL_CTL_ADD, fd, ...)
        self.stats.total_accepted += 1;
        self.connections.insert(fd, conn);
        Ok(fd)
    }

    /// Buang Connection dari Reactor (RAII: Connection akan di-drop).
    pub fn unregister(&mut self, fd: i32) {
        if let Some(mut conn) = self.connections.remove(&fd) {
            conn.shutdown(); // RAII drop akan close(fd)
            self.stats.total_closed += 1;
        }
    }

    /// Tukar interest untuk satu fd.
    pub fn reregister(&mut self, fd: i32, interest: Interest) -> Result<(), ReactorError> {
        if !self.connections.contains_key(&fd) {
            return Err(ReactorError::UnknownFd(fd));
        }
        // Dalam produksi: epoll_ctl(self.epoll_fd, EPOLL_CTL_MOD, fd, ...)
        Ok(())
    }

    /// Proses satu event.
    pub fn process_event(&mut self, event: &Event) {
        let fd = event.fd;

        // Dapatkan connection
        let Some(conn) = self.connections.get_mut(&fd) else {
            // Event untuk fd yang tak dikenali — abaikan
            return;
        };

        // Proses taint state machine
        let action = conn.handle_event(event);

        match action {
            Action::Keep => {
                // Update aktiviti
                conn.last_activity_ms = 0; // placeholder
            }
            Action::Close => {
                self.unregister(fd);
            }
            Action::Reregister { read, write } => {
                // Tukar interest
                let _ = self.reregister(fd, Interest { read, write });
            }
        }
    }

    /// Jalankan satu iterasi event loop.
    /// Dalam produksi: epoll_wait() → process events.
    /// Untuk v1.33.0-alpha: stub yang memproses events daripada vec.
    pub fn run_once(&mut self, events: &[Event]) -> usize {
        let mut processed = 0;
        for event in events {
            self.process_event(event);
            processed += 1;
        }

        // Check timeout untuk semua koneksi
        self.check_all_timeouts();

        // Check taint untuk semua koneksi
        self.check_all_taints();

        processed
    }

    /// Jalankan event loop (infinite — sehingga stop() dipanggil).
    /// Untuk v1.33.0-alpha: stub.
    pub fn run(&mut self) {
        self.running = true;
        eprintln!("logicodex v1.33.0-alpha: Reactor started (single-threaded)");

        while self.running {
            // Dalam produksi:
            //   let n = epoll_wait(self.epoll_fd, events.as_mut_ptr(), max_events, timeout);
            //   for i in 0..n { process_event(events[i]); }

            // Untuk now: sleep briefly
            self.running = false; // stub — satu iterasi sahaja
        }

        eprintln!("logicodex v1.33.0-alpha: Reactor stopped");
    }

    /// Hentikan Reactor.
    pub fn stop(&mut self) {
        self.running = false;
    }

    /// Check timeout untuk semua koneksi.
    fn check_all_timeouts(&mut self) {
        let to_close: Vec<i32> = self
            .connections
            .iter_mut()
            .filter_map(|(fd, conn)| {
                match conn.check_timeout() {
                    Action::Close => Some(*fd),
                    _ => None,
                }
            })
            .collect();

        for fd in to_close {
            self.unregister(fd);
            self.stats.total_timeout_closed += 1;
        }
    }

    /// Check taint untuk semua koneksi.
    fn check_all_taints(&mut self) {
        let to_close: Vec<i32> = self
            .connections
            .iter_mut()
            .filter_map(|(fd, conn)| {
                match conn.check_taint() {
                    Action::Close => Some(*fd),
                    _ => None,
                }
            })
            .collect();

        for fd in to_close {
            self.unregister(fd);
            self.stats.total_taint_closed += 1;
        }
    }

    /// Aplikasi backpressure untuk satu servis.
    pub fn apply_backpressure(
        &mut self,
        fd: i32,
        decision: BackpressureDecision,
    ) -> Result<(), ReactorError> {
        let Some(conn) = self.connections.get_mut(&fd) else {
            return Err(ReactorError::UnknownFd(fd));
        };

        match conn.apply_backpressure(decision) {
            Action::Close => {
                self.unregister(fd);
            }
            _ => {} // Lain: teruskan
        }

        Ok(())
    }

    /// Dapatkan Connection mengikut fd (mutable).
    pub fn connection_mut(&mut self, fd: i32) -> Option<&mut Connection> {
        self.connections.get_mut(&fd)
    }

    /// Bilangan koneksi aktif.
    pub fn connection_count(&self) -> usize {
        self.connections.len()
    }

    /// Dapatkan statistik.
    pub fn stats(&self) -> &ConnectionStats {
        &self.stats
    }

    /// Dapatkan ServiceRegistry (mutable).
    pub fn registry_mut(&mut self) -> &mut ServiceRegistry {
        &mut self.registry
    }

    /// Dapatkan ServiceRegistry (read-only).
    pub fn registry(&self) -> &ServiceRegistry {
        &self.registry
    }

    /// Tutup semua koneksi (shutdown graceful).
    pub fn shutdown_all(&mut self) {
        let fds: Vec<i32> = self.connections.keys().cloned().collect();
        for fd in fds {
            self.unregister(fd);
        }
        eprintln!("logicodex v1.33.0-alpha: All connections closed (graceful shutdown)");
    }
}

impl Drop for Reactor {
    fn drop(&mut self) {
        self.shutdown_all();
        // Dalam produksi: close(self.epoll_fd)
    }
}

/// Error Reactor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReactorError {
    DuplicateFd(i32),
    MaxConnectionsReached,
    UnknownFd(i32),
    ServiceNotFound(String),
}

impl std::fmt::Display for ReactorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReactorError::DuplicateFd(fd) => write!(f, "Fd {} sudah didaftarkan", fd),
            ReactorError::MaxConnectionsReached => write!(f, "Maksimum konegsi tercapai"),
            ReactorError::UnknownFd(fd) => write!(f, "Fd {} tidak dikenali", fd),
            ReactorError::ServiceNotFound(name) => write!(f, "Servis '{}' tidak wujud", name),
        }
    }
}

impl std::error::Error for ReactorError {}
