// =========================================================================
// Logicodex v1.31.0-alpha — Tier 2: Streaming Semantic Compiler
//
// The "brain" that replaces keeping full AST in RAM.
// 
// Architecture:
//   Tier 1 (Parser)    → Full AST per function (temporary, discarded after Pass 2)
//   Tier 2 (This mod)  → MetadataGraph with SemanticSummary (persistent, lightweight)
//   Tier 3 (Codegen)   → LLVM IR chunks (streamed, one function at a time)
//
// Flow:
//   Program → Pass 1 (pre-declare) → MetadataGraph → Pass 2 (stream) → Results
//                ↑ lightweight index       ↑ deep analysis per function
//                                           ↑ discard AST after each function
// =========================================================================

pub mod metadata;
pub mod pass;

// Re-exports for convenience
pub use metadata::{
    Capability, InlineCost, MemoryReport, MetadataGraph, SemanticSummary,
};
pub use pass::{compile_streaming, pass1_predeclare, pass2_streaming, CompileMode, StreamingResult};
