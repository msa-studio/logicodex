#!/usr/bin/env python3
"""
Logicodex v1.45 — Layer 4: Malformed Packet Injector

Sends random/garbage data to test error handling.
Expected: EPOLLERR → immediate connection cleanup, no crash.

Usage: python3 malformed.py --target 127.0.0.1:9999 --packets 1000
       python3 malformed.py --dry-run
"""

import argparse
import os
import socket
import sys

def send_garbage(target: str, count: int):
    host, port = target.rsplit(":", 1)
    for i in range(count):
        try:
            sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            sock.settimeout(2)
            sock.connect((host, int(port)))
            # Send random garbage
            garbage = os.urandom(min(64 + i % 1024, 2048))
            sock.send(garbage)
            sock.close()
        except (socket.timeout, ConnectionRefusedError, BrokenPipeError):
            pass

def main():
    parser = argparse.ArgumentParser(description="Malformed packet injector")
    parser.add_argument("--target", default="127.0.0.1:9999")
    parser.add_argument("--packets", type=int, default=1000)
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args()

    if args.dry_run:
        print("[malformed] DRY RUN — Script valid")
        print(f"  Would inject: {args.packets} random packets to {args.target}")
        print("  Expected: EPOLLERR → immediate cleanup, no crash")
        return 0

    print(f"[malformed] Injecting {args.packets} malformed packets...")
    send_garbage(args.target, args.packets)
    print("[malformed] Complete — Server should have cleaned all connections")
    return 0

if __name__ == "__main__":
    sys.exit(main())
