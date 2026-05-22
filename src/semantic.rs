// =========================================================================
// Project: Logicodex Language Engine (Phase 2 Deployment Integration)
// Version: v1.21-alpha (Formal Specifications & Zero-Overhead Severity Model)
// Architect & Creator: Mohamad Supardi Abdul (mymsastudio@gmail.com)
// Copyright (c) 2026. All Rights Reserved.
// Licensed under permissive dual-license: MIT & Apache License 2.0
// =========================================================================
use crate::ast::{BinaryOp, Expr, Program, Stmt, Type};
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
