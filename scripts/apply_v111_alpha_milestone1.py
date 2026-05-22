#!/usr/bin/env python3
"""Apply Logicodex v1.11-alpha Phase 2 Milestone 1 updates."""
from __future__ import annotations

from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[1]
OLD_VERSION = "1.0.1-alpha"
OLD_TAG = "v1.0.1-alpha"
NEW_VERSION = "1.11-alpha"
NEW_TAG = "v1.11-alpha"
NEW_HEADER = """// =========================================================================
// Project: Logicodex Language Engine (Phase 2 - Milestone 1)
// Version: v1.11-alpha (EBNF Formal Grammar Integration)
// Architect & Creator: Mohamad Supardi Abdul (mymsastudio@gmail.com)
// Copyright (c) 2026. All Rights Reserved.
// Licensed under permissive dual-license: MIT & Apache License 2.0
// ========================================================================="""

SPEC_TEXT = """# ❖ Logicodex Formal Grammar Specification (v1.11-alpha)

Notation Legend: `::=` means "defined as"; `|` alternation; `*` zero-or-more; `+` one-or-more; `?` optional; terminals enclosed in quotes.

## Layer 1 — Surface Lexical Layer (core_map.json Input)

```ebnf
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
```

## Layer 2 — Canonical Token Layer (Lexer Output / Parser Invariant)

The Logicodex parser operates exclusively on canonical token kinds emitted by the normalization lexer. Surface spellings are erased post-lexing.

```ebnf
BeginBlock ::= TokenKind::BeginBlock
EndBlock   ::= TokenKind::EndBlock
```

## Layer 3 — Syntactic Grammar Layer (AST Structural Contract)

```ebnf
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
```

## Layer 4 — Semantic Constraint Layer

Grammar validation guarantees structural correctness. Type compilation and unsafe capability checks (e.g., restricting raw hardware pointer addressing to freestanding configurations) are strictly enforced during the separate Static Analysis phase.
"""

TRACKED_TEXT_FILES = [
    "Cargo.toml",
    "NOTICE",
    "README.md",
    "WHITE_PAPER.md",
    "MANUAL.md",
    "ROADMAP.md",
    "scripts/update_release_archives.sh",
]

SOURCE_HEADER_FILES = [
    *sorted((ROOT / "src").rglob("*.rs")),
    *sorted((ROOT / "examples").rglob("*.ldx")),
    *sorted((ROOT / "stdlib").rglob("*.ldx")),
    ROOT / "dict" / "core_map.json",
]

for rel in TRACKED_TEXT_FILES:
    path = ROOT / rel
    if path.exists():
        text = path.read_text(encoding="utf-8")
        text = text.replace(OLD_TAG, NEW_TAG).replace(OLD_VERSION, NEW_VERSION)
        text = text.replace("V1.0.1-alpha", "V1.11-alpha")
        text = text.replace("Internal Security & OS Freestanding Test", "EBNF Formal Grammar Integration")
        path.write_text(text, encoding="utf-8")

for path in SOURCE_HEADER_FILES:
    if path.exists():
        text = path.read_text(encoding="utf-8")
        text = re.sub(r"\A// =========================================================================\n// Project:.*?\n// =========================================================================", NEW_HEADER, text, count=1, flags=re.S)
        text = text.replace(OLD_TAG, NEW_TAG).replace(OLD_VERSION, NEW_VERSION)
        text = text.replace("Internal Security & OS Freestanding Test", "EBNF Formal Grammar Integration")
        path.write_text(text, encoding="utf-8")

spec_path = ROOT / "spec" / "v1.11-alpha" / "UpdateIssue1-ebnf.md"
spec_path.parent.mkdir(parents=True, exist_ok=True)
spec_path.write_text(SPEC_TEXT, encoding="utf-8")

roadmap = ROOT / "ROADMAP.md"
text = roadmap.read_text(encoding="utf-8")
text = text.replace(
    "- [ ] **Issue #01 — Formal EBNF Grammar Definition:** Document the exact grammar rules for both **Novice Pseudocode** and **Expert Shorthand** to eliminate parsing ambiguity.",
    "- [X] **Issue #01 — Formal EBNF Grammar Definition (COMPLETED / SOLVED):** Document the exact grammar rules for both **Novice Pseudocode** and **Expert Shorthand** to eliminate parsing ambiguity."
)
text = text.replace(
    "| Issue #01 | Open | TBD | A reviewed EBNF document exists and covers novice, localized, and expert shorthand forms. |",
    "| Issue #01 | [X] COMPLETED / SOLVED | Mohamad Supardi Abdul (MSA Studio) | 1. Formal 4-Layer grammar checked in as a living document inside `spec/v1.11-alpha/UpdateIssue1-ebnf.md`.<br>2. Recursive-descent compiler entry pipeline verified to ingest token maps collision-free.<br>3. Concrete freestanding token productions (`hw` and `addr`) structurally declared to enable upcoming security capability gates. |"
)
roadmap.write_text(text, encoding="utf-8")

print(f"Applied {NEW_TAG} Phase 2 Milestone 1 updates")
print(f"Created {spec_path.relative_to(ROOT)}")
