# Summary

[Logicodex Functions And Guide](./title.md)

---

# Permulaan

- [Pemasangan dan Konfigurasi](./ch01-installasi.md)
  - [Keperluan Sistem](./ch01-installasi.md#keperluan)
  - [Memasang dari Sumber](./ch01-installasi.md#sumber)
  - [Struktur Projek](./ch01-installasi.md#struktur)
- [Program Pertama Anda](./ch02-hello.md)
  - [Hello World dalam 3 Gaya](./ch02-hello.md#hello)
  - [Kompilasi dan Jalankan](./ch02-hello.md#kompilasi)
  - [Memahami Ralat Compiler](./ch02-hello.md#ralat)

# Asas Bahasa

- [Sintaks dan Token](./ch03-sintaks.md)
  - [Alias Melayu vs Canonical](./ch03-sintaks.md#alias)
  - [Tatasusila (Grammar) Asas](./ch03-sintaks.md#grammar)
  - [Komen dan Dokumentasi](./ch03-sintaks.md#komen)
- [Tipe Data](./ch04-tipe.md)
  - [Tipe Asas (I32, F64, Bool, Text)](./ch04-tipe.md#asas)
  - [Tipe Komposit (Struct, Enum, Array)](./ch04-tipe.md#komposit)
  - [Tipe Pointer (PTR, Ref)](./ch04-tipe.md#pointer)
  - [Tipe Khas (Result, Option, Channel)](./ch04-tipe.md#khas)
- [Pengawalan Aliran](./ch05-aliran.md)
  - [If / Else / Else If](./ch05-aliran.md#if)
  - [Match (Pattern Matching)](./ch05-aliran.md#match)
  - [For Loop dan While](./ch05-aliran.md#loop)
  - [Break, Continue, Return](./ch05-aliran.md#control)
- [Fungsi dan Skop](./ch06-fungsi.md)
  - [Mendefinisikan Fungsi](./ch06-fungsi.md#define)
  - [Parameter dan Return Type](./ch06-fungsi.md#parameter)
  - [Skop dan Lifetime Variable](./ch06-fungsi.md#skop)
  - [Closure dan Callback](./ch06-fungsi.md#closure)

# Sistem Capability

- [Gate: Keizinan Akses](./ch07-gate.md)
  - [Mengisytiharkan Gate](./ch07-gate.md#isytihar)
  - [3 Jenis Gate (DirectCall, Message, Hardware)](./ch07-gate.md#jenis)
  - [Service Manifest](./ch07-gate.md#service)
- [Door: Saluran Data](./ch08-door.md)
  - [Membuat Channel](./ch08-door.md#channel)
  - [Send dan Receive](./ch08-door.md#sendrecv)
  - [Backpressure Policy](./ch08-door.md#backpressure)
- [Keamanan dan Audit](./ch09-security.md)
  - [Fail `.cap` dan Supply-Chain](./ch09-security.md#cap)
  - [Privilege Escalation Detection](./ch09-security.md#privilege)
  - [Amalan Terbaik Keamanan](./ch09-security.md#amalan)

# Concurrency

- [Actor: Unit Berkomputasi](./ch10-actor.md)
  - [Mendefinisikan Actor](./ch10-actor.md#define)
  - [Spawn dan Lifecycle](./ch10-actor.md#spawn)
  - [Komunikasi Antara Actor](./ch10-actor.md#komunikasi)
- [Shard dan Penjadualan](./ch11-shard.md)
  - [Konsep Shard dan Core](./ch11-shard.md#konsep)
  - [Topologi Statik](./ch11-shard.md#topologi)
  - [Memory Budgeting](./ch11-shard.md#memory)
- [Reactor: Event Loop](./ch12-reactor.md)
  - [Service dan Port Binding](./ch12-reactor.md#service)
  - [Connection dan Taint FSM](./ch12-reactor.md#connection)
  - [Mengendalikan Serangan Rangkaian](./ch12-reactor.md#serangan)

# Grafik dan Audio

- [Raylib FFI: Semua Fungsi](./ch13-raylib.md)
  - [Inisialisasi dan Window](./ch13-raylib.md#inisialisasi)
  - [Menggambar (Draw Functions)](./ch13-raylib.md#draw)
  - [Input (Keyboard, Mouse)](./ch13-raylib.md#input)
  - [Texture dan Math](./ch13-raylib.md#texture)
- [Audio Programming](./ch14-audio.md)
  - [Audio Device (Init, Close, Volume)](./ch14-audio.md#device)
  - [Sound (Load, Play, Stop)](./ch14-audio.md#sound)
  - [Music (Stream, Seek, Volume)](./ch14-audio.md#music)
  - [Audio Stream dan Callback](./ch14-audio.md#stream)
  - [StrictAudioContext: 4 Peraturan](./ch14-audio.md#strict)

# Kompilasi dan Penyebaran

- [Target Kompilasi](./ch15-target.md)
  - [Native (ELF)](./ch15-target.md#native)
  - [WebAssembly (WASM)](./ch15-target.md#wasm)
  - [Freestanding (Bare Metal)](./ch15-target.md#freestanding)
- [Proses Build Lengkap](./ch16-build.md)
  - [Build Script dan Dependensi](./ch16-build.md#script)
  - [Kompilasi Cross-Platform](./ch16-build.md#cross)
  - [Optimisasi dan Debug](./ch16-build.md#optimisasi)

# Rujukan

- [Pustaka Standard](./ch17-stdlib.md)
  - [Modul `core` (Memori, Sync)](./ch17-stdlib.md#core)
  - [Modul `std` (Audio, File, Net)](./ch17-stdlib.md#std)
  - [Modul `ffi` (Raylib, C ABI)](./ch17-stdlib.md#ffi)
- [Resepi dan Contoh](./ch18-resepi.md)
  - [HTTP Server Sederhana](./ch18-resepi.md#http)
  - [Aplikasi Grafik Interaktif](./ch18-resepi.md#grafik)
  - [Pemain Audio](./ch18-resepi.md#audio-player)
  - [Aplikasi Bare Metal (Freestanding)](./ch18-resepi.md#baremetal)
- [Penyelesaian Masalah](./ch19-troubleshoot.md)
  - [Ralat Kompilasi Umum](./ch19-troubleshoot.md#kompilasi)
  - [Masalah Runtime](./ch19-troubleshoot.md#runtime)
  - [Masalah FFI dan Raylib](./ch19-troubleshoot.md#ffi)
  - [Masalah WASM](./ch19-troubleshoot.md#wasm)

---

[Jadual Fungsi Lengkap Raylib](./raylib-functions.md)
[Jadual Fungsi Audio Lengkap](./audio-functions.md)
