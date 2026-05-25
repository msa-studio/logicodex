# Logicodex v1.45 — Quantitative Benchmark Suite

## The Deterministic Benchmark Framework

Every number is architecture-correlated. No noisy generic benchmarks.

---

## Quick Start

```bash
# Run all 4 layers (full mode, ~2 hours)
cd benches/harness && bash run_all.sh

# Quick mode (~10 minutes)
bash run_all.sh --quick

# Compare against baseline
bash run_all.sh --compare

# Run individual layers
cd benches/criterion/micro && cargo bench --bench gate_latency
cd benches/reactor && bash throughput.sh 30 64
cd benches/stability && bash valgrind_check.sh
cd benches/security && python3 slowloris.py --dry-run
```

## 4-Layer Matrix

| Layer | Focus | Files | Target |
|---|---|---|---|
| **1 — Micro** | Per-operation latency | `criterion/micro/*.rs` (6 benchmarks) | < 50ns gate, < 100ns channel, CI 95% |
| **2 — Reactor** | PPS scaling 1→8 cores | `reactor/echo_server.rs + flood_client.rs` | > 85% scaling efficiency |
| **3 — Stability** | RSS 24h, valgrind | `stability/rss_monitor.py + valgrind_check.sh` | **0KB creep**, 0 leaks |
| **4 — Security** | Attack simulation | `security/*.py` (4 simulators) | **Fail-stop 100%** |

## Regression Rules (BASELINE.json)

| Condition | Action |
|---|---|
| Latency regression > 5% | WARNING — investigate |
| Latency regression > 10% | FAIL — block merge |
| Scaling efficiency drops > 5% | FAIL — architectural regression |
| RSS delta > 0 KB | FAIL — memory leak |
| Valgrind: definitely lost > 0 | FAIL — definite leak |
| Any security fail-stop = false | FAIL — security vulnerability |

## Directory Structure

```
benches/
├── BASELINE.json              ← Gold standard (committed, versioned)
├── README.md                  ← This file
├── criterion/
│   └── micro/                 ← Layer 1: 6 criterion benchmarks
├── reactor/                   ← Layer 2: echo server + flood client
├── stability/                 ← Layer 3: RSS monitor + valgrind
├── security/                  ← Layer 4: 4 attack simulators
├── harness/                   ← run_all.sh + compare_baseline.py
└── results/                   ← Generated (gitignored)
```

## Adding a New Benchmark

1. Write benchmark in appropriate layer directory
2. Add expected values to `BASELINE.json`
3. Update `validate_v145_benchmarks.py` (Tier C validator)
4. Run `compare_baseline.py` to verify

## CI Integration

Tier C validator runs on every PR:
```bash
python3 scripts/validators/tier_c_stress/validate_v145_benchmarks.py
```

Full benchmark suite runs nightly:
```bash
cd benches/harness && bash run_all.sh --compare
```
