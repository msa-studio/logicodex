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
// Version: v1.21-alpha (Specification Baseline & Practical Severity Roadmap)
// Architect & Creator: Mohamad Supardi Abdul (mymsastudio@gmail.com)
// Copyright (c) 2026. All Rights Reserved.
// Licensed under permissive dual-license: MIT & Apache License 2.0
// =========================================================================
"""

EBNF_SPEC = '''# Logicodex Grammar Specification Baseline (v1.21-alpha)
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
HardwareZoneToken::= "ZON_PERKAKASAN" | "hw_unsafe"
AddressToken     ::= "ALAMAT" | "addr"
ReturnToken      ::= "PULANG" | "return"
AssignOp         ::= "="
AddOp            ::= "+"
SubOp            ::= "-"

## Layer 2 — Canonical Token Layer (Lexer Output / Parser Invariant)
The Logicodex parser operates exclusively on canonical token kinds emitted by the normalization lexer. Surface spellings are erased post-lexing.
BeginBlock ::= TokenKind::BeginBlock
EndBlock   ::= TokenKind::EndBlock

## Layer 3 — Syntactic Grammar Layer (Current AST Contract)
Program          ::= ( GlobalDeclaration | FunctionDef )*
GlobalDeclaration::= HardwareDecl | UseDecl | HardwareZone
HardwareDecl     ::= HardwareToken Identifier ":" Type "=" AddressToken LiteralInt ";"
HardwareZone     ::= HardwareZoneToken Block
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
Grammar validation establishes structural correctness for the implemented language subset. Type compilation and unsafe capability checks are handled by the separate Static Analysis phase and should continue to expand through tested milestones.
'''

PROVENANCE_SPEC = '''# Logicodex Undefined Behavior and Pointer Provenance Design Baseline (v1.21-alpha)

## 1. Industry-Derived Layer Classification
Logicodex categorizes semantic violations based on established low-level language paradigms to facilitate seamless optimization mapping via the LLVM backend:
- **Linear Layer (C-Style Paradigms):** Focuses on raw pointer arithmetic, memory-mapped offsets, and volatile I/O actions. Pointer offsets (e.g., `ptr + 5`) preserve Strict Provenance. Hardcoded literal casts via `addr` generate Hardware/Wild Provenance, forcing the optimizer to assume aliasing and bypass unsafe memory optimization drops.
- **Object-Oriented Layer (C++ Style Paradigms):** Focuses on flat struct layouts, deterministic sequential memory placement, and scoped destructor functions (drop semantics). Re-use of expired memories or double execution of object destruction logic is strictly treated as an explicit object boundary violation.
- **Safety Layer (Rust-Style Paradigms):** Focuses on strict compile-time index bounds checking and deterministic automatic resource cleanup via RAII patterns.

## 2. Practical Error Severity Classification Baseline
If a runtime error escapes static compilation analysis or triggers during future runtime attestation work, the compiler should classify response routines through three structural severity tiers. Each tier must be implemented, tested, and benchmarked before production-readiness claims are made:
- **TIER 1: CRITICAL:** Intended for executable-integrity failure or unsafe hardware-region access in explicitly selected freestanding contexts. This remains a long-term fail-stop objective requiring target-specific implementation, review, and tests.
- **TIER 2: MEDIUM:** Intended for dynamic division by zero, runtime resource depletion, or isolated execution failure. The practical first step is normal process or function failure paths with clear diagnostics.
- **TIER 3: LOW:** Intended for warnings such as safe integer truncation, benign wrap-around, or deprecated library use. Prefer diagnostics and metadata unless measured runtime behavior is explicitly implemented.

## 3. Generic Hardware I/O Capability Gating
Raw hardware-address manipulation must be explicitly scoped through `ZON_PERKAKASAN { ... }` or `hw_unsafe { ... }`. Outside this lexical zone, semantic analysis rejects raw address pointer bindings with the Level 1 diagnostic: `KRITIKAL: Ralat Umum Tahap 1 - Percubaan Mutasi Perkakasan Tanpa Kebenaran Skop Zon Selamat / CRITICAL: General Error Level 1 - Attempted Hardware Mutation Without Safe Zone Scope Authorization.` This remains a practical baseline gate; volatile lowering, dynamic device manifests, and hardware mutex coordination remain deferred roadmap work.
'''

REPOS_CONTEXT = '''# Logicodex Repository Context Document
This authoritative document inventories the core architectural assets of the Logicodex Language repository and establishes the operational context for each component under the current logicodex v 1.21 alpha milestone.

## 1. Compiler Core Frontend & Backend (`src/`)
- `src/main.rs`: The execution entry point. Houses the Clap CLI driver framework, manages compilation flags (`--target`, `--secure`), and prints the official terminal ASCII logo.
- `src/lexer.rs`: The dynamic dictionary tokenizer. Consumes raw `.ldx` files and queries `core_map.json` to substitute localized or shorthand words into uniform canonical token IDs.
- `src/parser.rs`: The structural AST builder. Utilizes a hand-rolled handwritten recursive-descent strategy and Pratt parsing engine to process token streams into strict compiler primitives.
- `src/semantic.rs`: The safety gatekeeper. Performs type inference checks, structural scoping constraints, constant-folding arithmetic validations, and filters programming hazards before lowering code.
- `src/codegen.rs`: The LLVM intermediate generator. Transpiles checked AST structures directly into optimized LLVM IR nodes and emits compiler-core output while documenting future severity handling points.
- `src/target.rs`: The platform deployment matrix. Configures cross-compilation configurations, optimization passes (`O3`), and target triples (hosted Windows/Linux vs experimental freestanding).

## 2. Operating System Native Bridges (`src/os/`)
- `src/os/windows.rs`: Implements native console output through the Windows Win32 API suite.
- `src/os/linux.rs`: Implements native Linux output through x86_64 POSIX-style syscall integration experiments.

## 3. Lexical Dictionaries & Code Reference (`dict/`, `examples/`)
- `dict/core_map.json`: The core dynamic mapping scheme. Houses the canonical dictionary that standardizes novice Malay pseudocode and expert shortcut semantics into identical primitives.
- `examples/`: Contains official functional validation files with the `.ldx` extension, demonstrating both localized verbose programming styles and experimental freestanding memory-operation examples.

## 4. Documentation & Specifications (`spec/`, Root)
- `README.md`: The official Executive Summary manifesto outlining the dual-syntax thesis and project governance.
- `WHITE_PAPER.md`: The research white paper describing the compiler pipeline, alpha status boundary, and long-term systems objectives.
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
    text = text.replace("EBNF Formal Grammar Integration", "Specification Baseline & Practical Severity Roadmap")
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

The **current logicodex v 1.21 alpha** milestone establishes a practical compiler-core baseline and a documented security research direction. It includes a four-layer grammar baseline, an Undefined Behavior and Pointer Provenance design note, and a Critical/Medium/Low severity taxonomy. Stronger security, freestanding, and measured-overhead goals are long-term engineering objectives until implemented, benchmarked, and validated by repeatable tests.

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

The **current logicodex v 1.21 alpha** deployment milestone establishes a synchronized specification baseline and practical compiler-core checkpoint. It includes the canonical EBNF grammar, the Undefined Behavior and Pointer Provenance design note, and the repository context inventory required for audit-driven compiler engineering.

The severity model classifies runtime and future attestation events into **Critical**, **Medium**, and **Low** tiers. These tiers are documented as an engineering target so diagnostics and mitigation paths can be implemented, tested, and benchmarked before any measured-overhead or production-readiness claim is made.

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
        "This roadmap tracks the open issues and architectural milestones established to transition **Logicodex** from a research/compiler initiative into a more complete, evidence-backed ecosystem, directly responding to the **V1.21-alpha technical evaluation**. For Phase 2 Deployment Integration, it also records the completion status of Issue #01 and links the formal EBNF grammar artifact checked into the repository.",
        "This roadmap tracks the open issues and architectural milestones established to transition **Logicodex** from a research/compiler initiative into a more complete, evidence-backed ecosystem under the **V1.21-alpha Phase 2 Deployment Integration** milestone. It records Issue #01 as complete and Issue #02 as an incremental provenance and hardware-zone implementation track, linking the formal EBNF grammar artifact, the Undefined Behavior and Pointer Provenance specification, and the practical severity model checked into the repository.",
    )
    text = text.replace(
        "- [X] **Issue #01 — Formal EBNF Grammar Definition (COMPLETED / SOLVED):** Document the exact grammar rules for both **Novice Pseudocode** and **Expert Shorthand** to eliminate parsing ambiguity.",
        "- [X] **Issue #01 — Formal EBNF Grammar Definition (COMPLETED / SOLVED):** Document the exact grammar rules for both **Novice Pseudocode** and **Expert Shorthand** to eliminate parsing ambiguity.",
    )
    text = text.replace(
        "- [ ] **Issue #02 — Undefined Behavior Catalog & Pointer Provenance:** Define explicit rules governing raw pointer operations, physical memory-mapped boundaries, hosted-memory isolation, and freestanding memory-access constraints.",
        "- [~] **Issue #02 — Undefined Behavior Catalog & Pointer Provenance (IN PROGRESS):** Define explicit rules governing raw pointer operations, physical memory-mapped boundaries, hosted-memory isolation, freestanding memory-access constraints, hardware-zone lexical gates, and practical severity classification tiers.",
    )
    issue1_row = "| Issue #01 | [X] COMPLETED / SOLVED | Mohamad Supardi Abdul | 1. Formal 4-Layer grammar checked in as a living document inside `spec/v1.21-alpha/UpdateIssue1-ebnf.md`.<br>2. Recursive-descent compiler entry pipeline verified to ingest token maps collision-free.<br>3. Concrete freestanding token productions (`hw` and `addr`) structurally declared to enable upcoming security capability gates. |"
    issue2_row = "| Issue #02 | [~] IN PROGRESS - generic hardware scope gate implemented | Mohamad Supardi Abdul | Layered error modeling and practical severity tiers are documented, and raw hardware address pointer bindings are now gated behind `ZON_PERKAKASAN` / `hw_unsafe` lexical zones. Volatile lowering, device manifests, and hardware mutex coordination remain deferred implementation work. |"
    text = re.sub(r"\| Issue #01 \|.*?\|", issue1_row, text)
    text = re.sub(r"\| Issue #02 \|.*?\|", issue2_row, text)
    write("ROADMAP.md", text)


def update_cargo_compatibility() -> None:
    """Preserve the stable Rust 1.75 / Edition 2021 dependency floor on rerun."""
    cargo = read("Cargo.toml")
    cargo = re.sub(r'edition = "[^"]+"', 'edition = "2021"', cargo, count=1)
    cargo = re.sub(r'clap = \{ version = "[^"]+", features = \["derive"\] \}', 'clap = { version = "=4.4.18", features = ["derive"] }', cargo)
    if 'clap_lex = "=0.6.0"' not in cargo:
        cargo = cargo.replace('clap = { version = "=4.4.18", features = ["derive"] }\n', 'clap = { version = "=4.4.18", features = ["derive"] }\nclap_lex = "=0.6.0"\n')
    cargo = re.sub(r'inkwell = \{ version = "[^"]+", features = \["llvm[0-9]+-0"\] \}', 'inkwell = { version = "0.4.0", features = ["llvm15-0"] }', cargo)
    if 'public_release_label = "v1.21-alpha"' not in cargo:
        cargo = cargo.replace('source_extension = "ldx"\n', 'source_extension = "ldx"\npublic_release_label = "v1.21-alpha"\n')
    if 'rust_channel = "1.75.0"' not in cargo:
        cargo = cargo.replace('public_release_label = "v1.21-alpha"\n', 'public_release_label = "v1.21-alpha"\nrust_channel = "1.75.0"\n')
    cargo = re.sub(r'llvm_feature = "[^"]+"', 'llvm_feature = "llvm15-0"', cargo)
    write("Cargo.toml", cargo)
    write("rust-toolchain.toml", '[toolchain]\nchannel = "1.75.0"\n')


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
    update_cargo_compatibility()
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
