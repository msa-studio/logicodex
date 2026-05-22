# Logicodex Phase 1 MVP Developer Manual

Architect & Creator: Mohamad Supardi Abdul (mymsastudio@gmail.com)

```text
=========================================================
  _                 _               _                 
 | |    ___   __ _ (_)  ___  ___   __| |  ___ __  __  
 | |   / _ \ / _` || | / __|/ _ \ / _` | / _ \\ \/ /  
 | |__| (_) | (_| || || (__| (_) | (_| ||  __/ >  <   
 |_____\___/ \__, ||_| \___|\___/ \__,_| \___|/_/\_\  
             |___/                                    
                  [ COMPILER PHASE 1 - bare-metal ]
=========================================================
```

**Repository:** `logicodex`  
**Compiler executable:** `logicodex`  
**Official source extension:** `.ldx`  
**Version:** 1.0.1-alpha
**Security Profile:** Internal Security & OS Freestanding Test

## Overview

Logicodex is a native programming language compiler implemented in Rust. The Phase 1 MVP demonstrates a dual-syntax frontend in which novice-oriented pseudocode and expert shorthand are normalized through `dict/core_map.json` into the same compiler token identities. Once lexing is complete, both source styles produce the same AST, pass through the same semantic analyzer, and are lowered to LLVM machine code.

## Build Requirements

| Dependency | Purpose |
|---|---|
| Rust and Cargo | Build the compiler executable. |
| LLVM 17 development libraries | Required by the configured `inkwell` backend feature. |
| C-compatible linker | Links generated object files and the platform runtime bridge. |

## Build and Use

```bash
cd logicodex
cargo build --release
./target/release/logicodex logo
./target/release/logicodex tokens examples/01_tambah_pemula.ldx
./target/release/logicodex check examples/01_tambah_pemula.ldx
./target/release/logicodex compile examples/01_tambah_pemula.ldx --emit-ir -o ./tambah_pemula
```

Set `LOGICODEX_LINKER` to override the linker used by the compiler.

## Runtime Bridge

The compiler lowers `PAPAR` and `print` to `logicodex_print_i64`. The Linux bridge writes through native syscall-oriented assembly, while the Windows bridge is structured around Win32 console output. This keeps Phase 1 free from a mandatory virtual machine or garbage collector.
