// =========================================================================
// Project: Logicodex Language Engine (Phase 2 Deployment Integration)
// Version: v1.21-alpha (Specification Baseline & Practical Severity Roadmap)
// Architect & Creator: Mohamad Supardi Abdul (mymsastudio@gmail.com)
// Copyright (c) 2026. All Rights Reserved.
// Licensed under permissive dual-license: MIT & Apache License 2.0
// =========================================================================
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenKind {
    Start,
    End,
    Let,
    Mut,
    If,
    Then,
    Else,
    While,
    Loop,
    Break,
    Continue,
    Print,
    Hardware,
    HwZone,
    Address,
    Use,
    Fn,
    Return,
    Ffi,
    CInterop,
    Resource,
    Drop,
    Unsafe,
    Extern,
    Struct,
    Enum,
    True,
    False,
    TypeI32,
    TypeI64,
    TypeU16,
    TypeU32,
    TypeStr,
    TypeF64,
    TypeBool,
    Ptr,
    // Ketuk 1: Core Memory Model
    Buffer,
    // Ketuk 2: Result Abstraction
    Result,
    Ok,
    Err,
    Match,
    ArrowFat,
    Underscore,
    // Ketuk 3: File Handle ABI
    FileHandle,
    Open,
    Close,
    Read,
    Write,
    Seek,
    Dot,
    // v1.30.1-alpha: Threading Foundation
    Actor,
    Channel,
    Spawn,
    Send,
    Recv,
    Join,
    // v1.30.1-alpha Phase 3: Backpressure + Scheduler
    TrySend,
    TryRecv,
    Yield,
    Sleep,
    Assign,
    Plus,
    Minus,
    Star,
    Slash,
    And,
    Or,
    BitAnd,
    BitOr,
    ShiftL,
    ShiftR,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    EqualEqual,
    BangEqual,
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    Colon,
    Semicolon,
    Comma,
    Arrow,
    StringLiteral,
    Identifier,
    Integer,
    Newline,
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
        Self {
            kind,
            lexeme: lexeme.into(),
            line,
            column,
        }
    }
}

#[derive(Debug, Deserialize)]
struct CoreMap {
    tokens: TokenDictionary,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum TokenDictionary {
    V2(HashMap<String, DictionaryTokenV2>),
    V1(Vec<DictionaryTokenV1>),
}

#[derive(Debug, Deserialize)]
struct DictionaryTokenV2 {
    expert: String,
    primary_ms: String,
    #[serde(default)]
    aliases: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct DictionaryTokenV1 {
    identity: String,
    lexemes: Vec<String>,
}

// core_map.json is consumed strictly in this lexer layer. Surface lexemes such as
// `MULA`, `START`, `{`, `PERKAKASAN`, `HARDWARE`, and `hw` are normalized into
// canonical TokenKind values before Parser::new receives the token stream.
#[derive(Debug, Clone)]
pub struct Lexicon {
    lexeme_to_kind: HashMap<String, TokenKind>,
    symbolic_lexemes: Vec<String>,
}

#[derive(Debug, Error)]
pub enum LexError {
    #[error(
        "gagal membaca dictionary {path}: {source} / failed to read dictionary {path}: {source}"
    )]
    DictionaryRead {
        path: String,
        source: std::io::Error,
    },
    #[error("gagal menghuraikan dictionary {path}: {source} / failed to parse dictionary {path}: {source}")]
    DictionaryParse {
        path: String,
        source: serde_json::Error,
    },
    #[error(
        "identiti token dictionary tidak diketahui `{0}` / unknown dictionary token identity `{0}`"
    )]
    UnknownIdentity(String),
    #[error("aksara tidak dijangka `{ch}` pada {line}:{column} / unexpected character `{ch}` at {line}:{column}")]
    UnexpectedCharacter {
        ch: char,
        line: usize,
        column: usize,
    },
    #[error("literal integer `{literal}` pada {line}:{column} tidak muat dalam i64 / integer literal `{literal}` at {line}:{column} does not fit in i64")]
    IntegerOverflow {
        literal: String,
        line: usize,
        column: usize,
    },
    #[error("literal string tidak ditutup pada {line}:{column} / unterminated string literal at {line}:{column}")]
    UnterminatedString { line: usize, column: usize },
}

impl TryFrom<&str> for TokenKind {
    type Error = LexError;

    fn try_from(identity: &str) -> Result<Self, Self::Error> {
        match identity {
            "START" | "BEGIN" => Ok(TokenKind::Start),
            "END" => Ok(TokenKind::End),
            "LET" => Ok(TokenKind::Let),
            "MUT" | "MUTABLE" => Ok(TokenKind::Mut),
            "IF" => Ok(TokenKind::If),
            "THEN" => Ok(TokenKind::Then),
            "ELSE" => Ok(TokenKind::Else),
            "WHILE" => Ok(TokenKind::While),
            "LOOP" => Ok(TokenKind::Loop),
            "LOOP_BREAK" | "BREAK" => Ok(TokenKind::Break),
            "LOOP_CONTINUE" | "CONTINUE" => Ok(TokenKind::Continue),
            "PRINT" => Ok(TokenKind::Print),
            "HW" | "HARDWARE" => Ok(TokenKind::Hardware),
            "HW_ZONE" => Ok(TokenKind::HwZone),
            "ADDR" | "ADDRESS" => Ok(TokenKind::Address),
            "USE" => Ok(TokenKind::Use),
            "FN" | "FUNCTION" => Ok(TokenKind::Fn),
            "RETURN" => Ok(TokenKind::Return),
            "FFI" | "FOREIGN_INTERFACE" => Ok(TokenKind::Ffi),
            "C_INTEROP" | "C_LEGACY" => Ok(TokenKind::CInterop),
            "RESOURCE" => Ok(TokenKind::Resource),
            "DROP" => Ok(TokenKind::Drop),
            "UNSAFE" => Ok(TokenKind::Unsafe),
            "EXTERN" => Ok(TokenKind::Extern),
            "STRUCT" => Ok(TokenKind::Struct),
            "ENUM" => Ok(TokenKind::Enum),
            "TRUE" => Ok(TokenKind::True),
            "FALSE" => Ok(TokenKind::False),
            "I32" => Ok(TokenKind::TypeI32),
            "I64" => Ok(TokenKind::TypeI64),
            "U16" => Ok(TokenKind::TypeU16),
            "U32" => Ok(TokenKind::TypeU32),
            "STR" | "STRING" => Ok(TokenKind::TypeStr),
            "F64" => Ok(TokenKind::TypeF64),
            "BOOL" => Ok(TokenKind::TypeBool),
            "PTR" => Ok(TokenKind::Ptr),
            "BUFFER" => Ok(TokenKind::Buffer),
            "RESULT" => Ok(TokenKind::Result),
            "OK" => Ok(TokenKind::Ok),
            "ERR" => Ok(TokenKind::Err),
            "MATCH" => Ok(TokenKind::Match),
            "ARROW_FAT" => Ok(TokenKind::ArrowFat),
            "UNDERSCORE" => Ok(TokenKind::Underscore),
            "FILE_HANDLE" => Ok(TokenKind::FileHandle),
            "OPEN" => Ok(TokenKind::Open),
            "ACTOR" => Ok(TokenKind::Actor),
            "CHANNEL" => Ok(TokenKind::Channel),
            "SPAWN" => Ok(TokenKind::Spawn),
            "SEND" => Ok(TokenKind::Send),
            "RECV" => Ok(TokenKind::Recv),
            "JOIN" => Ok(TokenKind::Join),
            "TRY_SEND" => Ok(TokenKind::TrySend),
            "TRY_RECV" => Ok(TokenKind::TryRecv),
            "YIELD" => Ok(TokenKind::Yield),
            "SLEEP" => Ok(TokenKind::Sleep),
            "CLOSE" => Ok(TokenKind::Close),
            "READ" => Ok(TokenKind::Read),
            "WRITE" => Ok(TokenKind::Write),
            "SEEK" => Ok(TokenKind::Seek),
            "DOT" => Ok(TokenKind::Dot),
            "ASSIGN" => Ok(TokenKind::Assign),
            "PLUS" => Ok(TokenKind::Plus),
            "MINUS" => Ok(TokenKind::Minus),
            "STAR" => Ok(TokenKind::Star),
            "SLASH" => Ok(TokenKind::Slash),
            "AND" => Ok(TokenKind::And),
            "OR" => Ok(TokenKind::Or),
            "BIT_AND" => Ok(TokenKind::BitAnd),
            "BIT_OR" => Ok(TokenKind::BitOr),
            "SHIFT_L" => Ok(TokenKind::ShiftL),
            "SHIFT_R" => Ok(TokenKind::ShiftR),
            "GREATER" => Ok(TokenKind::Greater),
            "GREATER_EQUAL" => Ok(TokenKind::GreaterEqual),
            "LESS" => Ok(TokenKind::Less),
            "LESS_EQUAL" => Ok(TokenKind::LessEqual),
            "EQUAL_EQUAL" => Ok(TokenKind::EqualEqual),
            "BANG_EQUAL" => Ok(TokenKind::BangEqual),
            "LEFT_PAREN" => Ok(TokenKind::LeftParen),
            "RIGHT_PAREN" => Ok(TokenKind::RightParen),
            "LEFT_BRACKET" => Ok(TokenKind::LeftBracket),
            "RIGHT_BRACKET" => Ok(TokenKind::RightBracket),
            "COLON" => Ok(TokenKind::Colon),
            "SEMICOLON" => Ok(TokenKind::Semicolon),
            "COMMA" => Ok(TokenKind::Comma),
            "ARROW" => Ok(TokenKind::Arrow),
            other => Err(LexError::UnknownIdentity(other.to_string())),
        }
    }
}

fn strip_json_line_comments(input: &str) -> String {
    input
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

impl Lexicon {
    pub fn from_path(path: &Path) -> Result<Self, LexError> {
        let raw = fs::read_to_string(path).map_err(|source| LexError::DictionaryRead {
            path: path.display().to_string(),
            source,
        })?;
        let sanitized = strip_json_line_comments(&raw);
        let map: CoreMap =
            serde_json::from_str(&sanitized).map_err(|source| LexError::DictionaryParse {
                path: path.display().to_string(),
                source,
            })?;
        let mut lexeme_to_kind = HashMap::new();
        let mut symbolic_lexemes = Vec::new();

        match map.tokens {
            TokenDictionary::V2(entries) => {
                for (identity, entry) in entries {
                    let kind = TokenKind::try_from(identity.as_str())?;
                    let mut lexemes = vec![entry.expert, entry.primary_ms];
                    lexemes.extend(entry.aliases);
                    for lexeme in lexemes {
                        Self::register_lexeme(
                            &mut lexeme_to_kind,
                            &mut symbolic_lexemes,
                            lexeme,
                            kind,
                        );
                    }
                }
            }
            TokenDictionary::V1(entries) => {
                for entry in entries {
                    let kind = TokenKind::try_from(entry.identity.as_str())?;
                    for lexeme in entry.lexemes {
                        Self::register_lexeme(
                            &mut lexeme_to_kind,
                            &mut symbolic_lexemes,
                            lexeme,
                            kind,
                        );
                    }
                }
            }
        }
        for (lexeme, kind) in default_aliases() {
            if lexeme.chars().all(|c| !c.is_alphanumeric() && c != '_') || *lexeme == "->" {
                symbolic_lexemes.push(lexeme.to_string());
            }
            lexeme_to_kind.entry(lexeme.to_string()).or_insert(*kind);
        }
        symbolic_lexemes.sort_by_key(|s| std::cmp::Reverse(s.len()));
        symbolic_lexemes.dedup();
        Ok(Self {
            lexeme_to_kind,
            symbolic_lexemes,
        })
    }

    fn register_lexeme(
        lexeme_to_kind: &mut HashMap<String, TokenKind>,
        symbolic_lexemes: &mut Vec<String>,
        lexeme: String,
        kind: TokenKind,
    ) {
        if lexeme.chars().all(|c| !c.is_alphanumeric() && c != '_') || lexeme == "->" {
            symbolic_lexemes.push(lexeme.clone());
        }
        lexeme_to_kind.insert(lexeme, kind);
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
        Self {
            source,
            chars: source.chars().collect(),
            index: 0,
            line: 1,
            column: 1,
            lexicon,
        }
    }

    pub fn tokenize(mut self) -> Result<Vec<Token>, LexError> {
        let mut tokens = Vec::new();
        while !self.is_at_end() {
            let ch = self.peek();
            match ch {
                ' ' | '\t' => {
                    self.advance();
                }
                '\r' => {
                    let line = self.line;
                    let column = self.column;
                    self.advance();
                    if !self.is_at_end() && self.peek() == '\n' {
                        self.advance_line();
                    } else {
                        self.line += 1;
                        self.column = 1;
                    }
                    tokens.push(Token::new(TokenKind::Newline, "\n", line, column));
                }
                '\n' => {
                    let line = self.line;
                    let column = self.column;
                    self.advance_line();
                    tokens.push(Token::new(TokenKind::Newline, "\n", line, column));
                }
                '#' => {
                    self.skip_comment();
                }
                '/' if self.peek_next() == Some('/') => {
                    self.skip_comment();
                }
                '"' => tokens.push(self.string_literal()?),
                c if c.is_ascii_digit() => tokens.push(self.integer()?),
                c if is_ident_start(c) => tokens.push(self.identifier_or_keyword()),
                _ => {
                    if let Some(token) = self.symbolic() {
                        tokens.push(token);
                    } else {
                        return Err(LexError::UnexpectedCharacter {
                            ch,
                            line: self.line,
                            column: self.column,
                        });
                    }
                }
            }
        }
        tokens.push(Token::new(TokenKind::Eof, "", self.line, self.column));
        Ok(tokens)
    }

    fn is_at_end(&self) -> bool {
        self.index >= self.chars.len()
    }
    fn peek(&self) -> char {
        self.chars[self.index]
    }
    fn peek_next(&self) -> Option<char> {
        self.chars.get(self.index + 1).copied()
    }

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
        while !self.is_at_end() && self.peek() != '\n' {
            self.advance();
        }
    }

    fn integer(&mut self) -> Result<Token, LexError> {
        let start = self.index;
        let line = self.line;
        let column = self.column;
        while !self.is_at_end() && self.peek().is_ascii_digit() {
            self.advance();
        }
        let literal: String = self.chars[start..self.index].iter().collect();
        literal
            .parse::<i64>()
            .map_err(|_| LexError::IntegerOverflow {
                literal: literal.clone(),
                line,
                column,
            })?;
        Ok(Token::new(TokenKind::Integer, literal, line, column))
    }

    fn string_literal(&mut self) -> Result<Token, LexError> {
        let line = self.line;
        let column = self.column;
        self.advance();
        let start = self.index;
        while !self.is_at_end() {
            if self.peek() == '"' {
                let literal: String = self.chars[start..self.index].iter().collect();
                self.advance();
                return Ok(Token::new(TokenKind::StringLiteral, literal, line, column));
            }
            if self.peek() == '\n' {
                return Err(LexError::UnterminatedString { line, column });
            }
            self.advance();
        }
        Err(LexError::UnterminatedString { line, column })
    }

    fn identifier_or_keyword(&mut self) -> Token {
        let start = self.index;
        let line = self.line;
        let column = self.column;
        while !self.is_at_end() && is_ident_continue(self.peek()) {
            self.advance();
        }
        let lexeme: String = self.chars[start..self.index].iter().collect();
        let kind = self
            .lexicon
            .keyword(&lexeme)
            .unwrap_or(TokenKind::Identifier);
        Token::new(kind, lexeme, line, column)
    }

    fn symbolic(&mut self) -> Option<Token> {
        for symbol in &self.lexicon.symbolic_lexemes {
            if self.source[self.byte_offset()..].starts_with(symbol) {
                let line = self.line;
                let column = self.column;
                for _ in symbol.chars() {
                    self.advance();
                }
                let kind = self
                    .lexicon
                    .keyword(symbol)
                    .expect("symbol must be present in lexeme map");
                return Some(Token::new(kind, symbol.clone(), line, column));
            }
        }
        None
    }

    fn byte_offset(&self) -> usize {
        self.chars[..self.index].iter().map(|c| c.len_utf8()).sum()
    }
}

fn default_aliases() -> &'static [(&'static str, TokenKind)] {
    &[
        ("MULA", TokenKind::Start),
        ("START", TokenKind::Start),
        ("BEGIN", TokenKind::Start),
        ("{", TokenKind::Start),
        ("TAMAT", TokenKind::End),
        ("END", TokenKind::End),
        ("}", TokenKind::End),
        ("BINA", TokenKind::Let),
        ("let", TokenKind::Let),
        ("CREATE", TokenKind::Let),
        ("MUTASI", TokenKind::Mut),
        ("mut", TokenKind::Mut),
        ("MUTABLE", TokenKind::Mut),
        ("JIKA", TokenKind::If),
        ("if", TokenKind::If),
        ("IF", TokenKind::If),
        ("MAKA", TokenKind::Then),
        ("then", TokenKind::Then),
        ("THEN", TokenKind::Then),
        ("MELAINKAN", TokenKind::Else),
        ("JIKALAU_TIDAK", TokenKind::Else),
        ("else", TokenKind::Else),
        ("ELSE", TokenKind::Else),
        ("SELAGI", TokenKind::While),
        ("selagi", TokenKind::While),
        ("while", TokenKind::While),
        ("WHILE", TokenKind::While),
        ("ULANG", TokenKind::Loop),
        ("ulang", TokenKind::Loop),
        ("loop", TokenKind::Loop),
        ("LOOP", TokenKind::Loop),
        ("HENTI", TokenKind::Break),
        ("henti", TokenKind::Break),
        ("break", TokenKind::Break),
        ("BREAK", TokenKind::Break),
        ("TERUS", TokenKind::Continue),
        ("LANGKAU", TokenKind::Continue),
        ("langkau", TokenKind::Continue),
        ("continue", TokenKind::Continue),
        ("CONTINUE", TokenKind::Continue),
        ("PAPAR", TokenKind::Print),
        ("print", TokenKind::Print),
        ("PERKAKASAN", TokenKind::Hardware),
        ("KAWASAN_PERKAKAS", TokenKind::Hardware),
        ("hw", TokenKind::Hardware),
        ("HARDWARE", TokenKind::Hardware),
        ("ZON_PERKAKASAN", TokenKind::HwZone),
        ("hw_unsafe", TokenKind::HwZone),
        ("HW_ZONE", TokenKind::HwZone),
        ("ALAMAT", TokenKind::Address),
        ("addr", TokenKind::Address),
        ("ADDRESS", TokenKind::Address),
        ("GUNA", TokenKind::Use),
        ("use", TokenKind::Use),
        ("FUNGSI", TokenKind::Fn),
        ("fn", TokenKind::Fn),
        ("FUNCTION", TokenKind::Fn),
        ("PULANG", TokenKind::Return),
        ("return", TokenKind::Return),
        ("RETURN", TokenKind::Return),
        ("PAUTAN", TokenKind::Ffi),
        ("ffi", TokenKind::Ffi),
        ("FOREIGN_INTERFACE", TokenKind::Ffi),
        ("C_LUAR", TokenKind::CInterop),
        ("c", TokenKind::CInterop),
        ("C_LEGACY", TokenKind::CInterop),
        ("SUMBER", TokenKind::Resource),
        ("resource", TokenKind::Resource),
        ("RESOURCE", TokenKind::Resource),
        ("LEPAS", TokenKind::Drop),
        ("drop", TokenKind::Drop),
        ("DROP", TokenKind::Drop),
        ("BERISIKO", TokenKind::Unsafe),
        ("berisiko", TokenKind::Unsafe),
        ("unsafe", TokenKind::Unsafe),
        ("UNSAFE", TokenKind::Unsafe),
        ("LUAR", TokenKind::Extern),
        ("luar", TokenKind::Extern),
        ("extern", TokenKind::Extern),
        ("EXTERN", TokenKind::Extern),
        ("BENTUK", TokenKind::Struct),
        ("bentuk", TokenKind::Struct),
        ("struct", TokenKind::Struct),
        ("STRUCT", TokenKind::Struct),
        ("PILIHAN", TokenKind::Enum),
        ("pilihan", TokenKind::Enum),
        ("enum", TokenKind::Enum),
        ("ENUM", TokenKind::Enum),
        ("BENAR", TokenKind::True),
        ("benar", TokenKind::True),
        ("true", TokenKind::True),
        ("SALAH", TokenKind::False),
        ("PALSU", TokenKind::False),
        ("palsu", TokenKind::False),
        ("false", TokenKind::False),
        ("I32", TokenKind::TypeI32),
        ("i32", TokenKind::TypeI32),
        ("INT32", TokenKind::TypeI32),
        ("I64", TokenKind::TypeI64),
        ("U16", TokenKind::TypeU16),
        ("U32", TokenKind::TypeU32),
        ("u32", TokenKind::TypeU32),
        ("UINT32", TokenKind::TypeU32),
        ("TEKS", TokenKind::TypeStr),
        ("str", TokenKind::TypeStr),
        ("STRING", TokenKind::TypeStr),
        ("F64", TokenKind::TypeF64),
        ("Bool", TokenKind::TypeBool),
        ("BOOL", TokenKind::TypeBool),
        ("PTR", TokenKind::Ptr),
        ("ptr", TokenKind::Ptr),
        ("BUFFER", TokenKind::Buffer),
        ("Buffer", TokenKind::Buffer),
        ("buffer", TokenKind::Buffer),
        ("RESULT", TokenKind::Result),
        ("Result", TokenKind::Result),
        ("result", TokenKind::Result),
        ("OK", TokenKind::Ok),
        ("Ok", TokenKind::Ok),
        ("ok", TokenKind::Ok),
        ("ERR", TokenKind::Err),
        ("Err", TokenKind::Err),
        ("err", TokenKind::Err),
        ("MATCH", TokenKind::Match),
        ("Match", TokenKind::Match),
        ("match", TokenKind::Match),
        ("_", TokenKind::Underscore),
        ("FILE_HANDLE", TokenKind::FileHandle),
        ("FileHandle", TokenKind::FileHandle),
        ("file_handle", TokenKind::FileHandle),
        ("ACTOR", TokenKind::Actor),
        ("Actor", TokenKind::Actor),
        ("actor", TokenKind::Actor),
        ("CHANNEL", TokenKind::Channel),
        ("Channel", TokenKind::Channel),
        ("channel", TokenKind::Channel),
        ("SPAWN", TokenKind::Spawn),
        ("Spawn", TokenKind::Spawn),
        ("spawn", TokenKind::Spawn),
        ("SEND", TokenKind::Send),
        ("Send", TokenKind::Send),
        ("send", TokenKind::Send),
        ("RECV", TokenKind::Recv),
        ("Recv", TokenKind::Recv),
        ("recv", TokenKind::Recv),
        ("JOIN", TokenKind::Join),
        ("Join", TokenKind::Join),
        ("join", TokenKind::Join),
        ("TRY_SEND", TokenKind::TrySend),
        ("TrySend", TokenKind::TrySend),
        ("try_send", TokenKind::TrySend),
        ("TRY_RECV", TokenKind::TryRecv),
        ("TryRecv", TokenKind::TryRecv),
        ("try_recv", TokenKind::TryRecv),
        ("YIELD", TokenKind::Yield),
        ("Yield", TokenKind::Yield),
        ("yield", TokenKind::Yield),
        ("SLEEP", TokenKind::Sleep),
        ("Sleep", TokenKind::Sleep),
        ("sleep", TokenKind::Sleep),
        ("OPEN", TokenKind::Open),
        ("Open", TokenKind::Open),
        ("open", TokenKind::Open),
        ("CLOSE", TokenKind::Close),
        ("Close", TokenKind::Close),
        ("close", TokenKind::Close),
        ("READ", TokenKind::Read),
        ("Read", TokenKind::Read),
        ("read", TokenKind::Read),
        ("WRITE", TokenKind::Write),
        ("Write", TokenKind::Write),
        ("write", TokenKind::Write),
        ("SEEK", TokenKind::Seek),
        ("Seek", TokenKind::Seek),
        ("seek", TokenKind::Seek),
        (".", TokenKind::Dot),
        ("=", TokenKind::Assign),
        ("+", TokenKind::Plus),
        ("-", TokenKind::Minus),
        ("*", TokenKind::Star),
        ("/", TokenKind::Slash),
        ("&&", TokenKind::And),
        ("dan", TokenKind::And),
        ("DAN", TokenKind::And),
        ("AND", TokenKind::And),
        ("||", TokenKind::Or),
        ("atau", TokenKind::Or),
        ("ATAU", TokenKind::Or),
        ("OR", TokenKind::Or),
        ("bit_dan", TokenKind::BitAnd),
        ("DAN_BIT", TokenKind::BitAnd),
        ("&", TokenKind::BitAnd),
        ("BIT_AND", TokenKind::BitAnd),
        ("bit_atau", TokenKind::BitOr),
        ("ATAU_BIT", TokenKind::BitOr),
        ("|", TokenKind::BitOr),
        ("BIT_OR", TokenKind::BitOr),
        ("<<", TokenKind::ShiftL),
        ("anjak_kiri", TokenKind::ShiftL),
        ("ANJAK_KIRI", TokenKind::ShiftL),
        ("SHIFT_L", TokenKind::ShiftL),
        (">>", TokenKind::ShiftR),
        ("anjak_kanan", TokenKind::ShiftR),
        ("ANJAK_KANAN", TokenKind::ShiftR),
        ("SHIFT_R", TokenKind::ShiftR),
        (">=", TokenKind::GreaterEqual),
        (">", TokenKind::Greater),
        ("<=", TokenKind::LessEqual),
        ("<", TokenKind::Less),
        ("==", TokenKind::EqualEqual),
        ("!=", TokenKind::BangEqual),
        ("(", TokenKind::LeftParen),
        (")", TokenKind::RightParen),
        ("[", TokenKind::LeftBracket),
        ("]", TokenKind::RightBracket),
        (":", TokenKind::Colon),
        (";", TokenKind::Semicolon),
        (",", TokenKind::Comma),
        ("->", TokenKind::Arrow),
        ("=>", TokenKind::ArrowFat),
    ]
}

fn is_ident_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_'
}
fn is_ident_continue(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
}

#[cfg(test)]
mod tests {
    use super::{Lexer, Lexicon, TokenKind};
    use std::path::Path;

    fn token_kinds_for(source: &str) -> Vec<TokenKind> {
        let lexicon = Lexicon::from_path(Path::new("dict/core_map.json"))
            .expect("core_map.json should load for lexer tests");
        Lexer::new(source, &lexicon)
            .tokenize()
            .expect("source should tokenize")
            .into_iter()
            .map(|token| token.kind)
            .collect()
    }

    #[test]
    fn token_expansion_aliases_normalize_to_canonical_kinds() {
        let kinds = token_kinds_for(
            "while selagi loop ulang break henti continue langkau && dan || atau & bit_dan | bit_atau << anjak_kiri >> anjak_kanan struct bentuk enum pilihan true benar false palsu unsafe berisiko extern luar",
        );
        assert_eq!(
            kinds,
            vec![
                TokenKind::While,
                TokenKind::While,
                TokenKind::Loop,
                TokenKind::Loop,
                TokenKind::Break,
                TokenKind::Break,
                TokenKind::Continue,
                TokenKind::Continue,
                TokenKind::And,
                TokenKind::And,
                TokenKind::Or,
                TokenKind::Or,
                TokenKind::BitAnd,
                TokenKind::BitAnd,
                TokenKind::BitOr,
                TokenKind::BitOr,
                TokenKind::ShiftL,
                TokenKind::ShiftL,
                TokenKind::ShiftR,
                TokenKind::ShiftR,
                TokenKind::Struct,
                TokenKind::Struct,
                TokenKind::Enum,
                TokenKind::Enum,
                TokenKind::True,
                TokenKind::True,
                TokenKind::False,
                TokenKind::False,
                TokenKind::Unsafe,
                TokenKind::Unsafe,
                TokenKind::Extern,
                TokenKind::Extern,
                TokenKind::Eof,
            ]
        );
    }
}
