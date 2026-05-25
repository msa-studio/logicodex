// =========================================================================
// Logicodex v1.37.0-alpha — Network Reactor: Deterministic Event Loop
//
// Reactor LIVE dengan epoll (Linux) — direct syscall, tiada libc.
// Event-driven, synchronous blocking — tiada async/await, tiada runtime berat.
//
// v1.37: Semua stubs diganti dengan implementasi sebenar:
//   - epoll_create1, epoll_ctl, epoll_wait via direct syscall
//   - SYS_RECV / SYS_SEND untuk socket I/O
//   - clock_gettime(CLOCK_MONOTONIC) untuk timestamp
//   - Event loop berterusan sehingga stop() dipanggil
//   - EPOLLIN/EPOLLOUT/EPOLLERR/EPOLLHUP dispatch
//
// Pola:
//   Reactor::new() → register(fd, interest) → run() → epoll_wait → process events
// =========================================================================

use super::connection::{Connection, ConnectionStats, TaintState};
use super::event::{Action, Event, EventKind};
use super::policy::{BackpressureDecision, PolicyConfig};
use super::service::{ServiceRegistry, ServiceRegistryStats};
use crate::os::syscall;
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
    /// v1.37: Creates real epoll instance via epoll_create1 syscall.
    pub fn new() -> Self {
        let epoll_fd = syscall::epoll_create1(syscall::linux::EPOLL_CLOEXEC);
        if epoll_fd < 0 {
            eprintln!("logicodex v1.37: WARNING epoll_create1 failed (fd={}), reactor will use stub mode", epoll_fd);
        }
        Self {
            connections: HashMap::new(),
            registry: ServiceRegistry::new(),
            stats: ConnectionStats::default(),
            running: false,
            epoll_fd,
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
    /// v1.37: Registers fd with epoll via EPOLL_CTL_ADD.
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

        // v1.37: epoll_ctl(EPOLL_CTL_ADD) — register fd with epoll
        if self.epoll_fd >= 0 {
            let mut events = syscall::linux::EPOLLIN;
            if conn.taint.is_active() {
                let ret = syscall::epoll_ctl(
                    self.epoll_fd,
                    syscall::linux::EPOLL_CTL_ADD,
                    fd,
                    events,
                );
                if ret < 0 {
                    conn.shutdown();
                    return Err(ReactorError::EpollError(format!(
                        "EPOLL_CTL_ADD fd={} failed: {}", fd, ret
                    )));
                }
            }
        }

        self.stats.total_accepted += 1;
        self.connections.insert(fd, conn);
        Ok(fd)
    }

    /// Buang Connection dari Reactor (RAII: Connection akan di-drop).
    /// v1.37: Removes fd from epoll via EPOLL_CTL_DEL.
    pub fn unregister(&mut self, fd: i32) {
        // v1.37: epoll_ctl(EPOLL_CTL_DEL) — remove fd from epoll
        if self.epoll_fd >= 0 {
            syscall::epoll_ctl(self.epoll_fd, syscall::linux::EPOLL_CTL_DEL, fd, 0);
        }
        if let Some(mut conn) = self.connections.remove(&fd) {
            conn.shutdown(); // RAII drop akan close(fd) via sys_close
            self.stats.total_closed += 1;
        }
    }

    /// Tukar interest untuk satu fd.
    /// v1.37: Uses epoll_ctl(EPOLL_CTL_MOD).
    pub fn reregister(&mut self, fd: i32, interest: Interest) -> Result<(), ReactorError> {
        if !self.connections.contains_key(&fd) {
            return Err(ReactorError::UnknownFd(fd));
        }
        if self.epoll_fd >= 0 {
            let mut events = 0u32;
            if interest.read {
                events |= syscall::linux::EPOLLIN;
            }
            if interest.write {
                events |= syscall::linux::EPOLLOUT;
            }
            let ret = syscall::epoll_ctl(
                self.epoll_fd,
                syscall::linux::EPOLL_CTL_MOD,
                fd,
                events,
            );
            if ret < 0 {
                return Err(ReactorError::EpollError(format!(
                    "EPOLL_CTL_MOD fd={} failed: {}", fd, ret
                )));
            }
        }
        Ok(())
    }

    /// Proses satu event.
    /// v1.37: Updates last_activity_ms with real monotonic timestamp.
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
                // v1.37: Update aktiviti dengan timestamp monotonic sebenar
                conn.last_activity_ms = syscall::clock_gettime_monotonic_ms();
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
    /// v1.37: Processes events from epoll_wait if epoll is live, else from vec.
    pub fn run_once(&mut self, events: &[Event]) -> usize {
        if self.epoll_fd >= 0 {
            // v1.37: epoll mode — process real epoll events
            self.process_epoll_events(100) // 100ms timeout
        } else {
            // Stub mode — process events from vec
            let mut processed = 0;
            for event in events {
                self.process_event(event);
                processed += 1;
            }
            self.check_all_timeouts();
            self.check_all_taints();
            processed
        }
    }

    /// v1.37: Process events from epoll_wait.
    fn process_epoll_events(&mut self, timeout_ms: i32) -> usize {
        let mut event_buf = vec![0u8; self.max_events * 12]; // epoll_event = 12 bytes (u32 + u64)
        let n = syscall::epoll_wait(
            self.epoll_fd,
            event_buf.as_mut_ptr(),
            self.max_events as i32,
            timeout_ms,
        );

        if n < 0 {
            return 0;
        }

        let n = n as usize;
        for i in 0..n {
            // Parse epoll_event: { u32 events, u64 data }
            let offset = i * 12;
            let events = u32::from_le_bytes([
                event_buf[offset],
                event_buf[offset + 1],
                event_buf[offset + 2],
                event_buf[offset + 3],
            ]);
            let data = u64::from_le_bytes([
                event_buf[offset + 4],
                event_buf[offset + 5],
                event_buf[offset + 6],
                event_buf[offset + 7],
                event_buf[offset + 8],
                event_buf[offset + 9],
                event_buf[offset + 10],
                event_buf[offset + 11],
            ]);
            let fd = data as i32;

            // Check for errors first
            if events & syscall::linux::EPOLLERR != 0 || events & syscall::linux::EPOLLHUP != 0 {
                self.unregister(fd);
                continue;
            }

            // Map epoll events to Logicodex Event
            let kind = if events & syscall::linux::EPOLLIN != 0 {
                EventKind::Readable
            } else if events & syscall::linux::EPOLLOUT != 0 {
                EventKind::Writable
            } else {
                EventKind::Error
            };

            let event = Event { fd, kind };
            self.process_event(&event);
        }

        // Check timeout dan taint untuk semua koneksi
        self.check_all_timeouts();
        self.check_all_taints();

        n
    }

    /// Jalankan event loop (infinite — sehingga stop() dipanggil).
    /// v1.37: Uses epoll_wait for real event-driven I/O. Runs continuously.
    pub fn run(&mut self) {
        self.running = true;
        eprintln!("logicodex v1.37.0-alpha: Reactor started (epoll={}, {} connections)",
            self.epoll_fd, self.connections.len());

        while self.running {
            if self.epoll_fd >= 0 {
                // v1.37: Real epoll event loop — B1 + B4 + B5
                self.process_epoll_events(-1); // blocking wait
            } else {
                // Stub mode — fallback
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        }

        eprintln!("logicodex v1.37.0-alpha: Reactor stopped");
    }

    /// Hentikan Reactor.
    pub fn stop(&mut self) {
        self.running = false;
    }

    /// Check jika reactor sedang berjalan.
    pub fn is_running(&self) -> bool {
        self.running
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
        // v1.37: Close epoll fd via sys_close
        if self.epoll_fd >= 0 {
            syscall::sys_close(self.epoll_fd);
            self.epoll_fd = -1;
        }
    }
}

/// Error Reactor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReactorError {
    DuplicateFd(i32),
    MaxConnectionsReached,
    UnknownFd(i32),
    ServiceNotFound(String),
    /// v1.37: epoll syscall error
    EpollError(String),
}

impl std::fmt::Display for ReactorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReactorError::DuplicateFd(fd) => write!(f, "Fd {} sudah didaftarkan", fd),
            ReactorError::MaxConnectionsReached => write!(f, "Maksimum konegsi tercapai"),
            ReactorError::UnknownFd(fd) => write!(f, "Fd {} tidak dikenali", fd),
            ReactorError::ServiceNotFound(name) => write!(f, "Servis '{}' tidak wujud", name),
            ReactorError::EpollError(msg) => write!(f, "epoll error: {}", msg),
        }
    }
}

impl std::error::Error for ReactorError {}
