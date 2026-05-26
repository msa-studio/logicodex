# POC-02: Grammar Conformance

This POC validates that the Logicodex compiler accepts **every grammar rule**
in the v1.21 language specification. The `conformance.ldx` file is a single
program that exercises every construct.

## Files

| File | Description |
|------|-------------|
| `conformance.ldx` | Single file testing all grammar rules |
| `test_runner.sh` | Automated test runner with per-rule reporting |
| `README.md` | This file |

## Grammar Coverage Checklist

### Statements

| # | Grammar Rule | Status | Line(s) in conformance.ldx |
|---|-------------|--------|---------------------------|
| 1 | `variable_declaration` (BINA/let) | :white_check_mark: Tested | 17-37 |
| 2 | `print_statement` (PAPAR/print) | :white_check_mark: Tested | 43-47 |
| 3 | `return_statement` (PULANG/return) | :white_check_mark: Tested | 237-245 |
| 4 | `if_statement` (JIKA MAKA/if) | :white_check_mark: Tested | 89-97 |
| 5 | `if_else_statement` (JIKA ... MELAINKAN/if ... else) | :white_check_mark: Tested | 103-111 |
| 6 | `if_elseif_statement` (JIKA ... SEBALIKNYA JIKA/if ... else if) | :white_check_mark: Tested | 117-132 |
| 7 | `while_statement` (SELAGI ... DAN benar/while) | :white_check_mark: Tested | 138-146 |
| 8 | `while_true_loop` (SELAGI benar DAN benar/while true) | :white_check_mark: Tested | 152-163 |
| 9 | `break_statement` (HENTI/break) | :white_check_mark: Tested | 169-178 |
| 10 | `continue_statement` (TERUS/continue) | :white_check_mark: Tested | 184-196 |

### Types

| # | Type | Status | Example Declaration |
|---|------|--------|-------------------|
| 11 | `I32` (32-bit signed int) | :white_check_mark: Tested | `BINA x = 42;` |
| 12 | `I64` (64-bit signed int) | :white_check_mark: Tested | `BINA y = 10000000000;` |
| 13 | `F32` (32-bit float) | :white_check_mark: Tested | `BINA z = 3.14;` |
| 14 | `F64` (64-bit double) | :white_check_mark: Tested | `BINA w = 2.718;` |
| 15 | `Bool` (boolean) | :white_check_mark: Tested | `BINA b = benar;` |

### Operators

| # | Operator | Category | Status | Line(s) |
|---|----------|----------|--------|---------|
| 16 | `+` | Arithmetic | :white_check_mark: Tested | 55 |
| 17 | `-` | Arithmetic | :white_check_mark: Tested | 56 |
| 18 | `*` | Arithmetic | :white_check_mark: Tested | 57 |
| 19 | `/` | Arithmetic | :white_check_mark: Tested | 58 |
| 20 | `%` | Arithmetic | :white_check_mark: Tested | 59 |
| 21 | `<` | Comparison | :white_check_mark: Tested | 73 |
| 22 | `>` | Comparison | :white_check_mark: Tested | 74 |
| 23 | `==` | Comparison | :white_check_mark: Tested | 75 |
| 24 | `!=` | Comparison | :white_check_mark: Tested | 76 |
| 25 | `<=` | Comparison | :white_check_mark: Tested | 77 |
| 26 | `>=` | Comparison | :white_check_mark: Tested | 78 |
| 27 | `&` | Bitwise AND | :white_check_mark: Tested | 90 |
| 28 | `\|` | Bitwise OR | :white_check_mark: Tested | 91 |
| 29 | `^` | Bitwise XOR | :white_check_mark: Tested | 92 |
| 30 | `<<` | Bitwise shift left | :white_check_mark: Tested | 93 |
| 31 | `>>` | Bitwise shift right | :white_check_mark: Tested | 94 |
| 32 | `~` | Unary NOT | :white_check_mark: Tested | 101 |

### Literals

| # | Literal | Status | Line(s) |
|---|---------|--------|---------|
| 33 | Integer positive | :white_check_mark: Tested | `BINA x = 42;` |
| 34 | Integer negative | :white_check_mark: Tested | `BINA x = -7;` |
| 35 | Integer zero | :white_check_mark: Tested | `BINA x = 0;` |
| 36 | Float | :white_check_mark: Tested | `BINA x = 3.14;` |
| 37 | Boolean `benar` / `true` | :white_check_mark: Tested | 107-108 |
| 38 | Boolean `palsu` / `false` | :white_check_mark: Tested | 107-108 |

### Functions

| # | Feature | Status | Line(s) |
|---|---------|--------|---------|
| 39 | Function definition (no params) | :white_check_mark: Tested | 221-224 |
| 40 | Function definition (single param) | :white_check_mark: Tested | 226-229 |
| 41 | Function definition (multi param) | :white_check_mark: Tested | 231-234 |
| 42 | Function definition (returns Bool) | :white_check_mark: Tested | 236-239 |
| 43 | Function definition (returns I64) | :white_check_mark: Tested | 241-244 |
| 44 | Function call (no args) | :white_check_mark: Tested | 253 |
| 45 | Function call (single arg) | :white_check_mark: Tested | 256 |
| 46 | Function call (multi args) | :white_check_mark: Tested | 259 |
| 47 | Function call (in expression) | :white_check_mark: Tested | 300 |
| 48 | Nested function calls | :white_check_mark: Tested | 304 |
| 49 | Early return | :white_check_mark: Tested | 246-251 |

### Advanced Constructs

| # | Feature | Status | Line(s) |
|---|---------|--------|---------|
| 50 | Nested blocks | :white_check_mark: Tested | 265-280 |
| 51 | Compound assignment | :white_check_mark: Tested | 324-330 |
| 52 | Operator precedence | :white_check_mark: Tested | 310-320 |
| 53 | All types in one function | :white_check_mark: Tested | 339-352 |
| 54 | Function + while combination | :white_check_mark: Tested | 289-299 |

## Quick Start

### Compile conformance test:

```bash
# Compile the conformance file
logicodex compile conformance.ldx -o conformance

# Run the binary
./conformance
```

### Run the test runner:

```bash
chmod +x test_runner.sh
./test_runner.sh
```

## Success Criteria

The conformance test PASSES if:

1. `logicodex compile conformance.ldx` exits with code 0
2. The generated binary runs without crashing
3. Output matches expected values (documented in comments)

The test FAILS if:

1. Parser rejects any valid construct
2. Type checker reports false errors
3. Code generation produces invalid output

## Expected Program Output

```
42
true
false
3
-1
true
false
false
true
true
false
1
7
6
8
2
-1
true
false
100
201
3
10
10
300
12
5
14
7
false
25
0
1
2
3
1
55
36
14
7
20
4
42
100000
1.5
2.5
true
42
```

## Notes

- Every grammar construct appears at least once
- Comments on every line document the expected behavior
- Both Malay and Expert keywords are used (Malay in source, Expert in comments)
- The program is designed to compile as a single unit, not as isolated tests
