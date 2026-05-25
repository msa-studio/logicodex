---
name: Bug Report
about: Report a compiler bug or unexpected behavior
title: '[BUG] '
labels: bug
assignees: ''
---

## Bug Description

A clear description of the bug.

## Environment

- **Logicodex version:** `logicodex --version` output
- **Target:** Native / WASM / Freestanding (which architecture?)
- **OS:** Linux / macOS / Windows (and version)
- **LLVM version:** `llvm-config --version`

## Minimal Reproduction

A minimal `.ldx` file that reproduces the bug:

```logicodex
program reproduce {
    fn main() -> I32 {
        // Your code here
        return 0;
    }
}
```

## Expected Behavior

What you expected to happen.

## Actual Behavior

What actually happened (include full error output if applicable).

## Additional Context

Any other information that might be helpful.
