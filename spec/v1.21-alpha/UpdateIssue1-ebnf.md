# Logicodex Grammar Specification Baseline (v1.21-alpha)
Notation Legend: `::=` means "defined as"; `|` alternation; `*` zero-or-more; `+` one-or-more; `?` optional; terminals enclosed in quotes.

## Layer 1 — Surface Lexical Layer (core_map.json Input)
Identifier       ::= [a-zA-Z_] [a-zA-Z0-9_]*
LiteralInt       ::= [0-9]+
StringLiteral    ::= '"' [^"\\]* '"'

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

## Layer 2 — Canonical Token Layer (Lexer Output / Parser Invariant)
The Logicodex parser operates exclusively on canonical token kinds emitted by the normalization lexer. Surface spellings are erased post-lexing.
BeginBlock ::= TokenKind::BeginBlock
EndBlock   ::= TokenKind::EndBlock

## Layer 3 — Syntactic Grammar Layer (Current AST Contract)
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

## Layer 4 — Semantic Constraint Layer
Grammar validation establishes structural correctness for the implemented language subset. Type compilation and unsafe capability checks are handled by the separate Static Analysis phase and should continue to expand through tested milestones.

## Three-Tier Token Catalog Addendum

The current logicodex v 1.21 alpha dictionary uses the following three-tier token records. This table documents token vocabulary support and must not be read as proof that every listed token already has full parser, semantic, backend, or runtime behavior.

| Token | Canonical Malay | Expert shorthand | English pseudocode | Status |
|---|---|---|---|---|
| `TOKEN_START` | `MULA` | `{` | `START` | Dictionary and lexer-recognition support; executable behavior depends on parser/semantic milestones where applicable. |
| `TOKEN_END` | `TAMAT` | `}` | `END` | Dictionary and lexer-recognition support; executable behavior depends on parser/semantic milestones where applicable. |
| `TOKEN_LET` | `BINA` | `let` | `CREATE` | Dictionary and lexer-recognition support; executable behavior depends on parser/semantic milestones where applicable. |
| `TOKEN_MUT` | `MUTASI` | `mut` | `MUTABLE` | Dictionary and lexer-recognition support; executable behavior depends on parser/semantic milestones where applicable. |
| `TOKEN_HW` | `PERKAKASAN` | `hw` | `HARDWARE` | Dictionary and lexer-recognition support; executable behavior depends on parser/semantic milestones where applicable. |
| `TOKEN_ADDR` | `ALAMAT` | `addr` | `ADDRESS` | Dictionary and lexer-recognition support; executable behavior depends on parser/semantic milestones where applicable. |
| `TOKEN_IF` | `JIKA` | `if` | `IF` | Dictionary and lexer-recognition support; executable behavior depends on parser/semantic milestones where applicable. |
| `TOKEN_THEN` | `MAKA` | `then` | `THEN` | Dictionary and lexer-recognition support; executable behavior depends on parser/semantic milestones where applicable. |
| `TOKEN_ELSE` | `MELAINKAN` | `else` | `ELSE` | Dictionary and lexer-recognition support; executable behavior depends on parser/semantic milestones where applicable. |
| `TOKEN_WHILE` | `SELAGI` | `while` | `WHILE` | Dictionary and lexer-recognition support; executable behavior depends on parser/semantic milestones where applicable. |
| `TOKEN_LOOP_BREAK` | `HENTI` | `break` | `BREAK` | Dictionary and lexer-recognition support; executable behavior depends on parser/semantic milestones where applicable. |
| `TOKEN_LOOP_CONTINUE` | `TERUS` | `continue` | `CONTINUE` | Dictionary and lexer-recognition support; executable behavior depends on parser/semantic milestones where applicable. |
| `TOKEN_FN` | `FUNGSI` | `fn` | `FUNCTION` | Dictionary and lexer-recognition support; executable behavior depends on parser/semantic milestones where applicable. |
| `TOKEN_RETURN` | `PULANG` | `return` | `RETURN` | Dictionary and lexer-recognition support; executable behavior depends on parser/semantic milestones where applicable. |
| `TOKEN_FFI` | `PAUTAN` | `ffi` | `FOREIGN_INTERFACE` | Dictionary and lexer-recognition support; executable behavior depends on parser/semantic milestones where applicable. |
| `TOKEN_C_INTEROP` | `C_LUAR` | `c` | `C_LEGACY` | Dictionary and lexer-recognition support; executable behavior depends on parser/semantic milestones where applicable. |
| `TOKEN_RESOURCE` | `SUMBER` | `resource` | `RESOURCE` | Dictionary and lexer-recognition support; executable behavior depends on parser/semantic milestones where applicable. |
| `TOKEN_DROP` | `LEPAS` | `drop` | `DROP` | Dictionary and lexer-recognition support; executable behavior depends on parser/semantic milestones where applicable. |
| `TOKEN_I32` | `I32` | `i32` | `INT32` | Dictionary and lexer-recognition support; executable behavior depends on parser/semantic milestones where applicable. |
| `TOKEN_U32` | `U32` | `u32` | `UINT32` | Dictionary and lexer-recognition support; executable behavior depends on parser/semantic milestones where applicable. |
| `TOKEN_STR` | `TEKS` | `str` | `STRING` | Dictionary and lexer-recognition support; executable behavior depends on parser/semantic milestones where applicable. |
| `TOKEN_BIT_AND` | `DAN_BIT` | `&` | `BIT_AND` | Dictionary and lexer-recognition support; executable behavior depends on parser/semantic milestones where applicable. |
| `TOKEN_BIT_OR` | `ATAU_BIT` | `|` | `BIT_OR` | Dictionary and lexer-recognition support; executable behavior depends on parser/semantic milestones where applicable. |
