// =========================================================================
// Project: Logicodex Language Engine (Phase 2 Deployment Integration)
// Version: v1.21-alpha (Specification Baseline & Practical Severity Roadmap)
// Architect & Creator: Mohamad Supardi Abdul (mymsastudio@gmail.com)
// Copyright (c) 2026. All Rights Reserved.
// Licensed under permissive dual-license: MIT & Apache License 2.0
// =========================================================================
use crate::ast::{BinaryOp, Expr, Param, Program, Stmt, Type};
use crate::lexer::{Token, TokenKind};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("expected {expected} at {line}:{column}, found `{found}`")]
    Expected {
        expected: String,
        found: String,
        line: usize,
        column: usize,
    },
    #[error("unexpected token `{found}` at {line}:{column}")]
    Unexpected {
        found: String,
        line: usize,
        column: usize,
    },
    #[error("invalid integer literal `{literal}` at {line}:{column}")]
    InvalidInteger {
        literal: String,
        line: usize,
        column: usize,
    },
}

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Program, ParseError> {
        self.consume_optional_semicolons();
        let wrapped = self.matches(TokenKind::Start);
        let mut statements = Vec::new();
        while !self.is_at_end() && !self.check(TokenKind::End) {
            self.consume_optional_semicolons();
            if self.check(TokenKind::End) || self.is_at_end() {
                break;
            }
            statements.push(self.declaration_or_statement()?);
        }
        if wrapped {
            self.consume(TokenKind::End, "program terminator TAMAT or }")?;
        }
        self.consume_optional_semicolons();
        self.consume(TokenKind::Eof, "end of file")?;
        Ok(Program::new(statements))
    }

    fn declaration_or_statement(&mut self) -> Result<Stmt, ParseError> {
        if self.matches(TokenKind::Use) {
            return self.use_declaration();
        }
        if self.matches(TokenKind::Hardware) {
            return self.hardware_declaration();
        }
        if self.matches(TokenKind::HwZone) {
            return self.hardware_zone_block();
        }
        if self.matches(TokenKind::Fn) {
            return self.function_definition();
        }
        self.statement()
    }

    fn use_declaration(&mut self) -> Result<Stmt, ParseError> {
        let module = self
            .consume(TokenKind::Identifier, "module name after use")?
            .lexeme
            .clone();
        self.consume(TokenKind::Semicolon, "; after use declaration")?;
        Ok(Stmt::Use { module })
    }

    fn hardware_zone_block(&mut self) -> Result<Stmt, ParseError> {
        let body = self.block()?;
        Ok(Stmt::HardwareZone { body })
    }

    fn hardware_declaration(&mut self) -> Result<Stmt, ParseError> {
        let name = self
            .consume(TokenKind::Identifier, "hardware region identifier")?
            .lexeme
            .clone();
        self.consume(TokenKind::Colon, ": after hardware identifier")?;
        let ty = self.parse_type()?;
        self.consume(TokenKind::Assign, "= in hardware declaration")?;
        self.consume(TokenKind::Address, "addr before literal hardware address")?;
        let address =
            Expr::AddressOfLiteral(self.consume_integer_literal("literal hardware address")?);
        self.consume(TokenKind::Semicolon, "; after hardware declaration")?;
        Ok(Stmt::HardwareDecl { name, ty, address })
    }

    fn function_definition(&mut self) -> Result<Stmt, ParseError> {
        let name = self
            .consume(TokenKind::Identifier, "function name")?
            .lexeme
            .clone();
        self.consume(TokenKind::LeftParen, "( after function name")?;
        let mut params = Vec::new();
        if !self.check(TokenKind::RightParen) {
            loop {
                let param_name = self
                    .consume(TokenKind::Identifier, "parameter name")?
                    .lexeme
                    .clone();
                self.consume(TokenKind::Colon, ": after parameter name")?;
                let ty = self.parse_type()?;
                params.push(Param {
                    name: param_name,
                    ty,
                });
                if !self.matches(TokenKind::Comma) {
                    break;
                }
            }
        }
        self.consume(TokenKind::RightParen, ") after parameter list")?;
        let return_type = if self.matches(TokenKind::Arrow) {
            Some(self.parse_type()?)
        } else {
            None
        };
        let body = self.block()?;
        Ok(Stmt::Function {
            name,
            params,
            return_type,
            body,
        })
    }

    fn statement(&mut self) -> Result<Stmt, ParseError> {
        let stmt = if self.matches(TokenKind::Let) {
            self.let_statement()?
        } else if self.matches(TokenKind::Print) {
            self.print_statement()?
        } else if self.matches(TokenKind::Return) {
            self.return_statement()?
        } else if self.matches(TokenKind::If) {
            self.if_statement()?
        } else {
            let value = self.expression()?;
            self.consume(TokenKind::Semicolon, "; after expression")?;
            Stmt::ExprStmt { value }
        };
        self.consume_optional_semicolons();
        Ok(stmt)
    }

    fn let_statement(&mut self) -> Result<Stmt, ParseError> {
        let name = self
            .consume(TokenKind::Identifier, "variable name")?
            .lexeme
            .clone();
        let declared_type = if self.matches(TokenKind::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };
        self.consume(TokenKind::Assign, "=")?;
        let value = self.expression()?;
        self.consume(TokenKind::Semicolon, "; after let statement")?;
        Ok(Stmt::Let {
            name,
            declared_type,
            value,
        })
    }

    fn print_statement(&mut self) -> Result<Stmt, ParseError> {
        let value = self.expression()?;
        self.consume(TokenKind::Semicolon, "; after print statement")?;
        Ok(Stmt::Print { value })
    }

    fn return_statement(&mut self) -> Result<Stmt, ParseError> {
        let value = self.expression()?;
        self.consume(TokenKind::Semicolon, "; after return statement")?;
        Ok(Stmt::Return { value })
    }

    fn if_statement(&mut self) -> Result<Stmt, ParseError> {
        let condition = self.expression()?;
        self.matches(TokenKind::Then);
        let then_branch = self.block()?;
        let else_branch = if self.matches(TokenKind::Else) {
            self.block()?
        } else {
            Vec::new()
        };
        Ok(Stmt::If {
            condition,
            then_branch,
            else_branch,
        })
    }

    fn block(&mut self) -> Result<Vec<Stmt>, ParseError> {
        self.consume(TokenKind::Start, "block start MULA or {")?;
        let mut statements = Vec::new();
        while !self.is_at_end() && !self.check(TokenKind::End) {
            self.consume_optional_semicolons();
            if self.check(TokenKind::End) || self.is_at_end() {
                break;
            }
            statements.push(self.declaration_or_statement()?);
        }
        self.consume(TokenKind::End, "block end TAMAT or }")?;
        Ok(statements)
    }

    fn parse_type(&mut self) -> Result<Type, ParseError> {
        if self.matches(TokenKind::TypeI32) {
            return Ok(Type::I32);
        }
        if self.matches(TokenKind::TypeI64) {
            return Ok(Type::I64);
        }
        if self.matches(TokenKind::TypeU16) {
            return Ok(Type::U16);
        }
        if self.matches(TokenKind::TypeU32) {
            return Ok(Type::U32);
        }
        if self.matches(TokenKind::TypeF64) {
            return Ok(Type::F64);
        }
        if self.matches(TokenKind::TypeBool) {
            return Ok(Type::Bool);
        }
        if self.matches(TokenKind::Ptr) {
            self.consume(TokenKind::Less, "< after PTR")?;
            let inner = self.parse_type()?;
            self.consume(TokenKind::Greater, "> after pointer type")?;
            return Ok(Type::Pointer(Box::new(inner)));
        }
        let t = self.peek();
        Err(ParseError::Expected {
            expected: "type".to_string(),
            found: t.lexeme.clone(),
            line: t.line,
            column: t.column,
        })
    }

    fn expression(&mut self) -> Result<Expr, ParseError> {
        self.equality()
    }

    fn equality(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.comparison()?;
        while self.matches(TokenKind::EqualEqual) || self.matches(TokenKind::BangEqual) {
            let op = match self.previous().kind {
                TokenKind::EqualEqual => BinaryOp::Equal,
                TokenKind::BangEqual => BinaryOp::NotEqual,
                _ => unreachable!(),
            };
            let right = self.comparison()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.term()?;
        while self.matches(TokenKind::Greater)
            || self.matches(TokenKind::GreaterEqual)
            || self.matches(TokenKind::Less)
            || self.matches(TokenKind::LessEqual)
        {
            let op = match self.previous().kind {
                TokenKind::Greater => BinaryOp::Greater,
                TokenKind::GreaterEqual => BinaryOp::GreaterEqual,
                TokenKind::Less => BinaryOp::Less,
                TokenKind::LessEqual => BinaryOp::LessEqual,
                _ => unreachable!(),
            };
            let right = self.term()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.factor()?;
        while self.matches(TokenKind::Plus) || self.matches(TokenKind::Minus) {
            let op = if self.previous().kind == TokenKind::Plus {
                BinaryOp::Add
            } else {
                BinaryOp::Subtract
            };
            let right = self.factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.primary()?;
        while self.matches(TokenKind::Star) || self.matches(TokenKind::Slash) {
            let op = if self.previous().kind == TokenKind::Star {
                BinaryOp::Multiply
            } else {
                BinaryOp::Divide
            };
            let right = self.primary()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn primary(&mut self) -> Result<Expr, ParseError> {
        if self.matches(TokenKind::Integer) {
            let token = self.previous();
            let value = token
                .lexeme
                .parse::<i64>()
                .map_err(|_| ParseError::InvalidInteger {
                    literal: token.lexeme.clone(),
                    line: token.line,
                    column: token.column,
                })?;
            return Ok(Expr::Integer(value));
        }
        if self.matches(TokenKind::True) {
            return Ok(Expr::Boolean(true));
        }
        if self.matches(TokenKind::False) {
            return Ok(Expr::Boolean(false));
        }
        if self.matches(TokenKind::StringLiteral) {
            return Ok(Expr::StringLiteral(self.previous().lexeme.clone()));
        }
        if self.matches(TokenKind::Identifier) {
            return Ok(Expr::Variable(self.previous().lexeme.clone()));
        }
        if self.matches(TokenKind::Address) {
            let value = self.consume_integer_literal("literal after addr")?;
            return Ok(Expr::AddressOfLiteral(value));
        }
        if self.matches(TokenKind::LeftParen) {
            let expr = self.expression()?;
            self.consume(TokenKind::RightParen, ")")?;
            return Ok(Expr::Grouped(Box::new(expr)));
        }
        let t = self.peek();
        Err(ParseError::Unexpected {
            found: t.lexeme.clone(),
            line: t.line,
            column: t.column,
        })
    }

    fn consume_integer_literal(&mut self, expected: &str) -> Result<i64, ParseError> {
        let token = self.consume(TokenKind::Integer, expected)?.clone();
        token
            .lexeme
            .parse::<i64>()
            .map_err(|_| ParseError::InvalidInteger {
                literal: token.lexeme.clone(),
                line: token.line,
                column: token.column,
            })
    }

    fn consume_optional_semicolons(&mut self) {
        while self.matches(TokenKind::Semicolon) {}
    }

    fn matches(&mut self, kind: TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn consume(&mut self, kind: TokenKind, expected: &str) -> Result<&Token, ParseError> {
        if self.check(kind) {
            return Ok(self.advance());
        }
        let t = self.peek();
        Err(ParseError::Expected {
            expected: expected.to_string(),
            found: t.lexeme.clone(),
            line: t.line,
            column: t.column,
        })
    }

    fn check(&self, kind: TokenKind) -> bool {
        !self.is_at_end() && self.peek().kind == kind
    }
    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }
    fn is_at_end(&self) -> bool {
        self.peek().kind == TokenKind::Eof
    }
    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }
    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }
}
