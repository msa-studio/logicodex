from pathlib import Path

root = Path('/home/ubuntu/logicodex')
files = {}

files['README.md'] = r'''# Logicodex Language

**Logicodex** is a Phase 1 MVP programming language compiler designed to close the practical gap between novice-friendly pseudocode and expert low-level systems programming. The language accepts two surface syntaxes that are intentionally different in appearance but identical in meaning. Both syntaxes are mapped through a dynamic dictionary into the same token identities, parsed into the same Abstract Syntax Tree, validated by the same static semantic analyzer, and lowered to native machine code through LLVM.

The first syntax is a Malay-inspired novice syntax using words such as **MULA**, **TAMAT**, **BINA**, **JIKA**, **MAKA**, **JIKALAU_TIDAK**, and **PAPAR**. The second syntax is a concise expert syntax using familiar forms such as **{**, **}**, **let**, **if**, **else**, and **print**. The compiler does not maintain two parsers; the dictionary-driven lexer normalizes both syntaxes before parsing, which means there is no runtime cost for using the novice form.

| Area | Phase 1 MVP Capability |
|---|---|
| Implementation language | Rust |
| Native backend | LLVM through `inkwell` |
| Runtime model | No virtual machine and no garbage collector |
| Supported values | Signed 64-bit integers and booleans |
| Supported statements | Variable binding, conditional branching, and printing |
| Supported expressions | Identifiers, integer literals, boolean literals, binary arithmetic, and comparisons |
| Supported OS targets | Linux ELF and Windows PE-oriented runtime layers |

The repository includes working examples under `examples/`, a dynamic token dictionary under `dict/core_map.json`, and a detailed `MANUAL.md` that explains installation, compilation, usage, IR inspection, and target behavior.
'''

files['dict/core_map.json'] = r'''{
  "version": "1.0.0",
  "language": "Logicodex",
  "description": "Dynamic dictionary mapping novice pseudocode and expert shorthand lexemes into unified compiler token identities.",
  "tokens": [
    { "identity": "START", "lexemes": ["MULA", "{"], "description": "Begin a program or block." },
    { "identity": "END", "lexemes": ["TAMAT", "}"], "description": "End a program or block." },
    { "identity": "LET", "lexemes": ["BINA", "let"], "description": "Create an immutable variable binding." },
    { "identity": "IF", "lexemes": ["JIKA", "if"], "description": "Begin a conditional branch." },
    { "identity": "THEN", "lexemes": ["MAKA"], "description": "Separate a novice condition from its branch body." },
    { "identity": "ELSE", "lexemes": ["JIKALAU_TIDAK", "else"], "description": "Begin the alternative branch." },
    { "identity": "PRINT", "lexemes": ["PAPAR", "print"], "description": "Print an integer or boolean value." },
    { "identity": "TRUE", "lexemes": ["BENAR", "true"], "description": "Boolean true literal." },
    { "identity": "FALSE", "lexemes": ["PALSU", "false"], "description": "Boolean false literal." },
    { "identity": "ASSIGN", "lexemes": ["="], "description": "Assignment delimiter in a binding." },
    { "identity": "PLUS", "lexemes": ["+"], "description": "Integer addition operator." },
    { "identity": "MINUS", "lexemes": ["-"], "description": "Integer subtraction operator." },
    { "identity": "STAR", "lexemes": ["*"], "description": "Integer multiplication operator." },
    { "identity": "SLASH", "lexemes": ["/"], "description": "Integer division operator." },
    { "identity": "GREATER", "lexemes": [">"] , "description": "Greater-than comparison." },
    { "identity": "GREATER_EQUAL", "lexemes": [">="], "description": "Greater-than-or-equal comparison." },
    { "identity": "LESS", "lexemes": ["<"], "description": "Less-than comparison." },
    { "identity": "LESS_EQUAL", "lexemes": ["<="], "description": "Less-than-or-equal comparison." },
    { "identity": "EQUAL_EQUAL", "lexemes": ["=="], "description": "Equality comparison." },
    { "identity": "BANG_EQUAL", "lexemes": ["!="], "description": "Inequality comparison." },
    { "identity": "LEFT_PAREN", "lexemes": ["("], "description": "Open grouped expression." },
    { "identity": "RIGHT_PAREN", "lexemes": [")"], "description": "Close grouped expression." },
    { "identity": "SEMICOLON", "lexemes": [";"], "description": "Optional statement separator." }
  ]
}
'''

files['stdlib/io.ldx'] = r'''# Logicodex Phase 1 standard I/O contract
#
# The compiler lowers PAPAR/print statements to the intrinsic runtime symbol
# `logicodex_print_i64`. The target layer supplies that symbol using either direct
# Linux syscalls or the Win32 console subsystem. User programs do not import this
# file directly in Phase 1; it documents the stable built-in ABI.

MULA
    # Built-in: PAPAR <integer-or-boolean-expression>
    # Boolean values are rendered as 1 for BENAR/true and 0 for PALSU/false.
TAMAT
'''

files['examples/01_tambah_pemula.ldx'] = r'''MULA
BINA x = 5
BINA y = 10
BINA total = x + y
JIKA total > 12 MAKA
    PAPAR 1
JIKALAU_TIDAK
    PAPAR 0
TAMAT
'''

files['examples/01_tambah_pakar.ldx'] = r'''{
let x = 5
let y = 10
let total = x + y
if total > 12 {
    print 1
} else {
    print 0
}
}
'''

files['src/ast.rs'] = r'''use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    pub statements: Vec<Stmt>,
}

impl Program {
    pub fn new(statements: Vec<Stmt>) -> Self {
        Self { statements }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stmt {
    Let { name: String, value: Expr },
    Print { value: Expr },
    If { condition: Expr, then_branch: Vec<Stmt>, else_branch: Vec<Stmt> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Integer(i64),
    Boolean(bool),
    Variable(String),
    Binary { left: Box<Expr>, op: BinaryOp, right: Box<Expr> },
    Grouped(Box<Expr>),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Equal,
    NotEqual,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Type {
    Int,
    Bool,
}

impl fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            BinaryOp::Add => "+",
            BinaryOp::Subtract => "-",
            BinaryOp::Multiply => "*",
            BinaryOp::Divide => "/",
            BinaryOp::Greater => ">",
            BinaryOp::GreaterEqual => ">=",
            BinaryOp::Less => "<",
            BinaryOp::LessEqual => "<=",
            BinaryOp::Equal => "==",
            BinaryOp::NotEqual => "!=",
        };
        write!(f, "{text}")
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Int => write!(f, "Int"),
            Type::Bool => write!(f, "Bool"),
        }
    }
}
'''

files['src/lexer.rs'] = r'''use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenKind {
    Start,
    End,
    Let,
    If,
    Then,
    Else,
    Print,
    True,
    False,
    Assign,
    Plus,
    Minus,
    Star,
    Slash,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    EqualEqual,
    BangEqual,
    LeftParen,
    RightParen,
    Semicolon,
    Identifier,
    Integer,
    Eof,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub line: usize,
    pub column: usize,
}

impl Token {
    pub fn new(kind: TokenKind, lexeme: impl Into<String>, line: usize, column: usize) -> Self {
        Self { kind, lexeme: lexeme.into(), line, column }
    }
}

#[derive(Debug, Deserialize)]
struct CoreMap {
    tokens: Vec<DictionaryToken>,
}

#[derive(Debug, Deserialize)]
struct DictionaryToken {
    identity: String,
    lexemes: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Lexicon {
    lexeme_to_kind: HashMap<String, TokenKind>,
    symbolic_lexemes: Vec<String>,
}

#[derive(Debug, Error)]
pub enum LexError {
    #[error("failed to read dictionary {path}: {source}")]
    DictionaryRead { path: String, source: std::io::Error },
    #[error("failed to parse dictionary {path}: {source}")]
    DictionaryParse { path: String, source: serde_json::Error },
    #[error("unknown dictionary token identity `{0}`")]
    UnknownIdentity(String),
    #[error("unexpected character `{ch}` at {line}:{column}")]
    UnexpectedCharacter { ch: char, line: usize, column: usize },
    #[error("integer literal `{literal}` at {line}:{column} does not fit in i64")]
    IntegerOverflow { literal: String, line: usize, column: usize },
}

impl TryFrom<&str> for TokenKind {
    type Error = LexError;

    fn try_from(identity: &str) -> Result<Self, Self::Error> {
        match identity {
            "START" => Ok(TokenKind::Start),
            "END" => Ok(TokenKind::End),
            "LET" => Ok(TokenKind::Let),
            "IF" => Ok(TokenKind::If),
            "THEN" => Ok(TokenKind::Then),
            "ELSE" => Ok(TokenKind::Else),
            "PRINT" => Ok(TokenKind::Print),
            "TRUE" => Ok(TokenKind::True),
            "FALSE" => Ok(TokenKind::False),
            "ASSIGN" => Ok(TokenKind::Assign),
            "PLUS" => Ok(TokenKind::Plus),
            "MINUS" => Ok(TokenKind::Minus),
            "STAR" => Ok(TokenKind::Star),
            "SLASH" => Ok(TokenKind::Slash),
            "GREATER" => Ok(TokenKind::Greater),
            "GREATER_EQUAL" => Ok(TokenKind::GreaterEqual),
            "LESS" => Ok(TokenKind::Less),
            "LESS_EQUAL" => Ok(TokenKind::LessEqual),
            "EQUAL_EQUAL" => Ok(TokenKind::EqualEqual),
            "BANG_EQUAL" => Ok(TokenKind::BangEqual),
            "LEFT_PAREN" => Ok(TokenKind::LeftParen),
            "RIGHT_PAREN" => Ok(TokenKind::RightParen),
            "SEMICOLON" => Ok(TokenKind::Semicolon),
            other => Err(LexError::UnknownIdentity(other.to_string())),
        }
    }
}

impl Lexicon {
    pub fn from_path(path: &Path) -> Result<Self, LexError> {
        let raw = fs::read_to_string(path).map_err(|source| LexError::DictionaryRead { path: path.display().to_string(), source })?;
        let map: CoreMap = serde_json::from_str(&raw).map_err(|source| LexError::DictionaryParse { path: path.display().to_string(), source })?;
        let mut lexeme_to_kind = HashMap::new();
        let mut symbolic_lexemes = Vec::new();

        for entry in map.tokens {
            let kind = TokenKind::try_from(entry.identity.as_str())?;
            for lexeme in entry.lexemes {
                if lexeme.chars().all(|c| !c.is_alphanumeric() && c != '_') {
                    symbolic_lexemes.push(lexeme.clone());
                }
                lexeme_to_kind.insert(lexeme, kind);
            }
        }
        symbolic_lexemes.sort_by_key(|s| std::cmp::Reverse(s.len()));
        Ok(Self { lexeme_to_kind, symbolic_lexemes })
    }

    fn keyword(&self, text: &str) -> Option<TokenKind> {
        self.lexeme_to_kind.get(text).copied()
    }
}

pub struct Lexer<'a> {
    source: &'a str,
    chars: Vec<char>,
    index: usize,
    line: usize,
    column: usize,
    lexicon: &'a Lexicon,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str, lexicon: &'a Lexicon) -> Self {
        Self { source, chars: source.chars().collect(), index: 0, line: 1, column: 1, lexicon }
    }

    pub fn tokenize(mut self) -> Result<Vec<Token>, LexError> {
        let mut tokens = Vec::new();
        while !self.is_at_end() {
            let ch = self.peek();
            match ch {
                ' ' | '\r' | '\t' => { self.advance(); }
                '\n' => { self.advance_line(); }
                '#' => { self.skip_comment(); }
                c if c.is_ascii_digit() => tokens.push(self.integer()?),
                c if is_ident_start(c) => tokens.push(self.identifier_or_keyword()),
                _ => {
                    if let Some(token) = self.symbolic() {
                        tokens.push(token);
                    } else {
                        return Err(LexError::UnexpectedCharacter { ch, line: self.line, column: self.column });
                    }
                }
            }
        }
        tokens.push(Token::new(TokenKind::Eof, "", self.line, self.column));
        Ok(tokens)
    }

    fn is_at_end(&self) -> bool { self.index >= self.chars.len() }
    fn peek(&self) -> char { self.chars[self.index] }

    fn advance(&mut self) -> char {
        let ch = self.chars[self.index];
        self.index += 1;
        self.column += 1;
        ch
    }

    fn advance_line(&mut self) {
        self.index += 1;
        self.line += 1;
        self.column = 1;
    }

    fn skip_comment(&mut self) {
        while !self.is_at_end() && self.peek() != '\n' { self.advance(); }
    }

    fn integer(&mut self) -> Result<Token, LexError> {
        let start = self.index;
        let line = self.line;
        let column = self.column;
        while !self.is_at_end() && self.peek().is_ascii_digit() { self.advance(); }
        let literal: String = self.chars[start..self.index].iter().collect();
        literal.parse::<i64>().map_err(|_| LexError::IntegerOverflow { literal: literal.clone(), line, column })?;
        Ok(Token::new(TokenKind::Integer, literal, line, column))
    }

    fn identifier_or_keyword(&mut self) -> Token {
        let start = self.index;
        let line = self.line;
        let column = self.column;
        while !self.is_at_end() && is_ident_continue(self.peek()) { self.advance(); }
        let lexeme: String = self.chars[start..self.index].iter().collect();
        let kind = self.lexicon.keyword(&lexeme).unwrap_or(TokenKind::Identifier);
        Token::new(kind, lexeme, line, column)
    }

    fn symbolic(&mut self) -> Option<Token> {
        for symbol in &self.lexicon.symbolic_lexemes {
            if self.source[self.byte_offset()..].starts_with(symbol) {
                let line = self.line;
                let column = self.column;
                for _ in symbol.chars() { self.advance(); }
                let kind = self.lexicon.keyword(symbol).expect("symbol must be present in lexeme map");
                return Some(Token::new(kind, symbol.clone(), line, column));
            }
        }
        None
    }

    fn byte_offset(&self) -> usize {
        self.chars[..self.index].iter().map(|c| c.len_utf8()).sum()
    }
}

fn is_ident_start(ch: char) -> bool { ch.is_ascii_alphabetic() || ch == '_' }
fn is_ident_continue(ch: char) -> bool { ch.is_ascii_alphanumeric() || ch == '_' }
'''

files['src/parser.rs'] = r'''use crate::ast::{BinaryOp, Expr, Program, Stmt};
use crate::lexer::{Token, TokenKind};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("expected {expected} at {line}:{column}, found `{found}`")]
    Expected { expected: String, found: String, line: usize, column: usize },
    #[error("unexpected token `{found}` at {line}:{column}")]
    Unexpected { found: String, line: usize, column: usize },
    #[error("invalid integer literal `{literal}` at {line}:{column}")]
    InvalidInteger { literal: String, line: usize, column: usize },
}

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self { Self { tokens, current: 0 } }

    pub fn parse(&mut self) -> Result<Program, ParseError> {
        self.consume_optional_semicolons();
        let wrapped = self.matches(TokenKind::Start);
        let mut statements = Vec::new();
        while !self.is_at_end() && !self.check(TokenKind::End) {
            self.consume_optional_semicolons();
            if self.check(TokenKind::End) || self.is_at_end() { break; }
            statements.push(self.statement()?);
        }
        if wrapped { self.consume(TokenKind::End, "program terminator TAMAT or }")?; }
        self.consume_optional_semicolons();
        self.consume(TokenKind::Eof, "end of file")?;
        Ok(Program::new(statements))
    }

    fn statement(&mut self) -> Result<Stmt, ParseError> {
        let stmt = if self.matches(TokenKind::Let) { self.let_statement()? }
        else if self.matches(TokenKind::Print) { self.print_statement()? }
        else if self.matches(TokenKind::If) { self.if_statement()? }
        else {
            let t = self.peek();
            return Err(ParseError::Unexpected { found: t.lexeme.clone(), line: t.line, column: t.column });
        };
        self.consume_optional_semicolons();
        Ok(stmt)
    }

    fn let_statement(&mut self) -> Result<Stmt, ParseError> {
        let name = self.consume(TokenKind::Identifier, "variable name")?.lexeme.clone();
        self.consume(TokenKind::Assign, "=")?;
        let value = self.expression()?;
        Ok(Stmt::Let { name, value })
    }

    fn print_statement(&mut self) -> Result<Stmt, ParseError> {
        let value = self.expression()?;
        Ok(Stmt::Print { value })
    }

    fn if_statement(&mut self) -> Result<Stmt, ParseError> {
        let condition = self.expression()?;
        if self.matches(TokenKind::Then) {
            let then_branch = self.statements_until(&[TokenKind::Else, TokenKind::End, TokenKind::Eof])?;
            let else_branch = if self.matches(TokenKind::Else) {
                self.statements_until(&[TokenKind::End, TokenKind::Eof])?
            } else { Vec::new() };
            Ok(Stmt::If { condition, then_branch, else_branch })
        } else if self.matches(TokenKind::Start) {
            let then_branch = self.statements_until(&[TokenKind::End])?;
            self.consume(TokenKind::End, "closing } for if branch")?;
            let else_branch = if self.matches(TokenKind::Else) {
                if self.matches(TokenKind::Start) {
                    let branch = self.statements_until(&[TokenKind::End])?;
                    self.consume(TokenKind::End, "closing } for else branch")?;
                    branch
                } else {
                    vec![self.statement()?]
                }
            } else { Vec::new() };
            Ok(Stmt::If { condition, then_branch, else_branch })
        } else {
            let t = self.peek();
            Err(ParseError::Expected { expected: "MAKA or { after if condition".to_string(), found: t.lexeme.clone(), line: t.line, column: t.column })
        }
    }

    fn statements_until(&mut self, terminators: &[TokenKind]) -> Result<Vec<Stmt>, ParseError> {
        let mut statements = Vec::new();
        while !self.is_at_end() && !terminators.iter().any(|kind| self.check(*kind)) {
            self.consume_optional_semicolons();
            if self.is_at_end() || terminators.iter().any(|kind| self.check(*kind)) { break; }
            statements.push(self.statement()?);
        }
        Ok(statements)
    }

    fn expression(&mut self) -> Result<Expr, ParseError> { self.equality() }

    fn equality(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.comparison()?;
        while self.matches(TokenKind::EqualEqual) || self.matches(TokenKind::BangEqual) {
            let op = match self.previous().kind { TokenKind::EqualEqual => BinaryOp::Equal, TokenKind::BangEqual => BinaryOp::NotEqual, _ => unreachable!() };
            let right = self.comparison()?;
            expr = Expr::Binary { left: Box::new(expr), op, right: Box::new(right) };
        }
        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.term()?;
        while self.matches(TokenKind::Greater) || self.matches(TokenKind::GreaterEqual) || self.matches(TokenKind::Less) || self.matches(TokenKind::LessEqual) {
            let op = match self.previous().kind {
                TokenKind::Greater => BinaryOp::Greater,
                TokenKind::GreaterEqual => BinaryOp::GreaterEqual,
                TokenKind::Less => BinaryOp::Less,
                TokenKind::LessEqual => BinaryOp::LessEqual,
                _ => unreachable!(),
            };
            let right = self.term()?;
            expr = Expr::Binary { left: Box::new(expr), op, right: Box::new(right) };
        }
        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.factor()?;
        while self.matches(TokenKind::Plus) || self.matches(TokenKind::Minus) {
            let op = if self.previous().kind == TokenKind::Plus { BinaryOp::Add } else { BinaryOp::Subtract };
            let right = self.factor()?;
            expr = Expr::Binary { left: Box::new(expr), op, right: Box::new(right) };
        }
        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.primary()?;
        while self.matches(TokenKind::Star) || self.matches(TokenKind::Slash) {
            let op = if self.previous().kind == TokenKind::Star { BinaryOp::Multiply } else { BinaryOp::Divide };
            let right = self.primary()?;
            expr = Expr::Binary { left: Box::new(expr), op, right: Box::new(right) };
        }
        Ok(expr)
    }

    fn primary(&mut self) -> Result<Expr, ParseError> {
        if self.matches(TokenKind::Integer) {
            let token = self.previous();
            let value = token.lexeme.parse::<i64>().map_err(|_| ParseError::InvalidInteger { literal: token.lexeme.clone(), line: token.line, column: token.column })?;
            return Ok(Expr::Integer(value));
        }
        if self.matches(TokenKind::True) { return Ok(Expr::Boolean(true)); }
        if self.matches(TokenKind::False) { return Ok(Expr::Boolean(false)); }
        if self.matches(TokenKind::Identifier) { return Ok(Expr::Variable(self.previous().lexeme.clone())); }
        if self.matches(TokenKind::LeftParen) {
            let expr = self.expression()?;
            self.consume(TokenKind::RightParen, ")")?;
            return Ok(Expr::Grouped(Box::new(expr)));
        }
        let t = self.peek();
        Err(ParseError::Unexpected { found: t.lexeme.clone(), line: t.line, column: t.column })
    }

    fn consume_optional_semicolons(&mut self) { while self.matches(TokenKind::Semicolon) {} }

    fn matches(&mut self, kind: TokenKind) -> bool {
        if self.check(kind) { self.advance(); true } else { false }
    }

    fn consume(&mut self, kind: TokenKind, expected: &str) -> Result<&Token, ParseError> {
        if self.check(kind) { return Ok(self.advance()); }
        let t = self.peek();
        Err(ParseError::Expected { expected: expected.to_string(), found: t.lexeme.clone(), line: t.line, column: t.column })
    }

    fn check(&self, kind: TokenKind) -> bool { !self.is_at_end() && self.peek().kind == kind }
    fn advance(&mut self) -> &Token { if !self.is_at_end() { self.current += 1; } self.previous() }
    fn is_at_end(&self) -> bool { self.peek().kind == TokenKind::Eof }
    fn peek(&self) -> &Token { &self.tokens[self.current] }
    fn previous(&self) -> &Token { &self.tokens[self.current - 1] }
}
'''

files['src/semantic.rs'] = r'''use crate::ast::{BinaryOp, Expr, Program, Stmt, Type};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SemanticError {
    #[error("variable `{0}` is already defined in this scope")]
    DuplicateVariable(String),
    #[error("variable `{0}` is not defined")]
    UndefinedVariable(String),
    #[error("operator `{op}` requires {expected} operands but received {left} and {right}")]
    TypeMismatch { op: BinaryOp, expected: &'static str, left: Type, right: Type },
    #[error("if condition must be Bool but received {0}")]
    NonBooleanCondition(Type),
    #[error("division by a constant zero is rejected by static analysis")]
    DivisionByZero,
}

#[derive(Debug, Default)]
pub struct Analyzer {
    scopes: Vec<HashMap<String, Type>>,
}

impl Analyzer {
    pub fn analyze(program: &Program) -> Result<(), SemanticError> {
        let mut analyzer = Self { scopes: vec![HashMap::new()] };
        analyzer.block(&program.statements)
    }

    fn block(&mut self, statements: &[Stmt]) -> Result<(), SemanticError> {
        for stmt in statements { self.statement(stmt)?; }
        Ok(())
    }

    fn scoped_block(&mut self, statements: &[Stmt]) -> Result<(), SemanticError> {
        self.scopes.push(HashMap::new());
        let result = self.block(statements);
        self.scopes.pop();
        result
    }

    fn statement(&mut self, stmt: &Stmt) -> Result<(), SemanticError> {
        match stmt {
            Stmt::Let { name, value } => {
                let ty = self.expression(value)?;
                let scope = self.scopes.last_mut().expect("semantic analyzer must always have a scope");
                if scope.contains_key(name) { return Err(SemanticError::DuplicateVariable(name.clone())); }
                scope.insert(name.clone(), ty);
                Ok(())
            }
            Stmt::Print { value } => { self.expression(value)?; Ok(()) }
            Stmt::If { condition, then_branch, else_branch } => {
                let ty = self.expression(condition)?;
                if ty != Type::Bool { return Err(SemanticError::NonBooleanCondition(ty)); }
                self.scoped_block(then_branch)?;
                self.scoped_block(else_branch)
            }
        }
    }

    fn expression(&self, expr: &Expr) -> Result<Type, SemanticError> {
        match expr {
            Expr::Integer(_) => Ok(Type::Int),
            Expr::Boolean(_) => Ok(Type::Bool),
            Expr::Variable(name) => self.resolve(name),
            Expr::Grouped(inner) => self.expression(inner),
            Expr::Binary { left, op, right } => {
                if *op == BinaryOp::Divide && matches!(right.as_ref(), Expr::Integer(0)) { return Err(SemanticError::DivisionByZero); }
                let left_ty = self.expression(left)?;
                let right_ty = self.expression(right)?;
                match op {
                    BinaryOp::Add | BinaryOp::Subtract | BinaryOp::Multiply | BinaryOp::Divide => {
                        if left_ty == Type::Int && right_ty == Type::Int { Ok(Type::Int) } else { Err(SemanticError::TypeMismatch { op: *op, expected: "Int", left: left_ty, right: right_ty }) }
                    }
                    BinaryOp::Greater | BinaryOp::GreaterEqual | BinaryOp::Less | BinaryOp::LessEqual => {
                        if left_ty == Type::Int && right_ty == Type::Int { Ok(Type::Bool) } else { Err(SemanticError::TypeMismatch { op: *op, expected: "Int", left: left_ty, right: right_ty }) }
                    }
                    BinaryOp::Equal | BinaryOp::NotEqual => {
                        if left_ty == right_ty { Ok(Type::Bool) } else { Err(SemanticError::TypeMismatch { op: *op, expected: "matching", left: left_ty, right: right_ty }) }
                    }
                }
            }
        }
    }

    fn resolve(&self, name: &str) -> Result<Type, SemanticError> {
        for scope in self.scopes.iter().rev() {
            if let Some(ty) = scope.get(name) { return Ok(*ty); }
        }
        Err(SemanticError::UndefinedVariable(name.to_string()))
    }
}
'''

files['src/codegen.rs'] = r'''use crate::ast::{BinaryOp, Expr, Program, Stmt};
use crate::os::target::{build_target_machine, OutputKind};
use anyhow::{anyhow, Context as AnyhowContext, Result};
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::IntType;
use inkwell::values::{FunctionValue, IntValue, PointerValue};
use inkwell::IntPredicate;
use std::collections::HashMap;
use std::path::Path;

pub struct CodegenOptions {
    pub module_name: String,
    pub emit_ir: bool,
}

pub struct CodegenArtifact {
    pub object_path: std::path::PathBuf,
    pub ir_path: Option<std::path::PathBuf>,
}

pub struct LlvmCompiler<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    i64_type: IntType<'ctx>,
    bool_type: IntType<'ctx>,
    variables: Vec<HashMap<String, PointerValue<'ctx>>>,
    print_fn: FunctionValue<'ctx>,
}

impl<'ctx> LlvmCompiler<'ctx> {
    pub fn compile_to_object(program: &Program, object_path: &Path, options: &CodegenOptions) -> Result<CodegenArtifact> {
        let context = Context::create();
        let mut compiler = Self::new(&context, &options.module_name);
        compiler.emit_program(program)?;
        compiler.module.verify().map_err(|e| anyhow!("LLVM module verification failed: {e}"))?;

        let target_machine = build_target_machine(OutputKind::Object)?;
        target_machine.write_to_file(&compiler.module, inkwell::targets::FileType::Object, object_path)
            .map_err(|e| anyhow!("failed to emit object file {}: {e}", object_path.display()))?;

        let ir_path = if options.emit_ir {
            let mut ir_path = object_path.to_path_buf();
            ir_path.set_extension("ll");
            compiler.module.print_to_file(&ir_path).map_err(|e| anyhow!("failed to write LLVM IR {}: {e}", ir_path.display()))?;
            Some(ir_path)
        } else { None };

        Ok(CodegenArtifact { object_path: object_path.to_path_buf(), ir_path })
    }

    fn new(context: &'ctx Context, module_name: &str) -> Self {
        let module = context.create_module(module_name);
        let builder = context.create_builder();
        let i64_type = context.i64_type();
        let bool_type = context.bool_type();
        let print_type = context.void_type().fn_type(&[i64_type.into()], false);
        let print_fn = module.add_function("logicodex_print_i64", print_type, None);
        Self { context, module, builder, i64_type, bool_type, variables: vec![HashMap::new()], print_fn }
    }

    fn emit_program(&mut self, program: &Program) -> Result<()> {
        let i32_type = self.context.i32_type();
        let main_type = i32_type.fn_type(&[], false);
        let main = self.module.add_function("main", main_type, None);
        let entry = self.context.append_basic_block(main, "entry");
        self.builder.position_at_end(entry);
        self.emit_block(&program.statements)?;
        self.builder.build_return(Some(&i32_type.const_int(0, false))).context("failed to build return")?;
        Ok(())
    }

    fn emit_block(&mut self, statements: &[Stmt]) -> Result<()> {
        self.variables.push(HashMap::new());
        for stmt in statements { self.emit_stmt(stmt)?; }
        self.variables.pop();
        Ok(())
    }

    fn emit_stmt(&mut self, stmt: &Stmt) -> Result<()> {
        match stmt {
            Stmt::Let { name, value } => {
                let current_fn = self.current_function()?;
                let alloca = self.create_entry_alloca(current_fn, name);
                let value = self.emit_expr(value)?;
                self.builder.build_store(alloca, value).context("failed to store variable")?;
                self.variables.last_mut().expect("codegen scope exists").insert(name.clone(), alloca);
                Ok(())
            }
            Stmt::Print { value } => {
                let value = self.emit_expr(value)?;
                self.builder.build_call(self.print_fn, &[value.into()], "printtmp").context("failed to build print call")?;
                Ok(())
            }
            Stmt::If { condition, then_branch, else_branch } => self.emit_if(condition, then_branch, else_branch),
        }
    }

    fn emit_if(&mut self, condition: &Expr, then_branch: &[Stmt], else_branch: &[Stmt]) -> Result<()> {
        let parent = self.current_function()?;
        let condition_value = self.emit_expr(condition)?;
        let zero = self.bool_type.const_zero();
        let condition_bool = self.builder.build_int_compare(IntPredicate::NE, condition_value, zero, "ifcond").context("failed to compare if condition")?;
        let then_block = self.context.append_basic_block(parent, "then");
        let else_block = self.context.append_basic_block(parent, "else");
        let merge_block = self.context.append_basic_block(parent, "ifcont");
        self.builder.build_conditional_branch(condition_bool, then_block, else_block).context("failed to build conditional branch")?;

        self.builder.position_at_end(then_block);
        self.emit_block(then_branch)?;
        if self.builder.get_insert_block().and_then(|b| b.get_terminator()).is_none() {
            self.builder.build_unconditional_branch(merge_block).context("failed to branch from then block")?;
        }

        self.builder.position_at_end(else_block);
        self.emit_block(else_branch)?;
        if self.builder.get_insert_block().and_then(|b| b.get_terminator()).is_none() {
            self.builder.build_unconditional_branch(merge_block).context("failed to branch from else block")?;
        }

        self.builder.position_at_end(merge_block);
        Ok(())
    }

    fn emit_expr(&mut self, expr: &Expr) -> Result<IntValue<'ctx>> {
        match expr {
            Expr::Integer(value) => Ok(self.i64_type.const_int(*value as u64, true)),
            Expr::Boolean(value) => Ok(self.bool_to_i64(self.bool_type.const_int(u64::from(*value), false))),
            Expr::Variable(name) => {
                let ptr = self.lookup_variable(name).ok_or_else(|| anyhow!("internal codegen error: undefined variable `{name}`"))?;
                Ok(self.builder.build_load(self.i64_type, ptr, name).context("failed to load variable")?.into_int_value())
            }
            Expr::Grouped(inner) => self.emit_expr(inner),
            Expr::Binary { left, op, right } => {
                let l = self.emit_expr(left)?;
                let r = self.emit_expr(right)?;
                match op {
                    BinaryOp::Add => self.builder.build_int_add(l, r, "addtmp").context("failed to build add"),
                    BinaryOp::Subtract => self.builder.build_int_sub(l, r, "subtmp").context("failed to build subtract"),
                    BinaryOp::Multiply => self.builder.build_int_mul(l, r, "multmp").context("failed to build multiply"),
                    BinaryOp::Divide => self.builder.build_int_signed_div(l, r, "divtmp").context("failed to build division"),
                    BinaryOp::Greater => self.compare_to_i64(IntPredicate::SGT, l, r, "gttmp"),
                    BinaryOp::GreaterEqual => self.compare_to_i64(IntPredicate::SGE, l, r, "getmp"),
                    BinaryOp::Less => self.compare_to_i64(IntPredicate::SLT, l, r, "lttmp"),
                    BinaryOp::LessEqual => self.compare_to_i64(IntPredicate::SLE, l, r, "letmp"),
                    BinaryOp::Equal => self.compare_to_i64(IntPredicate::EQ, l, r, "eqtmp"),
                    BinaryOp::NotEqual => self.compare_to_i64(IntPredicate::NE, l, r, "netmp"),
                }
            }
        }
    }

    fn compare_to_i64(&self, predicate: IntPredicate, left: IntValue<'ctx>, right: IntValue<'ctx>, name: &str) -> Result<IntValue<'ctx>> {
        let cmp = self.builder.build_int_compare(predicate, left, right, name).context("failed to build comparison")?;
        Ok(self.bool_to_i64(cmp))
    }

    fn bool_to_i64(&self, value: IntValue<'ctx>) -> IntValue<'ctx> {
        self.builder.build_int_z_extend(value, self.i64_type, "booltoint").expect("zext from i1 to i64 is valid")
    }

    fn create_entry_alloca(&self, function: FunctionValue<'ctx>, name: &str) -> PointerValue<'ctx> {
        let entry_builder = self.context.create_builder();
        let entry = function.get_first_basic_block().expect("function has entry block");
        match entry.get_first_instruction() {
            Some(first) => entry_builder.position_before(&first),
            None => entry_builder.position_at_end(entry),
        }
        entry_builder.build_alloca(self.i64_type, name).expect("alloca in entry block is valid")
    }

    fn lookup_variable(&self, name: &str) -> Option<PointerValue<'ctx>> {
        for scope in self.variables.iter().rev() {
            if let Some(ptr) = scope.get(name) { return Some(*ptr); }
        }
        None
    }

    fn current_function(&self) -> Result<FunctionValue<'ctx>> {
        self.builder.get_insert_block().and_then(|block| block.get_parent()).ok_or_else(|| anyhow!("no active LLVM function"))
    }
}
'''

files['src/os/mod.rs'] = r'''pub mod target;

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "linux")]
pub use linux::runtime_assembly;

#[cfg(target_os = "windows")]
pub use windows::runtime_assembly;

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
pub fn runtime_assembly() -> &'static str {
    ".global logicodex_print_i64\nlogicodex_print_i64:\n    ret\n"
}
'''

files['src/os/linux.rs'] = r'''pub fn runtime_assembly() -> &'static str {
    r#"
    .text
    .global logicodex_print_i64
    .type logicodex_print_i64, @function
logicodex_print_i64:
    push %rbp
    mov %rsp, %rbp
    sub $64, %rsp
    mov %rdi, %rax
    lea -2(%rbp), %rsi
    movb $10, (%rsi)
    mov $1, %rcx
    cmp $0, %rax
    jne .Lconvert
    movb $48, -3(%rbp)
    lea -3(%rbp), %rsi
    mov $2, %rdx
    jmp .Lwrite
.Lconvert:
    mov $0, %r8
    cmp $0, %rax
    jge .Ldigits
    neg %rax
    mov $1, %r8
.Ldigits:
    mov $10, %r9
.Lloop:
    xor %rdx, %rdx
    div %r9
    add $48, %dl
    dec %rsi
    mov %dl, (%rsi)
    inc %rcx
    cmp $0, %rax
    jne .Lloop
    cmp $0, %r8
    je .Lprepare
    dec %rsi
    movb $45, (%rsi)
    inc %rcx
.Lprepare:
    mov %rcx, %rdx
.Lwrite:
    mov $1, %rax
    mov $1, %rdi
    syscall
    leave
    ret
"#
}
'''

files['src/os/windows.rs'] = r'''pub fn runtime_assembly() -> &'static str {
    r#"
    .text
    .globl logicodex_print_i64
    .def logicodex_print_i64; .scl 2; .type 32; .endef
logicodex_print_i64:
    pushq %rbp
    movq %rsp, %rbp
    subq $96, %rsp
    leaq -40(%rbp), %rdx
    movb $10, 31(%rdx)
    movq %rcx, %rax
    movq $1, %r10
    cmpq $0, %rax
    jne .Lwin_convert
    movb $48, 30(%rdx)
    leaq 30(%rdx), %rdx
    movl $2, %r8d
    jmp .Lwin_console
.Lwin_convert:
    movq $0, %r11
    cmpq $0, %rax
    jge .Lwin_digits
    negq %rax
    movq $1, %r11
.Lwin_digits:
    leaq 31(%rdx), %r9
    movq $10, %r10
.Lwin_loop:
    xorq %rdx, %rdx
    divq %r10
    addb $48, %dl
    decq %r9
    movb %dl, (%r9)
    cmpq $0, %rax
    jne .Lwin_loop
    cmpq $0, %r11
    je .Lwin_measure
    decq %r9
    movb $45, (%r9)
.Lwin_measure:
    leaq -40(%rbp), %rdx
    addq $32, %rdx
    subq %r9, %rdx
    movq %r9, %rdx
    movl $32, %r8d
.Lwin_console:
    movq $-11, %rcx
    call GetStdHandle
    movq %rax, %rcx
    leaq -8(%rbp), %r9
    movq $0, 32(%rsp)
    call WriteFile
    addq $96, %rsp
    popq %rbp
    ret
"#
}
'''

files['src/os/target.rs'] = r'''use anyhow::{anyhow, Result};
use inkwell::targets::{CodeModel, InitializationConfig, RelocMode, Target, TargetMachine};
use inkwell::OptimizationLevel;

#[derive(Debug, Clone, Copy)]
pub enum OutputKind {
    Object,
}

pub fn build_target_machine(_kind: OutputKind) -> Result<TargetMachine> {
    Target::initialize_all(&InitializationConfig::default());
    let triple = TargetMachine::get_default_triple();
    let target = Target::from_triple(&triple).map_err(|e| anyhow!("failed to load LLVM target for {}: {e}", triple.as_str().to_string_lossy()))?;
    target.create_target_machine(
        &triple,
        "generic",
        "",
        OptimizationLevel::Aggressive,
        RelocMode::PIC,
        CodeModel::Default,
    ).ok_or_else(|| anyhow!("failed to create LLVM target machine for {}", triple.as_str().to_string_lossy()))
}
'''

files['src/main.rs'] = r'''mod ast;
mod codegen;
mod lexer;
mod os;
mod parser;
mod semantic;

use anyhow::{Context, Result};
use clap::{Parser as ClapParser, Subcommand};
use codegen::{CodegenOptions, LlvmCompiler};
use lexer::{Lexer, Lexicon};
use parser::Parser;
use semantic::Analyzer;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, ClapParser)]
#[command(name = "logicodex", version, about = "Native compiler for the Logicodex programming language")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Compile {
        #[arg(value_name = "FILE")]
        file: PathBuf,
        #[arg(short, long, value_name = "OUTPUT")]
        output: Option<PathBuf>,
        #[arg(long, default_value = "dict/core_map.json")]
        dict: PathBuf,
        #[arg(long)]
        emit_ir: bool,
        #[arg(long, help = "Stop after generating the native object file")]
        object_only: bool,
    },
    Check {
        #[arg(value_name = "FILE")]
        file: PathBuf,
        #[arg(long, default_value = "dict/core_map.json")]
        dict: PathBuf,
    },
    Tokens {
        #[arg(value_name = "FILE")]
        file: PathBuf,
        #[arg(long, default_value = "dict/core_map.json")]
        dict: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Compile { file, output, dict, emit_ir, object_only } => compile(&file, output, &dict, emit_ir, object_only),
        Commands::Check { file, dict } => { parse_and_analyze(&file, &dict)?; println!("{}: semantic validation succeeded", file.display()); Ok(()) }
        Commands::Tokens { file, dict } => print_tokens(&file, &dict),
    }
}

fn compile(file: &Path, output: Option<PathBuf>, dict: &Path, emit_ir: bool, object_only: bool) -> Result<()> {
    let program = parse_and_analyze(file, dict)?;
    let output_path = output.unwrap_or_else(|| default_output(file, object_only));
    let object_path = if object_only { output_path.clone() } else { output_path.with_extension("o") };

    let artifact = LlvmCompiler::compile_to_object(&program, &object_path, &CodegenOptions { module_name: module_name(file), emit_ir })?;
    if let Some(ir_path) = artifact.ir_path.as_ref() { println!("LLVM IR written to {}", ir_path.display()); }

    if object_only {
        println!("Object file written to {}", artifact.object_path.display());
        return Ok(());
    }

    let runtime_asm = output_path.with_extension("runtime.s");
    fs::write(&runtime_asm, os::runtime_assembly()).with_context(|| format!("failed to write runtime assembly {}", runtime_asm.display()))?;
    link_executable(&artifact.object_path, &runtime_asm, &output_path)?;
    println!("Native executable written to {}", output_path.display());
    Ok(())
}

fn parse_and_analyze(file: &Path, dict: &Path) -> Result<ast::Program> {
    let source = fs::read_to_string(file).with_context(|| format!("failed to read source file {}", file.display()))?;
    let lexicon = Lexicon::from_path(dict).with_context(|| format!("failed to load dictionary {}", dict.display()))?;
    let tokens = Lexer::new(&source, &lexicon).tokenize()?;
    let mut parser = Parser::new(tokens);
    let program = parser.parse()?;
    Analyzer::analyze(&program)?;
    Ok(program)
}

fn print_tokens(file: &Path, dict: &Path) -> Result<()> {
    let source = fs::read_to_string(file).with_context(|| format!("failed to read source file {}", file.display()))?;
    let lexicon = Lexicon::from_path(dict).with_context(|| format!("failed to load dictionary {}", dict.display()))?;
    for token in Lexer::new(&source, &lexicon).tokenize()? {
        println!("{:?}\t{}\t{}:{}", token.kind, token.lexeme, token.line, token.column);
    }
    Ok(())
}

fn default_output(file: &Path, object_only: bool) -> PathBuf {
    let mut path = file.with_extension(if object_only { "o" } else if cfg!(target_os = "windows") { "exe" } else { "" });
    if !object_only && !cfg!(target_os = "windows") {
        let stem = file.file_stem().and_then(|s| s.to_str()).unwrap_or("a.out");
        path.set_file_name(stem);
    }
    path
}

fn module_name(file: &Path) -> String {
    file.file_stem().and_then(|s| s.to_str()).unwrap_or("logicodex_module").replace('-', "_")
}

fn link_executable(object_path: &Path, runtime_asm: &Path, output_path: &Path) -> Result<()> {
    let linker = std::env::var("LOGICODEX_LINKER").unwrap_or_else(|_| "cc".to_string());
    let status = Command::new(&linker)
        .arg(object_path)
        .arg(runtime_asm)
        .arg("-o")
        .arg(output_path)
        .status()
        .with_context(|| format!("failed to invoke linker `{linker}`"))?;
    if status.success() { Ok(()) } else { anyhow::bail!("linker `{linker}` failed with status {status}") }
}
'''

files['MANUAL.md'] = r'''# Logicodex Language Phase 1 MVP Developer Manual

**Author:** Manus AI  
**Repository:** `logicodex`  
**Compiler executable:** `logicodex`

## 1. Purpose and Architecture

Logicodex is a native programming language compiler implemented in Rust. Its Phase 1 MVP demonstrates a practical dual-syntax front end in which novice-oriented pseudocode and expert shorthand are normalized through `dict/core_map.json` into the same compiler token identities. Once lexing is complete, the rest of the pipeline is syntax-neutral: both source forms produce the same AST, pass through the same semantic analyzer, and are lowered to optimized LLVM machine code.

| Pipeline Stage | Repository File | Responsibility |
|---|---|---|
| Dynamic lexical mapping | `src/lexer.rs`, `dict/core_map.json` | Converts novice and expert lexemes into shared token identities. |
| Parsing | `src/parser.rs` | Builds the AST from normalized tokens. |
| Static analysis | `src/semantic.rs` | Enforces variable scope, operand types, boolean conditions, and constant zero division rejection. |
| LLVM generation | `src/codegen.rs` | Lowers AST nodes to LLVM IR and emits a native object file. |
| OS runtime | `src/os/` | Supplies platform-specific runtime I/O glue for `logicodex_print_i64`. |
| CLI | `src/main.rs` | Provides `compile`, `check`, and `tokens` commands. |

> The key design point is that **novice syntax has zero downstream cost**. It is normalized before parsing, so the parser, semantic analyzer, and LLVM backend do not branch on whether a program was written with `MULA/BINA/PAPAR` or `{ / let / print`.

## 2. Environment Setup

Logicodex uses Rust and LLVM. Rust can be installed through `rustup`, the official Rust toolchain installer, and LLVM must be installed with development libraries that match the `inkwell` feature configured in `Cargo.toml`.[1] [2] The supplied `Cargo.toml` selects `inkwell` with the `llvm17-0` feature, so a production build should use LLVM 17 development packages.[3]

| Platform | Required Tools | Example Installation |
|---|---|---|
| Ubuntu/Debian Linux | Rust, Cargo, LLVM 17, Clang or GCC linker | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs  sh` then install LLVM 17 packages from your distribution or LLVM APT repository. |
| Windows | Rust MSVC toolchain, LLVM 17, Visual Studio Build Tools or Clang | Install Rust from `rustup.rs`, install LLVM 17 from the LLVM release installer, and ensure `LLVM_SYS_170_PREFIX` points to the LLVM installation if auto-detection fails. |
| macOS for development inspection | Rust, LLVM 17, Clang | The Phase 1 runtime targets Linux and Windows; macOS can still inspect tokens, parse, and emit LLVM object code when LLVM is configured. |

For Linux systems using the official LLVM APT packages, the typical setup is similar to the following. Adjust package names if your distribution exposes a different LLVM package series.

```bash
sudo apt-get update
sudo apt-get install -y build-essential curl git clang lld llvm-17 llvm-17-dev libpolly-17-dev
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
rustup default stable
```

For Windows PowerShell, a representative setup is as follows.

```powershell
winget install Rustlang.Rustup
winget install LLVM.LLVM
winget install Microsoft.VisualStudio.2022.BuildTools
$env:LLVM_SYS_170_PREFIX = "C:\Program Files\LLVM"
```

## 3. Building the Compiler

From the repository root, build the compiler in release mode.

```bash
cd logicodex
cargo build --release
```

The release binary is produced at:

```bash
./target/release/logicodex
```

The project configures release builds with link-time optimization and aggressive optimization settings. The compiler itself remains a standard Rust executable, while programs compiled by Logicodex are emitted through LLVM as native object files and linked to an OS-specific runtime layer.

## 4. Usage Blueprint

The novice example is located at `examples/01_tambah_pemula.ldx` and uses Malay-inspired teaching syntax.

```logicodex
MULA
BINA x = 5
BINA y = 10
BINA total = x + y
JIKA total > 12 MAKA
    PAPAR 1
JIKALAU_TIDAK
    PAPAR 0
TAMAT
```

Compile it on Linux with:

```bash
./target/release/logicodex compile examples/01_tambah_pemula.ldx --emit-ir -o ./tambah_pemula
./tambah_pemula
```

The expected terminal output is:

```text
1
```

The expert shorthand example is located at `examples/01_tambah_pakar.ldx`.

```logicodex
{
let x = 5
let y = 10
let total = x + y
if total > 12 {
    print 1
} else {
    print 0
}
}
```

Compile it with:

```bash
./target/release/logicodex compile examples/01_tambah_pakar.ldx --emit-ir -o ./tambah_pakar
./tambah_pakar
```

On Windows, use the `.exe` output name and a Windows-capable linker environment.

```powershell
.\target\release\logicodex.exe compile .\examples\01_tambah_pakar.ldx --emit-ir -o .\tambah_pakar.exe
.\tambah_pakar.exe
```

## 5. Validation Guide

The compiler includes a `tokens` command that proves both syntaxes normalize into the same token identities.

```bash
./target/release/logicodex tokens examples/01_tambah_pemula.ldx
./target/release/logicodex tokens examples/01_tambah_pakar.ldx
```

Although the original lexemes differ, the important token identities are the same: `Start`, `Let`, `Identifier`, `Assign`, `Integer`, `If`, `Print`, `Else`, and `End`. This is the intended zero-penalty dynamic dictionary mapping model.

To validate the generated LLVM IR, compile with `--emit-ir`.

```bash
./target/release/logicodex compile examples/01_tambah_pemula.ldx --emit-ir -o ./tambah_pemula
cat examples/01_tambah_pemula.ll 2>/dev/null || cat ./tambah_pemula.ll
```

You should see LLVM IR containing direct arithmetic, comparison, conditional branch blocks, and calls to the runtime symbol `logicodex_print_i64`. There is no bytecode interpreter, virtual machine dispatch loop, or garbage collector state in the emitted program.

| Check | Command | Expected Evidence |
|---|---|---|
| Lexer mapping | `logicodex tokens examples/01_tambah_pemula.ldx` | Novice words normalize to compiler token kinds. |
| Expert mapping | `logicodex tokens examples/01_tambah_pakar.ldx` | Expert shorthand normalizes to matching compiler token kinds. |
| Static checking | `logicodex check examples/01_tambah_pemula.ldx` | Reports semantic validation success. |
| LLVM output | `logicodex compile ... --emit-ir` | Produces `.ll` beside the generated object path. |
| Native execution | Run compiled binary | Prints `1`. |

## 6. Repository File Map

```text
logicodex/
├── Cargo.toml
├── MANUAL.md
├── README.md
├── dict/
│   └── core_map.json
├── stdlib/
│   └── io.ldx
├── examples/
│   ├── 01_tambah_pemula.ldx
│   └── 01_tambah_pakar.ldx
└── src/
    ├── main.rs
    ├── ast.rs
    ├── lexer.rs
    ├── parser.rs
    ├── semantic.rs
    ├── codegen.rs
    └── os/
        ├── mod.rs
        ├── windows.rs
        ├── linux.rs
        └── target.rs
```

## 7. Notes on Native Runtime Design

The compiler lowers `PAPAR` and `print` to a single intrinsic symbol named `logicodex_print_i64`. The OS module supplies target-specific assembly for that symbol. On Linux, the runtime writes to file descriptor `1` through the native `syscall` instruction. On Windows, the runtime is structured around Win32 console output symbols. This design keeps the language runtime tiny and avoids a VM or garbage collector.

Memory management in Phase 1 is intentionally simple. Values are fixed-width scalars stored in LLVM stack slots. Rust owns the compiler's memory through RAII, and compiled Logicodex programs do not allocate heap memory for Phase 1 constructs. As a result, there is no Phase 1 heap lifetime model and no garbage collector requirement.

## References

[1]: https://www.rust-lang.org/tools/install "Install Rust"
[2]: https://llvm.org/docs/GettingStarted.html "LLVM Getting Started"
[3]: https://github.com/TheDan64/inkwell "Inkwell LLVM Bindings for Rust"
'''

for rel, content in files.items():
    path = root / rel
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding='utf-8')
print(f'created {len(files)} files under {root}')
