// =========================================================================
// Logicodex v1.32.0-alpha — Tier 2: Streaming Semantic Compiler + Capability Fabric
//
// Architecture:
//   Tier 1 (Parser)    → Full AST per function (temporary, discarded after Pass 2)
//   Tier 2 (This mod)  → MetadataGraph + SemanticSummary + CapabilityTopology
//   Tier 3 (Codegen)   → LLVM IR chunks (streamed, one function at a time)
//
// Flow:
//   Program → Pass 1 (pre-declare) → MetadataGraph + CapabilityTopology
//                                          ↓
//                Pass 2 (stream + verify gates) → Results + .cap file
// =========================================================================

pub mod metadata;
pub mod pass;
pub mod gate;
pub mod topology;

// Re-exports for convenience
pub use metadata::{
    Capability, InlineCost, MemoryReport, MetadataGraph, SemanticSummary,
};
pub use pass::{compile_streaming, pass1_predeclare, pass2_streaming, CompileMode, StreamingResult};
pub use gate::{GateRef, GateType, GateContract, GateDomain, GateParseError};
pub use topology::{CapabilityTopology, CapabilityDiff, TopologyVerifyResult, TopologyViolation, diff_topology};
