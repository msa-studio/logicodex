#!/usr/bin/env python3
"""
Logicodex v1.45 — Layer 4: SYN Flood / Connection Flood Simulator

Tests backpressure under rapid connection open/close.
Expected: Backpressure triggers (Block/DropOldest) before resource exhaustion.

Usage: python3 syn_flood.py --target 127.0.0.1:9999 --rate 1000 --duration 30
       python3 syn_flood.py --dry-run
"""

import argparse
import socket
import sys
import time
from concurrent.futures import ThreadPoolExecutor

def flood_worker(target: str):
    """Open and immediately close a connection."""
    host, port = target.rsplit(":", 1)
    try:
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        sock.settimeout(2)
        sock.connect((host, int(port)))
        sock.close()
    except (socket.timeout, ConnectionRefusedError):
        pass

def main():
    parser = argparse.ArgumentParser(description="Connection flood simulator")
    parser.add_argument("--target", default="127.0.0.1:9999")
    parser.add_argument("--rate", type=int, default=1000, help="Connections per second")
    parser.add_argument("--duration", type=int, default=30)
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args()

    if args.dry_run:
        print("[syn_flood] DRY RUN — Script valid")
        print(f"  Would flood: {args.target} at {args.rate} conn/s")
        print("  Expected: Backpressure policy triggers")
        return 0

    print(f"[syn_flood] Flooding {args.target} at {args.rate} conn/s for {args.duration}s...")
    end = time.time() + args.duration
    total = 0

    with ThreadPoolExecutor(max_workers=64) as pool:
        while time.time() < end:
            batch = min(args.rate // 10, 100)
            for _ in range(batch):
                pool.submit(flood_worker, args.target)
            total += batch
            time.sleep(0.1)

    print(f"[syn_flood] Complete — {total} connections attempted")
    return 0

if __name__ == "__main__":
    sys.exit(main())
