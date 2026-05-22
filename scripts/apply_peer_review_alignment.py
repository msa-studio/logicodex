#!/usr/bin/env python3
"""Apply elite peer-review alignment updates for Logicodex v1.0.1-alpha."""
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
WHITE = ROOT / "WHITE_PAPER.md"
README = ROOT / "README.md"
MANUAL = ROOT / "MANUAL.md"
LEXER = ROOT / "src" / "lexer.rs"
MAIN = ROOT / "src" / "main.rs"
CODEGEN = ROOT / "src" / "codegen.rs"

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

# --- WHITE PAPER ---
white = WHITE.read_text()
white = white.replace(
    "**Document Status:** Consolidated academic-grade alpha architecture, security, interoperability, and governance paper.",
    "**Document Status:** Architectural Manifesto and Engineering Roadmap Specification."
)
white = white.replace("Ckt0", "crt0")

abstract_scope = (
    "\n\n**Scope clarification:** In v1.0.1-alpha, **Phase 1** delivers the verified working core compiler infrastructure: "
    "the dictionary-aware lexer, recursive-descent parser, AST construction, semantic analyzer, and LLVM-Inkwell backend path. "
    "The **WebAssembly Target**, **Logicodex Migrator Engine**, and **Continuous Runtime Memory Attestation** are formally defined architectural roadmap capabilities for **Phase 2/3**, not claims of production-complete implementation in the Phase 1 compiler."
)
abstract_anchor = "The v1.0.1-alpha consolidation also extends the language beyond a simple Phase 1 compiler demonstration. It incorporates architectural planning for runtime memory self-attestation, active self-defense against executable-code tampering, freestanding target mode, raw physical memory access controls, cross-language migration, and open-source governance. The result is a language model intended for **education, AI-assisted generation, legacy migration, secure native tools, firmware, kernels, hypervisors, and high-assurance bare-metal computation**."
if abstract_scope.strip() not in white:
    white = white.replace(abstract_anchor, abstract_anchor + abstract_scope)

intro_scope = (
    "\n\nFor status precision, this introduction separates implemented Phase 1 infrastructure from roadmap architecture. "
    "The current compiler tree demonstrates the verified core path: `dict/core_map.json` loading, tokenization, parsing, AST construction, semantic analysis, and LLVM-Inkwell-oriented backend generation. "
    "The WebAssembly Target, Logicodex Migrator Engine, and Continuous Runtime Memory Attestation remain explicit Phase 2/3 roadmap specifications so contributors can design toward them without assuming that the alpha compiler already implements their full production semantics."
)
intro_anchor = "This consolidated paper standardizes the project identity as **Logicodex** and merges the earlier Logica architectural material with the Logicodex Phase 1, v1.0.0, v1.0.0-alpha, and v1.0.1-alpha security-oriented documentation. All future official white-paper references should use the **Logicodex** name."
if intro_scope.strip() not in white:
    white = white.replace(intro_anchor, intro_anchor + intro_scope)

lexer_clarification = (
    "\n\n**Lexer/parser boundary clarification:** `core_map.json` is utilized strictly by the **Lexer** during tokenization. "
    "A source character such as `{` and a source word such as `MULA` are matched as text lexemes before the Parser is invoked, then emitted as the same canonical `TokenKind::Start` primitive. "
    "This is token-level normalization, not a grammatical macro rewrite and not parser-side syntax desugaring. The Parser consumes only the normalized token stream and therefore never needs to know whether a block began through novice, English-verbose, or expert shorthand spelling."
)
lexer_anchor = "This design supports localization, education, AI-assisted generation, domain-specific vocabulary, and professional shorthand without fragmenting the language. The dictionary is not a macro processor. Macro systems rewrite text, while Logicodex's dictionary maps human-facing expressions into compiler-facing primitives before parsing. Once lexing finishes, the parser receives canonical token identities, and the semantic analyzer does not need to know whether the user wrote a novice-oriented word or an expert symbol."
if lexer_clarification.strip() not in white:
    white = white.replace(lexer_anchor, lexer_anchor + lexer_clarification)

vga_warning = (
    "\n\n> **Engineering warning:** This memory-mapped I/O operation is strictly valid under **Freestanding (OS-less)** execution targets. "
    "When running under hosted operating systems such as Linux or Windows with virtual memory paging and ASLR active, direct physical address manipulation without kernel-space memory mapping, such as `/dev/mem` access or Ring-0 driver mediation, will be rejected by the operating system page-fault defense."
)
vga_anchor = "A concrete freestanding example is the classic VGA text buffer write at physical address `0xB8000`. The example below writes raw ASCII character bytes and attribute bytes directly to screen memory. It is intentionally documented as a freestanding, capability-gated operation rather than ordinary hosted application behavior."
if vga_warning.strip() not in white:
    white = white.replace(vga_anchor, vga_anchor + vga_warning)

white = white.replace(
    "> **Security invariant:** A Logicodex binary compiled under the secure profile should treat modification of its executable `.text` segment as a catastrophic integrity failure and respond with immediate panic, sensitive-register clearing, and hard process self-destruction.",
    "> **Security invariant:** A Logicodex binary compiled under the secure profile should treat modification of its executable `.text` segment as a catastrophic integrity failure and respond with immediate panic, sensitive-register clearing, and target-appropriate hard self-destruction."
)
white = white.replace(
    "| Hard self-destruction | Abort the process or freestanding execution context. | Isolate the threat and preserve the integrity boundary. |",
    "| Hard self-destruction | Abort the hosted process or terminate the freestanding execution context through architecture-specific reset or halt behavior. | Isolate the threat and preserve the integrity boundary. |"
)
self_defense_note = (
    "\n\nLogicodex distinguishes mitigation behavior by compilation target. In **hosted environments** such as Windows and Linux, Hard Self-Destruction means immediate process termination through native operating-system abort signals or equivalent fail-stop termination. "
    "In **freestanding environments** such as operating-system kernels, firmware, hypervisors, and bare-metal targets, there may be no host process to abort. In those contexts, Hard Self-Destruction means intentionally forcing a CPU Triple Fault where appropriate, entering an assembly `hlt` halt loop, or invoking a hardware watchdog system reset. "
    "The compiler specification therefore defines the invariant as fail-stop behavior, while the backend and runtime bridge choose the concrete mechanism according to target capabilities."
)
self_defense_anchor = "This model is intentionally strict because the threat signal is direct code-integrity failure. In ordinary software, a crash is undesirable. In high-assurance runtime defense, continuing after verified code tampering may be worse than termination. The semantic layer contributes to safety by enforcing identifier and structural correctness today, while the roadmap extends it toward bounds-aware memory access, restricted raw pointer capabilities, deterministic ownership, and RAII-style scope cleanup."
if self_defense_note.strip() not in white:
    white = white.replace(self_defense_anchor, self_defense_note + "\n\n" + self_defense_anchor)

white = white.replace(
    "| Phase 2 | Package Manager and FFI Bridges | Package registry, C ABI binding generator, platform standard libraries. | Makes Logicodex usable with real operating systems and existing libraries. |",
    "| Phase 2 | Package Manager, FFI Bridges, and Wasm Target Prototype | Package registry, C ABI binding generator, platform standard libraries, and initial WebAssembly target integration. | Makes Logicodex usable with real operating systems, existing libraries, and portable sandbox targets. |"
)
white = white.replace(
    "| Phase 3 | Local Small Language Model Integration | Compiler-assisted AI repair, intent-to-Logicodex generation, semantic feedback loop. | Turns the compiler into an AI-aware teaching and development environment. |",
    "| Phase 3 | Migrator, Continuous Attestation, and Local Small Language Model Integration | Logicodex Migrator Engine drafts, concrete runtime memory-attestation implementation, compiler-assisted AI repair, intent-to-Logicodex generation, and semantic feedback loops. | Turns the compiler into an AI-aware modernization, teaching, and high-assurance development environment. |"
)
white = white.replace(
    "| Phase 4 | Global WebAssembly Ecosystem | Wasm target, browser playground, sandboxed package execution, educational cloud. | Brings Logicodex to web-native learning and portable deployment. |",
    "| Phase 4 | Global WebAssembly Ecosystem | Browser playground, sandboxed package execution, educational cloud, and mature Wasm distribution workflows. | Brings Logicodex to web-native learning and portable deployment after the Phase 2/3 target groundwork. |"
)
roadmap_note = (
    "\n\n**Specification roadmap note:** The formal definitions for the following critical items are currently under draft for the next milestone release to prevent unsafe-by-omission assumptions: the complete EBNF grammar specification; the nominal and structural type-system boundaries; and the pointer provenance plus Undefined Behavior (UB) catalog required for systems optimization. "
    "Until these documents are published, roadmap examples involving raw pointers, FFI lowering, and freestanding hardware regions should be treated as architectural contracts rather than unrestricted implementation permission."
)
roadmap_anchor = "The immediate engineering milestones are to replace plan-file generation with actual secure backend insertion, implement cryptographic digest construction at final link time, add target-specific runtime verifier stubs, define a precise raw pointer type system, introduce linker-script examples for bootable freestanding artifacts, strengthen diagnostics, and formalize deterministic resource cleanup."
if roadmap_note.strip() not in white:
    white = white.replace(roadmap_anchor, roadmap_note + "\n\n" + roadmap_anchor)

WHITE.write_text(white)

# --- README ---
readme = f"""# Logicodex Language — v1.0.1-alpha
{logo}

## Executive Summary

Logicodex is a dual-syntax, LLVM-backed systems programming language created by **Mohamad Supardi Abdul** (`mymsastudio@gmail.com`). It is designed to reduce the cognitive gap between readable human intent and native machine execution by allowing novice-oriented pseudocode and expert shorthand to compile through one deterministic compiler pipeline.

The current **Phase 1** alpha delivers the verified working core compiler infrastructure: the `dict/core_map.json` dictionary loader, Lexer, Parser, AST construction, Semantic Analyzer, and LLVM-Inkwell backend path for native-oriented output. Roadmap capabilities including the **WebAssembly Target**, **Logicodex Migrator Engine**, and **Continuous Runtime Memory Attestation** are formally defined engineering specifications for **Phase 2/3** and should not be read as production-complete Phase 1 features.

## Compiler Pipeline

{diagram}

The dictionary is consumed strictly during lexing. Surface forms such as `MULA`, `BEGIN`, and `{{` normalize into canonical token identities such as `TokenKind::Start` before parsing begins. The parser therefore consumes a uniform token stream rather than performing macro rewriting or grammar-level dialect conversion.

## Architectural Highlights

| Area | v1.0.1-alpha Status | Engineering Direction |
|---|---|---|
| Dual syntax | Implemented in the Phase 1 frontend through dictionary-aware tokenization. | Expand localized and domain-specific token families while preserving deterministic builds. |
| Static semantics | Implemented for the Phase 1 core language. | Extend toward formal EBNF, type-system boundaries, pointer provenance, and UB catalog definitions. |
| LLVM backend | Implemented through the Rust Inkwell path for core expressions and native object generation. | Mature target triples, ABI contracts, and linker policies. |
| WebAssembly target | Architectural roadmap for Phase 2/3. | Add a Wasm target prototype, browser playground, and sandboxed package execution. |
| Migrator Engine | Architectural roadmap for Phase 2/3. | Convert legacy source into readable Logicodex with explicit semantic review. |
| Continuous runtime memory attestation | Architectural roadmap for Phase 2/3. | Convert the current security plan contract into concrete digest insertion, verifier stubs, and target-specific fail-stop mitigation. |
| Freestanding support | Alpha target profile and plan generation. | Add linker scripts, bootloader examples, raw-pointer gates, and hardware-region policies. |

## Freestanding and Hosted Safety Boundary

The `--target freestanding` profile is intended for OS-less integration contexts such as kernels, bootloaders, firmware, and hypervisors. Examples that write to physical addresses such as VGA text memory at `0xB8000` are valid only under freestanding execution or equivalent kernel-space mapping authority. Hosted operating systems such as Linux and Windows normally protect processes with virtual memory paging and ASLR, so direct physical address manipulation from user space should be expected to fail through page-fault defenses unless mediated by mechanisms such as `/dev/mem` or Ring-0 drivers.

## Security Roadmap Boundary

The active self-defense model defines a fail-stop invariant for executable-code tampering. In hosted environments, hard self-destruction means process termination through native abort behavior. In freestanding environments, it means a target-specific termination primitive such as a CPU Triple Fault, `hlt` halt loop, or hardware watchdog reset. The Phase 1 tree documents this contract; Phase 2/3 work must implement the production verifier and mitigation runtime.

## Governance and Licensing

Logicodex is distributed under permissive dual licensing through **MIT License** and **Apache License 2.0**. The name **Logicodex**, **Logicodex Language**, and official branding assets remain protected project identifiers to preserve ecosystem clarity and avoid misleading forks.

## Collaboration

Contributors interested in compiler engineering, LLVM optimization, operating-system targets, formal specification, documentation, or AI-assisted migration are invited to coordinate with **MSA Studio** through `mymsastudio@gmail.com`.
"""
README.write_text(readme)

# --- MANUAL adjacency update ---
manual = MANUAL.read_text().replace("Ckt0", "crt0")
manual_note = """\n\n## Peer-Review Alignment Notes for v1.0.1-alpha\n\nThe Phase 1 compiler implements the verified core path: dictionary loading, lexing, parsing, AST construction, semantic analysis, and LLVM-Inkwell backend generation. WebAssembly targeting, the Logicodex Migrator Engine, and Continuous Runtime Memory Attestation are Phase 2/3 roadmap specifications. The dictionary is consumed during lexing only; parser behavior is based on canonical `TokenKind` values rather than macro rewriting. Freestanding memory examples such as `0xB8000` are OS-less or kernel-authority examples and are not valid hosted user-space memory operations under Linux or Windows without explicit kernel-space mapping.\n"""
if "## Peer-Review Alignment Notes for v1.0.1-alpha" not in manual:
    manual += manual_note
MANUAL.write_text(manual)

# --- LEXER source commentary ---
lexer = LEXER.read_text()
lexer_comment = """// core_map.json is consumed strictly in this lexer layer. Surface lexemes such as
// `MULA`, `BEGIN`, or `{` are normalized into canonical TokenKind values before
// Parser::new receives the token stream. This is token-level normalization, not
// parser-side macro rewriting or grammar desugaring.
"""
if "core_map.json is consumed strictly in this lexer layer" not in lexer:
    lexer = lexer.replace("#[derive(Debug, Clone)]\npub struct Lexicon {", lexer_comment + "#[derive(Debug, Clone)]\npub struct Lexicon {")
LEXER.write_text(lexer)

# --- MAIN source commentary and generated plan wording ---
main = MAIN.read_text()
main = main.replace(
    "A mismatch represents suspected process injection, fileless malware tampering, or unauthorized runtime patching, and must trigger immediate panic termination, sensitive-register clearing, and hard process self-destruction.",
    "A mismatch represents suspected process injection, fileless malware tampering, or unauthorized runtime patching, and must trigger immediate panic termination, sensitive-register clearing, and target-appropriate hard self-destruction. Hosted targets translate this to native process abort behavior; freestanding targets translate it to a CPU Triple Fault where appropriate, an assembly hlt halt loop, or a hardware watchdog reset."
)
main = main.replace(
    "The planned `*int` raw pointer representation is reserved for memory-mapped I/O, including VGA text memory at `0xB8000` and serial UART ports such as `0x3F8`, under explicit backend safety gates.",
    "The planned `*int` raw pointer representation is reserved for memory-mapped I/O, including VGA text memory at `0xB8000` and serial UART ports such as `0x3F8`, under explicit backend safety gates. This is a freestanding or kernel-authority operation: hosted Linux or Windows processes with virtual memory paging and ASLR cannot directly manipulate physical addresses without kernel-space mapping such as `/dev/mem` or Ring-0 driver mediation."
)
MAIN.write_text(main)

# --- CODEGEN plan field wording ---
codegen = CODEGEN.read_text()
codegen = codegen.replace(
    'panic_strategy: "clear_sensitive_registers_and_abort_process",',
    'panic_strategy: "clear_sensitive_registers_and_target_specific_fail_stop",'
)
CODEGEN.write_text(codegen)

print("Applied peer-review alignment updates to WHITE_PAPER.md, README.md, MANUAL.md, src/lexer.rs, src/main.rs, and src/codegen.rs")
