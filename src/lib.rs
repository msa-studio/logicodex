// =========================================================================
// Logicodex Language Engine — Library Target
// v1.44: Freestanding Compiler Support
//
// This library target exists primarily for integration tests in tests/.
// The binary target (src/main.rs) is the primary entry point for users.
//
// For freestanding targets, this crate is no_std with alloc.
// For hosted targets, it uses the full standard library.
// =========================================================================

// v1.44 G6: no_std support for freestanding targets
// When building for bare metal, we use only core + alloc (no std::fs, std::process, etc.)
// The compiler itself runs on a hosted machine, but generates code for freestanding targets.
#![cfg_attr(target_os = "none", no_std)]

// v1.44 G4: allocator for no_std environments
#[cfg(target_os = "none")]
extern crate alloc;

// v1.44 G6: Re-export alloc types for no_std compatibility
#[cfg(target_os = "none")]
pub use alloc::{
    string::String, string::ToString, vec::Vec, boxed::Box,
    collections::HashMap, collections::HashSet, format,
};

// v1.44 G6: On hosted targets, re-export from std for uniform API
#[cfg(not(target_os = "none"))]
pub use std::{
    string::String, string::ToString, vec::Vec, boxed::Box,
    collections::HashMap, collections::HashSet, format,
};

// Core v1.21 modules (always compiled)
pub mod ast;
pub mod codegen;
// v1.30 Option Engine modules (gated behind feature flag)
#[cfg(feature = "v1_30")]
pub mod codegen_contract;
#[cfg(feature = "v1_30")]
pub mod ffi;
#[cfg(feature = "v1_30")]
pub mod hir;
#[cfg(feature = "v1_30")]
pub mod layout;
pub mod lexer;
pub mod os;
pub mod parser;
pub mod semantic;
#[cfg(feature = "v1_30")]
pub mod semantic_gate;
pub mod span;
// pub mod net;  // Module file not present
#[cfg(feature = "v1_30")]
pub mod tier2;
pub mod types;
