# ❖ Logicodex Formal Grammar Specification (v1.11-alpha)

Notation Legend: `::=` means "defined as"; `|` alternation; `*` zero-or-more; `+` one-or-more; `?` optional; terminals enclosed in quotes.

## Layer 1 — Surface Lexical Layer (core_map.json Input)

```ebnf
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
```

## Layer 2 — Canonical Token Layer (Lexer Output / Parser Invariant)

The Logicodex parser operates exclusively on canonical token kinds emitted by the normalization lexer. Surface spellings are erased post-lexing.

```ebnf
BeginBlock ::= TokenKind::BeginBlock
EndBlock   ::= TokenKind::EndBlock
```

## Layer 3 — Syntactic Grammar Layer (AST Structural Contract)

```ebnf
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
```

## Layer 4 — Semantic Constraint Layer

Grammar validation guarantees structural correctness. Type compilation and unsafe capability checks (e.g., restricting raw hardware pointer addressing to freestanding configurations) are strictly enforced during the separate Static Analysis phase.
