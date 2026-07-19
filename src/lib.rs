// =========================================================================
// Logicodex Language Engine — Library Target
// Freestanding Compiler Support
//
// This library target exists primarily for integration tests in tests/.
// The binary target (src/main.rs) is the primary entry point for users.
//
// For freestanding targets, this crate is no_std with alloc.
// For hosted targets, it uses the full standard library.
// =========================================================================

// no_std support for freestanding targets
// When building for bare metal, we use only core + alloc (no std::fs, std::process, etc.)
// The compiler itself runs on a hosted machine, but generates code for freestanding targets.
#![cfg_attr(target_os = "none", no_std)]

// allocator for no_std environments
#[cfg(target_os = "none")]
extern crate alloc;

// Re-export alloc types for no_std compatibility
#[cfg(target_os = "none")]
pub use alloc::{
    boxed::Box, collections::HashMap, collections::HashSet, format, string::String,
    string::ToString, vec::Vec,
};

// On hosted targets, re-export from std for uniform API
#[cfg(not(target_os = "none"))]
pub use std::{
    boxed::Box, collections::HashMap, collections::HashSet, format, string::String,
    string::ToString, vec::Vec,
};

// Core modules
pub mod ast;
pub mod codegen;
// HIR pipeline modules (HIR is the single engine; no feature flag)
pub mod codegen_contract;
pub mod contract_metadata;
pub mod ffi;
pub mod hir;
pub mod layout;
pub mod lexer;
pub mod lod;
pub mod module_loader;
pub mod os;
pub mod package;
pub mod parser;
// Mixed-lifecycle compatibility module. Its AST Analyzer is
// LegacyReferenceOnly and is not canonical semantic authority.
pub mod semantic;
// Active canonical HIR semantic authority.
pub mod semantic_gate;
pub mod span;
// pub mod net;  // Module file not present
pub mod tier2;
pub mod types;
