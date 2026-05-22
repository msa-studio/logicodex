// =========================================================================
// Project: Logicodex Language Engine (Phase 2 Deployment Integration)
// Version: v1.21-alpha (Formal Specifications & Zero-Overhead Severity Model)
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

// core_map.json is consumed strictly in this lexer layer. Surface lexemes such as
// `MULA`, `BEGIN`, or `{` are normalized into canonical TokenKind values before
// Parser::new receives the token stream. This is token-level normalization, not
// parser-side macro rewriting or grammar desugaring.
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

fn strip_json_line_comments(input: &str) -> String {
    input
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

impl Lexicon {
    pub fn from_path(path: &Path) -> Result<Self, LexError> {
        let raw = fs::read_to_string(path).map_err(|source| LexError::DictionaryRead { path: path.display().to_string(), source })?;
        let sanitized = strip_json_line_comments(&raw);
        let map: CoreMap = serde_json::from_str(&sanitized).map_err(|source| LexError::DictionaryParse { path: path.display().to_string(), source })?;
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
                '/' if self.peek_next() == Some('/') => { self.skip_comment(); }
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
    fn peek_next(&self) -> Option<char> { self.chars.get(self.index + 1).copied() }

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
