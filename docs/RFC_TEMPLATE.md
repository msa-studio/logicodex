# RFC: [Feature Name]

## 1. Motivation
<!-- Why is this needed? What problem does it solve? -->

## 2. Architecture Alignment
<!-- Answer ALL. Any unchecked box = automatic rejection. -->

- [ ] **Static Topology:** How does this respect compile-time topology determination?
  <!-- Shards, capabilities, and gates must be known at compile time -->

- [ ] **Explicit Ownership:** How does this respect the ownership model?
  <!-- Every resource has exactly one owner; channels transfer ownership -->

- [ ] **Shard Isolation:** How does this respect per-core shard boundaries?
  <!-- No shared mutable state between shards; SPSC doors only -->

- [ ] **Deterministic Behavior:** How does this respect deterministic execution?
  <!-- Same input + same topology = same output, every time -->

## 3. Benchmark Impact

### Layer 1 (Micro-Latency)
| Benchmark | Expected Change | Acceptable? |
|---|---|---|
| gate_invoke_latency | ___ % | Must be < 5% |
| door_send/recv | ___ % | Must be < 5% |
| mempool_acquire | ___ % | Must be < 5% |
| callable_lookup | ___ % | Must be < 5% |
| hir_lower_expr | ___ % | Must be < 5% |
| llvm_emit_add | ___ % | Must be < 5% |

### Layer 2 (Throughput)
| Scenario | Expected PPS Change | Acceptable? |
|---|---|---|
| echo_1core | ___ % | Must be < 5% |
| echo_8core | ___ % | Must be < 5% |
| scaling_efficiency | ___ % | Must not drop > 5% |

### Layer 3 (Stability)
| Metric | Expected Change | Acceptable? |
|---|---|---|
| RSS footprint | ___ KB | Must be < 1KB |

## 4. Security Considerations

- [ ] **New unsafe blocks:** ___ (count)
  <!-- Each block must have safety precondition documentation -->

- [ ] **New attack surface:**
  <!-- List any new attack vectors introduced -->

- [ ] **Taint FSM impact:** none / extends / modifies
  <!-- If modifies, explain why -->

- [ ] **Backpressure policy impact:** none / extends / modifies
  <!-- If modifies, explain why -->

## 5. Implementation Plan

### Files
| Action | Path | Description |
|---|---|---|
| Create/Modify | `src/...` | <!-- What this file does --> |

### Tests
| Tier | File | What it tests |
|---|---|---|
| A/B/C | `tests/...` | <!-- Test description --> |

### Validators
| Tier | File | What it validates |
|---|---|---|
| A/B/C | `scripts/validators/...` | <!-- Validation description --> |

### Estimated LOC
- New code: ~___
- Tests: ~___
- Total: ~___

## 6. Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| <!-- e.g., Performance regression --> | High/Med/Low | High/Med/Low | <!-- How to handle --> |

## 7. Acceptance Criteria

- [ ] All Tier A validators pass
- [ ] No regression > 5% vs `benches/BASELINE.json`
- [ ] New tests added with ≥ 80% path coverage
- [ ] All new `unsafe` blocks documented with safety preconditions
- [ ] RFC reviewed and approved by at least 1 other contributor
- [ ] CI passes (all 3 validator tiers)

---

**Submitted by:** @username
**Date:** YYYY-MM-DD
**Target release:** vX.Y.Z-alpha
