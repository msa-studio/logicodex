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

pub mod capability_ir;
pub mod ctl_mapper;
pub mod gate;
pub mod metadata;
pub mod pass;
pub mod shard;
pub mod topology;

// Re-exports for convenience
pub use capability_ir::{
    CapabilityGraph, CapabilityRef, CompileTarget, IRDoorEdge, IRGateEdge, IRServiceNode,
    IRShardNode, IRVerifyResult, IRViolation,
};
pub use ctl_mapper::{
    get_wit_operations, map_and_generate_wit, map_and_generate_wit_with_overrides, CtlMapper,
    CtlMappingStats, WitDomain, WitOperation,
};
pub use gate::{GateContract, GateDomain, GateParseError, GateRef, GateType};
pub use metadata::{Capability, InlineCost, MemoryReport, MetadataGraph, SemanticSummary};
pub use pass::{
    compile_streaming, pass1_predeclare, pass2_streaming, CompileMode, StreamingResult,
};
pub use shard::{
    CommEdge, CommType, DoorRef, ServiceGraph, ServiceNode, ShardAssignment, ShardTopology,
    ShardVerifyResult, ShardViolation,
};
pub use topology::{
    diff_topology, CapabilityDiff, CapabilityTopology, TopologyVerifyResult, TopologyViolation,
};
