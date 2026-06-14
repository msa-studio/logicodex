#!/usr/bin/env python3
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
WHITE = ROOT / "WHITE_PAPER.md"
README = ROOT / "README.md"
MANUAL = ROOT / "MANUAL.md"

logo = """```text
=========================================================
  _                 _               _                 
 | |    ___   __ _ (_)  ___  ___   __| |  ___ __  __  
 | |   / _ \\ / _` || | / __|/ _ \\ / _` | / _ \\ \\/ /  
 | |__| (_) | (_| || || (__| (_) | (_| ||  __/ >  <   
 |_____\\___/ \\__, ||_| \\___|\\___/ \\__,_| \\___|/_/\\_\\  
             |___/                                    
             [ LOGICODEX COMPILER v1.0.1-alpha ]
             [ DUAL-SYNTAX LLVM SYSTEMS LANGUAGE ]
=========================================================
Architect & Creator: Mohamad Supardi Abdul (mymsastudio@gmail.com)
```"""

diagram = """```text
[ Novice Code (.ldx) ] ──► (Lexer + core_map.json) ──► [ Unified Token Stream ]
                                                              │
[ Expert Code (.ldx) ] ──► (Lexer + core_map.json) ──► [ Abstract Syntax Tree ]
                                                              │
[ Native Binary ] ◄── (LLVM Backend Optimization O3) ◄── [ LLVM IR Generation ]
```"""

white = WHITE.read_text()
white = white.replace("Ckt0", "crt0")
white = white.replace("## 4. Architectural Objectives", "## 4. Compiler Frontend and Architecture")
needle = "Logicodex is organized as a deterministic ahead-of-time compiler pipeline with a deliberately flexible frontend. The pipeline begins with official `.ldx` source files and a dynamic dictionary. It then performs lexing, parsing, semantic analysis, LLVM-oriented IR generation, object emission, and platform-specific linking or freestanding object generation."
replacement = needle + "\n\nThe following diagram summarizes the core dual-syntax compiler pipeline. Both novice and expert `.ldx` inputs enter the same dictionary-aware lexer, collapse into a unified token stream and AST, and then lower through LLVM IR generation toward optimized native binaries.\n\n" + diagram
if diagram not in white:
    white = white.replace(needle, replacement)

freestanding_needle = "In this profile, the backend emits an object intended for later integration by a bootloader, kernel linker script, hypervisor build, or firmware image generator. The compiler does not claim to provide a complete bootable image at this stage. It provides the **layout framework** required for operating-system development: target selection, entry-symbol control, runtime bypass, and physical-memory access documentation."
freestanding_add = """In this profile, the backend emits an object intended for later integration by a bootloader, kernel linker script, hypervisor build, or firmware image generator. The compiler does not claim to provide a complete bootable image at this stage. It provides the **layout framework** required for operating-system development: target selection, entry-symbol control, runtime bypass, and physical-memory access documentation.

A concrete freestanding example is the classic VGA text buffer write at physical address `0xB8000`. The example below writes raw ASCII character bytes and attribute bytes directly to screen memory. It is intentionally documented as a freestanding, capability-gated operation rather than ordinary hosted application behavior.

**Novice pseudocode variant:**

```logicodex
MULA PROGRAM tulis_vga

GUNA JENIS U16
GUNA JENIS PTR<U16>

TANDA KAWASAN_PERKAKAS VGA_TEXT SEBAGAI PTR<U16> = ALAMAT 0xB8000

FUNGSI mula_sistem() -> I32
MULA
    # 0x074C = ASCII 'L' with light-gray-on-black text attribute 0x07.
    TULIS_VOLATIL(VGA_TEXT + 0, 0x074C)
    TULIS_VOLATIL(VGA_TEXT + 1, 0x076F)
    TULIS_VOLATIL(VGA_TEXT + 2, 0x0767)
    TULIS_VOLATIL(VGA_TEXT + 3, 0x0769)
    TULIS_VOLATIL(VGA_TEXT + 4, 0x0763)
    PULANG 0
TAMAT

TAMAT PROGRAM
```

**Expert shorthand variant:**

```logicodex
program vga_write {
    use U16;
    use PTR<U16>;

    hw VGA_TEXT: PTR<U16> = addr 0xB8000;

    fn _start() -> I32 {
        // 0x074C = ASCII 'L' with light-gray-on-black text attribute 0x07.
        vstore(VGA_TEXT + 0, 0x074C);
        vstore(VGA_TEXT + 1, 0x076F);
        vstore(VGA_TEXT + 2, 0x0767);
        vstore(VGA_TEXT + 3, 0x0769);
        vstore(VGA_TEXT + 4, 0x0763);
        return 0;
    }
}
```

The novice and expert forms compile toward the same conceptual volatile stores. On x86 text-mode targets, each `U16` cell combines an ASCII byte and a color attribute byte, while other target families would bind equivalent display or serial-output hardware through target-specific capability declarations."""
if "## 11. Physical Memory Mapping" in white and "**Novice pseudocode variant:**" not in white:
    white = white.replace(freestanding_needle, freestanding_add)

security_needle = "The most important v1.0.1-alpha security addition is the **Runtime Memory Integrity Verification Engine**. The engine is defined as an active self-attestation loop that protects the executable `.text` segment after program launch. The compiler-side contract is straightforward: produce a compile-time digest of immutable executable code, store it as a protected Golden Hash, and schedule a runtime verifier that continuously or periodically recomputes the digest from live memory."
security_replacement = security_needle + "\n\n**Technical note:** The **Runtime Memory Integrity Verification Engine (SHA/AES-NI Continuous Attestation Loop)** is presented in v1.0.1-alpha as an **architectural design specification for the milestone**, with freestanding hardware intrinsic bindings treated as a later engineering objective. The current alpha documentation defines the compiler contract, threat model, data-flow invariant, and mitigation semantics; it does not claim that all hardware-specific secure runtime bindings are complete."
if "SHA/AES-NI Continuous Attestation Loop" not in white:
    white = white.replace(security_needle, security_replacement)

WHITE.write_text(white)

readme = f"""# ❖ Logicodex Language — v1.0.1-alpha (MVP Upgrade)
{logo}

### 💡 Executive Summary

Logicodex is a next-generation, statically-typed system programming language backed by the LLVM compiler infrastructure. It is designed to natively eliminate the cognitive friction between human intent and machine execution efficiency.

By utilizing a context-aware frontend parser, Logicodex allows novice developers to write structured, human-readable pseudocode, while empowering staff systems engineers to write hyper-dense shorthand system code within the exact same environment. Both syntaxes resolve into an identical Abstract Syntax Tree (AST) and compile directly down to native-oriented machine code targets (Windows PE / Linux ELF) without a mandatory virtual machine or garbage collector.

### 📝 Preface: The AI Co-Exploration Model

Architected and idealized by Mohamad Supardi Abdul (mymsastudio@gmail.com), Logicodex was developed via a unique co-exploration paradigm pairing human systems design with Advanced Artificial Intelligence (AI). This allowed us to synthetically stress-test grammatical rules, isolate abstraction leaks, and build a hardened compiler frontend gred-produksi.

### 🚀 Core Architectural Highlights

- **Context-Aware Dynamic Dictionary:** Driven by `dict/core_map.json`, mapping dual syntaxes seamlessly.
- **Active Self-Defense Security:** Architectural blueprint for real-time memory integrity attestation, defending against process injections and fileless malware.
- **Freestanding Target Support (`--target freestanding`):** Bypasses standard OS runtime lifecycles (eliminating crt0 dependencies) to allow direct kernel and firmware programming.
- **Aggressive Hardware Optimization:** Maps directly to optimized LLVM IR at the `O3` execution tier.

### 🛠️ Verification & Compilation Pipeline

{diagram}

### 🌍 Open-Source Open Governance & Licensing

Distributed under the permissive dual-licensing of **MIT License** and **Apache License 2.0**.

Copyright (c) 2026 Mohamad Supardi Abdul. All Rights Reserved.

*The name 'Logicodex', 'Logicodex Language', and its official branding assets are protected trademarks to preserve standardization and quality ecosystem controls.*

### 🤝 Call to Collaboration

We invite the brightest brains in computer science, system utility design, and LLVM optimization to join MSA Studio in accelerating Phase 2. Contact us at mymsastudio@gmail.com.
"""
README.write_text(readme)

manual = MANUAL.read_text()
manual = manual.replace("Ckt0", "crt0")
if "## Compiler Frontend and Architecture" not in manual:
    manual = manual.replace("## Overview\n\n", "## Overview\n\n")
    insert_after = "Logicodex is a native programming language compiler implemented in Rust. The Phase 1 MVP demonstrates a dual-syntax frontend in which novice-oriented pseudocode and expert shorthand are normalized through `dict/core_map.json` into the same compiler token identities. Once lexing is complete, both source styles produce the same AST, pass through the same semantic analyzer, and are lowered to LLVM machine code."
    manual = manual.replace(insert_after, insert_after + "\n\n## Compiler Frontend and Architecture\n\n" + diagram)
MANUAL.write_text(manual)

print("Applied v1.0.1-alpha documentation refinement to README.md, WHITE_PAPER.md, and MANUAL.md")
