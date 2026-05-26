# POC-03: Native Binary Generation

This POC demonstrates the complete pipeline from Logicodex source code to a
native ELF binary that runs without any Logicodex runtime dependency.

## What It Proves

1. **CodeGen produces valid LLVM IR**: The compiler generates correct `.ll` files
2. **Object file generation**: LLVM IR compiles to native `.o` object files
3. **Linker compatibility**: Standard tools (`ld`, `clang`) can link Logicodex objects
4. **OS ABI conformance**: The binary follows the platform calling convention
5. **Zero runtime dependency**: The output is a standalone native executable

## Files

| File | Description |
|------|-------------|
| `hello_native.ldx` | Source program with both Malay and Expert syntax |
| `build.sh` | Automated build-and-test script |
| `README.md` | This file |

## The Program

`hello_native.ldx` defines a `main` function (`utama()`) that:
1. Declares a variable `nombor` with value `42`
2. Prints it to stdout via the `print` statement
3. Returns `0` (success) to the OS

The file includes both Malay and Expert syntax versions. Only one should be
active at a time (the other is commented out).

## Manual Compilation Steps

### Step 1: Compile Logicodex to LLVM IR

```bash
logicodex compile --emit=llvm-ir hello_native.ldx -o hello_native.ll
```

This produces an LLVM IR file containing the low-level representation
of your program. You can inspect it:

```bash
cat hello_native.ll
```

Expected content (conceptual):
```llvm
; ModuleID = 'hello_native.ldx'
@fmt_i32 = private unnamed_addr constant [4 x i8] c"%d\0A\00"

define i32 @main() {
entry:
  %nombor = alloca i32
  store i32 42, i32* %nombor
  %v = load i32, i32* %nombor
  call i32 (i8*, ...) @printf(i8* getelementptr ([4 x i8], [4 x i8]* @fmt_i32, i64 0, i64 0), i32 %v)
  ret i32 0
}

declare i32 @printf(i8*, ...)
```

### Step 2: Compile LLVM IR to Object File

Using `llc` (LLVM compiler):

```bash
llc -filetype=obj hello_native.ll -o hello_native.o
```

Or directly from Logicodex:

```bash
logicodex compile hello_native.ldx -o hello_native.o
```

This produces a native object file containing machine code for your platform.

### Step 3: Link Object File to Binary

Using `clang` (recommended):

```bash
clang hello_native.o -o hello_native
```

Using `ld` (explicit):

```bash
ld hello_native.o -o hello_native -lc -dynamic-linker /lib64/ld-linux-x86-64.so.2
```

Using `gcc`:

```bash
gcc hello_native.o -o hello_native
```

### Step 4: Verify the Binary

```bash
# Check file type
file hello_native
# Expected: ELF 64-bit LSB executable, x86-64, version 1 (SYSV), dynamically linked

# Check it runs
./hello_native
# Expected output: 42

# Check exit code
echo $?
# Expected: 0
```

## Automated Build

Run the provided script for one-command build and test:

```bash
chmod +x build.sh
./build.sh
```

## Expected Output

```
========================================
  POC-03: Native Binary Generation
========================================
[Step 1/5] Compiling to LLVM IR...  OK
[Step 2/5] Compiling to object...   OK
[Step 3/5] Linking to binary...     OK
[Step 4/5] Running binary...        OK
  Output: 42
[Step 5/5] Verifying exit code...   OK
  Exit code: 0
========================================
  SUCCESS: Native binary works!
========================================
The Logicodex compiler produced a working
native ELF binary with zero dependencies.
POC-03 PASSED
```

## Cross-Platform Notes

| Platform | Object Format | Linker Command |
|----------|--------------|----------------|
| Linux x86_64 | ELF64 | `clang hello.o -o hello` |
| Linux ARM64 | ELF64 | `clang hello.o -o hello` |
| macOS x86_64 | Mach-O | `clang hello.o -o hello` |
| macOS ARM64 | Mach-O | `clang hello.o -o hello` |
| Windows x64 | COFF | `lld-link hello.obj /OUT:hello.exe` |

The Logicodex compiler targets LLVM IR, which is platform-neutral.
The platform-specific compilation happens at the LLVM/MC layer.

## Binary Analysis

After building, you can analyze the binary:

```bash
# Disassemble
objdump -d hello_native

# List symbols
nm hello_native

# Check dynamic dependencies
ldd hello_native

# Size breakdown
size hello_native
```

## Requirements

- `logicodex` compiler in PATH
- `clang` or `gcc` for linking (or `ld` with correct flags)
- Target platform: Linux x86_64 (others supported via cross-compilation)
