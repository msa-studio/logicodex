// =========================================================================
// Logicodex v1.45 — Layer 2: Reactor Throughput Benchmark — Echo Server
//
// Single-threaded epoll echo server for measuring PPS.
// Mirrors the Sharded Reactor pattern (v1.34/v1.39).
//
// Usage: ./echo_server <port> [core_id]
//   If core_id is set, pins to that CPU core (via sched_setaffinity).
// =========================================================================

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::os::unix::io::{AsRawFd, RawFd};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

/// Connection state for taint FSM (mirrors v1.33).
#[derive(Debug, Clone, Copy)]
enum ConnState {
    Healthy,
    Suspicious,
    Closing,
}

struct Connection {
    fd: RawFd,
    state: ConnState,
    rx_buf: Vec<u8>,
    tx_buf: Vec<u8>,
    last_activity_ms: u64,
}

/// Global packet counter (shared across threads for multi-core test).
static TOTAL_PACKETS: AtomicU64 = AtomicU64::new(0);
static TOTAL_BYTES: AtomicU64 = AtomicU64::new(0);

fn run_echo_server(port: u16, core_id: Option<usize>) -> std::io::Result<()> {
    // Pin to core if requested (simulates shard affinity)
    if let Some(core) = core_id {
        unsafe {
            let mut cpu_set = std::mem::zeroed::<libc::cpu_set_t>();
            libc::CPU_SET(core, &mut cpu_set);
            libc::sched_setaffinity(0, std::mem::size_of::<libc::cpu_set_t>(), &cpu_set);
        }
    }

    let listener = TcpListener::bind(format!("127.0.0.1:{}", port))?;
    listener.set_nonblocking(true)?;
    println!("[echo] Listening on 127.0.0.1:{} (core={:?})", port, core_id);

    let epoll_fd = unsafe { libc::epoll_create1(libc::EPOLL_CLOEXEC) };
    if epoll_fd < 0 {
        return Err(std::io::Error::last_os_error());
    }

    // Register listener
    let mut event = libc::epoll_event {
        events: (libc::EPOLLIN) as u32,
        u64: listener.as_raw_fd() as u64,
    };
    unsafe {
        libc::epoll_ctl(epoll_fd, libc::EPOLL_CTL_ADD, listener.as_raw_fd(), &mut event);
    }

    let mut connections: std::collections::HashMap<RawFd, Connection> = std::collections::HashMap::new();
    let mut events: [libc::epoll_event; 1024] = unsafe { std::mem::zeroed() };

    let start = Instant::now();
    let mut last_report = start;

    loop {
        let timeout_ms = 100; // 100ms timeout for periodic reporting
        let n = unsafe {
            libc::epoll_wait(epoll_fd, events.as_mut_ptr(), 1024, timeout_ms)
        };

        if n < 0 {
            break;
        }

        for i in 0..n {
            let fd = events[i as usize].u64 as RawFd;
            let ev = events[i as usize].events as i32;

            if fd == listener.as_raw_fd() {
                // Accept new connection
                if let Ok((stream, _)) = listener.accept() {
                    stream.set_nonblocking(true).ok();
                    let new_fd = stream.as_raw_fd();
                    std::mem::forget(stream); // Keep FD open (we manage it manually)

                    let mut ev = libc::epoll_event {
                        events: (libc::EPOLLIN | libc::EPOLLET) as u32,
                        u64: new_fd as u64,
                    };
                    unsafe {
                        libc::epoll_ctl(epoll_fd, libc::EPOLL_CTL_ADD, new_fd, &mut ev);
                    }
                    connections.insert(new_fd, Connection {
                        fd: new_fd,
                        state: ConnState::Healthy,
                        rx_buf: Vec::with_capacity(4096),
                        tx_buf: Vec::with_capacity(4096),
                        last_activity_ms: start.elapsed().as_millis() as u64,
                    });
                }
                continue;
            }

            // Handle existing connection
            if ev & libc::EPOLLIN != 0 {
                if let Some(conn) = connections.get_mut(&fd) {
                    let mut buf = [0u8; 4096];
                    match unsafe { libc::recv(fd, buf.as_mut_ptr() as *mut _, buf.len(), 0) } {
                        n if n > 0 => {
                            conn.rx_buf.extend_from_slice(&buf[..n as usize]);
                            conn.last_activity_ms = start.elapsed().as_millis() as u64;

                            // Echo back (simple echo server)
                            conn.tx_buf.extend_from_slice(&buf[..n as usize]);
                            unsafe { libc::send(fd, conn.tx_buf.as_ptr() as *const _, conn.tx_buf.len(), 0) };
                            conn.tx_buf.clear();

                            TOTAL_PACKETS.fetch_add(1, Ordering::Relaxed);
                            TOTAL_BYTES.fetch_add(n as u64, Ordering::Relaxed);
                        }
                        _ => {
                            conn.state = ConnState::Closing;
                        }
                    }
                }
            }

            if ev & (libc::EPOLLERR | libc::EPOLLHUP) != 0 {
                if let Some(conn) = connections.remove(&fd) {
                    unsafe { libc::close(conn.fd) };
                }
            }
        }

        // Clean up closing connections
        connections.retain(|_, conn| {
            if matches!(conn.state, ConnState::Closing) {
                unsafe { libc::close(conn.fd) };
                false
            } else {
                true
            }
        });

        // Periodic stats report (every 5 seconds)
        let now = Instant::now();
        if now.duration_since(last_report).as_secs() >= 5 {
            let elapsed = now.duration_since(start).as_secs_f64();
            let pkts = TOTAL_PACKETS.load(Ordering::Relaxed);
            let bytes = TOTAL_BYTES.load(Ordering::Relaxed);
            let pps = if elapsed > 0.0 { pkts as f64 / elapsed } else { 0.0 };
            let mbps = if elapsed > 0.0 { (bytes as f64 * 8.0) / (elapsed * 1_000_000.0) } else { 0.0 };
            println!("[stats] elapsed={:.1}s packets={} PPS={:.0} throughput={:.1} Mbps conns={}",
                elapsed, pkts, pps, mbps, connections.len());
            last_report = now;
        }
    }

    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let port = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(9999);
    let core_id = args.get(2).and_then(|s| s.parse().ok());

    if let Err(e) = run_echo_server(port, core_id) {
        eprintln!("[echo] Error: {}", e);
        std::process::exit(1);
    }
}
