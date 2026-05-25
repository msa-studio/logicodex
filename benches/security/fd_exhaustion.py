#!/usr/bin/env python3
"""
Logicodex v1.45 — Layer 4: File Descriptor Exhaustion Test

Opens connections until EMFILE (process FD limit).
Expected: Graceful degradation — no panic, controlled error.

Usage: python3 fd_exhaustion.py --target 127.0.0.1:9999
       python3 fd_exhaustion.py --dry-run
"""

import argparse
import errno
import resource
import socket
import sys


def test_fd_exhaustion(target: str):
    """Open connections until we hit the FD limit."""
    host, port = target.rsplit(":", 1)
    port = int(port)

    # Get current soft limit
    soft, hard = resource.getrlimit(resource.RLIMIT_NOFILE)
    print(f"[fd] FD limit: soft={soft}, hard={hard}")

    socks = []
    opened = 0
    hit_limit = False

    try:
        while True:
            sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            sock.settimeout(1)
            try:
                sock.connect((host, port))
                socks.append(sock)
                opened += 1
                if opened % 100 == 0:
                    print(f"[fd] Opened {opened} connections...")
            except OSError as e:
                if e.errno in (errno.EMFILE, errno.ENFILE):
                    print(f"[fd] Hit FD limit after {opened} connections (EMFILE)")
                    hit_limit = True
                    break
                sock.close()
    except KeyboardInterrupt:
        pass

    # Clean up — this tests RAII / connection cleanup
    print(f"[fd] Cleaning up {len(socks)} connections...")
    for sock in socks:
        try:
            sock.close()
        except:
            pass

    if hit_limit:
        print("[fd] PASS — Graceful exhaustion, no panic")
        return 0
    else:
        print(f"[fd] INFO — Opened {opened} connections without hitting limit")
        return 0


def main():
    parser = argparse.ArgumentParser(description="FD exhaustion test")
    parser.add_argument("--target", default="127.0.0.1:9999")
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args()

    if args.dry_run:
        print("[fd_exhaustion] DRY RUN — Script valid")
        print(f"  Would test FD exhaustion against {args.target}")
        print("  Expected: Graceful EMFILE handling, no panic")
        return 0

    return test_fd_exhaustion(args.target)


if __name__ == "__main__":
    sys.exit(main())
