# Archived validators (pre-HIR / pre-single-engine era)

These 26 validators were written for the v1.21-v1.45 sprint/pipeline architecture.
They string-match AST variants and structures (Slice, Buffer, Channel, Result/Ok,
V121/V130 parser variants, threading "fasa" stages, the v1.30 --pipeline machinery)
that were intentionally removed when HIR became the single compilation engine. They
now fail structurally - they test an architecture that no longer exists, not a
regression in current code.

Kept as a historical record. NOT part of CI; not a health check.

## What guards the code now
- `cargo test` - the full suite (drift-resistant) is the real QA gate (229/0).
- `make boot-evidence` - freestanding x86_64 QEMU boot proof.
- `scripts/validators/tier_c_stress/validate_v144_freestanding.py` - the one
  validator still testing live (freestanding) capability.
