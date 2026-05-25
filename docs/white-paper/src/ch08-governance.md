# Chapter 8: Pengurusan Projek — Architecture Freeze, RFC, Governance

> *"Projek matang bukan projek yang tidak berubah — ia adalah projek yang berubah secara terkawal."*

> **Nota Hubungan Dokumen:** Pengurusan projek ini dibina di atas asas yang ditetapkan dalam [`WHITE_PAPER.md`](../../WHITE_PAPER.md) (v1.21 baseline) — khususnya Seksyen 14 (Open-Source Governance, Dual Licensing, dan Trademark Safeguards) dan Seksyen 15 (Research Roadmap). Architecture Freeze, RFC process, dan validator tiering yang dibincangkan di sini adalah **pelaksanaan operasi** daripada prinsip governance yang dibentangkan dalam baseline document. Kedua-dua dokumen selaras: baseline menetapkan prinsip; wiki ini merakam pelaksanaan dan keputusan praktikal.

---

## Architecture Freeze Policy: Apa dan Mengapa? {#freeze}

### Definisi

Architecture Freeze adalah keadaan di mana tiada ciri arkitektur baharu boleh ditambah tanpa melalui proses RFC (Request for Comments) dan mendapat kelulusan arkitek. Freeze tidak bermaksud "projek mati" — ia bermaksud "perubahan arkitektur memerlukan justifikasi."

### Sejarah Freeze dalam Logicodex

Freeze dimulakan selepas v1.45.0-alpha selepas perbincangan ini:

> *"Kita kena jaga integrity architecture dengan freeze dulu unless explicitly minta unfreeze. Tapi boleh buat minor adjustment untuk improve existing functions."*

### Apa yang Dibenarkan Semasa Freeze

| Kategori | Contoh | Perlu RFC? |
|---|---|---|
| **Bug fix** | Membetulkan ralat dalam kod sedia ada | Tidak |
| **Minor adjustment** | Validator tiering, benchmark framework, documentation polish | Tidak |
| **Performance optimization** | Mempercepat fungsi sedia ada | Tidak |
| **Feature kecil dalam modul sedia ada** | Menambah satu fungsi audio baru | Ya (ringkas) |
| **Modul baharu** | Menambah database connector | Ya (penuh) |
| **Perubahan arkitektur** | Mengubah model concurrency | Ya (penuh + justification) |
| **Unfreeze** | Menamatkan freeze secara keseluruhan | Ya (justification mendalam) |

### Apa yang Dihalang Semasa Freeze

| Dihalang | Sebab |
|---|---|
| Menambah "fitur kecil" tanpa RFC | Feature creep tanpa disiplin |
| Mengubah prinsip asal | Melanggar integriti arkitektur |
| Membypass alignment checks | RFC tanpa 3/4 checks diluluskan |
| "Cuba-cuba" implementation | Tiada justification, tiada rekabentuk |

---

## RFC Template: 4 Mandatory Alignment Checks {#rfc}

### Struktur RFC

Setiap RFC mesti mengikuti template berikut:

```markdown
# RFC-XXXX: [Tajuk]

## Penulis
- Nama:
- Tarikh:
- Versi Logicodex:

## Ringkasan
[2-3 ayat ringkasan cadangan]

## Motivasi
[Mengapa RFC ini diperlukan? Apa masalah yang diselesaikan?]

## Rekabentuk Teknikal
[Detail rekabentuk, termasuk kod contoh jika relevan]

## Mandatory Alignment Checks

- [ ] **Static Topology**: Cadangan ini mengekalkan model topology statik.
  Justifikasi: ...

- [ ] **Explicit Ownership**: Cadangan ini mengekalkan model ownership eksplisit.
  Justifikasi: ...

- [ ] **Shard Isolation**: Cadangan ini mengekalkan isolasi shard.
  Justifikasi: ...

- [ ] **Deterministic Behavior**: Cadangan ini mengekalkan tingkah laku deterministik.
  Justifikasi: ...

**Sekurang-kurangnya 3 daripada 4 checks mesti diluluskan.**

## Trade-off
[Apa yang diorban untuk mendapatkan manfaat ini?]

## Backward Compatibility
[Adakah ini memecahkan kod sedia ada? Bagaimana migrasi?]

## Rujukan
[RFC lain, perbincangan, atau dokumentasi yang relevan]
```

### Proses Review

```text
Penulis hantar RFC
        │
        ▼
┌───────────────┐
│ Auto-check    │── Gagal 2+ checks → TOLAK automatik
│ 4 alignment   │
└───────────────┘
        │
        ▼ (lulus ≥3 checks)
┌───────────────┐
│ Review oleh   │── Reject → Penulis kemas kini
│ Arkitek       │
│ (7 hari)      │── Approve → Merge ke roadmap
└───────────────┘
        │
        ▼ (approve)
┌───────────────┐
│ Implementasi  │── 14 hari untuk prototype
│ & Validation  │── 30 hari untuk production
└───────────────┘
```

---

## Validator Tiering: A/B/C {#validator}

### Definisi

Validator tiering adalah sistem pengelasan validators ke dalam 3 tier berdasarkan kepentingan dan impak kegagalan.

### Struktur Tier

| Tier | Nama | Kepentingan | Kegagalan | Fail |
|---|---|---|---|---|
| **A** | Core (7) | Integriti asas | **Build BERHENTI** | `scripts/validators/tier_a_core/` |
| **B** | Feature (13) | Ketepatan ciri | **Warning** | `scripts/validators/tier_b_feature/` |
| **C** | Platform (8) | Stress/performance | **CI sahaja** | `scripts/validators/tier_c_stress/` |

### Tier A: Core (7 validators)

| Validator | Apa Diperiksa |
|---|---|
| `validate_build.rs` | Kod kompil tanpa ralat |
| `parse_valid_ldx.rs` | Parser berfungsi untuk semua contoh `.ldx` |
| `semantic_no_errors.rs` | Analisis semantik lulus untuk kod sah |
| `llvm_ir_generation.rs` | LLVM IR dihasilkan untuk setiap contoh |
| `cap_file_generation.rs` | Fail `.cap` dihasilkan |
| `topology_verify.rs` | Topology verification lulus |
| `no_panic_in_prod.rs` | Tiada `unwrap()` dalam kod production |

### Tier B: Feature (13 validators)

| Validator | Apa Diperiksa |
|---|---|
| `validate_threading.rs` | Actor spawn, channel send/recv |
| `validate_capability.rs` | Gate declaration, topology verify |
| `validate_reactor.rs` | epoll event loop, connection management |
| `validate_sharding.rs` | Shard assignment, CPU affinity |
| `validate_wasm_codegen.rs` | WASM output generation |
| `validate_host_reactor.rs` | Guest → Host dispatch |
| `validate_raylib_ffi.rs` | Raylib function registration |
| `validate_audio_context.rs` | StrictAudioContext enforcement |
| `validate_freestanding.rs` | `_start`, panic handler, allocator |
| `validate_ctl_mapper.rs` | WIT generation |
| `validate_streaming.rs` | 2-Pass engine, SemanticSummary |
| `validate_door_latency.rs` | Door latency < 100ns |
| `validate_backpressure.rs` | Block/DropOldest/Error policies |

### Tier C: Platform/Stress (8 validators)

| Validator | Apa Diperiksa |
|---|---|
| `stress_8_core.rs` | Scaling efficiency > 85% at 8 cores |
| `stress_memory_leak.rs` | RSS tidak naik selepas 1 jam |
| `stress_slowloris.rs` | Taint FSM tutup connection jahat |
| `stress_syn_flood.rs` | Backpressure bertahan |
| `stress_malformed.rs` | EPOLLERR cleanup betul |
| `cross_compile_x86_64.rs` | Kompilasi untuk x86_64 |
| `cross_compile_aarch64.rs` | Kompilasi untuk aarch64 |
| `cross_compile_riscv64.rs` | Kompilasi untuk riscv64 |

### Workflow Validasi

```bash
# Tier A — mesti lulus sebelum commit
$ cargo test --locked
   Compiling logicodex v1.45.0-alpha
   Running 7 tests
   test validate_build ... ok
   test parse_valid_ldx ... ok
   test semantic_no_errors ... ok
   test llvm_ir_generation ... ok
   test cap_file_generation ... ok
   test topology_verify ... ok
   test no_panic_in_prod ... ok
   
   test result: ok. 7 passed; 0 failed

# Tier B — warning jika gagal, tetapi build diteruskan
$ python3 scripts/validators/tier_b_feature/*.py
   Running 13 tests
   ... (13 passed)

# Tier C — hanya di CI
$ python3 scripts/validators/tier_c_stress/*.py
   Running 8 tests
   ... (8 passed)
```

---

## Benchmark Framework: 4 Layer {#benchmark}

### Definisi

Framework benchmark 4-layer memberikan ukuran kuantitatif untuk setiap komponen kritikal Logicodex. Setiap layer menangani satu aspek prestasi.

### Layer 1: Micro-Benchmarks (Criterion)

| Benchmark | Apa Diukur | Target | Fail |
|---|---|---|---|
| `gate_latency.rs` | Capability gate check | < 50ns | `benches/micro/` |
| `door_latency.rs` | Channel send/recv/roundtrip | < 100ns | `benches/micro/` |
| `mempool_latency.rs` | Bump allocator acquire | < 20ns | `benches/micro/` |
| `callable_lookup.rs` | CallableRegistry by-name lookup | < 30ns | `benches/micro/` |
| `hir_lower.rs` | AST → HIR lowering | < 200ns | `benches/micro/` |
| `llvm_emit.rs` | LLVM IR generation | < 500ns | `benches/micro/` |

### Layer 2: Reactor Throughput

| Benchmark | Apa Diukur | Target | Fail |
|---|---|---|---|
| `echo_server.rs` | epoll-based TCP echo | Throughput maksimum | `benches/reactor/` |
| `flood_client.rs` | Multi-threaded flood | Beban maksimum | `benches/reactor/` |
| `throughput.sh` | 1/2/4/8 core scaling | Efficiency > 85% | `benches/reactor/` |

### Layer 3: System Stability

| Benchmark | Apa Diukur | Target | Fail |
|---|---|---|---|
| `rss_monitor.py` | /proc/[pid]/status snapshot | Slope ≤ 0.001 KB/h | `benches/stability/` |
| `valgrind_check.sh` | Full leak check | 0 kebocoran | `benches/stability/` |
| `longrun.sh` | 1h/6h/24h automated | Tiada crash | `benches/stability/` |

### Layer 4: Security Stress

| Benchmark | Apa Diukur | Target | Fail |
|---|---|---|---|
| `slowloris.py` | Partial read attack | Taint FSM detect | `benches/security/` |
| `syn_flood.py` | Connection flood | Backpressure hold | `benches/security/` |
| `malformed.py` | Random byte injection | EPOLLERR cleanup | `benches/security/` |
| `fd_exhaustion.py` | EMFILE boundary | Graceful degradation | `benches/security/` |

### BASELINE.json: Gold Standard

```json
{
  "version": "1.45.0",
  "generated": "2026-05-25T00:00:00Z",
  "thresholds": {
    "warn_regression_pct": 5.0,
    "fail_regression_pct": 10.0
  },
  "micro": {
    "gate_latency_ns": { "value": 45, "unit": "ns" },
    "door_latency_ns": { "value": 85, "unit": "ns" },
    "mempool_latency_ns": { "value": 15, "unit": "ns" },
    "callable_lookup_ns": { "value": 25, "unit": "ns" },
    "hir_lower_ns": { "value": 180, "unit": "ns" },
    "llvm_emit_ns": { "value": 450, "unit": "ns" }
  },
  "reactor": {
    "echo_throughput_mbps": { "value": 850, "unit": "MB/s" },
    "scaling_efficiency_8c": { "value": 87.5, "unit": "%" }
  },
  "stability": {
    "rss_slope_kbh": { "value": 0.0005, "unit": "KB/h" },
    "valgrind_leaks": { "value": 0, "unit": "count" },
    "longrun_24h_status": { "value": "pass", "unit": "enum" }
  },
  "security": {
    "slowloris_detected": { "value": true, "unit": "bool" },
    "syn_flood_survived": { "value": true, "unit": "bool" }
  }
}
```

### Regression Detection

```python
# compare_baseline.py
import json
import sys

def compare(baseline_path, current_path):
    with open(baseline_path) as f:
        baseline = json.load(f)
    with open(current_path) as f:
        current = json.load(f)
    
    failures = []
    
    for category in ["micro", "reactor"]:
        for metric, base_val in baseline[category].items():
            curr_val = current[category][metric]
            if curr_val["value"] > base_val["value"] * 1.10:
                failures.append(f"FAIL: {metric} regressed {curr_val['value'] / base_val['value']:.1%}")
            elif curr_val["value"] > base_val["value"] * 1.05:
                failures.append(f"WARN: {metric} regressed {curr_val['value'] / base_val['value']:.1%}")
    
    return failures
```

---

## Commit & Push: Pengurusan Versi {#version}

### Strategi Versioning

Logicodex menggunakan Semantic Versioning (SemVer) dengan label `-alpha`:

```text
v{MAJOR}.{MINOR}.{PATCH}-alpha

MAJOR: Perubahan arkitektur besar (contoh: v1 → v2)
MINOR: Fitur baru atau milestone (contoh: v1.21 → v1.30)
PATCH: Bug fix atau polish (contoh: v1.44 → v1.44.1)
-alpha: Status development (akan menjadi -beta, kemudian release)
```

### Garis Masa Versi

```text
v1.21.0-alpha ──► v1.30.0-alpha ──► v1.31.0-alpha ──► v1.32.0-alpha
  Compiler           Threading         Streaming          Capability
  Core               + IO + Audio      Engine             Security

──► v1.33.0-alpha ──► v1.34.0-alpha ──► v1.35.0-alpha ──► v1.36.0-alpha
     Network           Sharded            CapabilityGraph    CTL
     Reactor           Reactor            IR                 Mapper

──► v1.37.0-alpha ──► v1.38.0-alpha ──► v1.39.0-alpha ──► v1.40.0-alpha
     Network           Deferred           Sharded            WASM
     Runtime           Cleanup            Runtime            Backend

──► v1.41.0-alpha ──► v1.42.0-alpha ──► v1.43.0-alpha ──► v1.44.0-alpha
     Host Reactor      Raylib FFI         Raylib Audio       Freestanding
                       (8 items)          (22 functions)     Compiler

──► v1.44.1-alpha ──► v1.45.0-alpha ──► v1.46.0-alpha ──► v2.00.0-alpha
     Maintenance       Benchmark          Streaming WASM     Pointer
     (Validator)       Framework          + WASI             Provenance
```

### Polisi Commit

| Peraturan | Implementasi |
|---|---|
| **Tiada commit tanpa lulus Tier A** | Pre-commit hook menjalankan `cargo test --locked` + `scripts/validators/tier_a_core/*.py` |
| **Tier B mesti lulus sebelum merge** | PR gate menjalankan `scripts/validators/tier_b_feature/*.py` |
| **Tier C hanya di CI** | GitHub Actions menjalankan `scripts/validators/tier_c_stress/*.py` |
| **Benchmark sebelum release** | Setiap minor release mesti lulus semua 4 layer benchmark |
| **Update BASELINE.json** | Jika benchmark meningkat, kemas kini BASELINE.json |
| **CHANGELOG.md** | Setiap commit mesti ada catatan dalam CHANGELOG |

---

## Ringkasan Governance

| Mekanisme | Apa | Mengapa |
|---|---|---|
| **Architecture Freeze** | Tiada fitur baharu tanpa RFC | Menjaga integriti arkitektur |
| **RFC Template** | 4 mandatory alignment checks | Memastikan cadangan selaras dengan prinsip |
| **Validator Tiering** | A/B/C — berbeza impak kegagalan | Sumber diarahkan ke perkara paling penting |
| **Benchmark Framework** | 4 layer — micro ke security | Ukur prestasi secara kuantitatif |
| **BASELINE.json** | Gold standard dengan thresholds | Regression detection automatik |
| **Semantic Versioning** | v{major}.{minor}.{patch}-alpha | Komunikasi status dengan jelas |

Kesemua mekanisme ini bekerja bersama: RFC memastikan perubahan berfikir, validator memastikan perubahan betul, benchmark memastikan perubahan tidak merosakkan prestasi, dan versioning memberikan konteks kepada pengguna.
