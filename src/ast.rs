// =========================================================================
// Project: Logicodex Language Engine
// Pipeline: single HIR compilation engine (.ldx -> AST -> HIR -> LLVM)
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
        /// `true` if declared `public` (exported across module boundaries).
        /// Private by default: an item with no `public` is module-local.
        is_public: bool,
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
    /// For loop: `for variable in iterable { body }`
    For {
        variable: String,
        iterable: Expr,
        body: Vec<Stmt>,
    },
    Break,
    Continue,
    /// Block statement: `{ statements }`
    Block(Vec<Stmt>),
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
    /// Actor declaration — concurrency unit (1 OS Thread).
    /// Syntax: `actor SensorSuhu { let pintu: Pintu<...> = ...; ... }`
    Actor {
        name: String,
        /// Captured parameters (B.1b): explicit handles the actor receives, e.g.
        /// a channel. Empty for a niladic actor (uses the old spawn ABI).
        params: Vec<Param>,
        body: Vec<Stmt>,
    },
    /// Variable assignment: target = value (target can be any Expr)
    Assign {
        target: Expr,
        value: Expr,
    },
    /// Buffer index assignment: buf[index] = value
    IndexAssign {
        target: String,
        index: Expr,
        value: Expr,
    },
    /// Service manifest — deterministic network reactor.
    /// Syntax: `service WebServer { port: 443, requires: Net.Admin, handler: WebHandler, policy: Block }`
    Service {
        name: String,
        port: u16,
        requires: Option<String>,
        handler: String,
        policy: String,
    },
}

/// Match arm: pattern => body
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchArm {
    pub pattern: MatchPattern,
    pub body: Vec<Stmt>,
}

/// Match pattern
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MatchPattern {
    /// Matches Ok variant: Ok(name)
    Ok { binding: String },
    /// Matches Err variant: Err(name)
    Err { binding: String },
    /// Matches Some variant: Some(name)
    Some { binding: String },
    /// Matches None variant: None
    None,
    /// Wildcard: _
    Wildcard,
    /// Literal pattern: 5, "hello"
    Literal(Expr),
    /// Identifier pattern: x
    Identifier(String),
    /// Tuple pattern: (a, b, c)
    Tuple(Vec<MatchPattern>),
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
    /// Unary operation: op expr
    Unary {
        op: String,
        operand: Box<Expr>,
    },
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
    /// Module-qualified call: `module.function(args)`.
    /// A dedicated node, not a field access followed by a call: the call carries
    /// which module it targets, so resolution never has to infer it from
    /// structure. This is the truth-travels-with-the-node principle the module
    /// system depends on to avoid name-resolution aliasing.
    QualifiedCall {
        module: String,
        function: String,
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
    /// Collections Foundation Stage 0: fixed array literal — [a, b, c]
    ArrayLiteral {
        elements: Vec<Expr>,
    },
    /// Ketuk 2: Result construction — Ok(value)
    Ok {
        value: Box<Expr>,
    },
    /// Ketuk 2: Result construction — Err(error)
    Err {
        value: Box<Expr>,
    },
    /// Foundation Option construction — Some(value)
    Some {
        value: Box<Expr>,
    },
    /// Foundation Option construction — None
    None,
    /// Ketuk 3: File Handle ABI — Method call on opaque type.
    /// Syntax: h.read(1024), h.close(), h.seek(0, Start)
    MethodCall {
        object: String,
        method: String,
        args: Vec<Expr>,
    },
    /// Field access: net.admin
    FieldAccess {
        base: Box<Expr>,
        field: String,
    },
    /// Enum variant reference: `Warna::Merah`.
    EnumVariant {
        enum_name: String,
        variant: String,
    },
    /// Spawn a Kotak (create OS thread).
    /// Syntax: `lahirkan SensorSuhu()`
    /// Create a channel: `Channel::baru(capacity)`. Capacity is an expression
    /// (typically an integer literal). Lowered to logicodex_channel_create at
    /// codegen; the handle is stored in the bound variable's slot (ABI-1,
    /// by-handle — codegen owns the name->handle mapping, runtime stays nameless).
    ChannelCreate {
        capacity: Box<Expr>,
    },
    Spawn {
        actor_name: String,
        args: Vec<Expr>,
    },
    /// Send value through Channel.
    /// Syntax: `channel_data.send(Ok(DataSuhu{ nilai: 25.5 }))`
    Send {
        channel_name: String,
        value: Box<Expr>,
    },
    /// Receive value from Channel.
    /// Syntax: `channel_data.recv()`
    Recv {
        channel_name: String,
    },
    /// Wait for Actor to finish.
    /// Syntax: `join SensorSuhu`
    Join {
        actor_name: String,
    },
    // Backpressure + Scheduler
    /// TrySend — non-blocking send, returns Result<bool, IoError>.
    /// Syntax: `channel.try_send(value)`
    TrySend {
        channel_name: String,
        value: Box<Expr>,
    },
    /// TryRecv — non-blocking recv, returns Option<T>.
    /// Syntax: `channel.try_recv()`
    TryRecv {
        channel_name: String,
    },
    /// Yield — yield control to the scheduler.
    /// Syntax: `yield()`
    Yield,
    /// Sleep — sleep for N milliseconds.
    /// Syntax: `sleep(1000)`
    Sleep {
        duration_ms: Box<Expr>,
    },
    /// TimeoutRecv — recv with timeout, returns Result<T, TimeoutError>.
    /// Syntax: `channel.timeout_recv(5000)`
    TimeoutRecv {
        channel_name: String,
        timeout_ms: Box<Expr>,
    },
    Grouped(Box<Expr>),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
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
    BitXor,
    ShiftLeft,
    ShiftRight,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    I32,
    I64,
    U16,
    U32,
    I8,
    I16,
    U8,
    U64,
    F64,
    Bool,
    Pointer(Box<Type>),
    String,
    /// User-defined named type (struct/enum), resolved during lowering.
    Named(String),
    // ─── Ketuk 1: Core Memory Model ───
    Slice {
        element: Box<Type>,
    },
    Array {
        element: Box<Type>,
        len: usize,
    },
    Buffer {
        element: Box<Type>,
    },
    /// Ketuk 2: Result type for IO operations — Ok(T) or Err(E).
    /// Syntax: Result<T, E>
    Result {
        ok: Box<Type>,
        err: Box<Type>,
    },
    /// Foundation Option type — Some(T) or None.
    /// Syntax: Option<T>
    Option {
        some: Box<Type>,
    },
    /// Ketuk 3: File Handle ABI — Opaque type (internal structure hidden).
    /// Syntax: FileHandle, FileMode
    Opaque {
        name: String,
    },
    // ─── Threading Foundation — Kotak & Pintu ───
    /// Channel<T, U> — SPSC channel with type-level capability.
    /// Syntax: Channel<SensorSuhu, KotakEnjin, DataSuhu>
    Channel {
        from: String,
        to: String,
        message_type: String,
    },
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
        matches!(
            self,
            Type::Slice { .. } | Type::Array { .. } | Type::Buffer { .. }
        )
    }

    /// Ketuk 1: Get the element type if this is a slice or buffer.
    pub fn element_type(&self) -> Option<&Type> {
        match self {
            Type::Slice { element } | Type::Array { element, .. } | Type::Buffer { element } => {
                Some(element)
            }
            _ => None,
        }
    }

    /// Ketuk 2: Check if this is a Result type.
    pub fn is_result(&self) -> bool {
        matches!(self, Type::Result { .. })
    }

    /// Check if this is an Option type.
    pub fn is_option(&self) -> bool {
        matches!(self, Type::Option { .. })
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

    /// Ketuk 3: Check if this is an opaque type.
    pub fn is_opaque(&self) -> bool {
        matches!(self, Type::Opaque { .. })
    }

    /// Ketuk 3: Get the name of an opaque type.
    pub fn opaque_name(&self) -> Option<&str> {
        match self {
            Type::Opaque { name } => Some(name),
            _ => None,
        }
    }

    /// Check if this is a Pintu type.
    pub fn is_channel(&self) -> bool {
        matches!(self, Type::Channel { .. })
    }

    /// Get Pintu capability (from, to, message_type).
    pub fn channel_capability(&self) -> Option<(&str, &str, &str)> {
        match self {
            Type::Channel {
                from,
                to,
                message_type,
            } => Some((from, to, message_type)),
            _ => None,
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
            BinaryOp::Modulo => "%",
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
            BinaryOp::BitXor => "^",
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
            Type::I8 => write!(f, "I8"),
            Type::I16 => write!(f, "I16"),
            Type::U8 => write!(f, "U8"),
            Type::U64 => write!(f, "U64"),
            Type::F64 => write!(f, "F64"),
            Type::Bool => write!(f, "Bool"),
            Type::Pointer(inner) => write!(f, "PTR<{inner}>"),
            Type::String => write!(f, "String"),
            Type::Slice { element } => write!(f, "[]{element}"),
            Type::Array { element, len } => write!(f, "[{element}; {len}]"),
            Type::Buffer { element } => write!(f, "Buffer<{element}>"),
            Type::Result { ok, err } => write!(f, "Result<{ok}, {err}>"),
            Type::Option { some } => write!(f, "Option<{some}>"),
            Type::Opaque { name } => write!(f, "{name}"),
            Type::Named(name) => write!(f, "{name}"),
            Type::Channel {
                from,
                to,
                message_type,
            } => {
                write!(f, "Channel<{from}, {to}, {message_type}>")
            }
        }
    }
}
