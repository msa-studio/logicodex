# Logicodex Grammar Specification Baseline (v1.21-alpha)
Notation Legend: `::=` means "defined as"; `|` alternation; `*` zero-or-more; `+` one-or-more; `?` optional; terminals enclosed in quotes.

## Layer 1 — Surface Lexical Layer (core_map.json Input)
Identifier       ::= [a-zA-Z_] [a-zA-Z0-9_]*
LiteralInt       ::= [0-9]+
StringLiteral    ::= '"' [^"\\]* '"'

/* Lexical Mapping Targets from schema v2 */
BeginBlockToken  ::= "{" | "MULA" | "START" | "BEGIN"
EndBlockToken    ::= "}" | "TAMAT" | "END"
LetToken         ::= "let" | "BINA" | "CREATE"
PrintToken       ::= "print" | "PAPAR" | "PRINT"
IfToken          ::= "if" | "JIKA" | "IF"
ThenToken        ::= "then" | "MAKA" | "THEN"
ElseToken        ::= "else" | "MELAINKAN" | "ELSE" | "JIKALAU_TIDAK"
HardwareToken    ::= "hw" | "PERKAKASAN" | "HARDWARE" | "KAWASAN_PERKAKAS"
AddressToken     ::= "addr" | "ALAMAT" | "ADDRESS"
HardwareZoneToken::= "hw_unsafe" | "ZON_PERKAKASAN" | "HW_ZONE"
UseToken         ::= "use" | "GUNA" | "USE"
FunctionToken    ::= "fn" | "FUNGSI" | "FUNCTION"
ReturnToken      ::= "return" | "PULANG" | "RETURN"
AssignOp         ::= "="
AddOp            ::= "+"
SubOp            ::= "-"

## Layer 2 — Canonical Token Layer (Lexer Output / Parser Invariant)
The Logicodex parser operates on canonical token kinds emitted by the normalization lexer. The `expert` spellings in `core_map.json` are the compiler reference surface, while `primary_ms` and `aliases` are accepted human-facing spellings that are erased post-lexing.
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

| Token | Expert canonical shorthand | Primary Malay alias | English/pseudocode aliases | Status |
|---|---|---|---|---|
| `START` | `{` | `MULA` | `START`, `BEGIN` | Dictionary and lexer-recognition support; executable behavior depends on parser/semantic milestones where applicable. |
| `END` | `}` | `TAMAT` | `END` | Dictionary and lexer-recognition support; executable behavior depends on parser/semantic milestones where applicable. |
| `LET` | `let` | `BINA` | `CREATE` | Dictionary and lexer-recognition support; executable behavior depends on parser/semantic milestones where applicable. |
| `MUT` | `mut` | `MUTASI` | `MUTABLE` | Dictionary and lexer-recognition support; executable behavior depends on parser/semantic milestones where applicable. |
| `HW` | `hw` | `PERKAKASAN` | `HARDWARE`, `KAWASAN_PERKAKAS` | Dictionary and lexer-recognition support; executable behavior depends on parser/semantic milestones where applicable. |
| `ADDR` | `addr` | `ALAMAT` | `ADDRESS` | Dictionary and lexer-recognition support; executable behavior depends on parser/semantic milestones where applicable. |
| `IF` | `if` | `JIKA` | `IF` | Dictionary and lexer-recognition support; executable behavior depends on parser/semantic milestones where applicable. |
| `THEN` | `then` | `MAKA` | `THEN` | Dictionary and lexer-recognition support; executable behavior depends on parser/semantic milestones where applicable. |
| `ELSE` | `else` | `MELAINKAN` | `ELSE`, `JIKALAU_TIDAK` | Dictionary and lexer-recognition support; executable behavior depends on parser/semantic milestones where applicable. |
| `WHILE` | `while` | `SELAGI` | `WHILE` | Dictionary and lexer-recognition support; executable behavior depends on parser/semantic milestones where applicable. |
| `LOOP_BREAK` | `break` | `HENTI` | `BREAK` | Dictionary and lexer-recognition support; executable behavior depends on parser/semantic milestones where applicable. |
| `LOOP_CONTINUE` | `continue` | `TERUS` | `CONTINUE` | Dictionary and lexer-recognition support; executable behavior depends on parser/semantic milestones where applicable. |
| `FN` | `fn` | `FUNGSI` | `FUNCTION` | Dictionary and lexer-recognition support; executable behavior depends on parser/semantic milestones where applicable. |
| `RETURN` | `return` | `PULANG` | `RETURN` | Dictionary and lexer-recognition support; executable behavior depends on parser/semantic milestones where applicable. |
| `FFI` | `ffi` | `PAUTAN` | `FOREIGN_INTERFACE` | Dictionary and lexer-recognition support; executable behavior depends on parser/semantic milestones where applicable. |
| `C_INTEROP` | `c` | `C_LUAR` | `C_LEGACY` | Dictionary and lexer-recognition support; executable behavior depends on parser/semantic milestones where applicable. |
| `RESOURCE` | `resource` | `SUMBER` | `RESOURCE` | Dictionary and lexer-recognition support; executable behavior depends on parser/semantic milestones where applicable. |
| `DROP` | `drop` | `LEPAS` | `DROP` | Dictionary and lexer-recognition support; executable behavior depends on parser/semantic milestones where applicable. |
| `I32` | `i32` | `I32` | `INT32` | Dictionary and lexer-recognition support; executable behavior depends on parser/semantic milestones where applicable. |
| `U32` | `u32` | `U32` | `UINT32` | Dictionary and lexer-recognition support; executable behavior depends on parser/semantic milestones where applicable. |
| `STR` | `str` | `TEKS` | `STRING` | Dictionary and lexer-recognition support; executable behavior depends on parser/semantic milestones where applicable. |
| `BIT_AND` | `&` | `DAN_BIT` | `BIT_AND` | Dictionary and lexer-recognition support; executable behavior depends on parser/semantic milestones where applicable. |
| `BIT_OR` | `|` | `ATAU_BIT` | `BIT_OR` | Dictionary and lexer-recognition support; executable behavior depends on parser/semantic milestones where applicable. |
