#!/usr/bin/env python3
"""Generate GrammarandDictionary.md from dict/core_map.json for Logicodex v1.21-alpha."""
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CORE_MAP = ROOT / "dict/core_map.json"
OUTPUT = ROOT / "GrammarandDictionary.md"


def fmt_aliases(values: list[str]) -> str:
    if not values:
        return "—"
    return ", ".join(f"`{value}`" for value in values)


def policy_summary(token: dict) -> str:
    parts: list[str] = []
    if token.get("beginner_line_terminated"):
        parts.append("newline-terminated Malay alias")
    critical = token.get("critical_policy") or {}
    if critical.get("requires_explicit_block"):
        parts.append("explicit block")
    if critical.get("requires_explicit_terminator"):
        parts.append("explicit terminator")
    if critical.get("requires_explicit_terminator_inside"):
        parts.append("explicit terminator inside")
    return ", ".join(parts) if parts else "—"


def token_table(tokens: dict) -> str:
    rows = [
        "| Token | Expert canonical shorthand | Primary Malay alias (`primary_ms`) | English/pseudocode aliases | Policy | Description |",
        "|---|---:|---:|---|---|---|",
    ]
    for name, spec in tokens.items():
        rows.append(
            "| {name} | `{expert}` | `{primary}` | {aliases} | {policy} | {description} |".format(
                name=name,
                expert=spec.get("expert", ""),
                primary=spec.get("primary_ms", ""),
                aliases=fmt_aliases(spec.get("aliases", [])),
                policy=policy_summary(spec),
                description=spec.get("description", "").replace("|", "\\|"),
            )
        )
    return "\n".join(rows)


def main() -> None:
    data = json.loads(CORE_MAP.read_text(encoding="utf-8"))
    tokens = data["tokens"]
    policy = data.get("policy", {})

    content = f"""# Logicodex Grammar and Dictionary

This document describes the **grammar** and **token dictionary** for **current Logicodex v1.21-alpha** based on the `dict/core_map.json` schema v{data.get('schema_version')}.[1] The current dictionary defines **expert canonical shorthand mode** through the `expert` field as the canonical compiler reference, while `primary_ms` is the main human-facing Malay alias. The `aliases` field contains explicitly accepted English pseudocode and compatibility spellings.

> **Short principle:** the compiler reference surface is the expert canonical shorthand stored in `expert`; Malay-first users may write the `primary_ms` aliases; and English/pseudocode aliases are accepted only when they are listed explicitly in the dictionary.

## 1. Current Dictionary Policy

| Policy | Value |
|---|---|
| Version | `{data.get('version')}` |
| Language | `{data.get('language')}` |
| Reference mode | `{policy.get('reference_mode')}` |
| Primary human language | `{policy.get('primary_human_language')}` |
| Alias order rule | `{policy.get('alias_order_rule')}` |
| Malay alias newline statement policy (`beginner_statement_policy`) | {policy.get('beginner_statement_policy')} |
| Critical boundary policy | {policy.get('critical_boundary_policy')} |

This policy means that expert canonical shorthand such as `let`, `print`, `return`, `fn`, `if`, and block braces `{{` `}}` is the most stable compiler reference form. Malay pseudocode aliases such as `BINA`, `PAPAR`, `PULANG`, `FUNGSI`, `JIKA`, `MAKA`, `MELAINKAN`, `MULA`, and `TAMAT` are official human-facing aliases mapped to the same canonical token identities.

## 2. v1.21-alpha Grammar Summary

The current grammar is **statement-oriented** and supports programs wrapped in explicit blocks using `{{ ... }}` or `MULA ... TAMAT`.[2] At the top level, the parser accepts `use` declarations, `hw` declarations, `hw_unsafe` hardware zones, `fn` definitions, and regular statements such as `let`, `print`, `return`, `if`, `while`, `loop`, `break`, `continue`, or expression statements.

| Grammar form | Expert form | Malay form | Note |
|---|---|---|---|
| Program/block | `{{ ... }}` | `MULA ... TAMAT` | Explicit blocks are used for programs, functions, `if`, `else`, `while`, `loop`, and `hw_unsafe`. |
| Binding | `let name: Type = expr;` | `BINA name: Type = expr;` | Type annotations may be explicit or inferred. Expert `let` requires a semicolon. |
| Print | `print expr;` | `PAPAR expr;` | Malay aliases may be newline-terminated outside critical zones. |
| Function | `fn name(params) -> Type {{ ... }}` | `FUNGSI name(params) -> Type MULA ... TAMAT` | Parameters use `name: Type` and are separated by commas. |
| Return | `return expr;` | `PULANG expr;` | Valid inside functions only. |
| Conditional | `if condition then {{ ... }} else {{ ... }}` | `JIKA condition MAKA MULA ... TAMAT MELAINKAN MULA ... TAMAT` | `then`/`MAKA` may be followed by a newline before the block. |
| While loop | `while condition {{ ... }}` | `SELAGI condition MULA ... TAMAT` | Executable in v1.21-alpha with Boolean conditions. |
| Unconditional loop | `loop {{ ... }}` | `ULANG MULA ... TAMAT` | Executable in v1.21-alpha; use `break` or `continue` for loop control. |
| Loop control | `break;` / `continue;` | `HENTI;` / `LANGKAU;` | Valid only inside `while` or `loop` bodies. |
| Hardware declaration | `hw name: Type = addr literal;` | `PERKAKASAN name: Type = ALAMAT literal;` | Critical syntax; an explicit semicolon is required. |
| Hardware zone | `hw_unsafe {{ ... }}` | `ZON_PERKAKASAN MULA ... TAMAT` | Explicit block and explicit statement terminators are required. |
| Pointer type | `ptr<Type>` | `PTR<Type>` | Critical boundary syntax used for address/provenance typing. |
| Complex roadmap declarations | `struct`, `enum`, `unsafe`, `extern` | `BENTUK`, `PILIHAN`, `BERISIKO`, `LUAR` | Recognized by the lexer and dictionary, then stopped at parser level with a bilingual unimplemented diagnostic. |

## 3. Expression Structure

Expressions support integer literals, Boolean literals, string literals, variables, address literals using `addr`, grouped expressions using parentheses, and binary operators.[2] Operator precedence is handled from logical `or`/`||` through logical `and`/`&&`, bitwise operators, equality, comparison, shifts, arithmetic terms, and factors.

| Category | Operator or form | Example |
|---|---|---|
| Logical OR | `||`, `or` | `ready || fallback` |
| Logical AND | `&&`, `and` | `ok && enabled` |
| Bitwise | `&`, `|` | `mask & flag` |
| Equality | `==`, `!=` | `flag == true` |
| Comparison | `>`, `>=`, `<`, `<=` | `total >= limit` |
| Shift | `<<`, `>>` | `value << 1` |
| Arithmetic | `+`, `-`, `*`, `/` | `seed * scale + 1` |
| Grouping | `(expr)` | `(a + b) * 2` |
| Address literal | `addr 65280` / `ALAMAT 65280` | `hw port: U16 = addr 65280;` |
| Boolean literal | `true`, `false` / `BENAR`, `SALAH` | `let ok: Bool = true;` |

## 4. Semicolon and Layout Tolerance

The parser accepts repeated blank lines, Windows CRLF input, extra semicolons as layout separators, trailing layout before EOF, and newlines after `then`/`MAKA` or `else`/`MELAINKAN` before a block begins. However, **expert canonical shorthand remains strict**: `let`, `print`, `return`, `break`, and `continue` require explicit semicolons.

| Location | Is a semicolon required? | Description |
|---|---|---|
| Expert statements | Yes | `let x = 1;`, `print x;`, `return x;`, `break;`, and `continue;` must end with `;`. |
| Malay aliases outside critical zones | Not always | `BINA`, `PAPAR`, and `PULANG` may be newline-terminated outside critical zones. |
| Loop control aliases | Yes | `HENTI;` and `LANGKAU;` require explicit semicolons. |
| Hardware zone | Yes | Every statement inside `hw_unsafe` / `ZON_PERKAKASAN` requires an explicit terminator. |
| Hardware declaration and address syntax | Yes | `hw port: U16 = addr 65280;` requires `;`. |

## 5. Usage Examples

### 5.1 Expert Canonical Shorthand Mode

This example uses expert canonical shorthand as the complete compiler reference surface and keeps an explicit terminator on every statement.

```logicodex
{{
let seed: I64 = 21;
let scale: I64 = 2;
let limit: I64 = 50;
let total: I64 = seed * scale;
print total;
fn clamp(value: I64, max: I64) -> I64 {{
if value > max then
{{
return max;
}}
else
{{
return value;
}}
}}
while total < limit {{
print total;
break;
}}
}}
```

### 5.2 Official Malay Pseudocode

This example preserves the same program meaning using `primary_ms` aliases. Semicolons are retained so the example remains easy to compare with expert canonical shorthand and clear for human readers.

```logicodex
MULA
BINA seed: I64 = 21;
BINA scale: I64 = 2;
BINA limit: I64 = 50;
BINA total: I64 = seed * scale;
PAPAR total;
FUNGSI clamp(value: I64, max: I64) -> I64 MULA
JIKA value > max MAKA
MULA
PULANG max;
TAMAT
MELAINKAN
MULA
PULANG value;
TAMAT
TAMAT
SELAGI total < limit MULA
PAPAR total;
HENTI;
TAMAT
TAMAT
```

### 5.3 Pseudo-English Compatibility

This example uses English/pseudocode aliases listed in the dictionary. This is not the reference mode, but the aliases are mapped to the same canonical tokens when they exist in `core_map.json`.

```logicodex
START
CREATE seed: I64 = 21;
CREATE scale: I64 = 2;
CREATE limit: I64 = 50;
CREATE total: I64 = seed * scale;
PRINT total;
FUNCTION clamp(value: I64, max: I64) -> I64 START
IF value > max THEN
START
RETURN max;
END
ELSE
START
RETURN value;
END
END
WHILE total < limit START
PRINT total;
BREAK;
END
END
```

### 5.4 Malay Aliases with Newline Terminators

Outside critical zones, `BINA`, `PAPAR`, and `PULANG` may use newline terminators. This is the ergonomic baseline for Malay aliases without changing the strictness of expert canonical shorthand mode.

```logicodex
MULA
BINA x = 1
BINA y = 2
BINA total = x + y
PAPAR total
TAMAT
```

### 5.5 Strict Hardware Zone

Hardware I/O and raw address syntax remain under a stricter boundary policy.[3] Blocks must be explicit and every statement inside the zone requires a semicolon, even when Malay aliases are used.

```logicodex
MULA
ZON_PERKAKASAN MULA
PERKAKASAN port: U16 = ALAMAT 65280;
BINA mirror: PTR<U16> = port;
PAPAR mirror;
TAMAT
TAMAT
```

## 6. Complete Dictionary Token Table

The following table is generated directly from `dict/core_map.json`.[1] The **Expert canonical shorthand** column is the compiler reference form; **Primary Malay alias (`primary_ms`)** is the main human-facing Malay alias; and **English/pseudocode aliases** lists additional accepted spellings when they are explicitly present.

{token_table(tokens)}

## 7. Current Limit Notes

Logicodex v1.21-alpha uses a **split-implementation** boundary. Control flow and operator tokens listed as executable in the table have AST, parser, semantic, and code-generation coverage. Complex declaration tokens such as `struct`, `enum`, `unsafe`, and `extern` are recognized by the lexer and dictionary, but are intentionally stopped at parser level with a bilingual unimplemented diagnostic until their type-layout, ABI, and safety models are finalized.

## 8. Regenerating This Document

Regenerate this document after `dict/core_map.json` changes by running the following command from the repository root:

```bash
python3 scripts/generate_grammar_and_dictionary_md.py
```

After regeneration, run the regular project validator so the dictionary schema, parser, and policy remain consistent:

```bash
python3 scripts/validate_v121_executable_logic.py
cargo test --target x86_64-unknown-linux-gnu
RUSTFLAGS='-D warnings' cargo build --target x86_64-unknown-linux-gnu
```

## References

[1]: dict/core_map.json "Logicodex core_map.json schema v2 dictionary"
[2]: src/parser.rs "Logicodex parser grammar implementation"
[3]: src/semantic.rs "Logicodex semantic analyzer policy implementation"
"""
    OUTPUT.write_text(content, encoding="utf-8")
    print(f"wrote {OUTPUT}")


if __name__ == "__main__":
    main()
