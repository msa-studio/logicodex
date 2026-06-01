# Logicodex Examples — Grammar-Capability Matrix & Debug Strategy

> **Branch**: `feat/examples-grammar-debug`
> **Purpose**: Systematic examples that test ACTUAL parser/semantic capabilities
> **Based on**: Source code analysis of `src/parser.rs`, `src/lexer.rs`, `src/semantic.rs`

---

## Quick Start

```bash
# Level 0 (sanity check — simplest possible)
logicodex check examples/00_sanity.ldx

# Level 1 (basic constructs)
logicodex check examples/01a_variables.ldx
logicodex check examples/01b_arithmetic.ldx

# Level 5 (comprehensive stress test)
logicodex check examples/05_comprehensive.ldx
```

---

## Grammar Capability Matrix

Based on ACTUAL source code analysis (not documentation):

### ✅ FULLY WORKING (Parser + Semantic)

| Feature | Malay | Expert | File |
|---------|-------|--------|------|
| Print | `PAPAR` | `print` | 00, 01a |
| Variable decl | `BINA :Type =` | `let :Type =` | 01a |
| Assignment | `x = 10` | `x = 10` | 01a |
| Integer literal | `42` | `42` | 00 |
| Boolean literal | `BENAR`/`SALAH` | `true`/`false` | 01c |
| Arithmetic + - * / | (symbols) | + - * / | 01b |
| Parentheses grouping | `(...)` | `(...)` | 01b |
| Comparison == != < > <= >= | (symbols) | == != < > <= >= | 01c |
| Logical &&/\|\| | `dan`/`atau` | `&&`/\|\| | 01c |
| If-then-else | `JIKA/MAKA/MELAINKAN` | `if/then/else` | 02a |
| While loop | `SELAGI` | `while` | 02b |
| Infinite loop | `ULANG` | `loop` | 02c |
| Break | `HENTI` | `break` | 02c |
| Continue | `TERUS` | `continue` | 02c |
| Function def | `FUNGSI MULA/TAMAT` | `fn {}` | 03a |
| Function call | `name(args)` | `name(args)` | 03a |
| Return | `PULANG` | `return` | 03a |
| Match Result | `MATCH/OK/ERR/_` | `match/Ok/Err/_` | 04a |
| Bitwise & \| << >> | `bit_dan/atau/anjak_*` | & \| << >> | 04b |
| Hardware zone | `ZON_PERKAKASAN` | `hw_unsafe` | 04c |

### ❌ NOT WORKING (Despite AST support or documentation)

| Feature | Status | Why It Fails |
|---------|--------|--------------|
| Unary minus (`-x`) | ❌ NOT PARSED | Parser has no rule for `Unary` in `parse_primary_expr` |
| Logical NOT (`!x`) | ❌ NOT PARSED | Same as above |
| For loop | ❌ NOT A TOKEN | `for` doesn't exist in TokenKind enum |
| `mut` keyword | ❌ IGNORED | Lexed as `TokenKind::Mut` but parser discards it |
| String type `:str` | ❌ NOT IN PARSER | `parse_type()` has no `TypeStr` branch |
| Struct | ❌ REJECTED | Parser explicitly rejects with error message |
| Enum | ❌ REJECTED | Parser explicitly rejects |
| `unsafe` block | ❌ REJECTED | Parser explicitly rejects |
| `extern` block | ❌ REJECTED | Parser explicitly rejects |
| Match literal pattern | ❌ NOT PARSED | Only `Ok(v)`, `Err(e)`, `_` work |

### ⚠️ ALIASES CONFUSION (Dictionary vs Hardcoded)

| Issue | Impact |
|-------|--------|
| 42 keywords in lexer but NOT in dict/core_map.json | Malay aliases may not work without dict file |
| `SHL`/`SHR` only in dict, not hardcoded | `anjak_kiri` works, `SHL` fails if dict missing |
| Case sensitivity | `BINA` works, `bina` does NOT work |
| Dictionary version mismatch | File says v1.21, code says v1.30/v1.45 |

---

## Debug Strategy: When `logicodex check` Fails

### Step 1: Identify Which Pass Fails

```bash
# Check lexing (does it produce tokens?)
# If you see "unexpected token" or "unknown identifier":
#   → Lexer issue or keyword not recognized

# Check parsing (does parser accept the sequence?)
# If you see "expected X, found Y":
#   → Parser grammar mismatch — feature not implemented

# Check semantic (does it pass validation?)
# If you see "semantic error" or "type mismatch":
#   → Semantic analyzer rejection — check type rules
```

### Step 2: Common Failures and Fixes

| Error Message | Likely Cause | Fix |
|---------------|--------------|-----|
| `unknown identifier 'BINA'` | Dictionary not loaded | Ensure `dict/core_map.json` exists |
| `expected identifier, found '{'` | Using `{` with Malay keywords | Malay uses `MULA`/`TAMAT`, not `{}` |
| `expected '{', found 'MULA'` | Mixing Expert/Malay syntax | Don't mix — use Expert `{}` OR Malay `MULA/TAMAT` |
| `type mismatch` | Wrong type annotation | Ensure `:Type` matches literal (I32=5, F64=3.14) |
| `unary operator not supported` | Using `-x` or `!x` | Use `(0 - x)` instead of `-x` |
| `expected 'while', found 'for'` | For loop not implemented | Use `while` with manual counter |
| `struct is not supported in v1.21` | Parser rejects | Remove struct usage |

### Step 3: Isolate the Problem

```bash
# Start from Level 0 and go up:
logicodex check examples/00_sanity.ldx        # Should ALWAYS work
logicodex check examples/01a_variables.ldx    # Tests let + types
logicodex check examples/02a_if_then_else.ldx # Tests control flow
logicodex check examples/03a_functions.ldx    # Tests functions
# ...and so on. First failure tells you which grammar is broken.
```

---

## File Index

| File | Level | What It Tests |
|------|-------|---------------|
| `00_sanity.ldx` | 0 | Minimal valid program |
| `01a_variables.ldx` | 1 | Variable decl, types, assignment |
| `01b_arithmetic.ldx` | 1 | + - * /, precedence, parentheses |
| `01c_boolean.ldx` | 1 | Bool literals, comparisons, &&/\|\| |
| `02a_if_then_else.ldx` | 2 | Conditional branching |
| `02b_while_loop.ldx` | 2 | Counter loop |
| `02c_loop_break_continue.ldx` | 2 | Infinite loop + break/continue |
| `03a_functions.ldx` | 3 | Function def, params, call, return |
| `03b_recursion.ldx` | 3 | Recursive function |
| `04a_match_result.ldx` | 4 | Match expression with Result |
| `04b_bitwise.ldx` | 4 | &, \|, <<, >> with aliases |
| `04c_hardware.ldx` | 4 | Hardware unsafe zone |
| `05_comprehensive.ldx` | 5 | Full pipeline stress test |

---

*Generated from source code analysis — not from documentation*
