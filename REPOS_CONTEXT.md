# Logicodex Repository Context Document
This authoritative document inventories the core architectural assets of the Logicodex Language repository and establishes the operational context for each component under the current logicodex v 1.21 alpha milestone.

## 1. Compiler Core Frontend & Backend (`src/`)
- `src/main.rs`: The execution entry point. Houses the Clap CLI driver framework, manages compilation flags (`--target`, `--secure`), and prints the official terminal ASCII logo.
- `src/lexer.rs`: The dynamic dictionary tokenizer. Consumes raw `.ldx` files and queries `core_map.json` to substitute localized or shorthand words into uniform canonical token IDs.
- `src/parser.rs`: The structural AST builder. Utilizes a hand-rolled handwritten recursive-descent strategy and Pratt parsing engine to process token streams into strict compiler primitives.
- `src/semantic.rs`: The safety gatekeeper. Performs type inference checks, structural scoping constraints, constant-folding arithmetic validations, and filters programming hazards before lowering code.
- `src/codegen.rs`: The LLVM intermediate generator. Transpiles checked AST structures into LLVM IR while documenting future severity-handling insertion points.
- `src/target.rs`: The platform deployment matrix. Configures target profiles for hosted Windows/Linux and experimental freestanding work.

## 2. Operating System Native Bridges (`src/os/`)
- `src/os/windows.rs`: Implements native console output through the Windows Win32 API suite.
- `src/os/linux.rs`: Implements hyper-performance native outputs by executing raw x86_64 POSIX-compliant assembly Linux Syscalls, completely avoiding external standard C libraries dependencies.

## 3. Lexical Dictionaries & Code Reference (`dict/`, `examples/`)
- `dict/core_map.json`: The core dynamic mapping scheme. Houses the canonical dictionary that standardizes novice Malay pseudocode and expert shortcut semantics into identical primitives.
- `examples/`: Contains official functional validation files with the `.ldx` extension, demonstrating both localized verbose programming styles and advanced freestanding memory operations.

## 4. Documentation & Specifications (`spec/`, Root)
- `README.md`: The official Executive Summary manifesto outlining the dual-syntax thesis and project governance.
- `WHITE_PAPER.md`: The research white paper detailing the compiler pipeline, runtime attestation design direction, and long-term systems objectives.
- `ROADMAP.md`: The project management tracking center mapping open milestones, tracking tickets, and automated verification acceptance criteria.
- `spec/v1.11-alpha/UpdateIssue1-ebnf.md`: Houses the formalized 4-Layer grammar definition.
- `spec/v1.21-alpha/UpdateIssue2-provenance.md`: Houses the newly integrated Undefined Behavior layers and 3-tier error severity model.

## Practical Messaging Policy

Repository documentation should describe implemented compiler behavior as implemented, prototype behavior as experimental, and broader security or freestanding ambitions as long-term objectives. This keeps the project credible while preserving the full research direction for future milestones.
