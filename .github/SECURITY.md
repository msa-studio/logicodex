# Security Policy

## Supported Versions

| Version | Status |
|---------|--------|
| v1.45.0-alpha | ✅ Current — receiving security updates |
| v1.44.x | ✅ Supported |
| v1.43.x and earlier | ❌ No longer supported |

## Reporting a Vulnerability

**Email:** mymsastudio@gmail.com  
**Subject prefix:** `[SECURITY]`

### What to Include

- Logicodex version (`logicodex --version`)
- Target platform (Linux x86_64, macOS, WASM, Freestanding)
- Description of the vulnerability
- Steps to reproduce (minimal `.ldx` file if applicable)
- Impact assessment (what an attacker could do)

### Response Timeline

| Stage | Timeline |
|-------|----------|
| Acknowledgment | Within 7 days |
| Initial assessment | Within 14 days |
| Fix or mitigation | Within 30 days (critical), 60 days (moderate) |
| Public disclosure | Coordinated with reporter |

## Security Model

Logicodex uses **compile-time capability gates** to prevent unauthorized access to dangerous operations (hardware I/O, network, file system). All capability checks happen at compile time with **zero runtime cost**.

### What We Consider Security Issues

- Bypass of capability gates (calling gated operations without declaring the gate)
- Memory safety violations in production code (`unsafe` blocks that violate invariants)
- Information disclosure through `.cap` audit files
- Taint FSM bypass (malicious connections not being detected)
- WASM sandbox escape (guest accessing host hardware without Host Reactor)
- Audio callback safety violations (ISR doing I/O, allocation, or recursion)

### What Is Out of Scope

- Issues in test code (`#[cfg(test)]` blocks)
- Issues in example programs (unless the issue is in the compiler itself)
- Build failures due to missing dependencies (documented in HANDBOOK.md § Installation)

## Security Audit History

| Date | Version | Auditor | Findings |
|------|---------|---------|----------|
| 2026-05-25 | v1.45.0-alpha | Internal | 0 unwrap in production, 141 unsafe blocks documented, 0 dead code |
