// =========================================================================
// Project: Logicodex Language Engine (Phase 2 Deployment Integration)
// Version: v1.21-alpha (Specification Baseline & Practical Severity Roadmap)
// Architect & Creator: Mohamad Supardi Abdul (mymsastudio@gmail.com)
// Copyright (c) 2026. All Rights Reserved.
// Licensed under permissive dual-license: MIT & Apache License 2.0
// =========================================================================
use crate::ast::{BinaryOp, ExternFnDecl, Expr, MatchArm, MatchPattern, Param, Program, Stmt, Type};
use crate::lexer::{Token, TokenKind};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompilerPipeline {
    /// The one engine: full grammar + HIR lowering.
    V130,
}

impl Default for CompilerPipeline {
    fn default() -> Self {
        CompilerPipeline::V130
    }
}

impl std::str::FromStr for CompilerPipeline {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "v1.30" | "V130" | "130" => Ok(CompilerPipeline::V130),
            // `v1.21` is a deprecated alias; there is now a single engine.
            "v1.21" | "V121" | "121" => Ok(CompilerPipeline::V130),
            _ => Err(format!(
                "pipeline tidak dikenali: {s} / unrecognized pipeline: {s}. Expected: v1.30"
            )),
        }
    }
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("dijangka {expected} pada {line}:{column}, ditemui `{found}` / expected {expected} at {line}:{column}, found `{found}`")]
    Expected {
        expected: String,
        found: String,
        line: usize,
        column: usize,
    },
    #[error("token tidak dijangka `{found}` pada {line}:{column} / unexpected token `{found}` at {line}:{column}")]
    Unexpected {
        found: String,
        line: usize,
        column: usize,
    },
    #[error("literal integer tidak sah `{literal}` pada {line}:{column} / invalid integer literal `{literal}` at {line}:{column}")]
    InvalidInteger {
        literal: String,
        line: usize,
        column: usize,
    },
    #[error("Ralat: Kata kunci '{keyword}' dikenali tetapi belum disokong (unimplemented) dalam v1.21-alpha pada {line}:{column} / Error: Keyword '{keyword}' is recognized but not yet supported (unimplemented) in v1.21-alpha at {line}:{column}")]
    UnimplementedFeature {
        keyword: String,
        line: usize,
        column: usize,
    },
}

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    critical_depth: usize,
    pipeline: CompilerPipeline,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: 0,
            critical_depth: 0,
            pipeline: CompilerPipeline::default(),
        }
    }

    pub fn with_pipeline(mut self, pipeline: CompilerPipeline) -> Self {
        self.pipeline = pipeline;
        self
    }

    pub fn parse(&mut self) -> Result<Program, ParseError> {
        self.consume_layout_separators();
        let wrapped = self.matches(TokenKind::Start);
        let mut statements = Vec::new();
        while !self.is_at_end() && !self.check(TokenKind::End) {
            self.consume_layout_separators();
            if self.check(TokenKind::End) || self.is_at_end() {
                break;
            }
            statements.push(self.declaration_or_statement()?);
        }
        if wrapped {
            self.consume(TokenKind::End, "program terminator TAMAT or }")?;
        }
        self.consume_trailing_layout();
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
        if self.check(TokenKind::Struct) {
            return self.struct_declaration();
        }
        if self.check(TokenKind::Enum) {
            return self.enum_declaration();
        }
        if self.check(TokenKind::Unsafe) {
            return self.unsafe_block();
        }
        if self.check(TokenKind::Extern) {
            return self.extern_block();
        }
        self.statement()
    }

    fn struct_declaration(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // consume 'struct'
        let name = self
            .consume(TokenKind::Identifier, "struct name")?
            .lexeme
            .clone();
        self.consume(TokenKind::Start, "block start MULA or {")?;
        self.consume_newlines();
        let mut fields = Vec::new();
        while !self.check(TokenKind::End) && !self.is_at_end() {
            let field_name = self
                .consume(TokenKind::Identifier, "field name")?
                .lexeme
                .clone();
            self.consume(TokenKind::Colon, ": after field name")?;
            let ty = self.parse_type()?;
            fields.push(Param {
                name: field_name,
                ty,
            });
            self.consume_statement_terminator("; after field declaration", false)?;
            self.consume_newlines();
        }
        self.consume(TokenKind::End, "block end TAMAT or }")?;
        Ok(Stmt::StructDecl { name, fields })
    }

    fn enum_declaration(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // consume 'enum'
        let name = self
            .consume(TokenKind::Identifier, "enum name")?
            .lexeme
            .clone();
        self.consume(TokenKind::Start, "block start MULA or {")?;
        self.consume_newlines();
        let mut variants = Vec::new();
        while !self.check(TokenKind::End) && !self.is_at_end() {
            let variant = self
                .consume(TokenKind::Identifier, "variant name")?
                .lexeme
                .clone();
            variants.push(variant);
            self.consume_statement_terminator("; after variant", false)?;
            self.consume_newlines();
        }
        self.consume(TokenKind::End, "block end TAMAT or }")?;
        Ok(Stmt::EnumDecl { name, variants })
    }

    fn unsafe_block(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // consume 'unsafe'
        let body = self.block()?;
        Ok(Stmt::UnsafeBlock { body })
    }

    fn extern_block(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // consume 'extern'
        let abi = if self.matches(TokenKind::StringLiteral) {
            self.previous().lexeme.clone()
        } else {
            "C".to_string()
        };
        self.consume(TokenKind::Fn, "fn after extern")?;
        let mut functions = Vec::new();
        // Parse at least one extern function
        loop {
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
            self.consume_statement_terminator("; after extern function declaration", false)?;
            functions.push(ExternFnDecl {
                name,
                params,
                return_type,
            });
            if !self.check(TokenKind::Fn) {
                break;
            }
            self.advance(); // consume 'fn'
        }
        Ok(Stmt::ExternBlock { abi, functions })
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
        self.critical_depth += 1;
        let body = self.block();
        self.critical_depth -= 1;
        Ok(Stmt::HardwareZone { body: body? })
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

    /// BUGFIX #2: Peek ahead to detect `buf[index] = value` assignment pattern
    fn peek_index_assignment(&self) -> bool {
        let current = self.current;
        // Need at least: Identifier [ expr ] = expr
        if current + 4 >= self.tokens.len() {
            return false;
        }
        self.tokens[current + 1].kind == TokenKind::LeftBracket
    }

    /// v1.30.1-alpha: Parse `kotak Name { ... }`
    fn actor_statement(&mut self) -> Result<Stmt, ParseError> {
        let name = self.consume(TokenKind::Identifier, "Actor name after 'actor'")?.lexeme.clone();
        let body = self.block()?;
        Ok(Stmt::Actor { name, body })
    }

    // v1.33.0-alpha: Service manifest — deterministic network reactor
    fn service_statement(&mut self) -> Result<Stmt, ParseError> {
        let name = self.consume(TokenKind::Identifier, "Service name after 'service'")?.lexeme.clone();
        self.consume(TokenKind::Start, "block start MULA or {")?;
        
        // Parse service fields: port, requires, handler, policy
        let mut port = 0u16;
        let mut requires = None;
        let mut handler = String::new();
        let mut policy = "Block".to_string();
        
        while !self.check(TokenKind::End) {
            let field = self.consume(TokenKind::Identifier, "service field name")?.lexeme.clone();
            self.consume(TokenKind::Colon, "':' after service field")?;
            
            match field.as_str() {
                "port" => {
                    let val = self.consume(TokenKind::Integer, "port number")?.lexeme.clone();
                    port = val.parse().unwrap_or(0);
                }
                "requires" => {
                    let gate = self.expression()?;
                    if let Expr::Variable(g) = gate {
                        requires = Some(g);
                    } else if let Expr::FieldAccess { base, field } = gate {
                        // Handle Net.Admin format
                        if let Expr::Variable(base_name) = *base {
                            requires = Some(format!("{}.{}", base_name, field));
                        }
                    }
                }
                "handler" => {
                    handler = self.consume(TokenKind::Identifier, "handler name")?.lexeme.clone();
                }
                "policy" => {
                    let pol = self.consume(TokenKind::Identifier, "policy name")?.lexeme.clone();
                    policy = pol;
                }
                _ => {
                    // Abaikan field tak dikenali
                    self.advance(); // skip value
                }
            }
            
            // Optional comma between fields
            if self.matches(TokenKind::Comma) {
                // consume comma
            }
        }
        
        self.consume(TokenKind::End, "block end TAMAT or }")?;
        Ok(Stmt::Service { name, port, requires, handler, policy })
    }

    /// BUGFIX #2: Parse `buf[index] = value` as Stmt::Assign
    fn index_assignment_statement(&mut self) -> Result<Stmt, ParseError> {
        let name = self.consume(TokenKind::Identifier, "buffer name")?.lexeme.clone();
        self.consume(TokenKind::LeftBracket, "'[' after buffer name")?;
        let index = self.expression()?;
        self.consume(TokenKind::RightBracket, "']' after index")?;
        self.consume(TokenKind::Assign, "'=' after index expression")?;
        let value = self.expression()?;
        self.consume_statement_terminator("; after assignment", false)?;
        Ok(Stmt::Assign {
            target: Expr::Index {
                base: Box::new(Expr::Variable(name)),
                index: Box::new(index),
            },
            value,
        })
    }

    /// Peek `name =` (single `=`, not `==`) for plain variable assignment.
    fn peek_simple_assignment(&self) -> bool {
        let current = self.current;
        if current + 1 >= self.tokens.len() {
            return false;
        }
        self.tokens[current + 1].kind == TokenKind::Assign
    }

    /// Parse `name = value` as Stmt::Assign with a Variable target.
    fn variable_assignment_statement(&mut self) -> Result<Stmt, ParseError> {
        let name = self.consume(TokenKind::Identifier, "variable name")?.lexeme.clone();
        self.consume(TokenKind::Assign, "'=' after variable name")?;
        let value = self.expression()?;
        self.consume_statement_terminator("; after assignment", false)?;
        Ok(Stmt::Assign {
            target: Expr::Variable(name),
            value,
        })
    }

    fn statement(&mut self) -> Result<Stmt, ParseError> {
        let stmt = if self.matches(TokenKind::Let) {
            let beginner = self.allows_beginner_line_terminator(&self.previous().lexeme);
            self.let_statement(beginner)?
        } else if self.matches(TokenKind::Print) {
            let beginner = self.allows_beginner_line_terminator(&self.previous().lexeme);
            self.print_statement(beginner)?
        } else if self.matches(TokenKind::Return) {
            let beginner = self.allows_beginner_line_terminator(&self.previous().lexeme);
            self.return_statement(beginner)?
        } else if self.matches(TokenKind::If) {
            self.if_statement()?
        } else if self.matches(TokenKind::While) {
            self.while_statement()?
        } else if self.matches(TokenKind::Loop) {
            self.loop_statement()?
        } else if self.matches(TokenKind::Match) {
            self.match_statement()?
        } else if self.matches(TokenKind::Actor) {
            self.actor_statement()?
        } else if self.matches(TokenKind::Service) {
            self.service_statement()?
        } else if self.matches(TokenKind::Break) {
            let beginner = self.allows_control_flow_line_terminator(&self.previous().lexeme);
            self.consume_statement_terminator("; after break statement", beginner)?;
            Stmt::Break
        } else if self.matches(TokenKind::Continue) {
            let beginner = self.allows_control_flow_line_terminator(&self.previous().lexeme);
            self.consume_statement_terminator("; after continue statement", beginner)?;
            Stmt::Continue
        } else if self.check(TokenKind::Identifier) && self.peek_index_assignment() {
            // BUGFIX #2: buf[index] = value assignment syntax
            self.index_assignment_statement()?
        } else if self.check(TokenKind::Identifier) && self.peek_simple_assignment() {
            // Plain variable assignment: name = value
            self.variable_assignment_statement()?
        } else {
            let expr = self.expression()?;
            if self.matches(TokenKind::Assign) {
                let value = self.expression()?;
                self.consume_statement_terminator("; after assignment", false)?;
                Stmt::Assign { target: expr, value }
            } else {
                self.consume_statement_terminator("; after expression", false)?;
                Stmt::ExprStmt { value: expr }
            }
        };
        self.consume_layout_separators();
        Ok(stmt)
    }

    /// Ketuk 2: Parse `match expr { Ok(v) => body, Err(e) => body }`
    fn match_statement(&mut self) -> Result<Stmt, ParseError> {
        let value = self.expression()?;
        self.consume(TokenKind::Start, "'{' after match expression")?;
        self.consume_newlines();
        let mut arms = Vec::new();
        if !self.check(TokenKind::End) {
            loop {
                arms.push(self.match_arm()?);
                self.consume_newlines();
                if !self.matches(TokenKind::Comma) {
                    break;
                }
                self.consume_newlines();
            }
        }
        self.consume_newlines();
        self.consume(TokenKind::End, "'}' after match arms")?;
        Ok(Stmt::Match { value, arms })
    }

    fn match_arm(&mut self) -> Result<MatchArm, ParseError> {
        let pattern = if self.matches(TokenKind::Ok) {
            self.consume(TokenKind::LeftParen, "'(' after 'Ok'")?;
            let binding = self
                .consume(TokenKind::Identifier, "binding name in Ok pattern")?
                .lexeme
                .clone();
            self.consume(TokenKind::RightParen, "')' after Ok binding")?;
            MatchPattern::Ok { binding }
        } else if self.matches(TokenKind::Err) {
            self.consume(TokenKind::LeftParen, "'(' after 'Err'")?;
            let binding = self
                .consume(TokenKind::Identifier, "binding name in Err pattern")?
                .lexeme
                .clone();
            self.consume(TokenKind::RightParen, "')' after Err binding")?;
            MatchPattern::Err { binding }
        } else if self.matches(TokenKind::Underscore) {
            MatchPattern::Wildcard
        } else if self.matches(TokenKind::Integer) {
            let token = self.previous();
            let value = token
                .lexeme
                .parse::<i64>()
                .map_err(|_| ParseError::InvalidInteger {
                    literal: token.lexeme.clone(),
                    line: token.line,
                    column: token.column,
                })?;
            MatchPattern::Literal(Expr::Integer(value))
        } else if self.matches(TokenKind::StringLiteral) {
            MatchPattern::Literal(Expr::StringLiteral(self.previous().lexeme.clone()))
        } else if self.matches(TokenKind::Identifier) {
            MatchPattern::Identifier(self.previous().lexeme.clone())
        } else {
            return Err(ParseError::Expected {
                expected: "Ok(x), Err(e), _, literal, or identifier pattern".to_string(),
                found: self.peek().lexeme.clone(),
                line: self.peek().line,
                column: self.peek().column,
            });
        };
        self.consume(TokenKind::ArrowFat, "'=>' after match pattern")?;
        // Body: either a single expression or a block
        let body = if self.check(TokenKind::Start) {
            let stmts = self.block()?;
            stmts
        } else {
            let expr = self.expression()?;
            vec![Stmt::ExprStmt { value: expr }]
        };
        Ok(MatchArm { pattern, body })
    }

    fn let_statement(&mut self, beginner: bool) -> Result<Stmt, ParseError> {
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
        self.consume_statement_terminator("; after let statement", beginner)?;
        Ok(Stmt::Let {
            name,
            declared_type,
            value,
        })
    }

    fn print_statement(&mut self, beginner: bool) -> Result<Stmt, ParseError> {
        let value = self.expression()?;
        self.consume_statement_terminator("; after print statement", beginner)?;
        Ok(Stmt::Print { value })
    }

    fn return_statement(&mut self, beginner: bool) -> Result<Stmt, ParseError> {
        let value = self.expression()?;
        self.consume_statement_terminator("; after return statement", beginner)?;
        Ok(Stmt::Return { value })
    }

    fn if_statement(&mut self) -> Result<Stmt, ParseError> {
        let condition = self.expression()?;
        self.matches(TokenKind::Then);
        self.consume_newlines();
        let then_branch = self.block()?;
        self.consume_newlines();
        let else_branch = if self.matches(TokenKind::Else) {
            self.consume_newlines();
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

    fn while_statement(&mut self) -> Result<Stmt, ParseError> {
        let condition = self.expression()?;
        self.consume_newlines();
        let body = self.block()?;
        Ok(Stmt::While { condition, body })
    }

    fn loop_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume_newlines();
        let body = self.block()?;
        Ok(Stmt::Loop { body })
    }

    fn block(&mut self) -> Result<Vec<Stmt>, ParseError> {
        self.consume(TokenKind::Start, "block start MULA or {")?;
        self.consume_layout_separators();
        let mut statements = Vec::new();
        while !self.is_at_end() && !self.check(TokenKind::End) {
            self.consume_layout_separators();
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
        if self.matches(TokenKind::TypeStr) {
            return Ok(Type::String);
        }
        if self.matches(TokenKind::Ptr) {
            self.consume(TokenKind::Less, "< after PTR")?;
            let inner = self.parse_type()?;
            self.consume(TokenKind::Greater, "> after pointer type")?;
            return Ok(Type::Pointer(Box::new(inner)));
        }
        // Ketuk 1: Core Memory Model — Slice syntax: []T
        if self.matches(TokenKind::LeftBracket) {
            self.consume(TokenKind::RightBracket, "']' after '[' in slice type")?;
            let element = self.parse_type()?;
            return Ok(Type::Slice { element: Box::new(element) });
        }
        // Ketuk 1: Core Memory Model — Buffer syntax: Buffer<T> or Buffer<T, N>
        if self.matches(TokenKind::Buffer) {
            self.consume(TokenKind::Less, "'<' after 'Buffer'")?;
            let element = self.parse_type()?;
            // Optional capacity: Buffer<f32, 1024>
            let _capacity = if self.matches(TokenKind::Comma) {
                let cap_tok = self.consume(TokenKind::Integer, "capacity integer after comma")?;
                cap_tok.lexeme.parse::<i64>().unwrap_or(0)
            } else {
                0 // unknown capacity — runtime enforcement only
            };
            self.consume(TokenKind::Greater, "'>' after Buffer type")?;
            return Ok(Type::Buffer { element: Box::new(element) });
        }
        // Ketuk 2: Result type syntax: Result<T, E>
        if self.matches(TokenKind::Result) {
            self.consume(TokenKind::Less, "'<' after 'Result'")?;
            let ok = self.parse_type()?;
            self.consume(TokenKind::Comma, "',' between Ok and Err types")?;
            let err = self.parse_type()?;
            self.consume(TokenKind::Greater, "'>' after Result type")?;
            return Ok(Type::Result { ok: Box::new(ok), err: Box::new(err) });
        }
        // Ketuk 3: File Handle ABI — Opaque types
        if self.matches(TokenKind::FileHandle) {
            return Ok(Type::Opaque { name: "FileHandle".to_string() });
        }
        // v1.30.1-alpha: Pintu<T, U, V> — SPSC channel with type-level capability
        if self.matches(TokenKind::Channel) {
            self.consume(TokenKind::Less, "'<' after 'Pintu'")?;
            let from = self.consume(TokenKind::Identifier, "sender Kotak name")?.lexeme.clone();
            self.consume(TokenKind::Comma, "',' after sender")?;
            let to = self.consume(TokenKind::Identifier, "receiver Kotak name")?.lexeme.clone();
            self.consume(TokenKind::Comma, "',' after receiver")?;
            let message_type = self.consume(TokenKind::Identifier, "message type")?.lexeme.clone();
            self.consume(TokenKind::Greater, "'>' after message type")?;
            return Ok(Type::Channel { from, to, message_type });
        }
        // User-defined named type (struct/enum) — any bare identifier.
        if self.check(TokenKind::Identifier) {
            let name = self.advance().lexeme.clone();
            return Ok(Type::Named(name));
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
        self.logical_or()
    }

    fn logical_or(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.logical_and()?;
        while self.matches(TokenKind::Or) {
            let right = self.logical_and()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op: BinaryOp::Or,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn logical_and(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.bit_or()?;
        while self.matches(TokenKind::And) {
            let right = self.bit_or()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op: BinaryOp::And,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn bit_or(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.bit_and()?;
        while self.matches(TokenKind::BitOr) {
            let right = self.bit_and()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op: BinaryOp::BitOr,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn bit_and(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.equality()?;
        while self.matches(TokenKind::BitAnd) {
            let right = self.equality()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op: BinaryOp::BitAnd,
                right: Box::new(right),
            };
        }
        Ok(expr)
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
        let mut expr = self.shift()?;
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
            let right = self.shift()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn shift(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.term()?;
        while self.matches(TokenKind::ShiftL) || self.matches(TokenKind::ShiftR) {
            let op = match self.previous().kind {
                TokenKind::ShiftL => BinaryOp::ShiftLeft,
                TokenKind::ShiftR => BinaryOp::ShiftRight,
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

    fn unary(&mut self) -> Result<Expr, ParseError> {
        if self.matches(TokenKind::Minus) {
            let operand = self.unary()?;
            Ok(Expr::Unary { op: "-".to_string(), operand: Box::new(operand) })
        } else if self.matches(TokenKind::Bang) {
            let operand = self.unary()?;
            Ok(Expr::Unary { op: "!".to_string(), operand: Box::new(operand) })
        } else {
            self.primary()
        }
    }

    fn factor(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.unary()?;
        while self.matches(TokenKind::Star) || self.matches(TokenKind::Slash) {
            let op = if self.previous().kind == TokenKind::Star {
                BinaryOp::Multiply
            } else {
                BinaryOp::Divide
            };
            let right = self.unary()?;
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
        // v1.30.1-alpha: Spawn — lahirkan KotakName()
        if self.matches(TokenKind::Spawn) {
            let actor_name = self.consume(TokenKind::Identifier, "Kotak name after 'lahirkan'")?.lexeme.clone();
            self.consume(TokenKind::LeftParen, "'(' after Kotak name")?;
            let mut args = Vec::new();
            if !self.check(TokenKind::RightParen) {
                loop {
                    args.push(self.expression()?);
                    if !self.matches(TokenKind::Comma) { break; }
                }
            }
            self.consume(TokenKind::RightParen, "')' after lahirkan args")?;
            return Ok(Expr::Spawn { actor_name, args });
        }
        // v1.30.1-alpha: Join — join ActorName
        if self.matches(TokenKind::Join) {
            let actor_name = self.consume(TokenKind::Identifier, "Actor name after 'join'")?.lexeme.clone();
            return Ok(Expr::Join { actor_name });
        }
        // v1.30.1-alpha Phase 3: Yield — yield control to scheduler
        if self.matches(TokenKind::Yield) {
            self.consume(TokenKind::LeftParen, "'(' after 'yield'")?;
            self.consume(TokenKind::RightParen, "')' after 'yield'")?;
            return Ok(Expr::Yield);
        }
        // v1.30.1-alpha Phase 3: Sleep — sleep(duration_ms)
        if self.matches(TokenKind::Sleep) {
            self.consume(TokenKind::LeftParen, "'(' after 'sleep'")?;
            let duration_ms = self.expression()?;
            self.consume(TokenKind::RightParen, "')' after sleep duration")?;
            return Ok(Expr::Sleep { duration_ms: Box::new(duration_ms) });
        }
        // Ketuk 2: Ok(value) and Err(value) result constructors
        if self.matches(TokenKind::Ok) {
            self.consume(TokenKind::LeftParen, "'(' after 'Ok'")?;
            let value = self.expression()?;
            self.consume(TokenKind::RightParen, "')' after Ok value")?;
            return Ok(Expr::Ok { value: Box::new(value) });
        }
        if self.matches(TokenKind::Err) {
            self.consume(TokenKind::LeftParen, "'(' after 'Err'")?;
            let value = self.expression()?;
            self.consume(TokenKind::RightParen, "')' after Err value")?;
            return Ok(Expr::Err { value: Box::new(value) });
        }
        if self.matches(TokenKind::Identifier) {
            let name = self.previous().lexeme.clone();
            // Enum variant path: `Enum::Variant` (two `:` tokens; no `::` token).
            if self.check(TokenKind::Colon)
                && self.tokens.get(self.current + 1).map_or(false, |t| t.kind == TokenKind::Colon)
            {
                self.advance(); // first ':'
                self.advance(); // second ':'
                let variant = self.consume(TokenKind::Identifier, "variant name after '::'")?.lexeme.clone();
                return Ok(Expr::EnumVariant { enum_name: name, variant });
            }
            // Check if followed by '(' → function or struct constructor call
            if self.check(TokenKind::LeftParen) {
                self.advance(); // consume '('
                let mut args = Vec::new();
                if !self.check(TokenKind::RightParen) {
                    loop {
                        args.push(self.expression()?);
                        if !self.matches(TokenKind::Comma) {
                            break;
                        }
                    }
                }
                self.consume(TokenKind::RightParen, "')' selepas argumen")?;
                return Ok(Expr::Call {
                    callee: Box::new(Expr::Variable(name)),
                    args,
                });
            }
            // Ketuk 1: Check if followed by '[' → buffer/slice indexing
            if self.check(TokenKind::LeftBracket) {
                self.advance(); // consume '['
                let index = self.expression()?;
                self.consume(TokenKind::RightBracket, "']' selepas indeks")?;
                return Ok(Expr::Index {
                    base: Box::new(Expr::Variable(name)),
                    index: Box::new(index),
                });
            }
            // Ketuk 3: Check if followed by '.' → method call on opaque type
            // h.read(1024), h.close(), h.seek(0)
            // v1.30.1-alpha: pintu.hantar(val), pintu.terima()
            if self.check(TokenKind::Dot) {
                self.advance(); // consume '.'
                let member = self.consume(TokenKind::Identifier, "field or method name after '.'")?.lexeme.clone();
                // Bare field access `obj.field` (not followed by '(').
                if !self.check(TokenKind::LeftParen) {
                    return Ok(Expr::FieldAccess {
                        base: Box::new(Expr::Variable(name.clone())),
                        field: member,
                    });
                }
                let method = member;
                self.consume(TokenKind::LeftParen, "'(' after method name")?;
                // v1.30.1-alpha: Channel method calls — send, recv
                if method == "send" {
                    let value = self.expression()?;
                    self.consume(TokenKind::RightParen, "')' after send value")?;
                    return Ok(Expr::Send { channel_name: name, value: Box::new(value) });
                }
                if method == "recv" {
                    self.consume(TokenKind::RightParen, "')' after recv'")?;
                    return Ok(Expr::Recv { channel_name: name });
                }
                // v1.30.1-alpha Phase 3: Backpressure — try_send, try_recv, timeout_recv
                if method == "try_send" {
                    let value = self.expression()?;
                    self.consume(TokenKind::RightParen, "')' after try_send value")?;
                    return Ok(Expr::TrySend { channel_name: name, value: Box::new(value) });
                }
                if method == "try_recv" {
                    self.consume(TokenKind::RightParen, "')' after try_recv'")?;
                    return Ok(Expr::TryRecv { channel_name: name });
                }
                if method == "timeout_recv" {
                    let timeout_ms = self.expression()?;
                    self.consume(TokenKind::RightParen, "')' after timeout_recv'")?;
                    return Ok(Expr::TimeoutRecv { channel_name: name, timeout_ms: Box::new(timeout_ms) });
                }
                let mut args = Vec::new();
                if !self.check(TokenKind::RightParen) {
                    loop {
                        args.push(self.expression()?);
                        if !self.matches(TokenKind::Comma) { break; }
                    }
                }
                self.consume(TokenKind::RightParen, "')' after method args")?;
                return Ok(Expr::MethodCall {
                    object: name,
                    method,
                    args,
                });
            }
            return Ok(Expr::Variable(name));
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

    fn consume_layout_separators(&mut self) {
        loop {
            let before = self.current;
            self.consume_optional_semicolons();
            while self.matches(TokenKind::Newline) {}
            if self.current == before {
                break;
            }
        }
    }

    fn consume_optional_semicolons(&mut self) {
        while self.matches(TokenKind::Semicolon) {}
    }

    fn consume_newlines(&mut self) {
        while self.matches(TokenKind::Newline) {}
    }

    fn consume_trailing_layout(&mut self) {
        self.consume_layout_separators();
    }

    fn consume_statement_terminator(
        &mut self,
        expected: &str,
        allow_newline: bool,
    ) -> Result<(), ParseError> {
        if self.matches(TokenKind::Semicolon) {
            self.consume_layout_separators();
            return Ok(());
        }
        if allow_newline
            && (self.matches(TokenKind::Newline)
                || self.check(TokenKind::End)
                || self.check(TokenKind::Else)
                || self.check(TokenKind::Eof))
        {
            self.consume_layout_separators();
            return Ok(());
        }
        self.consume(TokenKind::Semicolon, expected)?;
        self.consume_layout_separators();
        Ok(())
    }

    fn allows_beginner_line_terminator(&self, lexeme: &str) -> bool {
        self.critical_depth == 0 && matches!(lexeme, "BINA" | "CREATE" | "PAPAR" | "PULANG")
    }

    fn allows_control_flow_line_terminator(&self, lexeme: &str) -> bool {
        self.critical_depth == 0
            && (lexeme.eq_ignore_ascii_case("henti") || lexeme.eq_ignore_ascii_case("langkau"))
    }

    fn unimplemented_feature(&self) -> Result<Stmt, ParseError> {
        let t = self.peek();
        Err(ParseError::UnimplementedFeature {
            keyword: t.lexeme.clone(),
            line: t.line,
            column: t.column,
        })
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
        self.peek().kind == kind
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

#[cfg(test)]
mod tests {
    use super::Parser;
    use crate::lexer::{Lexer, Lexicon};
    use std::path::Path;

    fn parse_source(source: &str) -> Result<(), String> {
        let lexicon =
            Lexicon::from_path(Path::new("dict/core_map.json")).map_err(|e| e.to_string())?;
        let tokens = Lexer::new(source, &lexicon)
            .tokenize()
            .map_err(|e| e.to_string())?;
        Parser::new(tokens)
            .parse()
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    #[test]
    fn accepts_beginner_newline_separators_and_trailing_eof_layout() {
        let source = "MULA\nBINA x = 1\nPAPAR x\nTAMAT\n\n\t  \r\n";
        assert!(parse_source(source).is_ok(), "{:?}", parse_source(source));
    }

    #[test]
    fn accepts_if_then_block_start_on_following_line() {
        let source = "MULA\nBINA x = 1\nJIKA x > 0 MAKA\nMULA\nPAPAR x\nTAMAT\nMELAINKAN\nMULA\nPAPAR 0\nTAMAT\nTAMAT\n";
        assert!(parse_source(source).is_ok(), "{:?}", parse_source(source));
    }

    #[test]
    fn keeps_expert_let_semicolon_mandatory() {
        let source = "{\nlet x = 1\n}\n";
        assert!(parse_source(source).is_err());
    }

    #[test]
    fn keeps_critical_hardware_zone_statements_semicolon_mandatory() {
        let source = "MULA\nZON_PERKAKASAN MULA\nBINA port: I32 = addr 65280\nTAMAT\nTAMAT\n";
        assert!(parse_source(source).is_err());

        let strict_source =
            "MULA\nZON_PERKAKASAN MULA\nBINA port: I32 = addr 65280;\nPAPAR port;\nTAMAT\nTAMAT\n";
        assert!(
            parse_source(strict_source).is_ok(),
            "{:?}",
            parse_source(strict_source)
        );
    }

    #[test]
    fn accepts_duplicate_blank_lines_crlf_and_extra_semicolons() {
        let source = "MULA\r\n\r\n;;\r\nBINA x = 1\r\n\r\nPAPAR x\r\n;;\r\nTAMAT\r\n\r\n";
        assert!(parse_source(source).is_ok(), "{:?}", parse_source(source));
    }

    #[test]
    fn accepts_else_block_start_after_blank_line() {
        let source = "MULA\nBINA x = 0\nJIKA x > 1 MAKA\nMULA\nPAPAR x\nTAMAT\nMELAINKAN\n\nMULA\nPAPAR 0\nTAMAT\nTAMAT\n";
        assert!(parse_source(source).is_ok(), "{:?}", parse_source(source));
    }

    #[test]
    fn accepts_split_control_flow_and_logic_group_operators() {
        let source = "MULA\nBINA x = 0\nSELAGI x < 3 DAN benar MULA\nPAPAR x\nHENTI\nTAMAT\nULANG MULA\nLANGKAU\nTAMAT\nBINA y = (1 << 2) | 1 & 3\nTAMAT\n";
        assert!(parse_source(source).is_ok(), "{:?}", parse_source(source));
    }

    #[test]
    fn traps_complex_tokens_as_unimplemented_in_split_scope() {
        for source in [
            "MULA\nbentuk Thing MULA\nTAMAT\nTAMAT\n",
            "MULA\npilihan Mode MULA\nTAMAT\nTAMAT\n",
            "MULA\nberisiko MULA\nTAMAT\nTAMAT\n",
            "MULA\nluar fn c_call() -> I32 MULA\nTAMAT\nTAMAT\n",
        ] {
            let err = parse_source(source).expect_err("complex token must trap as unimplemented");
            assert!(
                err.contains("belum disokong") || err.contains("unimplemented"),
                "unexpected error: {err}"
            );
        }
    }
}
