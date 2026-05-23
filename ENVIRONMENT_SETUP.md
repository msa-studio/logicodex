# Logicodex Compiler Success Environment Setup Guide

Dokumen ini mengandungi spesifikasi konfigurasi persekitaran (**Environment Baseline**) yang telah disahkan stabil untuk membina dan menjalankan **Enjin Kompiler Logicodex v1.21-alpha** daripada kod sumber asli Rust dan LLVM.

Untuk mengelakkan isu **Binary Linker Incompatibility**, khususnya ralat pemaut Windows native `cc` / `ld`, persekitaran pembangunan dipindahkan sepenuhnya ke dalam subsistem Linux melalui **WSL2**.

## 1. Architectural Baseline

| Komponen | Baseline Disahkan |
|---|---|
| Sistem Operasi Host | Windows 10/11 Pro |
| Subsystem Pembangunan | WSL2, Ubuntu 22.04 LTS atau versi Ubuntu terkini yang serasi |
| Seni Bina Perkakasan | x86_64 multi-core CPU |
| Memori Minimum Disarankan | 16 GB RAM |
| Memori Gred Pengeluaran Yang Telah Ditala | 64 GB RAM, dengan sela masa kompilasi mampat di bawah 60 saat pada jentera yang sesuai |
| IDE / Editor | Visual Studio Code di Windows, dipautkan melalui ekstensi rasmi WSL / Remote - WSL |

## 2. Rantaian Alatan Sistem Luaran

LLVM 15 MSVC runtime pada Windows boleh menimbulkan isu fail `.lib` dan utiliti `llvm-config.exe` yang tidak konsisten. Oleh itu, pemasangan pakej pembangunan tegar hendaklah menggunakan pengurus pakej `apt` Linux di dalam terminal Ubuntu WSL2.

Jalankan arahan berikut di dalam terminal Ubuntu WSL2:

```bash
sudo apt update
sudo apt install build-essential clang llvm-15 llvm-15-dev libpolly-15-dev libzstd-dev libxml2-dev git -y
```

## 3. Nota Penggunaan

Persekitaran ini bertujuan menjadi baseline praktikal untuk **current logicodex v 1.21 alpha**. Semua operasi binaan, semakan Rust, dan integrasi LLVM hendaklah dijalankan dari shell Ubuntu WSL2, bukan daripada Windows native shell, bagi mengurangkan risiko ketidakserasian linker dan registry toolchain.

Selepas dependencies sistem dipasang, buka direktori repo Logicodex melalui VS Code Remote WSL supaya terminal, extension, dan proses build menggunakan konteks Linux yang sama.
