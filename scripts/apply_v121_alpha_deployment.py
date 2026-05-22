#!/usr/bin/env python3
"""Apply the Logicodex v1.21-alpha Phase 2 deployment integration update.

This script is intentionally deterministic so the repository update can be audited
and re-applied. It updates version metadata, source headers, specification files,
repository context documentation, roadmap tracking rows, and release archive naming.
"""
from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

OLD_VERSION_PATTERNS = [
    "v1.11-alpha",
    "V1.11-alpha",
    "1.11-alpha",
    "v1.0.1-alpha",
    "V1.0.1-alpha",
    "1.0.1-alpha",
]

HEADER = """// =========================================================================
// Project: Logicodex Language Engine (Phase 2 Deployment Integration)
// Version: v1.21-alpha (Formal Specifications & Zero-Overhead Severity Model)
// Architect & Creator: Mohamad Supardi Abdul (mymsastudio@gmail.com)
// Copyright (c) 2026. All Rights Reserved.
// Licensed under permissive dual-license: MIT & Apache License 2.0
// =========================================================================
"""

EBNF_SPEC = '''# ❖ Logicodex Formal Grammar Specification (v1.21-alpha)
Notation Legend: `::=` means "defined as"; `|` alternation; `*` zero-or-more; `+` one-or-more; `?` optional; terminals enclosed in quotes.

## Layer 1 — Surface Lexical Layer (core_map.json Input)
Identifier       ::= [a-zA-Z_] [a-zA-Z0-9_]*
LiteralInt       ::= [0-9]+
StringLiteral    ::= '"' [^"\\\\]* '"'

/* Lexical Mapping Targets */
BeginBlockToken  ::= "MULA" | "BEGIN" | "{"
EndBlockToken    ::= "TAMAT" | "END" | "}"
LetToken         ::= "BINA" | "let"
PrintToken       ::= "PAPAR" | "print"
IfToken          ::= "JIKA" | "if"
ThenToken        ::= "MAKA" | "then"
ElseToken        ::= "JIKALAU_TIDAK" | "else"
HardwareToken    ::= "KAWASAN_PERKAKAS" | "hw"
AddressToken     ::= "ALAMAT" | "addr"
ReturnToken      ::= "PULANG" | "return"
AssignOp         ::= "="
AddOp            ::= "+"
SubOp            ::= "-"

## Layer 2 — Canonical Token Layer (Lexer Output / Parser Invariant)
The Logicodex parser operates exclusively on canonical token kinds emitted by the normalization lexer. Surface spellings are erased post-lexing.
BeginBlock ::= TokenKind::BeginBlock
EndBlock   ::= TokenKind::EndBlock

## Layer 3 — Syntactic Grammar Layer (AST Structural Contract)
Program          ::= ( GlobalDeclaration | FunctionDef )*
GlobalDeclaration::= HardwareDecl | UseDecl
HardwareDecl     ::= HardwareToken Identifier ":" Type "=" AddressToken LiteralInt ";"
UseDecl          ::= "use" Identifier ";"
FunctionDef      ::= "fn" Identifier "(" ParamList? ")" ("->" Type)? Block
Block            ::= BeginBlockToken Statement* EndBlockToken
Statement        ::= LetStmt | AssignStmt | PrintStmt | ReturnStmt | IfStmt | ExprStmt
LetStmt          ::= LetToken Identifier (":" Type)? AssignOp Expression ";"
PrintStmt        ::= PrintToken Expression ";"
ReturnStmt       ::= ReturnToken Expression ";"
IfStmt           ::= IfToken Expression ThenToken? Block ( ElseToken Block )?
Expression       ::= BinaryExpr | PrimaryExpr
BinaryExpr       ::= Expression ( AddOp | SubOp ) Expression
PrimaryExpr      ::= Identifier | LiteralInt | StringLiteral | "(" Expression ")"
Type             ::= "I32" | "I64" | "U16" | "U32" | "F64" | "Bool" | "PTR<" Type ">"

## Layer 4 — Semantic Constraint Layer
Grammar validation guarantees structural correctness. Type compilation and unsafe capability checks are strictly enforced during the separate Static Analysis phase.
'''

PROVENANCE_SPEC = '''# ❖ Logicodex Undefined Behavior & Pointer Provenance Specification (v1.21-alpha)

## 1. Industry-Derived Layer Classification
Logicodex categorizes semantic violations based on established low-level language paradigms to facilitate seamless optimization mapping via the LLVM backend:
- **Linear Layer (C-Style Paradigms):** Focuses on raw pointer arithmetic, memory-mapped offsets, and volatile I/O actions. Pointer offsets (e.g., `ptr + 5`) preserve Strict Provenance. Hardcoded literal casts via `addr` generate Hardware/Wild Provenance, forcing the optimizer to assume aliasing and bypass unsafe memory optimization drops.
- **Object-Oriented Layer (C++ Style Paradigms):** Focuses on flat struct layouts, deterministic sequential memory placement, and scoped destructor functions (drop semantics). Re-use of expired memories or double execution of object destruction logic is strictly treated as an explicit object boundary violation.
- **Safety Layer (Rust-Style Paradigms):** Focuses on strict compile-time index bounds checking and deterministic automatic resource cleanup via RAII patterns.

## 2. Zero-Overhead General Error Severity Classification
If a runtime error escapes static compilation analysis or triggers during active runtime attestation, the compiler handles response routines through three structural severity tiers injected directly into the LLVM IR pipeline:
- **🔴 TIER 1: CRITICAL (Hardware & Machine Layer Rupture):** Triggers upon Golden Hash integrity failure or unmapped physical memory access in freestanding mode. It bypasses hosted OS routines, emitting naked machine instructions to trigger an immediate process termination or standard signal (Hosted) or forces a CPU Triple Fault / Hardware Watchdog Reset (Freestanding Bare-Metal) to completely freeze the execution environment and contain threats.
- **🟡 TIER 2: MEDIUM (Process & Execution Layer Failure):** Triggers upon dynamic division by zero, runtime resource depletion, or structural thread deadlocks. It terminates the active thread or isolated sub-process cleanly, returns a panic exit code (e.g., `exit(1)`), and flushes resource drops to standard error logs without bringing down the machine.
- **🟢 TIER 3: LOW (Non-Critical Warning Layer):** Triggers upon safe casting integer truncation, benign unsigned math wrap-around, or deprecated library calls. It operates at zero execution speed deduction, emitting a standard error diagnostic trace or shifting execution safely into user-defined localized `catch` statement blocks.
'''

REPOS_CONTEXT = '''# ❖ Logicodex Repository Context Document
This authoritative document inventories the core architectural assets of the Logicodex Language repository and establishes the operational context for each component under the v1.21-alpha milestone.

## 1. Compiler Core Frontend & Backend (`src/`)
- `src/main.rs`: The execution entry point. Houses the Clap CLI driver framework, manages compilation flags (`--target`, `--secure`), and prints the official terminal ASCII logo.
- `src/lexer.rs`: The dynamic dictionary tokenizer. Consumes raw `.ldx` files and queries `core_map.json` to substitute localized or shorthand words into uniform canonical token IDs.
- `src/parser.rs`: The structural AST builder. Utilizes a hand-rolled handwritten recursive-descent strategy and Pratt parsing engine to process token streams into strict compiler primitives.
- `src/semantic.rs`: The safety gatekeeper. Performs type inference checks, structural scoping constraints, constant-folding arithmetic validations, and filters programming hazards before lowering code.
- `src/codegen.rs`: The LLVM intermediate generator. Transpiles checked AST structures directly into optimized LLVM IR nodes and injects zero-overhead runtime error severity blocks.
- `src/target.rs`: The platform deployment matrix. Configures cross-compilation configurations, optimization passes (`O3`), and target triples (Hosted Windows/Linux vs Freestanding Bare-Metal).

## 2. Operating System Native Bridges (`src/os/`)
- `src/os/windows.rs`: Implements bare-metal native console outputs by linking operations directly to the Windows Win32 API suite.
- `src/os/linux.rs`: Implements hyper-performance native outputs by executing raw x86_64 POSIX-compliant assembly Linux Syscalls, completely avoiding external standard C libraries dependencies.

## 3. Lexical Dictionaries & Code Reference (`dict/`, `examples/`)
- `dict/core_map.json`: The core dynamic mapping scheme. Houses the canonical dictionary that standardizes novice Malay pseudocode and expert shortcut semantics into identical primitives.
- `examples/`: Contains official functional validation files with the `.ldx` extension, demonstrating both localized verbose programming styles and advanced freestanding memory operations.

## 4. Documentation & Specifications (`spec/`, Root)
- `README.md`: The official Executive Summary manifesto outlining the dual-syntax thesis and project governance.
- `WHITE_PAPER.md`: The academic-grade research specification detailing the compiler pipeline, runtime attestation math, and bare-metal OS potential.
- `ROADMAP.md`: The project management tracking center mapping open milestones, tracking tickets, and automated verification acceptance criteria.
- `spec/v1.11-alpha/UpdateIssue1-ebnf.md`: Houses the formalized 4-Layer grammar definition.
- `spec/v1.21-alpha/UpdateIssue2-provenance.md`: Houses the newly integrated Undefined Behavior layers and 3-tier error severity model.
'''


def read(rel: str) -> str:
    return (ROOT / rel).read_text(encoding="utf-8")


def write(rel: str, text: str) -> None:
    path = ROOT / rel
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


def replace_versions(text: str) -> str:
    for old in OLD_VERSION_PATTERNS:
        if old.startswith("V"):
            text = text.replace(old, "V1.21-alpha")
        elif old.startswith("v"):
            text = text.replace(old, "v1.21-alpha")
        else:
            text = text.replace(old, "1.21-alpha")
    text = text.replace("Phase 2 - Milestone 1", "Phase 2 Deployment Integration")
    text = text.replace("EBNF Formal Grammar Integration", "Formal Specifications & Zero-Overhead Severity Model")
    text = text.replace("Phase 2, Milestone 1", "Phase 2 Deployment Integration")
    return text


def update_header(path: Path) -> None:
    text = path.read_text(encoding="utf-8")
    pattern = re.compile(r"\A// =========================================================================\n// Project:.*?\n// =========================================================================\n", re.DOTALL)
    if pattern.match(text):
        text = pattern.sub(HEADER, text, count=1)
    else:
        text = HEADER + text
    path.write_text(replace_versions(text), encoding="utf-8")


def update_text_file(rel: str) -> None:
    path = ROOT / rel
    if path.exists():
        path.write_text(replace_versions(path.read_text(encoding="utf-8")), encoding="utf-8")


def update_readme() -> None:
    rel = "README.md"
    text = replace_versions(read(rel))
    marker = "## v1.21-alpha Phase 2 Deployment Integration\n"
    section = """## v1.21-alpha Phase 2 Deployment Integration

The **v1.21-alpha** milestone synchronizes the formal language specification baseline with the Undefined Behavior and Pointer Provenance model. It adds a canonical four-layer EBNF grammar, a layered C/C++/Rust-derived memory-safety classification, and a zero-overhead Critical/Medium/Low severity architecture intended for direct LLVM IR lowering without runtime speed penalties.

"""
    if marker not in text:
        lines = text.splitlines(True)
        insert_at = 0
        while insert_at < len(lines) and (lines[insert_at].startswith("#") or not lines[insert_at].strip()):
            insert_at += 1
            if insert_at > 4:
                break
        text = "".join(lines[:insert_at]) + section + "".join(lines[insert_at:])
    write(rel, text)


def update_white_paper() -> None:
    rel = "WHITE_PAPER.md"
    text = replace_versions(read(rel))
    marker = "## v1.21-alpha Specification Synchronization\n"
    section = """## v1.21-alpha Specification Synchronization

The **v1.21-alpha** deployment milestone elevates the repository into a synchronized formal-specification baseline. The milestone now includes the canonical EBNF grammar, the Undefined Behavior and Pointer Provenance specification, and the repository context inventory required for audit-driven compiler engineering.

The severity model classifies runtime and attestation events into **Critical**, **Medium**, and **Low** structural tiers. These tiers are documented as compiler-lowerable response blocks so that safety diagnostics and mitigation paths can be injected into LLVM IR while preserving the project’s zero-overhead execution thesis for non-triggering code paths.

"""
    if marker not in text:
        insert_at = text.find("\n## ")
        if insert_at == -1:
            text = text.rstrip() + "\n\n" + section
        else:
            insert_at += 1
            text = text[:insert_at] + section + text[insert_at:]
    write(rel, text)


def update_roadmap() -> None:
    text = replace_versions(read("ROADMAP.md"))
    text = text.replace(
        "This roadmap tracks the open issues and architectural milestones established to transition **Logicodex** from a research/compiler initiative into a production-ready ecosystem, directly responding to the **V1.21-alpha technical evaluation**. For Phase 2 Deployment Integration, it also records the completion status of Issue #01 and links the formal EBNF grammar artifact checked into the repository.",
        "This roadmap tracks the open issues and architectural milestones established to transition **Logicodex** from a research/compiler initiative into a production-ready ecosystem under the **V1.21-alpha Phase 2 Deployment Integration** milestone. It records the completion status of Issue #01 and Issue #02, linking the formal EBNF grammar artifact, the Undefined Behavior and Pointer Provenance specification, and the zero-overhead severity model checked into the repository.",
    )
    text = text.replace(
        "- [X] **Issue #01 — Formal EBNF Grammar Definition (COMPLETED / SOLVED):** Document the exact grammar rules for both **Novice Pseudocode** and **Expert Shorthand** to eliminate parsing ambiguity.",
        "- [X] **Issue #01 — Formal EBNF Grammar Definition (COMPLETED / SOLVED):** Document the exact grammar rules for both **Novice Pseudocode** and **Expert Shorthand** to eliminate parsing ambiguity.",
    )
    text = text.replace(
        "- [ ] **Issue #02 — Undefined Behavior Catalog & Pointer Provenance:** Define explicit rules governing raw pointer operations, physical memory-mapped boundaries, hosted-memory isolation, and freestanding memory-access constraints.",
        "- [X] **Issue #02 — Undefined Behavior Catalog & Pointer Provenance (COMPLETED / SOLVED):** Define explicit rules governing raw pointer operations, physical memory-mapped boundaries, hosted-memory isolation, freestanding memory-access constraints, and zero-overhead severity mitigation tiers.",
    )
    issue1_row = "| Issue #01 | [X] COMPLETED / SOLVED | Mohamad Supardi Abdul | 1. Formal 4-Layer grammar checked in as a living document inside `spec/v1.21-alpha/UpdateIssue1-ebnf.md`.<br>2. Recursive-descent compiler entry pipeline verified to ingest token maps collision-free.<br>3. Concrete freestanding token productions (`hw` and `addr`) structurally declared to enable upcoming security capability gates. |"
    issue2_row = "| Issue #02 | [X] COMPLETED / SOLVED | Mohamad Supardi Abdul | Layered error modeling (C/C++/Rust) integrated into specification. Zero-overhead 3-tier severity mitigation architecture (Critical/Medium/Low) structurally hardcoded to enable direct LLVM IR compilation blocks without execution speed penalties. |"
    text = re.sub(r"\| Issue #01 \|.*?\|", issue1_row, text)
    text = re.sub(r"\| Issue #02 \|.*?\|", issue2_row, text)
    write("ROADMAP.md", text)


def update_release_script() -> None:
    rel = "scripts/update_release_archives.sh"
    text = replace_versions(read(rel))
    text = re.sub(r'NAME="logicodex-v[^"]+"', 'NAME="logicodex-v1.21-alpha"', text)
    write(rel, text)


def main() -> None:
    # Source-like files with mandatory top comment contract.
    header_paths = []
    header_paths.extend(sorted((ROOT / "src").rglob("*.rs")))
    header_paths.extend(sorted((ROOT / "examples").rglob("*.ldx")))
    header_paths.extend(sorted((ROOT / "stdlib").rglob("*.ldx")))
    header_paths.append(ROOT / "dict" / "core_map.json")
    for path in header_paths:
        update_header(path)

    for rel in ["Cargo.toml", "NOTICE", "MANUAL.md"]:
        update_text_file(rel)
    update_readme()
    update_white_paper()
    update_roadmap()
    update_release_script()

    # Preserve current source version strings beyond headers and Cargo metadata.
    update_text_file("src/main.rs")

    write("spec/v1.21-alpha/UpdateIssue1-ebnf.md", EBNF_SPEC)
    write("spec/v1.21-alpha/UpdateIssue2-provenance.md", PROVENANCE_SPEC)
    write("REPOS_CONTEXT.md", REPOS_CONTEXT)


if __name__ == "__main__":
    main()
