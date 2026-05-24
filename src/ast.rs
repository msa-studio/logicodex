// =========================================================================
// Project: Logicodex Language Engine (Phase 2 Deployment Integration)
// Version: v1.21-alpha (Specification Baseline & Practical Severity Roadmap)
// Architect & Creator: Mohamad Supardi Abdul (mymsastudio@gmail.com)
// Copyright (c) 2026. All Rights Reserved.
// Licensed under permissive dual-license: MIT & Apache License 2.0
// =========================================================================
use std::fmt;

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
    Use {
        module: String,
    },
    HardwareDecl {
        name: String,
        ty: Type,
        address: Expr,
    },
    HardwareZone {
        body: Vec<Stmt>,
    },
    Function {
        name: String,
        params: Vec<Param>,
        return_type: Option<Type>,
        body: Vec<Stmt>,
    },
    Let {
        name: String,
        declared_type: Option<Type>,
        value: Expr,
    },
    Print {
        value: Expr,
    },
    Return {
        value: Expr,
    },
    ExprStmt {
        value: Expr,
    },
    If {
        condition: Expr,
        then_branch: Vec<Stmt>,
        else_branch: Vec<Stmt>,
    },
    While {
        condition: Expr,
        body: Vec<Stmt>,
    },
    Loop {
        body: Vec<Stmt>,
    },
    Break,
    Continue,
    StructDecl {
        name: String,
        fields: Vec<Param>,
    },
    EnumDecl {
        name: String,
        variants: Vec<String>,
    },
    UnsafeBlock {
        body: Vec<Stmt>,
    },
    ExternBlock {
        abi: String,
        functions: Vec<ExternFnDecl>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternFnDecl {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Param {
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Integer(i64),
    Boolean(bool),
    StringLiteral(String),
    Variable(String),
    AddressOfLiteral(i64),
    /// Function or struct constructor call: `Name(arg1, arg2, ...)`
    /// Used for both regular function calls (e.g., `InitWindow(800, 600, "Hi")`)
    /// and struct constructors (e.g., `Color(255, 0, 0, 255)`).
    Call {
        callee: Box<Expr>, // usually Expr::Variable(name)
        args: Vec<Expr>,
    },
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },
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
    And,
    Or,
    BitAnd,
    BitOr,
    ShiftLeft,
    ShiftRight,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    I32,
    I64,
    U16,
    U32,
    F64,
    Bool,
    Pointer(Box<Type>),
    String,
    /// Audio Engine (v1.30): Function pointer type for hardware-safe callbacks.
    /// Syntax: `fn(*mut f32, i32)` or `fn(i32, i32) -> f64`
    FunctionPointer {
        params: Vec<Type>,
        return_type: Option<Box<Type>>,
    },
}

impl Type {
    pub fn is_pointer(&self) -> bool {
        matches!(self, Type::Pointer(_))
    }

    /// Audio Engine (v1.30): Check if this type is a function pointer (audio callback).
    pub fn is_function_pointer(&self) -> bool {
        matches!(self, Type::FunctionPointer { .. })
    }

    /// Audio Engine (v1.30): Check if this function pointer is suitable for audio ISR context.
    /// Returns true if params suggest audio callback signature (buffer pointer + frame count).
    pub fn is_audio_callback_fp(&self) -> bool {
        match self {
            Type::FunctionPointer { params, .. } => {
                params.len() == 2
                    && params[0].is_pointer()
                    && matches!(params[1], Type::I32)
            }
            _ => false,
        }
    }

    #[allow(dead_code)]
    pub fn storage_width_bits(&self) -> u32 {
        match self {
            Type::I32 | Type::U32 => 32,
            Type::U16 => 16,
            Type::I64 | Type::F64 | Type::Pointer(_) | Type::String => 64,
            Type::Bool => 1,
            // Function pointers are pointer-sized (64-bit on native targets)
            Type::FunctionPointer { .. } => 64,
        }
    }
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
            BinaryOp::And => "&&",
            BinaryOp::Or => "||",
            BinaryOp::BitAnd => "&",
            BinaryOp::BitOr => "|",
            BinaryOp::ShiftLeft => "<<",
            BinaryOp::ShiftRight => ">>",
        };
        write!(f, "{text}")
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::I32 => write!(f, "I32"),
            Type::I64 => write!(f, "I64"),
            Type::U16 => write!(f, "U16"),
            Type::U32 => write!(f, "U32"),
            Type::F64 => write!(f, "F64"),
            Type::Bool => write!(f, "Bool"),
            Type::Pointer(inner) => write!(f, "PTR<{inner}>"),
            Type::String => write!(f, "String"),
            Type::FunctionPointer { params, return_type } => {
                write!(f, "fn(")?;
                for (i, p) in params.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{p}")?;
                }
                write!(f, ")")?;
                if let Some(ret) = return_type {
                    write!(f, " -> {ret}")?;
                }
                Ok(())
            }
        }
    }
}
