// =========================================================================
// Logicodex v1.45 — Layer 2: Reactor Throughput — Flood Client
//
// Multi-threaded UDP/TCP packet generator for stress testing.
// Sends fixed-size payloads as fast as possible, reports PPS.
//
// Usage: ./flood_client <host:port> <threads> <duration_secs> [payload_size]
//   Example: ./flood_client 127.0.0.1:9999 4 30 64
// =========================================================================

use std::net::TcpStream;
use std::io::Write;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

static TOTAL_SENT: AtomicU64 = AtomicU64::new(0);
static TOTAL_BYTES: AtomicU64 = AtomicU64::new(0);

fn flood_worker(addr: String, payload: Vec<u8>, duration: Duration) {
    let end = Instant::now() + duration;
    let mut local_sent = 0u64;
    let mut local_bytes = 0u64;

    // Each worker opens its own connection
    let mut stream = match TcpStream::connect(&addr) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[flood] Worker connect failed: {}", e);
            return;
        }
    };

    while Instant::now() < end {
        match stream.write_all(&payload) {
            Ok(_) => {
                local_sent += 1;
                local_bytes += payload.len() as u64;
            }
            Err(_) => {
                // Reconnect on error
                if let Ok(s) = TcpStream::connect(&addr) {
                    stream = s;
                }
            }
        }
    }

    TOTAL_SENT.fetch_add(local_sent, Ordering::Relaxed);
    TOTAL_BYTES.fetch_add(local_bytes, Ordering::Relaxed);
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 4 {
        eprintln!("Usage: {} <host:port> <threads> <duration_secs> [payload_size]", args[0]);
        eprintln!("  Example: {} 127.0.0.1:9999 4 30 64", args[0]);
        std::process::exit(1);
    }

    let addr = args[1].clone();
    let num_threads: usize = args[2].parse().expect("threads must be a number");
    let duration_secs: u64 = args[3].parse().expect("duration must be a number");
    let payload_size: usize = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(64);
    let duration = Duration::from_secs(duration_secs);

    let payload = vec![0xABu8; payload_size];

    println!("[flood] Starting flood: addr={}, threads={}, duration={}s, payload={}B",
        addr, num_threads, duration_secs, payload_size);

    let start = Instant::now();

    let handles: Vec<_> = (0..num_threads)
        .map(|i| {
            let addr = addr.clone();
            let payload = payload.clone();
            thread::spawn(move || {
                // Stagger starts by 10ms to avoid thundering herd
                thread::sleep(Duration::from_millis(i as u64 * 10));
                flood_worker(addr, payload, duration);
            })
        })
        .collect();

    for h in handles {
        h.join().unwrap();
    }

    let elapsed = start.elapsed().as_secs_f64();
    let total_sent = TOTAL_SENT.load(Ordering::Relaxed);
    let total_bytes = TOTAL_BYTES.load(Ordering::Relaxed);
    let pps = total_sent as f64 / elapsed;
    let gbps = (total_bytes as f64 * 8.0) / (elapsed * 1_000_000_000.0);

    println!("\n[flood] COMPLETE");
    println!("  Duration:  {:.2}s", elapsed);
    println!("  Packets:   {}", total_sent);
    println!("  Bytes:     {}", total_bytes);
    println!("  PPS:       {:.0}", pps);
    println!("  Gbps:      {:.3}", gbps);
    println!("  Per-core:  {:.0} PPS/thread", pps / num_threads as f64);
}
