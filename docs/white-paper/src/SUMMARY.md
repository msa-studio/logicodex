# Summary

[White Paper: Compiler Philosophy and Architecture](./title.md)

---

# Falsafah

- [Kenapa Logicodex Wujud?](./ch01-kenapa.md)
  - [Krisis Polarisasi dalam Pengaturcaraan Sistem](./ch01-kenapa.md#krisis-polarisasi)
  - [Jalan Ketiga: Syntax Untuk Manusia, Semantik Untuk Mesin](./ch01-kenapa.md#jalan-ketiga)
  - [Paradigma Ko-Eksplorasi Manusia-AI](./ch01-kenapa.md#koeksplorasi)
- [Falsafah Rekabentuk Utama](./ch02-falsafah.md)
  - [Prinsip 1: Determinisme Absolute](./ch02-falsafah.md#determinisme)
  - [Prinsip 2: Zero Runtime Mediation](./ch02-falsafah.md#zero-runtime)
  - [Prinsip 3: Pendedahan Kompleksiti Secara Progresif](./ch02-falsafah.md#progresif)
  - [Prinsip 4: Capability-Based Security](./ch02-falsafah.md#capability)
  - [Prinsip 5: Alias-to-Canonical Lexing](./ch02-falsafah.md#alias)

# Evolusi Arkitektur

- [Dari Compiler Core ke Sistem Platform: Sejarah v1.21 hingga v1.45](./ch03-evolusi.md)
  - [Fasa 1: Compiler Core v1.21 (Asas)](./ch03-evolusi.md#fasa1)
  - [Fasa 2: Threading + IO + Audio v1.30 (Demo Threading)](./ch03-evolusi.md#fasa2)
  - [Fasa 3: Capability Security v1.32 (Pintu Keselamatan)](./ch03-evolusi.md#fasa3)
  - [Fasa 4: Network Reactor v1.33-v1.37 (I/O Deterministik)](./ch03-evolusi.md#fasa4)
  - [Fasa 5: Sharded Runtime v1.34-v1.39 (Multi-Core)](./ch03-evolusi.md#fasa5)
  - [Fasa 6: WASM + Host Reactor v1.40-v1.41 (Sandbox)](./ch03-evolusi.md#fasa6)
  - [Fasa 7: Raylib FFI + Audio v1.42-v1.43 (Grafik & Audio)](./ch03-evolusi.md#fasa7)
  - [Fasa 8: Freestanding Compiler v1.44 (Bare Metal)](./ch03-evolusi.md#fasa8)
  - [Fasa 9: Stabilisasi v1.44.1-v1.45 (Maintenance & Benchmark)](./ch03-evolusi.md#fasa9)
- [Refleksi: Justifikasi Keputusan Arkitektur Utama](./ch04-justifikasi.md)
  - [Kenapa Actor Model + Channel, Bukan Thread Biasa?](./ch04-justifikasi.md#actor)
  - [Kenapa Zero-Copy Ownership Transfer?](./ch04-justifikasi.md#zerocopy)
  - [Kenapa Compile-Time Capability, Bukan Runtime Check?](./ch04-justifikasi.md#compiletime)
  - [Kenapa Taint FSM untuk Network?](./ch04-justifikasi.md#taint)
  - [Kenapa CapabilityGraph IR sebagai "Single Source of Truth"?](./ch04-justifikasi.md#ir)
  - [Kenapa CTL Mapper "Project INTO, Not Borrow FROM"?](./ch04-justifikasi.md#ctl)
  - [Kenapa 3 Backend Target Serentak?](./ch04-justifikasi.md#backend)
  - [Kenapa StrictAudioContext untuk Audio?](./ch04-justifikasi.md#audio)
  - [Kenapa Architecture Freeze + RFC Process?](./ch04-justifikasi.md#freeze)

# Model Teknikal

- [Model Concurrency: Actor + Channel + Shard + Reactor](./ch05-concurrency.md)
  - [Actor: Unit Komputasi Terpencil](./ch05-concurrency.md#actor)
  - [Channel: SPSC Ring Buffer Zero-Copy](./ch05-concurrency.md#channel)
  - [Door: Cross-Shard Transport](./ch05-concurrency.md#door)
  - [Shard: Unit Penjadualan pada Core CPU](./ch05-concurrency.md#shard)
  - [Reactor: Event Loop Deterministik](./ch05-concurrency.md#reactor)
- [Model Keselamatan: Capability + Gate + Door + Service](./ch06-security.md)
  - [Gate: Kontrak Keselamatan Masa Kompil](./ch06-security.md#gate)
  - [Service: Port-Based Actor + RAII Connection](./ch06-security.md#service)
  - [Taint FSM: Healthy → Suspicious → Closing](./ch06-security.md#taint)
  - [Backpressure: Block / DropOldest / Error](./ch06-security.md#backpressure)
  - [Supply-Chain Security: `.cap` File + Privilege Escalation Detection](./ch06-security.md#supply)
- [Intermediate Representations: HIR → CapabilityGraph → CTL → WIT](./ch07-ir.md)
  - [HIR: High-Level IR (v1.36)](./ch07-ir.md#hir)
  - [CapabilityGraph IR: Single Source of Truth (v1.35)](./ch07-ir.md#capgraph)
  - [CTL Mapper: Capability Translation Layer (v1.36)](./ch07-ir.md#ctl)
  - [Output: Native ELF / `.cap` / WIT dari Satu IR](./ch07-ir.md#output)

# Pengurusan & Masa Depan

- [Pengurusan Projek: Architecture Freeze, RFC, Governance](./ch08-governance.md)
  - [Architecture Freeze Policy: Apa dan Mengapa?](./ch08-governance.md#freeze)
  - [RFC Template: 4 Mandatory Alignment Checks](./ch08-governance.md#rfc)
  - [Validator Tiering: A/B/C](./ch08-governance.md#validator)
  - [Benchmark Framework: 4 Layer](./ch08-governance.md#benchmark)
  - [Commit & Push: Pengurusan Versi](./ch08-governance.md#version)
- [Perbandingan dengan Bahasa Sistem Lain](./ch09-perbandingan.md)
  - [vs C/C++: Memory Safety tanpa Runtime Cost](./ch09-perbandingan.md#cpp)
  - [vs Rust: Ownership tanpa Cognitive Overhead](./ch09-perbandingan.md#rust)
  - [vs Zig: Comptime vs Compile-Time Capability](./ch09-perbandingan.md#zig)
  - [vs Go: Concurrency Model yang Berbeza](./ch09-perbandingan.md#go)
  - [vs WASM-first Languages: Capability-Native WASM](./ch09-perbandingan.md#wasm)
- [Kesimpulan dan Masa Depan](./ch10-kesimpulan.md)
  - [Ringkasan Kejayaan v1.21–v1.45](./ch10-kesimpulan.md#ringkasan)
  - [Masa Depan: v1.46+ dan v2.00 Pointer Provenance](./ch10-kesimpulan.md#masadepan)
  - [Jemputan Sumbangan](./ch10-kesimpulan.md#sumbangan)

---

[Appendix A: Glosari Istilah](./appendix-glosari.md)
[Appendix B: Timeline Evolusi](./appendix-timeline.md)
[Appendix C: Rujukan & Bibliografi](./appendix-rujukan.md)
