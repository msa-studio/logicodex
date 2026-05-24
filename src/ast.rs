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
    /// Ketuk 2: Pattern matching on Result<T, E>.
    /// Syntax: `match expr { Ok(v) => body, Err(e) => body }`
    Match {
        value: Expr,
        arms: Vec<MatchArm>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchArm {
    pub pattern: MatchPattern,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MatchPattern {
    /// Matches Ok variant: Ok(name)
    Ok { binding: String },
    /// Matches Err variant: Err(name)
    Err { binding: String },
    /// Wildcard: _
    Wildcard,
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
    /// Ketuk 1: Buffer/Slice indexing — buf[index]
    Index {
        base: Box<Expr>,
        index: Box<Expr>,
    },
    /// Ketuk 2: Result construction — Ok(value)
    Ok { value: Box<Expr> },
    /// Ketuk 2: Result construction — Err(error)
    Err { value: Box<Expr> },
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
    // ─── Ketuk 1: Core Memory Model ───
    Slice { element: Box<Type> },
    Buffer { element: Box<Type> },
    /// Ketuk 2: Result type for IO operations — Ok(T) or Err(E).
    /// Syntax: Result<T, E>
    Result { ok: Box<Type>, err: Box<Type> },
}

impl Type {
    pub fn is_pointer(&self) -> bool {
        matches!(self, Type::Pointer(_))
    }

    /// Ketuk 1: Check if this type is a slice ([]T).
    pub fn is_slice(&self) -> bool {
        matches!(self, Type::Slice { .. })
    }

    /// Ketuk 1: Check if this type is a buffer (Buffer<T>).
    pub fn is_buffer(&self) -> bool {
        matches!(self, Type::Buffer { .. })
    }

    /// Ketuk 1: Check if this is a contiguous memory type (slice or buffer).
    pub fn is_contiguous(&self) -> bool {
        matches!(self, Type::Slice { .. } | Type::Buffer { .. })
    }

    /// Ketuk 1: Get the element type if this is a slice or buffer.
    pub fn element_type(&self) -> Option<&Type> {
        match self {
            Type::Slice { element } | Type::Buffer { element } => Some(element),
            _ => None,
        }
    }

    /// Ketuk 2: Check if this is a Result type.
    pub fn is_result(&self) -> bool {
        matches!(self, Type::Result { .. })
    }

    /// Ketuk 2: Get the Ok type from a Result.
    pub fn ok_type(&self) -> Option<&Type> {
        match self {
            Type::Result { ok, .. } => Some(ok),
            _ => None,
        }
    }

    /// Ketuk 2: Get the Err type from a Result.
    pub fn err_type(&self) -> Option<&Type> {
        match self {
            Type::Result { err, .. } => Some(err),
            _ => None,
        }
    }

    #[allow(dead_code)]
    pub fn storage_width_bits(&self) -> u32 {
        match self {
            Type::I32 | Type::U32 => 32,
            Type::U16 => 16,
            Type::I64 | Type::F64 | Type::Pointer(_) | Type::String => 64,
            Type::Bool => 1,
            // Slice and Buffer are pointer-sized (fat pointer: ptr + len)
            Type::Slice { .. } | Type::Buffer { .. } => 128,
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
            Type::Slice { element } => write!(f, "[]{element}"),
            Type::Buffer { element } => write!(f, "Buffer<{element}>"),
            Type::Result { ok, err } => write!(f, "Result<{ok}, {err}>"),
        }
    }
}
