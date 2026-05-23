#!/usr/bin/env python3
"""Generate GrammarandDictionary.md from dict/core_map.json for Logicodex v1.21-alpha."""
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CORE_MAP = ROOT / "dict" / "core_map.json"
OUTPUT = ROOT / "GrammarandDictionary.md"


def fmt_aliases(values: list[str]) -> str:
    if not values:
        return "—"
    return ", ".join(f"`{value}`" for value in values)


def policy_summary(token: dict) -> str:
    parts: list[str] = []
    if token.get("beginner_line_terminated"):
        parts.append("newline beginner")
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
        "| Token | Expert canonical | Primary Melayu | Aliases | Policy | Keterangan |",
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

Dokumen ini menerangkan **grammar** dan **dictionary token** untuk **current logicodex v 1.21 alpha** berdasarkan `dict/core_map.json` schema v{data.get('schema_version')}.[1] Dictionary semasa menetapkan **expert mode** sebagai rujukan canonical compiler, manakala `primary_ms` ialah alias manusia utama dalam Bahasa Melayu. Alias lain kekal sebagai bentuk pseudocode atau compatibility spelling yang terhad.

> **Prinsip ringkas:** compiler reference surface ialah `expert`; pengguna Melayu boleh menulis menggunakan `primary_ms`; dan alias tambahan digunakan hanya apabila sudah disenaraikan secara eksplisit dalam dictionary.

## 1. Policy Dictionary Semasa

| Policy | Nilai |
|---|---|
| Version | `{data.get('version')}` |
| Language | `{data.get('language')}` |
| Reference mode | `{policy.get('reference_mode')}` |
| Primary human language | `{policy.get('primary_human_language')}` |
| Alias order rule | `{policy.get('alias_order_rule')}` |
| Beginner statement policy | {policy.get('beginner_statement_policy')} |
| Critical boundary policy | {policy.get('critical_boundary_policy')} |

Policy ini bermaksud syntax expert seperti `let`, `print`, `return`, `fn`, `if`, dan block braces `{{` `}}` ialah bentuk paling stabil untuk compiler. Pseudo Melayu seperti `BINA`, `PAPAR`, `PULANG`, `FUNGSI`, `JIKA`, `MAKA`, `MELAINKAN`, `MULA`, dan `TAMAT` ialah alias rasmi manusia yang dipetakan kepada token canonical yang sama.

## 2. Ringkasan Grammar v1.21-alpha

Grammar semasa adalah **statement-oriented** dan menyokong program yang dibalut dengan block `{{ ... }}` atau `MULA ... TAMAT`.[2] Di peringkat tertinggi, parser menerima deklarasi `use`, deklarasi hardware `hw`, zon hardware `hw_unsafe`, definisi fungsi `fn`, dan statement biasa seperti `let`, `print`, `return`, `if`, atau expression statement.

| Bentuk Grammar | Bentuk Expert | Bentuk Melayu | Nota |
|---|---|---|---|
| Program/block | `{{ ... }}` | `MULA ... TAMAT` | Block eksplisit digunakan untuk program, fungsi, `if`, `else`, dan `hw_unsafe`. |
| Binding | `let name: Type = expr;` | `BINA name: Type = expr;` | Type annotation boleh wujud atau diinfer. Expert `let` wajib semicolon. |
| Print | `print expr;` | `PAPAR expr;` | Dalam beginner Melayu, newline boleh menjadi terminator di luar zon kritikal. |
| Function | `fn name(params) -> Type {{ ... }}` | `FUNGSI name(params) -> Type MULA ... TAMAT` | Parameter menggunakan `name: Type` dan dipisahkan dengan comma. |
| Return | `return expr;` | `PULANG expr;` | Sah di dalam fungsi sahaja. |
| Conditional | `if condition then {{ ... }} else {{ ... }}` | `JIKA condition MAKA MULA ... TAMAT MELAINKAN MULA ... TAMAT` | `then/MAKA` boleh diikuti newline sebelum block. |
| Hardware declaration | `hw name: Type = addr literal;` | `PERKAKASAN name: Type = ALAMAT literal;` | Termasuk syntax kritikal; semicolon eksplisit wajib. |
| Hardware zone | `hw_unsafe {{ ... }}` | `ZON_PERKAKASAN MULA ... TAMAT` | Block eksplisit wajib dan statement di dalamnya wajib terminator. |
| Pointer type | `ptr<Type>` | `PTR<Type>` | Termasuk boundary kritikal; digunakan untuk address/provenance typing. |

## 3. Struktur Expression

Expression semasa menyokong literal integer, boolean, string literal, variable, literal address menggunakan `addr`, grouped expression menggunakan parentheses, dan operator binary.[2] Keutamaan operator dikendalikan daripada comparison/equality ke arithmetic term/factor.

| Kategori | Operator atau Bentuk | Contoh |
|---|---|---|
| Arithmetic | `+`, `-`, `*`, `/` | `seed * scale + 1` |
| Comparison | `>`, `>=`, `<`, `<=` | `total >= limit` |
| Equality | `==`, `!=` | `flag == true` |
| Grouping | `(expr)` | `(a + b) * 2` |
| Address literal | `addr 65280` / `ALAMAT 65280` | `hw port: U16 = addr 65280;` |
| Boolean literal | `true`, `false` / `BENAR`, `SALAH` | `let ok: Bool = true;` |

## 4. Semicolon dan Layout Tolerance

Parser semasa menerima blank lines berganda, CRLF Windows, extra semicolon sebagai layout separator, trailing layout sebelum EOF, dan newline selepas `then/MAKA` atau `else/MELAINKAN` sebelum block bermula. Walau bagaimanapun, **expert syntax kekal strict**: `let`, `print`, dan `return` dalam expert mode perlu diakhiri dengan semicolon.

| Lokasi | Semicolon diperlukan? | Huraian |
|---|---|---|
| Expert statements | Ya | `let x = 1;`, `print x;`, dan `return x;` mesti mempunyai `;`. |
| Beginner Melayu di luar zon kritikal | Tidak semestinya | `BINA`, `PAPAR`, dan `PULANG` boleh newline-terminated di luar critical zone. |
| Hardware zone | Ya | Semua statement di dalam `hw_unsafe` / `ZON_PERKAKASAN` perlu terminator eksplisit. |
| Hardware declaration dan address syntax | Ya | `hw port: U16 = addr 65280;` wajib `;`. |

## 5. Contoh Penggunaan

### 5.1 Canonical Expert Mode

Contoh ini menggunakan surface canonical compiler sepenuhnya dan mengekalkan terminator eksplisit pada setiap statement.

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
if total >= limit then
{{
print limit;
}}
else
{{
print total;
}}
}}
```

### 5.2 Pseudo Melayu Rasmi

Contoh ini mengekalkan maksud program yang sama menggunakan alias `primary_ms`. Semicolon dikekalkan supaya contoh ini mudah dibandingkan dengan expert mode dan masih jelas untuk beginner.

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
JIKA total >= limit MAKA
MULA
PAPAR limit;
TAMAT
MELAINKAN
MULA
PAPAR total;
TAMAT
TAMAT
```

### 5.3 Pseudo English Compatibility

Contoh ini menggunakan alias English/pseudocode yang disenaraikan dalam dictionary. Ia bukan reference mode, tetapi dipetakan kepada token canonical yang sama apabila alias tersebut wujud dalam `core_map.json`.

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
IF total >= limit THEN
START
PRINT limit;
END
ELSE
START
PRINT total;
END
END
```

### 5.4 Beginner Melayu dengan Newline Terminator

Di luar zon kritikal, `BINA`, `PAPAR`, dan `PULANG` boleh menggunakan newline sebagai terminator. Ini ialah ergonomic baseline untuk beginner tanpa mengubah strictness expert mode.

```logicodex
MULA
BINA x = 1
BINA y = 2
BINA total = x + y
PAPAR total
TAMAT
```

### 5.5 Hardware Zone yang Strict

Hardware I/O dan raw address syntax berada di bawah boundary policy yang lebih strict.[3] Block perlu eksplisit dan statement di dalam zon perlu semicolon walaupun menggunakan alias Melayu.

```logicodex
MULA
ZON_PERKAKASAN MULA
PERKAKASAN port: U16 = ALAMAT 65280;
BINA mirror: PTR<U16> = port;
PAPAR mirror;
TAMAT
TAMAT
```

## 6. Dictionary Token Lengkap

Jadual berikut dijana terus daripada `dict/core_map.json`.[1] Lajur **Expert canonical** ialah bentuk rujukan compiler; lajur **Primary Melayu** ialah alias manusia utama; dan lajur **Aliases** ialah ejaan tambahan yang diterima jika disenaraikan.

{token_table(tokens)}

## 7. Nota Had Semasa

Sebahagian token dalam dictionary ialah **vocabulary marker** untuk fasa seterusnya dan belum semestinya mempunyai rule parser lengkap dalam v1.21-alpha. Contohnya termasuk `while`, `break`, `continue`, `ffi`, `resource`, dan `drop`. Dokumen ini membezakan token yang wujud dalam dictionary daripada grammar yang benar-benar diparse dalam baseline semasa.

## 8. Cara Menjana Semula Dokumen

Dokumen ini boleh dijana semula selepas `dict/core_map.json` berubah dengan arahan berikut dari root repository:

```bash
python3 scripts/generate_grammar_and_dictionary_md.py
```

Selepas menjana semula, jalankan validator projek biasa supaya schema dictionary, parser, dan policy kekal konsisten:

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
