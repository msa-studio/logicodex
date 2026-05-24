#![allow(dead_code)]
#![allow(unused_variables)]

// =========================================================================
// Logicodex v1.30 architecture simulation: source spans and diagnostics.
//
// This module is dormant in the v1.21-alpha runtime. It exists only to test
// whether the v1.30 architecture skeleton can compile without changing the
// current executable compiler path.
// =========================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FileId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    pub file_id: FileId,
    pub start_line: u32,
    pub start_col: u32,
    pub end_line: u32,
    pub end_col: u32,
}

impl Span {
    pub const fn new(
        file_id: FileId,
        start_line: u32,
        start_col: u32,
        end_line: u32,
        end_col: u32,
    ) -> Self {
        Self {
            file_id,
            start_line,
            start_col,
            end_line,
            end_col,
        }
    }

    pub const fn unknown() -> Self {
        Self {
            file_id: FileId(0),
            start_line: 0,
            start_col: 0,
            end_line: 0,
            end_col: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spanned<T> {
    pub node: T,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub code: DiagnosticCode,
    pub severity: Severity,
    pub message_ms: String,
    pub message_en: String,
    pub primary_span: Span,
    pub notes: Vec<DiagnosticNote>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiagnosticCode {
    ParserUnsupportedFeature,
    TypeMismatch,
    UnsafeBoundaryViolation,
    FfiBoundaryViolation,
    LayoutError,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticNote {
    pub span: Option<Span>,
    pub message_ms: String,
    pub message_en: String,
}
