> ⚠️ **NOT UPDATED — will revisit.** This document predates the current syntax/architecture and may contain stale information. Authoritative current references: `README.md`, `examples/`, and `docs/architecture/`. Tracked under `docs/DOCUMENTATION_POLICY.md`.

# Logicodex v1.21 Syntax Analysis

## Executive Summary

### Is the grammar strict or loose?
**The grammar is INCONSISTENT — it has two different modes that don't compose cleanly:**
1. **Expert/Canonical mode**: Requires semicolons (`;`) and curly braces `{}`
2. **Beginner/Malay mode**: Uses `MULA`/`TAMAT` blocks and allows newlines as statement terminators

### Root Cause of Example Failures
The CI examples were failing because:
1. **Mixed-mode syntax**: Using Malay keywords (`BINA`, `PAPAR`) inside expert braces `{}`
2. **Beginner mode only works at `critical_depth == 0`**: Inside nested blocks, semicolons become mandatory
3. **Hardware zone**: `critical_depth += 1` inside ZON_PERKAKASAN disables beginner terminators
4. **Silent errors**: `cargo run --quiet` suppressed error messages

---

## How Beginner Mode Works (Malay Aliases)

### `allows_beginner_line_terminator()` — line 1044
```rust
self.critical_depth == 0 && matches!(lexeme, "BINA" | "CREATE" | "PAPAR" | "PULANG")
```
**Rules:**
- Must be at top level (`critical_depth == 0`)
- Keyword must be `BINA`/`CREATE` (let), `PAPAR` (print), `PULANG` (return)
- When active, newline/TAMAT/EOF counts as `;`
- When NOT active, semicolon is mandatory

### `critical_depth` increments only in:
- `hardware_zone_block()` — line 267

### `consume_statement_terminator()` — line 1020
```rust
fn consume_statement_terminator(&mut self, expected: &str, allow_newline: bool) {
    if matches(Semicolon) -> OK
    if allow_newline && (Newline || End || Else || Eof) -> OK
    else -> ERROR: expected "; after ..."
}
```

---

## Syntax Modes

### Mode 1: Expert/Canonical (Strict)
```
{                           // Start block
    let x = 5;             // Semicolon required
    print x;               // Semicolon required
    if x > 3 {             // Curly braces
        print 1;           // Semicolon required
    } else {               // Curly braces
        print 0;           // Semicolon required
    }
}
```

### Mode 2: Beginner/Malay (Lenient)
```
MULA                        // Start block (critical_depth == 0)
BINA x = 5                  // No semicolon needed
PAPAR x                     // No semicolon needed
JIKA x > 3 MAKA             // MAKA = then (optional)
MULA
PAPAR 1                     // No semicolon needed
TAMAT
MELAINKAN                   // else
MULA
PAPAR 0                     // No semicolon needed
TAMAT
TAMAT                       // End block
```

### What DOES NOT Work (Mixed Mode — the failure pattern):
```
MULA
BINA x = 5                  // OK — beginner mode, no semicolon
PAPAR x                     // OK — beginner mode, no semicolon
JIKA x > 3 MAKA
MULA                        // Block starts → still critical_depth == 0
PAPAR 1                     // Inside block, still beginner mode — OK
TAMAT
MELAINKAN
MULA
PAPAR 0                     // OK
TAMAT
BINA y = 1                  // PROBLEM: this MIGHT fail in some contexts
TAMAT
```

Wait — let me re-check. The `if_statement()` calls `self.block()` which calls `self.block()`:

```rust
fn block(&mut self) -> Result<Vec<Stmt>, ParseError> {
    self.consume(TokenKind::Start, "block start MULA or {")?;  // Consumes MULA
    // ...
    self.consume(TokenKind::End, "block end TAMAT or }")?;     // Consumes TAMAT
}
```

The `if_statement()`:
```rust
fn if_statement(&mut self) -> Result<Stmt, ParseError> {
    let condition = self.expression()?;
    self.matches(TokenKind::Then);    // Optional MAKA
    self.consume_newlines();           // Skip newlines
    let then_branch = self.block()?;   // Expects MULA...TAMAT
    self.consume_newlines();
    let else_branch = if self.matches(TokenKind::Else) {  // MELAINKAN
        self.consume_newlines();
        self.block()?                  // Expects MULA...TAMAT
    } else {
        Vec::new()
    };
    // No final TAMAT consumed here — returns to parse()
}
```

So the structure is:
```
MULA                          // parse() Start block
BINA x = 5                    // statement
JIKA x > 3 MAKA               // if statement
MULA                          // then_branch block start
PAPAR 1                       // statement inside then
TAMAT                         // then_branch block end
MELAINKAN                     // else
MULA                          // else_branch block start
PAPAR 0                       // statement inside else
TAMAT                         // else_branch block end
TAMAT                         // parse() End block
```

This should work because `critical_depth` is only incremented in `hardware_zone_block()`, NOT in `if_statement()` or `block()`.

---

## What Actually Fails (Root Cause Analysis)

Let me trace through `tambah.ldx` (which has FAIL in CI):

```
MULA
BINA x = 5
JIKA x > 0 MAKA
MULA
PAPAR x
TAMAT
MELAINKAN
MULA
PAPAR 0
TAMAT
TAMAT
```

### Step-by-step trace:

1. **MULA** → `parse()` sees `Start` token, sets `wrapped = true`
2. **BINA x = 5** → `declaration_or_statement()`:
   - Not Use/Hardware/HwZone/Fn
   - Not Struct/Enum/Unsafe/Extern
   - Falls through to `statement()`
   - `matches(TokenKind::Let)` → BINA is NOT Let
   - `matches(TokenKind::Print)` → BINA is NOT Print
   - ...all the way down...
   - Falls through to `else` branch: `let value = self.expression()`
   - **BINA as expression** → tries to parse `BINA` as identifier!
   - `x` = identifier → variable reference
   - `=` → assignment? No, it's not a statement yet
   
**THIS IS THE BUG!** 

### The Lexicon Problem

Looking at the lexicon registration:
```rust
("BINA", TokenKind::Let),        // line ~
("BINA", TokenKind::Beginner),   // Wait — what is Beginner token kind?
```

Actually let me check more carefully...

The lexicon has:
```rust
("let", TokenKind::Let),
("LET", TokenKind::Let),
("Let", TokenKind::Let),
("Create", TokenKind::Let),
("CREATE", TokenKind::Let),
("BinA", TokenKind::Let),
("BINA", TokenKind::Let),
("bina", TokenKind::Let),
```

So `BINA` DOES map to `TokenKind::Let`! That means:
```rust
statement() {
    if self.matches(TokenKind::Let) {   // BINA matches Let
        let beginner = self.allows_beginner_line_terminator(&"BINA");
        // returns true (critical_depth == 0)
        self.let_statement(true)?         // beginner = true
    }
}
```

Then `let_statement(true)`:
```rust
fn let_statement(&mut self, beginner: bool) -> Result<Stmt, ParseError> {
    let name = self.consume(TokenKind::Identifier, "variable name")?.lexeme.clone();
    // Consumes "x" → Identifier ✓
    let declared_type = if self.matches(TokenKind::Colon) { ... } else { None };
    // No colon, type = None ✓
    self.consume(TokenKind::Assign, "=")?;
    // Consumes "=" ✓
    let value = self.expression()?;
    // Parses "5" → Integer literal ✓
    self.consume_statement_terminator("; after let statement", beginner);
    // beginner = true, so:
    //   if matches(Semicolon) → no semicolon
    //   if allow_newline && (Newline || End || Else || Eof) → YES! Newline
    //   → Consumes newline ✓
    //   → Returns Ok(())
    Ok(Stmt::Let { name: "x", declared_type: None, value: Integer(5) })
}
```

This should work! Let me trace the rest...

3. **PAPAR x** → `statement()`:
   - `matches(TokenKind::Print)` → PAPAR maps to Print ✓
   - `beginner = allows_beginner_line_terminator("PAPAR")` → true ✓
   - `print_statement(true)`:
     - Parses `x` as expression ✓
     - `consume_statement_terminator("; after print statement", true)`
     - Newline after x → OK ✓

4. **JIKA x > 0 MAKA** → `statement()`:
   - `matches(TokenKind::If)` → JIKA maps to If ✓
   - `if_statement()`:
     - Parses `x > 0` as expression ✓
     - `matches(TokenKind::Then)` → MAKA maps to Then ✓
     - `consume_newlines()`
     - `block()`: expects `MULA` ✓

5. **MULA** (inside if) → `block()`:
   - `consume(Start, "block start MULA or {")` ✓
   - Inside block: `PAPAR x`
   - `matches(Print)` → PAPAR ✓
   - `print_statement(true)` → beginner mode ✓
   - `consume_statement_terminator` → Newline after → OK ✓
   - **HEN** → Wait, what's HEN? Let me check the file again...

Let me re-read the actual file:
