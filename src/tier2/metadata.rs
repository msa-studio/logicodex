// =========================================================================
// Logicodex v1.31.0-alpha — Tier 2: Persistent Metadata Graph
//
// The heart of the Streaming Semantic Compiler.
// Instead of keeping full AST in RAM, we extract only the semantic essence:
//   - What symbols exist
//   - What types they have
//   - What capabilities they require (Pure, IO, Unsafe, etc.)
//
// This enables the 2-Pass Engine:
//   Pass 1: Lightning scan → build MetadataGraph (lightweight)
//   Pass 2: Deep analysis → stream functions one-by-one, discard after codegen
// =========================================================================

use crate::ast::{Param, Stmt, Type};
use std::collections::HashMap;

// ─── Capability ───
// What effects a function/actor can have. Used for optimization
// and safety verification without keeping full AST.
// Manual bitflags on u8 (no external crate needed).

/// Capability flags — effects a symbol may have.
/// Compact representation (u8) for memory efficiency.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Capability(pub u8);

impl Capability {
    /// No side effects — safe to inline, reorder, cache
    pub const PURE: Capability = Capability(0x01);
    /// Performs I/O (file, network, print)
    pub const IO: Capability = Capability(0x02);
    /// Contains unsafe operations (FFI, raw pointers)
    pub const UNSAFE: Capability = Capability(0x04);
    /// Spawns actors or uses channels (concurrency)
    pub const CONCURRENT: Capability = Capability(0x08);
    /// Uses hardware zones (embedded-specific)
    pub const HARDWARE: Capability = Capability(0x10);
    /// May not terminate (loops, recursion)
    pub const DIVERGING: Capability = Capability(0x20);

    /// Check if this capability includes a specific flag.
    pub fn contains(&self, other: Capability) -> bool {
        (self.0 & other.0) == other.0
    }

    /// Insert (union) another capability.
    pub fn insert(&mut self, other: Capability) {
        self.0 |= other.0;
    }

    /// Check if this capability includes I/O.
    pub fn has_io(&self) -> bool {
        self.contains(Capability::IO)
    }

    /// Check if this capability includes unsafe operations.
    pub fn has_unsafe(&self) -> bool {
        self.contains(Capability::UNSAFE)
    }

    /// Check if this capability includes concurrency.
    pub fn has_concurrent(&self) -> bool {
        self.contains(Capability::CONCURRENT)
    }

    /// Merge two capabilities (union).
    pub fn merge(&mut self, other: Capability) {
        self.insert(other);
    }
}

impl Default for Capability {
    fn default() -> Self {
        // Default: assume pure until proven otherwise
        Capability::PURE
    }
}

impl std::fmt::Debug for Capability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut flags = Vec::new();
        if self.contains(Capability::PURE) { flags.push("PURE"); }
        if self.contains(Capability::IO) { flags.push("IO"); }
        if self.contains(Capability::UNSAFE) { flags.push("UNSAFE"); }
        if self.contains(Capability::CONCURRENT) { flags.push("CONCURRENT"); }
        if self.contains(Capability::HARDWARE) { flags.push("HARDWARE"); }
        if self.contains(Capability::DIVERGING) { flags.push("DIVERGING"); }
        if flags.is_empty() { flags.push("NONE"); }
        write!(f, "Capability({})", flags.join("|"))
    }
}

// ─── InlineCost ───
// Estimated cost for inlining decisions.
// Small functions (cost ≤ SMALL) are always inline candidates.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InlineCost {
    /// Trivial: single expression, always inline
    Trivial = 0,
    /// Small: ≤ 3 statements, inline candidate
    Small = 1,
    /// Medium: ≤ 10 statements, inline if hot
    Medium = 2,
    /// Large: > 10 statements, don't inline
    Large = 3,
    /// Recursive: never inline
    Recursive = 4,
}

impl InlineCost {
    pub fn from_statement_count(count: usize, is_recursive: bool) -> Self {
        if is_recursive {
            InlineCost::Recursive
        } else if count <= 1 {
            InlineCost::Trivial
        } else if count <= 3 {
            InlineCost::Small
        } else if count <= 10 {
            InlineCost::Medium
        } else {
            InlineCost::Large
        }
    }
}

// ─── SemanticSummary ───
// The compressed semantic essence of a single function or actor.
// This is all that remains in RAM after a function is compiled and dropped.
// Size: ~64 bytes vs. thousands of bytes for full AST.

#[derive(Debug, Clone)]
pub struct SemanticSummary {
    /// Unique symbol ID (assigned during Pass 1)
    pub symbol_id: u32,
    /// Symbol name (function name, actor name)
    pub name: String,
    /// Parameter types
    pub params: Vec<Type>,
    /// Return type (None for Unit)
    pub ret_type: Option<Type>,
    /// Effects/capabilities
    pub effects: Capability,
    /// Inline cost estimate
    pub inline_cost: InlineCost,
    /// Is this an actor (not a regular function)?
    pub is_actor: bool,
    /// For actors: what channels does it use?
    pub channels_used: Vec<String>,
    /// Is this symbol mutually recursive with others?
    pub is_recursive: bool,
    /// Symbols this function calls (for dependency tracking)
    pub callees: Vec<u32>,
}

impl SemanticSummary {
    /// Create a new summary for a function.
    pub fn new_function(
        symbol_id: u32,
        name: String,
        params: Vec<Type>,
        ret_type: Option<Type>,
    ) -> Self {
        Self {
            symbol_id,
            name,
            params,
            ret_type,
            effects: Capability::default(),
            inline_cost: InlineCost::Medium,
            is_actor: false,
            channels_used: Vec::new(),
            is_recursive: false,
            callees: Vec::new(),
        }
    }

    /// Create a new summary for an actor.
    pub fn new_actor(
        symbol_id: u32,
        name: String,
        channels_used: Vec<String>,
    ) -> Self {
        let mut effects = Capability::default();
        effects.insert(Capability::CONCURRENT);
        Self {
            symbol_id,
            name,
            params: Vec::new(),
            ret_type: None,
            effects,
            inline_cost: InlineCost::Large, // Actors are never inlined
            is_actor: true,
            channels_used,
            is_recursive: false,
            callees: Vec::new(),
        }
    }

    /// Size in bytes (for memory reporting).
    pub fn size_bytes(&self) -> usize {
        let base = std::mem::size_of::<Self>();
        let params_size = self.params.iter().map(|t| std::mem::size_of_val(t)).sum::<usize>();
        let channels_size = self.channels_used.iter().map(|s| s.len()).sum::<usize>();
        let callees_size = self.callees.len() * std::mem::size_of::<u32>();
        base + params_size + channels_size + callees_size
    }
}

// ─── MetadataGraph ───
// The Tier 2 persistent metadata store.
// Lives across both Pass 1 and Pass 2. Replaces keeping full AST.

#[derive(Debug, Default)]
pub struct MetadataGraph {
    /// All known symbols by ID
    summaries: HashMap<u32, SemanticSummary>,
    /// Name → symbol ID lookup
    name_to_id: HashMap<String, u32>,
    /// Next available symbol ID
    next_id: u32,
    /// Dependency graph: symbol_id → Vec<callee_id>
    call_graph: HashMap<u32, Vec<u32>>,
    /// Actor registry (same purpose as semantic.rs actor_registry)
    actor_names: Vec<String>,
    /// Channel topology: (from, to, message_type)
    channel_topology: Vec<(String, String, String)>,
}

impl MetadataGraph {
    pub fn new() -> Self {
        Self::default()
    }

    /// Allocate a new symbol ID.
    pub fn alloc_id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Register a symbol summary.
    pub fn register(&mut self, summary: SemanticSummary) {
        let id = summary.symbol_id;
        let name = summary.name.clone();
        self.summaries.insert(id, summary);
        self.name_to_id.insert(name.clone(), id);
    }

    /// Look up a symbol by name.
    pub fn lookup(&self, name: &str) -> Option<&SemanticSummary> {
        self.name_to_id.get(name).and_then(|id| self.summaries.get(id))
    }

    /// Look up a symbol by name (mutable).
    pub fn lookup_mut(&mut self, name: &str) -> Option<&mut SemanticSummary> {
        let id = self.name_to_id.get(name).copied()?;
        self.summaries.get_mut(&id)
    }

    /// Look up by ID.
    pub fn get_by_id(&self, id: u32) -> Option<&SemanticSummary> {
        self.summaries.get(&id)
    }

    /// Register a function call dependency.
    pub fn add_call(&mut self, caller_id: u32, callee_id: u32) {
        self.call_graph
            .entry(caller_id)
            .or_default()
            .push(callee_id);
    }

    /// Register an actor name.
    pub fn register_actor(&mut self, name: String) {
        if !self.actor_names.contains(&name) {
            self.actor_names.push(name.clone());
        }
    }

    /// Register a channel topology entry.
    pub fn register_channel(&mut self, from: String, to: String, message_type: String) {
        self.channel_topology.push((from, to, message_type));
    }

    /// Check if a name is a known actor.
    pub fn is_actor(&self, name: &str) -> bool {
        self.actor_names.contains(&name.to_string())
    }

    /// Get all actor names.
    pub fn actor_names(&self) -> &[String] {
        &self.actor_names
    }

    /// Get channel topology.
    pub fn channel_topology(&self) -> &[(String, String, String)] {
        &self.channel_topology
    }

    /// Detect mutual recursion between two symbols.
    pub fn is_mutually_recursive(&self, a: u32, b: u32) -> bool {
        let a_calls_b = self.call_graph.get(&a).map_or(false, |v| v.contains(&b));
        let b_calls_a = self.call_graph.get(&b).map_or(false, |v| v.contains(&a));
        a_calls_b && b_calls_a
    }

    /// Total memory used by this graph (bytes).
    pub fn total_memory_bytes(&self) -> usize {
        let summaries_size: usize = self.summaries.values().map(|s| s.size_bytes()).sum();
        let graph_size: usize = self
            .call_graph
            .values()
            .map(|v| v.len() * std::mem::size_of::<u32>())
            .sum();
        let actors_size: usize = self.actor_names.iter().map(|s| s.len()).sum();
        let channels_size: usize = self.channel_topology.iter().map(|(a, b, c)| a.len() + b.len() + c.len()).sum();
        std::mem::size_of::<Self>() + summaries_size + graph_size + actors_size + channels_size
    }

    /// Number of symbols in the graph.
    pub fn symbol_count(&self) -> usize {
        self.summaries.len()
    }

    /// Check if a symbol name is known (for forward reference resolution).
    pub fn has_symbol(&self, name: &str) -> bool {
        self.name_to_id.contains_key(name)
    }

    /// Resolve a callee name to its symbol ID (for Pass 2).
    pub fn resolve_callee(&self, name: &str) -> Option<u32> {
        self.name_to_id.get(name).copied()
    }
}

// ─── MemoryReport ───
// For comparing memory usage between traditional AST and streaming approach.

#[derive(Debug)]
pub struct MemoryReport {
    pub metadata_graph_bytes: usize,
    pub symbol_count: usize,
    pub actor_count: usize,
    pub channel_count: usize,
    pub estimated_ast_bytes: usize, // Rough estimate for comparison
}

impl MemoryReport {
    pub fn from_graph(graph: &MetadataGraph, estimated_ast_bytes: usize) -> Self {
        Self {
            metadata_graph_bytes: graph.total_memory_bytes(),
            symbol_count: graph.symbol_count(),
            actor_count: graph.actor_names.len(),
            channel_count: graph.channel_topology.len(),
            estimated_ast_bytes,
        }
    }

    /// Memory saved compared to keeping full AST.
    pub fn memory_saved_bytes(&self) -> usize {
        self.estimated_ast_bytes.saturating_sub(self.metadata_graph_bytes)
    }

    /// Compression ratio (lower is better).
    pub fn compression_ratio(&self) -> f64 {
        if self.estimated_ast_bytes == 0 {
            1.0
        } else {
            self.metadata_graph_bytes as f64 / self.estimated_ast_bytes as f64
        }
    }
}
