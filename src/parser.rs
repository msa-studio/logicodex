// =========================================================================
// Project: Logicodex Language Engine (Phase 2 Deployment Integration)
// Version: v1.21-alpha (Formal Specifications & Zero-Overhead Severity Model)
// Architect & Creator: Mohamad Supardi Abdul (mymsastudio@gmail.com)
// Copyright (c) 2026. All Rights Reserved.
// Licensed under permissive dual-license: MIT & Apache License 2.0
// =========================================================================
use crate::ast::{BinaryOp, Expr, Program, Stmt};
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
