# Logicodex v1.45.0 — Quantitative Stabilization Plan
## The Deterministic Benchmark Framework

**Status:** PLANNING (approved → implementation)
**Objective:** Transform Logicodex from "feature-complete prototype" to "runtime architecture with audited performance characteristics"
**Philosophy:** Every number must be architecture-correlated. No noisy generic benchmarks.

---

## Executive Summary

Logicodex v1.44 has **137/137 validators passing**, **~40,700 LOC**, **3 compilation backends** (Native, WASM, Freestanding), and **15 freestanding gaps resolved**. But we have **zero quantitative performance data**. This plan closes that gap.

### Why This Matters

| Audience | What they ask | What v1.45 delivers |
|---|---|---|
| **Systems engineer** | "Does it scale linearly on 8 cores?" | PPS vs Core graph, >85% scaling efficiency |
| **Security auditor** | "Does it fail-stop under attack?" | Taint FSM + backpressure stress test results |
| **Memory-critical embedded** | "Does it leak over 24h?" | RSS stability graph, 0KB creep target |
| **CTO evaluating adoption** | "Is this production-ready?" | BASELINE.json with every merge, >5% regression = CI fail |

### Architecture-Correlated Metrics

Every benchmark maps directly to a Logicodex architectural component:

```
Gate invocation cost        → Capability Fabric (v1.32)
Door message overhead       → Channel< T > zero-copy (v1.30)
MemoryPool acquire/release  → Allocator + provenance (v1.21)
PPS vs Core scaling         → Sharded Reactor (v1.34/v1.39)
RSS 24h stability           → RAII auto-cleanup (v1.33)
Taint FSM under attack      → Security model (v1.32/v1.37)
```

---

## 1. The Benchmark Matrix (4 Layers)

### Layer 1: Micro-Benchmarking (Latency Sensitive)

**Goal:** Measure per-operation latency with statistical significance (n ≥ 1000, confidence ≥ 95%).

| Benchmark | Target | Architecture Component |
|---|---|---|
| `gate_invoke_latency` | < 50ns | Capability Fabric gate check |
| `door_send_latency` | < 100ns | Channel send (zero-copy) |
| `door_recv_latency` | < 100ns | Channel receive |
| `mempool_acquire` | < 20ns | MemoryPool alloc |
| `mempool_release` | < 20ns | MemoryPool free |
| `callable_lookup` | < 30ns | CallableRegistry by-name lookup |
| `hir_lower_expr` | < 200ns | AST → HIR expression lowering |
| `llvm_emit_add` | < 500ns | LLVM IR generation for `i64 + i64` |

**Tool:** `criterion-rs` (Rust standard for statistical benchmarking)
**Output:** HTML report + CSV + `benches/results/micro/latency.json`
**Statistical rigor:** Min 1000 iterations, warm-up 3s, outlier filtering (IQR), confidence interval 95%

### Layer 2: Reactor Throughput (Scaling Efficiency)

**Goal:** Measure PPS (packets per second) scaling across 1/2/4/8 cores. Target: >85% scaling efficiency.

```
Scaling Efficiency = (PPS_N_cores / N) / PPS_1_core * 100%

Target:
  1 core:  baseline
  2 cores: ≥ 90% efficiency
  4 cores: ≥ 87% efficiency
  8 cores: ≥ 85% efficiency
```

| Benchmark | Target | Architecture Component |
|---|---|---|
| `echo_1core_pps` | > 100K PPS | Network Reactor (v1.37) |
| `echo_2core_pps` | > 180K PPS | Sharded Reactor (v1.39) |
| `echo_4core_pps` | > 350K PPS | Shard affinity (v1.34) |
| `echo_8core_pps` | > 680K PPS | Parallel execution |
| `scaling_efficiency` | ≥ 85% | Cross-shard SPSC doors |

**Tool:** Custom epoll-based echo server + `iperf`-style UDP flood generator
**Output:** `benches/results/reactor/throughput.json` + scaling graph
**Environment:** Linux bare-metal or QEMU with SMP (disable CPU frequency scaling: `cpufreq-set -g performance`)

### Layer 3: System Stability (Long-Running Profile)

**Goal:** Prove zero memory creep over extended execution. Target: 0KB RSS growth.

| Benchmark | Duration | Target | Architecture Component |
|---|---|---|---|
| `rss_1h_stability` | 1 hour | ΔRSS ≤ 0KB | RAII auto-cleanup (v1.33) |
| `rss_6h_stability` | 6 hours | ΔRSS ≤ 0KB | Connection lifecycle |
| `rss_24h_stability` | 24 hours | ΔRSS ≤ 0KB | Memory pool + allocator |
| `valgrind_leak_check` | 10 min | 0 leaks, 0 errors | All allocation paths |
| `fd_count_stability` | 1 hour | Δfd = 0 | Socket RAII (no leaks) |

**Tool:** `valgrind --leak-check=full` + `procfs` (/proc/[pid]/status, /proc/[pid]/fd/) snapshots every 60s
**Output:** `benches/results/stability/rss_timeseries.csv` + stability graph
**Acceptance:** Linear regression slope on RSS vs time must be ≤ 0.001 KB/hour (effectively zero)

### Layer 4: Stress/Security Profile
n**Goal:** Verify Taint FSM and backpressure behave correctly under attack.

| Benchmark | Attack Type | Expected Behavior | Architecture Component |
|---|---|---|---|
| `slowloris_100conn` | Slowloris (partial HTTP headers) | Taint → Suspicious → Closing | Taint FSM (v1.33) |
| `syn_flood_1k` | Connection flooding | Backpressure: Block/DropOldest | Backpressure policy |
| `malformed_pkt` | Random byte injection | EPOLLERR → immediate cleanup | Error handling |
| `fd_exhaustion` | Open until EMFILE | Graceful degrade, no panic | Resource limits |
| `callback_recursion` | Audio callback calls itself | StrictAudioContext rejection | v1.42/v1.43 |

**Tool:** Custom attack simulators in Python (no external dependencies)
**Output:** `benches/results/security/stress_profile.json`
**Acceptance:** All attacks must result in controlled fail-stop (no crash, no undefined behavior)

---

## 2. File Structure

```
benches/
├── README.md                          # How to run benchmarks
├── BASELINE.json                      # Gold standard (committed, versioned)
├── criterion/
│   ├── micro/
│   │   ├── gate_latency.rs            # Layer 1: Gate invoke
│   │   ├── door_latency.rs            # Layer 1: Channel send/recv
│   │   ├── mempool_latency.rs         # Layer 1: MemoryPool
│   │   ├── callable_lookup.rs         # Layer 1: CallableRegistry
│   │   ├── hir_lower.rs               # Layer 1: AST → HIR
│   │   └── llvm_emit.rs               # Layer 1: LLVM IR gen
│   └── Criterion.toml                 # Criterion configuration
├── reactor/
│   ├── echo_server.rs                 # Echo server harness
│   ├── flood_client.rs                # UDP flood generator
│   └── throughput.sh                  # 1/2/4/8 core runner
├── stability/
│   ├── rss_monitor.py                 # procfs RSS snapshotter
│   ├── longrun.sh                     # 1h/6h/24h runner
│   └── valgrind_check.sh              # Leak check runner
├── security/
│   ├── slowloris.py                   # Slowloris simulator
│   ├── syn_flood.py                   # Connection flood
│   ├── malformed.py                   # Random byte injector
│   └── fd_exhaustion.py               # FD limit test
├── harness/
│   ├── run_all.sh                     # Run all 4 layers
│   ├── run_layer1.sh                  # Micro-benchmarks only
│   ├── run_layer2.sh                  # Throughput only
│   ├── run_layer3.sh                  # Stability only
│   ├── run_layer4.sh                  # Security only
│   └── compare_baseline.py            # Compare vs BASELINE.json
└── results/                           # Generated (gitignored)
    ├── micro/
    │   └── latency.json
    ├── reactor/
    │   └── throughput.json
    ├── stability/
    │   └── rss_timeseries.csv
    └── security/
        └── stress_profile.json
```

---

## 3. BASELINE.json Format

The gold standard. Updated manually on each significant merge. CI compares against it.

```json
{
  "version": "1.45.0-alpha",
  "date": "2026-05-25",
  "machine": "qemu-x86_64-smp8-4GB",
  "thresholds": {
    "regression_warn_percent": 5.0,
    "regression_fail_percent": 10.0
  },
  "layer1_micro": {
    "gate_invoke_latency_ns": { "mean": 45, "p95": 52, "p99": 68 },
    "door_send_latency_ns":   { "mean": 85, "p95": 98, "p99": 120 },
    "door_recv_latency_ns":   { "mean": 82, "p95": 95, "p99": 115 },
    "mempool_acquire_ns":     { "mean": 18, "p95": 22, "p99": 30 },
    "mempool_release_ns":     { "mean": 16, "p95": 20, "p99": 28 },
    "callable_lookup_ns":     { "mean": 25, "p95": 30, "p99": 42 },
    "hir_lower_expr_ns":      { "mean": 180, "p95": 210, "p99": 280 },
    "llvm_emit_add_ns":       { "mean": 420, "p95": 480, "p99": 620 }
  },
  "layer2_reactor": {
    "echo_1core_pps": 105000,
    "echo_2core_pps": 195000,
    "echo_4core_pps": 370000,
    "echo_8core_pps": 720000,
    "scaling_efficiency_2core_pct": 92.8,
    "scaling_efficiency_4core_pct": 88.1,
    "scaling_efficiency_8core_pct": 85.7
  },
  "layer3_stability": {
    "rss_1h_delta_kb": 0,
    "rss_6h_delta_kb": 0,
    "rss_24h_delta_kb": 0,
    "valgrind_leaks": 0,
    "valgrind_errors": 0,
    "fd_count_delta": 0
  },
  "layer4_security": {
    "slowloris_fail_stop": true,
    "syn_flood_backpressure_triggered": true,
    "malformed_cleanup_immediate": true,
    "fd_exhaustion_graceful": true,
    "callback_recursion_rejected": true
  }
}
```

### Regression Rules

```
IF new_mean > baseline_mean * 1.05  → WARNING ( investigate )
IF new_mean > baseline_mean * 1.10  → FAIL    ( block merge )
IF scaling_efficiency drops > 5%    → FAIL    ( architectural regression )
IF rss_delta > 0 KB                 → FAIL    ( memory leak )
IF valgrind_leaks > 0               → FAIL    ( definite leak )
IF any security_fail_stop = false   → FAIL    ( security vulnerability )
```

---

## 4. RFC_TEMPLATE.md

Every new feature after v1.45 must complete this template. No exceptions.

```markdown
# RFC: [Feature Name]

## 1. Motivation
Why is this needed? What problem does it solve?

## 2. Architecture Alignment
Answer ALL of the following:
- [ ] How does this respect **Static Topology**?
- [ ] How does this respect **Explicit Ownership**?
- [ ] How does this respect **Shard Isolation**?
- [ ] How does this respect **Deterministic Behavior**?

## 3. Benchmark Impact
- [ ] Which Layer 1 micro-benchmarks are affected? (list)
- [ ] Expected latency change: ___% (must be < 5% or justify)
- [ ] Which Layer 2 throughput scenario is affected? (list)
- [ ] Expected PPS change: ___% (must be < 5% or justify)
- [ ] Memory footprint change: ___ bytes (must be < 1KB or justify)

## 4. Security Considerations
- [ ] New unsafe code? (count blocks, justify each)
- [ ] New attack surface? (list)
- [ ] Taint FSM impact? (none / extends / modifies)
- [ ] Backpressure policy impact? (none / extends / modifies)

## 5. Implementation Plan
- Files to create/modify:
- Tests to add:
- Validators to add (tier A/B/C):
- Estimated LOC:

## 6. Risks & Mitigations
| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|

## 7. Acceptance Criteria
- [ ] All Tier A validators pass
- [ ] No regression > 5% vs BASELINE.json
- [ ] New tests added ≥ 80% path coverage
- [ ] RFC reviewed by at least 1 other contributor
```

---

## 5. Validator Integration

Benchmarks become **Tier C (Performance)** validators:

```
scripts/validators/
├── tier_a_core/          (existing — 7 validators)
├── tier_b_feature/       (existing — 13 validators)
└── tier_c_stress/        (existing — 7 validators)
    ├── validate_v140_wasm_codegen.py
    ├── validate_v141_host_reactor.py
    ├── validate_v142_raylib_pending.py
    ├── validate_v143_raylib_audio.py
    ├── validate_v144_freestanding.py
    ├── validate_demo_raylib_box.py
    ├── validate_v130_pipeline.py
    └── NEW: validate_v145_benchmarks.py   # ← v1.45
```

### `validate_v145_benchmarks.py` checks:
1. `benches/BASELINE.json` exists and is valid JSON
2. All 4 layer result files exist (can be empty before first run)
3. `benches/README.md` documents how to run
4. `docs/RFC_TEMPLATE.md` exists
5. No regression in latest `benches/results/` vs `BASELINE.json`

---

## 6. Success Criteria (Definition of Done)

| # | Criterion | Measurement |
|---|---|---|
| 1 | All 8 Layer 1 benchmarks run and produce `latency.json` | `ls benches/results/micro/latency.json` |
| 2 | Layer 2 shows >85% scaling efficiency on 8 cores | `scaling_efficiency_8core_pct >= 85.0` |
| 3 | Layer 3 shows 0KB RSS delta over 1h minimum | `rss_1h_delta_kb == 0` |
| 4 | Layer 4 shows fail-stop on all 5 attack vectors | All `*_fail_stop == true` |
| 5 | `BASELINE.json` committed and documented | File exists + schema documented |
| 6 | `RFC_TEMPLATE.md` in `docs/` | File exists |
| 7 | Benchmark CI integration (Tier C validator) | `validate_v145_benchmarks.py` passes |

---

## 7. Estimated Effort

| Phase | Files | LOC (est.) | Time (est.) |
|---|---|---|---|
| Layer 1: Micro (criterion) | 6 `.rs` + config | ~400 | 1 session |
| Layer 2: Reactor throughput | 2 `.rs` + scripts | ~350 | 1 session |
| Layer 3: Stability | 3 Python + scripts | ~250 | 1 session |
| Layer 4: Security stress | 5 Python | ~400 | 1 session |
| Harness + BASELINE | 5 shell + JSON | ~200 | 1 session |
| RFC_TEMPLATE.md | 1 markdown | ~80 | 1 session |
| **Total** | **22 files** | **~1,680 LOC** | **6 sessions** |

---

*Plan drafted: 2026-05-25*
*Status: AWAITING APPROVAL*
*Next step: User reviews → approve → implement session by session*
