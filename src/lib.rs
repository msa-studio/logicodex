// =========================================================================
// Logicodex Language Engine — Library Target
// Sprint 1: Type System Foundation
//
// This library target exists primarily for integration tests in tests/.
// The binary target (src/main.rs) is the primary entry point for users.
//
// All modules are re-exported for test access.
// =========================================================================

pub mod ast;
pub mod codegen;
pub mod codegen_contract;
pub mod ffi;
pub mod hir;
pub mod layout;
pub mod lexer;
pub mod os;
pub mod parser;
pub mod semantic;
pub mod semantic_gate;
pub mod span;
pub mod net;
pub mod tier2;
pub mod types;
