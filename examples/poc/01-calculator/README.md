# POC-01: Dual-Syntax Calculator

This POC proves that Logicodex produces **identical LLVM IR** regardless of
whether the source uses Malay or Expert syntax. Same semantics, same output.

## Files

| File | Description |
|------|-------------|
| `calc_malay.ldx` | Full calculator in Malay syntax |
| `calc_expert.ldx` | Identical logic in Expert canonical syntax |
| `compare.sh` | Automated comparison script |

## What It Tests

- **Variables**: `BINA` / `let` declarations with initialization
- **Arithmetic**: `+`, `-`, `*`, `/` on I32 values
- **Functions**: Parameter passing and return values
- **If/Else chains**: `JIKA/MAKA/MELAINKAN` / `if/else`
- **While loops**: Counting, condition-based iteration
- **Break**: `HENTI` / `break` for early loop exit
- **Nested blocks**: Inner scope variable visibility
- **Factorial calculation**: Recursive concept via while loop
- **Prime checking**: Algorithm using modulo and nested conditions

## Quick Start

### Manual compilation and comparison:

```bash
# Step 1: Compile Malay version to LLVM IR
logicodex compile --emit=llvm-ir calc_malay.ldx -o calc_malay.ll

# Step 2: Compile Expert version to LLVM IR
logicodex compile --emit=llvm-ir calc_expert.ldx -o calc_expert.ll

# Step 3: Diff the IR outputs
diff -u calc_malay.ll calc_expert.ll

# Expected: NO OUTPUT (files are identical)
```

### Automated (run the comparison script):

```bash
chmod +x compare.sh
./compare.sh
```

## Expected Output

When run, the script prints:

```
========================================
  POC-01: Dual-Syntax Calculator
========================================
Compiling Malay version...
Compiling Expert version...
Diffing LLVM IR outputs...
========================================
  SUCCESS: Identical LLVM IR!
========================================
Both syntaxes compile to the exact same IR.
This proves the parser normalizes correctly.
```

## What Identical IR Proves

1. **Parser normalization**: Both frontends parse to the same AST
2. **Semantic equivalence**: Malay keywords are syntactic sugar only
3. **CodeGen consistency**: One backend handles both syntaxes
4. **No runtime penalty**: Zero cost abstraction for either syntax

## Expected Program Output (both versions)

When compiled and executed, both programs produce the same console output:

```
15
30
1
29
120
true
3
20
23
6
55
64
100
50
50
```
