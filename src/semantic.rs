// =========================================================================
// Project: Logicodex Language Engine (Phase 2 Deployment Integration)
// Version: v1.21-alpha (Specification Baseline & Practical Severity Roadmap)
// Architect & Creator: Mohamad Supardi Abdul (mymsastudio@gmail.com)
// Copyright (c) 2026. All Rights Reserved.
// Licensed under permissive dual-license: MIT & Apache License 2.0
// =========================================================================
use crate::ast::{BinaryOp, Expr, Program, Stmt, Type};
use std::collections::{HashMap, HashSet};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeverityPolicy {
    Desktop,
    Embedded,
    Kernel,
}

impl SeverityPolicy {
    pub fn from_target(target: &str, secure: bool) -> Self {
        let lower = target.to_ascii_lowercase();
        if lower.contains("kernel") || secure {
            Self::Kernel
        } else if lower.contains("embedded")
            || lower.contains("mcu")
            || lower.contains("bare")
            || lower.contains("freestanding")
        {
            Self::Embedded
        } else {
            Self::Desktop
        }
    }
}

#[derive(Debug, Error)]
pub enum SemanticError {
    #[error("variable `{0}` is already defined in this scope")]
    DuplicateVariable(String),
    #[error("variable `{0}` is not defined")]
    UndefinedVariable(String),
    #[error("function `{0}` is already defined")]
    DuplicateFunction(String),
    #[error("hardware region `{0}` is already defined")]
    DuplicateHardwareRegion(String),
    #[error("operator `{op}` requires {expected} operands but received {left} and {right}")]
    TypeMismatch {
        op: BinaryOp,
        expected: &'static str,
        left: Type,
        right: Type,
    },
    #[error("binding `{name}` was declared as {declared} but expression has type {actual}")]
    DeclaredTypeMismatch {
        name: String,
        declared: Type,
        actual: Type,
    },
    #[error("if condition must be Bool but received {0}")]
    NonBooleanCondition(Type),
    #[error("division by a constant zero is rejected by static analysis")]
    DivisionByZero,
    #[error("numeric literal {value} does not fit in declared type {ty}")]
    NumericBounds { value: i64, ty: Type },
    #[error("literal address {0} has no declared hardware provenance region")]
    MissingProvenance(i64),
    #[error("pointer value for `{name}` must originate from an explicit addr literal or hardware region")]
    InvalidPointerInitializer { name: String },
    #[error("bare address literal is rejected under {policy:?} target policy; declare a hardware region first")]
    BareAddressRejected { policy: SeverityPolicy },
    #[error("return statement is outside a function")]
    ReturnOutsideFunction,
    #[error("function `{name}` returns {expected} but returned expression has type {actual}")]
    ReturnTypeMismatch {
        name: String,
        expected: Type,
        actual: Type,
    },
}

#[derive(Debug, Default)]
pub struct Analyzer {
    scopes: Vec<HashMap<String, Type>>,
    hardware_regions: HashMap<String, (Type, i64)>,
    hardware_addresses: HashSet<i64>,
    functions: HashMap<String, (Vec<Type>, Option<Type>)>,
    current_function: Option<(String, Option<Type>)>,
    policy: SeverityPolicy,
}

impl Default for SeverityPolicy {
    fn default() -> Self {
        Self::Desktop
    }
}

impl Analyzer {
    pub fn analyze(program: &Program) -> Result<(), SemanticError> {
        Self::analyze_with_policy(program, SeverityPolicy::Desktop)
    }

    pub fn analyze_for_target(
        program: &Program,
        target: &str,
        secure: bool,
    ) -> Result<(), SemanticError> {
        Self::analyze_with_policy(program, SeverityPolicy::from_target(target, secure))
    }

    pub fn analyze_with_policy(
        program: &Program,
        policy: SeverityPolicy,
    ) -> Result<(), SemanticError> {
        let mut analyzer = Self {
            scopes: vec![HashMap::new()],
            policy,
            ..Self::default()
        };
        analyzer.block(&program.statements)
    }

    fn block(&mut self, statements: &[Stmt]) -> Result<(), SemanticError> {
        for stmt in statements {
            self.statement(stmt)?;
        }
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
            Stmt::Use { .. } => Ok(()),
            Stmt::HardwareDecl { name, ty, address } => {
                if self.hardware_regions.contains_key(name) {
                    return Err(SemanticError::DuplicateHardwareRegion(name.clone()));
                }
                let Expr::AddressOfLiteral(addr) = address else {
                    return Err(SemanticError::InvalidPointerInitializer { name: name.clone() });
                };
                self.hardware_regions
                    .insert(name.clone(), (ty.clone(), *addr));
                self.hardware_addresses.insert(*addr);
                self.define(name, Type::Pointer(Box::new(ty.clone())))
            }
            Stmt::Function {
                name,
                params,
                return_type,
                body,
            } => {
                if self.functions.contains_key(name) {
                    return Err(SemanticError::DuplicateFunction(name.clone()));
                }
                self.functions.insert(
                    name.clone(),
                    (
                        params.iter().map(|p| p.ty.clone()).collect(),
                        return_type.clone(),
                    ),
                );
                self.scopes.push(HashMap::new());
                for param in params {
                    self.define(&param.name, param.ty.clone())?;
                }
                let previous = self
                    .current_function
                    .replace((name.clone(), return_type.clone()));
                let result = self.block(body);
                self.current_function = previous;
                self.scopes.pop();
                result
            }
            Stmt::Let {
                name,
                declared_type,
                value,
            } => {
                let inferred = self.expression(value)?;
                let ty = declared_type.clone().unwrap_or(inferred.clone());
                if let Some(declared) = declared_type {
                    self.check_assignment(name, declared, &inferred, value)?;
                }
                self.define(name, ty)
            }
            Stmt::Print { value } | Stmt::ExprStmt { value } => {
                self.expression(value)?;
                Ok(())
            }
            Stmt::Return { value } => {
                let actual = self.expression(value)?;
                let Some((function_name, expected)) = &self.current_function else {
                    return Err(SemanticError::ReturnOutsideFunction);
                };
                if let Some(expected) = expected {
                    if !types_compatible(expected, &actual) {
                        return Err(SemanticError::ReturnTypeMismatch {
                            name: function_name.clone(),
                            expected: expected.clone(),
                            actual,
                        });
                    }
                }
                Ok(())
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let ty = self.expression(condition)?;
                if ty != Type::Bool {
                    return Err(SemanticError::NonBooleanCondition(ty));
                }
                self.scoped_block(then_branch)?;
                self.scoped_block(else_branch)
            }
        }
    }

    fn check_assignment(
        &self,
        name: &str,
        declared: &Type,
        actual: &Type,
        value: &Expr,
    ) -> Result<(), SemanticError> {
        if declared.is_pointer() {
            match value {
                Expr::AddressOfLiteral(addr)
                    if self.hardware_addresses.contains(addr)
                        || self.policy == SeverityPolicy::Desktop =>
                {
                    Ok(())
                }
                Expr::AddressOfLiteral(_) => Err(SemanticError::BareAddressRejected {
                    policy: self.policy,
                }),
                Expr::Variable(source) if self.hardware_regions.contains_key(source) => Ok(()),
                _ => Err(SemanticError::InvalidPointerInitializer {
                    name: name.to_string(),
                }),
            }?;
        }
        if let Expr::Integer(value) = value {
            if !integer_fits(*value, declared) {
                return Err(SemanticError::NumericBounds {
                    value: *value,
                    ty: declared.clone(),
                });
            }
        }
        if !types_compatible(declared, actual) {
            return Err(SemanticError::DeclaredTypeMismatch {
                name: name.to_string(),
                declared: declared.clone(),
                actual: actual.clone(),
            });
        }
        Ok(())
    }

    fn expression(&self, expr: &Expr) -> Result<Type, SemanticError> {
        match expr {
            Expr::Integer(_) => Ok(Type::I64),
            Expr::Boolean(_) => Ok(Type::Bool),
            Expr::StringLiteral(_) => Ok(Type::String),
            Expr::Variable(name) => self.resolve(name),
            Expr::AddressOfLiteral(addr) => {
                if self.policy != SeverityPolicy::Desktop && !self.hardware_addresses.contains(addr)
                {
                    return Err(SemanticError::MissingProvenance(*addr));
                }
                Ok(Type::Pointer(Box::new(Type::U16)))
            }
            Expr::Grouped(inner) => self.expression(inner),
            Expr::Binary { left, op, right } => {
                if *op == BinaryOp::Divide && matches!(right.as_ref(), Expr::Integer(0)) {
                    return Err(SemanticError::DivisionByZero);
                }
                let left_ty = self.expression(left)?;
                let right_ty = self.expression(right)?;
                match op {
                    BinaryOp::Add | BinaryOp::Subtract | BinaryOp::Multiply | BinaryOp::Divide => {
                        if is_numeric(&left_ty) && is_numeric(&right_ty) {
                            Ok(promote_numeric(left_ty, right_ty))
                        } else {
                            Err(SemanticError::TypeMismatch {
                                op: *op,
                                expected: "numeric",
                                left: left_ty,
                                right: right_ty,
                            })
                        }
                    }
                    BinaryOp::Greater
                    | BinaryOp::GreaterEqual
                    | BinaryOp::Less
                    | BinaryOp::LessEqual => {
                        if is_numeric(&left_ty) && is_numeric(&right_ty) {
                            Ok(Type::Bool)
                        } else {
                            Err(SemanticError::TypeMismatch {
                                op: *op,
                                expected: "numeric",
                                left: left_ty,
                                right: right_ty,
                            })
                        }
                    }
                    BinaryOp::Equal | BinaryOp::NotEqual => {
                        if types_compatible(&left_ty, &right_ty) {
                            Ok(Type::Bool)
                        } else {
                            Err(SemanticError::TypeMismatch {
                                op: *op,
                                expected: "matching",
                                left: left_ty,
                                right: right_ty,
                            })
                        }
                    }
                }
            }
        }
    }

    fn define(&mut self, name: &str, ty: Type) -> Result<(), SemanticError> {
        let scope = self
            .scopes
            .last_mut()
            .expect("semantic analyzer must always have a scope");
        if scope.contains_key(name) {
            return Err(SemanticError::DuplicateVariable(name.to_string()));
        }
        scope.insert(name.to_string(), ty);
        Ok(())
    }

    fn resolve(&self, name: &str) -> Result<Type, SemanticError> {
        for scope in self.scopes.iter().rev() {
            if let Some(ty) = scope.get(name) {
                return Ok(ty.clone());
            }
        }
        Err(SemanticError::UndefinedVariable(name.to_string()))
    }
}

fn is_numeric(ty: &Type) -> bool {
    matches!(
        ty,
        Type::I32 | Type::I64 | Type::U16 | Type::U32 | Type::F64
    )
}

fn promote_numeric(left: Type, right: Type) -> Type {
    if left == Type::F64 || right == Type::F64 {
        Type::F64
    } else if left == Type::I64 || right == Type::I64 {
        Type::I64
    } else if left == Type::U32 || right == Type::U32 {
        Type::U32
    } else if left == Type::I32 || right == Type::I32 {
        Type::I32
    } else {
        Type::U16
    }
}

fn types_compatible(expected: &Type, actual: &Type) -> bool {
    expected == actual
        || (is_numeric(expected) && is_numeric(actual))
        || (expected.is_pointer() && actual.is_pointer())
}

fn integer_fits(value: i64, ty: &Type) -> bool {
    match ty {
        Type::I32 => value >= i32::MIN as i64 && value <= i32::MAX as i64,
        Type::I64 => true,
        Type::U16 => value >= 0 && value <= u16::MAX as i64,
        Type::U32 => value >= 0 && value <= u32::MAX as i64,
        Type::F64 => true,
        Type::Bool => value == 0 || value == 1,
        Type::Pointer(_) | Type::String => false,
    }
}
