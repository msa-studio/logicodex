> ⚠️ **NOT UPDATED — will revisit.** This document predates the current syntax/architecture and may contain stale information. Authoritative current references: `README.md`, `examples/`, and `docs/architecture/`. Tracked under `docs/DOCUMENTATION_POLICY.md`.

# Logicodex Functions And Guide

## Panduan Lengkap Fungsi dan Penggunaan

**Version:** v1.45.0-alpha  
**Date:** May 2026  
**Author:** Mohamad Supardi Abdul (`mymsastudio@gmail.com`)

---

> *"Dari program pertama anda sehingga aplikasi produksi — setiap fungsi, setiap parameter, setiap contoh."*

---

## Apa yang Ada dalam Panduan Ini

Panduan ini adalah **manual praktikal** untuk pengguna Logicodex — dari pemula yang menulis program pertama sehingga jurutera sistem yang membina aplikasi produksi. Ia merangkumi:

1. **Permulaan** — Pemasangan, program pertama, dan memahami ralat
2. **Asas Bahasa** — Sintaks, tipe data, pengawalan aliran, fungsi
3. **Sistem Capability** — Gate, door, service, dan keamanan
4. **Concurrency** — Actor, channel, shard, dan reactor
5. **Grafik dan Audio** — Semua 54 fungsi Raylib dan 22 fungsi audio
6. **Kompilasi dan Penyebaran** — Native, WASM, dan freestanding
7. **Rujukan** — Pustaka standard, resepi, dan penyelesaian masalah

> **Untuk pembaca:** Jika anda ingin memahami *mengapa* Logicodex direka begitu rupa (falsafah, justifikasi arkitektur), rujuklah wiki **"White Paper: Compiler Philosophy and Architecture."** Panduan ini fokus pada *cara* menggunakan Logicodex.

---

```text
============================================================
  _                 _               _                 
 | |    ___   __ _ (_)  ___  ___   __| |  ___ __  __  
 | |   / _ \ / _` || | / __|/ _ \ / _` | / _ \ \/ /  
 | |__| (_) | (_| || || (__| (_) | (_| ||  __/ >  <   
 |_____\___/ \__, ||_| \___|\___/ \__,_| \___|/_/\_\  
             |___/                                    
============================================================
        Functions And Guide — v1.45.0-alpha
============================================================
```
